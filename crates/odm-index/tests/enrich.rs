//! Tests for the slice04 record enrichment + in-memory maps (Arc 04).
//!
//! Test names carry the substrings the ledger Verify commands filter on
//! (`gates_carry_evidence`, `v1_index_triggers_rebuild`, `build_one_evidence`,
//! `meta_hash_tracks_evidence`, `inmemory_maps_built`).

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::gates::GateSet;
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use odm_index::record::IndexRecord;
use odm_index::{
    FORMAT_VERSION, IndexMaps, Load, RebuildReason, Snapshot, build_records, reconcile,
};
use odm_store::Store;
use tempfile::TempDir;

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 26).unwrap()
}

fn slice_gates() -> GateSet {
    GateSet::new(vec!["planned".to_string(), "built".to_string(), "tested".to_string()])
}

fn index_path(root: &std::path::Path) -> std::path::PathBuf {
    root.join(".odm").join("index")
}

/// Seeds a slice node with the given reached gates (gate, evidence).
fn seed_slice(store: &Store, number: u32, name: &str, reached: &[(&str, Evidence)]) -> Id {
    let mut f =
        Frontmatter::new(Id::new(), number, NodeType::Slice, name, day(), day(), Origin::Planned);
    let gs = slice_gates();
    for (gate, evidence) in reached {
        f.status_mut().set_gate(&gs, gate, None, *evidence, day()).unwrap();
    }
    let doc = Document::new(f, format!("# {name}\n"));
    let id = doc.frontmatter().id();
    store.persist(&doc).unwrap();
    id
}

fn record_for(records: &[IndexRecord], id: Id) -> &IndexRecord {
    records.iter().find(|r| r.id == id).expect("a record for the id")
}

// ----- I-1 / I-3: gates carry evidence; build_one populates it ---------------

#[test]
fn gates_carry_evidence_and_build_one_evidence() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = seed_slice(
        &store,
        1,
        "S",
        &[("planned", Evidence::Reconciled), ("built", Evidence::Attested)],
    );

    let records = build_records(&store).unwrap();
    let rec = record_for(&records, id);
    let gates: Vec<(&str, Evidence)> =
        rec.gates.iter().map(|g| (g.gate.as_str(), g.evidence)).collect();
    // Gate-name sorted (BTreeMap), each carrying its evidence level.
    assert_eq!(gates, [("built", Evidence::Attested), ("planned", Evidence::Reconciled)]);
}

// ----- I-3: meta_hash tracks gate evidence -----------------------------------

#[test]
fn meta_hash_tracks_evidence() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();

    // Persist the same node twice at the same id, differing only in the gate's
    // evidence level; the meta_hash must change.
    let write = |evidence: Evidence| {
        let mut f = Frontmatter::new(id, 1, NodeType::Slice, "S", day(), day(), Origin::Planned);
        f.status_mut().set_gate(&slice_gates(), "built", None, evidence, day()).unwrap();
        store.persist(&Document::new(f, "# S\n")).unwrap();
    };

    write(Evidence::Asserted);
    let before = record_for(&build_records(&store).unwrap(), id).meta_hash;
    write(Evidence::Reproduced);
    let after = record_for(&build_records(&store).unwrap(), id).meta_hash;

    assert_ne!(before, after, "an evidence change invalidates the meta_hash");
}

// ----- I-2: a v1 on-disk index self-heals to a rebuild -----------------------

#[test]
fn v1_index_triggers_rebuild() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    seed_slice(&store, 1, "S", &[("built", Evidence::Asserted)]);

    // Forge an on-disk index stamped with the old version 1: take a valid
    // current-version encoding, rewrite the format-version field to 1, and
    // re-checksum. (slice06's `v2_index_triggers_rebuild` carries the live
    // FORMAT_VERSION guard; this proves an even older format still self-heals.)
    let mut bytes =
        build_records(&store).map(|recs| Snapshot::new(0, recs).encode().unwrap()).unwrap();
    bytes[8] = 1; // FORMAT_VERSION low byte (u16 LE at offset 8)
    bytes[9] = 0;
    rechecksum(&mut bytes);
    std::fs::create_dir_all(index_path(dir.path()).parent().unwrap()).unwrap();
    std::fs::write(index_path(dir.path()), &bytes).unwrap();

    // Load sees the version mismatch (not a silent bad parse).
    match Snapshot::load(&index_path(dir.path())).unwrap() {
        Load::RebuildNeeded(RebuildReason::VersionMismatch { found }) => assert_eq!(found, 1),
        other => panic!("expected VersionMismatch, got {other:?}"),
    }

    // reconcile routes it through slice01's self-heal → a full cold rebuild.
    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert!(r.delta.rebuilt, "a v1 index rebuilds cold");
    assert_eq!(r.snapshot.records.len(), 1);
}

/// Recomputes the trailing SHA-256 checksum over the prefix after mutating it.
fn rechecksum(bytes: &mut [u8]) {
    use sha2::{Digest as _, Sha256};
    let split = bytes.len() - 32;
    let mut hasher = Sha256::new();
    hasher.update(&bytes[..split]);
    let digest = hasher.finalize();
    bytes[split..].copy_from_slice(&digest);
}

// ----- I-4: in-memory maps build from the records ----------------------------

#[test]
fn inmemory_maps_built() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());

    // A project + two slices; S1 is gated, tagged, and depends_on S2.
    let project =
        Frontmatter::new(Id::new(), 1, NodeType::Project, "P", day(), day(), Origin::Planned);
    let pid = project.id();
    store.persist(&Document::new(project, "# P\n")).unwrap();
    let s2 = seed_slice(&store, 3, "S2", &[]);

    let mut s1f =
        Frontmatter::new(Id::new(), 2, NodeType::Slice, "S1", day(), day(), Origin::Planned)
            .with_tags(vec!["store".to_string()]);
    s1f.status_mut().set_gate(&slice_gates(), "built", None, Evidence::Reproduced, day()).unwrap();
    s1f.edges_mut().depends_on.push(odm_core::frontmatter::Dependency::Bare(s2));
    let s1 = s1f.id();
    store.persist(&Document::new(s1f, "# S1\n")).unwrap();

    let records = build_records(&store).unwrap();
    let maps = IndexMaps::build(&records);

    assert_eq!(maps.ids_by_type(NodeType::Project), [pid]);
    assert_eq!(maps.ids_by_type(NodeType::Slice).len(), 2);
    assert!(maps.ids_by_gate("built").contains(&s1));
    assert!(maps.ids_by_gate("planned").is_empty(), "no node reached planned here");
    assert_eq!(maps.ids_by_tag("store"), [s1]);
    assert!(maps.ids_by_tag("absent").is_empty());
    // Edge adjacency: S1 → S2 (depends_on); S2 has none.
    assert_eq!(maps.edges_of(s1).len(), 1);
    assert_eq!(maps.edges_of(s1)[0].target, s2);
    assert!(maps.edges_of(s2).is_empty());
}

// ===== slice06: origin + decomposed enrichment =============================

/// Seeds an arc that affirms `decomposed` over `children`, with the given origin.
fn seed_arc(store: &Store, number: u32, origin: Origin, children: &[Id]) -> Id {
    let mut f = Frontmatter::new(Id::new(), number, NodeType::Arc, "A", day(), day(), origin);
    f.affirm_decomposed(children.to_vec(), day());
    let doc = Document::new(f, "# A\n");
    let id = doc.frontmatter().id();
    store.persist(&doc).unwrap();
    id
}

// ----- V-1 / V-2: the record carries origin + decomposed; build_one fills them

#[test]
fn record_carries_origin_decomposed() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let child = seed_slice(&store, 2, "S", &[]);
    let arc = seed_arc(&store, 1, Origin::Amendment, &[child]);

    let records = build_records(&store).unwrap();
    let arc_rec = record_for(&records, arc);
    assert_eq!(arc_rec.origin, Origin::Amendment, "origin carried");
    let d = arc_rec.decomposed.as_ref().expect("decomposed carried");
    assert_eq!(d.on, day());
    assert_eq!(d.children, vec![child]);

    // A node that never affirmed decomposition carries `None`.
    assert!(record_for(&records, child).decomposed.is_none());
}

#[test]
fn build_one_origin_decomposed() {
    // build_one (via build_records) populates both for a discovered, undecomposed node.
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let mut f =
        Frontmatter::new(Id::new(), 1, NodeType::Slice, "Found", day(), day(), Origin::Discovered);
    f.status_mut().set_gate(&slice_gates(), "built", None, Evidence::Reproduced, day()).unwrap();
    let id = f.id();
    store.persist(&Document::new(f, "# Found\n")).unwrap();

    let records = build_records(&store).unwrap();
    let rec = record_for(&records, id);
    assert_eq!(rec.origin, Origin::Discovered);
    assert!(rec.decomposed.is_none(), "a slice with no decomposition affirmed");
}

// ----- V-2: meta_hash tracks decomposed + origin -----------------------------

#[test]
fn meta_hash_tracks_decomposed_and_origin() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();

    let write = |origin: Origin, children: &[Id]| {
        let mut f = Frontmatter::new(id, 1, NodeType::Arc, "A", day(), day(), origin);
        if !children.is_empty() {
            f.affirm_decomposed(children.to_vec(), day());
        }
        store.persist(&Document::new(f, "# A\n")).unwrap();
    };

    let kid = Id::new();
    write(Origin::Planned, &[]);
    let base = record_for(&build_records(&store).unwrap(), id).meta_hash;

    // Changing origin alone flips the meta_hash (provenance is meaning).
    write(Origin::Discovered, &[]);
    let after_origin = record_for(&build_records(&store).unwrap(), id).meta_hash;
    assert_ne!(base, after_origin, "an origin change invalidates the meta_hash");

    // Affirming a decomposition flips it too (recomposition is meaning).
    write(Origin::Discovered, &[kid]);
    let after_decomp = record_for(&build_records(&store).unwrap(), id).meta_hash;
    assert_ne!(after_origin, after_decomp, "a decomposition change invalidates the meta_hash");
}

// ----- V-1: a v2 on-disk index self-heals to a rebuild (FORMAT_VERSION 3) -----

#[test]
fn v2_index_triggers_rebuild() {
    assert_eq!(FORMAT_VERSION, 3, "this slice bumped the format to v3");

    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    seed_slice(&store, 1, "S", &[("built", Evidence::Asserted)]);

    // Forge an on-disk index stamped with the old version 2.
    let mut bytes =
        build_records(&store).map(|recs| Snapshot::new(0, recs).encode().unwrap()).unwrap();
    bytes[8] = 2; // FORMAT_VERSION low byte (u16 LE at offset 8)
    bytes[9] = 0;
    rechecksum(&mut bytes);
    std::fs::create_dir_all(index_path(dir.path()).parent().unwrap()).unwrap();
    std::fs::write(index_path(dir.path()), &bytes).unwrap();

    match Snapshot::load(&index_path(dir.path())).unwrap() {
        Load::RebuildNeeded(RebuildReason::VersionMismatch { found }) => assert_eq!(found, 2),
        other => panic!("expected VersionMismatch(2), got {other:?}"),
    }
    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert!(r.delta.rebuilt, "a v2 index rebuilds cold");
}
