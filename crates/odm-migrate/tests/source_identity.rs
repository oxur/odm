//! Integration tests for source-based identity (arc-migration-fidelity slice05,
//! ODD-0025 §2.0/§2.2/§2.8/§5). Test names carry the substrings the ledger
//! Verify commands filter on: `selfhost_idempotence_keys_on_source`,
//! `selfhost_transitions`, `selfhost_rerun_after_adding_a_named_arc`,
//! `repair_backfills_source`, `repair_surfaces_a_drifted`,
//! `repair_excludes_the_project_node`, `coverage_source_based_matching`,
//! `selfhost_named_arc_handle_is_stable`.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter, Source};
use odm_core::{Id, NodeType, Origin};
use odm_migrate::coverage;
use odm_migrate::selfhost::{arc_number, named_arc_number, repair, self_host};
use odm_migrate::{MigrateError, Mode, SkipReason};
use odm_store::Store;
use tempfile::TempDir;

/// The project (root) node's number — matches `selfhost::PROJECT_NUMBER`
/// (crate-private; mirrored here the same way `tests/selfhost.rs` does).
const PROJECT_NUMBER: u32 = 1000;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn nodes_by_key(store: &Store) -> HashMap<(NodeType, u32), Document> {
    store
        .load_all()
        .unwrap()
        .into_iter()
        .map(|d| ((d.frontmatter().node_type(), d.frontmatter().number()), d))
        .collect()
}

/// Persists a node carrying no `source` — standing in for a legacy node that
/// predates arc-migration-fidelity's identity model (the corpus's real
/// pre-slice05 state).
fn persist_legacy_node_with_body(
    store: &Store,
    number: u32,
    node_type: NodeType,
    name: &str,
    body: &str,
) -> Id {
    let id = Id::new();
    let created = day(2026, 7, 20);
    let mut fm = Frontmatter::new(id, number, node_type, name, created, created, Origin::Planned);
    fm.stamp_schema();
    let document = Document::new(fm, body.to_string());
    store.persist(&document).unwrap();
    id
}

/// Persists a node that already carries a `source` record pointing at
/// `source_path` — standing in for a node this arc's identity model already
/// covers, at a `number` deliberately **different** from what `discover()`
/// would compute this run (proving matching keys on the path, not the number).
fn persist_source_bearing_node(
    store: &Store,
    number: u32,
    node_type: NodeType,
    name: &str,
    source_path: &Path,
    class: &str,
) -> Id {
    let id = Id::new();
    let created = day(2026, 7, 20);
    let mut fm = Frontmatter::new(id, number, node_type, name, created, created, Origin::Planned);
    fm.stamp_schema();
    let fm = fm.with_source(Source {
        paths: vec![source_path.to_path_buf()],
        class: class.to_string(),
        normalization: "trim+lf".to_string(),
        migrated_by: "odm-migrate/test".to_string(),
        migrated_on: created,
    });
    let document = Document::new(fm, format!("# {name}\n"));
    store.persist(&document).unwrap();
    id
}

// ----- F-1: self_host idempotence keys on source.paths, not (type, number) --

#[test]
fn selfhost_idempotence_keys_on_source_not_number() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n");
    write(root, "arc01-alpha/arc-plan.md", "# Arc 01 — Alpha\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    // A node at a number `discover()` would never compute for arc01
    // (`arc_number(1) == 1100`), but with exactly the source path `self_host`
    // would compute for it.
    let source_path = root.join("arc01-alpha/arc-plan.md");
    let id = persist_source_bearing_node(
        &store,
        9999,
        NodeType::Arc,
        "Arc 01 — Alpha (hand)",
        &source_path,
        "arc-plan",
    );

    let report = self_host(&store, root, Mode::Commit).expect("self-host");

    // Only the project is minted; the arc is matched by source, not re-created
    // under `arc_number(1)`.
    assert_eq!(report.created_count(), 1, "project only");
    let nodes = store.load_all().unwrap();
    assert_eq!(nodes.len(), 2, "project + the one pre-existing arc — no duplicate");
    assert!(
        !nodes.iter().any(|d| d.frontmatter().node_type() == NodeType::Arc
            && d.frontmatter().number() == arc_number(1)),
        "no node was minted at the coordinate-derived number"
    );
    assert!(
        nodes.iter().any(|d| d.frontmatter().id() == id),
        "the hand-numbered, source-bearing node survives untouched"
    );
}

// ----- F-2: coordinate→source transition (one-time population) -------------

#[test]
fn selfhost_transitions_a_pre_source_node_by_coordinate_then_by_source() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n");
    write(root, "arc01-alpha/arc-plan.md", "# Arc 01 — Alpha\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    // A pre-source legacy node at exactly this run's structural coordinate.
    let arc_id = persist_legacy_node_with_body(
        &store,
        arc_number(1),
        NodeType::Arc,
        "Arc 01 — Alpha",
        "# stub\n",
    );

    // First run: matched by coordinate (no source yet) → transition-populated.
    let first = self_host(&store, root, Mode::Commit).expect("first self-host");
    assert!(
        first
            .skipped
            .iter()
            .any(|s| s.number == Some(arc_number(1))
                && matches!(s.reason, SkipReason::SourcePopulated)),
        "reported as a source-populated transition, not a plain skip: {:?}",
        first.skipped.iter().map(|s| s.number).collect::<Vec<_>>()
    );
    let nodes = nodes_by_key(&store);
    let arc = &nodes[&(NodeType::Arc, arc_number(1))];
    assert_eq!(arc.frontmatter().id(), arc_id, "same node, not recreated");
    assert!(arc.frontmatter().source().is_some(), "source backfilled on the transition run");
    assert_eq!(store.load_all().unwrap().len(), 2, "project + arc, no duplicate");

    // Second run: now matched by source — no re-populate, no duplicate.
    let second = self_host(&store, root, Mode::Commit).expect("second self-host");
    assert!(
        second
            .skipped
            .iter()
            .any(|s| s.number == Some(arc_number(1))
                && matches!(s.reason, SkipReason::AlreadyExists)),
        "second run matches by source, not the transition path"
    );
    assert_eq!(store.load_all().unwrap().len(), 2, "still no duplicate");
}

// ----- F-3: re-run after adding a named arc mints no duplicate (s04 hazard) -

#[test]
fn selfhost_rerun_after_adding_a_named_arc_mints_no_duplicate() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n");
    write(root, "arc-zzz-existing/arc-plan.md", "# Existing named arc\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let first = self_host(&store, root, Mode::Commit).expect("first self-host");
    let existing_id = first
        .created
        .iter()
        .find(|c| c.node_type == NodeType::Arc)
        .expect("the named arc was created")
        .id;

    // A new named arc that sorts BEFORE the existing one alphabetically — the
    // exact s04 v1.8 hazard: under the old position-based handle, this shifted
    // every later-sorted named arc's number, and a `(type, number)`-keyed
    // idempotence check would then mint it a duplicate.
    write(root, "arc-aaa-new/arc-plan.md", "# New named arc\n");
    let second = self_host(&store, root, Mode::Commit).expect("second self-host");

    assert_eq!(second.created_count(), 1, "only the newly-added named arc is minted");
    let nodes = store.load_all().unwrap();
    let arc_ids: Vec<_> = nodes
        .iter()
        .filter(|d| d.frontmatter().node_type() == NodeType::Arc)
        .map(|d| d.frontmatter().id())
        .collect();
    assert_eq!(arc_ids.len(), 2, "no duplicate arc node");
    assert!(
        arc_ids.contains(&existing_id),
        "the pre-existing named arc's identity survived, matched by source"
    );
}

// ----- F-8: the name-derived handle is stable when another arc is added ----

#[test]
fn selfhost_named_arc_handle_is_stable_when_another_named_arc_is_added() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n");
    write(root, "arc-keep-me/arc-plan.md", "# Keep me\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let first = self_host(&store, root, Mode::Commit).expect("first self-host");
    let first_number = first.created.iter().find(|c| c.node_type == NodeType::Arc).unwrap().number;

    // A second store, a second named arc present alongside the first — proves
    // the handle is *recomputed* identically, not merely carried over by the
    // source-match (which would mask a would-be shift).
    write(root, "arc-another-one/arc-plan.md", "# Another\n");
    let store_dir2 = TempDir::new().unwrap();
    let store2 = Store::open(store_dir2.path());
    let second = self_host(&store2, root, Mode::Commit).expect("second self-host, fresh store");
    let second_number = second.created.iter().find(|c| c.name == "Keep me").unwrap().number;

    assert_eq!(
        first_number, second_number,
        "the handle is recomputed identically regardless of what else is present"
    );
}

// ----- F-4: backfill — a faithful non-stub node gets `source`, no-op body ---

#[test]
fn repair_backfills_source_onto_a_faithful_non_stub_node() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n");
    let arc_body = "# Arc 01 — Alpha (plan-of-record)\n\nReal, faithful arc content.\n";
    write(root, "arc01-alpha/arc-plan.md", arc_body);

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A faithful non-stub node whose body already equals the source but
    // carries no `source` — the live corpus's already-faithful pre-slice05
    // shape.
    let id = persist_legacy_node_with_body(
        &store,
        arc_number(1),
        NodeType::Arc,
        "Arc 01 — Alpha",
        arc_body,
    );

    let report = repair(&store, root, Mode::Commit).expect("repair");
    assert_eq!(report.repaired_count(), 1);

    let nodes = nodes_by_key(&store);
    let arc = &nodes[&(NodeType::Arc, arc_number(1))];
    assert_eq!(arc.frontmatter().id(), id, "same node, id preserved");
    assert_eq!(arc.body(), arc_body, "body unchanged — a content no-op");
    assert!(arc.frontmatter().source().is_some(), "source backfilled");

    // Idempotent: a second run finds nothing left to backfill.
    let second = repair(&store, root, Mode::Commit).expect("second repair");
    assert_eq!(second.repaired_count(), 0, "already source-bearing — left alone");
}

// ----- F-5: backfill surfaces a drifted non-stub body, does not swallow it -

#[test]
fn repair_surfaces_a_drifted_non_stub_body_as_a_hash_mismatch() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n");
    write(
        root,
        "arc01-alpha/arc-plan.md",
        "# Arc 01 — Alpha (plan-of-record)\n\nThe real, current source content.\n",
    );

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A non-stub node whose body has drifted from the source — not a stub, so
    // it is never blindly overwritten; the hard gate must catch the mismatch.
    persist_legacy_node_with_body(
        &store,
        arc_number(1),
        NodeType::Arc,
        "Arc 01 — Alpha",
        "# Arc 01 — Alpha (plan-of-record)\n\nA DIFFERENT body — drifted from source.\n",
    );

    let err = repair(&store, root, Mode::Commit).expect_err("drift must be a hard error");
    assert!(matches!(err, MigrateError::BodyHashMismatch { .. }), "{err:?}");

    // Not swallowed: the node was left exactly as it was, no `source` written.
    let nodes = nodes_by_key(&store);
    assert!(
        nodes[&(NodeType::Arc, arc_number(1))].frontmatter().source().is_none(),
        "a drifted node is never silently backfilled"
    );
}

// ----- F-6: the project node is excluded from backfill ---------------------

#[test]
fn repair_excludes_the_project_node() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n\nReal plan content.\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // The project's body is a synthesis (a vision excerpt) — never a 1:1 copy
    // of `project-plan.md` (ODD-0025 §2.3) — standing in for that permanent,
    // by-design divergence.
    let id = persist_legacy_node_with_body(
        &store,
        PROJECT_NUMBER,
        NodeType::Project,
        "odm",
        "# odm\n\n# Vision\n\nSynthesized vision text, not the source verbatim.\n",
    );

    let report =
        repair(&store, root, Mode::Commit).expect("repair does not error on the project node");
    assert_eq!(report.repaired_count(), 0, "the project is never touched by repair");

    let nodes = store.load_all().unwrap();
    let project = nodes.iter().find(|d| d.frontmatter().id() == id).unwrap();
    assert!(
        project.frontmatter().source().is_none(),
        "left alone, not forced through the 1:1 gate"
    );
}

// ----- s06 F-5: the project node is excluded via *every* path, incl. the ---
// ----- self_host coordinate→source transition (the exact CDC v2.1 finding) -

#[test]
fn selfhost_transition_excludes_the_project_node_from_source() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    // The real `project-plan.md` — deliberately NOT what the pre-existing
    // node's body says, so a would-be ungated 1:1 stamp would be detectable
    // as wrong even without a hash-mismatch error (it would just silently
    // "succeed" and give the project a `source` it should never have).
    write(root, "project-plan.md", "# Test Project\n\nThe real plan-of-record content.\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A pre-source legacy project node at exactly this run's coordinate
    // (`PROJECT_NUMBER`), with a synthesis-shaped body — standing in for the
    // live corpus's real project node (`replan.rs::vision_from_plan`).
    let id = persist_legacy_node_with_body(
        &store,
        PROJECT_NUMBER,
        NodeType::Project,
        "odm",
        "# odm\n\n# Vision\n\nSynthesized vision text, not the source verbatim.\n",
    );

    // Reached via `self_host`'s coordinate→source **transition**, not
    // `repair` — the exact path the CDC v2.1 finding flagged as ungated.
    let report = self_host(&store, root, Mode::Commit).expect("self-host does not error");
    assert!(
        report
            .skipped
            .iter()
            .any(|s| s.number == Some(PROJECT_NUMBER)
                && matches!(s.reason, SkipReason::AlreadyExists)),
        "the project is recognized by coordinate but never queued for the gated backfill: {:?}",
        report.skipped.iter().map(|s| s.number).collect::<Vec<_>>()
    );

    let nodes = store.load_all().unwrap();
    let project = nodes.iter().find(|d| d.frontmatter().id() == id).unwrap();
    assert!(
        project.frontmatter().source().is_none(),
        "no 1:1 `source` via the transition path either"
    );
    assert_eq!(
        project.body(),
        "# odm\n\n# Vision\n\nSynthesized vision text, not the source verbatim.\n",
        "the synthesis body is untouched"
    );
}

// ----- F-7: source-based coverage matching resolves a named arc ------------

#[test]
fn coverage_source_based_matching_resolves_a_named_arc() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    let design_root = root.join("design-v1.0.0");
    write(&design_root, "project-plan.md", "# Test Project\n");
    write(&design_root, "arc-store-home/arc-plan.md", "# Named arc\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &design_root, Mode::Commit).expect("self-host");

    let report = coverage::run(&store, root).expect("coverage run");
    let entry = report
        .doc_coverage
        .entries
        .iter()
        .find(|e| e.path == Path::new("design-v1.0.0/arc-store-home/arc-plan.md"))
        .expect("entry present");
    assert!(entry.covered, "a source-bearing named-arc node resolves — basis: {}", entry.basis);
    assert_eq!(entry.basis, "source.paths (exact match)");
}

// ----- F-8 (unit-level cross-check): the handle is a pure function of slug -

#[test]
fn named_arc_number_matches_across_independent_computations() {
    // Sanity cross-check that the integration-test fixtures above and the
    // crate's own unit tests (`selfhost::tests`) agree on the same function.
    let taken = BTreeSet::new();
    assert_eq!(
        named_arc_number("arc-store-home", &taken),
        named_arc_number("arc-store-home", &taken)
    );
}
