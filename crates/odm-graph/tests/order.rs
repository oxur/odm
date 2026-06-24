//! Tests for derived-order queries and evidence-leveled satisfaction. Neutral
//! test types (the domain-agnostic source grep does not cover tests).

use std::collections::{HashMap, HashSet};

use odm_graph::{Block, Graph, OrderInputs, SoftDep};

/// Edge kinds: `Dep` is ordering, `Block` is blocked-by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dep,
    Block,
}

/// A neutral, totally-ordered confidence level (stands in for `Evidence`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Low,
    Mid,
    High,
}

const ORDERING: &[Kind] = &[Kind::Dep];
const BLOCKS: &[Kind] = &[Kind::Block];

fn build(nodes: &[u32], edges: &[(u32, Kind, u32)]) -> Graph<u32, Kind> {
    let mut g = Graph::new();
    for &n in nodes {
        g.add_node(n);
    }
    for &(a, k, b) in edges {
        g.add_edge(&a, k, &b);
    }
    g
}

fn inputs<'a>(
    complete: &'a HashSet<u32>,
    satisfied: &'a HashMap<(u32, u32), Level>,
    threshold: Level,
) -> OrderInputs<'a, u32, Kind, Level> {
    OrderInputs {
        ordering_kinds: ORDERING,
        block_kinds: BLOCKS,
        tears: &[],
        complete,
        satisfied,
        threshold,
    }
}

// ----- H-1: topological order -----------------------------------------------

#[test]
fn topo_order() {
    // 3 depends on 2 depends on 1: order must list 1 before 2 before 3.
    let g = build(&[1, 2, 3], &[(3, Kind::Dep, 2), (2, Kind::Dep, 1)]);
    let order = g.topological_order(ORDERING, &[]).expect("acyclic");
    let pos = |x: u32| order.iter().position(|&n| n == x).unwrap();
    assert!(pos(1) < pos(2) && pos(2) < pos(3));
    assert_eq!(order.len(), 3);

    // A cycle has no order.
    let cyclic = build(&[1, 2], &[(1, Kind::Dep, 2), (2, Kind::Dep, 1)]);
    assert!(cyclic.topological_order(ORDERING, &[]).is_err());
}

// ----- H-2 / H-10: next (ready frontier), soft does not block ---------------

#[test]
fn next_ready_frontier() {
    // 2 -Dep-> 1. 1 has no deps; 2 depends on 1.
    let g = build(&[1, 2], &[(2, Kind::Dep, 1)]);
    let complete = HashSet::new();
    let mut satisfied = HashMap::new();

    // Nothing satisfied: 1 is ready (no deps); 2 is not (dep 1 unsatisfied).
    let frontier: Vec<u32> =
        g.next(&inputs(&complete, &satisfied, Level::Mid)).iter().map(|r| r.node).collect();
    assert_eq!(frontier, vec![1]);

    // Satisfy 2->1 at High: now both 1 and 2 are ready.
    satisfied.insert((2, 1), Level::High);
    let frontier: Vec<u32> =
        g.next(&inputs(&complete, &satisfied, Level::Mid)).iter().map(|r| r.node).collect();
    assert_eq!(frontier, vec![1, 2]);
}

#[test]
fn soft_satisfied_not_blocking() {
    // 2 depends on 1, satisfied only at Low (< threshold Mid). 2 must still be
    // in `next` (visibility, not gating) — flagged as soft.
    let g = build(&[1, 2], &[(2, Kind::Dep, 1)]);
    let complete = HashSet::new();
    let mut satisfied = HashMap::new();
    satisfied.insert((2, 1), Level::Low);

    let ready = g.next(&inputs(&complete, &satisfied, Level::Mid));
    let two = ready.iter().find(|r| r.node == 2).expect("2 is still listed");
    assert_eq!(two.soft, vec![SoftDep { dep: 1, evidence: Level::Low }]);
}

#[test]
fn next_excludes_complete_and_blocked() {
    // 1 complete (excluded). 2 blocked-by 3 (3 not complete) → withheld.
    let g = build(&[1, 2, 3], &[(2, Kind::Block, 3)]);
    let mut complete = HashSet::new();
    complete.insert(1u32);
    let satisfied = HashMap::new();

    let frontier: Vec<u32> =
        g.next(&inputs(&complete, &satisfied, Level::Mid)).iter().map(|r| r.node).collect();
    // 1 complete → out; 2 blocked → out; 3 ready (no deps, not blocked).
    assert_eq!(frontier, vec![3]);

    // Once 3 is complete, the block clears and 2 becomes ready.
    complete.insert(3u32);
    let frontier: Vec<u32> =
        g.next(&inputs(&complete, &satisfied, Level::Mid)).iter().map(|r| r.node).collect();
    assert_eq!(frontier, vec![2]);
}

// ----- H-3 / H-9: blocked reasons + soft surfacing --------------------------

#[test]
fn blocked_reasons() {
    // 3 depends on 1 (unsatisfied) and 2 (satisfied High); blocked-by 4.
    let g = build(&[1, 2, 3, 4], &[(3, Kind::Dep, 1), (3, Kind::Dep, 2), (3, Kind::Block, 4)]);
    let complete = HashSet::new();
    let mut satisfied = HashMap::new();
    satisfied.insert((3, 2), Level::High);

    let reasons = g.blocked(&3, &inputs(&complete, &satisfied, Level::Mid));
    assert!(reasons.contains(&Block::Unsatisfied { dep: 1 }));
    assert!(reasons.contains(&Block::ExternallyBlocked { by: 4 }));
    // Dep 2 is fully satisfied (High ≥ Mid) → not a reason.
    assert!(!reasons.iter().any(|r| matches!(r, Block::SoftSatisfied { dep: 2, .. })));
}

#[test]
fn soft_satisfied_surfacing() {
    // 2 depends on 1, satisfied only at Low. `blocked 2` must name the
    // low-evidence dep and the threshold to raise it to.
    let g = build(&[1, 2], &[(2, Kind::Dep, 1)]);
    let complete = HashSet::new();
    let mut satisfied = HashMap::new();
    satisfied.insert((2, 1), Level::Low);

    let reasons = g.blocked(&2, &inputs(&complete, &satisfied, Level::Mid));
    assert_eq!(
        reasons,
        vec![Block::SoftSatisfied { dep: 1, evidence: Level::Low, threshold: Level::Mid }]
    );
}

// ----- H-4: path / critical chain -------------------------------------------

#[test]
fn path_chain() {
    // 4 -> 3 -> 1, and 3 -> 2 (a shorter branch). Longest chain from 4 is
    // 4,3,1 or 4,3,2 (both length 3). Path 4 -> 1 exists; 4 -> 99 does not.
    let g = build(&[1, 2, 3, 4], &[(4, Kind::Dep, 3), (3, Kind::Dep, 1), (3, Kind::Dep, 2)]);
    let g99 = build(&[1, 4], &[]);

    let chain = g.path(&4, None, ORDERING, &[]).unwrap();
    assert_eq!(chain.first(), Some(&4));
    assert_eq!(chain.len(), 3, "critical chain has three nodes");

    let between = g.path(&4, Some(&1), ORDERING, &[]).unwrap();
    assert_eq!(between, vec![4, 3, 1]);

    assert!(g99.path(&1, Some(&4), ORDERING, &[]).is_none(), "no path");
}

// ----- H-7: evidence min-propagation ----------------------------------------

#[test]
fn evidence_min_propagation() {
    // 3 -> 2 -> 1, satisfied at High then High: effective = High.
    let g = build(&[1, 2, 3], &[(3, Kind::Dep, 2), (2, Kind::Dep, 1)]);
    let mut satisfied = HashMap::new();
    satisfied.insert((3, 2), Level::High);
    satisfied.insert((2, 1), Level::High);
    assert_eq!(g.min_evidence(&3, ORDERING, &[], &satisfied), Some(Level::High));

    // Lower one link to Low: 3's effective evidence drops to Low.
    satisfied.insert((2, 1), Level::Low);
    assert_eq!(g.min_evidence(&3, ORDERING, &[], &satisfied), Some(Level::Low));

    // A node with no satisfied dep has no effective evidence.
    assert_eq!(g.min_evidence(&1, ORDERING, &[], &satisfied), None);
}

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn min_propagation_is_monotone(
        // A simple chain 0 -> 1 -> ... -> n, each link a random level.
        levels in prop::collection::vec(prop_oneof![Just(Level::Low), Just(Level::Mid), Just(Level::High)], 1..8),
    ) {
        let n = levels.len();
        let nodes: Vec<u32> = (0..=n as u32).collect();
        let edges: Vec<(u32, Kind, u32)> = (0..n as u32).map(|i| (i, Kind::Dep, i + 1)).collect();
        let g = build(&nodes, &edges);
        let satisfied: HashMap<(u32, u32), Level> =
            (0..n as u32).map(|i| ((i, i + 1), levels[i as usize])).collect();

        // Effective evidence of the head = the minimum link in the chain.
        let expected = levels.iter().copied().min();
        prop_assert_eq!(g.min_evidence(&0, ORDERING, &[], &satisfied), expected);
    }
}
