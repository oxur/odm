//! Integration tests for the self-host cutover (arc06 slice04) over a **synthetic**
//! plan-set fixture (`test-data/plan-set/`) with deterministic closed/active/planned
//! states. Test names carry the substrings the ledger Verify commands filter on:
//! `selfhost_creates_work_nodes`, `selfhost_tree_matches_hierarchy`,
//! `selfhost_status_from_plan`, `selfhost_dry_run_writes_nothing`,
//! `selfhost_idempotent`.

use std::path::{Path, PathBuf};

use odm_core::NodeType;
use odm_core::frontmatter::Document;
use odm_core::schema::{SchemaMarker, SchemaVersion};
use odm_migrate::Mode;
use odm_migrate::selfhost::{arc_number, self_host, slice_number};
use odm_store::Store;
use tempfile::TempDir;

/// The synthetic plan-set fixture root.
fn plan_set() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/plan-set")
}

/// Loads every persisted node keyed by `(type, number)`.
fn nodes_by_key(store: &Store) -> std::collections::HashMap<(NodeType, u32), Document> {
    store
        .load_all()
        .expect("load_all")
        .into_iter()
        .map(|d| ((d.frontmatter().node_type(), d.frontmatter().number()), d))
        .collect()
}

// ----- S-1: plan-set → work nodes (schema-stamped); plan-set intact -----------

#[test]
fn selfhost_creates_work_nodes() {
    let legacy = TempDir::new().unwrap();
    copy_tree(&plan_set(), legacy.path());
    let before = snapshot_bytes(legacy.path());

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let report = self_host(&store, legacy.path(), Mode::Commit).expect("self-host");

    // project (1) + 2 in-scope arcs (arc07 excluded) + 4 slices = 7 work nodes.
    assert_eq!(report.created_count(), 7, "project + 2 arcs + 4 slices");
    let nodes = nodes_by_key(&store);
    assert_eq!(nodes[&(NodeType::Project, 1000)].frontmatter().name(), "Test Project — Plan");
    assert_eq!(
        nodes[&(NodeType::Arc, arc_number(1))].frontmatter().name(),
        "Arc 01 — Alpha (plan-of-record)"
    );
    // arc07 is out of MVP scope — never imported.
    assert!(!nodes.contains_key(&(NodeType::Arc, arc_number(7))), "arc07 excluded");

    // Every work node is schema-stamped `<type>/v1.0`.
    for ((ty, _), doc) in &nodes {
        assert_eq!(doc.frontmatter().schema(), Some(SchemaMarker::current(*ty)), "{ty:?} stamped");
        assert_eq!(doc.frontmatter().schema_version(), SchemaVersion::CURRENT);
    }

    // Never-delete: the plan-set Markdown is byte-for-byte intact.
    assert_eq!(before, snapshot_bytes(legacy.path()), "plan-set unchanged");
}

// ----- S-2: containment tree matches the directory hierarchy -----------------

#[test]
fn selfhost_tree_matches_hierarchy() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_set(), Mode::Commit).expect("self-host");
    let nodes = nodes_by_key(&store);

    let project_id = nodes[&(NodeType::Project, 1000)].frontmatter().id();
    let arc1_id = nodes[&(NodeType::Arc, arc_number(1))].frontmatter().id();

    // Project is the root (no part_of); every arc is part_of the project.
    assert!(
        nodes[&(NodeType::Project, 1000)].frontmatter().edges().part_of.is_none(),
        "project root"
    );
    for arc in [1, 2] {
        assert_eq!(
            nodes[&(NodeType::Arc, arc_number(arc))].frontmatter().edges().part_of,
            Some(project_id),
            "arc{arc} part_of project"
        );
    }
    // A slice is part_of its arc (arc01 slice01 = 1101 → arc01 = 1100).
    assert_eq!(
        nodes[&(NodeType::Slice, slice_number(1, 1, None))].frontmatter().edges().part_of,
        Some(arc1_id),
        "slice part_of arc"
    );
    // No orphan work nodes: every non-project work node has a resolvable parent.
    let ids: std::collections::HashSet<_> = nodes.values().map(|d| d.frontmatter().id()).collect();
    for ((ty, _), doc) in &nodes {
        if *ty != NodeType::Project {
            let parent = doc.frontmatter().edges().part_of.expect("parented");
            assert!(ids.contains(&parent), "parent resolves in corpus");
        }
    }
}

// ----- S-3: gate status reflects the plan (Asserted) -------------------------

#[test]
fn selfhost_status_from_plan() {
    use odm_core::status::Evidence;

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_set(), Mode::Commit).expect("self-host");
    let nodes = nodes_by_key(&store);

    // arc01 is closed (P-1 done) → the full arc sequence is reached (verified).
    let arc1 = nodes[&(NodeType::Arc, arc_number(1))].frontmatter();
    assert!(arc1.status().has_reached("verified"), "closed arc reaches terminal");
    assert_eq!(arc1.status().gate("verified").unwrap().evidence, Evidence::Asserted);

    // arc02 is active (P-2 open, no close file) → partial: in-progress, not complete.
    let arc2 = nodes[&(NodeType::Arc, arc_number(2))].frontmatter();
    assert!(arc2.status().has_reached("in-progress"), "active arc reached in-progress");
    assert!(!arc2.status().has_reached("complete"), "active arc not complete");

    // A complete slice (has closing-report) reaches its terminal gate (tested);
    // a planned slice (no closing-report) has reached nothing.
    let complete_slice = nodes[&(NodeType::Slice, slice_number(1, 1, None))].frontmatter();
    assert!(complete_slice.status().has_reached("tested"), "complete slice tested");
    let planned_slice = nodes[&(NodeType::Slice, slice_number(2, 2, None))].frontmatter();
    assert!(planned_slice.status().is_empty(), "planned slice has no gates");
}

// ----- S-6: dry-run writes nothing; idempotent re-run ------------------------

#[test]
fn selfhost_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let report = self_host(&store, &plan_set(), Mode::DryRun).expect("dry-run");
    assert!(report.dry_run);
    assert_eq!(report.created_count(), 7, "the plan still lists all nodes");
    assert!(store.load_all().unwrap().is_empty(), "dry-run wrote nothing");
    assert!(!store_dir.path().join("nodes").exists(), "no nodes/ created");
}

#[test]
fn selfhost_idempotent() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let first = self_host(&store, &plan_set(), Mode::Commit).expect("first");
    assert_eq!(first.created_count(), 7);
    let n = store.load_all().unwrap().len();

    // Re-run: every (type, number) already exists → 0 created, all skipped.
    let second = self_host(&store, &plan_set(), Mode::Commit).expect("second");
    assert_eq!(second.created_count(), 0, "idempotent re-run creates nothing");
    assert_eq!(second.skipped_count(), 7, "all skipped as already-existing");
    assert_eq!(store.load_all().unwrap().len(), n, "no duplicates on disk");
}

// ----- helpers ---------------------------------------------------------------

fn copy_tree(src: &Path, dst: &Path) {
    for entry in walk(src) {
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

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    if root.is_dir() {
        for entry in std::fs::read_dir(root).unwrap() {
            out.extend(walk(&entry.unwrap().path()));
        }
    }
    out
}

fn snapshot_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out: Vec<(PathBuf, Vec<u8>)> = walk(root)
        .into_iter()
        .filter(|p| p.is_file())
        .map(|p| (p.strip_prefix(root).unwrap().to_path_buf(), std::fs::read(&p).unwrap()))
        .collect();
    out.sort();
    out
}
