//! Command implementations over the [`Store`].
//!
//! Convention (per the slice constraints): query **results are data → stdout**
//! (`println!`), while mutation confirmations, dry-run notices, and errors are
//! **diagnostics → stderr** (`eprintln!`). Output stays plain so it is
//! TTY-agnostic and stable under `assert_cmd`.

use std::path::Path;

use anyhow::{Context as _, anyhow, bail};
use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter, SupersedeKind, Supersedes};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use serde::Serialize;
use tabled::{Table, Tabled, settings::Style};

use crate::context::Context;

/// Which context slot `use` sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseKind {
    /// The current project.
    Project,
    /// The current arc.
    Arc,
}

impl UseKind {
    fn node_type(self) -> NodeType {
        match self {
            UseKind::Project => NodeType::Project,
            UseKind::Arc => NodeType::Arc,
        }
    }

    fn label(self) -> &'static str {
        match self {
            UseKind::Project => "project",
            UseKind::Arc => "arc",
        }
    }
}

/// Today's date (UTC) — used to stamp `updated` and retirement.
fn today() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

// ---------------------------------------------------------------------------
// Reference resolution: id | number | unique name prefix
// ---------------------------------------------------------------------------

/// Resolves a user-supplied reference to a node, accepting a full ULID, a human
/// number, or a unique (case-insensitive) name prefix.
///
/// # Errors
///
/// Errors if no node matches, or if a name prefix is ambiguous.
fn resolve(store: &Store, reference: &str) -> anyhow::Result<Document> {
    // 1. A full ULID id.
    if let Ok(id) = reference.parse::<Id>() {
        return store
            .load(id)
            .with_context(|| format!("no node with id {reference} (try `odm list`)"));
    }

    let all = store.load_all()?;

    // 2. A human number.
    if let Ok(number) = reference.parse::<u32>() {
        let mut hits = all.into_iter().filter(|d| d.frontmatter().number() == number);
        return match hits.next() {
            Some(doc) => Ok(doc),
            None => Err(anyhow!("no node with number {number} (try `odm list`)")),
        };
    }

    // 3. A unique name prefix (case-insensitive).
    let needle = reference.to_lowercase();
    let mut matches: Vec<Document> = all
        .into_iter()
        .filter(|d| d.frontmatter().name().to_lowercase().starts_with(&needle))
        .collect();
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(anyhow!("no node matching {reference:?} (try `odm list`)")),
        n => {
            let names: Vec<String> = matches
                .iter()
                .map(|d| format!("#{} {}", d.frontmatter().number(), d.frontmatter().name()))
                .collect();
            Err(anyhow!("{reference:?} is ambiguous ({n} matches): {}", names.join(", ")))
        }
    }
}

// ---------------------------------------------------------------------------
// JSON views — stable, documented schemas for `--json`
// ---------------------------------------------------------------------------

/// JSON shape of a node (stable schema for `list`/`show --json`).
#[derive(Serialize)]
struct NodeJson {
    id: String,
    number: u32,
    #[serde(rename = "type")]
    node_type: String,
    name: String,
    origin: String,
    reserved: bool,
    tags: Vec<String>,
    component: Option<String>,
    retired: Option<RetiredJson>,
    part_of: Option<String>,
    supersedes: Option<SupersedesJson>,
}

#[derive(Serialize)]
struct RetiredJson {
    reason: String,
    on: String,
}

#[derive(Serialize)]
struct SupersedesJson {
    node: String,
    kind: String,
}

impl NodeJson {
    fn from(doc: &Document) -> Self {
        let fm = doc.frontmatter();
        let edges = fm.edges();
        Self {
            id: fm.id().to_string(),
            number: fm.number(),
            node_type: fm.node_type().as_str().to_string(),
            name: fm.name().to_string(),
            origin: fm.origin().as_str().to_string(),
            reserved: fm.reserved(),
            tags: fm.tags().to_vec(),
            component: fm.component().map(str::to_string),
            retired: fm
                .retired()
                .map(|r| RetiredJson { reason: r.reason.clone(), on: r.on.to_string() }),
            part_of: edges.part_of.map(|id| id.to_string()),
            supersedes: edges.supersedes.as_ref().map(|s| SupersedesJson {
                node: s.node.to_string(),
                kind: supersede_kind_str(s.kind).to_string(),
            }),
        }
    }
}

fn supersede_kind_str(kind: SupersedeKind) -> &'static str {
    match kind {
        SupersedeKind::Obsoletes => "obsoletes",
        SupersedeKind::Updates => "updates",
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// `new <type> <name>` — idempotent describe-or-create.
pub fn new(store: &Store, node_type: &str, name: &str, dry_run: bool) -> anyhow::Result<()> {
    let node_type: NodeType = node_type.parse().map_err(|_| {
        anyhow!("unknown type {node_type:?}; expected one of project|arc|slice|odd|adr|note")
    })?;

    let all = store.load_all()?;

    // Idempotent: a node of the same type and exact name already exists.
    if let Some(existing) = all
        .iter()
        .find(|d| d.frontmatter().node_type() == node_type && d.frontmatter().name() == name)
    {
        let fm = existing.frontmatter();
        eprintln!("exists: {} #{} {:?} ({})", node_type.as_str(), fm.number(), name, fm.id());
        return Ok(());
    }

    let next_number = all.iter().map(|d| d.frontmatter().number()).max().map_or(1, |m| m + 1);
    let id = Id::new();
    let created = id.created_at().date_naive();
    let fm = Frontmatter::new(id, next_number, node_type, name, created, created, Origin::Planned);
    let doc = Document::new(fm, format!("# {name}\n"));

    if dry_run {
        eprintln!("would create {} #{next_number} {name:?} ({id})", node_type.as_str());
        return Ok(());
    }

    store.persist(&doc)?;
    eprintln!("created {} #{next_number} {name:?} ({id})", node_type.as_str());
    Ok(())
}

/// A row in the `list` table.
#[derive(Tabled)]
struct ListRow {
    #[tabled(rename = "NUMBER")]
    number: u32,
    #[tabled(rename = "TYPE")]
    node_type: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "ID")]
    id: String,
}

/// `list` — full scan with optional type/tag/component filters.
pub fn list(
    store: &Store,
    type_filter: Option<&str>,
    tag: Option<&str>,
    component: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let type_filter = type_filter
        .map(|t| t.parse::<NodeType>().map_err(|_| anyhow!("unknown type {t:?}")))
        .transpose()?;

    let mut nodes = store.load_all()?;
    nodes.retain(|d| {
        let fm = d.frontmatter();
        type_filter.is_none_or(|t| fm.node_type() == t)
            && tag.is_none_or(|t| fm.tags().iter().any(|x| x == t))
            && component.is_none_or(|c| fm.component() == Some(c))
    });
    nodes.sort_by_key(|d| d.frontmatter().number());

    if json {
        let view: Vec<NodeJson> = nodes.iter().map(NodeJson::from).collect();
        println!("{}", serde_json::to_string_pretty(&view)?);
        return Ok(());
    }

    if nodes.is_empty() {
        println!("(no nodes)");
        return Ok(());
    }
    let rows: Vec<ListRow> = nodes
        .iter()
        .map(|d| {
            let fm = d.frontmatter();
            ListRow {
                number: fm.number(),
                node_type: fm.node_type().as_str().to_string(),
                name: fm.name().to_string(),
                id: fm.id().to_string(),
            }
        })
        .collect();
    println!("{}", Table::new(rows).with(Style::sharp()));
    Ok(())
}

/// `show X` — node + edges + way-finding (parent and children) in one call.
pub fn show(store: &Store, reference: &str, json: bool) -> anyhow::Result<()> {
    let doc = resolve(store, reference)?;
    let id = doc.frontmatter().id();
    let all = store.load_all()?;
    let children: Vec<&Document> =
        all.iter().filter(|d| d.frontmatter().edges().part_of == Some(id)).collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&NodeJson::from(&doc))?);
        return Ok(());
    }

    let fm = doc.frontmatter();
    println!("{} #{} {}", fm.node_type().as_str(), fm.number(), fm.name());
    println!("  id:        {id}");
    println!("  origin:    {}", fm.origin().as_str());
    println!("  created:   {}", fm.created());
    println!("  updated:   {}", fm.updated());
    if !fm.tags().is_empty() {
        println!("  tags:      {}", fm.tags().join(", "));
    }
    if let Some(component) = fm.component() {
        println!("  component: {component}");
    }
    if let Some(retired) = fm.retired() {
        println!("  retired:   {} ({})", retired.reason, retired.on);
    }
    let edges = fm.edges();
    if let Some(parent) = edges.part_of {
        println!("  part_of:   {parent}");
    }
    if let Some(s) = &edges.supersedes {
        println!("  supersedes: {} ({})", s.node, supersede_kind_str(s.kind));
    }
    // Way-finding: children in the containment tree.
    if children.is_empty() {
        println!("  children:  (none)");
    } else {
        println!("  children:");
        for child in children {
            let c = child.frontmatter();
            println!("    - {} #{} {}", c.node_type().as_str(), c.number(), c.name());
        }
    }
    Ok(())
}

/// `rename X <new-name>` — changes the name only; id and on-disk path are
/// unchanged (the path is a pure function of the immutable id).
pub fn rename(store: &Store, reference: &str, new_name: &str, dry_run: bool) -> anyhow::Result<()> {
    let mut doc = resolve(store, reference)?;
    let fm = doc.frontmatter();
    let (id, number, old_name) = (fm.id(), fm.number(), fm.name().to_string());

    if dry_run {
        eprintln!("would rename #{number} {old_name:?} -> {new_name:?} ({id})");
        return Ok(());
    }

    doc.frontmatter_mut().set_name(new_name);
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?; // same id => same path, file is rewritten in place
    eprintln!("renamed #{number} {old_name:?} -> {new_name:?} ({id})");
    Ok(())
}

/// `retire X --because <reason>` — marks the node withdrawn. The file is
/// preserved (git keeps history); this is never a destructive delete.
pub fn retire(store: &Store, reference: &str, reason: &str, dry_run: bool) -> anyhow::Result<()> {
    let mut doc = resolve(store, reference)?;
    let fm = doc.frontmatter();
    let (id, number, name) = (fm.id(), fm.number(), fm.name().to_string());

    if dry_run {
        eprintln!("would retire #{number} {name:?} ({id}): {reason}");
        return Ok(());
    }

    doc.frontmatter_mut().retire(reason, today());
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?; // overwrites in place — file kept, not deleted
    eprintln!("retired #{number} {name:?} ({id}): {reason}");
    Ok(())
}

/// `supersede X --with Y --kind <kind>` — records that Y supersedes X. The
/// lineage edge is stored on Y (the newer node), pointing at X.
pub fn supersede(
    store: &Store,
    old_ref: &str,
    with_ref: &str,
    kind: SupersedeKind,
    dry_run: bool,
) -> anyhow::Result<()> {
    let old = resolve(store, old_ref)?;
    let mut new_doc = resolve(store, with_ref)?;
    let old_id = old.frontmatter().id();
    let (old_number, new_number) = (old.frontmatter().number(), new_doc.frontmatter().number());

    if old_id == new_doc.frontmatter().id() {
        bail!("a node cannot supersede itself");
    }

    if dry_run {
        eprintln!(
            "would record #{new_number} supersedes #{old_number} ({})",
            supersede_kind_str(kind)
        );
        return Ok(());
    }

    new_doc.frontmatter_mut().edges_mut().supersedes = Some(Supersedes { node: old_id, kind });
    new_doc.frontmatter_mut().set_updated(today());
    store.persist(&new_doc)?;
    eprintln!("recorded: #{new_number} supersedes #{old_number} ({})", supersede_kind_str(kind));
    Ok(())
}

/// `use [project|arc] X` — sets the current context slot to node X.
pub fn use_context(
    store: &Store,
    root: &Path,
    kind: UseKind,
    reference: &str,
) -> anyhow::Result<()> {
    let doc = resolve(store, reference)?;
    let fm = doc.frontmatter();
    if fm.node_type() != kind.node_type() {
        bail!(
            "{reference:?} is a {}, not a {} (use `odm use {} <a {}>`)",
            fm.node_type().as_str(),
            kind.label(),
            kind.label(),
            kind.label()
        );
    }
    let mut ctx = Context::load(root)?;
    match kind {
        UseKind::Project => ctx.project = Some(fm.id()),
        UseKind::Arc => ctx.arc = Some(fm.id()),
    }
    ctx.save(root)?;
    eprintln!("context: {} = {} ({})", kind.label(), fm.name(), fm.id());
    Ok(())
}

/// `context` — shows the current project/arc selection.
pub fn context(store: &Store, root: &Path, json: bool) -> anyhow::Result<()> {
    let ctx = Context::load(root)?;
    let project = ctx.project.and_then(|id| store.load(id).ok());
    let arc = ctx.arc.and_then(|id| store.load(id).ok());

    if json {
        let view = serde_json::json!({
            "project": project.as_ref().map(NodeJson::from),
            "arc": arc.as_ref().map(NodeJson::from),
        });
        println!("{}", serde_json::to_string_pretty(&view)?);
        return Ok(());
    }

    match &project {
        Some(d) => println!(
            "project: #{} {} ({})",
            d.frontmatter().number(),
            d.frontmatter().name(),
            d.frontmatter().id()
        ),
        None => println!("project: (none)"),
    }
    match &arc {
        Some(d) => println!(
            "arc:     #{} {} ({})",
            d.frontmatter().number(),
            d.frontmatter().name(),
            d.frontmatter().id()
        ),
        None => println!("arc:     (none)"),
    }
    Ok(())
}
