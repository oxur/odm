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
//! **No scope cap (v1.6, arc-migration-fidelity F11).** Every arc directory is
//! imported — numbered (`arcNN-*`, including the post-MVP `arc07`/`arc08`) and
//! **named** (`arc-<slug>`, no `arcNN` coordinate) alike. A numbered arc's
//! `number` is still derived from its directory (`arc_number`); a named arc
//! gets a deterministic, collision-free `number` **handle** ([`named_arc_number`])
//! — `number` is a non-structural human handle (not identity, not order;
//! ODD-0013 §2.3), so a named arc simply takes the next slot in a band above
//! the numbered arcs' range. The former hardcoded A1–A6 scope cap was the root
//! cause of six real arcs going silently unrepresented and directly
//! contradicted "no file left behind" — removed outright, not widened.
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

/// The work-node number for arc `major` (`arc01` → 1100 … `arc08` → 1800) — a
/// stable, per-arc-disjoint value in the work range (≥ 1000). `const fn` so
/// [`NAMED_ARC_BASE`] can derive from it directly rather than repeating the
/// highest-numbered-arc value as a separate magic number.
#[must_use]
pub const fn arc_number(major: u32) -> u32 {
    PROJECT_NUMBER + major * 100
}

/// The offset within an arc's number band for a slice: the slice's `MM`
/// (`slice05` → 5) or, for a fractional slice (`slice05.1`), `MM*10 + k`
/// (→ 51). Shared by [`slice_number`] (numbered arcs) and named-arc slice
/// numbering ([`discover`]) so both derive a slice's offset identically.
fn slice_position(slice_major: u32, slice_minor: Option<u32>) -> u32 {
    slice_minor.map_or(slice_major, |k| slice_major * 10 + k)
}

/// The work-node number for a slice: `arc_number(arc_major) + position`.
/// Stable and unique within its arc.
#[must_use]
pub fn slice_number(arc_major: u32, slice_major: u32, slice_minor: Option<u32>) -> u32 {
    arc_number(arc_major) + slice_position(slice_major, slice_minor)
}

/// The spacing between named-arc number bands (v1.6 F12) — matches the
/// numbered arcs' own 100-wide spacing, leaving room for up to 99 slice
/// positions per named arc before it would collide with the next one.
pub const NAMED_ARC_STEP: u32 = 100;

/// The base `number` handle for the first **named** arc directory (no
/// `arcNN` coordinate) — one band above the highest numbered arc (A1–A8,
/// `arc_number(8) == 1800`), so numbered and named arcs can never collide.
/// `number` is a non-structural human handle here (v1.6 F12) — not identity
/// (the ULID), not order (the dependency DAG).
pub const NAMED_ARC_BASE: u32 = arc_number(8) + NAMED_ARC_STEP;

/// The `number` handle for the `index`-th named arc (0-based), assigned in
/// the deterministic directory-sort order [`discover`] walks named arc
/// directories in.
#[must_use]
pub fn named_arc_number(index: usize) -> u32 {
    NAMED_ARC_BASE + u32::try_from(index).unwrap_or(u32::MAX) * NAMED_ARC_STEP
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

    // Pass 2 — build each node (schema-stamped, parented, gates from status),
    // import its **verbatim** source body (ODD-0025 §2.1 — no synthesized H1,
    // no header injection; this replaced the stub-body root cause) and attach
    // its `source` record, then persist **children-up**: slices, then arcs,
    // then the project root last.
    let today = today();
    let mut built: Vec<(Frontmatter, String)> = Vec::new();
    for (node, id) in &to_create {
        let fm = build_node(node, *id, &ids, &children, today);
        let source_path = body_source_path(node);
        let body = std::fs::read_to_string(&source_path)
            .map_err(|source| MigrateError::SourceRead { path: source_path.clone(), source })?;
        let fm = fm.with_source(crate::fidelity::build_source(
            vec![source_path],
            source_class(node.node_type),
            today,
        ));
        built.push((fm, body));
    }
    built.sort_by_key(|(fm, _)| persist_rank(fm.node_type()));

    let mut created = Vec::new();
    for (fm, body) in &built {
        let document = Document::new(fm.clone(), body.clone());
        crate::fidelity::verify_body_hash(
            body,
            document.body(),
            format!("#{} ({})", fm.number(), fm.node_type()),
        )?;
        created.push(Created {
            number: fm.number(),
            id: fm.id(),
            name: fm.name().to_string(),
            node_type: fm.node_type(),
            retired: false,
        });
        if !mode.is_dry_run() {
            store
                .persist(&document)
                .map_err(|source| MigrateError::Persist { number: fm.number(), source })?;
        }
    }

    created.sort_by_key(|c| c.number);
    skipped.sort_by_key(|s| s.number);
    Ok(SelfHostReport { created, skipped, dry_run: mode.is_dry_run() })
}

/// The actual body-source **file** for a plan node (ODD-0025 §2.1: "whole
/// file" body for the frontmatter-less planning corpus). `PlanNode.source` is
/// already the file for the project (`project-plan.md`), but a *directory*
/// for an arc/slice, so this resolves the primary doc within it.
fn body_source_path(node: &PlanNode) -> std::path::PathBuf {
    match node.node_type {
        NodeType::Project => node.source.clone(),
        NodeType::Arc => node.source.join("arc-plan.md"),
        NodeType::Slice => node.source.join("slice-doc.md"),
        // discover() never produces any other type.
        _ => node.source.clone(),
    }
}

/// The `source.class` label for a plan node's type — the same vocabulary
/// `odm-migrate::coverage`'s `DocClass` uses (`"project-plan"`/`"arc-plan"`/
/// `"slice-doc"`), so a human reading `source.class` sees a consistent name
/// regardless of which detector or importer produced it.
fn source_class(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Project => "project-plan",
        NodeType::Arc => "arc-plan",
        _ => "slice-doc",
    }
}

/// One node repaired **update-in-place** (ODD-0025 §2.8): a stub's real body
/// + `source` record written into the *same* node.
#[derive(Debug, Clone)]
pub struct Repaired {
    /// The node's number (unchanged by repair).
    pub number: u32,
    /// The node's identity (unchanged by repair).
    pub id: Id,
    /// The node's name.
    pub name: String,
    /// The node's type (`arc` or `slice`).
    pub node_type: NodeType,
}

/// The outcome of a [`repair`] run.
#[derive(Debug, Clone)]
pub struct RepairReport {
    /// Nodes repaired (or, under `--dry-run`, that would be repaired).
    pub repaired: Vec<Repaired>,
    /// Whether this was a dry run (nothing written).
    pub dry_run: bool,
}

impl RepairReport {
    /// The number of nodes repaired (or planned, under `--dry-run`).
    #[must_use]
    pub fn repaired_count(&self) -> usize {
        self.repaired.len()
    }
}

/// Repairs every **stub** work node in `store` **update-in-place** (ODD-0025
/// §2.8): matches it to its plan-set source under `plan_root` by structural
/// coordinate (the same discovery [`self_host`] uses), imports the verbatim
/// body + a fresh `source` record through the hard body-hash gate
/// ([`crate::fidelity::verify_body_hash`]), and persists the **same** node —
/// `id`, `edges`, `status`, and `number` are all preserved (only `body`,
/// `source`, `updated`, and the schema marker change), via [`Store::persist`]
/// (which overwrites by id — **no `delete`**).
///
/// A stub is decided by [`crate::fidelity::is_stub_body`] (≤ 1 non-blank body
/// line), the exact predicate the coverage detector (s01) counts by — this
/// deliberately does not re-derive it (F-8). A node with no matching plan-set
/// entry (a stub whose source directory has since moved or been removed) is
/// left untouched, not an error — repair only ever touches nodes it can
/// faithfully resolve a source for. Only `arc`/`slice` nodes are ever stubs in
/// practice (the project node's body comes from a hand-authored vision
/// section, never the stub synthesis; ODD-0025 §2.8's mechanism is generic —
/// it would apply to a document-node stub identically, but `self_host`
/// mints no document nodes, so none exists to repair here).
///
/// Idempotent: a node that is no longer a stub (already repaired, or never
/// one) is left alone, so re-running `repair` is always safe.
///
/// # Errors
///
/// [`MigrateError`] if the corpus can't be loaded, a source body can't be
/// read, the hard body-hash gate fails, or a persist fails.
pub fn repair(store: &Store, plan_root: &Path, mode: Mode) -> Result<RepairReport, MigrateError> {
    let corpus = store.load_all().map_err(MigrateError::LoadCorpus)?;
    let plan_by_key: HashMap<(NodeType, u32), PlanNode> =
        discover(plan_root).into_iter().map(|n| ((n.node_type, n.number), n)).collect();

    let today = today();
    let mut repaired = Vec::new();
    for document in &corpus {
        let fm = document.frontmatter();
        if !matches!(fm.node_type(), NodeType::Arc | NodeType::Slice) {
            continue;
        }
        if fm.retired().is_some() || !crate::fidelity::is_stub_body(document.body()) {
            continue;
        }
        let Some(plan_node) = plan_by_key.get(&(fm.node_type(), fm.number())) else {
            continue; // no plan-set source to repair from — left as-is
        };

        let source_path = body_source_path(plan_node);
        let body = std::fs::read_to_string(&source_path)
            .map_err(|source| MigrateError::SourceRead { path: source_path.clone(), source })?;

        let mut new_fm = fm.clone();
        new_fm.set_updated(today);
        new_fm.stamp_schema();
        let new_fm = new_fm.with_source(crate::fidelity::build_source(
            vec![source_path],
            source_class(plan_node.node_type),
            today,
        ));

        let new_document = Document::new(new_fm, body.clone());
        crate::fidelity::verify_body_hash(
            &body,
            new_document.body(),
            format!("#{} ({}) repair", fm.number(), fm.node_type()),
        )?;

        repaired.push(Repaired {
            number: fm.number(),
            id: fm.id(),
            name: new_document.frontmatter().name().to_string(),
            node_type: fm.node_type(),
        });

        if !mode.is_dry_run() {
            store
                .persist(&new_document)
                .map_err(|source| MigrateError::Persist { number: fm.number(), source })?;
        }
    }

    repaired.sort_by_key(|r| r.number);
    Ok(RepairReport { repaired, dry_run: mode.is_dry_run() })
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

/// Discovers **every** arc/slice plan node under `plan_root` — numbered
/// (`arcNN-*`, no scope cap — v1.6 F11) and named (`arc-<slug>`) alike — plus
/// the project root. A numbered arc's `number` is derived ([`arc_number`]); a
/// named arc's is assigned deterministically ([`named_arc_number`], in
/// directory-sort order among just the named arcs). Deterministic (sorted).
fn discover(plan_root: &Path) -> Vec<PlanNode> {
    let closed_arcs = closed_arcs_from_pledger(plan_root);
    let mut nodes = Vec::new();

    // The project root.
    nodes.push(PlanNode {
        node_type: NodeType::Project,
        number: PROJECT_NUMBER,
        name: h1_or_slug(&plan_root.join("project-plan.md"), "odm"),
        parent_key: None,
        // The project is active while any arc is open.
        status: WorkStatus::Active,
        source: plan_root.join("project-plan.md"),
    });

    let mut numbered: Vec<(u32, std::path::PathBuf)> = Vec::new();
    let mut named: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(plan_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.to_ascii_lowercase().starts_with("arc") {
                continue;
            }
            match parse_prefix(&name, "arc") {
                Some((major, None)) => numbered.push((major, path)),
                // A fractional-major (`arc06.1-…`) or non-numeric (`arc-slug`)
                // `arc*` directory has no `arcNN` coordinate — named.
                _ => named.push(path),
            }
        }
    }
    numbered.sort_by_key(|(m, _)| *m);
    named.sort();

    // `(assigned number, structural major if any, dir)` — the major is kept
    // only to look up the Project Ledger's `P-N` closure signal, which is
    // meaningless for a named arc (no `P-N` row references it by number).
    let mut arcs: Vec<(u32, Option<u32>, std::path::PathBuf)> = Vec::new();
    for (major, path) in numbered {
        arcs.push((arc_number(major), Some(major), path));
    }
    for (index, path) in named.into_iter().enumerate() {
        arcs.push((named_arc_number(index), None, path));
    }

    for (arc_num, arc_major, arc_dir) in &arcs {
        let closed = arc_major.is_some_and(|m| closed_arcs.contains(&m)) || arc_close_file(arc_dir);
        let fallback = arc_major.map_or_else(
            || arc_dir.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
            |m| format!("arc{m}"),
        );
        nodes.push(PlanNode {
            node_type: NodeType::Arc,
            number: *arc_num,
            name: h1_or_slug(&arc_dir.join("arc-plan.md"), &fallback),
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
                number: arc_num + slice_position(*slice_major, *slice_minor),
                name: slice_name(slice_dir, *slice_major, *slice_minor),
                parent_key: Some((NodeType::Arc, *arc_num)),
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
    fn named_arc_numbers_are_collision_free_with_numbered_arcs() {
        // Numbered arcs occupy up to arc_number(8) == 1800, plus up to 99 in
        // slice-position headroom (1899 max); named arcs start clear of that.
        assert_eq!(NAMED_ARC_BASE, 1900);
        assert_eq!(named_arc_number(0), 1900);
        assert_eq!(named_arc_number(1), 2000);
        assert_eq!(named_arc_number(3), 2200);
        assert!(
            named_arc_number(0) > arc_number(8) + 99,
            "no collision with any numbered arc/slice"
        );
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
