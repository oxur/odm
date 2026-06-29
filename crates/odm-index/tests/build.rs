//! Tests for the cold-path build (Arc 04 slice02).
//!
//! Seeds a real corpus through `odm-store`, then builds the index over it. Test
//! names carry the substrings the ledger Verify commands filter on
//! (`cold_build_one_record_per_file`, `cold_build_empty_corpus`,
//! `cold_build_stat_fields`, `cold_build_content_hash`,
//! `cold_build_metadata_fields`, `cold_build_edge_mapping`,
//! `meta_hash_deterministic`, `cold_build_persists_snapshot`,
//! `cold_build_then_load_roundtrip`).

use chrono::NaiveDate;
use odm_core::frontmatter::{
    Dependency, Document, Edges, Frontmatter, SupersedeKind, Supersedes, TornEdge,
};
use odm_core::gates::GateSet;
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use odm_index::record::IndexRecord;
use odm_index::{
    EdgeKind, EdgeQualifier, Load, Snapshot, SupersedeKind as IdxSupersedeKind, build,
    build_records,
};
use odm_store::Store;
use sha2::{Digest as _, Sha256};
use tempfile::TempDir;

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 26).unwrap()
}

fn slice_gates() -> GateSet {
    GateSet::new(vec!["planned".to_string(), "built".to_string(), "tested".to_string()])
}

/// Persists a document and returns its id.
fn seed(root: &std::path::Path, doc: Document) -> Id {
    let id = doc.frontmatter().id();
    Store::open(root).persist(&doc).expect("seed persist");
    id
}

fn fm(number: u32, ty: NodeType, name: &str) -> Frontmatter {
    Frontmatter::new(Id::new(), number, ty, name, day(), day(), Origin::Planned)
}

/// The record for `id` in a built set.
fn record_for(records: &[IndexRecord], id: Id) -> &IndexRecord {
    records.iter().find(|r| r.id == id).expect("a record for the id")
}

// ----- B-1: one record per file; empty corpus → empty set -------------------

#[test]
fn cold_build_one_record_per_file() {
    let dir = TempDir::new().unwrap();
    seed(dir.path(), Document::new(fm(1, NodeType::Project, "Proj"), "# Proj\n"));
    seed(dir.path(), Document::new(fm(2, NodeType::Arc, "Arc"), "# Arc\n"));
    seed(dir.path(), Document::new(fm(3, NodeType::Slice, "Slice"), "# Slice\n"));

    let records = build_records(&Store::open(dir.path())).unwrap();
    assert_eq!(records.len(), 3, "one record per node file");
    // Sorted by id (creation order).
    assert!(records.windows(2).all(|w| w[0].id <= w[1].id), "records sorted by id");
}

#[test]
fn cold_build_empty_corpus_is_not_an_error() {
    let dir = TempDir::new().unwrap();
    // No `nodes/` directory at all.
    let records = build_records(&Store::open(dir.path())).unwrap();
    assert!(records.is_empty(), "a missing corpus yields an empty record set");
}

// ----- B-2: stat fields come from lstat --------------------------------------

#[test]
fn cold_build_stat_fields_from_lstat() {
    let dir = TempDir::new().unwrap();
    let id = seed(dir.path(), Document::new(fm(1, NodeType::Slice, "S"), "# S\nbody\n"));

    let records = build_records(&Store::open(dir.path())).unwrap();
    let rec = record_for(&records, id);

    let on_disk = std::fs::symlink_metadata(dir.path().join(&rec.rel_path)).unwrap();
    assert_eq!(rec.size, on_disk.len(), "size matches the file");
    assert!(rec.mtime_secs > 0, "mtime recorded");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        assert_eq!(rec.inode, on_disk.ino(), "inode from lstat");
        assert_eq!(rec.mode, on_disk.mode(), "mode from lstat");
        assert_ne!(rec.mode, 0);
    }
}

// ----- B-3: content_hash = SHA-256 of the raw bytes --------------------------

#[test]
fn cold_build_content_hash_is_sha256_of_bytes() {
    let dir = TempDir::new().unwrap();
    let id = seed(dir.path(), Document::new(fm(1, NodeType::Slice, "S"), "# S\nhello\n"));

    let records = build_records(&Store::open(dir.path())).unwrap();
    let rec = record_for(&records, id);

    let bytes = std::fs::read(dir.path().join(&rec.rel_path)).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let expected: [u8; 32] = hasher.finalize().into();
    assert_eq!(rec.content_hash, expected, "content_hash is SHA-256 of the file bytes");
    assert_ne!(rec.content_hash, rec.meta_hash, "the two fingerprints differ");
}

// ----- B-4: extracted metadata from the parsed document ----------------------

#[test]
fn cold_build_metadata_fields_from_document() {
    let dir = TempDir::new().unwrap();
    let mut f = fm(7, NodeType::Slice, "Store layer").with_tags(vec!["store".into(), "io".into()]);
    let gs = slice_gates();
    f.status_mut().set_gate(&gs, "planned", None, Evidence::Reproduced, day()).unwrap();
    f.status_mut().set_gate(&gs, "built", None, Evidence::Asserted, day()).unwrap();
    let id = seed(dir.path(), Document::new(f, "# Store layer\n"));

    let records = build_records(&Store::open(dir.path())).unwrap();
    let rec = record_for(&records, id);

    assert_eq!(rec.number, 7);
    assert_eq!(rec.node_type, NodeType::Slice);
    assert_eq!(rec.title, "Store layer");
    assert_eq!(rec.updated, day());
    assert_eq!(rec.tags, ["store", "io"]);
    // `gates` is the reached-gate set with evidence, gate-name sorted (BTreeMap).
    let gates: Vec<(&str, Evidence)> =
        rec.gates.iter().map(|g| (g.gate.as_str(), g.evidence)).collect();
    assert_eq!(gates, [("built", Evidence::Asserted), ("planned", Evidence::Reproduced)]);
}

// ----- B-5: domain edges mapped to EdgeRef across all kinds, with qualifiers -

#[test]
fn cold_build_edge_mapping_all_kinds() {
    let dir = TempDir::new().unwrap();

    // Targets the edges point at (need not be full nodes for the mapping test).
    let parent = Id::new();
    let dep = Id::new();
    let qualdep = Id::new();
    let blocker = Id::new();
    let verified = Id::new();
    let consumed = Id::new();
    let affected = Id::new();
    let old = Id::new();
    let torn = Id::new();

    let edges = Edges {
        part_of: Some(parent),
        depends_on: vec![
            Dependency::Bare(dep),
            Dependency::Qualified { node: qualdep, satisfied_at: "tested".to_string() },
        ],
        blocked_by: vec![blocker],
        verifies: vec![verified],
        consumes: vec![consumed],
        affects: vec![affected],
        supersedes: Some(Supersedes { node: old, kind: SupersedeKind::Obsoletes }),
        tears: vec![TornEdge {
            edge: Dependency::Bare(torn),
            because: "assumed ready".to_string(),
        }],
    };
    let id = seed(
        dir.path(),
        Document::new(fm(1, NodeType::Slice, "Edgy").with_edges(edges), "# Edgy\n"),
    );

    let records = build_records(&Store::open(dir.path())).unwrap();
    let e = &record_for(&records, id).edges;

    let has = |t: Id, k: EdgeKind, q: Option<EdgeQualifier>| {
        e.iter().any(|r| r.target == t && r.kind == k && r.qualifier == q)
    };
    assert!(has(parent, EdgeKind::PartOf, None));
    assert!(has(dep, EdgeKind::DependsOn, None), "bare depends_on carries no qualifier");
    assert!(
        has(qualdep, EdgeKind::DependsOn, Some(EdgeQualifier::SatisfiedAt("tested".into()))),
        "qualified depends_on preserves satisfied_at"
    );
    assert!(has(blocker, EdgeKind::BlockedBy, None));
    assert!(has(verified, EdgeKind::Verifies, None));
    assert!(has(consumed, EdgeKind::Consumes, None));
    assert!(has(affected, EdgeKind::Affects, None));
    assert!(
        has(old, EdgeKind::Supersedes, Some(EdgeQualifier::Supersede(IdxSupersedeKind::Obsoletes))),
        "supersede kind preserved"
    );
    assert!(
        has(torn, EdgeKind::Tears, Some(EdgeQualifier::Because("assumed ready".into()))),
        "tear rationale preserved"
    );
    assert_eq!(e.len(), 9, "every edge kind present is mapped once");
}

// ----- B-5 (cont.): the `Updates` supersede kind + a qualified tear ----------

#[test]
fn cold_build_edge_mapping_supersede_updates_and_qualified_tear() {
    let dir = TempDir::new().unwrap();
    let old = Id::new();
    let torn = Id::new();
    let edges = Edges {
        supersedes: Some(Supersedes { node: old, kind: SupersedeKind::Updates }),
        tears: vec![TornEdge {
            edge: Dependency::Qualified { node: torn, satisfied_at: "built".to_string() },
            because: "qualified tear".to_string(),
        }],
        ..Edges::default()
    };
    let id =
        seed(dir.path(), Document::new(fm(1, NodeType::Slice, "U").with_edges(edges), "# U\n"));

    let records = build_records(&Store::open(dir.path())).unwrap();
    let e = &record_for(&records, id).edges;
    assert!(
        e.iter().any(|r| r.target == old
            && r.qualifier == Some(EdgeQualifier::Supersede(IdxSupersedeKind::Updates))),
        "supersede `Updates` kind preserved"
    );
    assert!(
        e.iter().any(|r| r.target == torn && r.kind == EdgeKind::Tears),
        "a gate-qualified tear maps to its target node"
    );
}

// ----- B-1 (cont.): a malformed / non-UTF-8 node file errors, not panics -----

#[test]
fn cold_build_errors_on_malformed_node_file() {
    let dir = TempDir::new().unwrap();
    let bad = dir.path().join("nodes").join("2026").join("06");
    std::fs::create_dir_all(&bad).unwrap();
    std::fs::write(bad.join("bad.md"), "this has no frontmatter fence").unwrap();

    let err = build_records(&Store::open(dir.path())).unwrap_err();
    assert!(matches!(err, odm_index::BuildError::Parse { .. }), "got {err:?}");
}

#[test]
fn cold_build_errors_on_invalid_utf8_node_file() {
    let dir = TempDir::new().unwrap();
    let bad = dir.path().join("nodes").join("2026").join("06");
    std::fs::create_dir_all(&bad).unwrap();
    std::fs::write(bad.join("bad.md"), [0xFF, 0xFE, 0x00]).unwrap();

    let err = build_records(&Store::open(dir.path())).unwrap_err();
    assert!(matches!(err, odm_index::BuildError::Utf8 { .. }), "got {err:?}");
}

// ----- B-6: meta_hash is deterministic across runs ---------------------------

#[test]
fn meta_hash_deterministic_across_runs() {
    let dir = TempDir::new().unwrap();
    let mut f = fm(1, NodeType::Slice, "S").with_tags(vec!["b".into(), "a".into()]);
    let gs = slice_gates();
    f.status_mut().set_gate(&gs, "built", None, Evidence::Asserted, day()).unwrap();
    let mut e = Edges::default();
    e.depends_on.push(Dependency::Bare(Id::new()));
    let id = seed(dir.path(), Document::new(f.with_edges(e), "# S\n"));

    let first = build_records(&Store::open(dir.path())).unwrap();
    let second = build_records(&Store::open(dir.path())).unwrap();
    assert_eq!(
        record_for(&first, id).meta_hash,
        record_for(&second, id).meta_hash,
        "identical metadata ⇒ identical meta_hash across runs"
    );
}

// ----- B-7: assemble + persist via slice01's Snapshot ------------------------

#[test]
fn cold_build_persists_snapshot() {
    let dir = TempDir::new().unwrap();
    seed(dir.path(), Document::new(fm(1, NodeType::Project, "P"), "# P\n"));
    seed(dir.path(), Document::new(fm(2, NodeType::Slice, "S"), "# S\n"));

    let snapshot = build(&Store::open(dir.path())).unwrap();
    assert_eq!(snapshot.records.len(), 2);
    assert!(snapshot.index_timestamp > 0, "stamped at build time");

    let index_path = dir.path().join(".odm").join("index");
    snapshot.persist(&index_path).unwrap();
    assert!(index_path.exists(), "the snapshot file is written");
}

// ----- B-8: built → persist → load round-trips identically -------------------

#[test]
fn cold_build_then_load_roundtrip() {
    let dir = TempDir::new().unwrap();
    let mut f = fm(1, NodeType::Slice, "S").with_tags(vec!["x".into()]);
    let gs = slice_gates();
    f.status_mut().set_gate(&gs, "planned", None, Evidence::Reproduced, day()).unwrap();
    seed(dir.path(), Document::new(f, "# S\nbody\n"));
    seed(dir.path(), Document::new(fm(2, NodeType::Project, "P"), "# P\n"));

    let built = build(&Store::open(dir.path())).unwrap();
    let index_path = dir.path().join(".odm").join("index");
    built.persist(&index_path).unwrap();

    match Snapshot::load(&index_path).unwrap() {
        Load::Loaded(back) => assert_eq!(back, built, "built-then-loaded round-trips identically"),
        Load::RebuildNeeded(r) => panic!("expected a clean load, got rebuild: {r:?}"),
    }
}
