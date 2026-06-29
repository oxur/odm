//! Tests for the slice08 seeded synthetic-corpus generator (Arc 04 capstone).
//!
//! Guards the benchmark's two preconditions: the corpus is **reproducible** (same
//! seed → identical corpus) and **realistic** (it exercises the real build /
//! adapter / graph paths — a `part_of` forest, edges, gates+evidence, varied
//! `origin`, some `decomposed`). Test name carries the ledger Verify substring
//! (`synthetic_corpus_is_seeded_and_realistic`).

use odm_core::gates::GateSets;
use odm_core::graph::NodeGraph;
use odm_core::satisfaction::Satisfaction;
use odm_core::status::Evidence;
use odm_core::{NodeType, Origin};
use odm_index::record::IndexRecord;
use odm_index::{build_records, frontmatters_from_records, synth};
use odm_store::Store;
use tempfile::TempDir;

#[test]
fn synthetic_corpus_is_seeded_and_realistic() {
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    let c = TempDir::new().unwrap();

    // --- Determinism: same seed → identical ids; a different seed differs. -----
    let ids_a = synth::generate_corpus(&Store::open(a.path()), 200, 7);
    let ids_b = synth::generate_corpus(&Store::open(b.path()), 200, 7);
    let ids_c = synth::generate_corpus(&Store::open(c.path()), 200, 9);
    assert_eq!(ids_a, ids_b, "same seed → identical ids (reproducible corpus)");
    assert_ne!(ids_a, ids_c, "a different seed → a different corpus");
    assert_eq!(ids_a.len(), 200);

    // Identical *semantic content* too: the (id, meta_hash) projection matches
    // across the two seed-7 corpora (stat fields differ by write time; meaning
    // does not).
    let recs_a = build_records(&Store::open(a.path())).unwrap();
    let recs_b = build_records(&Store::open(b.path())).unwrap();
    let semantic = |r: &[IndexRecord]| r.iter().map(|x| (x.id, x.meta_hash)).collect::<Vec<_>>();
    assert_eq!(semantic(&recs_a), semantic(&recs_b), "same seed → identical records");

    // --- Realism: the corpus exercises every real field/path. ------------------
    assert_eq!(recs_a.len(), 200);
    let count = |t: NodeType| recs_a.iter().filter(|r| r.node_type == t).count();
    assert_eq!(count(NodeType::Project), 1, "exactly one project root");
    let (arcs, slices) = (count(NodeType::Arc), count(NodeType::Slice));
    assert!(arcs >= 1 && slices > arcs, "a forest: {arcs} arcs + {slices} slices");

    assert!(recs_a.iter().any(|r| !r.edges.is_empty()), "edges present");
    assert!(recs_a.iter().any(|r| !r.gates.is_empty()), "reached gates present");
    assert!(recs_a.iter().any(|r| r.decomposed.is_some()), "some affirmed decomposition");
    for origin in [Origin::Planned, Origin::Discovered, Origin::Amendment] {
        assert!(recs_a.iter().any(|r| r.origin == origin), "origin {origin:?} present");
    }

    // The real consumer pipeline (the bench's read path) runs over the corpus.
    let gates = GateSets::from_toml_str(synth::GATE_CONFIG).unwrap();
    let fms = frontmatters_from_records(&recs_a, &gates);
    let graph = NodeGraph::build(&fms);
    let sat = Satisfaction::compute(&fms, &gates, Evidence::Reproduced);
    let _ready = graph.next(&sat);
}

#[test]
fn synthetic_corpus_minimal_does_not_panic() {
    // n = 1: just the project root (no arcs/slices, no decomposed children).
    let d1 = TempDir::new().unwrap();
    let ids1 = synth::generate_corpus(&Store::open(d1.path()), 1, 1);
    assert_eq!(ids1.len(), 1);
    assert_eq!(build_records(&Store::open(d1.path())).unwrap().len(), 1);

    // n = 2: project + one arc, still no slices (the slice branch is skipped).
    let d2 = TempDir::new().unwrap();
    let ids2 = synth::generate_corpus(&Store::open(d2.path()), 2, 1);
    assert_eq!(ids2.len(), 2);
}
