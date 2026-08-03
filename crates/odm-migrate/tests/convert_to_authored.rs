//! Integration tests for [`convert_to_authored`] (arc-store-as-source
//! slice02, ODD-0026 §2.1 sub-decision (i)): re-classifying a genuinely-
//! migrated `project`/`arc`/`slice` node as authored, preserving its former
//! `source.paths` in the new `source.migrated_from` marker. Ledger:
//! docs/design-v1.0.0/arc-store-as-source/slice02-self-sourced-nodes/ledger.md

use std::path::Path;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter, Source};
use odm_core::{Id, NodeType, Origin};
use odm_migrate::Mode;
use odm_migrate::selfhost::{convert_to_authored, self_host};
use odm_store::Store;
use tempfile::TempDir;

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Persists a node with a genuine migration `source` — the shape every
/// self-hosted planning node carries today.
fn persist_migrated_node(
    store: &Store,
    number: u32,
    node_type: NodeType,
    name: &str,
    source_path: &str,
) -> Id {
    let id = Id::new();
    let created = day(2026, 7, 20);
    let mut fm = Frontmatter::new(id, number, node_type, name, created, created, Origin::Planned);
    fm.stamp_schema();
    let fm = fm.with_source(Source {
        paths: vec![source_path.into()],
        class: "arc-plan".to_string(),
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

fn node_by_id(store: &Store, id: Id) -> Document {
    store.load_all().unwrap().into_iter().find(|d| d.frontmatter().id() == id).unwrap()
}

// ----- F-10 sub-decision (i): conversion preserves provenance --------------

#[test]
fn convert_flips_origin_and_preserves_the_migrated_path() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let id = persist_migrated_node(
        &store,
        1100,
        NodeType::Arc,
        "Arc 01 — Alpha",
        "docs/design-v1.0.0/arc01-alpha/arc-plan.md",
    );

    let report = convert_to_authored(&store, Mode::Commit).expect("convert");
    assert_eq!(report.converted_count(), 1);
    assert_eq!(report.converted[0].id, id);
    assert_eq!(
        report.converted[0].migrated_from,
        vec![std::path::PathBuf::from("docs/design-v1.0.0/arc01-alpha/arc-plan.md")]
    );

    let node = node_by_id(&store, id);
    assert_eq!(node.frontmatter().origin(), Origin::Authored);
    let source = node.frontmatter().source().expect("still carries a source record");
    assert!(source.is_authored(), "source.class == \"authored\"");
    assert!(source.paths.is_empty(), "no external paths on an authored node");
    assert_eq!(
        source.migrated_from,
        vec![std::path::PathBuf::from("docs/design-v1.0.0/arc01-alpha/arc-plan.md")],
        "the historical path is preserved, not lost"
    );
    assert_eq!(node.body(), "# Arc 01 — Alpha\n", "body untouched by the conversion itself");
}

#[test]
fn convert_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let id = persist_migrated_node(&store, 1100, NodeType::Arc, "Arc", "docs/a.md");

    let report = convert_to_authored(&store, Mode::DryRun).expect("convert dry-run");
    assert_eq!(report.converted_count(), 1);
    assert!(report.dry_run);

    let node = node_by_id(&store, id);
    assert_eq!(node.frontmatter().origin(), Origin::Planned, "dry-run wrote nothing");
}

// ----- Idempotent: a second run converts nothing ----------------------------

#[test]
fn convert_is_idempotent() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_migrated_node(&store, 1100, NodeType::Arc, "Arc", "docs/a.md");

    let first = convert_to_authored(&store, Mode::Commit).expect("first convert");
    assert_eq!(first.converted_count(), 1);

    let second = convert_to_authored(&store, Mode::Commit).expect("second convert");
    assert_eq!(second.converted_count(), 0, "already-authored nodes are a no-op");
}

// ----- Exclusions: sourceless, retired, and synthesis nodes are untouched --

#[test]
fn convert_leaves_a_sourceless_node_alone() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let id = Id::new();
    let created = day(2026, 7, 20);
    let fm =
        Frontmatter::new(id, 1100, NodeType::Arc, "Legacy stub", created, created, Origin::Planned);
    store.persist(&Document::new(fm, "# stub\n".to_string())).unwrap();

    let report = convert_to_authored(&store, Mode::Commit).expect("convert");
    assert_eq!(report.converted_count(), 0, "sourceless is repair()'s territory, not this pass's");
    let node = node_by_id(&store, id);
    assert_eq!(node.frontmatter().origin(), Origin::Planned, "untouched");
}

#[test]
fn convert_leaves_a_retired_node_alone() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let id = persist_migrated_node(&store, 1100, NodeType::Arc, "Retired Arc", "docs/a.md");
    let mut doc = node_by_id(&store, id);
    doc.frontmatter_mut().retire("superseded by a later plan", day(2026, 7, 21));
    store.persist(&doc).unwrap();

    let report = convert_to_authored(&store, Mode::Commit).expect("convert");
    assert_eq!(report.converted_count(), 0, "a retired node is a historical record, not converted");
    let node = node_by_id(&store, id);
    assert_eq!(node.frontmatter().origin(), Origin::Planned, "untouched");
}

// ----- End to end: convert, then self_host never re-mints or churns it -----

#[test]
fn after_conversion_self_host_recognizes_and_never_churns_it() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "project-plan.md", "# Test Project\n");
    write(root, "arc01-alpha/arc-plan.md", "# Arc 01 — Alpha\n\nOriginal.\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, root, Mode::Commit).expect("self-host");

    let convert_report = convert_to_authored(&store, Mode::Commit).expect("convert");
    assert_eq!(convert_report.converted_count(), 2, "the project and the arc both convert");

    // The plan-tree file keeps evolving, as it does throughout this session.
    write(root, "arc01-alpha/arc-plan.md", "# Arc 01 — Alpha\n\nAmended after conversion.\n");
    let second = self_host(&store, root, Mode::Commit).expect("second self-host");
    assert_eq!(second.created_count(), 0, "nothing re-created");

    let arc = store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Arc)
        .unwrap();
    assert_eq!(arc.frontmatter().origin(), Origin::Authored);
    assert!(
        arc.body().contains("Original"),
        "body never re-snapshotted from the amended ./docs file"
    );
}
