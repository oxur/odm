//! Tests for the domain → graph translation layer (`odm_core::graph`).

use std::collections::BTreeSet;
use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::frontmatter::{Dependency, Edges, Frontmatter, SupersedeKind, Supersedes};
use odm_core::graph::{EdgeKind, NodeGraph};
use odm_core::{Id, NodeType, Origin};

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 24).unwrap()
}

fn node(id: Id, number: u32, name: &str, ty: NodeType) -> Frontmatter {
    Frontmatter::new(id, number, ty, name, day(), day(), Origin::Planned)
}

fn id(tag: char) -> Id {
    Id::from_str(&format!("01ARZ3NDEKTSV4RRFFQ69G5F{tag}0")).unwrap()
}

fn set(ids: Vec<Id>) -> BTreeSet<Id> {
    ids.into_iter().collect()
}

// ----- H-1: graph builds; one index per node --------------------------------

#[test]
fn graph_build_maps_every_node() {
    let nodes = vec![
        node(id('A'), 1, "Project", NodeType::Project),
        node(id('B'), 2, "Arc", NodeType::Arc),
        node(id('C'), 3, "Slice", NodeType::Slice),
    ];
    let g = NodeGraph::build(&nodes);
    assert_eq!(g.node_count(), 3);
    assert!(g.contains(id('A')) && g.contains(id('B')) && g.contains(id('C')));
    assert!(!g.contains(id('Z')));
}

#[test]
fn graph_build_skips_dangling_edges() {
    // A depends_on a node not in the set; build must not panic and the node
    // count stays 1 (no phantom node created for the missing target).
    let mut a = node(id('A'), 1, "A", NodeType::Slice);
    a.edges_mut().depends_on = vec![Dependency::Bare(id('Z'))];
    let g = NodeGraph::build(&[a]);
    assert_eq!(g.node_count(), 1);
    assert!(g.ordering_successors(id('A')).is_empty(), "dangling edge is not represented");
}

// ----- H-3: ordering DAG = depends_on ∪ consumes (only) ---------------------

#[test]
fn ordering_dag_membership() {
    // A has one edge of every kind to a distinct neighbor.
    let mut a = node(id('A'), 1, "A", NodeType::Slice);
    *a.edges_mut() = Edges {
        part_of: Some(id('P')),
        depends_on: vec![Dependency::Bare(id('D'))],
        consumes: vec![id('C')],
        blocked_by: vec![id('B')],
        verifies: vec![id('V')],
        affects: vec![id('F')],
        supersedes: Some(Supersedes { node: id('S'), kind: SupersedeKind::Updates }),
        tears: vec![Dependency::Bare(id('T'))],
    };
    // Provide all referenced nodes so edges are real.
    let mut nodes = vec![a];
    for t in ['P', 'D', 'C', 'B', 'V', 'F', 'S', 'T'] {
        nodes.push(node(id(t), 9, "n", NodeType::Slice));
    }
    let g = NodeGraph::build(&nodes);

    // Ordering DAG contains depends_on and consumes targets — and nothing else.
    assert_eq!(set(g.ordering_successors(id('A'))), set(vec![id('D'), id('C')]));

    // The other kinds are reachable only via their own accessors, never via the
    // ordering DAG.
    let ordering = set(g.ordering_successors(id('A')));
    for excluded in [id('P'), id('B'), id('V'), id('F'), id('S'), id('T')] {
        assert!(!ordering.contains(&excluded), "ordering DAG must exclude non-ordering edges");
    }
    // But they ARE present as their own edge kinds.
    assert_eq!(g.neighbors(id('A'), EdgeKind::BlockedBy), vec![id('B')]);
    assert_eq!(g.neighbors(id('A'), EdgeKind::Verifies), vec![id('V')]);
    assert_eq!(g.neighbors(id('A'), EdgeKind::Supersedes), vec![id('S')]);
}

// ----- H-4: part_of is a separate single-parent tree ------------------------

#[test]
fn part_of_tree() {
    // project P <- arc Q <- slices X, Y  (children point up via part_of)
    let mut q = node(id('Q'), 2, "Arc", NodeType::Arc);
    q.edges_mut().part_of = Some(id('P'));
    let mut x = node(id('X'), 3, "X", NodeType::Slice);
    x.edges_mut().part_of = Some(id('Q'));
    let mut y = node(id('Y'), 4, "Y", NodeType::Slice);
    y.edges_mut().part_of = Some(id('Q'));
    let p = node(id('P'), 1, "Project", NodeType::Project);
    let g = NodeGraph::build(&[p, q, x, y]);

    // Single parent up the tree.
    assert_eq!(g.parent(id('X')), Some(id('Q')));
    assert_eq!(g.parent(id('Q')), Some(id('P')));
    assert_eq!(g.parent(id('P')), None, "root has no parent");

    // Total recomposition: a parent's full child set via derived reverse part_of.
    assert_eq!(set(g.children(id('Q'))), set(vec![id('X'), id('Y')]));
    assert_eq!(set(g.children(id('P'))), set(vec![id('Q')]));
    assert!(g.children(id('X')).is_empty());

    // part_of is NOT in the ordering DAG.
    assert!(g.ordering_successors(id('X')).is_empty());
}

#[test]
fn backlinks_are_derived() {
    // D <-depends_on- A and B: D's ordering predecessors are A and B.
    let mut a = node(id('A'), 1, "A", NodeType::Slice);
    a.edges_mut().depends_on = vec![Dependency::Bare(id('D'))];
    let mut b = node(id('B'), 2, "B", NodeType::Slice);
    b.edges_mut().consumes = vec![id('D')];
    let d = node(id('D'), 3, "D", NodeType::Slice);
    let g = NodeGraph::build(&[a, b, d]);

    assert_eq!(set(g.ordering_predecessors(id('D'))), set(vec![id('A'), id('B')]));
    assert!(g.ordering_predecessors(id('A')).is_empty());
}
