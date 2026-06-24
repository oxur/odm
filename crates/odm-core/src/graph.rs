//! Translating a node set into the abstract [`odm_graph`] engine.
//!
//! This is the domain layer: it knows the odm edge kinds and which of them form
//! the **ordering DAG** (`depends_on ∪ consumes`) versus the **`part_of`
//! containment tree** (a separate, single-parent relation). The graph algebra
//! itself — forward/reverse adjacency, accessors by kind — lives in
//! `odm-graph`, which has no domain knowledge.
//!
//! Cycle detection, gates, and `next`/`blocked` are later slices; this builds
//! the graph and exposes the two views the rest of Arc 02 queries.

use odm_graph::Graph;

use crate::Id;
use crate::frontmatter::{Dependency, Frontmatter};

/// The kind of an edge between two nodes (ODD-0013 §3). This is the edge-weight
/// type the abstract engine is instantiated with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Containment: source is `part_of` the destination (the hierarchy tree).
    PartOf,
    /// The source needs the destination satisfied before it is ready.
    DependsOn,
    /// A hard external block on the source.
    BlockedBy,
    /// The source verifies the destination.
    Verifies,
    /// The source consumes a concrete output of the destination.
    Consumes,
    /// The source (a decision/doc) affects the destination.
    Affects,
    /// Lineage: the source supersedes the destination.
    Supersedes,
    /// A `depends_on` deliberately assumed to break a cycle.
    Tears,
}

/// The target id of a dependency edge (bare or gate-qualified).
fn dependency_target(dep: &Dependency) -> Id {
    match dep {
        Dependency::Bare(id) => *id,
        Dependency::Qualified { node, .. } => *node,
    }
}

/// The in-memory node graph: every node plus its typed edges, with forward and
/// derived-reverse lookups.
#[derive(Debug, Clone)]
pub struct NodeGraph {
    graph: Graph<Id, EdgeKind>,
}

impl NodeGraph {
    /// Builds the graph from a node set. Every node becomes exactly one vertex;
    /// each edge in a node's frontmatter becomes a typed edge. Edges whose
    /// target is not in the set are skipped (link-integrity flags those — see
    /// `odm_core::check`); they cannot be represented as a graph edge.
    #[must_use]
    pub fn build(nodes: &[Frontmatter]) -> Self {
        let mut graph = Graph::new();
        for fm in nodes {
            graph.add_node(fm.id());
        }
        for fm in nodes {
            let from = fm.id();
            let edges = fm.edges();
            if let Some(parent) = edges.part_of {
                graph.add_edge(&from, EdgeKind::PartOf, &parent);
            }
            for dep in &edges.depends_on {
                graph.add_edge(&from, EdgeKind::DependsOn, &dependency_target(dep));
            }
            for target in &edges.blocked_by {
                graph.add_edge(&from, EdgeKind::BlockedBy, target);
            }
            for target in &edges.verifies {
                graph.add_edge(&from, EdgeKind::Verifies, target);
            }
            for target in &edges.consumes {
                graph.add_edge(&from, EdgeKind::Consumes, target);
            }
            for target in &edges.affects {
                graph.add_edge(&from, EdgeKind::Affects, target);
            }
            if let Some(s) = &edges.supersedes {
                graph.add_edge(&from, EdgeKind::Supersedes, &s.node);
            }
            for dep in &edges.tears {
                graph.add_edge(&from, EdgeKind::Tears, &dependency_target(dep));
            }
        }
        Self { graph }
    }

    /// The number of nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Whether `node` is in the graph.
    #[must_use]
    pub fn contains(&self, node: Id) -> bool {
        self.graph.contains(&node)
    }

    /// The **ordering-DAG** successors of `node`: the nodes it must follow,
    /// i.e. the targets of its `depends_on` and `consumes` edges (and nothing
    /// else — `part_of`/`verifies`/`supersedes`/`affects`/`blocked_by`/`tears`
    /// are excluded).
    #[must_use]
    pub fn ordering_successors(&self, node: Id) -> Vec<Id> {
        let mut out = self.graph.successors(&node, &EdgeKind::DependsOn);
        out.extend(self.graph.successors(&node, &EdgeKind::Consumes));
        out
    }

    /// The reverse of [`ordering_successors`](Self::ordering_successors): nodes
    /// that depend on or consume `node`.
    #[must_use]
    pub fn ordering_predecessors(&self, node: Id) -> Vec<Id> {
        let mut out = self.graph.predecessors(&node, &EdgeKind::DependsOn);
        out.extend(self.graph.predecessors(&node, &EdgeKind::Consumes));
        out
    }

    /// The single containment parent of `node` in the `part_of` tree, if any.
    /// `part_of` is a separate single-parent relation, not part of the ordering
    /// DAG.
    #[must_use]
    pub fn parent(&self, node: Id) -> Option<Id> {
        self.graph.successors(&node, &EdgeKind::PartOf).into_iter().next()
    }

    /// The containment children of `node`: the nodes whose `part_of` points at
    /// it (derived reverse `part_of`). Powers total recomposition / `show`.
    #[must_use]
    pub fn children(&self, node: Id) -> Vec<Id> {
        self.graph.predecessors(&node, &EdgeKind::PartOf)
    }

    /// Forward neighbors of `node` along a specific [`EdgeKind`].
    #[must_use]
    pub fn neighbors(&self, node: Id, kind: EdgeKind) -> Vec<Id> {
        self.graph.successors(&node, &kind)
    }

    /// Reverse neighbors (backlinks) of `node` along a specific [`EdgeKind`].
    #[must_use]
    pub fn backlinks(&self, node: Id, kind: EdgeKind) -> Vec<Id> {
        self.graph.predecessors(&node, &kind)
    }
}
