//! Tests for the index record + snapshot format (Arc 04 slice01).
//!
//! Test names carry the substrings the ledger Verify commands filter on
//! (`index_record_shape`, `snapshot_header_fields`, `snapshot_roundtrip`,
//! `snapshot_atomic_persist`, `corrupt_or_version_mismatch_signals_rebuild`).

use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::{Id, NodeType};
use odm_index::record::{EdgeKind, EdgeRef, IndexRecord};
use odm_index::snapshot::{FORMAT_VERSION, HASH_ALGO, Load, MAGIC, RebuildReason, Snapshot};
use proptest::prelude::*;
use tempfile::TempDir;
use ulid::Ulid;

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 26).unwrap()
}

fn id(tag: char) -> Id {
    Id::from_str(&format!("01ARZ3NDEKTSV4RRFFQ69G5F{tag}0")).unwrap()
}

/// A fully-populated sample record (every field non-default), for shape +
/// round-trip checks. Population from real files is slice02; here it is built by
/// hand.
fn sample() -> IndexRecord {
    IndexRecord {
        id: id('A'),
        rel_path: "nodes/2026/06/01ARZ3NDEKTSV4RRFFQ69G5FA0.md".to_string(),
        mtime_secs: 1_750_000_000,
        mtime_nsec: 123_456_789,
        size: 4096,
        inode: 99,
        mode: 0o100_644,
        content_hash: [0xAB; 32],
        meta_hash: [0xCD; 32],
        node_type: NodeType::Slice,
        gates: vec!["planned".to_string(), "built".to_string()],
        tags: vec!["store".to_string()],
        edges: vec![
            EdgeRef {
                target: id('B'),
                kind: EdgeKind::DependsOn,
                qualifier: Some(odm_index::EdgeQualifier::SatisfiedAt("tested".to_string())),
            },
            EdgeRef { target: id('C'), kind: EdgeKind::PartOf, qualifier: None },
        ],
        title: "Store layer".to_string(),
        updated: day(),
    }
}

// ----- F-2: the record carries every field group ----------------------------

#[test]
fn index_record_shape_carries_all_fields() {
    let r = sample();
    // identity
    assert_eq!(r.id, id('A'));
    assert!(r.rel_path.ends_with(".md"));
    // stat cache
    assert_eq!(r.mtime_secs, 1_750_000_000);
    assert_eq!(r.mtime_nsec, 123_456_789);
    assert_eq!(r.size, 4096);
    assert_eq!(r.inode, 99);
    assert_eq!(r.mode, 0o100_644);
    // fingerprints (both present, distinct)
    assert_eq!(r.content_hash.len(), 32);
    assert_eq!(r.meta_hash.len(), 32);
    assert_ne!(r.content_hash, r.meta_hash);
    // extracted metadata
    assert_eq!(r.node_type, NodeType::Slice);
    assert_eq!(r.gates, ["planned", "built"]);
    assert_eq!(r.tags, ["store"]);
    assert_eq!(r.edges.len(), 2);
    assert_eq!(r.edges[0].kind, EdgeKind::DependsOn);
    assert_eq!(r.title, "Store layer");
    assert_eq!(r.updated, day());
}

// ----- F-3: the header carries all six elements ------------------------------

#[test]
fn snapshot_header_fields_present() {
    let snap = Snapshot::new(1_750_000_123, vec![sample(), sample()]);

    // The logical header: version, hash-algo, index-timestamp, record count.
    let h = snap.header();
    assert_eq!(h.format_version, FORMAT_VERSION);
    assert_eq!(h.hash_algo, HASH_ALGO);
    assert_eq!(h.index_timestamp, 1_750_000_123);
    assert_eq!(h.record_count, 2);

    // The on-disk header sits at fixed offsets: magic, version, algo.
    let bytes = snap.encode().unwrap();
    assert_eq!(bytes[0..8], MAGIC, "magic leads the file");
    assert_eq!(u16::from_le_bytes([bytes[8], bytes[9]]), FORMAT_VERSION);
    assert_eq!(bytes[10], HASH_ALGO.as_u8());
    // A trailing checksum follows the body: prefix(11) + body + checksum(32).
    assert!(bytes.len() >= 11 + 32);

    // The index-timestamp + count survive a round-trip (header is real, not
    // cosmetic).
    let back = Snapshot::decode(&bytes).unwrap();
    assert_eq!(back.index_timestamp, 1_750_000_123);
    assert_eq!(back.records.len(), 2);
}

// ----- F-4: encode ∘ decode = identity (proptest) ----------------------------

prop_compose! {
    fn arb_id()(n in any::<u128>()) -> Id {
        // A u128 is always a valid ULID; round-trip through its canonical string
        // (the only public Id constructor besides the random `new`).
        Id::from_str(&Ulid::from(n).to_string()).unwrap()
    }
}

fn arb_edge_kind() -> impl Strategy<Value = EdgeKind> {
    prop_oneof![
        Just(EdgeKind::PartOf),
        Just(EdgeKind::DependsOn),
        Just(EdgeKind::BlockedBy),
        Just(EdgeKind::Verifies),
        Just(EdgeKind::Consumes),
        Just(EdgeKind::Affects),
        Just(EdgeKind::Supersedes),
        Just(EdgeKind::Tears),
    ]
}

fn arb_node_type() -> impl Strategy<Value = NodeType> {
    prop_oneof![
        Just(NodeType::Project),
        Just(NodeType::Arc),
        Just(NodeType::Slice),
        Just(NodeType::Odd),
        Just(NodeType::Adr),
        Just(NodeType::Note),
    ]
}

prop_compose! {
    fn arb_date()(y in 1970i32..2100, m in 1u32..=12, d in 1u32..=28) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }
}

prop_compose! {
    fn arb_record()(
        id in arb_id(),
        rel_path in "nodes/[0-9]{4}/[0-9]{2}/[A-Z0-9]{10}\\.md",
        mtime_secs in any::<i64>(),
        mtime_nsec in any::<u32>(),
        size in any::<u64>(),
        inode in any::<u64>(),
        mode in any::<u32>(),
        content_hash in proptest::array::uniform32(any::<u8>()),
        meta_hash in proptest::array::uniform32(any::<u8>()),
        node_type in arb_node_type(),
        gates in proptest::collection::vec("[a-z-]{1,12}", 0..4),
        tags in proptest::collection::vec("[a-z]{1,8}", 0..4),
        edges in proptest::collection::vec((arb_id(), arb_edge_kind()), 0..5),
        title in ".{0,40}",
        updated in arb_date(),
    ) -> IndexRecord {
        IndexRecord {
            id, rel_path, mtime_secs, mtime_nsec, size, inode, mode,
            content_hash, meta_hash, node_type, gates, tags,
            edges: edges
                .into_iter()
                .map(|(target, kind)| EdgeRef { target, kind, qualifier: None })
                .collect(),
            title, updated,
        }
    }
}

proptest! {
    #[test]
    fn snapshot_roundtrip_encode_decode_identity(
        index_timestamp in any::<i64>(),
        records in proptest::collection::vec(arb_record(), 0..20),
    ) {
        let snap = Snapshot::new(index_timestamp, records);
        let bytes = snap.encode().unwrap();
        let back = Snapshot::decode(&bytes).unwrap();
        prop_assert_eq!(snap, back);
    }
}

// ----- F-5: atomic persistence reuses the store's writer ---------------------

#[test]
fn snapshot_atomic_persist_and_reload() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join(".odm").join("index");
    let snap = Snapshot::new(1_750_000_000, vec![sample(), sample(), sample()]);

    // Persist creates parent dirs (store's atomic::write does) and writes the file.
    snap.persist(&path).unwrap();
    assert!(path.exists(), "the snapshot file is written");

    // It reloads identically.
    match Snapshot::load(&path).unwrap() {
        Load::Loaded(back) => assert_eq!(back, snap),
        Load::RebuildNeeded(r) => panic!("expected a clean reload, got rebuild: {r:?}"),
    }
}

// ----- F-6: corruption / version mismatch / absence signal rebuild -----------

#[test]
fn corrupt_or_version_mismatch_signals_rebuild() {
    let snap = Snapshot::new(42, vec![sample()]);
    let good = snap.encode().unwrap();

    // A flipped body byte fails the checksum → rebuild, not a bad parse.
    let mut corrupt = good.clone();
    let mid = corrupt.len() / 2;
    corrupt[mid] ^= 0xFF;
    assert_eq!(Snapshot::decode(&corrupt), Err(RebuildReason::BadChecksum));

    // A truncated file (shorter than the minimum) → rebuild.
    assert_eq!(Snapshot::decode(&good[..5]), Err(RebuildReason::TooShort));

    // Wrong magic → rebuild. (Re-checksum so we reach the magic check, proving
    // it is detected independently of the checksum.)
    let mut bad_magic = good.clone();
    bad_magic[0] = b'X';
    rechecksum(&mut bad_magic);
    assert_eq!(Snapshot::decode(&bad_magic), Err(RebuildReason::BadMagic));

    // A future format version → rebuild with the found version.
    let mut bad_version = good.clone();
    let next = (FORMAT_VERSION + 1).to_le_bytes();
    bad_version[8] = next[0];
    bad_version[9] = next[1];
    rechecksum(&mut bad_version);
    assert_eq!(
        Snapshot::decode(&bad_version),
        Err(RebuildReason::VersionMismatch { found: FORMAT_VERSION + 1 })
    );

    // An unknown hash-algo id → rebuild.
    let mut bad_algo = good.clone();
    bad_algo[10] = 0xEE;
    rechecksum(&mut bad_algo);
    assert_eq!(Snapshot::decode(&bad_algo), Err(RebuildReason::UnknownHashAlgo(0xEE)));

    // An absent file → rebuild (Missing), never an error.
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("nope").join("index");
    match Snapshot::load(&missing).unwrap() {
        Load::RebuildNeeded(RebuildReason::Missing) => {}
        other => panic!("expected RebuildNeeded(Missing), got {other:?}"),
    }
}

// ----- F-6 (cont.): a corrupt file on disk self-heals through `load` ---------

#[test]
fn corrupt_or_version_mismatch_signals_rebuild_through_load() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join(".odm").join("index");
    Snapshot::new(7, vec![sample()]).persist(&path).unwrap();

    // Corrupt the persisted bytes in place, then load: a rebuild signal, not a
    // bad parse and not an error.
    let mut bytes = std::fs::read(&path).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;
    std::fs::write(&path, &bytes).unwrap();

    match Snapshot::load(&path).unwrap() {
        Load::RebuildNeeded(RebuildReason::BadChecksum) => {}
        other => panic!("expected RebuildNeeded(BadChecksum), got {other:?}"),
    }
}

// ----- F-6 (cont.): a genuine read error (not absence) propagates ------------

#[test]
fn load_read_error_propagates() {
    // A directory at the index path is not "missing" — `read` errors with a
    // kind other than NotFound, which must surface as IndexError::Read.
    let dir = TempDir::new().unwrap();
    let err = Snapshot::load(dir.path()).unwrap_err();
    assert!(matches!(err, odm_index::IndexError::Read { .. }), "got {err:?}");
}

/// Recomputes the trailing SHA-256 checksum after mutating the prefix, so a test
/// can isolate a non-checksum failure (magic/version/algo) from the checksum
/// guard. Mirrors `snapshot::checksum_of` (kept here, the helper is private).
fn rechecksum(bytes: &mut [u8]) {
    use sha2::{Digest as _, Sha256};
    let split = bytes.len() - 32;
    let mut hasher = Sha256::new();
    hasher.update(&bytes[..split]);
    let digest = hasher.finalize();
    bytes[split..].copy_from_slice(&digest);
}
