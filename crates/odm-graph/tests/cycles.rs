//! Tests for cycle detection and tears. Test types are neutral (the
//! domain-agnostic source grep does not cover test code).

use std::collections::BTreeSet;

use odm_graph::{Cycle, Graph, MissingRationale, Tear};

/// A tiny edge-kind: `Dep` is the ordering relation; `Other` is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dep,
    Other,
}

const ORDERING: &[Kind] = &[Kind::Dep];

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

fn members(c: &Cycle<u32>) -> BTreeSet<u32> {
    c.members().iter().copied().collect()
}

// ----- H-1: detect a cycle and name its members -----------------------------

#[test]
fn detect_cycle_names_members() {
    // 1 -> 2 -> 3 -> 1 (all Dep): a 3-cycle.
    let g = build(&[1, 2, 3], &[(1, Kind::Dep, 2), (2, Kind::Dep, 3), (3, Kind::Dep, 1)]);
    let cycle = g.detect_cycle(ORDERING, &[]).expect("a cycle");
    assert_eq!(members(&cycle), [1, 2, 3].into_iter().collect());
}

#[test]
fn detect_cycle_excludes_innocent_downstream() {
    // 1 -> 2 -> 1 is the cycle; 3 merely depends on the cycle (2 -> 3).
    let g = build(&[1, 2, 3], &[(1, Kind::Dep, 2), (2, Kind::Dep, 1), (2, Kind::Dep, 3)]);
    let cycle = g.detect_cycle(ORDERING, &[]).expect("a cycle");
    // Only the true members (1, 2) — not the downstream node 3.
    assert_eq!(members(&cycle), [1, 2].into_iter().collect());
}

#[test]
fn non_ordering_edges_do_not_form_cycles() {
    // 1 -> 2 -> 1 but via `Other` edges: not part of the ordering relation.
    let g = build(&[1, 2], &[(1, Kind::Other, 2), (2, Kind::Other, 1)]);
    assert!(g.detect_cycle(ORDERING, &[]).is_none());
}

#[test]
fn self_loop_is_a_cycle() {
    // A node that depends on itself is a 1-cycle.
    let g = build(&[1], &[(1, Kind::Dep, 1)]);
    let cycle = g.detect_cycle(ORDERING, &[]).expect("a self-cycle");
    assert_eq!(members(&cycle), [1].into_iter().collect());
    // Tearing the self-edge clears it.
    let tear = Tear::new(1, 1, "self-reference is intentional").unwrap();
    assert!(g.detect_cycle(ORDERING, &[tear]).is_none());
}

#[test]
fn cycle_found_after_acyclic_components() {
    // Two acyclic nodes (10, 11) explored first, then a separate cycle (1->2->1).
    // Exercises DFS continuing past finished roots to a later component.
    let g = build(&[10, 11, 1, 2], &[(10, Kind::Dep, 11), (1, Kind::Dep, 2), (2, Kind::Dep, 1)]);
    let cycle = g.detect_cycle(ORDERING, &[]).expect("a cycle");
    assert_eq!(members(&cycle), [1, 2].into_iter().collect());
}

// ----- H-2: acyclic reports no cycle ----------------------------------------

#[test]
fn acyclic_no_cycle() {
    // A chain 1 -> 2 -> 3.
    let g = build(&[1, 2, 3], &[(1, Kind::Dep, 2), (2, Kind::Dep, 3)]);
    assert!(g.detect_cycle(ORDERING, &[]).is_none());
}

// ----- H-3: a tear breaks the cycle -----------------------------------------

#[test]
fn tear_breaks_cycle() {
    let g = build(&[1, 2], &[(1, Kind::Dep, 2), (2, Kind::Dep, 1)]);
    assert!(g.detect_cycle(ORDERING, &[]).is_some(), "cyclic without a tear");

    // Tear the 2 -> 1 edge: the ordering relation is now acyclic.
    let tear = Tear::new(2, 1, "assume the runtime config is provided out of band").unwrap();
    assert!(g.detect_cycle(ORDERING, &[tear]).is_none(), "the tear breaks the cycle");
}

// ----- H-4: a tear requires a rationale -------------------------------------

#[test]
fn tear_requires_rationale() {
    assert_eq!(Tear::new(1u32, 2u32, ""), Err(MissingRationale));
    assert_eq!(Tear::new(1u32, 2u32, "   "), Err(MissingRationale));
    assert!(Tear::new(1u32, 2u32, "deliberately assumed; tracked in ODD-0099").is_ok());
    // The error renders a helpful message.
    assert!(MissingRationale.to_string().contains("rationale"));
}

// ----- H-5: cycle-without-tear is a hard, typed error -----------------------

#[test]
fn cycle_without_tear_errors() {
    let g = build(&[1, 2], &[(1, Kind::Dep, 2), (2, Kind::Dep, 1)]);
    let cycle = g.detect_cycle(ORDERING, &[]).expect("a cycle");
    // `Cycle` is a typed std::error::Error the caller (check v2) can surface.
    let as_error: &dyn std::error::Error = &cycle;
    assert!(as_error.to_string().contains("ordering cycle"));
    assert!(as_error.to_string().contains('1') && as_error.to_string().contains('2'));
}

// ----- H-6: active tears are enumerable -------------------------------------

#[test]
fn list_active_tears() {
    let g = build(&[1, 2, 3], &[(1, Kind::Dep, 2), (2, Kind::Dep, 1)]);

    let real = Tear::new(2, 1, "assumed").unwrap(); // tears a real Dep edge
    let phantom = Tear::new(3, 1, "no such edge").unwrap(); // (3 -> 1) is not an edge

    let active = g.active_tears(ORDERING, std::slice::from_ref(&real));
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].rationale(), "assumed");

    // A tear that names a non-existent ordering edge is not "active".
    let mixed = [real.clone(), phantom];
    let active = g.active_tears(ORDERING, &mixed);
    assert_eq!(active.len(), 1);
    assert_eq!((active[0].from(), active[0].to()), (&2, &1));
}
