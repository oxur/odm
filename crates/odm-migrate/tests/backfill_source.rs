//! Integration tests for the design/research `source` backfill
//! (arc-migration-fidelity slice09, MF-3's residual scope: the 14
//! `design`/`research` nodes that predate `source` entirely). Test names
//! carry the ledger's F-4 substring: `backfill_source`.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_migrate::mapping::backfill_source;
use odm_migrate::{Mode, migrate};
use odm_store::Store;
use tempfile::TempDir;

fn legacy_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/legacy")
}

/// Persists a sourceless `design` node directly — bypassing `migrate`, so the
/// store carries exactly the pre-slice03 shape `backfill_source` must repair:
/// a document node identified only by its legacy `number`, no `source` at all.
fn persist_sourceless(store: &Store, number: u32, body: &str) {
    let today = NaiveDate::from_ymd_opt(2026, 7, 27).unwrap();
    let fm = Frontmatter::new(
        Id::new(),
        number,
        NodeType::Design,
        format!("Legacy #{number}"),
        today,
        today,
        Origin::Planned,
    );
    let document = Document::new(fm, body.to_string());
    store.persist(&document).expect("persist");
}

fn node_by_number(store: &Store, number: u32) -> Document {
    store
        .load_all()
        .expect("load_all")
        .into_iter()
        .find(|d| d.frontmatter().number() == number)
        .unwrap_or_else(|| panic!("#{number} not found"))
}

// ----- F-4: a stub sourceless node is backfilled — body replaced + source --

#[test]
fn backfill_source_replaces_a_stub_body_and_adds_source() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_sourceless(&store, 1, "# Legacy #1\n"); // a stub — lone H1

    let report = backfill_source(&store, &legacy_fixture(), Mode::Commit).expect("backfill");
    assert_eq!(report.repaired_count(), 1);

    let node = node_by_number(&store, 1);
    assert!(
        node.body().contains("The first sketch. Body carried verbatim on migration."),
        "the stub body is replaced with the real legacy content"
    );
    let source = node.frontmatter().source().expect("source backfilled");
    assert_eq!(source.paths.len(), 1);
    assert!(
        source.paths[0].ends_with("01-draft/0001-early-draft.md"),
        "source path: {:?}",
        source.paths[0]
    );
    assert_eq!(source.class, "odd");
}

// ----- F-4: a faithful non-stub sourceless node keeps its body, gains source

#[test]
fn backfill_source_keeps_a_faithful_body_and_adds_source() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let faithful_body = "# The new approach\n\nSupersedes #4 — a `supersedes` edge (kind \
                          obsoletes) points at #4's new ULID.\n";
    persist_sourceless(&store, 5, faithful_body);

    let report = backfill_source(&store, &legacy_fixture(), Mode::Commit).expect("backfill");
    assert_eq!(report.repaired_count(), 1);

    let node = node_by_number(&store, 5);
    assert_eq!(node.body(), faithful_body, "a faithful non-stub body is left untouched");
    assert!(node.frontmatter().source().is_some());
}

// ----- F-4: a drifted non-stub body is a hard error, never silently backfilled

#[test]
fn backfill_source_rejects_a_drifted_non_stub_body() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_sourceless(&store, 5, "# The new approach\n\nThis text has drifted from the source.\n");

    let err = backfill_source(&store, &legacy_fixture(), Mode::Commit).unwrap_err();
    assert!(matches!(err, odm_migrate::MigrateError::BodyHashMismatch { .. }));
}

// ----- F-4: idempotent — an already-sourced node is left alone --------------

#[test]
fn backfill_source_is_idempotent() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A node created through the ordinary importer already carries `source`.
    migrate(&store, &legacy_fixture(), Mode::Commit).expect("migrate");

    let report = backfill_source(&store, &legacy_fixture(), Mode::Commit).expect("backfill");
    assert_eq!(report.repaired_count(), 0, "already-sourced nodes are untouched");
}

// ----- F-4: dry-run writes nothing -------------------------------------------

#[test]
fn backfill_source_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_sourceless(&store, 1, "# Legacy #1\n");

    let report = backfill_source(&store, &legacy_fixture(), Mode::DryRun).expect("dry-run");
    assert!(report.dry_run);
    assert_eq!(report.repaired_count(), 1, "the plan still lists what would be repaired");
    let node = node_by_number(&store, 1);
    assert!(node.frontmatter().source().is_none(), "dry-run wrote nothing");
}
