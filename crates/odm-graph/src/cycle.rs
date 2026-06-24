//! Cycle detection (Kahn) and explicit tears over the abstract graph.
//!
//! A cycle in the ordering relation is never silently tolerated: it is either
//! reported as a typed [`Cycle`] (a hard error the caller surfaces) or broken by
//! an explicit [`Tear`] — a `(from, to)` ordering edge deliberately assumed,
//! carrying a **required** rationale. Detection uses Kahn's algorithm over the
//! ordering edges minus the torn ones; active tears stay enumerable so assumed
//! dependencies remain visible.
//!
//! This stays domain-agnostic: the caller passes which edge kinds form the
//! ordering relation, and which `(from, to)` edges are torn.

use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::hash::Hash;

use petgraph::Direction;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use crate::Graph;

/// A deliberately-assumed ordering edge (`from -> to`) with a required
/// rationale. Tearing an edge removes it from the ordering relation, which can
/// break a cycle — but only as a recorded, visible decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tear<N> {
    from: N,
    to: N,
    rationale: String,
}

/// The error returned when constructing a [`Tear`] without a rationale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingRationale;

impl fmt::Display for MissingRationale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a tear requires a rationale for the assumed dependency")
    }
}

impl Error for MissingRationale {}

impl<N> Tear<N> {
    /// Creates a tear of the edge `from -> to` with a rationale.
    ///
    /// # Errors
    ///
    /// Returns [`MissingRationale`] if `rationale` is empty or whitespace — a
    /// tear is only valid as a recorded, justified decision.
    pub fn new(from: N, to: N, rationale: impl Into<String>) -> Result<Self, MissingRationale> {
        let rationale = rationale.into();
        if rationale.trim().is_empty() {
            return Err(MissingRationale);
        }
        Ok(Self { from, to, rationale })
    }

    /// The source endpoint of the torn edge.
    pub fn from(&self) -> &N {
        &self.from
    }

    /// The destination endpoint of the torn edge.
    pub fn to(&self) -> &N {
        &self.to
    }

    /// Why this dependency was deliberately assumed.
    #[must_use]
    pub fn rationale(&self) -> &str {
        &self.rationale
    }
}

/// A detected ordering cycle, naming the nodes that form it (in cycle order).
///
/// This is a typed error: an un-torn cycle is hard, and the caller (`check`)
/// consumes it as a failure rather than tolerating it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle<N> {
    members: Vec<N>,
}

impl<N> Cycle<N> {
    /// The nodes forming the cycle, in order (the last links back to the first).
    #[must_use]
    pub fn members(&self) -> &[N] {
        &self.members
    }
}

impl<N: fmt::Display> fmt::Display for Cycle<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ordering cycle: ")?;
        for (i, m) in self.members.iter().enumerate() {
            if i > 0 {
                f.write_str(" -> ")?;
            }
            write!(f, "{m}")?;
        }
        // Close the loop visually.
        if let Some(first) = self.members.first() {
            write!(f, " -> {first}")?;
        }
        Ok(())
    }
}

impl<N: fmt::Debug + fmt::Display> Error for Cycle<N> {}

impl<N, E> Graph<N, E>
where
    N: Clone + Eq + Hash,
    E: Clone + PartialEq,
{
    /// Detects a cycle in the ordering relation — the edges whose kind is in
    /// `ordering_kinds`, minus any edge named by a [`Tear`] — using Kahn's
    /// algorithm. Returns the cycle's members if one remains, or `None` if the
    /// ordering relation is acyclic.
    #[must_use]
    pub fn detect_cycle(&self, ordering_kinds: &[E], tears: &[Tear<N>]) -> Option<Cycle<N>> {
        let torn = self.torn_pairs(tears);
        let adjacency = self.ordering_adjacency(ordering_kinds, &torn);

        // Kahn: repeatedly remove zero-in-degree nodes; whatever cannot be
        // removed lies on (or behind) a cycle.
        let mut in_degree: HashMap<NodeIndex, usize> = HashMap::new();
        for targets in adjacency.values() {
            for &t in targets {
                *in_degree.entry(t).or_insert(0) += 1;
            }
        }
        let mut queue: VecDeque<NodeIndex> = self
            .inner
            .node_indices()
            .filter(|n| in_degree.get(n).copied().unwrap_or(0) == 0)
            .collect();
        let mut emitted: HashSet<NodeIndex> = HashSet::new();
        while let Some(node) = queue.pop_front() {
            if !emitted.insert(node) {
                continue;
            }
            for &next in &adjacency[&node] {
                if let Some(d) = in_degree.get_mut(&next) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(next);
                    }
                }
            }
        }

        if emitted.len() == self.inner.node_count() {
            return None;
        }

        // A cycle remains among the un-emitted nodes; name one precisely.
        let members = self
            .extract_cycle(&adjacency, &emitted)
            .into_iter()
            .map(|idx| self.inner[idx].clone())
            .collect();
        Some(Cycle { members })
    }

    /// The tears that name a real ordering edge in this graph (so a tear of a
    /// non-existent edge does not masquerade as active). Lets callers list the
    /// assumed dependencies that are actually in effect.
    #[must_use]
    pub fn active_tears<'a>(&self, ordering_kinds: &[E], tears: &'a [Tear<N>]) -> Vec<&'a Tear<N>> {
        tears.iter().filter(|t| self.has_ordering_edge(ordering_kinds, t.from(), t.to())).collect()
    }

    /// Maps each tear's endpoints to internal indices, keeping only those whose
    /// endpoints are both present.
    fn torn_pairs(&self, tears: &[Tear<N>]) -> HashSet<(NodeIndex, NodeIndex)> {
        tears
            .iter()
            .filter_map(|t| match (self.index.get(t.from()), self.index.get(t.to())) {
                (Some(&a), Some(&b)) => Some((a, b)),
                _ => None,
            })
            .collect()
    }

    /// Builds the effective ordering adjacency: outgoing edges of an ordering
    /// kind, excluding torn `(from, to)` pairs. Every node gets an entry.
    fn ordering_adjacency(
        &self,
        ordering_kinds: &[E],
        torn: &HashSet<(NodeIndex, NodeIndex)>,
    ) -> HashMap<NodeIndex, Vec<NodeIndex>> {
        let mut adjacency: HashMap<NodeIndex, Vec<NodeIndex>> =
            self.inner.node_indices().map(|n| (n, Vec::new())).collect();
        for edge in self.inner.edge_references() {
            let pair = (edge.source(), edge.target());
            if ordering_kinds.iter().any(|k| k == edge.weight()) && !torn.contains(&pair) {
                adjacency.entry(edge.source()).or_default().push(edge.target());
            }
        }
        adjacency
    }

    /// Whether there is an edge `from -> to` of an ordering kind.
    fn has_ordering_edge(&self, ordering_kinds: &[E], from: &N, to: &N) -> bool {
        let (Some(&a), Some(&b)) = (self.index.get(from), self.index.get(to)) else {
            return false;
        };
        self.inner
            .edges_directed(a, Direction::Outgoing)
            .any(|e| e.target() == b && ordering_kinds.iter().any(|k| k == e.weight()))
    }

    /// Extracts one concrete cycle from the un-emitted nodes via a DFS that
    /// stops at the first back edge (an edge into a node on the current path).
    fn extract_cycle(
        &self,
        adjacency: &HashMap<NodeIndex, Vec<NodeIndex>>,
        emitted: &HashSet<NodeIndex>,
    ) -> Vec<NodeIndex> {
        let mut on_path: HashSet<NodeIndex> = HashSet::new();
        let mut done: HashSet<NodeIndex> = HashSet::new();
        let mut path: Vec<NodeIndex> = Vec::new();
        for start in self.inner.node_indices() {
            if emitted.contains(&start) || done.contains(&start) {
                continue;
            }
            if let Some(cycle) =
                Self::dfs(start, adjacency, emitted, &mut on_path, &mut done, &mut path)
            {
                return cycle;
            }
        }
        Vec::new()
    }

    /// Depth-first walk returning the cycle path on the first back edge.
    fn dfs(
        node: NodeIndex,
        adjacency: &HashMap<NodeIndex, Vec<NodeIndex>>,
        emitted: &HashSet<NodeIndex>,
        on_path: &mut HashSet<NodeIndex>,
        done: &mut HashSet<NodeIndex>,
        path: &mut Vec<NodeIndex>,
    ) -> Option<Vec<NodeIndex>> {
        on_path.insert(node);
        path.push(node);
        for &next in &adjacency[&node] {
            if emitted.contains(&next) {
                continue;
            }
            if on_path.contains(&next) {
                let at = path.iter().position(|&p| p == next).unwrap_or(0);
                return Some(path[at..].to_vec());
            }
            if !done.contains(&next) {
                if let Some(cycle) = Self::dfs(next, adjacency, emitted, on_path, done, path) {
                    return Some(cycle);
                }
            }
        }
        on_path.remove(&node);
        done.insert(node);
        path.pop();
        None
    }
}
