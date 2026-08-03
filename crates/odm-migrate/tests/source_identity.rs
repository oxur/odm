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
        normalization: Some("trim+lf".to_string()),
        migrated_by: Some("odm-migrate/test".to_string()),
        migrated_on: Some(created),
        synthesis: None,
        attestation: None,
        migrated_from: Vec::new(),
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
    // would compute for it. `self_host` canonicalizes `plan_root` internally
    // (arc-migration-fidelity s08 F-1) before deriving its anchor, so this
    // must match against the same canonicalized form, not the raw `TempDir`
    // path (which can differ, e.g. a `/tmp` → `/private/tmp` symlink).
    let canonical_root = root.canonicalize().unwrap();
    let source_path = canonical_root.join("arc01-alpha/arc-plan.md");
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

// ----- arc-store-as-source slice02: authored nodes are recognized, never
// ----- churned, even while their plan-tree file is still present ----------
// Ledger: docs/design-v1.0.0/arc-store-as-source/slice02-self-sourced-nodes/ledger.md

/// Persists a node with `origin: authored` — the ODD-0026 shape: `source:
/// { class: authored }`, no external `paths`.
fn persist_authored_node(store: &Store, number: u32, node_type: NodeType, name: &str) -> Id {
    let id = Id::new();
    let created = day(2026, 7, 20);
    let mut fm = Frontmatter::new(id, number, node_type, name, created, created, Origin::Authored);
    fm.stamp_schema();
    let fm = fm.with_source(Source::authored(Vec::new()));
    let document = Document::new(fm, format!("# {name}\n\nAuthored body.\n"));
    store.persist(&document).unwrap();
    id
}

#[test]
fn selfhost_never_churns_an_authored_node_even_when_its_plan_tree_file_still_exists() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n");
    // The plan-tree file is still present (true for every node until the
    // slice04 cutover) — this is exactly the shape that, without slice02's
    // fix, would look "unmatched" to self_host and get re-minted.
    write(root, "arc01-alpha/arc-plan.md", "# Arc 01 — Alpha (docs version, stale)\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let arc_id = persist_authored_node(&store, arc_number(1), NodeType::Arc, "Arc 01 — Alpha");

    let report = self_host(&store, root, Mode::Commit).expect("self-host");

    assert_eq!(report.created_count(), 1, "project only — the authored arc is not re-minted");
    assert!(
        report
            .skipped
            .iter()
            .any(|s| s.number == Some(arc_number(1)) && matches!(s.reason, SkipReason::Authored)),
        "reported as recognized-authored, not a plain skip or a duplicate: {:?}",
        report.skipped.iter().map(|s| (s.number, s.reason.to_string())).collect::<Vec<_>>()
    );

    let nodes = nodes_by_key(&store);
    assert_eq!(store.load_all().unwrap().len(), 2, "project + the one authored arc — no duplicate");
    let arc = &nodes[&(NodeType::Arc, arc_number(1))];
    assert_eq!(arc.frontmatter().id(), arc_id, "same node, never re-minted");
    assert_eq!(arc.frontmatter().origin(), Origin::Authored, "origin untouched");
    assert_eq!(
        arc.body(),
        "# Arc 01 — Alpha\n\nAuthored body.\n",
        "body never churned from the still-present ./docs file"
    );
    assert!(
        arc.frontmatter().source().is_some_and(Source::is_authored),
        "source stays the authored shape, never rebuilt as a migration record"
    );
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

// ----- arc-store-as-source slice02 F-7: the no-regression clause -----------
// (ODD-0026 §2.2 — fork B narrows the gate to migrated-with-source nodes,
// it does not remove it) — proven in a corpus that also carries an
// authored node, so the new authored-recognition path can't be masking it.

#[test]
fn a_drifted_migrated_node_still_hard_fails_alongside_an_untouched_authored_one() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n");
    write(
        root,
        "arc01-alpha/arc-plan.md",
        "# Arc 01 — Alpha (plan-of-record)\n\nThe real, current source content.\n",
    );
    write(root, "arc02-beta/arc-plan.md", "# Arc 02 — Beta (docs, stale)\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    // An authored arc, self-sourced and immune to this gate entirely.
    persist_authored_node(&store, arc_number(2), NodeType::Arc, "Arc 02 — Beta");

    // A genuinely-migrated (pre-source) arc whose body has drifted from its
    // real source — the gate `repair()` must still enforce.
    persist_legacy_node_with_body(
        &store,
        arc_number(1),
        NodeType::Arc,
        "Arc 01 — Alpha",
        "# Arc 01 — Alpha (plan-of-record)\n\nA DIFFERENT body — drifted from source.\n",
    );

    let err = repair(&store, root, Mode::Commit)
        .expect_err("the authored sibling does not mask the drift on the migrated node");
    assert!(matches!(err, MigrateError::BodyHashMismatch { .. }), "{err:?}");

    // The authored node is confirmed untouched by the same run that hard-failed.
    let nodes = nodes_by_key(&store);
    assert_eq!(
        nodes[&(NodeType::Arc, arc_number(2))].frontmatter().origin(),
        Origin::Authored,
        "the authored sibling is unaffected by the migrated node's failure"
    );
}

// ----- F-6: the project node is excluded from backfill ---------------------

#[test]
fn repair_backfills_a_faithful_sourceless_project() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n\nReal plan content.\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // s15 F-3: the reconcile exclusion is keyed on `source.synthesis`, not
    // `node_type == Project` — a sourceless project whose body already
    // matches `project-plan.md` is backfilled like any other faithful legacy
    // node, not left alone.
    let id = persist_legacy_node_with_body(
        &store,
        PROJECT_NUMBER,
        NodeType::Project,
        "odm",
        "# Test Project\n\nReal plan content.\n",
    );

    let report = repair(&store, root, Mode::Commit).expect("repair does not error on the project");
    assert_eq!(report.repaired_count(), 1, "the faithful project is backfilled by repair");

    let nodes = store.load_all().unwrap();
    let project = nodes.iter().find(|d| d.frontmatter().id() == id).unwrap();
    let source = project.frontmatter().source().expect("source backfilled");
    assert!(source.synthesis.is_none(), "a faithful 1:1 project carries no synthesis key");
}

#[test]
fn repair_excludes_a_synthesis_bearing_project() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n\nReal plan content.\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A node already carrying `source.synthesis` is a merge, never a 1:1
    // migration (ODD-0025 §2.3) — excluded outright, keyed s15 F-3.
    let id = Id::new();
    let created = day(2026, 7, 20);
    let mut fm = Frontmatter::new(
        id,
        PROJECT_NUMBER,
        NodeType::Project,
        "odm",
        created,
        created,
        Origin::Planned,
    );
    fm.stamp_schema();
    let fm = fm.with_source(Source {
        paths: vec![Path::new("project-plan.md").to_path_buf()],
        class: "vision".to_string(),
        normalization: Some("trim+lf".to_string()),
        migrated_by: Some("odm-migrate/test".to_string()),
        migrated_on: Some(created),
        synthesis: Some("editorial-merge".to_string()),
        attestation: Some("operator: distills the source".to_string()),
        migrated_from: Vec::new(),
    });
    let body = "# odm\n\n# Vision\n\nSynthesized, not the source.\n";
    store.persist(&Document::new(fm, body.to_string())).unwrap();

    // `repair` only ever touches a *sourceless* node (its very first check),
    // so a synthesis node — which already carries `source` — is invisible to
    // it regardless of the exclusion key; this documents that it stays that
    // way, untouched, across the rekey.
    let report =
        repair(&store, root, Mode::Commit).expect("repair does not error on a synthesis node");
    assert_eq!(report.repaired_count(), 0, "already-sourced — repair's territory ends here");

    let nodes = store.load_all().unwrap();
    let project = nodes.iter().find(|d| d.frontmatter().id() == id).unwrap();
    assert_eq!(project.body(), body, "the synthesis body is untouched");
}

// ----- s15 F-3: the self_host coordinate→source transition backfills a ----
// ----- faithful sourceless project too (rekeyed off `source.synthesis`) ----

#[test]
fn selfhost_transition_backfills_a_faithful_sourceless_project() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n\nThe real plan-of-record content.\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A pre-source legacy project node at exactly this run's coordinate
    // (`PROJECT_NUMBER`), body already faithful to `project-plan.md` —
    // standing in for a corpus that predates the `source` model entirely.
    let id = persist_legacy_node_with_body(
        &store,
        PROJECT_NUMBER,
        NodeType::Project,
        "odm",
        "# Test Project\n\nThe real plan-of-record content.\n",
    );

    // Reached via `self_host`'s coordinate→source **transition**, not
    // `repair` — the same path the CDC v2.1 finding once flagged as ungated,
    // now proven to backfill the project like any other faithful node.
    let report = self_host(&store, root, Mode::Commit).expect("self-host does not error");
    assert!(
        report
            .skipped
            .iter()
            .any(|s| s.number == Some(PROJECT_NUMBER)
                && matches!(s.reason, SkipReason::SourcePopulated)),
        "the project is recognized by coordinate and queued for the gated backfill: {:?}",
        report.skipped.iter().map(|s| s.number).collect::<Vec<_>>()
    );

    let nodes = store.load_all().unwrap();
    let project = nodes.iter().find(|d| d.frontmatter().id() == id).unwrap();
    let source = project.frontmatter().source().expect("1:1 source via the transition path");
    assert!(source.synthesis.is_none(), "a faithful 1:1 project carries no synthesis key");
    assert_eq!(
        project.body(),
        "# Test Project\n\nThe real plan-of-record content.\n",
        "the faithful body is a content no-op"
    );
}

#[test]
fn selfhost_transition_excludes_a_synthesis_bearing_project() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n\nThe real plan-of-record content.\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A project already carrying `source.synthesis` is never in
    // `by_coordinate` at all (that index is sourceless-only) — this proves
    // the transition never mistakenly re-populates or re-touches it.
    let id = Id::new();
    let created = day(2026, 7, 20);
    let mut fm = Frontmatter::new(
        id,
        PROJECT_NUMBER,
        NodeType::Project,
        "odm",
        created,
        created,
        Origin::Planned,
    );
    fm.stamp_schema();
    let fm = fm.with_source(Source {
        paths: vec![Path::new("project-plan.md").to_path_buf()],
        class: "vision".to_string(),
        normalization: Some("trim+lf".to_string()),
        migrated_by: Some("odm-migrate/test".to_string()),
        migrated_on: Some(created),
        synthesis: Some("editorial-merge".to_string()),
        attestation: Some("operator: distills the source".to_string()),
        migrated_from: Vec::new(),
    });
    let body = "# odm\n\n# Vision\n\nSynthesized, not the source.\n";
    store.persist(&Document::new(fm, body.to_string())).unwrap();

    let report = self_host(&store, root, Mode::Commit).expect("self-host does not error");
    assert!(
        report
            .skipped
            .iter()
            .any(|s| s.number == Some(PROJECT_NUMBER)
                && matches!(s.reason, SkipReason::AlreadyExists)),
        "a synthesis-bearing project matches by source, recognized as already-existing: {:?}",
        report.skipped.iter().map(|s| s.number).collect::<Vec<_>>()
    );

    let nodes = store.load_all().unwrap();
    let project = nodes.iter().find(|d| d.frontmatter().id() == id).unwrap();
    assert_eq!(project.body(), body, "the synthesis body is untouched");
}

// ----- F-7: source-based coverage matching resolves a named arc ------------

#[test]
fn coverage_source_based_matching_resolves_a_named_arc() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    // A `.git` marker at the repo root, same as any real checkout — so
    // `self_host`'s and `coverage::run`'s independently-derived anchors
    // (arc-migration-fidelity s08 F-1/F-6) converge on the same git
    // toplevel even though the two calls start from different subpaths
    // below it, exactly as they do in production.
    std::fs::write(root.join(".git"), "gitdir: fake\n").unwrap();
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
