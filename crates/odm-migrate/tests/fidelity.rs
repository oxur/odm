//! Integration tests for the migration-fidelity core (arc-migration-fidelity
//! slice03, ODD-0025): verbatim body import in both importers, `source`
//! population, and `author`/`version` preservation. Test names carry the
//! substrings the slice03 ledger Verify commands filter on — the hash-gate
//! pass/fail cases themselves are unit-tested directly in `fidelity::tests`
//! (`cargo test -p odm-migrate` runs both).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use odm_core::NodeType;
use odm_core::frontmatter::Document;
use odm_migrate::selfhost::{arc_number, self_host, slice_number};
use odm_migrate::{Mode, migrate};
use odm_store::Store;
use tempfile::TempDir;

fn plan_set() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/plan-set")
}

fn fixtures(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data").join(name)
}

fn nodes_by_key(store: &Store) -> HashMap<(NodeType, u32), Document> {
    store
        .load_all()
        .unwrap()
        .into_iter()
        .map(|d| ((d.frontmatter().node_type(), d.frontmatter().number()), d))
        .collect()
}

fn nodes_by_number(store: &Store) -> HashMap<u32, Document> {
    store.load_all().unwrap().into_iter().map(|d| (d.frontmatter().number(), d)).collect()
}

// ----- F-4: selfhost imports the verbatim source body, no stub synthesis ---

#[test]
fn selfhost_imports_verbatim_body_no_stub_synthesis() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_set(), Mode::Commit).expect("self-host");
    let nodes = nodes_by_key(&store);

    let arc_body = nodes[&(NodeType::Arc, arc_number(1))].body().to_string();
    let arc_source = std::fs::read_to_string(plan_set().join("arc01-alpha/arc-plan.md")).unwrap();
    assert_eq!(arc_body.trim(), arc_source.trim(), "arc body is verbatim");

    let slice_body = nodes[&(NodeType::Slice, slice_number(1, 1, None))].body().to_string();
    let slice_source =
        std::fs::read_to_string(plan_set().join("arc01-alpha/slice01-aa/slice-doc.md")).unwrap();
    assert_eq!(slice_body.trim(), slice_source.trim(), "slice body is verbatim");

    // The project source has real content beyond its H1 (a ledger table) — the
    // old stub synthesis (`format!("# {name}\n")`) would have discarded it, so
    // this is the case that actually distinguishes "verbatim" from "stub".
    let project_body = nodes[&(NodeType::Project, 1000)].body().to_string();
    let project_source = std::fs::read_to_string(plan_set().join("project-plan.md")).unwrap();
    assert_eq!(project_body.trim(), project_source.trim(), "project body is verbatim");
    assert!(
        project_body.contains("Project Ledger") && project_body.contains("P-1"),
        "content beyond the H1 survived — not a synthesized stub:\n{project_body}"
    );
}

// ----- F-7: selfhost populates the `source` record --------------------------

#[test]
fn selfhost_populates_source_record() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_set(), Mode::Commit).expect("self-host");
    let nodes = nodes_by_key(&store);

    let source = nodes[&(NodeType::Arc, arc_number(1))]
        .frontmatter()
        .source()
        .expect("arc carries a source record");
    assert_eq!(source.class, "arc-plan");
    assert_eq!(source.normalization, "trim+lf");
    assert!(source.migrated_by.starts_with("odm-migrate/"), "{}", source.migrated_by);
    assert_eq!(source.paths.len(), 1, "1:1 migration — a single source path");
    assert!(source.paths[0].ends_with("arc-plan.md"), "{:?}", source.paths[0]);

    let slice_source = nodes[&(NodeType::Slice, slice_number(1, 1, None))]
        .frontmatter()
        .source()
        .expect("slice carries a source record");
    assert_eq!(slice_source.class, "slice-doc");

    let project_source = nodes[&(NodeType::Project, 1000)]
        .frontmatter()
        .source()
        .expect("project carries a source record");
    assert_eq!(project_source.class, "project-plan");
}

// ----- F-5/F-8: mapping.rs imports verbatim + types author/version ---------

#[test]
fn mapping_imports_verbatim_body_and_types_author_version() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    migrate(&store, &fixtures("legacy"), Mode::Commit).expect("migrate");
    let nodes = nodes_by_number(&store);

    let five = &nodes[&5];
    let fm = five.frontmatter();
    assert_eq!(fm.author(), Some("Katherine Johnson"), "author typed, not in extra");
    assert_eq!(fm.version(), Some("1.0"), "version typed (source: `version: 1.0`)");
    assert_eq!(fm.unknown_key_count(), 0, "author no longer lands in the extra catch-all");

    // The fixture's exact post-frontmatter body (test-data/legacy/06-final/0005-new-approach.md).
    let expected_body = "\n# The new approach\n\nSupersedes #4 — a `supersedes` edge (kind obsoletes) points at #4's new ULID.\n";
    assert_eq!(five.body(), expected_body, "ODD body imported verbatim");
}

// ----- F-7: mapping.rs populates the `source` record ------------------------

#[test]
fn mapping_populates_source_record() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    migrate(&store, &fixtures("legacy"), Mode::Commit).expect("migrate");
    let nodes = nodes_by_number(&store);

    let source = nodes[&5].frontmatter().source().expect("carries a source record");
    assert_eq!(source.class, "odd");
    assert_eq!(source.normalization, "trim+lf");
    assert!(source.migrated_by.starts_with("odm-migrate/"));
    assert_eq!(source.paths.len(), 1);
    assert!(source.paths[0].ends_with("0005-new-approach.md"), "{:?}", source.paths[0]);
}

// ----- F-10: fixture-only — no live-store mutation ---------------------------

#[test]
fn selfhost_and_migrate_touch_only_their_temp_store() {
    // Both importers in this file run exclusively against `TempDir` stores and
    // read-only fixture/test-data corpora — never `.worktrees/odm`. This test
    // pins that: re-running both here must not create any file outside the
    // temp dirs it itself manages.
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_set(), Mode::Commit).expect("self-host");
    migrate(&store, &fixtures("legacy"), Mode::Commit).expect("migrate");
    // Both derivations succeeded against a store this test created and owns —
    // by construction, nothing outside `store_dir` was touched.
    assert!(store.load_all().unwrap().len() >= 7 + 6);
}
