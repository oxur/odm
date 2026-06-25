//! The rollup model: the whole-plan view, assembled as a **pure function** of a
//! loaded corpus (ODD-0013 §6; arc03 D-2/D-3).
//!
//! [`Rollup::assemble`] is the single source the rendered `ROLLUP.md` (slice02),
//! `orient` (slice03), and `--json` (slice04) all consume — it is built once,
//! here, and packaged for rendering rather than re-derived per view. It performs
//! **no I/O and no caching** (D-2): it takes an already-loaded corpus and the
//! per-type gate-sets, and assembles every section by **reusing** the existing
//! odm-core ops — [`Recomposition`](crate::recompose) for the way-finding tree,
//! [`NodeGraph`](crate::graph) for ready/blocked and active tears,
//! [`Satisfaction`](crate::satisfaction) for edge satisfaction — never
//! reimplementing graph or recompose logic.
//!
//! Two sections are deliberately **slots, not yet wired** (arc03 open questions):
//!
//! - [`Drift`] — drift/`reconcile` is A5; the slot is present but carries no
//!   data and renders "not yet tracked (A5)" (Q-A3-2).
//! - [`Deferred`] — deferred-node surfacing + the re-entry predicate land with
//!   A5; the slot is defined but always **empty** here, and no `deferred` status
//!   variant is invented to populate it (Q-A3-1).

use std::collections::{HashMap, HashSet};

use crate::frontmatter::Frontmatter;
use crate::gates::GateSets;
use crate::graph::{Block, NodeGraph, frontmatter_tears};
use crate::recompose::Recomposition;
use crate::satisfaction::Satisfaction;
use crate::status::Evidence;
use crate::{Id, NodeType, Origin};

/// A node's identity and display fields, shared across every rollup section so
/// renderers and serializers need not re-resolve ids against the corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRef {
    /// Stable identity.
    pub id: Id,
    /// Human number.
    pub number: u32,
    /// Human label.
    pub name: String,
    /// Node type.
    pub node_type: NodeType,
}

/// One gate in a node's status vector, in the type's **gate-sequence order**
/// (arc03 D-4). A gate the node has not reached carries `evidence == None`
/// ("not reached"); the sequence position is preserved either way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateStatus {
    /// The gate name.
    pub gate: String,
    /// The evidence level the gate was reached at, or `None` if not reached.
    pub evidence: Option<Evidence>,
}

/// A node in the way-finding tree: its identity, origin, status vector, and its
/// containment children (the reverse-`part_of` subtree).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    /// The node.
    pub node: NodeRef,
    /// How the node arose.
    pub origin: Origin,
    /// The status vector in gate-sequence order (empty for types with no
    /// configured gate-set, e.g. documents).
    pub status: Vec<GateStatus>,
    /// Containment children, sorted by id (recomposition order).
    pub children: Vec<TreeNode>,
}

/// A dependency satisfied below the threshold (surfaced, never blocking).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftDep {
    /// The low-confidence dependency.
    pub dep: NodeRef,
    /// The level it is satisfied at (below the threshold).
    pub evidence: Evidence,
}

/// A node on the ready frontier (`next`), with any soft-satisfied dependencies
/// flagged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadyNode {
    /// The ready node.
    pub node: NodeRef,
    /// Dependencies satisfied only below the threshold.
    pub soft: Vec<SoftDep>,
}

/// Why a node is held back — each reason names the node it concerns, so a
/// renderer can list a blocked node's unsatisfied edges (arc03 R-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    /// An ordering dependency that is not yet satisfied.
    Unsatisfied {
        /// The unsatisfied dependency.
        dep: NodeRef,
    },
    /// An ordering dependency satisfied only below the threshold.
    SoftSatisfied {
        /// The low-confidence dependency.
        dep: NodeRef,
        /// The level it is satisfied at.
        evidence: Evidence,
        /// The threshold it must reach to be fully satisfied.
        threshold: Evidence,
    },
    /// A `blocked_by` edge whose target is not complete.
    ExternallyBlocked {
        /// The blocking node.
        by: NodeRef,
    },
}

/// A blocked node and the reasons it is held back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedNode {
    /// The blocked node.
    pub node: NodeRef,
    /// Each reason it is held back (unsatisfied / soft / externally blocked).
    pub reasons: Vec<BlockReason>,
}

/// An assumed dependency in effect (a "tear"), with its persisted rationale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveTear {
    /// The source node of the assumed edge.
    pub from: NodeRef,
    /// The target node of the assumed edge.
    pub to: NodeRef,
    /// Why the dependency was deliberately assumed.
    pub because: String,
}

/// The provenance (origin) view: every node grouped by how it arose — the
/// original-vs-emergent picture (ODD-0015 A3 / 0001-E2). Each group is sorted by
/// human number.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Provenance {
    /// Nodes that arose from deliberate up-front planning.
    pub planned: Vec<NodeRef>,
    /// Nodes that surfaced during the work itself.
    pub discovered: Vec<NodeRef>,
    /// Nodes that arose from an amendment to an existing plan.
    pub amendment: Vec<NodeRef>,
}

/// The drift slot. Drift detection is `reconcile` (A5); until then this carries
/// no data and a renderer shows "not yet tracked (A5)" (Q-A3-2). It is
/// `#[non_exhaustive]` so A5 can add fields without a breaking change — keeping
/// the rollup shape stable across that wiring.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Drift {}

/// The deferred slot. Deferred-node surfacing + the re-entry predicate land with
/// A5 (Q-A3-1); this is defined but always **empty** here, and no `deferred`
/// status variant is invented to fill it. A renderer emits no deferred section
/// while it is empty.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Deferred {
    /// Deferred nodes — always empty in A3; populated with A5.
    pub nodes: Vec<NodeRef>,
}

/// The assembled whole-plan view (ODD-0013 §6). A pure value: it owns its data
/// and borrows nothing, so it can be rendered to Markdown (slice02), composed by
/// `orient` (slice03), or serialized (slice04) without touching the corpus
/// again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rollup {
    /// The way-finding forest: the `part_of` roots, each with its subtree and
    /// status inline.
    pub tree: Vec<TreeNode>,
    /// The ready frontier ([`NodeGraph::next`]).
    pub ready: Vec<ReadyNode>,
    /// The blocked nodes and their reasons.
    pub blocked: Vec<BlockedNode>,
    /// The assumed dependencies in effect, each with its rationale.
    pub tears: Vec<ActiveTear>,
    /// The provenance/origin grouping over every node.
    pub provenance: Provenance,
    /// The drift slot (A5 — Q-A3-2).
    pub drift: Drift,
    /// The deferred slot (A5 — Q-A3-1); empty here.
    pub deferred: Deferred,
}

impl Rollup {
    /// Assembles the whole-plan view from a loaded corpus, the per-type
    /// gate-sets, and the satisfaction `threshold` (all already in memory — this
    /// performs no I/O and no caching, per D-2).
    ///
    /// Every section reuses an existing odm-core op: the tree from
    /// [`Recomposition`], ready/blocked from [`NodeGraph::next`]/
    /// [`NodeGraph::blocked`] over a [`Satisfaction`], active tears from
    /// [`NodeGraph::active_tears`] (sourced via
    /// [`frontmatter_tears`](crate::graph::frontmatter_tears)), and provenance
    /// by grouping on [`Frontmatter::origin`]. The [`Drift`] and [`Deferred`]
    /// slots are present but empty (A5).
    #[must_use]
    pub fn assemble(nodes: &[Frontmatter], gates: &GateSets, threshold: Evidence) -> Self {
        let by_id: HashMap<Id, &Frontmatter> = nodes.iter().map(|f| (f.id(), f)).collect();

        let recomp = Recomposition::build(nodes);
        let graph = NodeGraph::build(nodes);
        let satisfaction = Satisfaction::compute(nodes, gates, threshold);
        let tears = frontmatter_tears(nodes);

        let tree = recomp
            .roots()
            .iter()
            .filter_map(|&root| build_tree(root, &recomp, &by_id, gates))
            .collect();

        let ready: Vec<ReadyNode> = graph
            .next(&satisfaction)
            .into_iter()
            .filter_map(|r| {
                let node = node_ref(&by_id, r.node)?;
                let soft = r
                    .soft
                    .into_iter()
                    .filter_map(|s| {
                        Some(SoftDep { dep: node_ref(&by_id, s.dep)?, evidence: s.evidence })
                    })
                    .collect();
                Some(ReadyNode { node, soft })
            })
            .collect();
        // A soft-satisfied node is *ready* (soft deps never withhold it), so it
        // must not also appear as blocked — partition on the ready frontier.
        let ready_ids: HashSet<Id> = ready.iter().map(|r| r.node.id).collect();

        // Blocked: non-complete, non-ready nodes (in id order) with ≥1 reason.
        let mut ordered: Vec<&Frontmatter> = nodes.iter().collect();
        ordered.sort_by_key(|f| f.id());
        let mut blocked = Vec::new();
        for fm in ordered {
            if is_complete(fm, gates) || ready_ids.contains(&fm.id()) {
                continue;
            }
            let reasons: Vec<BlockReason> = graph
                .blocked(fm.id(), &satisfaction)
                .into_iter()
                .filter_map(|b| block_reason(&by_id, b))
                .collect();
            if reasons.is_empty() {
                continue;
            }
            if let Some(node) = node_ref(&by_id, fm.id()) {
                blocked.push(BlockedNode { node, reasons });
            }
        }

        let mut tears: Vec<ActiveTear> = graph
            .active_tears(&tears)
            .into_iter()
            .filter_map(|t| {
                Some(ActiveTear {
                    from: node_ref(&by_id, *t.from())?,
                    to: node_ref(&by_id, *t.to())?,
                    because: t.rationale().to_string(),
                })
            })
            .collect();
        tears.sort_by_key(|t| (t.from.id, t.to.id));

        let provenance = provenance(nodes);

        Self {
            tree,
            ready,
            blocked,
            tears,
            provenance,
            drift: Drift::default(),
            deferred: Deferred::default(),
        }
    }
}

/// Resolves an id to its [`NodeRef`], or `None` if it is not in the corpus.
///
/// Every id reached here comes from the graph or the recomposition forest, both
/// built from the same corpus, so a miss is unreachable in practice; `None`
/// keeps the assembly total and panic-free regardless.
fn node_ref(by_id: &HashMap<Id, &Frontmatter>, id: Id) -> Option<NodeRef> {
    by_id.get(&id).map(|fm| NodeRef {
        id: fm.id(),
        number: fm.number(),
        name: fm.name().to_string(),
        node_type: fm.node_type(),
    })
}

/// Builds a [`TreeNode`] for `id` and, recursively, its containment children
/// (sorted by id via [`Recomposition::children`]).
fn build_tree(
    id: Id,
    recomp: &Recomposition,
    by_id: &HashMap<Id, &Frontmatter>,
    gates: &GateSets,
) -> Option<TreeNode> {
    let fm = by_id.get(&id)?;
    let children = recomp
        .children(id)
        .iter()
        .filter_map(|&child| build_tree(child, recomp, by_id, gates))
        .collect();
    Some(TreeNode {
        node: node_ref(by_id, id)?,
        origin: fm.origin(),
        status: status_vector(fm, gates),
        children,
    })
}

/// A node's status vector in gate-sequence order: every gate in the type's
/// sequence, each tagged with the evidence it was reached at or `None` if not
/// reached (D-4). Empty for a type with no configured gate-set.
fn status_vector(fm: &Frontmatter, gates: &GateSets) -> Vec<GateStatus> {
    match gates.for_type(fm.node_type()) {
        Some(gset) => gset
            .sequence()
            .iter()
            .map(|gate| GateStatus {
                gate: gate.clone(),
                evidence: fm.status().gate(gate).map(|record| record.evidence),
            })
            .collect(),
        None => Vec::new(),
    }
}

/// Maps an engine [`Block`] reason to a [`BlockReason`] with resolved
/// [`NodeRef`]s, dropping any reason that names a node absent from the corpus.
fn block_reason(
    by_id: &HashMap<Id, &Frontmatter>,
    block: Block<Id, Evidence>,
) -> Option<BlockReason> {
    match block {
        Block::Unsatisfied { dep } => Some(BlockReason::Unsatisfied { dep: node_ref(by_id, dep)? }),
        Block::SoftSatisfied { dep, evidence, threshold } => {
            Some(BlockReason::SoftSatisfied { dep: node_ref(by_id, dep)?, evidence, threshold })
        }
        Block::ExternallyBlocked { by } => {
            Some(BlockReason::ExternallyBlocked { by: node_ref(by_id, by)? })
        }
    }
}

/// Whether `fm` has reached its type's terminal gate (mirrors the completeness
/// rule [`Satisfaction`] uses internally; a complete node is neither ready nor
/// blocked).
fn is_complete(fm: &Frontmatter, gates: &GateSets) -> bool {
    gates.terminal(fm.node_type()).is_some_and(|terminal| fm.status().has_reached(terminal))
}

/// Groups every node by [`Origin`], each group sorted by human number.
fn provenance(nodes: &[Frontmatter]) -> Provenance {
    let mut prov = Provenance::default();
    let mut ordered: Vec<&Frontmatter> = nodes.iter().collect();
    ordered.sort_by_key(|f| f.number());
    for fm in ordered {
        let node = NodeRef {
            id: fm.id(),
            number: fm.number(),
            name: fm.name().to_string(),
            node_type: fm.node_type(),
        };
        match fm.origin() {
            Origin::Planned => prov.planned.push(node),
            Origin::Discovered => prov.discovered.push(node),
            Origin::Amendment => prov.amendment.push(node),
        }
    }
    prov
}
