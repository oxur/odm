//! Edge satisfaction and evidence-leveled confidence (ODD-0013 §4.4).
//!
//! An edge `A depends_on B` is **satisfied** when `B` has reached the gate the
//! edge names (`satisfied_at`, or `B`'s type's terminal gate by default). The
//! evidence level recorded at that gate is the *confidence* of the
//! satisfaction. A satisfaction below the configured **threshold** (default
//! `reproduced`) is **soft** — surfaced, never used to block.
//!
//! This module also assembles a [`Satisfaction`] over a whole node set, which
//! [`crate::graph::NodeGraph`]'s derived-order queries consume.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::Id;
use crate::frontmatter::{Dependency, Frontmatter};
use crate::gates::{GateConfigError, GateSet, GateSets};
use crate::graph::{EdgeKind, ORDERING_KINDS};
use crate::status::{Evidence, Status};
use odm_graph::{OrderInputs, Tear};

/// The default satisfaction threshold when `odm.toml` does not set one.
pub const DEFAULT_THRESHOLD: Evidence = Evidence::Reproduced;

/// The gate an edge is satisfied at: its explicit `satisfied_at`, else the
/// target type's terminal gate.
#[must_use]
pub fn satisfying_gate<'a>(
    satisfied_at: Option<&'a str>,
    target_gates: &'a GateSet,
) -> Option<&'a str> {
    satisfied_at.or_else(|| target_gates.terminal())
}

/// The evidence at which a dependency on a target is satisfied, or `None` if the
/// target has not reached the satisfying gate.
#[must_use]
pub fn edge_satisfaction(
    target_status: &Status,
    target_gates: &GateSet,
    satisfied_at: Option<&str>,
) -> Option<Evidence> {
    let gate = satisfying_gate(satisfied_at, target_gates)?;
    target_status.gate(gate).map(|record| record.evidence)
}

/// Whether a satisfaction at `level` is soft (strictly below `threshold`).
#[must_use]
pub fn is_soft(level: Evidence, threshold: Evidence) -> bool {
    level < threshold
}

/// Loads the satisfaction threshold from an `odm.toml` string
/// (`[satisfaction] threshold = "<level>"`), defaulting to
/// [`DEFAULT_THRESHOLD`] when absent.
///
/// # Errors
///
/// Returns [`GateConfigError::Toml`] if the string is not valid TOML or the
/// threshold is not a known evidence level.
pub fn threshold_from_toml(toml_str: &str) -> Result<Evidence, GateConfigError> {
    #[derive(Deserialize)]
    struct Raw {
        satisfaction: Option<Sat>,
    }
    #[derive(Deserialize)]
    struct Sat {
        threshold: Option<Evidence>,
    }
    let raw: Raw = toml::from_str(toml_str).map_err(|e| GateConfigError::Toml(e.to_string()))?;
    Ok(raw.satisfaction.and_then(|s| s.threshold).unwrap_or(DEFAULT_THRESHOLD))
}

/// A warning that a node is being advanced while a dependency is unsatisfied
/// (the staleness guard — build-staleness applied to the plan).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staleness {
    /// The node being advanced.
    pub node: Id,
    /// The unsatisfied ordering dependencies.
    pub unsatisfied: Vec<Id>,
}

/// Returns a [`Staleness`] warning if `node` has any unsatisfied ordering
/// dependency, or `None` if it is safe to advance. Non-fatal by design — the
/// caller decides whether to proceed (slice 06 can make it fatal in CI).
#[must_use]
pub fn staleness_on_advance(node: Id, unsatisfied: Vec<Id>) -> Option<Staleness> {
    if unsatisfied.is_empty() { None } else { Some(Staleness { node, unsatisfied }) }
}

/// The satisfaction facts for a whole node set: which nodes are complete, which
/// ordering edges are satisfied (and at what evidence), and the threshold.
#[derive(Debug, Clone)]
pub struct Satisfaction {
    complete: HashSet<Id>,
    satisfied: HashMap<(Id, Id), Evidence>,
    threshold: Evidence,
    tears: Vec<Tear<Id>>,
}

impl Satisfaction {
    /// Computes satisfaction over a node set, given the per-type gate-sets and
    /// the threshold. A node is **complete** when it has reached its type's
    /// terminal gate. An ordering edge is **satisfied** when its target has
    /// reached the satisfying gate (recording the evidence level).
    ///
    /// (Tears are not yet sourced from frontmatter — see the slice report — so
    /// the tear set is empty; the graph still honors tears when given any.)
    #[must_use]
    pub fn compute(frontmatters: &[Frontmatter], gates: &GateSets, threshold: Evidence) -> Self {
        let by_id: HashMap<Id, &Frontmatter> = frontmatters.iter().map(|f| (f.id(), f)).collect();
        let mut complete = HashSet::new();
        let mut satisfied = HashMap::new();

        for fm in frontmatters {
            if let Some(terminal) = gates.terminal(fm.node_type()) {
                if fm.status().has_reached(terminal) {
                    complete.insert(fm.id());
                }
            }
            let from = fm.id();
            let edges = fm.edges();
            for dep in &edges.depends_on {
                let (target, satisfied_at) = dependency_parts(dep);
                record(&by_id, gates, from, target, satisfied_at, &mut satisfied);
            }
            for &target in &edges.consumes {
                record(&by_id, gates, from, target, None, &mut satisfied);
            }
        }

        Self { complete, satisfied, threshold, tears: Vec::new() }
    }

    /// The order-query inputs view for the graph engine.
    pub(crate) fn inputs(&self) -> OrderInputs<'_, Id, EdgeKind, Evidence> {
        OrderInputs {
            ordering_kinds: &ORDERING_KINDS,
            block_kinds: &BLOCK_KINDS,
            tears: &self.tears,
            complete: &self.complete,
            satisfied: &self.satisfied,
            threshold: self.threshold,
        }
    }

    /// The torn ordering edges (currently always empty — see [`compute`]).
    #[must_use]
    pub(crate) fn tears(&self) -> &[Tear<Id>] {
        &self.tears
    }

    /// The satisfied-edge map (edge → evidence level).
    #[must_use]
    pub(crate) fn satisfied_map(&self) -> &HashMap<(Id, Id), Evidence> {
        &self.satisfied
    }
}

/// Blocked-by edge kinds (a single kind today).
const BLOCK_KINDS: [EdgeKind; 1] = [EdgeKind::BlockedBy];

/// The target id and `satisfied_at` gate of a dependency edge.
fn dependency_parts(dep: &Dependency) -> (Id, Option<&str>) {
    match dep {
        Dependency::Bare(id) => (*id, None),
        Dependency::Qualified { node, satisfied_at } => (*node, Some(satisfied_at.as_str())),
    }
}

/// Records the satisfaction of `from -> target` if the target exists, has a
/// gate-set, and has reached the satisfying gate.
fn record(
    by_id: &HashMap<Id, &Frontmatter>,
    gates: &GateSets,
    from: Id,
    target: Id,
    satisfied_at: Option<&str>,
    satisfied: &mut HashMap<(Id, Id), Evidence>,
) {
    if let Some(target_fm) = by_id.get(&target) {
        if let Some(gate_set) = gates.for_type(target_fm.node_type()) {
            if let Some(level) = edge_satisfaction(target_fm.status(), gate_set, satisfied_at) {
                satisfied.insert((from, target), level);
            }
        }
    }
}
