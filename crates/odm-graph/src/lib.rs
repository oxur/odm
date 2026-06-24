//! A small directed-graph engine over **abstract** node ids and edge kinds.
//!
//! It carries zero domain knowledge: a node id is any `Clone + Eq + Hash` value
//! and an edge kind is any `Clone + Eq` value. The domain crate decides what
//! those mean and translates its model into this engine.
//!
//! Forward adjacency is stored once (on the source); **reverse adjacency is
//! derived**, never stored separately — the same directed edges read backward.
//! Accessors are filtered by edge kind, so callers can ask for, say, only the
//! `child -> parent` links or only the dependency links.
//!
//! This first piece is construction + adjacency only. Ordering, cycle handling,
//! and readiness build on top of it later.

#![deny(missing_docs)]

use std::collections::HashMap;
use std::hash::Hash;

use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

/// A directed graph of node ids (`N`) connected by typed edges (`E`).
///
/// Each distinct node id maps to exactly one internal index. Edges are directed
/// from a source to a destination and carry an edge kind; reverse lookups are
/// derived from the stored forward edges.
#[derive(Debug, Clone)]
pub struct Graph<N, E> {
    inner: DiGraph<N, E>,
    index: HashMap<N, NodeIndex>,
}

impl<N, E> Default for Graph<N, E>
where
    N: Clone + Eq + Hash,
    E: Clone + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E> Graph<N, E>
where
    N: Clone + Eq + Hash,
    E: Clone + PartialEq,
{
    /// Creates an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self { inner: DiGraph::new(), index: HashMap::new() }
    }

    /// Adds a node, returning `true` if it was newly inserted and `false` if it
    /// was already present. Idempotent: a given id always maps to one index.
    pub fn add_node(&mut self, node: N) -> bool {
        if self.index.contains_key(&node) {
            return false;
        }
        let idx = self.inner.add_node(node.clone());
        self.index.insert(node, idx);
        true
    }

    /// Adds a directed edge `from -(kind)-> to`. Returns `false` (adding
    /// nothing) if either endpoint is unknown — callers add nodes first.
    pub fn add_edge(&mut self, from: &N, kind: E, to: &N) -> bool {
        let (Some(&a), Some(&b)) = (self.index.get(from), self.index.get(to)) else {
            return false;
        };
        self.inner.add_edge(a, b, kind);
        true
    }

    /// Returns `true` if the node id is in the graph.
    #[must_use]
    pub fn contains(&self, node: &N) -> bool {
        self.index.contains_key(node)
    }

    /// The number of distinct nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// The number of edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    /// Every node id, in no particular order.
    pub fn nodes(&self) -> impl Iterator<Item = &N> {
        self.index.keys()
    }

    /// Forward neighbors of `node` reached by an edge of `kind` (the
    /// destinations of `node`'s outgoing `kind` edges).
    #[must_use]
    pub fn successors(&self, node: &N, kind: &E) -> Vec<N> {
        self.neighbors(node, kind, Direction::Outgoing)
    }

    /// Reverse neighbors of `node` for `kind` (the sources of incoming `kind`
    /// edges). Derived from the forward edges, not stored.
    #[must_use]
    pub fn predecessors(&self, node: &N, kind: &E) -> Vec<N> {
        self.neighbors(node, kind, Direction::Incoming)
    }

    /// All forward edges out of `node`, as `(kind, destination)` pairs.
    #[must_use]
    pub fn outgoing(&self, node: &N) -> Vec<(E, N)> {
        let Some(&idx) = self.index.get(node) else {
            return Vec::new();
        };
        self.inner
            .edges_directed(idx, Direction::Outgoing)
            .map(|e| (e.weight().clone(), self.inner[e.target()].clone()))
            .collect()
    }

    /// Neighbors of `node` in one `direction`, filtered to edges of `kind`. The
    /// returned id is always the *other* endpoint of the edge.
    fn neighbors(&self, node: &N, kind: &E, direction: Direction) -> Vec<N> {
        let Some(&idx) = self.index.get(node) else {
            return Vec::new();
        };
        self.inner
            .edges_directed(idx, direction)
            .filter(|e| e.weight() == kind)
            .map(|e| {
                let other = if e.source() == idx { e.target() } else { e.source() };
                self.inner[other].clone()
            })
            .collect()
    }
}
