//! Integration tests for source-path portability (arc-migration-fidelity
//! slice08, ODD-0025 §2.0/§2.2, CDC v2.8/v2.9 findings). Test names carry the
//! ledger Verify substrings: `selfhost_stores_relative_canonical_source_paths`
//! (F-1), `selfhost_source_path_is_invariant_to_arg_spelling` (F-2),
//! `repair_reads_the_stub_source_via_anchor_relative_resolve` (F-3),
//! `selfhost_transition_rewrites_absolute_source_paths_to_relative` (F-4),
//! `selfhost_cross_checkout_determinism` (F-5),
//! `coverage_set_difference_stable_across_roots` (F-6).

use std::path::Path;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter, Source};
use odm_core::{Id, NodeType, Origin};
use odm_migrate::Mode;
use odm_migrate::coverage;
use odm_migrate::migrate;
use odm_migrate::selfhost::{arc_number, repair, self_host, slice_number};
use odm_store::Store;
use tempfile::TempDir;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, 20).unwrap()
}

/// A plan-set fixture with a `.git` marker at its root — the realistic shape
/// (every real checkout has one), since s08's anchor is the git toplevel, not
/// just "the plan root" a caller happens to pass.
fn write_plan_set_with_git(root: &Path) {
    std::fs::write(root.join(".git"), "gitdir: fake\n").unwrap();
    write(root, "docs/design-v1.0.0/project-plan.md", "# Test Project\n");
    write(
        root,
        "docs/design-v1.0.0/arc01-alpha/arc-plan.md",
        "# Arc 01 — Alpha\n\nReal content.\n",
    );
    write(
        root,
        "docs/design-v1.0.0/arc01-alpha/slice01-aa/slice-doc.md",
        "# Slice 01\n\nReal content.\n",
    );
}

fn nodes_by_key(store: &Store) -> std::collections::HashMap<(NodeType, u32), Document> {
    store
        .load_all()
        .unwrap()
        .into_iter()
        .map(|d| ((d.frontmatter().node_type(), d.frontmatter().number()), d))
        .collect()
}

/// Persists a node with an already-populated `source` record at exactly the
/// given `source_path` — standing in for a node whose stored form is
/// whatever the caller wants to test transitioning away from (e.g. absolute,
/// the s07 shape).
fn persist_source_bearing_node(
    store: &Store,
    number: u32,
    node_type: NodeType,
    name: &str,
    body: &str,
    source_path: &Path,
    class: &str,
) -> Id {
    let id = Id::new();
    let mut fm = Frontmatter::new(id, number, node_type, name, day(), day(), Origin::Planned);
    fm.stamp_schema();
    let fm = fm.with_source(Source {
        paths: vec![source_path.to_path_buf()],
        class: class.to_string(),
        normalization: "trim+lf".to_string(),
        migrated_by: "odm-migrate/test".to_string(),
        migrated_on: day(),
    });
    let document = Document::new(fm, body.to_string());
    store.persist(&document).unwrap();
    id
}

// ----- F-1: relative-to-content-root storage --------------------------------

#[test]
fn selfhost_stores_relative_canonical_source_paths() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path());
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_root, Mode::Commit).expect("self-host");

    let nodes = nodes_by_key(&store);
    let arc = &nodes[&(NodeType::Arc, arc_number(1))];
    let source = arc.frontmatter().source().expect("source present");
    assert_eq!(source.paths.len(), 1);
    let stored = source.paths[0].to_string_lossy();
    assert_eq!(stored, "docs/design-v1.0.0/arc01-alpha/arc-plan.md", "{stored}");
    assert!(!stored.starts_with('/'), "{stored}");
    assert!(!stored.contains(".worktrees"), "{stored}");

    let slice = &nodes[&(NodeType::Slice, slice_number(1, 1, None))];
    let slice_stored =
        slice.frontmatter().source().unwrap().paths[0].to_string_lossy().into_owned();
    assert_eq!(slice_stored, "docs/design-v1.0.0/arc01-alpha/slice01-aa/slice-doc.md");
}

// ----- F-2: canonical form invariant to plan-root arg spelling --------------

#[test]
fn selfhost_source_path_is_invariant_to_arg_spelling() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path());
    let plan_root = repo.path().join("docs/design-v1.0.0");

    // Absolute (as-is), a trailing `.` (mimics a trailing-slash-ish
    // decoration), and an argument that routes through an internal `..` —
    // three different spellings of the same directory.
    let spellings: [std::path::PathBuf; 3] =
        [plan_root.clone(), plan_root.join("."), plan_root.join("../design-v1.0.0")];

    let mut stored_forms = Vec::new();
    for spelling in &spellings {
        let store_dir = TempDir::new().unwrap();
        let store = Store::open(store_dir.path());
        self_host(&store, spelling, Mode::Commit).expect("self-host");
        let nodes = nodes_by_key(&store);
        let arc = &nodes[&(NodeType::Arc, arc_number(1))];
        stored_forms.push(arc.frontmatter().source().unwrap().paths[0].clone());
    }

    assert!(
        stored_forms.windows(2).all(|w| w[0] == w[1]),
        "identical canonical form regardless of arg spelling: {stored_forms:?}"
    );
}

// ----- F-3: the stub source is read via anchor + relative resolve ----------

#[test]
fn repair_reads_the_stub_source_via_anchor_relative_resolve() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path());
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A pre-existing stub (no source yet) — `repair` must resolve its source
    // file via the anchor+relative round-trip (s08 F-3), not a path baked in
    // at construction time (there is none here — the point).
    let id = persist_source_bearing_node(
        &store,
        arc_number(1),
        NodeType::Arc,
        "Arc 01 — Alpha",
        "# stub\n",
        Path::new("unused"), // overwritten below — this node has NO source yet
        "arc-plan",
    );
    // Rewrite it sourceless, simulating the real "stub, no source" shape
    // `repair` actually targets (the helper above always sets one).
    let mut doc = store.load(id).unwrap();
    let mut fm = doc.frontmatter().clone();
    fm = Frontmatter::new(
        fm.id(),
        fm.number(),
        fm.node_type(),
        fm.name(),
        fm.created(),
        fm.updated(),
        fm.origin(),
    );
    fm.stamp_schema();
    doc = Document::new(fm, "# stub\n".to_string());
    store.persist(&doc).unwrap();

    let report = repair(&store, &plan_root, Mode::Commit).expect("repair");
    assert_eq!(report.repaired_count(), 1);

    let nodes = nodes_by_key(&store);
    let arc = &nodes[&(NodeType::Arc, arc_number(1))];
    assert!(
        arc.body().contains("Real content"),
        "verbatim body read via the anchor: {}",
        arc.body()
    );
    let stored = arc.frontmatter().source().unwrap().paths[0].to_string_lossy().into_owned();
    assert_eq!(stored, "docs/design-v1.0.0/arc01-alpha/arc-plan.md");
}

// ----- F-4: the transition rewrites an absolute stored path, 0 re-mint -----

#[test]
fn selfhost_transition_rewrites_absolute_source_paths_to_relative() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path());
    let plan_root = repo.path().join("docs/design-v1.0.0");
    let canonical_plan_root = plan_root.canonicalize().unwrap();

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    // Seed a node with an ABSOLUTE stored `source.paths` entry — the s07
    // shape this slice's live rewrite corrects.
    let absolute_source = canonical_plan_root.join("arc01-alpha/arc-plan.md");
    let id = persist_source_bearing_node(
        &store,
        arc_number(1),
        NodeType::Arc,
        "Arc 01 — Alpha",
        "# Arc 01 — Alpha\n\nReal content.\n",
        &absolute_source,
        "arc-plan",
    );

    let report = self_host(&store, &plan_root, Mode::Commit).expect("self-host");

    // The re-mint guard: arc01 is matched (by the canonical key derived from
    // its absolute stored path) and rewritten, never re-created.
    assert_eq!(
        report.created.iter().filter(|c| c.node_type == NodeType::Arc).count(),
        0,
        "arc01 was matched, not re-created: {:?}",
        report.created
    );
    assert_eq!(report.rewritten_count(), 1, "exactly the one absolute-path node rewritten");

    let nodes = nodes_by_key(&store);
    let arc = &nodes[&(NodeType::Arc, arc_number(1))];
    assert_eq!(arc.frontmatter().id(), id, "same node, not recreated");
    let stored = arc.frontmatter().source().unwrap().paths[0].to_string_lossy().into_owned();
    assert_eq!(
        stored, "docs/design-v1.0.0/arc01-alpha/arc-plan.md",
        "rewritten to relative+canonical"
    );

    // Idempotent: a second run finds nothing left to rewrite.
    let second = self_host(&store, &plan_root, Mode::Commit).expect("second self-host");
    assert_eq!(second.rewritten_count(), 0, "already canonical — nothing left to rewrite");
}

// ----- F-5: cross-checkout determinism (the point) --------------------------

#[test]
fn selfhost_cross_checkout_determinism() {
    let repo_a = TempDir::new().unwrap();
    write_plan_set_with_git(repo_a.path());
    let repo_b = TempDir::new().unwrap();
    write_plan_set_with_git(repo_b.path());

    let store_dir_a = TempDir::new().unwrap();
    let store_a = Store::open(store_dir_a.path());
    self_host(&store_a, &repo_a.path().join("docs/design-v1.0.0"), Mode::Commit).unwrap();

    let store_dir_b = TempDir::new().unwrap();
    let store_b = Store::open(store_dir_b.path());
    self_host(&store_b, &repo_b.path().join("docs/design-v1.0.0"), Mode::Commit).unwrap();

    let nodes_a = nodes_by_key(&store_a);
    let nodes_b = nodes_by_key(&store_b);
    let source_a =
        nodes_a[&(NodeType::Arc, arc_number(1))].frontmatter().source().unwrap().paths.clone();
    let source_b =
        nodes_b[&(NodeType::Arc, arc_number(1))].frontmatter().source().unwrap().paths.clone();
    assert_eq!(
        source_a, source_b,
        "identical source.paths regardless of the two TempDirs' different absolute roots"
    );
    assert_eq!(
        source_a,
        vec![std::path::PathBuf::from("docs/design-v1.0.0/arc01-alpha/arc-plan.md")]
    );
}

// ----- F-6: coverage set-difference stable across roots --------------------

#[test]
fn coverage_set_difference_stable_across_roots() {
    let repo_a = TempDir::new().unwrap();
    write_plan_set_with_git(repo_a.path());
    let repo_b = TempDir::new().unwrap();
    write_plan_set_with_git(repo_b.path());

    let store_dir_a = TempDir::new().unwrap();
    let store_a = Store::open(store_dir_a.path());
    self_host(&store_a, &repo_a.path().join("docs/design-v1.0.0"), Mode::Commit).unwrap();
    let report_a = coverage::run(&store_a, &repo_a.path().join("docs")).unwrap();

    let store_dir_b = TempDir::new().unwrap();
    let store_b = Store::open(store_dir_b.path());
    self_host(&store_b, &repo_b.path().join("docs/design-v1.0.0"), Mode::Commit).unwrap();
    let report_b = coverage::run(&store_b, &repo_b.path().join("docs")).unwrap();

    assert_eq!(report_a.doc_coverage.uncovered_count(), 0, "root A: 0 uncovered");
    assert_eq!(report_b.doc_coverage.uncovered_count(), 0, "root B: 0 uncovered");
    assert_eq!(
        report_a.doc_coverage.total(),
        report_b.doc_coverage.total(),
        "identical inventory size regardless of absolute root"
    );
}

// ----- s10 iteration 1: the design/ODD import seam relativizes source.paths -

/// Regression test for the seam CDC found missing (s10 iteration 1): the
/// legacy ODD importer (`mapping::build_node`) stored `source_path` verbatim
/// instead of routing it through `fidelity::relativize` like every other
/// creation path (`artifact.rs`, `notes.rs`, `mapping::backfill_source`) —
/// the exact CDC v2.8 Finding 1 bug, reintroduced on this one seam.
#[test]
fn migrate_stores_relative_canonical_source_paths() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    let legacy_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/legacy");
    migrate(&store, &legacy_path, Mode::Commit).expect("migrate");

    let nodes = store.load_all().expect("load_all");
    let five = nodes.iter().find(|d| d.frontmatter().number() == 5).expect("legacy #5 imported");
    let source = five.frontmatter().source().expect("source present");
    assert_eq!(source.paths.len(), 1);
    let stored = source.paths[0].to_string_lossy().into_owned();

    assert_eq!(stored, "test-data/legacy/06-final/0005-new-approach.md", "{stored}");
    assert!(!stored.starts_with('/'), "{stored}");
    assert!(!Path::new(&stored).is_absolute(), "{stored}");
    assert!(!stored.contains(".worktrees"), "{stored}");
}
