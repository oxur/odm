//! Tests for edge satisfaction, evidence ordering, the threshold, the staleness
//! guard, and the `NodeGraph` derived-order bridge.

use std::collections::BTreeSet;
use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::frontmatter::{Dependency, Frontmatter};
use odm_core::gates::GateSets;
use odm_core::graph::NodeGraph;
use odm_core::satisfaction::{
    Satisfaction, edge_satisfaction, is_soft, staleness_on_advance, threshold_from_toml,
};
use odm_core::status::{Evidence, Status};
use odm_core::{Id, NodeType, Origin};
use odm_graph::Block;

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 24).unwrap()
}

const CONFIG: &str = "\
[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

fn gates() -> GateSets {
    GateSets::from_toml_str(CONFIG).unwrap()
}

fn slice(id: Id, number: u32, name: &str) -> Frontmatter {
    Frontmatter::new(id, number, NodeType::Slice, name, day(), day(), Origin::Planned)
}

fn id(tag: char) -> Id {
    Id::from_str(&format!("01ARZ3NDEKTSV4RRFFQ69G5F{tag}0")).unwrap()
}

// ----- H-5: satisfaction at the satisfied_at (default terminal) gate ---------

#[test]
fn satisfaction_gate() {
    let gates = gates();
    let gset = gates.for_type(NodeType::Slice).unwrap();
    let mut status = Status::new();

    // Nothing reached: unsatisfied at the default (terminal) gate.
    assert_eq!(edge_satisfaction(&status, gset, None), None);

    // Reaching the terminal gate satisfies a bare (default) dependency.
    status.set_gate(gset, "tested", None, Evidence::Reproduced, day()).unwrap();
    assert_eq!(edge_satisfaction(&status, gset, None), Some(Evidence::Reproduced));

    // An explicit `satisfied_at` targets that specific gate.
    assert_eq!(edge_satisfaction(&status, gset, Some("built")), None);
    status.set_gate(gset, "built", None, Evidence::Asserted, day()).unwrap();
    assert_eq!(edge_satisfaction(&status, gset, Some("built")), Some(Evidence::Asserted));
}

// ----- H-6: evidence ordering -----------------------------------------------

#[test]
fn evidence_ordering() {
    use Evidence::{Asserted, Attested, Reconciled, Reproduced};
    assert!(Asserted < Attested && Attested < Reproduced && Reproduced < Reconciled);
    let mut levels = [Reproduced, Asserted, Reconciled, Attested];
    levels.sort();
    assert_eq!(levels, [Asserted, Attested, Reproduced, Reconciled]);
}

// ----- H-8: threshold (default + override) + soft classification ------------

#[test]
fn satisfaction_threshold() {
    // Default is `reproduced` when [satisfaction] is absent.
    assert_eq!(
        threshold_from_toml("[gates.slice]\nsequence=[\"a\"]").unwrap(),
        Evidence::Reproduced
    );
    // Configurable override.
    assert_eq!(
        threshold_from_toml("[satisfaction]\nthreshold = \"attested\"").unwrap(),
        Evidence::Attested
    );
    // Below-threshold ⇒ soft; at/above ⇒ not.
    assert!(is_soft(Evidence::Attested, Evidence::Reproduced));
    assert!(!is_soft(Evidence::Reproduced, Evidence::Reproduced));
    assert!(!is_soft(Evidence::Reconciled, Evidence::Reproduced));
    // A malformed threshold is an error.
    assert!(threshold_from_toml("[satisfaction]\nthreshold = \"guessed\"").is_err());
}

// ----- H-11: staleness guard ------------------------------------------------

#[test]
fn staleness_guard() {
    // Advancing with an unsatisfied dep warns (names the deps); none → no warn.
    let warn = staleness_on_advance(id('A'), vec![id('B'), id('C')]).expect("warns");
    assert_eq!(warn.node, id('A'));
    assert_eq!(warn.unsatisfied.len(), 2);
    assert!(staleness_on_advance(id('A'), vec![]).is_none());
}

// ----- integration: Satisfaction::compute + NodeGraph derived order ---------

#[test]
fn derived_order_over_a_corpus() {
    let gates = gates();
    let gset = gates.for_type(NodeType::Slice).unwrap();

    // B is fully done (reached terminal `tested` at Reproduced).
    let mut b = slice(id('B'), 2, "B");
    b.status_mut().set_gate(gset, "tested", None, Evidence::Reproduced, day()).unwrap();
    // A depends on B.
    let mut a = slice(id('A'), 1, "A");
    a.edges_mut().depends_on = vec![Dependency::Bare(id('B'))];

    let corpus = vec![a.clone(), b.clone()];
    let graph = NodeGraph::build(&corpus);
    let sat = Satisfaction::compute(&corpus, &gates, Evidence::Reproduced);

    // Topo lists B before A.
    let topo = graph.topological_order(&[]).unwrap();
    let pos = |x: Id| topo.iter().position(|&n| n == x).unwrap();
    assert!(pos(id('B')) < pos(id('A')));

    // next: A is ready (dep satisfied at Reproduced ≥ threshold, no soft); B is
    // complete so excluded.
    let frontier: BTreeSet<Id> = graph.next(&sat).into_iter().map(|r| r.node).collect();
    assert_eq!(frontier, [id('A')].into_iter().collect());
    assert!(graph.next(&sat).iter().find(|r| r.node == id('A')).unwrap().soft.is_empty());

    // A's effective evidence = the dep edge level.
    assert_eq!(graph.min_evidence(id('A'), &sat), Some(Evidence::Reproduced));
    // Path A -> B.
    assert_eq!(graph.path(id('A'), Some(id('B')), &[]), Some(vec![id('A'), id('B')]));
    assert_eq!(graph.path(id('A'), None, &[]), Some(vec![id('A'), id('B')]));

    // Now B is only attested (< threshold): A stays ready but soft-flagged, and
    // `blocked A` names the low-evidence dep.
    let mut b_low = slice(id('B'), 2, "B");
    b_low.status_mut().set_gate(gset, "tested", None, Evidence::Attested, day()).unwrap();
    let corpus = vec![a, b_low];
    let graph = NodeGraph::build(&corpus);
    let sat = Satisfaction::compute(&corpus, &gates, Evidence::Reproduced);

    let ready = graph.next(&sat);
    let a_ready = ready.iter().find(|r| r.node == id('A')).expect("A still listed (non-blocking)");
    assert_eq!(a_ready.soft.len(), 1, "soft-flagged");
    let reasons = graph.blocked(id('A'), &sat);
    assert!(matches!(
        reasons.as_slice(),
        [Block::SoftSatisfied { evidence: Evidence::Attested, .. }]
    ));
}
