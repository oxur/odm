//! Derived-order queries over the abstract graph: topological order, the ready
//! frontier (`next`), blocked reasons, dependency paths, and evidence
//! min-propagation.
//!
//! Like the rest of `odm-graph`, this is domain-agnostic. The caller supplies,
//! per query, which edge kinds form the ordering relation and which form the
//! "blocked-by" relation, which nodes are complete, and — for each *satisfied*
//! ordering edge — the confidence level `L` at which it is satisfied (any
//! totally-ordered value). A configurable `threshold: L` distinguishes a fully
//! satisfied edge from a **soft-satisfied** one (satisfied, but below the
//! threshold): soft edges are *surfaced*, never used to withhold a node.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::Graph;
use crate::cycle::{Cycle, Tear};

/// The per-query facts the caller supplies for derived-order queries.
pub struct OrderInputs<'a, N, E, L> {
    /// Edge kinds that form the ordering relation (e.g. depends-on ∪ consumes).
    pub ordering_kinds: &'a [E],
    /// Edge kinds that withhold a node while the other endpoint is incomplete.
    pub block_kinds: &'a [E],
    /// Ordering edges deliberately assumed away (excluded from ordering).
    pub tears: &'a [Tear<N>],
    /// Nodes considered complete (excluded from the ready frontier).
    pub complete: &'a HashSet<N>,
    /// For each satisfied ordering edge `(from, to)`, the level it is satisfied
    /// at. An edge absent from this map is unsatisfied.
    pub satisfied: &'a HashMap<(N, N), L>,
    /// Levels strictly below this are soft-satisfied (surfaced, non-blocking).
    pub threshold: L,
}

/// A node on the ready frontier, with any soft-satisfied dependencies flagged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ready<N, L> {
    /// The ready node.
    pub node: N,
    /// Dependencies that are satisfied only below the threshold.
    pub soft: Vec<SoftDep<N, L>>,
}

/// A dependency satisfied below the threshold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftDep<N, L> {
    /// The dependency node.
    pub dep: N,
    /// The level it is satisfied at (below the threshold).
    pub evidence: L,
}

/// A reason a node is held back or carries low confidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block<N, L> {
    /// An ordering dependency that is not yet satisfied.
    Unsatisfied {
        /// The unsatisfied dependency.
        dep: N,
    },
    /// An ordering dependency satisfied only below the threshold.
    SoftSatisfied {
        /// The low-confidence dependency.
        dep: N,
        /// The level it is satisfied at.
        evidence: L,
        /// The threshold it must reach to be fully satisfied.
        threshold: L,
    },
    /// A "blocked-by" edge whose other endpoint is not complete.
    ExternallyBlocked {
        /// The blocking node.
        by: N,
    },
}

/// Collects tear endpoints as `(from, to)` pairs.
fn torn_pairs<N: Clone + Eq + Hash>(tears: &[Tear<N>]) -> HashSet<(N, N)> {
    tears.iter().map(|t| (t.from().clone(), t.to().clone())).collect()
}

impl<N, E> Graph<N, E>
where
    N: Clone + Eq + Hash,
    E: Clone + PartialEq,
{
    /// The distinct ordering dependencies of `node`: destinations of its
    /// outgoing ordering edges, excluding torn ones.
    fn ordering_deps(&self, node: &N, kinds: &[E], torn: &HashSet<(N, N)>) -> Vec<N> {
        let mut deps: Vec<N> = Vec::new();
        for kind in kinds {
            for nb in self.successors(node, kind) {
                let edge = (node.clone(), nb.clone());
                if !torn.contains(&edge) && !deps.contains(&nb) {
                    deps.push(nb);
                }
            }
        }
        deps
    }

    /// Whether `node` has a blocked-by edge to a node that is not complete.
    fn has_active_block<L>(&self, node: &N, inputs: &OrderInputs<N, E, L>) -> bool {
        inputs
            .block_kinds
            .iter()
            .any(|kind| self.successors(node, kind).iter().any(|by| !inputs.complete.contains(by)))
    }

    /// A topological order over the ordering relation (Kahn). The order has
    /// every dependency before the node that depends on it.
    ///
    /// # Errors
    ///
    /// Returns the [`Cycle`] if the ordering relation (minus tears) has one.
    pub fn topological_order(
        &self,
        ordering_kinds: &[E],
        tears: &[Tear<N>],
    ) -> Result<Vec<N>, Cycle<N>> {
        // A cycle has no valid order — name it via the dedicated detector.
        if let Some(cycle) = self.detect_cycle(ordering_kinds, tears) {
            return Err(cycle);
        }

        // Acyclic: peel nodes whose every dependency is already emitted, so the
        // result lists each dependency before the node that depends on it.
        let torn = torn_pairs(tears);
        let nodes = self.nodes_in_order();
        let deps: HashMap<N, Vec<N>> = nodes
            .iter()
            .map(|n| (n.clone(), self.ordering_deps(n, ordering_kinds, &torn)))
            .collect();

        let mut emitted: Vec<N> = Vec::new();
        let mut done: HashSet<N> = HashSet::new();
        while emitted.len() < nodes.len() {
            let mut progressed = false;
            for node in &nodes {
                if done.contains(node) {
                    continue;
                }
                if deps[node].iter().all(|d| done.contains(d) || !deps.contains_key(d)) {
                    emitted.push(node.clone());
                    done.insert(node.clone());
                    progressed = true;
                }
            }
            // Acyclic ⇒ at least one node is peelable each pass.
            debug_assert!(progressed, "acyclic graph must always make progress");
            if !progressed {
                break;
            }
        }
        Ok(emitted)
    }

    /// The ready frontier (`next`): nodes that are not complete, whose every
    /// ordering dependency is satisfied (soft counts — see below), and which
    /// have no active blocked-by edge. Soft-satisfied dependencies do **not**
    /// withhold the node; they are flagged on the returned [`Ready`].
    #[must_use]
    pub fn next<L: Ord + Clone>(&self, inputs: &OrderInputs<N, E, L>) -> Vec<Ready<N, L>> {
        let torn = torn_pairs(inputs.tears);
        let mut frontier = Vec::new();
        for node in self.nodes_in_order() {
            if inputs.complete.contains(&node) {
                continue;
            }
            let deps = self.ordering_deps(&node, inputs.ordering_kinds, &torn);
            let mut soft = Vec::new();
            let mut all_satisfied = true;
            for dep in &deps {
                match inputs.satisfied.get(&(node.clone(), dep.clone())) {
                    Some(level) => {
                        if *level < inputs.threshold {
                            soft.push(SoftDep { dep: dep.clone(), evidence: level.clone() });
                        }
                    }
                    None => {
                        all_satisfied = false;
                        break;
                    }
                }
            }
            if all_satisfied && !self.has_active_block(&node, inputs) {
                frontier.push(Ready { node, soft });
            }
        }
        frontier
    }

    /// Why `node` is held back or low-confidence: each unsatisfied dependency,
    /// each soft-satisfied dependency (with the threshold to reach), and each
    /// active blocked-by edge.
    #[must_use]
    pub fn blocked<L: Ord + Clone>(
        &self,
        node: &N,
        inputs: &OrderInputs<N, E, L>,
    ) -> Vec<Block<N, L>> {
        let torn = torn_pairs(inputs.tears);
        let mut reasons = Vec::new();
        for dep in self.ordering_deps(node, inputs.ordering_kinds, &torn) {
            match inputs.satisfied.get(&(node.clone(), dep.clone())) {
                None => reasons.push(Block::Unsatisfied { dep }),
                Some(level) if *level < inputs.threshold => reasons.push(Block::SoftSatisfied {
                    dep,
                    evidence: level.clone(),
                    threshold: inputs.threshold.clone(),
                }),
                Some(_) => {}
            }
        }
        for kind in inputs.block_kinds {
            for by in self.successors(node, kind) {
                if !inputs.complete.contains(&by) {
                    reasons.push(Block::ExternallyBlocked { by });
                }
            }
        }
        reasons
    }

    /// A node's effective evidence: the minimum level across its transitive
    /// satisfied ordering dependencies (a chain is only as verified as its
    /// weakest link). `None` if it has no satisfied dependency.
    #[must_use]
    pub fn min_evidence<L: Ord + Clone>(
        &self,
        node: &N,
        ordering_kinds: &[E],
        tears: &[Tear<N>],
        satisfied: &HashMap<(N, N), L>,
    ) -> Option<L> {
        let torn = torn_pairs(tears);
        let mut min: Option<L> = None;
        let mut visited: HashSet<N> = HashSet::new();
        let mut stack = vec![node.clone()];
        while let Some(cur) = stack.pop() {
            if !visited.insert(cur.clone()) {
                continue;
            }
            for dep in self.ordering_deps(&cur, ordering_kinds, &torn) {
                if let Some(level) = satisfied.get(&(cur.clone(), dep.clone())) {
                    min = Some(match min {
                        Some(m) => m.min(level.clone()),
                        None => level.clone(),
                    });
                }
                stack.push(dep);
            }
        }
        min
    }

    /// A dependency path. With `to = None`, the longest dependency chain
    /// starting at `from` (the critical path of what `from` waits on). With
    /// `to = Some(target)`, a path from `from` to `target` along ordering
    /// edges, or `None` if there is none.
    #[must_use]
    pub fn path(
        &self,
        from: &N,
        to: Option<&N>,
        ordering_kinds: &[E],
        tears: &[Tear<N>],
    ) -> Option<Vec<N>> {
        let torn = torn_pairs(tears);
        match to {
            Some(target) => {
                let mut path = vec![from.clone()];
                let mut seen = HashSet::new();
                if self.find_path(from, target, ordering_kinds, &torn, &mut path, &mut seen) {
                    Some(path)
                } else {
                    None
                }
            }
            None => Some(self.longest_chain(from, ordering_kinds, &torn, &mut HashSet::new())),
        }
    }

    /// DFS for a path from `cur` to `target`, building `path`.
    fn find_path(
        &self,
        cur: &N,
        target: &N,
        ordering_kinds: &[E],
        torn: &HashSet<(N, N)>,
        path: &mut Vec<N>,
        seen: &mut HashSet<N>,
    ) -> bool {
        if cur == target {
            return true;
        }
        if !seen.insert(cur.clone()) {
            return false;
        }
        for dep in self.ordering_deps(cur, ordering_kinds, torn) {
            path.push(dep.clone());
            if self.find_path(&dep, target, ordering_kinds, torn, path, seen) {
                return true;
            }
            path.pop();
        }
        false
    }

    /// The longest dependency chain starting at `from` (cycle-safe via `on_path`).
    fn longest_chain(
        &self,
        from: &N,
        ordering_kinds: &[E],
        torn: &HashSet<(N, N)>,
        on_path: &mut HashSet<N>,
    ) -> Vec<N> {
        if !on_path.insert(from.clone()) {
            return vec![from.clone()];
        }
        let mut best: Vec<N> = Vec::new();
        for dep in self.ordering_deps(from, ordering_kinds, torn) {
            let chain = self.longest_chain(&dep, ordering_kinds, torn, on_path);
            if chain.len() > best.len() {
                best = chain;
            }
        }
        on_path.remove(from);
        let mut result = vec![from.clone()];
        result.extend(best);
        result
    }

    /// Node ids in insertion (index) order, for deterministic output.
    fn nodes_in_order(&self) -> Vec<N> {
        self.inner.node_indices().map(|i| self.inner[i].clone()).collect()
    }
}
