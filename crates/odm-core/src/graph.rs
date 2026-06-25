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
use crate::satisfaction::Satisfaction;
use crate::status::Evidence;
// Re-export the engine's derived-order types so downstream crates use the
// domain interface (odm-core) without depending on odm-graph directly.
pub use odm_graph::{Block, Cycle, Ready, SoftDep, Tear};

/// The edge kinds that form the ordering DAG (`depends_on ∪ consumes`).
pub(crate) const ORDERING_KINDS: [EdgeKind; 2] = [EdgeKind::DependsOn, EdgeKind::Consumes];

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

/// The torn ordering edges declared in a corpus's frontmatter (`edges.tears`),
/// mapped to the engine's [`Tear`] carrying the persisted rationale (`because`).
///
/// A tear with an empty rationale is rejected by [`Tear::new`] and skipped — the
/// `tear` command never persists one, so that case only guards hand-edited
/// frontmatter. This is the single bridge from on-disk `TornEdge`s into the
/// derived-order queries; `check` and the rollup model both consume it rather
/// than re-deriving tears.
#[must_use]
pub fn frontmatter_tears(nodes: &[Frontmatter]) -> Vec<Tear<Id>> {
    let mut tears = Vec::new();
    for fm in nodes {
        let from = fm.id();
        for torn in &fm.edges().tears {
            let to = dependency_target(&torn.edge);
            if let Ok(t) = Tear::new(from, to, torn.because.clone()) {
                tears.push(t);
            }
        }
    }
    tears
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
            for torn in &edges.tears {
                graph.add_edge(&from, EdgeKind::Tears, &dependency_target(&torn.edge));
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

    // --- derived order (ODD-0013 §4.1/§4.4) ---------------------------------

    /// A topological order over the ordering DAG, or the cycle if one exists.
    ///
    /// # Errors
    ///
    /// Returns the [`Cycle`] if the ordering relation has one.
    pub fn topological_order(&self, tears: &[Tear<Id>]) -> Result<Vec<Id>, Cycle<Id>> {
        self.graph.topological_order(&ORDERING_KINDS, tears)
    }

    /// The tears that name a real ordering edge in this graph — the assumed
    /// dependencies actually in effect (a tear of a non-existent edge is inert,
    /// so it is excluded). `check` lists these with their rationale so assumed
    /// dependencies stay visible (ODD-0013 §4.3).
    #[must_use]
    pub fn active_tears<'a>(&self, tears: &'a [Tear<Id>]) -> Vec<&'a Tear<Id>> {
        self.graph.active_tears(&ORDERING_KINDS, tears)
    }

    /// The ready frontier (`next`): not-complete nodes whose ordering deps are
    /// satisfied (soft counts) and which have no active block. Soft deps are
    /// flagged on each [`Ready`], never used to withhold the node.
    #[must_use]
    pub fn next(&self, satisfaction: &Satisfaction) -> Vec<Ready<Id, Evidence>> {
        self.graph.next(&satisfaction.inputs())
    }

    /// Why `node` is blocked or low-confidence (unsatisfied deps, soft-satisfied
    /// deps with the threshold to reach, and active blocks).
    #[must_use]
    pub fn blocked(&self, node: Id, satisfaction: &Satisfaction) -> Vec<Block<Id, Evidence>> {
        self.graph.blocked(&node, &satisfaction.inputs())
    }

    /// A dependency path: the critical chain from `node` (`to = None`), or a
    /// path from `node` to `to`.
    #[must_use]
    pub fn path(&self, node: Id, to: Option<Id>, tears: &[Tear<Id>]) -> Option<Vec<Id>> {
        self.graph.path(&node, to.as_ref(), &ORDERING_KINDS, tears)
    }

    /// `node`'s effective evidence: the minimum across its transitive satisfied
    /// dependency path.
    #[must_use]
    pub fn min_evidence(&self, node: Id, satisfaction: &Satisfaction) -> Option<Evidence> {
        self.graph.min_evidence(
            &node,
            &ORDERING_KINDS,
            satisfaction.tears(),
            satisfaction.satisfied_map(),
        )
    }
}
