//! Tests for the abstract graph engine. Test code is exempt from the
//! domain-agnostic source grep, so neutral local types are used freely.

use std::collections::BTreeSet;

use odm_graph::Graph;
use proptest::prelude::*;

/// A tiny edge-kind for tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Red,
    Blue,
}

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

// ----- H-5: forward/reverse accessors by edge kind --------------------------

#[test]
fn adjacency_by_kind() {
    // 1 -Red-> 2, 1 -Blue-> 3, 4 -Red-> 2
    let g = build(&[1, 2, 3, 4], &[(1, Kind::Red, 2), (1, Kind::Blue, 3), (4, Kind::Red, 2)]);

    let set = |v: Vec<u32>| v.into_iter().collect::<BTreeSet<_>>();

    // Forward, filtered by kind.
    assert_eq!(set(g.successors(&1, &Kind::Red)), set(vec![2]));
    assert_eq!(set(g.successors(&1, &Kind::Blue)), set(vec![3]));
    assert!(g.successors(&2, &Kind::Red).is_empty());

    // Reverse (derived), filtered by kind: who points at 2 via Red?
    assert_eq!(set(g.predecessors(&2, &Kind::Red)), set(vec![1, 4]));
    assert!(g.predecessors(&2, &Kind::Blue).is_empty());

    // All outgoing of 1, with kinds.
    let outgoing: std::collections::HashSet<(Kind, u32)> = g.outgoing(&1).into_iter().collect();
    let expected: std::collections::HashSet<(Kind, u32)> =
        [(Kind::Red, 2), (Kind::Blue, 3)].into_iter().collect();
    assert_eq!(outgoing, expected);
}

#[test]
fn add_node_is_idempotent_one_index_per_id() {
    let mut g: Graph<u32, Kind> = Graph::new();
    assert!(g.add_node(7));
    assert!(!g.add_node(7), "re-adding the same id is a no-op");
    assert_eq!(g.node_count(), 1);
    assert!(g.contains(&7));
}

#[test]
fn add_edge_requires_known_endpoints() {
    let mut g: Graph<u32, Kind> = Graph::new();
    g.add_node(1);
    assert!(!g.add_edge(&1, Kind::Red, &2), "unknown destination → no edge");
    assert!(!g.add_edge(&9, Kind::Red, &1), "unknown source → no edge");
    assert_eq!(g.edge_count(), 0);
    g.add_node(2);
    assert!(g.add_edge(&1, Kind::Red, &2));
    assert_eq!(g.edge_count(), 1);
}

#[test]
fn nodes_and_default() {
    let mut g: Graph<u32, Kind> = Graph::default();
    g.add_node(10);
    g.add_node(20);
    let ids: BTreeSet<u32> = g.nodes().copied().collect();
    assert_eq!(ids, [10, 20].into_iter().collect());
}

#[test]
fn missing_node_queries_are_empty() {
    let g: Graph<u32, Kind> = Graph::new();
    assert!(g.successors(&1, &Kind::Red).is_empty());
    assert!(g.predecessors(&1, &Kind::Red).is_empty());
    assert!(g.outgoing(&1).is_empty());
}

// ----- H-2: reverse adjacency is the transpose of forward -------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn reverse_edges_is_transpose(
        node_count in 1u32..12,
        raw_edges in prop::collection::vec((0u32..12, any::<bool>(), 0u32..12), 0..40),
    ) {
        let nodes: Vec<u32> = (0..node_count).collect();
        let edges: Vec<(u32, Kind, u32)> = raw_edges
            .into_iter()
            .filter(|&(a, _, b)| a < node_count && b < node_count)
            .map(|(a, red, b)| (a, if red { Kind::Red } else { Kind::Blue }, b))
            .collect();
        let g = build(&nodes, &edges);

        // For every ordered pair and kind: b is a successor of a  iff  a is a
        // predecessor of b. (Reverse adjacency == transpose of forward.)
        for &a in &nodes {
            for &b in &nodes {
                for kind in [Kind::Red, Kind::Blue] {
                    let fwd = g.successors(&a, &kind).contains(&b);
                    let rev = g.predecessors(&b, &kind).contains(&a);
                    prop_assert_eq!(fwd, rev, "a={} b={} kind={:?}", a, b, kind);
                }
            }
        }
    }
}
