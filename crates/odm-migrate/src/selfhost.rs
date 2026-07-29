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
//! gets a **name-derived**, collision-handled `number` **handle**
//! ([`named_arc_number`], v1.9 arc-migration-fidelity s05, F8) — `number` is a
//! non-structural human handle (not identity, not order; ODD-0013 §2.3), and
//! the handle is recomputable from the arc's slug alone, so it stays stable
//! when another named arc is added or removed elsewhere in the corpus. The
//! former hardcoded A1–A6 scope cap was the root cause of six real arcs going
//! silently unrepresented and directly contradicted "no file left behind" —
//! removed outright, not widened.
//!
//! **Source-keyed identity (v1.9, arc-migration-fidelity s05, F1/F2/F3/F9).**
//! `number` — numbered or named — keys **nothing** for correctness: idempotence
//! is keyed on **`source.paths`**, the stable identity every migrated node
//! carries (ODD-0025 §2.0/§2.2). A `number`, even a name-derived one, is still
//! only ever a display/CLI-lookup label. See [`self_host`]'s doc for the
//! source-match / coordinate-transition rule.
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
///
/// `pub(crate)`: the coverage detector ([`crate::coverage`]) reuses this to
/// derive a named arc's slice numbers directly from its own name-derived
/// handle (arc-migration-fidelity s09, F-6) — `slice_number` can't be reused
/// there since it re-derives `arc_number(arc_major)` from a **raw major**,
/// which a named arc's handle is not.
pub(crate) fn slice_position(slice_major: u32, slice_minor: Option<u32>) -> u32 {
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

/// How many slots the named-arc number band spans before repeating — large
/// enough that a genuine hash collision between two real arc slugs is
/// vanishingly unlikely, while comfortably fitting inside `u32`.
const NAMED_ARC_SLOTS: u32 = 1_000_000;

/// A stable (FNV-1a) hash of an arc slug — deterministic and recomputable
/// from the slug alone, with no dependency on any other arc in the corpus.
fn slug_hash(slug: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in slug.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// The `number` handle for a named arc directory's `slug` (its own directory
/// name, e.g. `"arc-migration-fidelity"`) — **name-derived** (v1.9,
/// arc-migration-fidelity s05 F8): a deterministic hash of the slug into the
/// ≥[`NAMED_ARC_BASE`] band, stepped by [`NAMED_ARC_STEP`]. Recomputable from
/// the slug alone, so it is **stable** when another named arc is added or
/// removed elsewhere in the corpus — replacing the position-based
/// `named_arc_number(index)` (v1.6–v1.8), which shifted every later arc's
/// handle on any addition and, under the old `(type, number)` idempotence
/// key, caused a re-run to mint a duplicate node (the s04 CDC v1.8 finding —
/// now impossible, since s05 also keys idempotence on `source.paths`, not
/// `number`).
///
/// `taken` is the set of handles already assigned earlier in the same
/// [`discover`] pass, in its deterministic (sorted) processing order — on the
/// vanishingly rare case two slugs hash to the same slot, the one processed
/// first keeps the natural slot and the other is bumped to the next free one,
/// deterministically (never a silent collision).
#[must_use]
pub fn named_arc_number(slug: &str, taken: &BTreeSet<u32>) -> u32 {
    let mut candidate = NAMED_ARC_BASE + (slug_hash(slug) % NAMED_ARC_SLOTS) * NAMED_ARC_STEP;
    while taken.contains(&candidate) {
        candidate += NAMED_ARC_STEP;
    }
    candidate
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

    /// The number of plan nodes skipped (already present) — a superset of
    /// [`rewritten_count`](Self::rewritten_count): a rewritten node is also
    /// "not created", just distinguishable in the list by its
    /// [`SkipReason::PathRewritten`] reason.
    #[must_use]
    pub fn skipped_count(&self) -> usize {
        self.skipped.len()
    }

    /// The number of matched nodes whose `source.paths` was rewritten to the
    /// canonical, anchor-relative form (arc-migration-fidelity s08 F-4) —
    /// counted from `skipped`, not a separate list, since a rewrite is a kind
    /// of "not created" outcome, just not an untouched one.
    #[must_use]
    pub fn rewritten_count(&self) -> usize {
        self.skipped.iter().filter(|s| matches!(s.reason, SkipReason::PathRewritten)).count()
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
/// `plan_root`, importing the project + every arc/slice directory into `store`.
///
/// **Idempotence keys on `source.paths`** (arc-migration-fidelity s05, F-1),
/// not `(type, number)`: a work node that already carries a `source` record is
/// matched by its stable source path, immune to a `number` that shifts between
/// runs (the s04 named-arc re-run-duplicate hazard, v1.8). A pre-`source`
/// legacy node — the corpus predates this slice — falls back to a **one-time**
/// structural `(type, number)` coordinate match (F-2): it is recognized as
/// already-imported (never re-created) and its `source` is backfilled in place,
/// so every subsequent run matches it by source too. `--dry-run`-safe,
/// never-delete. The project (root) node is persisted **last** so a partial
/// failure never leaves a dangling root (children-up ordering — the
/// reflexive-import safety discipline).
///
/// # Errors
///
/// [`MigrateError`] on a store load/persist failure.
pub fn self_host(
    store: &Store,
    plan_root: &Path,
    mode: Mode,
) -> Result<SelfHostReport, MigrateError> {
    // Canonicalize once, up front (arc-migration-fidelity s08 F-1/F-2): every
    // `PlanNode.source` — and therefore every `body_source_path` — is built by
    // joining onto `plan_root`, so if `plan_root` itself isn't already
    // absolute-and-canonical, those derived paths won't line up cleanly
    // against the (absolute) anchor below, and a caller-spelling difference
    // (relative arg, extra `..`, a different cwd) could leak into the stored
    // form. Falls back to the raw path if it doesn't exist (discover() itself
    // degrades the same way — an empty plan, not an error).
    let plan_root_buf = plan_root.canonicalize().unwrap_or_else(|_| plan_root.to_path_buf());
    let plan_root = plan_root_buf.as_path();
    let anchor = crate::fidelity::anchor_for(plan_root);

    let plan = discover(plan_root);
    let today = today();

    // The existing-node identity index: `by_source` (the primary key, F-1)
    // keyed on the **canonical, anchor-relative** form of every stored
    // `source.paths` entry — whether the entry itself is already stored
    // relative (post-s08) or still absolute (pre-s08; s08 F-4's transition
    // case, since `relativize` tolerates both) — from every work node that
    // already carries a `source` record; `by_coordinate` (the one-time
    // transition fallback, F-2) from every work node that does not — the
    // corpus's pre-slice05 state.
    let mut by_source: HashMap<String, Document> = HashMap::new();
    let mut by_coordinate: HashMap<(NodeType, u32), Document> = HashMap::new();
    for document in store.load_all().map_err(MigrateError::LoadCorpus)? {
        let fm = document.frontmatter();
        if !fm.node_type().is_work() {
            continue;
        }
        match fm.source() {
            Some(source) => {
                for path in &source.paths {
                    by_source.insert(crate::fidelity::relativize(&anchor, path), document.clone());
                }
            }
            None => {
                by_coordinate.insert((fm.node_type(), fm.number()), document.clone());
            }
        }
    }

    // Pass 1 — decide create / transition-populate / path-rewrite / skip,
    // minting an id per creation. `ids` maps every (type, number) **this
    // run** will have wired (present ∪ minted) so `part_of` can resolve,
    // regardless of what number an already-existing matched node happens to
    // carry on disk.
    let mut ids: HashMap<(NodeType, u32), Id> = HashMap::new();
    let mut to_create: Vec<(PlanNode, Id)> = Vec::new();
    let mut to_populate: Vec<(PlanNode, Document)> = Vec::new();
    let mut to_rewrite: Vec<(PlanNode, Document, String)> = Vec::new();
    let mut skipped = Vec::new();
    for node in plan {
        let key = (node.node_type, node.number);
        let source_key = crate::fidelity::relativize(&anchor, &body_source_path(&node));

        if let Some(existing_doc) = by_source.get(&source_key) {
            ids.insert(key, existing_doc.frontmatter().id());
            let stored_is_canonical = existing_doc
                .frontmatter()
                .source()
                .is_some_and(|s| s.paths == [std::path::PathBuf::from(&source_key)]);
            if stored_is_canonical {
                skipped.push(Skipped {
                    number: Some(node.number),
                    path: plan_root.to_path_buf(),
                    reason: SkipReason::AlreadyExists,
                });
            } else {
                // s08 F-4, the re-mint guard: matched by the canonical key
                // derived from a non-canonical stored form (still absolute,
                // pre-s08) — recognized, never re-created; the stored path
                // string is corrected in place below.
                skipped.push(Skipped {
                    number: Some(node.number),
                    path: plan_root.to_path_buf(),
                    reason: SkipReason::PathRewritten,
                });
                to_rewrite.push((node, existing_doc.clone(), source_key));
            }
            continue;
        }
        if let Some(existing_doc) = by_coordinate.get(&key) {
            ids.insert(key, existing_doc.frontmatter().id());
            let excluded = node.node_type == NodeType::Project
                || existing_doc.frontmatter().retired().is_some();
            if excluded {
                // The project's body is a synthesis, never a 1:1 migration
                // (ODD-0025 §2.3); a retired node is a historical record, not
                // live work (`replan.rs`'s established principle) — either is
                // recognized by coordinate, never queued for the gated backfill
                // below (arc-migration-fidelity s06 F-1/F-5/F-10, CDC v2.1
                // finding: this transition used to stamp `source` onto the
                // project ungated, and carried no retired guard at all).
                skipped.push(Skipped {
                    number: Some(node.number),
                    path: plan_root.to_path_buf(),
                    reason: SkipReason::AlreadyExists,
                });
            } else {
                skipped.push(Skipped {
                    number: Some(node.number),
                    path: plan_root.to_path_buf(),
                    reason: SkipReason::SourcePopulated,
                });
                to_populate.push((node, existing_doc.clone()));
            }
            continue;
        }

        let id = Id::new();
        ids.insert(key, id);
        to_create.push((node, id));
    }

    // s08 F-4: rewrite each matched-but-non-canonical node's `source.paths`
    // to the canonical relative form — **only** the path string changes; id,
    // edges, status, body, schema, and `updated` are all left exactly as
    // they were (a pure identity-form correction, not a reconcile — so it
    // runs even under a matched project/retired node, which can never reach
    // here since those are always canonicalized the same way they're
    // excluded, via `by_coordinate`/the transition below, not `by_source`).
    if !mode.is_dry_run() {
        for (node, existing_doc, canonical_key) in &to_rewrite {
            let mut new_source = existing_doc
                .frontmatter()
                .source()
                .expect("matched via by_source implies a source record")
                .clone();
            new_source.paths = vec![std::path::PathBuf::from(canonical_key)];
            let new_fm = existing_doc.frontmatter().clone().with_source(new_source);
            let new_document = Document::new(new_fm, existing_doc.body().to_string());
            store
                .persist(&new_document)
                .map_err(|source| MigrateError::Persist { number: node.number, source })?;
        }
    }

    // The one-time coordinate→source transition (F-2): backfill `source` onto
    // each matched legacy node **in place**, through the same gated,
    // project-excluding [`reconcile_source`] policy [`repair`] uses (s06 F-1 —
    // one `source`-population policy, not two). `to_populate` never contains
    // the project node (excluded above), so this is `Some` for every entry;
    // the `if let` still holds generically should that ever change. The gate
    // runs even under `--dry-run` — a dry run still catches a drifted body,
    // it just doesn't write (mirrors [`repair`]).
    for (node, existing_doc) in &to_populate {
        let Some(new_document) = reconcile_source(existing_doc, node, &anchor, today)? else {
            continue;
        };
        if !mode.is_dry_run() {
            store
                .persist(&new_document)
                .map_err(|source| MigrateError::Persist { number: node.number, source })?;
        }
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
    // its `source` record (stored relative-and-canonical, s08 F-1), then
    // persist **children-up**: slices, then arcs, then the project root last.
    let mut built: Vec<(Frontmatter, String)> = Vec::new();
    for (node, id) in &to_create {
        let fm = build_node(node, *id, &ids, &children, today);
        let source_path = body_source_path(node);
        let body = std::fs::read_to_string(&source_path)
            .map_err(|source| MigrateError::SourceRead { path: source_path.clone(), source })?;
        let relative = crate::fidelity::relativize(&anchor, &source_path);
        let fm = fm.with_source(crate::fidelity::build_source(
            vec![std::path::PathBuf::from(relative)],
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

/// One node repaired / source-backfilled **update-in-place** (ODD-0025 §2.8):
/// a stub's real body, or a faithful non-stub node's `source` record alone,
/// written into the *same* node.
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

/// Reconciles one existing node's `source` record against its plan-set
/// source, **update-in-place** (ODD-0025 §2.8) — the single gated,
/// project-excluding policy [`repair`] and [`self_host`]'s coordinate→source
/// transition both use (arc-migration-fidelity s06 F-1/F-5/F-10, unifying
/// what CDC's v2.1 finding identified as two independent `source`-backfill
/// paths, one of them ungated).
///
/// Returns `Ok(None)` for the **project** node, or a **retired** one, without
/// touching or reading anything else: the project's body is a *synthesis* of
/// `project-plan.md` §1 (`replan.rs::vision_from_plan`), never a 1:1 migration
/// (ODD-0025 §2.3), so it can never pass the gate below (F-6); a retired node
/// is a historical record, not live work — `replan.rs`'s established
/// principle, extended here so a tombstone that happens to share a coordinate
/// with a live plan directory is never silently rewritten.
///
/// For any other node:
/// - **A stub** ([`crate::fidelity::is_stub_body`], ≤ 1 non-blank body line —
///   the exact predicate the coverage detector, s01, counts by; not re-derived
///   here, F-8-s04) has its body **replaced** with the verbatim source text
///   (ODD-0025 §2.1's original repair case, s04).
/// - **A faithful non-stub node** keeps its **existing** body untouched; the
///   hard body-hash gate ([`crate::fidelity::verify_body_hash`]) verifies it
///   against the source body — a match is a content no-op that only adds
///   `source` (s05 F-4); a **mismatch is surfaced** as
///   [`MigrateError::BodyHashMismatch`], not swallowed (s05 F-5) — the caller
///   stops there rather than silently backfilling over drift.
///
/// Either way the returned `Document` carries the same `id`/`edges`/`status`/
/// `number` — only `body` (stub case), `source`, `updated`, and the schema
/// marker change. The caller decides whether to persist it (both `repair` and
/// `self_host` skip the write, but not the gate, under `--dry-run`).
///
/// # Errors
///
/// [`MigrateError::SourceRead`] if `plan_node`'s source body can't be read;
/// [`MigrateError::BodyHashMismatch`] if a non-stub body has drifted from it.
fn reconcile_source(
    document: &Document,
    plan_node: &PlanNode,
    anchor: &Path,
    today: NaiveDate,
) -> Result<Option<Document>, MigrateError> {
    if plan_node.node_type == NodeType::Project || document.frontmatter().retired().is_some() {
        return Ok(None);
    }
    let fm = document.frontmatter();
    // Round-trip through the canonical relative form before reading (s08
    // F-3): write and read share one anchor, so a node's stored `source`
    // resolves back to the exact file this reads, not a coincidentally-equal
    // absolute path derived independently.
    let relative = crate::fidelity::relativize(anchor, &body_source_path(plan_node));
    let source_path = crate::fidelity::resolve_from_anchor(anchor, &relative);
    let source_body = std::fs::read_to_string(&source_path)
        .map_err(|source| MigrateError::SourceRead { path: source_path.clone(), source })?;

    // A stub's body is worthless and is replaced outright; a faithful
    // non-stub body is kept as-is — the gate below proves it matches the
    // source rather than blindly overwriting a possibly hand-edited body.
    let new_body = if crate::fidelity::is_stub_body(document.body()) {
        source_body.clone()
    } else {
        document.body().to_string()
    };

    let mut new_fm = fm.clone();
    new_fm.set_updated(today);
    new_fm.stamp_schema();
    let new_fm = new_fm.with_source(crate::fidelity::build_source(
        vec![std::path::PathBuf::from(relative)],
        source_class(plan_node.node_type),
        today,
    ));

    let new_document = Document::new(new_fm, new_body);
    crate::fidelity::verify_body_hash(
        &source_body,
        new_document.body(),
        format!("#{} ({}) reconcile", fm.number(), fm.node_type()),
    )?;
    Ok(Some(new_document))
}

/// Repairs / backfills every work node in `store` that lacks a `source`
/// record, **update-in-place** (ODD-0025 §2.8, generalized past the stub
/// filter by arc-migration-fidelity s05 F-4/F-5/F-6): matches it to its
/// plan-set source under `plan_root` by structural coordinate (the same
/// discovery [`self_host`] uses — a node without `source` predates this arc's
/// identity model, so coordinate is the only handle available), then
/// reconciles it via [`reconcile_source`] — the project node and any retired
/// node are excluded there, so no explicit filter is needed here.
///
/// `id`, `edges`, and `status` are preserved (only `body` — for a stub — plus
/// `source`, `updated`, and the schema marker change), via [`Store::persist`]
/// (which overwrites by id — **no `delete`**). A node with no matching
/// plan-set entry (its source directory has since moved or been removed) is
/// left untouched, not an error — repair only ever touches nodes it can
/// faithfully resolve a source for.
///
/// Idempotent: a node that already carries `source` is left alone, so
/// re-running `repair` is always safe.
///
/// # Errors
///
/// [`MigrateError`] if the corpus can't be loaded, a source body can't be
/// read, the hard body-hash gate fails (a drifted non-stub body, F-5), or a
/// persist fails.
pub fn repair(store: &Store, plan_root: &Path, mode: Mode) -> Result<RepairReport, MigrateError> {
    // See `self_host`'s matching comment: canonicalize once so every path
    // derived from `plan_root` lines up cleanly against the anchor (s08 F-1).
    let plan_root_buf = plan_root.canonicalize().unwrap_or_else(|_| plan_root.to_path_buf());
    let plan_root = plan_root_buf.as_path();
    let anchor = crate::fidelity::anchor_for(plan_root);

    let corpus = store.load_all().map_err(MigrateError::LoadCorpus)?;
    let plan_by_key: HashMap<(NodeType, u32), PlanNode> =
        discover(plan_root).into_iter().map(|n| ((n.node_type, n.number), n)).collect();

    let today = today();
    let mut repaired = Vec::new();
    for document in &corpus {
        let fm = document.frontmatter();
        if fm.source().is_some() {
            continue;
        }
        let Some(plan_node) = plan_by_key.get(&(fm.node_type(), fm.number())) else {
            continue; // no plan-set source to repair from — left as-is
        };
        let Some(new_document) = reconcile_source(document, plan_node, &anchor, today)? else {
            continue; // the project node, or a retired node — excluded (F-6)
        };

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

/// The plan nodes under `plan_root`, for a caller that wants the *derived
/// facts* rather than an import — the C-5 re-stamp joins these to the existing
/// corpus by `(type, number)`.
pub(crate) fn plan_nodes(plan_root: &Path) -> Vec<PlanNode> {
    discover(plan_root)
}

/// Discovers **every** arc/slice plan node under `plan_root` — numbered
/// (`arcNN-*`, no scope cap — v1.6 F11) and named (`arc-<slug>`) alike — plus
/// the project root. A numbered arc's `number` is derived ([`arc_number`]); a
/// named arc's is **name-derived** ([`named_arc_number`], v1.9 F8) from its own
/// slug, collision-handled against the handles already assigned earlier in
/// this same pass (in directory-sort order). Deterministic (sorted).
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
    let mut named_taken: BTreeSet<u32> = BTreeSet::new();
    for path in named {
        let slug = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let number = named_arc_number(&slug, &named_taken);
        named_taken.insert(number);
        arcs.push((number, None, path));
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
        let handle = named_arc_number("arc-migration-fidelity", &BTreeSet::new());
        assert!(handle >= NAMED_ARC_BASE, "in the named-arc band");
        assert!(handle > arc_number(8) + 99, "no collision with any numbered arc/slice");
    }

    #[test]
    fn named_arc_number_is_recomputable_from_the_slug_alone() {
        // No dependency on any other arc: the same slug always hashes the same.
        let taken = BTreeSet::new();
        assert_eq!(
            named_arc_number("arc-migration-fidelity", &taken),
            named_arc_number("arc-migration-fidelity", &taken)
        );
        // A different slug (very likely) lands on a different slot.
        assert_ne!(
            named_arc_number("arc-migration-fidelity", &taken),
            named_arc_number("arc-store-home", &taken)
        );
    }

    #[test]
    fn named_arc_number_bumps_deterministically_on_collision() {
        let natural = named_arc_number("arc-x", &BTreeSet::new());
        let mut taken = BTreeSet::new();
        taken.insert(natural);
        assert_eq!(
            named_arc_number("arc-x", &taken),
            natural + NAMED_ARC_STEP,
            "a taken slot bumps to the next free one, deterministically"
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

    // ----- s06 F-1/F-5/F-10: reconcile_source is the one gated, project- ----
    // ----- excluding `source`-population policy -----------------------------

    #[test]
    fn reconcile_source_excludes_the_project_node() {
        let node = PlanNode {
            node_type: NodeType::Project,
            number: PROJECT_NUMBER,
            name: "odm".to_string(),
            parent_key: None,
            status: WorkStatus::Active,
            // A path that does not exist: exclusion must happen *before* any
            // attempt to read a source body — the project is never gated.
            source: Path::new("/nonexistent/project-plan.md").to_path_buf(),
        };
        let fm = Frontmatter::new(
            Id::new(),
            PROJECT_NUMBER,
            NodeType::Project,
            "odm",
            today(),
            today(),
            Origin::Planned,
        );
        let document =
            Document::new(fm, "# odm\n\n# Vision\n\nSynthesized, not the source.\n".to_string());

        let result = reconcile_source(&document, &node, Path::new("/anchor"), today());
        assert!(
            matches!(result, Ok(None)),
            "the project is excluded outright, not gated and rejected: {result:?}"
        );
    }

    #[test]
    fn reconcile_source_excludes_a_retired_node() {
        let node = PlanNode {
            node_type: NodeType::Slice,
            number: slice_number(1, 1, None),
            name: "Tombstone".to_string(),
            parent_key: Some((NodeType::Arc, arc_number(1))),
            status: WorkStatus::Planned,
            // A path that does not exist: a retired node is excluded before
            // any attempt to read a source body, same as the project.
            source: Path::new("/nonexistent/slice-doc.md").to_path_buf(),
        };
        let mut fm = Frontmatter::new(
            Id::new(),
            slice_number(1, 1, None),
            NodeType::Slice,
            "Tombstone",
            today(),
            today(),
            Origin::Planned,
        );
        fm.retire("superseded by a later slice", today());
        let document = Document::new(fm, "# Tombstone\n".to_string());

        let result = reconcile_source(&document, &node, Path::new("/anchor"), today());
        assert!(
            matches!(result, Ok(None)),
            "a retired node is a historical record, never reconciled: {result:?}"
        );
    }

    #[test]
    fn reconcile_source_gates_a_drifted_non_stub_body() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source_dir = tmp.path().join("arc01-alpha");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(source_dir.join("arc-plan.md"), "# Arc 01\n\nThe real content.\n").unwrap();

        let node = PlanNode {
            node_type: NodeType::Arc,
            number: arc_number(1),
            name: "Arc 01".to_string(),
            parent_key: Some((NodeType::Project, PROJECT_NUMBER)),
            status: WorkStatus::Active,
            source: source_dir,
        };
        let fm = Frontmatter::new(
            Id::new(),
            arc_number(1),
            NodeType::Arc,
            "Arc 01",
            today(),
            today(),
            Origin::Planned,
        );
        let document = Document::new(fm, "# Arc 01\n\nA drifted, different body.\n".to_string());

        let err = reconcile_source(&document, &node, tmp.path(), today()).unwrap_err();
        assert!(matches!(err, MigrateError::BodyHashMismatch { .. }), "{err:?}");
    }

    #[test]
    fn reconcile_source_backfills_a_faithful_non_stub_body_as_a_no_op() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source_dir = tmp.path().join("arc01-alpha");
        std::fs::create_dir_all(&source_dir).unwrap();
        let body = "# Arc 01\n\nThe real, faithful content.\n";
        std::fs::write(source_dir.join("arc-plan.md"), body).unwrap();

        let node = PlanNode {
            node_type: NodeType::Arc,
            number: arc_number(1),
            name: "Arc 01".to_string(),
            parent_key: Some((NodeType::Project, PROJECT_NUMBER)),
            status: WorkStatus::Active,
            source: source_dir,
        };
        let fm = Frontmatter::new(
            Id::new(),
            arc_number(1),
            NodeType::Arc,
            "Arc 01",
            today(),
            today(),
            Origin::Planned,
        );
        let document = Document::new(fm, body.to_string());

        let reconciled = reconcile_source(&document, &node, tmp.path(), today()).unwrap().unwrap();
        assert_eq!(reconciled.body(), body, "content no-op — the existing body is kept verbatim");
        let source = reconciled.frontmatter().source().expect("source backfilled");
        assert_eq!(
            source.paths,
            vec![Path::new("arc01-alpha/arc-plan.md")],
            "relative + canonical"
        );
    }
}
