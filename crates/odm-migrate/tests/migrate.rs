//! Integration tests for `odm-migrate` over the fixture legacy corpus
//! (`test-data/legacy/` valid, `test-data/legacy-edge/` edge cases). Test names
//! carry the substrings the slice01 ledger Verify commands filter on:
//! `maps_legacy_fields_to_node`, `migrate_is_idempotent`,
//! `migrate_dry_run_writes_nothing`, `migrate_never_deletes_legacy`,
//! `dustbin_imports_as_superseded`, `migrate_malformed_reports_not_panics`.

use std::path::{Path, PathBuf};

use odm_core::NodeType;
use odm_core::frontmatter::{Document, SupersedeKind};
use odm_migrate::{Mode, SkipReason, migrate};
use odm_store::Store;
use tempfile::TempDir;

/// The absolute path of a fixture corpus under the workspace `test-data/`.
fn fixtures(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data").join(name)
}

/// Loads every node the store persisted, keyed by preserved legacy number.
fn nodes_by_number(store: &Store) -> std::collections::HashMap<u32, Document> {
    store.load_all().expect("load_all").into_iter().map(|d| (d.frontmatter().number(), d)).collect()
}

// ----- M-2: the legacy → new mapping is faithful ----------------------------

#[test]
fn maps_legacy_fields_to_node() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    let report = migrate(&store, &fixtures("legacy"), Mode::Commit).expect("migrate");
    assert_eq!(report.created_count(), 6, "six valid docs → six nodes");
    assert_eq!(report.skipped_count(), 0, "no valid doc skipped");

    let nodes = nodes_by_number(&store);

    // #5 "The new approach": Final, supersedes 4, carries metadata.
    let five = &nodes[&5];
    let fm = five.frontmatter();
    assert_eq!(fm.node_type(), NodeType::Design, "type = odd");
    assert_eq!(fm.number(), 5, "legacy number preserved");
    assert_eq!(fm.name(), "The new approach", "title → name");
    assert_eq!(fm.component(), Some("engine"), "component carried");
    assert_eq!(fm.tags(), ["approach", "current"], "tags carried");
    // Cumulative odd gate reach: Final ⇒ draft…final all reached.
    for gate in ["draft", "under-review", "revised", "accepted", "active", "final"] {
        assert!(fm.status().has_reached(gate), "#5 should have reached {gate}");
    }
    // author has no typed field → carried into the frontmatter `extra`, so it
    // survives emit.
    assert!(five.emit().unwrap().contains("author: Katherine Johnson"), "author carried in extra");

    // The supersedes edge points at #4's freshly-minted ULID (kind obsoletes).
    let four = &nodes[&4];
    let sup = fm.edges().supersedes.as_ref().expect("#5 has a supersedes edge");
    assert_eq!(sup.node, four.frontmatter().id(), "supersedes → #4's new ULID");
    assert_eq!(sup.kind, SupersedeKind::Obsoletes);

    // Fresh ULIDs, distinct per node.
    assert_ne!(fm.id(), four.frontmatter().id(), "identities are fresh + distinct");

    // A progression doc (#2 Accepted) reaches draft…accepted but not active/final.
    let two = nodes[&2].frontmatter();
    assert!(two.status().has_reached("accepted"));
    assert!(!two.status().has_reached("active"));
    assert!(two.retired().is_none(), "an accepted doc is not retired");
}

// ----- M-3: idempotent describe-or-create -----------------------------------

#[test]
fn migrate_is_idempotent() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    let first = migrate(&store, &fixtures("legacy"), Mode::Commit).expect("first migrate");
    assert_eq!(first.created_count(), 6);
    let after_first = store.load_all().unwrap().len();

    // Second run: every doc's number already exists as an odd node → 0 created.
    let second = migrate(&store, &fixtures("legacy"), Mode::Commit).expect("second migrate");
    assert_eq!(second.created_count(), 0, "re-run creates nothing");
    assert_eq!(second.skipped_count(), 6, "all six skipped as already-existing");
    assert!(
        second.skipped.iter().all(|s| matches!(s.reason, SkipReason::AlreadyExists)),
        "skips are all AlreadyExists"
    );
    assert_eq!(store.load_all().unwrap().len(), after_first, "no duplicates on disk");
}

// ----- M-4: --dry-run writes nothing ----------------------------------------

#[test]
fn migrate_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    let report = migrate(&store, &fixtures("legacy"), Mode::DryRun).expect("dry-run migrate");
    assert!(report.dry_run, "report flags dry-run");
    assert_eq!(report.created_count(), 6, "the plan still lists all six");

    // Nothing on disk: no nodes/ tree, load_all empty.
    assert!(store.load_all().unwrap().is_empty(), "dry-run wrote no nodes");
    assert!(!store_dir.path().join("nodes").exists(), "no nodes/ directory created");
}

// ----- M-5: never-delete / supersede-not-delete -----------------------------

#[test]
fn migrate_never_deletes_legacy() {
    // Copy the fixture corpus into a temp dir, snapshot every file's bytes,
    // migrate from it, and assert every legacy file is byte-for-byte intact.
    let legacy_dir = TempDir::new().unwrap();
    copy_tree(&fixtures("legacy"), legacy_dir.path());
    let before = snapshot_bytes(legacy_dir.path());
    assert!(!before.is_empty(), "sanity: fixtures copied");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    migrate(&store, legacy_dir.path(), Mode::Commit).expect("migrate");

    let after = snapshot_bytes(legacy_dir.path());
    assert_eq!(before, after, "no legacy file removed or mutated");
}

#[test]
fn dustbin_imports_as_superseded() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    migrate(&store, &fixtures("legacy"), Mode::Commit).expect("migrate");
    let nodes = nodes_by_number(&store);

    // #4 (state Superseded) → a retired node; the supersedes edge lives on #5.
    let four = nodes[&4].frontmatter();
    let retire4 = four.retired().expect("#4 imports retired");
    assert_eq!(retire4.reason, "superseded", "retirement reason preserved");
    assert!(four.edges().supersedes.is_none(), "the edge is on the superseding node, not #4");

    // #6 (state Rejected, dustbin) → a retired node (reason preserved).
    let six = nodes[&6].frontmatter();
    assert_eq!(six.retired().expect("#6 retired").reason, "rejected");
}

// ----- M-6: malformed/edge input → reported skip/warn, never a panic --------

#[test]
fn migrate_malformed_reports_not_panics() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    // The edge corpus: a missing-number doc, an unknown-state doc, and a
    // dangling-supersedes doc. Must complete cleanly (no panic).
    let report =
        migrate(&store, &fixtures("legacy-edge"), Mode::Commit).expect("migrate completes");

    // Missing number and unknown state are reported skips (never silent drops).
    assert!(
        report.skipped.iter().any(|s| matches!(s.reason, SkipReason::Unmappable(_))
            && s.reason.to_string().contains("missing `number`")),
        "missing-number reported: {:?}",
        report.skipped
    );
    assert!(
        report.skipped.iter().any(|s| s.reason.to_string().contains("unknown legacy state")),
        "unknown-state reported: {:?}",
        report.skipped
    );

    // The dangling-supersedes doc (#21) still imports, with a warning (loud, not
    // a silent drop of the supersession).
    let nodes = nodes_by_number(&store);
    assert!(nodes.contains_key(&21), "#21 imports despite its dangling edge");
    assert!(nodes[&21].frontmatter().edges().supersedes.is_none(), "no dangling edge written");
    assert!(
        report.warnings.iter().any(|w| w.contains("dangling")),
        "dangling supersession warned: {:?}",
        report.warnings
    );
}

#[test]
fn migrate_malformed_frontmatter_is_a_reported_skip() {
    // A file with no closing `---` fence cannot be parsed → a Malformed skip
    // (never a panic), and the run still completes cleanly.
    let legacy_dir = TempDir::new().unwrap();
    let sub = legacy_dir.path().join("01-draft");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("broken.md"), "---\nnumber: 1\nstate: Draft\nno closing fence\n")
        .unwrap();

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let report = migrate(&store, legacy_dir.path(), Mode::Commit).expect("completes");

    assert_eq!(report.created_count(), 0);
    assert_eq!(report.skipped_count(), 1);
    assert!(
        matches!(report.skipped[0].reason, SkipReason::Malformed(_)),
        "broken frontmatter → Malformed skip: {:?}",
        report.skipped
    );
}

// ----- V-4: migrate stamps imported nodes with the current schema -----------

#[test]
fn migrate_stamps_schema_v1() {
    use odm_core::schema::{SchemaMarker, SchemaVersion};

    let legacy_dir = TempDir::new().unwrap();
    copy_tree(&fixtures("legacy"), legacy_dir.path());
    let before = snapshot_bytes(legacy_dir.path());

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    migrate(&store, legacy_dir.path(), Mode::Commit).expect("migrate");

    // Every imported node is stamped the current schema (the v0.1 → current upgrade path).
    let nodes = store.load_all().unwrap();
    assert!(!nodes.is_empty());
    for doc in &nodes {
        assert_eq!(
            doc.frontmatter().schema(),
            Some(SchemaMarker::current(NodeType::Design)),
            "#{} stamped the current schema",
            doc.frontmatter().number()
        );
        assert_eq!(doc.frontmatter().schema_version(), SchemaVersion::CURRENT);
    }
    // The legacy source is understood as v0.1 (no schema) and left untouched.
    assert_eq!(before, snapshot_bytes(legacy_dir.path()), "legacy files byte-unchanged");
}

// ----- V-5: backfill stamps pre-existing unversioned nodes ------------------

#[test]
fn backfill_stamps_unversioned_nodes() {
    use chrono::NaiveDate;
    use odm_core::schema::SchemaMarker;
    use odm_core::{Id, Origin};

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let day = NaiveDate::from_ymd_opt(2026, 7, 6).unwrap();

    // Persist an unversioned node (as slice02 wrote before the schema field).
    let fm = odm_core::frontmatter::Frontmatter::new(
        Id::new(),
        7,
        NodeType::Design,
        "Unversioned",
        day,
        day,
        Origin::Planned,
    );
    assert!(fm.schema().is_none(), "sanity: starts unversioned");
    store.persist(&Document::new(fm, "body\n")).unwrap();

    // Backfill stamps it the current schema (`design/v1.1`).
    let upgraded = odm_migrate::backfill_schema(&store, Mode::Commit).expect("backfill");
    assert_eq!(upgraded.len(), 1, "one node stamped");
    assert_eq!(upgraded[0].schema, "design/v1.1");
    assert_eq!(
        store.load_all().unwrap()[0].frontmatter().schema(),
        Some(SchemaMarker::current(NodeType::Design)),
        "node now carries the current schema"
    );

    // Idempotent: a second backfill stamps nothing.
    let again = odm_migrate::backfill_schema(&store, Mode::Commit).expect("re-backfill");
    assert!(again.is_empty(), "re-run stamps nothing (idempotent)");

    // Dry-run over a fresh unversioned node writes nothing.
    let dry_dir = TempDir::new().unwrap();
    let dry_store = Store::open(dry_dir.path());
    let fm2 = odm_core::frontmatter::Frontmatter::new(
        Id::new(),
        8,
        NodeType::Design,
        "Unversioned2",
        day,
        day,
        Origin::Planned,
    );
    dry_store.persist(&Document::new(fm2, "body\n")).unwrap();
    let dry = odm_migrate::backfill_schema(&dry_store, Mode::DryRun).expect("dry backfill");
    assert_eq!(dry.len(), 1, "dry-run reports the would-stamp");
    assert!(
        dry_store.load_all().unwrap()[0].frontmatter().schema().is_none(),
        "dry-run wrote nothing"
    );
}

#[test]
fn migrate_folds_in_the_backfill() {
    use chrono::NaiveDate;
    use odm_core::{Id, Origin};

    // A store with a pre-existing unversioned node; migrate over an empty legacy
    // dir still backfills it (migrate is the single schema-upgrade entry point).
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let day = NaiveDate::from_ymd_opt(2026, 7, 6).unwrap();
    let fm = odm_core::frontmatter::Frontmatter::new(
        Id::new(),
        9,
        NodeType::Design,
        "Old",
        day,
        day,
        Origin::Planned,
    );
    store.persist(&Document::new(fm, "body\n")).unwrap();

    let empty_legacy = TempDir::new().unwrap();
    let report = migrate(&store, empty_legacy.path(), Mode::Commit).expect("migrate");
    assert_eq!(report.created_count(), 0, "nothing to import from an empty dir");
    assert_eq!(report.upgraded_count(), 1, "the unversioned node is backfilled");
    assert_eq!(
        store.load_all().unwrap()[0].frontmatter().schema_version(),
        odm_core::schema::SchemaVersion::CURRENT
    );
}

// ----- helpers --------------------------------------------------------------

/// Recursively copies `src` into `dst` (files + dirs).
fn copy_tree(src: &Path, dst: &Path) {
    for entry in walkdir_all(src) {
        let rel = entry.strip_prefix(src).unwrap();
        let target = dst.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&target).unwrap();
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::copy(&entry, &target).unwrap();
        }
    }
}

/// Every path under `root` (dirs + files), used by [`copy_tree`].
fn walkdir_all(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    if root.is_dir() {
        for entry in std::fs::read_dir(root).unwrap() {
            out.extend(walkdir_all(&entry.unwrap().path()));
        }
    }
    out
}

/// A sorted `(relative-path, bytes)` snapshot of every `.md` file under `root`.
fn snapshot_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out: Vec<(PathBuf, Vec<u8>)> = walkdir_all(root)
        .into_iter()
        .filter(|p| p.is_file())
        .map(|p| (p.strip_prefix(root).unwrap().to_path_buf(), std::fs::read(&p).unwrap()))
        .collect();
    out.sort();
    out
}
