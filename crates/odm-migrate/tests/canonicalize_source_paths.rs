//! Integration tests for [`canonicalize_source_paths`] (arc-migration-
//! fidelity slice10 iteration 1): the one-time corrective for a design/
//! research node that already carries a `source` record whose stored path
//! form is not repo-content-root-relative — `backfill_source`'s complement,
//! since that pass only ever touches sourceless nodes. Test names carry the
//! iteration's substring: `canonicalize_source_paths`.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter, Source};
use odm_core::{Id, NodeType, Origin};
use odm_migrate::Mode;
use odm_migrate::mapping::canonicalize_source_paths;
use odm_store::Store;
use tempfile::TempDir;

fn legacy_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/legacy")
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, 29).unwrap()
}

/// Persists a `design` node with a `source.paths` entry exactly as given —
/// standing in for a node minted before the seam fix (an absolute path) or
/// already-canonical (a relative one).
fn persist_sourced(store: &Store, number: u32, source_path: &str) -> Id {
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
        normalization: "trim+lf".to_string(),
        migrated_by: "odm-migrate/test".to_string(),
        migrated_on: day(),
    });
    let document = Document::new(fm, "# Doc\n\nBody.\n".to_string());
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

// ----- an absolute stored path is rewritten to relative ---------------------

#[test]
fn canonicalize_source_paths_rewrites_an_absolute_path_to_relative() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let legacy = legacy_fixture();
    // The repo-anchor-relative form this absolute path collapses to.
    let anchor = odm_migrate::fidelity::anchor_for(&legacy.canonicalize().unwrap());
    let absolute = anchor.join("test-data/legacy/06-final/0005-new-approach.md");
    let id = persist_sourced(&store, 5, &absolute.to_string_lossy());

    let report = canonicalize_source_paths(&store, &legacy, Mode::Commit).expect("canonicalize");
    assert_eq!(report.rewritten_count(), 1);
    assert_eq!(report.rewritten[0].number, 5);
    assert_eq!(report.rewritten[0].id, id);

    let node = node_by_number(&store, 5);
    let stored = node.frontmatter().source().unwrap().paths[0].to_string_lossy().into_owned();
    assert_eq!(stored, "test-data/legacy/06-final/0005-new-approach.md");
    assert!(!stored.starts_with('/'), "{stored}");
    // Only the path string changed.
    assert_eq!(node.frontmatter().id(), id, "id unchanged");
    assert_eq!(node.body(), "# Doc\n\nBody.\n", "body unchanged");
}

// ----- an already-canonical relative path is left alone ---------------------

#[test]
fn canonicalize_source_paths_leaves_an_already_canonical_path_alone() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_sourced(&store, 5, "test-data/legacy/06-final/0005-new-approach.md");

    let report =
        canonicalize_source_paths(&store, &legacy_fixture(), Mode::Commit).expect("canonicalize");
    assert_eq!(report.rewritten_count(), 0, "already canonical — nothing to rewrite");
}

// ----- backfill_source's complement: a sourceless node is untouched ---------

#[test]
fn canonicalize_source_paths_does_not_touch_a_sourceless_node() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let fm = Frontmatter::new(
        Id::new(),
        9,
        NodeType::Design,
        "Sourceless",
        day(),
        day(),
        Origin::Planned,
    );
    store.persist(&Document::new(fm, "# Sourceless\n".to_string())).unwrap();

    let report =
        canonicalize_source_paths(&store, &legacy_fixture(), Mode::Commit).expect("canonicalize");
    assert_eq!(report.rewritten_count(), 0, "sourceless is backfill_source's territory");
}

// ----- dry-run writes nothing -------------------------------------------------

#[test]
fn canonicalize_source_paths_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let legacy = legacy_fixture();
    let anchor = odm_migrate::fidelity::anchor_for(&legacy.canonicalize().unwrap());
    let absolute = anchor.join("test-data/legacy/06-final/0005-new-approach.md");
    persist_sourced(&store, 5, &absolute.to_string_lossy());

    let report = canonicalize_source_paths(&store, &legacy, Mode::DryRun).expect("dry-run");
    assert!(report.dry_run);
    assert_eq!(report.rewritten_count(), 1, "the plan still lists what would be rewritten");

    let node = node_by_number(&store, 5);
    let stored = node.frontmatter().source().unwrap().paths[0].to_string_lossy().into_owned();
    assert_eq!(stored, absolute.to_string_lossy(), "dry-run wrote nothing — still absolute");
}
