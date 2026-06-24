//! Command implementations over the [`Store`].
//!
//! Output is dependency-injected: query **results (data) are written to `out`**
//! while mutation confirmations and dry-run notices (**diagnostics**) are
//! written to `err`. `run` wires these to stdout/stderr; tests wire them to
//! buffers and drive commands in-process. Output stays plain so it is
//! TTY-agnostic and stable to assert on.

use std::io::Write;
use std::path::Path;

use anyhow::{Context as _, anyhow, bail};
use chrono::NaiveDate;
use odm_core::check::{Finding, Violation};
use odm_core::frontmatter::{Document, Frontmatter, SupersedeKind, Supersedes};
use odm_core::gates::GateSets;
use odm_core::graph::{Block, NodeGraph};
use odm_core::satisfaction::{Satisfaction, threshold_from_toml};
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

/// `new <type> <name>` — idempotent describe-or-create. Confirmations go to
/// `err` (diagnostics).
pub fn new(
    store: &Store,
    node_type: &str,
    name: &str,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
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
        writeln!(err, "exists: {} #{} {:?} ({})", node_type.as_str(), fm.number(), name, fm.id())?;
        return Ok(());
    }

    let next_number = all.iter().map(|d| d.frontmatter().number()).max().map_or(1, |m| m + 1);
    let id = Id::new();
    let created = id.created_at().date_naive();
    let fm = Frontmatter::new(id, next_number, node_type, name, created, created, Origin::Planned);
    let doc = Document::new(fm, format!("# {name}\n"));

    if dry_run {
        writeln!(err, "would create {} #{next_number} {name:?} ({id})", node_type.as_str())?;
        return Ok(());
    }

    store.persist(&doc)?;
    writeln!(err, "created {} #{next_number} {name:?} ({id})", node_type.as_str())?;
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

/// `list` — full scan with optional type/tag/component filters. Data → `out`.
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
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    if nodes.is_empty() {
        writeln!(out, "(no nodes)")?;
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
// check (v1)
// ---------------------------------------------------------------------------

/// JSON shape of one `check` finding (stable schema for `check --json`).
#[derive(Serialize)]
struct FindingJson {
    node: String,
    number: u32,
    name: String,
    violation: String,
    detail: String,
    fix: String,
}

/// JSON shape of the whole `check` report.
#[derive(Serialize)]
struct CheckReport {
    ok: bool,
    findings: Vec<FindingJson>,
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

/// `check` — structural validation of the full corpus. Returns the exit code
/// ([`EXIT_OK`] when clean, [`EXIT_VIOLATIONS`] when findings exist). The report
/// is data → `out`.
///
/// # Errors
///
/// Returns an error (which the caller maps to a usage/operational exit code) if
/// the corpus cannot be loaded.
pub fn check(store: &Store, json: bool, out: &mut dyn Write) -> anyhow::Result<u8> {
    let docs = store.load_all().context("loading the corpus to check")?;
    let frontmatters: Vec<Frontmatter> = docs.iter().map(|d| d.frontmatter().clone()).collect();
    let findings = odm_core::check::check(&frontmatters);

    if json {
        let report = CheckReport {
            ok: findings.is_empty(),
            findings: findings
                .iter()
                .map(|f| FindingJson {
                    node: f.node.to_string(),
                    number: f.number,
                    name: f.name.clone(),
                    violation: violation_label(&f.violation).to_string(),
                    detail: violation_detail(&f.violation),
                    fix: violation_fix(store, f),
                })
                .collect(),
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&report)?)?;
        return Ok(exit_code(&findings));
    }

    if findings.is_empty() {
        writeln!(out, "check: ok ({} node(s), no problems)", frontmatters.len())?;
        return Ok(EXIT_OK);
    }

    writeln!(out, "check: {} problem(s) found", findings.len())?;
    for f in &findings {
        writeln!(
            out,
            "  #{} {:?} ({}): [{}] {}",
            f.number,
            f.name,
            f.node,
            violation_label(&f.violation),
            violation_detail(&f.violation)
        )?;
        writeln!(out, "    fix: {}", violation_fix(store, f))?;
    }
    Ok(EXIT_VIOLATIONS)
}

/// Maps a finding set to the `check` exit code.
fn exit_code(findings: &[Finding]) -> u8 {
    if findings.is_empty() { EXIT_OK } else { EXIT_VIOLATIONS }
}

// ---------------------------------------------------------------------------
// derived order: next / blocked / path (ODD-0013 §4.1/§4.4)
// ---------------------------------------------------------------------------

/// Loads the gate-sets and satisfaction threshold from `<root>/odm.toml`
/// (absent file ⇒ empty gate-sets and the default threshold).
fn load_gate_config(root: &Path) -> anyhow::Result<(GateSets, Evidence)> {
    let text = std::fs::read_to_string(root.join("odm.toml")).unwrap_or_default();
    let gates = GateSets::from_toml_str(&text).map_err(|e| anyhow!("gate config: {e}"))?;
    let threshold = threshold_from_toml(&text).map_err(|e| anyhow!("satisfaction config: {e}"))?;
    Ok((gates, threshold))
}

/// The corpus, graph, and satisfaction needed by every derived-order query.
struct Derived {
    docs: Vec<Document>,
    graph: NodeGraph,
    satisfaction: Satisfaction,
}

impl Derived {
    fn load(store: &Store, root: &Path) -> anyhow::Result<Self> {
        let docs = store.load_all()?;
        let frontmatters: Vec<Frontmatter> = docs.iter().map(|d| d.frontmatter().clone()).collect();
        let (gates, threshold) = load_gate_config(root)?;
        let graph = NodeGraph::build(&frontmatters);
        let satisfaction = Satisfaction::compute(&frontmatters, &gates, threshold);
        Ok(Self { docs, graph, satisfaction })
    }

    /// A short `#<number> <name>` label for a node id (falls back to the id).
    fn label(&self, id: Id) -> String {
        self.docs.iter().find(|d| d.frontmatter().id() == id).map_or_else(
            || id.to_string(),
            |d| format!("#{} {}", d.frontmatter().number(), d.frontmatter().name()),
        )
    }

    fn number(&self, id: Id) -> Option<u32> {
        self.docs.iter().find(|d| d.frontmatter().id() == id).map(|d| d.frontmatter().number())
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
