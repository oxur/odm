//! The **self-host cutover** (arc06 slice04): import odm's own `design-v1.0.0`
//! plan set — the project, its arcs, and their slices — into the node model as
//! `project`/`arc`/`slice` **work nodes** under `nodes/`, so odm manages its own
//! plan and `orient`/`rollup`/`check` run over the self-hosted corpus.
//!
//! Reuses the slice01 rails (report / idempotence / `--dry-run` / never-delete /
//! two-pass id resolution). The **directory hierarchy is the tree** — no prose
//! parsing for structure; `arcNN-*/` are arcs, `arcNN-*/sliceMM*/` are slices, and
//! the containing directory is the project root. Numbers are derived from the dir
//! prefixes (a stable, disjoint scheme — see [`arc_number`]/[`slice_number`]);
//! names from each doc's H1; gate status from the arc-close signal (the Project
//! Ledger P-row or an arc-level close file) + per-slice `closing-report.md`
//! presence, at [`Evidence::Asserted`] (claimed-from-record).
//!
//! **Scope = the v1.0.0 MVP (A1–A6).** `arc07`/`arc08` (the post-MVP horizon, 0
//! slices, owned separately) are **not** imported — see [`arc_in_scope`].
//!
//! **Never-delete:** this writes `nodes/` only; the plan-set Markdown is the
//! human-authored source and is never read-write-violated.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::gates::GateSet;
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;

use crate::{Created, MigrateError, Mode, SkipReason, Skipped};

/// The highest arc number in the v1.0.0 MVP scope (A1–A6). `arc07`/`arc08` are
/// the post-MVP horizon and are not imported.
const MAX_MVP_ARC: u32 = 6;

/// The project (root) node's number — a high, fixed value disjoint from the document
/// numbering space (legacy ODDs are `2`, `9`–`20`) and from the arc/slice ranges.
pub(crate) const PROJECT_NUMBER: u32 = 1000;

/// The canonical `project` gate-set (ODD-0013 §5.1). Kept in sync with the
/// `[gates.project]` written to `odm.toml` at cutover.
#[must_use]
pub fn canonical_project_gates() -> GateSet {
    GateSet::new(["planned", "in-progress", "complete", "verified"].map(String::from).to_vec())
}

/// The canonical `arc` gate-set (ODD-0013 §5.1).
#[must_use]
pub fn canonical_arc_gates() -> GateSet {
    GateSet::new(["planned", "in-progress", "complete", "verified"].map(String::from).to_vec())
}

/// The canonical `slice` gate-set (the built/tested vocabulary of LEDGER-DISCIPLINE;
/// ODD-0013 §5.1's leaf gates trimmed to what odm's own slices reach).
#[must_use]
pub fn canonical_slice_gates() -> GateSet {
    GateSet::new(["planned", "built", "tested"].map(String::from).to_vec())
}

/// The work-node number for arc `major` (`arc01` → 1100 … `arc06` → 1600) — a
/// stable, per-arc-disjoint value in the work range (≥ 1000).
#[must_use]
pub fn arc_number(major: u32) -> u32 {
    PROJECT_NUMBER + major * 100
}

/// The work-node number for a slice: `arc_number(arc_major) + position`, where
/// `position` is the slice's `MM` (`slice05` → 5) or, for a fractional slice
/// (`slice05.1`), `MM*10 + k` (→ 51). Stable and unique within its arc.
#[must_use]
pub fn slice_number(arc_major: u32, slice_major: u32, slice_minor: Option<u32>) -> u32 {
    let position = slice_minor.map_or(slice_major, |k| slice_major * 10 + k);
    arc_number(arc_major) + position
}

/// Whether an arc number is inside the MVP self-host scope (A1–A6).
#[must_use]
pub fn arc_in_scope(major: u32) -> bool {
    (1..=MAX_MVP_ARC).contains(&major)
}

/// The outcome of a self-host cutover run.
#[derive(Debug, Clone)]
pub struct SelfHostReport {
    /// Work nodes created (or, under `--dry-run`, that would be created).
    pub created: Vec<Created>,
    /// Plan nodes skipped (already present — idempotence).
    pub skipped: Vec<Skipped>,
    /// Whether this was a dry run (nothing written).
    pub dry_run: bool,
}

impl SelfHostReport {
    /// The number of work nodes created (or planned, under `--dry-run`).
    #[must_use]
    pub fn created_count(&self) -> usize {
        self.created.len()
    }

    /// The number of plan nodes skipped (already present).
    #[must_use]
    pub fn skipped_count(&self) -> usize {
        self.skipped.len()
    }
}

/// A closed/active/planned status derived from the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkStatus {
    /// A closed arc / completed slice — its full gate sequence is reached.
    Closed,
    /// The active arc / project — planning + in-progress reached (partial).
    Active,
    /// Planned only — no gates reached.
    Planned,
}

/// A discovered plan node, before id assignment.
#[derive(Debug, Clone)]
pub(crate) struct PlanNode {
    pub(crate) node_type: NodeType,
    pub(crate) number: u32,
    pub(crate) name: String,
    /// The `(type, number)` of its containment parent (`None` for the project).
    parent_key: Option<(NodeType, u32)>,
    status: WorkStatus,
    /// The plan directory (or file) this node derives from — the path whose git
    /// history supplies its real `created`/`updated` (RH F-20).
    pub(crate) source: std::path::PathBuf,
}

/// Runs the self-host cutover over the `design-v1.0.0` plan set rooted at
/// `plan_root`, importing the project + A1–A6 arcs + their slices into `store`.
///
/// Idempotent (keyed on `(type, number)`), `--dry-run`-safe, never-delete. The
/// project (root) node is persisted **last** so a partial failure never leaves a
/// dangling root (children-up ordering — the reflexive-import safety discipline).
///
/// # Errors
///
/// [`MigrateError`] on a store load/persist failure.
pub fn self_host(
    store: &Store,
    plan_root: &Path,
    mode: Mode,
) -> Result<SelfHostReport, MigrateError> {
    let plan = discover(plan_root);

    // Idempotence: work nodes already present, keyed on (type, number).
    let existing = existing_work_keys(store)?;

    // Pass 1 — decide create vs skip, minting an id per creation. `ids` maps every
    // (type, number) that will exist (present ∪ minted) so `part_of` can resolve.
    let mut ids: HashMap<(NodeType, u32), Id> = existing.clone();
    let mut to_create: Vec<(PlanNode, Id)> = Vec::new();
    let mut skipped = Vec::new();
    for node in plan {
        let key = (node.node_type, node.number);
        if existing.contains_key(&key) {
            skipped.push(Skipped {
                number: Some(node.number),
                path: plan_root.to_path_buf(),
                reason: SkipReason::AlreadyExists,
            });
            continue;
        }
        let id = Id::new();
        ids.insert(key, id);
        to_create.push((node, id));
    }

    // The child ids of each arc/project (for the `decomposed` affirmation on a
    // closed arc — suppresses the spurious advanced-without-decomposition warning).
    let mut children: BTreeMap<(NodeType, u32), Vec<Id>> = BTreeMap::new();
    for (node, id) in &to_create {
        if let Some(parent) = node.parent_key {
            children.entry(parent).or_default().push(*id);
        }
    }

    // Pass 2 — build each node (schema-stamped, parented, gates from status), then
    // persist **children-up**: slices, then arcs, then the project root last.
    let today = today();
    let mut built: Vec<(Frontmatter, u32)> = Vec::new();
    for (node, id) in &to_create {
        built.push((build_node(node, *id, &ids, &children, today), node.number));
    }
    built.sort_by_key(|(fm, _)| persist_rank(fm.node_type()));

    let mut created = Vec::new();
    for (fm, _) in &built {
        created.push(Created {
            number: fm.number(),
            id: fm.id(),
            name: fm.name().to_string(),
            node_type: fm.node_type(),
            retired: false,
        });
        if !mode.is_dry_run() {
            let document = Document::new(fm.clone(), format!("# {}\n", fm.name()));
            store
                .persist(&document)
                .map_err(|source| MigrateError::Persist { number: fm.number(), source })?;
        }
    }

    created.sort_by_key(|c| c.number);
    skipped.sort_by_key(|s| s.number);
    Ok(SelfHostReport { created, skipped, dry_run: mode.is_dry_run() })
}

/// The persist order rank: slices (0) before arcs (1) before the project (2), so
/// the tree is written children-up and the root lands last.
fn persist_rank(node_type: NodeType) -> u8 {
    match node_type {
        NodeType::Slice => 0,
        NodeType::Arc => 1,
        _ => 2, // project (and any non-work, though none are built here)
    }
}

/// Builds one work node: type/number/name carried, `stamp_schema`'d `<type>/v1.0`,
/// `part_of` resolved from the parent key, gates reached per `status`, and (for a
/// closed arc) its `decomposed` set affirmed against its children.
fn build_node(
    node: &PlanNode,
    id: Id,
    ids: &HashMap<(NodeType, u32), Id>,
    children: &BTreeMap<(NodeType, u32), Vec<Id>>,
    today: NaiveDate,
) -> Frontmatter {
    let mut fm = Frontmatter::new(
        id,
        node.number,
        node.node_type,
        &node.name,
        today,
        today,
        Origin::Planned,
    );
    fm.stamp_schema();

    if let Some(parent_key) = node.parent_key {
        if let Some(&parent_id) = ids.get(&parent_key) {
            fm.edges_mut().part_of = Some(parent_id);
        }
    }

    let gates = gate_set_for(node.node_type);
    let reach_to = terminal_index(node.node_type, node.status);
    for gate in gates.sequence().iter().take(reach_to) {
        let _ = fm.status_mut().set_gate(&gates, gate, None, Evidence::Asserted, today);
    }

    // A closed arc has fully decomposed into its slices — affirm it so `check`'s
    // recomposition pass does not warn "advanced without decomposition".
    if node.node_type == NodeType::Arc && node.status == WorkStatus::Closed {
        if let Some(kids) = children.get(&(NodeType::Arc, node.number)) {
            fm.affirm_decomposed(kids.clone(), today);
        }
    }
    fm
}

/// How many gates (from the sequence start) a node of `node_type` reaches for
/// `status`: a closed node reaches all; active reaches through in-progress/built;
/// planned reaches none.
fn terminal_index(node_type: NodeType, status: WorkStatus) -> usize {
    let len = gate_set_for(node_type).sequence().len();
    match status {
        WorkStatus::Closed => len,
        // "in-progress" (arc/project index 1) / "built" (slice index 1) reached.
        WorkStatus::Active => 2.min(len),
        WorkStatus::Planned => 0,
    }
}

/// The canonical gate-set for a work-node type (project/arc/slice).
fn gate_set_for(node_type: NodeType) -> GateSet {
    match node_type {
        NodeType::Project => canonical_project_gates(),
        NodeType::Arc => canonical_arc_gates(),
        _ => canonical_slice_gates(),
    }
}

/// Today (UTC) — the cutover date carried as `created`/`updated` (these work nodes
/// are created *now*; the plan docs' own history lives in git).
fn today() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

/// The `(type, number)` of every existing work node in the store (the idempotence
/// key set — a re-run skips these).
fn existing_work_keys(store: &Store) -> Result<HashMap<(NodeType, u32), Id>, MigrateError> {
    let documents = store.load_all().map_err(MigrateError::LoadCorpus)?;
    Ok(documents
        .into_iter()
        .map(|d| d.frontmatter().clone())
        .filter(|fm| fm.node_type().is_work())
        .map(|fm| ((fm.node_type(), fm.number()), fm.id()))
        .collect())
}

/// The plan nodes under `plan_root`, for a caller that wants the *derived
/// facts* rather than an import — the C-5 re-stamp joins these to the existing
/// corpus by `(type, number)`.
pub(crate) fn plan_nodes(plan_root: &Path) -> Vec<PlanNode> {
    discover(plan_root)
}

/// Discovers the in-scope plan nodes under `plan_root`: the project root, the
/// A1–A6 arcs, and their slices. Deterministic (sorted).
fn discover(plan_root: &Path) -> Vec<PlanNode> {
    let closed_arcs = closed_arcs_from_pledger(plan_root);
    let mut nodes = Vec::new();

    // The project root.
    nodes.push(PlanNode {
        node_type: NodeType::Project,
        number: PROJECT_NUMBER,
        name: h1_or_slug(&plan_root.join("project-plan.md"), "odm"),
        parent_key: None,
        // The project is active while any arc is open (A6 open).
        status: WorkStatus::Active,
        source: plan_root.join("project-plan.md"),
    });

    let mut arc_dirs: Vec<(u32, std::path::PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(plan_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some((major, _)) = parse_prefix(&name, "arc") {
                if arc_in_scope(major) {
                    arc_dirs.push((major, path));
                }
            }
        }
    }
    arc_dirs.sort_by_key(|(m, _)| *m);

    for (arc_major, arc_dir) in &arc_dirs {
        let closed = closed_arcs.contains(arc_major) || arc_close_file(arc_dir);
        nodes.push(PlanNode {
            node_type: NodeType::Arc,
            number: arc_number(*arc_major),
            name: h1_or_slug(&arc_dir.join("arc-plan.md"), &format!("arc{arc_major}")),
            parent_key: Some((NodeType::Project, PROJECT_NUMBER)),
            status: if closed { WorkStatus::Closed } else { WorkStatus::Active },
            source: arc_dir.clone(),
        });

        let mut slice_dirs: Vec<(u32, Option<u32>, std::path::PathBuf)> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(arc_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some((major, minor)) = parse_prefix(&name, "slice") {
                    slice_dirs.push((major, minor, path));
                }
            }
        }
        slice_dirs.sort_by_key(|(maj, min, _)| (*maj, min.unwrap_or(0)));

        for (slice_major, slice_minor, slice_dir) in &slice_dirs {
            let complete = slice_dir.join("closing-report.md").exists();
            nodes.push(PlanNode {
                node_type: NodeType::Slice,
                number: slice_number(*arc_major, *slice_major, *slice_minor),
                name: slice_name(slice_dir, *slice_major, *slice_minor),
                parent_key: Some((NodeType::Arc, arc_number(*arc_major))),
                status: if complete { WorkStatus::Closed } else { WorkStatus::Planned },
                source: slice_dir.clone(),
            });
        }
    }
    nodes
}

/// Whether an arc directory carries an arc-level close file (`closing-report.md`
/// or the legacy `arc-close.md`) — one of the two arc-closed signals.
fn arc_close_file(arc_dir: &Path) -> bool {
    arc_dir.join("closing-report.md").exists() || arc_dir.join("arc-close.md").exists()
}

/// The arc numbers marked `done` in the project-plan §5 Project Ledger — the
/// authoritative arc-close signal (A1/A2 predate the arc-level close file, so file
/// presence alone is insufficient; the P-row `P-N` maps to arc `N`). Tolerant of
/// row formatting: reads the status column of each `| P-N | … |` row.
fn closed_arcs_from_pledger(plan_root: &Path) -> BTreeSet<u32> {
    let text = std::fs::read_to_string(plan_root.join("project-plan.md")).unwrap_or_default();
    let mut closed = BTreeSet::new();
    for line in text.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // A Project-Ledger row: cells[1] = "P-N", cells[6] = status.
        let Some(pid) = cells.get(1).and_then(|c| c.strip_prefix("P-")) else {
            continue;
        };
        if let (Ok(n), Some(&status)) = (pid.parse::<u32>(), cells.get(6)) {
            if status == "done" {
                closed.insert(n); // P-N ↔ arc N
            }
        }
    }
    closed
}

/// Parses a `<prefix>NN[.M]-…` directory name into `(major, minor?)`, or `None`
/// if it does not match (`arc06-…` → `(6, None)`; `slice05.1-…` → `(5, Some(1))`).
///
/// `pub(crate)`: the coverage detector ([`crate::coverage`]) reuses this rather
/// than re-deriving the same coordinate parse.
pub(crate) fn parse_prefix(dir_name: &str, prefix: &str) -> Option<(u32, Option<u32>)> {
    let rest = dir_name.strip_prefix(prefix)?;
    let head = rest.split('-').next().unwrap_or(rest);
    match head.split_once('.') {
        Some((maj, min)) => Some((maj.parse().ok()?, Some(min.parse().ok()?))),
        None => Some((head.parse().ok()?, None)),
    }
}

/// The slice's name: the H1 of its `slice-doc.md` (fallback `cc-prompt.md`, then a
/// dir-derived label).
fn slice_name(slice_dir: &Path, major: u32, minor: Option<u32>) -> String {
    for doc in ["slice-doc.md", "cc-prompt.md"] {
        if let Some(h1) = first_h1(&slice_dir.join(doc)) {
            return h1;
        }
    }
    match minor {
        Some(k) => format!("slice {major}.{k}"),
        None => format!("slice {major}"),
    }
}

/// The first-H1 of a markdown file (a line starting with `# `), or a fallback.
fn h1_or_slug(path: &Path, fallback: &str) -> String {
    first_h1(path).unwrap_or_else(|| fallback.to_string())
}

/// The first Markdown H1 (`# …`) in a file, if it can be read and has one.
fn first_h1(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    text.lines().find_map(|line| line.strip_prefix("# ").map(|h| h.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbering_is_disjoint_and_stable() {
        assert_eq!(arc_number(1), 1100);
        assert_eq!(arc_number(6), 1600);
        assert_eq!(slice_number(1, 1, None), 1101);
        assert_eq!(slice_number(6, 4, None), 1604);
        // A fractional slice (slice05.1) → position 51, still inside its arc range.
        assert_eq!(slice_number(2, 5, Some(1)), 1251);
        // Disjoint from the odd numbering space (2, 9–20) and cross-type-unique.
        assert!(arc_number(1) > 20 && slice_number(1, 1, None) > 20);
    }

    #[test]
    fn arc_scope_excludes_post_mvp() {
        assert!(arc_in_scope(1) && arc_in_scope(6));
        assert!(!arc_in_scope(7) && !arc_in_scope(8));
    }

    #[test]
    fn parse_prefix_reads_major_and_minor() {
        assert_eq!(parse_prefix("arc06-migrate-self-host", "arc"), Some((6, None)));
        assert_eq!(parse_prefix("slice05.1-evidence", "slice"), Some((5, Some(1))));
        assert_eq!(parse_prefix("slice01-x", "slice"), Some((1, None)));
        assert_eq!(parse_prefix("readme", "arc"), None);
    }

    #[test]
    fn terminal_index_maps_status_to_gate_count() {
        assert_eq!(terminal_index(NodeType::Arc, WorkStatus::Closed), 4);
        assert_eq!(terminal_index(NodeType::Arc, WorkStatus::Active), 2);
        assert_eq!(terminal_index(NodeType::Slice, WorkStatus::Closed), 3);
        assert_eq!(terminal_index(NodeType::Slice, WorkStatus::Planned), 0);
    }
}
