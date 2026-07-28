//! `odm-migrate` — the legacy → node-model importer (Arc 06, ODD-0013 §9).
//!
//! Reads a legacy number-/state-directory ODD corpus and produces equivalent
//! nodes in the new model — **losslessly** (legacy provenance preserved),
//! **safely** (no legacy file is read-only-violated: this crate never writes,
//! moves, or deletes a legacy file — M-5), and **repeatably** (re-running is a
//! no-op, keyed on the preserved legacy `number` — M-3).
//!
//! The importer is a pure library returning a structured [`MigrationReport`];
//! rendering lives in `odm-cli`. It reuses `odm-store` for node persistence and
//! `odm-core` for the node model and gate-sets — it reimplements neither.
//!
//! ## Mapping (see [`mapping`])
//!
//! | Legacy | → | New |
//! |---|---|---|
//! | `number` | → | fresh ULID id; `number` preserved (idempotence key) |
//! | `state` (progression) | → | cumulative `design`/`research` gate reach |
//! | `state` (deferred/rejected/withdrawn/superseded) | → | retirement marker |
//! | `supersedes`/`superseded-by` | → | a `supersedes` edge on the superseding node |
//! | title/created/updated/tags/component | → | carried |
//! | `author`/`version` | → | typed `author`/`version` fields (ODD-0025 §2.2, slice03) |
//! | node type | → | `NodeType::Research` iff `tags` include `research`, else `NodeType::Design` |
//!
//! ## Fidelity (slice03, ODD-0025 §2.1/§2.2 — see [`fidelity`])
//!
//! Both importers import the source body **verbatim** — no synthesized
//! heading, no transformation — and every created node carries a `source`
//! record (path, class, normalization, migrating tool + version). The hard
//! body-hash gate ([`fidelity::verify_body_hash`]) is a migration-time
//! invariant check: the body about to be persisted must hash identically
//! (after normalization) to the body that was read.
//!
//! ## Numbering space (slice02, settled against the real corpus)
//!
//! Document numbers are a **distinct numbering space** from work-node
//! (`project`/`arc`/`slice`) numbers: the idempotence check keys on
//! `(is a document node, number)` — `existing_doc_numbers` filters to
//! `design`/`research` nodes — so a legacy ODD `#13` never collides with a work
//! node `#13`. Confirmed on odm's
//! own `docs/design` (ODD numbers `2`, `9`–`19`, no collisions).
//!
//! ## Supersession value shape (slice02)
//!
//! `supersedes`/`superseded-by` accept `null`, a bare number, or a human ref
//! string (`"ODD-0011"`) — see [`legacy::LegacyFrontmatter`]. odm's real corpus
//! uses `null` throughout (no supersession); the string form is handled for
//! robustness. No real ODD supersedes more than one predecessor, so the model's
//! single `supersedes` edge (warn-on-multiple) is sufficient — no amendment
//! needed. There are no `07-deferred` ODDs, so `deferred → retire` stands as the
//! interim mapping (revisitable if a real deferred ODD appears).

pub mod coverage;
pub mod fidelity;
pub mod legacy;
pub mod mapping;
pub mod replan;
pub mod restamp;
pub mod selfhost;

pub use selfhost::SelfHostReport;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use odm_core::frontmatter::{Document, Edges, SupersedeKind, Supersedes};
use odm_core::{Id, NodeType};
use odm_store::{Store, StoreError};

use crate::legacy::{LegacyDoc, LegacyError};
use crate::mapping::{DocGates, MapError, Prepared, build_node, prepare};
use crate::restamp::{SourceTags, restamp_taxonomy};

/// Which derivation a path calls for (RH F-14).
///
/// `self-host` used to be a separate verb for the plan-set case. It is the same
/// operation — read a tree of documents, derive nodes — differing only in the
/// tree's shape, so it folds into `migrate` and the shape is detected rather
/// than declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corpus {
    /// A plan set: `project-plan.md` plus `arcNN-*/` directories → work nodes.
    Plan,
    /// A legacy state-directory corpus: `NN-state/` dirs → document nodes.
    Legacy,
}

/// Classifies the tree at `path`.
///
/// A plan set is recognised by its **structure** — arc directories, or a
/// `project-plan.md` — because that is what actually distinguishes the two
/// derivations. Anything else is treated as legacy, which is the older and more
/// forgiving path.
#[must_use]
pub fn detect_corpus(path: &Path) -> Corpus {
    if path.join("project-plan.md").is_file() {
        return Corpus::Plan;
    }
    let has_arc_dirs = std::fs::read_dir(path).ok().is_some_and(|entries| {
        entries.flatten().any(|e| {
            e.path().is_dir()
                && e.file_name().to_string_lossy().to_ascii_lowercase().starts_with("arc")
        })
    });
    if has_arc_dirs { Corpus::Plan } else { Corpus::Legacy }
}

/// Whether a migration writes or only previews.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Persist the created nodes.
    Commit,
    /// Report the plan; write nothing (M-4).
    DryRun,
}

impl Mode {
    /// The mode selected by a `--dry-run` flag.
    #[must_use]
    pub fn from_dry_run(dry_run: bool) -> Self {
        if dry_run { Mode::DryRun } else { Mode::Commit }
    }

    /// Whether this is a dry run (writes nothing).
    #[must_use]
    pub fn is_dry_run(self) -> bool {
        self == Mode::DryRun
    }
}

/// A node whose schema marker was **backfilled** (stamped) this run — an existing
/// unversioned `nodes/` file brought up to `<type>/v1.0` (ODD-0020 V-5). Under
/// `--dry-run`, a node that *would* be stamped.
#[derive(Debug, Clone)]
pub struct Upgraded {
    /// The node's number.
    pub number: u32,
    /// The node's identity.
    pub id: Id,
    /// The node's name.
    pub name: String,
    /// The schema marker stamped (e.g. `"design/v1.0"`).
    pub schema: String,
}

/// A node the run created (or, under `--dry-run`, *would* create).
#[derive(Debug, Clone)]
pub struct Created {
    /// The preserved legacy number.
    pub number: u32,
    /// The freshly-minted node identity.
    pub id: Id,
    /// The node name (from the legacy title).
    pub name: String,
    /// The document type it classified as (`design` or `research`, C-2).
    pub node_type: NodeType,
    /// Whether it imported as a retired node (dustbin/parked state).
    pub retired: bool,
}

/// Why a legacy doc was skipped — never a silent drop (M-6).
#[derive(Debug, Clone)]
pub enum SkipReason {
    /// A node with this legacy `number` already exists as a document node
    /// (idempotence — M-3), or a duplicate `number` appeared earlier in the run.
    AlreadyExists,
    /// The file could not be read or its frontmatter was invalid.
    Malformed(String),
    /// The frontmatter parsed but could not be mapped (missing number / unknown
    /// state).
    Unmappable(MapError),
}

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipReason::AlreadyExists => write!(f, "already exists (skipped)"),
            SkipReason::Malformed(m) => write!(f, "malformed: {m}"),
            SkipReason::Unmappable(e) => write!(f, "{e}"),
        }
    }
}

/// A skipped legacy doc: its source, its number (if known), and why.
#[derive(Debug, Clone)]
pub struct Skipped {
    /// The legacy number, if it could be read.
    pub number: Option<u32>,
    /// The source path.
    pub path: PathBuf,
    /// Why it was skipped.
    pub reason: SkipReason,
}

/// The outcome of a migration run: what was created, what was skipped (and why),
/// and any warnings (e.g. an unresolved supersession). No information is dropped
/// — every legacy doc lands in exactly one of `created`/`skipped`.
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// Nodes created (or, under `--dry-run`, that would be created).
    pub created: Vec<Created>,
    /// Legacy docs skipped, with reasons.
    pub skipped: Vec<Skipped>,
    /// Nodes whose schema marker was backfilled (stamped) this run (ODD-0020 V-5).
    pub upgraded: Vec<Upgraded>,
    /// Non-fatal warnings (dangling / unrepresentable supersession edges).
    pub warnings: Vec<String>,
    /// Whether this was a dry run (nothing was written).
    pub dry_run: bool,
}

impl MigrationReport {
    /// The number of nodes created (or planned, under `--dry-run`).
    #[must_use]
    pub fn created_count(&self) -> usize {
        self.created.len()
    }

    /// The number of legacy docs skipped.
    #[must_use]
    pub fn skipped_count(&self) -> usize {
        self.skipped.len()
    }

    /// The number of existing nodes whose schema was backfilled this run.
    #[must_use]
    pub fn upgraded_count(&self) -> usize {
        self.upgraded.len()
    }
}

/// A fatal migration error — an I/O or store failure that stops the run. A
/// *per-doc* problem is never fatal: it is a [`Skipped`] entry (M-6).
#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    /// The existing corpus could not be loaded (for the idempotence check).
    #[error("loading the existing corpus")]
    LoadCorpus(#[source] StoreError),
    /// A created node could not be persisted.
    #[error("persisting migrated node #{number}")]
    Persist {
        /// The legacy number of the node that failed to write.
        number: u32,
        /// The underlying store error.
        #[source]
        source: StoreError,
    },
    /// A migrated node's body does not match its source, byte-for-byte after
    /// normalization (ODD-0025 §2.1) — a hard failure that stops the
    /// migration, never a per-doc skip.
    #[error(
        "body-hash mismatch for {context}: the migrated body does not match its source, \
         byte-for-byte after normalization"
    )]
    BodyHashMismatch {
        /// A human label identifying the node that failed (e.g. `"#1602 (slice)"`).
        context: String,
    },
    /// A node's source body could not be read (ODD-0025 §2.1's verbatim
    /// import — `selfhost.rs` reading `arc-plan.md`/`slice-doc.md`/
    /// `project-plan.md`).
    #[error("reading the source body at {path}")]
    SourceRead {
        /// The source file that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// Migrates the legacy ODD corpus rooted at `legacy_path` into `store`, using the
/// canonical `design`/`research` gate-sets (ODD-0013 §5.1).
///
/// # Errors
///
/// [`MigrateError`] on a store load/persist failure. Per-doc problems (malformed
/// frontmatter, unknown state, dangling supersession) are reported in the returned
/// [`MigrationReport`], not raised.
pub fn migrate(
    store: &Store,
    legacy_path: &Path,
    mode: Mode,
) -> Result<MigrationReport, MigrateError> {
    migrate_with_gates(store, legacy_path, mode, &DocGates::canonical())
}

/// Like [`migrate`], but with explicit document gate-sets (e.g. a repo's
/// configured `[gates.design]` / `[gates.research]`), for when they diverge from
/// the canonical defaults.
///
/// # Errors
///
/// See [`migrate`].
pub fn migrate_with_gates(
    store: &Store,
    legacy_path: &Path,
    mode: Mode,
    gates: &DocGates,
) -> Result<MigrationReport, MigrateError> {
    let mut report = MigrationReport {
        created: Vec::new(),
        skipped: Vec::new(),
        upgraded: Vec::new(),
        warnings: Vec::new(),
        dry_run: mode.is_dry_run(),
    };

    // Taxonomy re-stamp (RH C-2) FIRST: a node still carrying the pre-C-2
    // `type: odd` does not parse at all, so nothing below — not the backfill,
    // not the idempotence scan — can read the corpus until it is rewritten.
    // Classification needs the source docs' tags, hence the collect here.
    let sources = SourceTags::collect(legacy_path);
    let restamp = restamp_taxonomy(store, &sources, mode)?;
    report.upgraded = restamp.upgraded;

    // Under `--dry-run` nothing was written, so the files still carry the dead
    // type and will not parse. Read the corpus through the re-stamp's rewritten
    // documents so the preview reports what *would* happen rather than failing
    // on the state it is proposing to fix.
    let corpus = corpus_after_restamp(store, &restamp.rewritten)?;

    // Schema backfill (ODD-0020 V-5): stamp any existing unversioned `nodes/` file
    // to `<type>/v1.0` before importing — folds the backfill into migrate as the
    // single schema-upgrade entry point (idempotent once stamped). Newly-created
    // nodes below are stamped at build time, so they are never seen here.
    report.upgraded.extend(backfill_documents(store, corpus.clone(), mode)?);
    report.upgraded.sort_by_key(|u| u.number);

    // Read every legacy doc; a read/parse failure is a reported skip, not fatal.
    let mut docs: Vec<LegacyDoc> = Vec::new();
    for path in legacy::discover(legacy_path) {
        match legacy::parse_file(&path) {
            Ok(doc) => docs.push(doc),
            Err(e) => report.skipped.push(Skipped {
                number: None,
                path: error_path(&e),
                reason: SkipReason::Malformed(e.to_string()),
            }),
        }
    }

    // The idempotence set: legacy numbers already present as document nodes.
    let existing = existing_doc_numbers(&corpus);

    // Pass 1 — decide create vs skip, minting an id per creation. `planned` maps
    // every legacy number that will exist (already-present ∪ minted-this-run), so
    // a duplicate `number` (even within one run) is caught as AlreadyExists.
    let mut planned: HashMap<u32, Id> = existing.clone();
    let mut to_create: Vec<(LegacyDoc, Prepared, Id)> = Vec::new();
    for doc in docs {
        let prep = match prepare(&doc.front) {
            Ok(prep) => prep,
            Err(e) => {
                report.skipped.push(Skipped {
                    number: doc.front.number,
                    path: doc.path.clone(),
                    reason: SkipReason::Unmappable(e),
                });
                continue;
            }
        };
        if planned.contains_key(&prep.number) {
            report.skipped.push(Skipped {
                number: Some(prep.number),
                path: doc.path.clone(),
                reason: SkipReason::AlreadyExists,
            });
            continue;
        }
        let id = Id::new();
        planned.insert(prep.number, id);
        to_create.push((doc, prep, id));
    }

    // Pass 2 — resolve supersession relations against the full id map, then build
    // + (unless dry-run) persist each node with its edge attached. Body import
    // is verbatim (`doc.body`, already read source-faithful by `legacy::parse_file`);
    // the hard hash gate (ODD-0025 §2.1) verifies the persisted body is exactly
    // the body that was read, regardless of `--dry-run` (a dry run still catches
    // a fidelity break, it just doesn't write).
    let migrated_on = chrono::Utc::now().date_naive();
    let edges = resolve_supersedes(&to_create, &planned, &mut report.warnings);
    for (doc, prep, id) in &to_create {
        let fm = build_node(*id, &doc.front, prep, gates, &doc.path, migrated_on);
        let fm = attach_supersedes(fm, edges.get(&prep.number).copied());
        let document = Document::new(fm, doc.body.clone());
        crate::fidelity::verify_body_hash(
            &doc.body,
            document.body(),
            format!("#{} ({})", prep.number, document.frontmatter().node_type()),
        )?;
        let created = Created {
            number: prep.number,
            id: *id,
            name: document.frontmatter().name().to_string(),
            node_type: document.frontmatter().node_type(),
            retired: document.frontmatter().retired().is_some(),
        };
        if !mode.is_dry_run() {
            store
                .persist(&document)
                .map_err(|source| MigrateError::Persist { number: prep.number, source })?;
        }
        report.created.push(created);
    }

    report.created.sort_by_key(|c| c.number);
    report.skipped.sort_by(|a, b| a.number.cmp(&b.number).then(a.path.cmp(&b.path)));
    Ok(report)
}

/// Stamps `<type>/v1.0` on every node in the store that lacks a schema marker
/// (ODD-0020 V-5) — the slice02 document nodes predate the field. Idempotent (a
/// stamped node is skipped), and never deletes. Under [`Mode::DryRun`] it reports
/// what it *would* stamp without writing.
///
/// `nodes/` is odm-owned, so this rewrites node files in place — the never-mutate
/// rule applies to legacy `docs/design`, **not** to `nodes/`.
///
/// # Errors
///
/// [`MigrateError`] on a store load/persist failure.
pub fn backfill_schema(store: &Store, mode: Mode) -> Result<Vec<Upgraded>, MigrateError> {
    let documents = store.load_all().map_err(MigrateError::LoadCorpus)?;
    backfill_documents(store, documents, mode)
}

/// [`backfill_schema`] over an already-loaded corpus — the form `migrate` uses,
/// so the dry-run overlay (documents that exist only in memory until the
/// re-stamp is committed) is stamped alongside the ones read from disk.
fn backfill_documents(
    store: &Store,
    documents: Vec<Document>,
    mode: Mode,
) -> Result<Vec<Upgraded>, MigrateError> {
    let mut upgraded = Vec::new();
    for mut document in documents {
        if document.frontmatter().schema().is_some() {
            continue; // already versioned — idempotent skip
        }
        document.frontmatter_mut().stamp_schema();
        let (number, id, name, schema) = {
            let fm = document.frontmatter();
            let marker = fm.schema().expect("just stamped");
            (fm.number(), fm.id(), fm.name().to_string(), marker.to_string())
        };
        if !mode.is_dry_run() {
            store.persist(&document).map_err(|source| MigrateError::Persist { number, source })?;
        }
        upgraded.push(Upgraded { number, id, name, schema });
    }
    upgraded.sort_by_key(|u| u.number);
    Ok(upgraded)
}

/// The legacy `number`s already present as document nodes in `corpus` (the
/// idempotence key set — M-3).
fn existing_doc_numbers(corpus: &[Document]) -> HashMap<u32, Id> {
    corpus
        .iter()
        .map(|d| d.frontmatter())
        .filter(|fm| matches!(fm.node_type(), NodeType::Design | NodeType::Research))
        .map(|fm| (fm.number(), fm.id()))
        .collect()
}

/// The corpus as it stands **after** the taxonomy re-stamp: the rewritten
/// documents for the nodes it touched, and the on-disk parse for everything
/// else.
///
/// With nothing rewritten (the ordinary case, and every run after the first)
/// this is exactly `store.load_all()`.
fn corpus_after_restamp(
    store: &Store,
    rewritten: &HashMap<PathBuf, Document>,
) -> Result<Vec<Document>, MigrateError> {
    if rewritten.is_empty() {
        return store.load_all().map_err(MigrateError::LoadCorpus);
    }
    let mut docs = Vec::new();
    for path in store.node_paths().map_err(MigrateError::LoadCorpus)? {
        if let Some(document) = rewritten.get(&path) {
            docs.push(document.clone());
            continue;
        }
        // Not re-stamped, so it parses: load it through the store (whose
        // id-derived path is the file name) rather than re-implementing the
        // read. A stem that is not an id cannot be a node odm wrote.
        let Some(id) = path.file_stem().and_then(|s| s.to_str()).and_then(|s| s.parse().ok())
        else {
            continue;
        };
        docs.push(store.load(id).map_err(MigrateError::LoadCorpus)?);
    }
    docs.sort_by_key(|d| d.frontmatter().id());
    Ok(docs)
}

/// Resolves supersession relations from both `supersedes` (forward) and
/// `superseded-by` (reverse-derived) into `superseding_number → superseded_id`.
///
/// A dangling target (no node with that number) → a warning, no edge. A node
/// that would supersede more than one target → a warning (the model's
/// `supersedes` edge is single); the lowest-numbered target is kept for
/// determinism. Edges are only attachable to nodes created this run.
fn resolve_supersedes(
    to_create: &[(LegacyDoc, Prepared, Id)],
    number_to_id: &HashMap<u32, Id>,
    warnings: &mut Vec<String>,
) -> HashMap<u32, Id> {
    // Gather (superseding_number, superseded_number) pairs from both directions.
    let mut relations: BTreeSet<(u32, u32)> = BTreeSet::new();
    for (doc, prep, _) in to_create {
        if let Some(target) = doc.front.supersedes {
            relations.insert((prep.number, target));
        }
        if let Some(newer) = doc.front.superseded_by {
            relations.insert((newer, prep.number));
        }
    }

    let creating: BTreeSet<u32> = to_create.iter().map(|(_, prep, _)| prep.number).collect();
    // Group by superseding node (BTreeMap for deterministic warning order).
    let mut by_superseding: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (superseding, superseded) in relations {
        by_superseding.entry(superseding).or_default().push(superseded);
    }

    let mut edges: HashMap<u32, Id> = HashMap::new();
    for (superseding, mut targets) in by_superseding {
        targets.sort_unstable();
        if targets.len() > 1 {
            warnings.push(format!(
                "#{superseding} supersedes multiple targets {targets:?}; the node model's \
                 `supersedes` edge is single — kept #{}, dropped the rest",
                targets[0]
            ));
        }
        let target = targets[0];
        let Some(target_id) = number_to_id.get(&target).copied() else {
            warnings.push(format!(
                "#{superseding} supersedes #{target}, but no such node exists (dangling — \
                 no edge written)"
            ));
            continue;
        };
        if !creating.contains(&superseding) {
            warnings.push(format!(
                "#{superseding} supersedes #{target}, but the superseding node is not being \
                 created this run — edge not attached"
            ));
            continue;
        }
        edges.insert(superseding, target_id);
    }
    edges
}

/// Attaches a `supersedes` edge (kind `obsoletes` — legacy carries no kind) if
/// one was resolved for this node.
fn attach_supersedes(
    mut fm: odm_core::frontmatter::Frontmatter,
    target: Option<Id>,
) -> odm_core::frontmatter::Frontmatter {
    if let Some(node) = target {
        let mut edges: Edges = fm.edges().clone();
        edges.supersedes = Some(Supersedes { node, kind: SupersedeKind::Obsoletes });
        fm = fm.with_edges(edges);
    }
    fm
}

/// The path carried by a [`LegacyError`], for the skip report.
fn error_path(e: &LegacyError) -> PathBuf {
    match e {
        LegacyError::Io { path, .. } | LegacyError::Frontmatter { path, .. } => path.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::LegacyFrontmatter;

    /// A minimal creating triple for `resolve_supersedes` unit tests.
    fn triple(
        number: u32,
        supersedes: Option<u32>,
        superseded_by: Option<u32>,
    ) -> (LegacyDoc, Prepared, Id) {
        let front = LegacyFrontmatter {
            number: Some(number),
            title: None,
            author: None,
            version: None,
            component: None,
            tags: Vec::new(),
            created: None,
            updated: None,
            state: Some("final".to_string()),
            supersedes,
            superseded_by,
        };
        let prep = prepare(&front).unwrap();
        (LegacyDoc { front, body: String::new(), path: PathBuf::new() }, prep, Id::new())
    }

    fn id_map(to_create: &[(LegacyDoc, Prepared, Id)]) -> HashMap<u32, Id> {
        to_create.iter().map(|(_, p, i)| (p.number, *i)).collect()
    }

    #[test]
    fn mode_from_dry_run_maps_flag() {
        assert_eq!(Mode::from_dry_run(true), Mode::DryRun);
        assert_eq!(Mode::from_dry_run(false), Mode::Commit);
        assert!(Mode::DryRun.is_dry_run());
        assert!(!Mode::Commit.is_dry_run());
    }

    #[test]
    fn skip_reason_displays_each_variant() {
        assert!(SkipReason::AlreadyExists.to_string().contains("already exists"));
        assert!(SkipReason::Malformed("bad".into()).to_string().contains("malformed: bad"));
        assert!(
            SkipReason::Unmappable(MapError::MissingNumber)
                .to_string()
                .contains("missing `number`")
        );
    }

    #[test]
    fn resolve_supersedes_attaches_forward_and_reverse() {
        // #10 supersedes #1 (forward); #2 superseded-by #10 (reverse) — but only
        // #10→#1 keeps a single edge; #10→{1,2} warns (multiple targets).
        let to_create =
            vec![triple(10, Some(1), None), triple(1, None, None), triple(2, None, Some(10))];
        let ids = id_map(&to_create);
        let mut warnings = Vec::new();
        let edges = resolve_supersedes(&to_create, &ids, &mut warnings);
        assert!(warnings.iter().any(|w| w.contains("multiple targets")), "warns: {warnings:?}");
        // Lowest-numbered target (#1) is kept.
        assert_eq!(edges.get(&10), Some(&ids[&1]));
    }

    #[test]
    fn resolve_supersedes_warns_on_dangling_target() {
        let to_create = vec![triple(5, Some(999), None)];
        let ids = id_map(&to_create);
        let mut warnings = Vec::new();
        let edges = resolve_supersedes(&to_create, &ids, &mut warnings);
        assert!(edges.is_empty(), "no edge for a dangling target");
        assert!(warnings.iter().any(|w| w.contains("dangling")), "warns: {warnings:?}");
    }

    #[test]
    fn resolve_supersedes_warns_when_superseding_not_created() {
        // #1 is superseded-by #99; #99 exists (in the id map) but is not being
        // created this run → the edge can't be attached, and it is warned about.
        let to_create = vec![triple(1, None, Some(99))];
        let mut ids = id_map(&to_create);
        ids.insert(99, Id::new()); // #99 exists but is not in to_create
        let mut warnings = Vec::new();
        let edges = resolve_supersedes(&to_create, &ids, &mut warnings);
        assert!(edges.is_empty());
        assert!(warnings.iter().any(|w| w.contains("not being created")), "warns: {warnings:?}");
    }
}

#[cfg(test)]
mod corpus_tests {
    use super::{Corpus, detect_corpus};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_a_plan_set_is_detected_by_its_project_plan() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("project-plan.md"), "# Plan\n").unwrap();
        assert_eq!(detect_corpus(dir.path()), Corpus::Plan);
    }

    #[test]
    fn test_a_plan_set_is_detected_by_its_arc_directories() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("arc01-substrate")).unwrap();
        assert_eq!(detect_corpus(dir.path()), Corpus::Plan);
    }

    #[test]
    fn test_a_state_directory_corpus_is_legacy() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("01-draft")).unwrap();
        fs::create_dir(dir.path().join("06-final")).unwrap();
        assert_eq!(detect_corpus(dir.path()), Corpus::Legacy);
    }

    #[test]
    fn test_an_unrecognised_tree_falls_back_to_legacy() {
        // Legacy is the older, more forgiving path, so it is the safer default.
        let dir = TempDir::new().unwrap();
        assert_eq!(detect_corpus(dir.path()), Corpus::Legacy);
    }
}
