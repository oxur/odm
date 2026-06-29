//! Command implementations over the [`Store`].
//!
//! Output is dependency-injected: query **results (data) are written to `out`**
//! while mutation confirmations and dry-run notices (**diagnostics**) are
//! written to `err`. `run` wires these to stdout/stderr; tests wire them to
//! buffers and drive commands in-process. Output stays plain so it is
//! TTY-agnostic and stable to assert on.

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use anyhow::{Context as _, anyhow, bail};
use chrono::NaiveDate;
use odm_core::check::{Finding, Violation};
use odm_core::frontmatter::{
    Dependency, Document, Frontmatter, SupersedeKind, Supersedes, TornEdge,
};
use odm_core::gates::GateSets;
use odm_core::graph::{Block, NodeGraph, Tear};
use odm_core::recompose::{self, Issue};
use odm_core::satisfaction::{Satisfaction, staleness_on_advance, threshold_from_toml};
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use serde::Serialize;
use tabled::{Table, Tabled, settings::Style};

use crate::context::Context;

/// Exit code: the command succeeded (and, for `check`, the corpus is clean).
pub const EXIT_OK: u8 = 0;
/// Exit code: `check` ran and found violations.
pub const EXIT_VIOLATIONS: u8 = 1;

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

/// `new <type> <name> [--parent <ref>]` — idempotent describe-or-create.
/// Confirmations go to `err` (diagnostics).
pub fn new(
    store: &Store,
    node_type: &str,
    name: &str,
    parent: Option<&str>,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let node_type: NodeType = node_type.parse().map_err(|_| {
        anyhow!("unknown type {node_type:?}; expected one of project|arc|slice|odd|adr|note")
    })?;

    // Resolve the parent (if any) up front, so an unresolvable ref fails before
    // anything is written.
    let parent_id = parent.map(|p| resolve(store, p)).transpose()?.map(|d| d.frontmatter().id());

    let all = store.load_all()?;

    // Idempotent: a node of the same type and exact name already exists.
    if let Some(existing) = all
        .iter()
        .find(|d| d.frontmatter().node_type() == node_type && d.frontmatter().name() == name)
    {
        let fm = existing.frontmatter();
        writeln!(err, "exists: {} #{} {:?} ({})", node_type.as_str(), fm.number(), name, fm.id())?;
        return Ok(());
    }

    let next_number = all.iter().map(|d| d.frontmatter().number()).max().map_or(1, |m| m + 1);
    let id = Id::new();
    let created = id.created_at().date_naive();
    let mut fm =
        Frontmatter::new(id, next_number, node_type, name, created, created, Origin::Planned);
    if let Some(parent_id) = parent_id {
        fm.edges_mut().part_of = Some(parent_id);
    }
    let doc = Document::new(fm, format!("# {name}\n"));

    let parent_note = parent_id.map(|p| format!(" (part_of {p})")).unwrap_or_default();
    if dry_run {
        writeln!(
            err,
            "would create {} #{next_number} {name:?} ({id}){parent_note}",
            node_type.as_str()
        )?;
        return Ok(());
    }

    store.persist(&doc)?;
    writeln!(err, "created {} #{next_number} {name:?} ({id}){parent_note}", node_type.as_str())?;
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

/// `list` — list nodes with optional type/tag/component filters. Data → `out`.
///
/// The human table is **index-backed** (slice04): it `reconcile`s the `.odm/`
/// index (freshening it against any edit) and renders from the index records —
/// no full corpus parse. `--json` stays a full-node dump over `load_all`: it
/// emits fields the index deliberately does not carry (`origin`/`reserved`/
/// `retired`), since the index is the filter/sort accelerator, not a full-node
/// store (ODD-0014 §3.5).
pub fn list(
    store: &Store,
    type_filter: Option<&str>,
    tag: Option<&str>,
    component: Option<&str>,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let type_filter = type_filter
        .map(|t| t.parse::<NodeType>().map_err(|_| anyhow!("unknown type {t:?}")))
        .transpose()?;

    if json {
        // Full-node serialization stays load_all-backed (see the doc comment).
        let mut nodes = store.load_all()?;
        nodes.retain(|d| {
            let fm = d.frontmatter();
            type_filter.is_none_or(|t| fm.node_type() == t)
                && tag.is_none_or(|t| fm.tags().iter().any(|x| x == t))
                && component.is_none_or(|c| fm.component() == Some(c))
        });
        nodes.sort_by_key(|d| d.frontmatter().number());
        let view: Vec<NodeJson> = nodes.iter().map(NodeJson::from).collect();
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    // Human table: reconcile-then-read the index (slice03 finding #2 / I-9).
    let index = odm_index::default_index_path(store.root());
    let snapshot = odm_index::reconcile(store, &index)?.snapshot;
    let mut records: Vec<&odm_index::IndexRecord> = snapshot
        .records
        .iter()
        .filter(|r| {
            type_filter.is_none_or(|t| r.node_type == t)
                && tag.is_none_or(|t| r.tags.iter().any(|x| x == t))
                && component.is_none_or(|c| r.component.as_deref() == Some(c))
        })
        .collect();
    records.sort_by_key(|r| r.number);

    if records.is_empty() {
        writeln!(out, "(no nodes)")?;
        return Ok(());
    }
    let rows: Vec<ListRow> = records
        .iter()
        .map(|r| ListRow {
            number: r.number,
            node_type: r.node_type.as_str().to_string(),
            name: r.title.clone(),
            id: r.id.to_string(),
        })
        .collect();
    writeln!(out, "{}", Table::new(rows).with(Style::sharp()))?;
    Ok(())
}

/// `show X` — node + edges + way-finding (parent and children). Data → `out`.
pub fn show(store: &Store, reference: &str, json: bool, out: &mut dyn Write) -> anyhow::Result<()> {
    let doc = resolve(store, reference)?;
    let id = doc.frontmatter().id();
    let all = store.load_all()?;
    let children: Vec<&Document> =
        all.iter().filter(|d| d.frontmatter().edges().part_of == Some(id)).collect();

    if json {
        writeln!(out, "{}", serde_json::to_string_pretty(&NodeJson::from(&doc))?)?;
        return Ok(());
    }

    let fm = doc.frontmatter();
    writeln!(out, "{} #{} {}", fm.node_type().as_str(), fm.number(), fm.name())?;
    writeln!(out, "  id:        {id}")?;
    writeln!(out, "  origin:    {}", fm.origin().as_str())?;
    writeln!(out, "  created:   {}", fm.created())?;
    writeln!(out, "  updated:   {}", fm.updated())?;
    if !fm.tags().is_empty() {
        writeln!(out, "  tags:      {}", fm.tags().join(", "))?;
    }
    if let Some(component) = fm.component() {
        writeln!(out, "  component: {component}")?;
    }
    if let Some(retired) = fm.retired() {
        writeln!(out, "  retired:   {} ({})", retired.reason, retired.on)?;
    }
    let edges = fm.edges();
    if let Some(parent) = edges.part_of {
        writeln!(out, "  part_of:   {parent}")?;
    }
    if let Some(s) = &edges.supersedes {
        writeln!(out, "  supersedes: {} ({})", s.node, supersede_kind_str(s.kind))?;
    }
    // Way-finding: children in the containment tree.
    if children.is_empty() {
        writeln!(out, "  children:  (none)")?;
    } else {
        writeln!(out, "  children:")?;
        for child in children {
            let c = child.frontmatter();
            writeln!(out, "    - {} #{} {}", c.node_type().as_str(), c.number(), c.name())?;
        }
    }
    Ok(())
}

/// `rename X <new-name>` — changes the name only; id and on-disk path are
/// unchanged (the path is a pure function of the immutable id).
pub fn rename(
    store: &Store,
    reference: &str,
    new_name: &str,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut doc = resolve(store, reference)?;
    let fm = doc.frontmatter();
    let (id, number, old_name) = (fm.id(), fm.number(), fm.name().to_string());

    if dry_run {
        writeln!(err, "would rename #{number} {old_name:?} -> {new_name:?} ({id})")?;
        return Ok(());
    }

    doc.frontmatter_mut().set_name(new_name);
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?; // same id => same path, file is rewritten in place
    writeln!(err, "renamed #{number} {old_name:?} -> {new_name:?} ({id})")?;
    Ok(())
}

/// `retire X --because <reason>` — marks the node withdrawn. The file is
/// preserved (git keeps history); this is never a destructive delete.
pub fn retire(
    store: &Store,
    reference: &str,
    reason: &str,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut doc = resolve(store, reference)?;
    let fm = doc.frontmatter();
    let (id, number, name) = (fm.id(), fm.number(), fm.name().to_string());

    if dry_run {
        writeln!(err, "would retire #{number} {name:?} ({id}): {reason}")?;
        return Ok(());
    }

    doc.frontmatter_mut().retire(reason, today());
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?; // overwrites in place — file kept, not deleted
    writeln!(err, "retired #{number} {name:?} ({id}): {reason}")?;
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
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let old = resolve(store, old_ref)?;
    let mut new_doc = resolve(store, with_ref)?;
    let old_id = old.frontmatter().id();
    let (old_number, new_number) = (old.frontmatter().number(), new_doc.frontmatter().number());

    if old_id == new_doc.frontmatter().id() {
        bail!("a node cannot supersede itself");
    }

    if dry_run {
        writeln!(
            err,
            "would record #{new_number} supersedes #{old_number} ({})",
            supersede_kind_str(kind)
        )?;
        return Ok(());
    }

    new_doc.frontmatter_mut().edges_mut().supersedes = Some(Supersedes { node: old_id, kind });
    new_doc.frontmatter_mut().set_updated(today());
    store.persist(&new_doc)?;
    writeln!(
        err,
        "recorded: #{new_number} supersedes #{old_number} ({})",
        supersede_kind_str(kind)
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// graph mutators: link / unlink / set-gate / tear (ODD-0013 §3, §4.3, §5.1)
//
// These wire the existing odm-core ops (`edges_mut`, `Status::set_gate`,
// `Tear::new`) to the CLI and persist atomically via odm-store. Edges are
// stored on the **source**; reverse edges stay derived (never written).
// ---------------------------------------------------------------------------

/// The edge kind `link`/`unlink` operates on (source-stored edges only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkEdge {
    /// Ordering dependency.
    DependsOn,
    /// Hard external block.
    BlockedBy,
    /// Consumes a concrete output.
    Consumes,
    /// Verifies the target.
    Verifies,
    /// Affects the target's docs.
    Affects,
    /// Containment parent (single-parent).
    PartOf,
}

impl LinkEdge {
    fn as_str(self) -> &'static str {
        match self {
            LinkEdge::DependsOn => "depends_on",
            LinkEdge::BlockedBy => "blocked_by",
            LinkEdge::Consumes => "consumes",
            LinkEdge::Verifies => "verifies",
            LinkEdge::Affects => "affects",
            LinkEdge::PartOf => "part_of",
        }
    }
}

/// `link X <edge> Y` — adds the edge on the source X. `depends_on` may carry a
/// `--satisfied-at <gate>`. `part_of` enforces a single parent (it replaces any
/// existing parent rather than appending). Re-linking is idempotent.
pub fn link(
    store: &Store,
    source_ref: &str,
    edge: LinkEdge,
    target_ref: &str,
    satisfied_at: Option<&str>,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut src = resolve(store, source_ref)?;
    let target_id = resolve(store, target_ref)?.frontmatter().id();
    let src_id = src.frontmatter().id();
    let (number, name) = (src.frontmatter().number(), src.frontmatter().name().to_string());

    if src_id == target_id {
        bail!(
            "a node cannot {} itself; pick a different target",
            match edge {
                LinkEdge::PartOf => "be `part_of`",
                _ => "link to",
            }
        );
    }
    if satisfied_at.is_some() && edge != LinkEdge::DependsOn {
        bail!("`--satisfied-at` applies only to `depends_on` (got `{}`)", edge.as_str());
    }

    if dry_run {
        writeln!(err, "would link #{number} {name:?} {} {target_id}", edge.as_str())?;
        return Ok(());
    }

    let edges = src.frontmatter_mut().edges_mut();
    match edge {
        LinkEdge::DependsOn => {
            // Replace any existing dependency on the same target (so re-linking
            // can update `satisfied_at`), then add the new one.
            edges.depends_on.retain(|d| dependency_target(d) != target_id);
            let dep = match satisfied_at {
                Some(gate) => {
                    Dependency::Qualified { node: target_id, satisfied_at: gate.to_string() }
                }
                None => Dependency::Bare(target_id),
            };
            edges.depends_on.push(dep);
        }
        LinkEdge::BlockedBy => push_unique(&mut edges.blocked_by, target_id),
        LinkEdge::Consumes => push_unique(&mut edges.consumes, target_id),
        LinkEdge::Verifies => push_unique(&mut edges.verifies, target_id),
        LinkEdge::Affects => push_unique(&mut edges.affects, target_id),
        LinkEdge::PartOf => edges.part_of = Some(target_id), // single-parent: replace
    }
    src.frontmatter_mut().set_updated(today());
    store.persist(&src)?;
    writeln!(err, "linked #{number} {name:?} {} {target_id}", edge.as_str())?;
    Ok(())
}

/// Pushes `id` onto `v` only if absent (idempotent edge add).
fn push_unique(v: &mut Vec<Id>, id: Id) {
    if !v.contains(&id) {
        v.push(id);
    }
}

/// `unlink X <edge> Y` — removes the edge from X. Removing an absent edge is a
/// clear no-op (reported, not an error).
pub fn unlink(
    store: &Store,
    source_ref: &str,
    edge: LinkEdge,
    target_ref: &str,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut src = resolve(store, source_ref)?;
    let target_id = resolve(store, target_ref)?.frontmatter().id();
    let (number, name) = (src.frontmatter().number(), src.frontmatter().name().to_string());

    let present = {
        let edges = src.frontmatter().edges();
        match edge {
            LinkEdge::DependsOn => {
                edges.depends_on.iter().any(|d| dependency_target(d) == target_id)
            }
            LinkEdge::BlockedBy => edges.blocked_by.contains(&target_id),
            LinkEdge::Consumes => edges.consumes.contains(&target_id),
            LinkEdge::Verifies => edges.verifies.contains(&target_id),
            LinkEdge::Affects => edges.affects.contains(&target_id),
            LinkEdge::PartOf => edges.part_of == Some(target_id),
        }
    };
    if !present {
        writeln!(err, "no-op: #{number} {name:?} has no `{}` edge to {target_id}", edge.as_str())?;
        return Ok(());
    }

    if dry_run {
        writeln!(err, "would unlink #{number} {name:?} {} {target_id}", edge.as_str())?;
        return Ok(());
    }

    let edges = src.frontmatter_mut().edges_mut();
    match edge {
        LinkEdge::DependsOn => edges.depends_on.retain(|d| dependency_target(d) != target_id),
        LinkEdge::BlockedBy => edges.blocked_by.retain(|&id| id != target_id),
        LinkEdge::Consumes => edges.consumes.retain(|&id| id != target_id),
        LinkEdge::Verifies => edges.verifies.retain(|&id| id != target_id),
        LinkEdge::Affects => edges.affects.retain(|&id| id != target_id),
        LinkEdge::PartOf => edges.part_of = None,
    }
    src.frontmatter_mut().set_updated(today());
    store.persist(&src)?;
    writeln!(err, "unlinked #{number} {name:?} {} {target_id}", edge.as_str())?;
    Ok(())
}

/// The details recorded by [`set_gate`]: the gate name, who recorded it, and at
/// what evidence level.
pub struct GateReach<'a> {
    /// The gate name (must be in the node type's gate-set).
    pub gate: &'a str,
    /// Who recorded reaching it, if known.
    pub by: Option<String>,
    /// The evidence level.
    pub evidence: Evidence,
}

/// `set-gate X <gate> [--by] [--evidence]` — records a reached gate via
/// [`odm_core::status::Status::set_gate`], validating it against the node type's
/// gate-set. Records the slice05.1 per-level first-reach automatically.
pub fn set_gate(
    store: &Store,
    root: &Path,
    reference: &str,
    reach: GateReach<'_>,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let GateReach { gate, by, evidence } = reach;
    let mut doc = resolve(store, reference)?;
    let node_type = doc.frontmatter().node_type();
    let (number, name) = (doc.frontmatter().number(), doc.frontmatter().name().to_string());

    let (gates, _threshold) = load_gate_config(root)?;
    let gate_set = gates.for_type(node_type).ok_or_else(|| {
        anyhow!(
            "no gate-set for type `{}`; add a `[gates.{}]` sequence to odm.toml",
            node_type.as_str(),
            node_type.as_str()
        )
    })?;

    if dry_run {
        writeln!(err, "would set gate {gate:?}={} on #{number} {name:?}", evidence.as_str())?;
        return Ok(());
    }

    doc.frontmatter_mut()
        .status_mut()
        .set_gate(gate_set, gate, by, evidence, today())
        .map_err(|e| {
            anyhow!(
                "unknown gate {:?} for type `{}`; allowed: {}. Run `odm set-gate {reference} <one-of-those>`",
                e.gate,
                node_type.as_str(),
                e.allowed.join(", ")
            )
        })?;
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?;
    writeln!(err, "set gate {gate:?}={} on #{number} {name:?}", evidence.as_str())?;
    Ok(())
}

/// `tear X depends_on Y --because <r>` — declares a deliberately-assumed
/// dependency edge. The rationale is validated via [`Tear::new`] (empty →
/// rejected) and **persisted** as a [`TornEdge`] (`{ edge, because }`) in
/// `edges.tears` on X, so it survives to `check`'s active-tears listing
/// (ODD-0013 §4.3). A re-tear of the same target refreshes the rationale.
pub fn tear(
    store: &Store,
    source_ref: &str,
    target_ref: &str,
    because: &str,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut src = resolve(store, source_ref)?;
    let target_id = resolve(store, target_ref)?.frontmatter().id();
    let src_id = src.frontmatter().id();
    let (number, name) = (src.frontmatter().number(), src.frontmatter().name().to_string());

    // Validate the rationale through the model (empty/whitespace → rejected).
    Tear::new(src_id, target_id, because).map_err(|_| {
        anyhow!(
            "a tear needs a rationale; run `odm tear {source_ref} depends_on {target_ref} --because \"<why>\"`"
        )
    })?;

    if dry_run {
        writeln!(err, "would tear #{number} {name:?} depends_on {target_id}")?;
        return Ok(());
    }

    let edges = src.frontmatter_mut().edges_mut();
    let entry = TornEdge { edge: Dependency::Bare(target_id), because: because.to_string() };
    match edges.tears.iter_mut().find(|t| dependency_target(&t.edge) == target_id) {
        // Re-tearing the same target refreshes the rationale (the latest `--because`).
        Some(existing) => existing.because = because.to_string(),
        None => edges.tears.push(entry),
    }
    src.frontmatter_mut().set_updated(today());
    store.persist(&src)?;
    writeln!(err, "tore #{number} {name:?} depends_on {target_id} (because: {because})")?;
    Ok(())
}

/// `decomposed X [--children <ref…>]` — affirms that X's children fully account
/// for its scope (ODD-0013 §4.5), via [`Frontmatter::affirm_decomposed`]. With
/// `--children`, affirms against those explicit nodes; without, against X's
/// current containment children (reverse `part_of`) — the form that clears a
/// `check` drift / advanced-without-decomposition finding.
///
/// Only parent-capable nodes (`project`/`arc`) can be decomposed.
pub fn decomposed(
    store: &Store,
    reference: &str,
    children: &[String],
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut doc = resolve(store, reference)?;
    let node_type = doc.frontmatter().node_type();
    let id = doc.frontmatter().id();
    let (number, name) = (doc.frontmatter().number(), doc.frontmatter().name().to_string());

    if node_type.valid_child_types().is_empty() {
        bail!(
            "only a project or arc can be `decomposed`; #{number} {name:?} is a {}",
            node_type.as_str()
        );
    }

    // Resolve the child set: the explicit `--children`, or X's current
    // containment children (derived reverse `part_of`) when none are given.
    let child_ids: Vec<Id> = if children.is_empty() {
        store
            .load_all()?
            .iter()
            .filter(|d| d.frontmatter().edges().part_of == Some(id))
            .map(|d| d.frontmatter().id())
            .collect()
    } else {
        let mut ids = Vec::new();
        for c in children {
            ids.push(resolve(store, c)?.frontmatter().id());
        }
        ids
    };

    if dry_run {
        writeln!(
            err,
            "would affirm decomposition of #{number} {name:?} ({} child(ren))",
            child_ids.len()
        )?;
        return Ok(());
    }

    let count = child_ids.len();
    doc.frontmatter_mut().affirm_decomposed(child_ids, today());
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?;
    writeln!(err, "affirmed decomposition of #{number} {name:?} ({count} child(ren))")?;
    Ok(())
}

/// `use [project|arc] X` — sets the current context slot to node X.
pub fn use_context(
    store: &Store,
    root: &Path,
    kind: UseKind,
    reference: &str,
    err: &mut dyn Write,
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
    writeln!(err, "context: {} = {} ({})", kind.label(), fm.name(), fm.id())?;
    Ok(())
}

/// `context` — shows the current project/arc selection. Data → `out`.
pub fn context(store: &Store, root: &Path, json: bool, out: &mut dyn Write) -> anyhow::Result<()> {
    let ctx = Context::load(root)?;
    let project = ctx.project.and_then(|id| store.load(id).ok());
    let arc = ctx.arc.and_then(|id| store.load(id).ok());

    if json {
        let view = serde_json::json!({
            "project": project.as_ref().map(NodeJson::from),
            "arc": arc.as_ref().map(NodeJson::from),
        });
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    match &project {
        Some(d) => writeln!(
            out,
            "project: #{} {} ({})",
            d.frontmatter().number(),
            d.frontmatter().name(),
            d.frontmatter().id()
        )?,
        None => writeln!(out, "project: (none)")?,
    }
    match &arc {
        Some(d) => writeln!(
            out,
            "arc:     #{} {} ({})",
            d.frontmatter().number(),
            d.frontmatter().name(),
            d.frontmatter().id()
        )?,
        None => writeln!(out, "arc:     (none)")?,
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// check (v2) — the single mechanical gate: aggregates every graph-level
// invariant (ODD-0013 §7, §4.3/§4.4/§4.5). It consumes the predicates built in
// arc01 slice06 (schema + link-integrity), arc02 slice02 (cycles), slice04
// (satisfaction/staleness), and slice05 (recomposition) — it does not
// reimplement them.
// ---------------------------------------------------------------------------

/// The severity of a [`CheckEntry`]. **Errors** always fail the run (exit `1`);
/// **warnings** fail only under `--strict` (ODD-0013 §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    /// A hard violation: always fails the run.
    Error,
    /// A soft signal (staleness, soft-satisfaction): fails only under `--strict`.
    Warning,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// One aggregated check finding (internal; rendered to text or [`EntryJson`]).
struct CheckEntry {
    severity: Severity,
    /// A stable, one-word code (e.g. `missing-field`, `cycle`, `orphan`).
    code: &'static str,
    /// The node the finding attaches to (a graph-spanning cycle attaches to its
    /// first member); `None` only if no single node applies.
    node: Option<Id>,
    number: Option<u32>,
    name: Option<String>,
    detail: String,
    /// The exact command or edit that resolves it (errors-as-affordances).
    fix: String,
}

/// JSON shape of one `check` finding (stable schema for `check --json`).
#[derive(Serialize)]
struct EntryJson {
    severity: String,
    code: String,
    node: Option<String>,
    number: Option<u32>,
    name: Option<String>,
    detail: String,
    fix: String,
}

/// One active tear surfaced in `check`'s active-tears listing (ODD-0013 §4.3):
/// an assumed dependency edge in effect, with the rationale that justifies it.
struct ActiveTear {
    from_label: String,
    to_label: String,
    from: Id,
    to: Id,
    because: String,
}

/// JSON shape of one active tear in `check --json` (additive to the v2 schema —
/// existing `ok`/`errors`/`warnings`/`findings` keys are unchanged).
#[derive(Serialize)]
struct TearJson {
    from: String,
    to: String,
    because: String,
}

/// The `check --json` schema marker (arc03 slice04). Additive; versions the
/// contract from its introduction forward.
pub(crate) const CHECK_SCHEMA: &str = "check/v1";

/// JSON shape of the whole `check` report (stable, documented schema).
#[derive(Serialize)]
struct CheckReport {
    /// The schema-version marker (`"check/v1"`).
    schema: &'static str,
    /// Whether the run passed (no failing findings for the active mode).
    ok: bool,
    errors: usize,
    warnings: usize,
    findings: Vec<EntryJson>,
    /// The assumed dependencies (tears) in effect, each with its rationale.
    tears: Vec<TearJson>,
}

/// A one-word violation label (stable across the JSON schema and human output).
fn violation_label(v: &Violation) -> &'static str {
    match v {
        Violation::MissingField { .. } => "missing-field",
        Violation::DanglingPartOf { .. } => "dangling-part_of",
        Violation::DanglingEdge { .. } => "dangling-edge",
        Violation::SelfSupersede => "self-supersede",
        Violation::SupersessionCycle { .. } => "supersession-cycle",
        // `Violation` is #[non_exhaustive] (v2 adds kinds); render unknowns
        // generically rather than failing the build when they appear.
        _ => "violation",
    }
}

/// A human-readable detail line for a violation.
fn violation_detail(v: &Violation) -> String {
    match v {
        Violation::MissingField { field } => format!("required field {field:?} is empty"),
        Violation::DanglingPartOf { target } => {
            format!("`part_of` references {target}, which is not in the corpus")
        }
        Violation::DanglingEdge { edge, target } => {
            format!("`{edge}` references {target}, which is not in the corpus")
        }
        Violation::SelfSupersede => "`supersedes` points at the node itself".to_string(),
        Violation::SupersessionCycle { cycle } => {
            let ids: Vec<String> = cycle.iter().map(ToString::to_string).collect();
            format!("`supersedes` forms a cycle: {}", ids.join(" -> "))
        }
        _ => "structural violation".to_string(),
    }
}

/// The exact fix affordance for a finding (errors-as-affordances). Where an
/// Arc-01 command can fix it, the command is named; otherwise the precise file
/// edit is named (the `link`/`unlink` commands that would set edges arrive in
/// Arc 02).
fn violation_fix(store: &Store, finding: &Finding) -> String {
    let file = store.path_of(finding.node);
    let file = file.display();
    match &finding.violation {
        Violation::MissingField { field: "name" } => {
            format!("run `odm rename {} \"<a name>\"`", finding.node)
        }
        Violation::MissingField { field } => {
            format!("set `{field}` in {file}")
        }
        Violation::DanglingPartOf { .. } => {
            format!(
                "edit {file}: repoint `edges.part_of` at an existing node (or `odm new` its parent)"
            )
        }
        Violation::DanglingEdge { edge, .. } => {
            format!("edit {file}: repoint `edges.{edge}` at an existing node")
        }
        Violation::SelfSupersede => {
            format!("edit {file}: remove the self-referential `edges.supersedes`")
        }
        Violation::SupersessionCycle { .. } => {
            format!("edit {file}: break the `supersedes` cycle by removing one link")
        }
        _ => format!("inspect {file}"),
    }
}

/// A short `#<number> <name>` label for a node id, falling back to the id.
fn label_of(by_id: &HashMap<Id, &Frontmatter>, id: Id) -> String {
    by_id.get(&id).map_or_else(|| id.to_string(), |f| format!("#{} {}", f.number(), f.name()))
}

/// The target id of a dependency edge (bare or gate-qualified).
fn dependency_target(dep: &Dependency) -> Id {
    match dep {
        Dependency::Bare(id) => *id,
        Dependency::Qualified { node, .. } => *node,
    }
}

/// Whether a node has *advanced* past its initial (planning) gate — the trigger
/// for the staleness check. Unjudgeable (so `false`) when its type has no
/// configured gate-set.
fn has_advanced(fm: &Frontmatter, gates: &GateSets) -> bool {
    gates.for_type(fm.node_type()).is_some_and(|gset| {
        gset.sequence().iter().enumerate().any(|(i, gate)| i > 0 && fm.status().has_reached(gate))
    })
}

/// The severity of a recomposition finding. A structural break (**orphan**) or
/// a now-false assertion (**decomposition-drift**) is a hard `Error`; the
/// advisory "develop this further" findings (**undeveloped-stub**,
/// **advanced-without-decomposition**) are `Warning`s that fail only under
/// `--strict` — matching the staleness / soft-satisfaction treatment
/// (ODD-0013 §4.4; slice06 CDC rec #2). Everyday `check` no longer exits 1
/// merely because an arc was advanced before its decomposition was affirmed.
fn recompose_severity(issue: &Issue) -> Severity {
    match issue {
        Issue::UndevelopedStub { .. } | Issue::AdvancedWithoutDecomposition => Severity::Warning,
        // Orphan, decomposition-drift, and any future structural issue: error.
        _ => Severity::Error,
    }
}

/// Renders a recomposition finding to `(code, detail, fix)`.
fn recompose_render(store: &Store, f: &recompose::Finding) -> (&'static str, String, String) {
    let file = store.path_of(f.node);
    let file = file.display();
    match &f.issue {
        Issue::Orphan => (
            "orphan",
            "no resolvable containment parent (recomposition is not total)".to_string(),
            format!("edit {file}: set `edges.part_of` to its container (or `odm new` the parent)"),
        ),
        Issue::UndevelopedStub { gate } => (
            "undeveloped-stub",
            format!("advanced to gate {gate:?} with zero children"),
            format!("decompose it (`odm new` its children) or hold its gate at planning in {file}"),
        ),
        Issue::DecompositionDrift { added, removed } => (
            "decomposition-drift",
            format!(
                "children changed since `decomposed` was affirmed (added {}, removed {})",
                added.len(),
                removed.len()
            ),
            format!("re-affirm `decomposed` in {file} after the child-set change"),
        ),
        Issue::AdvancedWithoutDecomposition => (
            "advanced-without-decomposition",
            "reached its terminal gate without affirming `decomposed: complete`".to_string(),
            format!("affirm `decomposed` in {file} before completing it"),
        ),
        _ => (
            "recomposition",
            "structural decomposition issue".to_string(),
            format!("inspect {file}"),
        ),
    }
}

/// Aggregates every graph-level invariant over the given frontmatters (the
/// index-backed projection, slice06) into a single ordered list of findings:
/// schema + link-integrity (v1), cycles-without-tears, recomposition
/// (orphan/stub/drift/advance-without), out-of-order/staleness, and
/// below-threshold (soft-satisfied) dependencies. It owns no I/O beyond the gate
/// config — the caller supplies the (reconciled) frontmatters.
fn aggregate(
    store: &Store,
    root: &Path,
    frontmatters: &[Frontmatter],
) -> anyhow::Result<(Vec<CheckEntry>, Vec<ActiveTear>)> {
    let by_id: HashMap<Id, &Frontmatter> = frontmatters.iter().map(|f| (f.id(), f)).collect();
    let (gates, threshold) = load_gate_config(root)?;
    let graph = NodeGraph::build(frontmatters);
    let satisfaction = Satisfaction::compute(frontmatters, &gates, threshold);

    let mut entries = Vec::new();

    // (a) schema + link-integrity + supersession (v1) — hard errors.
    for f in odm_core::check::check(frontmatters) {
        entries.push(CheckEntry {
            severity: Severity::Error,
            code: violation_label(&f.violation),
            node: Some(f.node),
            number: Some(f.number),
            name: Some(f.name.clone()),
            detail: violation_detail(&f.violation),
            fix: violation_fix(store, &f),
        });
    }

    // (b) cycle-without-tear (slice02) — a hard error; passes once torn.
    let tears = odm_core::graph::frontmatter_tears(frontmatters);
    if let Err(cycle) = graph.topological_order(&tears) {
        let members = cycle.members();
        let chain: Vec<String> = members.iter().map(|&id| label_of(&by_id, id)).collect();
        let head = members.first().copied();
        let fix = match (members.first(), members.get(1)) {
            (Some(a), Some(b)) => {
                format!("`odm tear {a} depends_on {b} --because \"<reason>\"` to break the cycle")
            }
            _ => "break the dependency cycle by tearing one edge".to_string(),
        };
        entries.push(CheckEntry {
            severity: Severity::Error,
            code: "cycle",
            node: head,
            number: head.and_then(|id| by_id.get(&id).map(|f| f.number())),
            name: head.and_then(|id| by_id.get(&id).map(|f| f.name().to_string())),
            detail: format!("ordering cycle (depends_on/consumes): {}", chain.join(" -> ")),
            fix,
        });
    }

    // (c) recomposition (slice05) — orphan/drift are errors; stub/advance-without
    // are warnings (fail only under --strict). See `recompose_severity`.
    for f in recompose::integrity(frontmatters, &gates) {
        let (code, detail, fix) = recompose_render(store, &f);
        entries.push(CheckEntry {
            severity: recompose_severity(&f.issue),
            code,
            node: Some(f.node),
            number: Some(f.number),
            name: Some(f.name.clone()),
            detail,
            fix,
        });
    }

    // (d) soft-satisfaction + (e) out-of-order/staleness (slice04) — warnings.
    // Walk nodes in id order for deterministic output.
    let mut ordered: Vec<&Frontmatter> = frontmatters.iter().collect();
    ordered.sort_by_key(|f| f.id());
    for fm in ordered {
        let reasons = graph.blocked(fm.id(), &satisfaction);

        for reason in &reasons {
            if let Block::SoftSatisfied { dep, evidence, threshold } = reason {
                entries.push(CheckEntry {
                    severity: Severity::Warning,
                    code: "soft-satisfied",
                    node: Some(fm.id()),
                    number: Some(fm.number()),
                    name: Some(fm.name().to_string()),
                    detail: format!(
                        "dependency {} satisfied only at evidence={} (threshold {})",
                        label_of(&by_id, *dep),
                        evidence.as_str(),
                        threshold.as_str()
                    ),
                    fix: format!(
                        "raise {}'s evidence to {} (re-run its verification)",
                        label_of(&by_id, *dep),
                        threshold.as_str()
                    ),
                });
            }
        }

        // Staleness: a node advanced past planning while a dependency is
        // unsatisfied — out of order.
        if has_advanced(fm, &gates) {
            let unsatisfied: Vec<Id> = reasons
                .iter()
                .filter_map(|r| match r {
                    Block::Unsatisfied { dep } => Some(*dep),
                    _ => None,
                })
                .collect();
            if let Some(stale) = staleness_on_advance(fm.id(), unsatisfied) {
                let deps: Vec<String> =
                    stale.unsatisfied.iter().map(|&id| label_of(&by_id, id)).collect();
                entries.push(CheckEntry {
                    severity: Severity::Warning,
                    code: "staleness",
                    node: Some(fm.id()),
                    number: Some(fm.number()),
                    name: Some(fm.name().to_string()),
                    detail: format!(
                        "advanced while dependencies are unsatisfied: {}",
                        deps.join(", ")
                    ),
                    fix: format!(
                        "satisfy {} before advancing, or `odm tear` the edge if intentional",
                        deps.join(", ")
                    ),
                });
            }
        }
    }

    // Active tears: assumed dependencies actually in effect (naming a real
    // ordering edge), each with its persisted rationale. Listed by `check` so
    // they stay visible (ODD-0013 §4.3); not findings — informational.
    let mut active: Vec<ActiveTear> = graph
        .active_tears(&tears)
        .into_iter()
        .map(|t| ActiveTear {
            from_label: label_of(&by_id, *t.from()),
            to_label: label_of(&by_id, *t.to()),
            from: *t.from(),
            to: *t.to(),
            because: t.rationale().to_string(),
        })
        .collect();
    active.sort_by_key(|t| (t.from, t.to));

    Ok((entries, active))
}

/// `check` — the single mechanical gate: aggregates every graph-level invariant
/// over the full corpus. Returns the exit code ([`EXIT_OK`] when the run passes,
/// [`EXIT_VIOLATIONS`] when it fails). The report is data → `out`.
///
/// Errors always fail. Warnings (staleness, soft-satisfaction) fail only under
/// `strict` (the CI mode). A clean corpus prints `check: ok`.
///
/// # Errors
///
/// Returns an error (which the caller maps to exit code `2`) if the corpus
/// cannot be loaded or the gate config is invalid.
pub fn check(
    store: &Store,
    root: &Path,
    strict: bool,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<u8> {
    let (gates, _threshold) = load_gate_config(root)?;
    let frontmatters =
        index_frontmatters(store, &gates).context("reconciling the index to check")?;
    let (entries, tears) = aggregate(store, root, &frontmatters)?;

    let errors = entries.iter().filter(|e| e.severity == Severity::Error).count();
    let warnings = entries.iter().filter(|e| e.severity == Severity::Warning).count();
    let failed = errors > 0 || (strict && warnings > 0);
    let code = if failed { EXIT_VIOLATIONS } else { EXIT_OK };

    if json {
        let report = CheckReport {
            schema: CHECK_SCHEMA,
            ok: !failed,
            errors,
            warnings,
            findings: entries
                .iter()
                .map(|e| EntryJson {
                    severity: e.severity.as_str().to_string(),
                    code: e.code.to_string(),
                    node: e.node.map(|id| id.to_string()),
                    number: e.number,
                    name: e.name.clone(),
                    detail: e.detail.clone(),
                    fix: e.fix.clone(),
                })
                .collect(),
            tears: tears
                .iter()
                .map(|t| TearJson {
                    from: t.from.to_string(),
                    to: t.to.to_string(),
                    because: t.because.clone(),
                })
                .collect(),
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&report)?)?;
        return Ok(code);
    }

    if entries.is_empty() {
        writeln!(out, "check: ok ({} node(s), no problems)", frontmatters.len())?;
        write_active_tears(out, &tears)?;
        return Ok(EXIT_OK);
    }

    writeln!(out, "check: {errors} error(s), {warnings} warning(s)")?;
    for e in &entries {
        let who = match (e.number, &e.name, e.node) {
            (Some(n), Some(name), Some(id)) => format!("#{n} {name:?} ({id})"),
            _ => "(corpus)".to_string(),
        };
        writeln!(out, "  [{}] {who}: [{}] {}", e.severity.as_str(), e.code, e.detail)?;
        writeln!(out, "    fix: {}", e.fix)?;
    }
    write_active_tears(out, &tears)?;
    if !strict && warnings > 0 && errors == 0 {
        writeln!(out, "(warnings do not fail; run with --strict to enforce)")?;
    }
    Ok(code)
}

/// Writes the active-tears listing (assumed dependencies in effect, each with
/// its rationale) to `out`, or nothing when there are no active tears.
fn write_active_tears(out: &mut dyn Write, tears: &[ActiveTear]) -> anyhow::Result<()> {
    if tears.is_empty() {
        return Ok(());
    }
    writeln!(out, "active tears ({}):", tears.len())?;
    for t in tears {
        writeln!(out, "  {} depends_on {} (because: {})", t.from_label, t.to_label, t.because)?;
    }
    Ok(())
}

/// One `check` finding surfaced for another command (e.g. `orient`) to render,
/// already reduced to its display parts. The caller filters by `is_error`.
pub(crate) struct IntegrityFinding {
    /// Whether this is a hard error (vs. a warning).
    pub(crate) is_error: bool,
    /// The stable one-word code (`orphan`, `cycle`, …).
    pub(crate) code: &'static str,
    /// `#<number> <name>` for the node, or `(corpus)` when none applies.
    pub(crate) who: String,
    /// The human-readable detail line.
    pub(crate) detail: String,
}

/// Runs the full `check` aggregation over the index-backed frontmatters and
/// returns its findings (schema, links, cycles, recomposition, staleness,
/// soft-satisfaction) for another command to surface. `orient` filters these to
/// errors so a structural break is unmissable (slice02 ruling 3). Reuses
/// [`aggregate`] — integrity is never re-walked.
///
/// # Errors
///
/// Returns an error if the gate config is invalid.
pub(crate) fn integrity_findings(
    store: &Store,
    root: &Path,
    frontmatters: &[Frontmatter],
) -> anyhow::Result<Vec<IntegrityFinding>> {
    let (entries, _tears) = aggregate(store, root, frontmatters)?;
    Ok(entries
        .into_iter()
        .map(|e| IntegrityFinding {
            is_error: e.severity == Severity::Error,
            code: e.code,
            who: match (e.number, e.name) {
                (Some(n), Some(name)) => format!("#{n} {name}"),
                _ => "(corpus)".to_string(),
            },
            detail: e.detail,
        })
        .collect())
}

// ---------------------------------------------------------------------------
// derived order: next / blocked / path (ODD-0013 §4.1/§4.4)
// ---------------------------------------------------------------------------

/// Loads the gate-sets and satisfaction threshold from `<root>/odm.toml`
/// (absent file ⇒ empty gate-sets and the default threshold).
pub(crate) fn load_gate_config(root: &Path) -> anyhow::Result<(GateSets, Evidence)> {
    let text = std::fs::read_to_string(root.join("odm.toml")).unwrap_or_default();
    let gates = GateSets::from_toml_str(&text).map_err(|e| anyhow!("gate config: {e}"))?;
    let threshold = threshold_from_toml(&text).map_err(|e| anyhow!("satisfaction config: {e}"))?;
    Ok((gates, threshold))
}

/// The index-backed corpus frontmatters: `reconcile` the `.odm/` index against
/// the corpus (the warm path — freshens any edit) and reconstruct one
/// `Frontmatter` per record via the index→graph adapter — **no full corpus
/// parse**. The reconstructed frontmatters carry exactly what the graph,
/// satisfaction, recomposition, and provenance read — id, type, number, edges,
/// status with evidence, and (since slice06) `origin` and `decomposed` — so
/// every consumer built on them is identical to its `load_all` baseline. Bodies
/// stay out of the index (ODD-0014 §3.5); a consumer that needs one (only
/// `orient`'s vision) does its own targeted [`Store::load`].
///
/// # Errors
///
/// Returns an error if the index cannot be reconciled.
pub(crate) fn index_frontmatters(
    store: &Store,
    gates: &GateSets,
) -> anyhow::Result<Vec<Frontmatter>> {
    let snapshot =
        odm_index::reconcile(store, &odm_index::default_index_path(store.root()))?.snapshot;
    Ok(odm_index::frontmatters_from_records(&snapshot.records, gates))
}

/// The corpus, graph, and satisfaction needed by every derived-order query.
///
/// **Index-backed (slice05):** `load` freshens the `.odm/` index (`reconcile`)
/// and reconstructs the `Frontmatter`s from the index records via the index→graph
/// adapter — no corpus parse — then feeds the *existing* `NodeGraph::build` /
/// `Satisfaction::compute`. The reconstructed frontmatters carry exactly what the
/// graph + satisfaction read (id/type/edges/status+evidence), so the derived
/// order is identical to the `load_all` baseline.
struct Derived {
    frontmatters: Vec<Frontmatter>,
    graph: NodeGraph,
    satisfaction: Satisfaction,
}

impl Derived {
    fn load(store: &Store, root: &Path) -> anyhow::Result<Self> {
        let (gates, threshold) = load_gate_config(root)?;
        let frontmatters = index_frontmatters(store, &gates)?;
        let graph = NodeGraph::build(&frontmatters);
        let satisfaction = Satisfaction::compute(&frontmatters, &gates, threshold);
        Ok(Self { frontmatters, graph, satisfaction })
    }

    /// A short `#<number> <name>` label for a node id (falls back to the id).
    fn label(&self, id: Id) -> String {
        self.frontmatters
            .iter()
            .find(|f| f.id() == id)
            .map_or_else(|| id.to_string(), |f| format!("#{} {}", f.number(), f.name()))
    }

    fn number(&self, id: Id) -> Option<u32> {
        self.frontmatters.iter().find(|f| f.id() == id).map(Frontmatter::number)
    }
}

#[derive(Serialize)]
struct SoftDepJson {
    dep: String,
    number: Option<u32>,
    evidence: String,
}

#[derive(Serialize)]
struct ReadyJson {
    node: String,
    number: Option<u32>,
    effective_evidence: Option<String>,
    soft: Vec<SoftDepJson>,
}

/// `next` — the ready frontier, soft-satisfied deps flagged. Data → `out`.
pub fn next(store: &Store, root: &Path, json: bool, out: &mut dyn Write) -> anyhow::Result<()> {
    let derived = Derived::load(store, root)?;
    let ready = derived.graph.next(&derived.satisfaction);

    if json {
        let view: Vec<ReadyJson> = ready
            .iter()
            .map(|r| ReadyJson {
                node: r.node.to_string(),
                number: derived.number(r.node),
                effective_evidence: derived
                    .graph
                    .min_evidence(r.node, &derived.satisfaction)
                    .map(|e| e.as_str().to_string()),
                soft: r
                    .soft
                    .iter()
                    .map(|s| SoftDepJson {
                        dep: s.dep.to_string(),
                        number: derived.number(s.dep),
                        evidence: s.evidence.as_str().to_string(),
                    })
                    .collect(),
            })
            .collect();
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    if ready.is_empty() {
        writeln!(out, "next: (nothing ready)")?;
        return Ok(());
    }
    for r in &ready {
        writeln!(out, "{}", derived.label(r.node))?;
        for soft in &r.soft {
            writeln!(
                out,
                "  ⚠ dep {} satisfied at evidence={}",
                derived.label(soft.dep),
                soft.evidence.as_str()
            )?;
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct BlockJson {
    kind: String,
    node: String,
    number: Option<u32>,
    evidence: Option<String>,
    threshold: Option<String>,
}

/// `blocked X` — the unsatisfied / soft-satisfied / externally-blocked reasons.
pub fn blocked(
    store: &Store,
    root: &Path,
    reference: &str,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let derived = Derived::load(store, root)?;
    let target = resolve(store, reference)?;
    let reasons = derived.graph.blocked(target.frontmatter().id(), &derived.satisfaction);

    if json {
        let view: Vec<BlockJson> = reasons.iter().map(block_json).collect();
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    if reasons.is_empty() {
        writeln!(out, "blocked: nothing holding {}", derived.label(target.frontmatter().id()))?;
        return Ok(());
    }
    for reason in &reasons {
        match reason {
            Block::Unsatisfied { dep } => {
                writeln!(out, "  unsatisfied dependency {}", derived.label(*dep))?;
            }
            Block::SoftSatisfied { dep, evidence, threshold } => {
                writeln!(
                    out,
                    "  low-evidence dependency {} at evidence={} (raise to {})",
                    derived.label(*dep),
                    evidence.as_str(),
                    threshold.as_str()
                )?;
            }
            Block::ExternallyBlocked { by } => {
                writeln!(out, "  blocked by {}", derived.label(*by))?;
            }
        }
    }
    Ok(())
}

fn block_json(reason: &Block<Id, Evidence>) -> BlockJson {
    match reason {
        Block::Unsatisfied { dep } => BlockJson {
            kind: "unsatisfied".to_string(),
            node: dep.to_string(),
            number: None,
            evidence: None,
            threshold: None,
        },
        Block::SoftSatisfied { dep, evidence, threshold } => BlockJson {
            kind: "soft-satisfied".to_string(),
            node: dep.to_string(),
            number: None,
            evidence: Some(evidence.as_str().to_string()),
            threshold: Some(threshold.as_str().to_string()),
        },
        Block::ExternallyBlocked { by } => BlockJson {
            kind: "blocked-by".to_string(),
            node: by.to_string(),
            number: None,
            evidence: None,
            threshold: None,
        },
    }
}

#[derive(Serialize)]
struct PathJson {
    path: Option<Vec<String>>,
}

/// `path X [Y]` — the critical dependency chain from X, or a path from X to Y.
pub fn path(
    store: &Store,
    root: &Path,
    reference: &str,
    to: Option<&str>,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let derived = Derived::load(store, root)?;
    let from = resolve(store, reference)?.frontmatter().id();
    let target = to.map(|t| resolve(store, t)).transpose()?.map(|d| d.frontmatter().id());
    let chain = derived.graph.path(from, target, &[]);

    if json {
        let view =
            PathJson { path: chain.as_ref().map(|c| c.iter().map(ToString::to_string).collect()) };
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    match chain {
        Some(chain) => {
            let labels: Vec<String> = chain.iter().map(|&id| derived.label(id)).collect();
            writeln!(out, "{}", labels.join(" -> "))?;
        }
        None => writeln!(out, "path: no dependency path")?,
    }
    Ok(())
}
