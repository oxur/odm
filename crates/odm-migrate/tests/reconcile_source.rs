//! Integration tests for [`reconcile_source`] (arc-migration-fidelity s12
//! F-1/F-2): `backfill_source`'s complement — reconciling a design/research
//! node that **already carries** a `source`, either because its legacy file
//! moved (stored `source.paths` no longer resolves) or because the file's
//! content legitimately changed since it was last snapshotted (or both, the
//! concrete live shape ODD-0013/0017/0018 hit — s11's L-8b moves).

use std::path::Path;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter, Source};
use odm_core::{Id, NodeType, Origin};
use odm_migrate::Mode;
use odm_migrate::fidelity::anchor_for;
use odm_migrate::mapping::reconcile_source;
use odm_store::Store;
use tempfile::TempDir;

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, 29).unwrap()
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Persists a `design` node with a `source.paths` entry exactly as given,
/// with the given body — standing in for a node minted at some earlier
/// point, before its legacy file moved and/or its content changed.
fn persist_sourced(store: &Store, number: u32, source_path: &str, body: &str) -> Id {
    let id = Id::new();
    let mut fm = Frontmatter::new(
        id,
        number,
        NodeType::Design,
        format!("Doc #{number}"),
        day(),
        day(),
        Origin::Planned,
    );
    fm.stamp_schema();
    let fm = fm.with_source(Source {
        paths: vec![source_path.into()],
        class: "odd".to_string(),
        normalization: Some("trim+lf".to_string()),
        migrated_by: Some("odm-migrate/test".to_string()),
        migrated_on: Some(day()),
        synthesis: None,
        attestation: None,
        migrated_from: Vec::new(),
    });
    let document = Document::new(fm, body.to_string());
    store.persist(&document).expect("persist");
    id
}

fn node_by_number(store: &Store, number: u32) -> Document {
    store
        .load_all()
        .expect("load_all")
        .into_iter()
        .find(|d| d.frontmatter().number() == number)
        .unwrap_or_else(|| panic!("#{number} not found"))
}

const LEGACY_BODY_V1: &str = "# Doc 99\n\nOriginal content.\n";
const LEGACY_BODY_V2: &str = "# Doc 99\n\nAmended content — the source legitimately changed.\n";

fn legacy_frontmatter(number: u32) -> String {
    format!(
        "---\nnumber: {number}\ntitle: \"Doc {number}\"\nstate: Accepted\n\
         created: 2026-06-01\nupdated: 2026-07-29\n---\n"
    )
}

// ----- F-2: a moved legacy file is re-discovered by number ------------------

#[test]
fn reconcile_source_rediscovers_a_moved_legacy_file() {
    let repo = TempDir::new().unwrap();
    std::fs::write(repo.path().join(".git"), "gitdir: fake\n").unwrap();
    // The legacy file now lives at the *new* location (simulating an s11
    // L-8b-shaped `01-draft/` -> `04-accepted/` move) — the old path is gone.
    write(
        repo.path(),
        "docs/design/04-accepted/0099-doc.md",
        &format!("{}{}", legacy_frontmatter(99), LEGACY_BODY_V1),
    );
    let legacy_path = repo.path().join("docs/design");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // The node's stored source.paths still points at the *old*, now-gone
    // location, with the body it had at that time.
    persist_sourced(&store, 99, "docs/design/01-draft/0099-doc.md", LEGACY_BODY_V1);

    let report = reconcile_source(&store, &legacy_path, Mode::Commit).expect("reconcile");
    assert_eq!(report.reconciled_count(), 1);
    assert!(report.reconciled[0].path_moved, "the move is detected");
    assert!(!report.reconciled[0].body_drifted, "content itself didn't change here");

    let node = node_by_number(&store, 99);
    let stored = node.frontmatter().source().unwrap().paths[0].to_string_lossy().into_owned();
    assert_eq!(
        stored, "docs/design/04-accepted/0099-doc.md",
        "source.paths rewritten to the new, canonical location"
    );
    assert_eq!(node.body(), LEGACY_BODY_V1, "body unchanged — only the path had gone stale");
}

// ----- F-1: a drifted (but not moved) body is re-snapshotted ----------------

#[test]
fn reconcile_source_resnapshots_a_drifted_body_in_place() {
    let repo = TempDir::new().unwrap();
    std::fs::write(repo.path().join(".git"), "gitdir: fake\n").unwrap();
    write(
        repo.path(),
        "docs/design/04-accepted/0099-doc.md",
        &format!("{}{}", legacy_frontmatter(99), LEGACY_BODY_V2),
    );
    let legacy_path = repo.path().join("docs/design");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let id = persist_sourced(&store, 99, "docs/design/04-accepted/0099-doc.md", LEGACY_BODY_V1);

    let report = reconcile_source(&store, &legacy_path, Mode::Commit).expect("reconcile");
    assert_eq!(report.reconciled_count(), 1);
    assert!(!report.reconciled[0].path_moved, "the path already resolved");
    assert!(report.reconciled[0].body_drifted, "the content changed");

    let node = node_by_number(&store, 99);
    assert_eq!(node.frontmatter().id(), id, "same node, not re-minted");
    assert_eq!(node.body(), LEGACY_BODY_V2, "body re-snapshotted to the current legacy content");
}

// ----- F-1 + F-2: a node needing both a moved path and a changed body ------

#[test]
fn reconcile_source_handles_a_node_needing_both_move_and_resnapshot() {
    let repo = TempDir::new().unwrap();
    std::fs::write(repo.path().join(".git"), "gitdir: fake\n").unwrap();
    write(
        repo.path(),
        "docs/design/04-accepted/0099-doc.md",
        &format!("{}{}", legacy_frontmatter(99), LEGACY_BODY_V2),
    );
    let legacy_path = repo.path().join("docs/design");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let id = persist_sourced(&store, 99, "docs/design/01-draft/0099-doc.md", LEGACY_BODY_V1);

    let report = reconcile_source(&store, &legacy_path, Mode::Commit).expect("reconcile");
    assert_eq!(report.reconciled_count(), 1);
    assert!(report.reconciled[0].path_moved);
    assert!(report.reconciled[0].body_drifted);

    let node = node_by_number(&store, 99);
    assert_eq!(node.frontmatter().id(), id);
    let stored = node.frontmatter().source().unwrap().paths[0].to_string_lossy().into_owned();
    assert_eq!(stored, "docs/design/04-accepted/0099-doc.md");
    assert_eq!(node.body(), LEGACY_BODY_V2);
}

// ----- F-3: the living-plan-node policy — reconcile-to-current, no reject --

#[test]
fn reconcile_source_reconciles_cleanly_across_repeated_source_edits() {
    // A "living" source: it changes again between two reconcile runs. Both
    // runs must succeed and pick up whatever the source currently says — no
    // special exclusion for a still-changing document (s12 F-3, ODD-0025
    // amended: the gate is migration-time-only, so this is by-design).
    let repo = TempDir::new().unwrap();
    std::fs::write(repo.path().join(".git"), "gitdir: fake\n").unwrap();
    let legacy_path = repo.path().join("docs/design");
    write(
        repo.path(),
        "docs/design/04-accepted/0099-doc.md",
        &format!("{}{}", legacy_frontmatter(99), LEGACY_BODY_V1),
    );

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_sourced(&store, 99, "docs/design/04-accepted/0099-doc.md", "# Doc 99\n\nStale.\n");

    let first = reconcile_source(&store, &legacy_path, Mode::Commit).expect("first reconcile");
    assert_eq!(first.reconciled_count(), 1);
    assert_eq!(node_by_number(&store, 99).body(), LEGACY_BODY_V1);

    // The source is edited again — a "living doc" continuing to change.
    write(
        repo.path(),
        "docs/design/04-accepted/0099-doc.md",
        &format!("{}{}", legacy_frontmatter(99), LEGACY_BODY_V2),
    );
    let second = reconcile_source(&store, &legacy_path, Mode::Commit).expect("second reconcile");
    assert_eq!(second.reconciled_count(), 1, "the second edit reconciles cleanly, no rejection");
    assert_eq!(node_by_number(&store, 99).body(), LEGACY_BODY_V2);
}

// ----- F-6: idempotent + dry-run-safe ---------------------------------------

#[test]
fn reconcile_source_is_idempotent_and_dry_run_writes_nothing() {
    let repo = TempDir::new().unwrap();
    std::fs::write(repo.path().join(".git"), "gitdir: fake\n").unwrap();
    write(
        repo.path(),
        "docs/design/04-accepted/0099-doc.md",
        &format!("{}{}", legacy_frontmatter(99), LEGACY_BODY_V2),
    );
    let legacy_path = repo.path().join("docs/design");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_sourced(&store, 99, "docs/design/01-draft/0099-doc.md", LEGACY_BODY_V1);

    // Dry-run: reports what it would do, writes nothing.
    let dry = reconcile_source(&store, &legacy_path, Mode::DryRun).expect("dry-run");
    assert!(dry.dry_run);
    assert_eq!(dry.reconciled_count(), 1);
    let untouched = node_by_number(&store, 99);
    assert_eq!(untouched.body(), LEGACY_BODY_V1, "dry-run wrote nothing");
    let stored = untouched.frontmatter().source().unwrap().paths[0].to_string_lossy().into_owned();
    assert_eq!(stored, "docs/design/01-draft/0099-doc.md", "path unchanged under dry-run");

    // Fire for real, then run again: second pass is a clean no-op.
    reconcile_source(&store, &legacy_path, Mode::Commit).expect("commit");
    let second = reconcile_source(&store, &legacy_path, Mode::Commit).expect("second run");
    assert_eq!(second.reconciled_count(), 0, "already reconciled — nothing left to do");
}

// ----- sanity: the anchor helper resolves the same repo everywhere ---------

#[test]
fn anchor_for_the_fixture_repo_is_its_git_toplevel() {
    let repo = TempDir::new().unwrap();
    std::fs::write(repo.path().join(".git"), "gitdir: fake\n").unwrap();
    let legacy_path = repo.path().join("docs/design");
    std::fs::create_dir_all(&legacy_path).unwrap();
    assert_eq!(
        anchor_for(&legacy_path.canonicalize().unwrap()),
        repo.path().canonicalize().unwrap()
    );
}
