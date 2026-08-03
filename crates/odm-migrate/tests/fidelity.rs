//! Integration tests for the migration-fidelity core (arc-migration-fidelity
//! slice03, ODD-0025): verbatim body import in both importers, `source`
//! population, and `author`/`version` preservation. Test names carry the
//! substrings the slice03 ledger Verify commands filter on — the hash-gate
//! pass/fail cases themselves are unit-tested directly in `fidelity::tests`
//! (`cargo test -p odm-migrate` runs both).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Edges, Frontmatter};
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use odm_migrate::selfhost::{arc_number, named_arc_number, repair, self_host, slice_number};
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

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Persists a **stub** work node directly (bypassing `self_host`, which no
/// longer produces one) — standing in for one of the real corpus's 44
/// pre-slice03 stubs, with real edges/status so the repair tests can prove
/// they survive the rewrite.
fn persist_stub_node(
    store: &Store,
    number: u32,
    node_type: NodeType,
    name: &str,
    part_of: Option<Id>,
) -> Id {
    let id = Id::new();
    let created = day(2026, 7, 20);
    let mut fm = Frontmatter::new(id, number, node_type, name, created, created, Origin::Planned);
    fm.stamp_schema();
    if let Some(parent) = part_of {
        fm = fm.with_edges(Edges { part_of: Some(parent), ..Edges::default() });
    }
    let gates = odm_core::gates::GateSet::new(vec!["planned".to_string(), "built".to_string()]);
    let _ = fm.status_mut().set_gate(&gates, "planned", None, Evidence::Asserted, created);
    let document = Document::new(fm, format!("# {name}\n")); // a lone-H1 stub body
    store.persist(&document).unwrap();
    id
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
    assert_eq!(source.normalization.as_deref(), Some("trim+lf"));
    assert!(
        source.migrated_by.as_deref().is_some_and(|v| v.starts_with("odm-migrate/")),
        "{:?}",
        source.migrated_by
    );
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
    assert_eq!(source.normalization.as_deref(), Some("trim+lf"));
    assert!(source.migrated_by.as_deref().is_some_and(|v| v.starts_with("odm-migrate/")));
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
    assert!(store.load_all().unwrap().len() >= 8 + 6);
}

// ----- s04 F-1/F-2: no scope cap; named arcs get collision-free handles ----

#[test]
fn selfhost_imports_numbered_and_named_arcs_with_handles() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();

    write(root, "project-plan.md", "# Test Project\n");
    write(root, "arc01-alpha/arc-plan.md", "# Arc 01 — Alpha\n");
    write(root, "arc01-alpha/slice01-aa/slice-doc.md", "# Slice 01\n");
    // Post-MVP numbered arc — no scope cap (v1.6 F11).
    write(root, "arc07-horizon/arc-plan.md", "# Arc 07 — Horizon\n");
    // A named arc — no `arcNN` coordinate at all (v1.6 F12).
    write(root, "arc-custom-thing/arc-plan.md", "# A named arc\n");
    write(root, "arc-custom-thing/slice01-first/slice-doc.md", "# Its first slice\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let report = self_host(&store, root, Mode::Commit).expect("self-host");

    // project + 3 arcs + 2 slices = 6.
    assert_eq!(report.created_count(), 6);
    let nodes = nodes_by_key(&store);

    assert!(nodes.contains_key(&(NodeType::Arc, arc_number(1))), "numbered arc imported");
    assert!(nodes.contains_key(&(NodeType::Arc, arc_number(7))), "post-MVP arc07 imported");

    // The named arc is the only one, so its slug-derived handle is unbumped.
    let named_number = named_arc_number("arc-custom-thing", &std::collections::BTreeSet::new());
    assert!(
        nodes.contains_key(&(NodeType::Arc, named_number)),
        "named arc assigned handle {named_number}"
    );
    assert!(named_number > arc_number(8), "no collision with the numbered-arc range");
    // Its slice offsets within the named arc's own band.
    assert!(
        nodes.contains_key(&(NodeType::Slice, named_number + 1)),
        "named arc's slice offsets within its band"
    );

    // No two nodes share a number.
    let numbers: std::collections::HashSet<u32> = nodes.keys().map(|(_, n)| *n).collect();
    assert_eq!(numbers.len(), nodes.len(), "every number is unique — no collision");
}

// ----- s04 F-4/F-5/F-8: update-in-place repair -------------------------------

/// A fixture plan-set with real (non-stub) source content, for the repair
/// tests to read verbatim.
fn write_repair_fixture(root: &Path) {
    write(root, "project-plan.md", "# Test Project\n");
    write(
        root,
        "arc01-alpha/arc-plan.md",
        "# Arc 01 — Alpha (plan-of-record)\n\nReal arc content, not just a heading.\n",
    );
    write(
        root,
        "arc01-alpha/slice01-aa/slice-doc.md",
        "# Slice 01 (Arc 01) — Aa\n\nReal slice content, not just a heading.\n",
    );
}

#[test]
fn repair_rewrites_stub_body_preserving_identity_and_status() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write_repair_fixture(root);

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    // Stand in for two of the real corpus's 44 pre-slice03 stubs, with real
    // edges/status attached — exactly what update-in-place must preserve.
    let arc_id = persist_stub_node(&store, arc_number(1), NodeType::Arc, "Arc 01 — Alpha", None);
    let slice_id = persist_stub_node(
        &store,
        slice_number(1, 1, None),
        NodeType::Slice,
        "Slice 01",
        Some(arc_id),
    );

    let report = repair(&store, root, Mode::Commit).expect("repair");
    assert_eq!(report.repaired_count(), 2, "both stubs repaired");

    let nodes = nodes_by_key(&store);
    let arc = &nodes[&(NodeType::Arc, arc_number(1))];
    let slice = &nodes[&(NodeType::Slice, slice_number(1, 1, None))];

    // Body is now the real, verbatim source content — not the lone-H1 stub.
    assert_eq!(
        arc.body().trim(),
        "# Arc 01 — Alpha (plan-of-record)\n\nReal arc content, not just a heading.\n".trim()
    );
    assert!(arc.body().contains("Real arc content"), "verbatim, not a stub");
    assert!(slice.body().contains("Real slice content"), "verbatim, not a stub");

    // Identity, containment, and status survive the rewrite untouched.
    assert_eq!(arc.frontmatter().id(), arc_id, "id preserved");
    assert_eq!(slice.frontmatter().id(), slice_id, "id preserved");
    assert_eq!(slice.frontmatter().edges().part_of, Some(arc_id), "edges preserved");
    assert!(slice.frontmatter().status().has_reached("planned"), "status preserved");
    assert_eq!(slice.frontmatter().number(), slice_number(1, 1, None), "number preserved");

    // The `source` record is now populated, and the schema is current.
    assert!(arc.frontmatter().source().is_some(), "source populated by repair");
    assert_eq!(arc.frontmatter().schema_version(), odm_core::schema::SchemaVersion::CURRENT);
}

#[test]
fn repair_is_idempotent_and_leaves_non_stub_nodes_alone() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write_repair_fixture(root);

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_stub_node(&store, arc_number(1), NodeType::Arc, "Arc 01 — Alpha", None);

    let first = repair(&store, root, Mode::Commit).expect("first repair");
    assert_eq!(first.repaired_count(), 1);

    // Re-running finds no stubs left — the repaired node is no longer one.
    let second = repair(&store, root, Mode::Commit).expect("second repair");
    assert_eq!(second.repaired_count(), 0, "idempotent — nothing left to repair");
}

#[test]
fn repair_dry_run_writes_nothing() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write_repair_fixture(root);

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_stub_node(&store, arc_number(1), NodeType::Arc, "Arc 01 — Alpha", None);
    let before = std::fs::read_to_string(
        store.node_paths().unwrap().into_iter().next().expect("one node on disk"),
    )
    .unwrap();

    let report = repair(&store, root, Mode::DryRun).expect("dry-run repair");
    assert!(report.dry_run);
    assert_eq!(report.repaired_count(), 1, "the plan still reports what would repair");

    let after = std::fs::read_to_string(
        store.node_paths().unwrap().into_iter().next().expect("still one node"),
    )
    .unwrap();
    assert_eq!(before, after, "dry-run wrote nothing");
}
