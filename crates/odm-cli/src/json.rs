//! Stable `--json` view structs for the `rollup` and `orient` envelopes
//! (arc03 slice04).
//!
//! These are `#[derive(Serialize)]` projections of the slice02
//! [`Rollup`](odm_core::rollup::Rollup) model — the **same** model the Markdown
//! renders consume (D-3), so the JSON cannot drift from the human output. They
//! follow the established `--json` pattern (the `check`/`list`/`show` view
//! structs in [`crate::commands`]): a thin serialization layer in the CLI, built
//! from the domain model, rendered with `serde_json::to_string_pretty`.
//!
//! Each top-level envelope carries an additive `schema` marker
//! (`"<command>/v1"`) so interop consumers (ODD-0017 export) can pin a version
//! and detect future evolution. The marker versions the contract **from its
//! introduction forward**; prior unmarked evolution of the `check` envelope is
//! pre-history.

use odm_core::rollup::{
    ActiveTear, BlockReason, BlockedNode, Drift, GateStatus, NodeRef, Provenance, ReadyNode,
    Rollup, TreeNode,
};
use serde::Serialize;

use crate::commands::IntegrityFinding;

/// The `rollup --json` schema marker.
pub(crate) const ROLLUP_SCHEMA: &str = "rollup/v1";
/// The `orient --json` schema marker.
pub(crate) const ORIENT_SCHEMA: &str = "orient/v1";

/// A node's identity + display fields, shared across every envelope.
#[derive(Serialize)]
pub(crate) struct NodeRefJson {
    id: String,
    number: u32,
    name: String,
    #[serde(rename = "type")]
    node_type: String,
}

impl From<&NodeRef> for NodeRefJson {
    fn from(n: &NodeRef) -> Self {
        Self {
            id: n.id.to_string(),
            number: n.number,
            name: n.name.clone(),
            node_type: n.node_type.as_str().to_string(),
        }
    }
}

/// One gate in a status vector: the gate name and the evidence it was reached
/// at, or `null` if not reached (gate-sequence order — D-4).
#[derive(Serialize)]
pub(crate) struct GateStatusJson {
    gate: String,
    evidence: Option<String>,
}

impl From<&GateStatus> for GateStatusJson {
    fn from(g: &GateStatus) -> Self {
        Self { gate: g.gate.clone(), evidence: g.evidence.map(|e| e.as_str().to_string()) }
    }
}

/// A dependency satisfied below the threshold (surfaced, non-blocking).
#[derive(Serialize)]
pub(crate) struct SoftDepJson {
    dep: NodeRefJson,
    evidence: String,
}

/// A ready node and its soft-satisfied dependencies.
#[derive(Serialize)]
pub(crate) struct ReadyJson {
    node: NodeRefJson,
    soft: Vec<SoftDepJson>,
}

impl From<&ReadyNode> for ReadyJson {
    fn from(r: &ReadyNode) -> Self {
        Self {
            node: (&r.node).into(),
            soft: r
                .soft
                .iter()
                .map(|s| SoftDepJson {
                    dep: (&s.dep).into(),
                    evidence: s.evidence.as_str().to_string(),
                })
                .collect(),
        }
    }
}

/// Why a node is held back (internally tagged by `kind`).
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum BlockReasonJson {
    /// An unsatisfied ordering dependency.
    Unsatisfied { dep: NodeRefJson },
    /// A dependency satisfied only below the threshold.
    SoftSatisfied { dep: NodeRefJson, evidence: String, threshold: String },
    /// A `blocked_by` edge whose target is not complete.
    ExternallyBlocked { by: NodeRefJson },
}

impl From<&BlockReason> for BlockReasonJson {
    fn from(r: &BlockReason) -> Self {
        match r {
            BlockReason::Unsatisfied { dep } => Self::Unsatisfied { dep: dep.into() },
            BlockReason::SoftSatisfied { dep, evidence, threshold } => Self::SoftSatisfied {
                dep: dep.into(),
                evidence: evidence.as_str().to_string(),
                threshold: threshold.as_str().to_string(),
            },
            BlockReason::ExternallyBlocked { by } => Self::ExternallyBlocked { by: by.into() },
        }
    }
}

/// A blocked node and the reasons it is held back.
#[derive(Serialize)]
pub(crate) struct BlockedJson {
    node: NodeRefJson,
    reasons: Vec<BlockReasonJson>,
}

impl From<&BlockedNode> for BlockedJson {
    fn from(b: &BlockedNode) -> Self {
        Self { node: (&b.node).into(), reasons: b.reasons.iter().map(Into::into).collect() }
    }
}

/// A node in the way-finding tree (recursive).
#[derive(Serialize)]
pub(crate) struct TreeNodeJson {
    id: String,
    number: u32,
    name: String,
    #[serde(rename = "type")]
    node_type: String,
    origin: String,
    status: Vec<GateStatusJson>,
    children: Vec<TreeNodeJson>,
}

impl From<&TreeNode> for TreeNodeJson {
    fn from(t: &TreeNode) -> Self {
        Self {
            id: t.node.id.to_string(),
            number: t.node.number,
            name: t.node.name.clone(),
            node_type: t.node.node_type.as_str().to_string(),
            origin: t.origin.as_str().to_string(),
            status: t.status.iter().map(Into::into).collect(),
            children: t.children.iter().map(Into::into).collect(),
        }
    }
}

/// An assumed dependency in effect, with its rationale.
#[derive(Serialize)]
pub(crate) struct ActiveTearJson {
    from: NodeRefJson,
    to: NodeRefJson,
    because: String,
}

impl From<&ActiveTear> for ActiveTearJson {
    fn from(t: &ActiveTear) -> Self {
        Self { from: (&t.from).into(), to: (&t.to).into(), because: t.because.clone() }
    }
}

/// The provenance (origin) grouping.
#[derive(Serialize)]
pub(crate) struct ProvenanceJson {
    planned: Vec<NodeRefJson>,
    discovered: Vec<NodeRefJson>,
    amendment: Vec<NodeRefJson>,
}

impl From<&Provenance> for ProvenanceJson {
    fn from(p: &Provenance) -> Self {
        Self {
            planned: p.planned.iter().map(Into::into).collect(),
            discovered: p.discovered.iter().map(Into::into).collect(),
            amendment: p.amendment.iter().map(Into::into).collect(),
        }
    }
}

/// The drift slot. Drift/`reconcile` is A5 (Q-A3-2); `tracked` is always `false`
/// until then. `#[non_exhaustive]` on the model lets A5 add fields.
#[derive(Serialize)]
pub(crate) struct DriftJson {
    tracked: bool,
}

impl From<&Drift> for DriftJson {
    fn from(_: &Drift) -> Self {
        Self { tracked: false }
    }
}

/// The full `rollup --json` envelope (mirrors the model, section for section).
#[derive(Serialize)]
pub(crate) struct RollupJson {
    schema: &'static str,
    tree: Vec<TreeNodeJson>,
    ready: Vec<ReadyJson>,
    blocked: Vec<BlockedJson>,
    tears: Vec<ActiveTearJson>,
    provenance: ProvenanceJson,
    drift: DriftJson,
    /// The deferred slot — always empty in A3 (Q-A3-1).
    deferred: Vec<NodeRefJson>,
}

impl From<&Rollup> for RollupJson {
    fn from(m: &Rollup) -> Self {
        Self {
            schema: ROLLUP_SCHEMA,
            tree: m.tree.iter().map(Into::into).collect(),
            ready: m.ready.iter().map(Into::into).collect(),
            blocked: m.blocked.iter().map(Into::into).collect(),
            tears: m.tears.iter().map(Into::into).collect(),
            provenance: (&m.provenance).into(),
            drift: (&m.drift).into(),
            deferred: m.deferred.nodes.iter().map(Into::into).collect(),
        }
    }
}

/// One surfaced integrity finding in the orient envelope.
#[derive(Serialize)]
pub(crate) struct IntegrityJson {
    severity: &'static str,
    code: String,
    who: String,
    detail: String,
}

/// The current-focus arc and its status vector.
#[derive(Serialize)]
pub(crate) struct FocusJson {
    pub(crate) arc: NodeRefJson,
    pub(crate) status: Vec<GateStatusJson>,
}

/// The full `orient --json` envelope. `project`/`vision`/`focus` are `null` in
/// the no-project fallback states; `hint` is non-null only in a fallback,
/// naming the exact fix command (never-bare-errors). The key set is fixed across
/// all states so the shape is lockable.
#[derive(Serialize)]
pub(crate) struct OrientJson {
    schema: &'static str,
    project: Option<NodeRefJson>,
    vision: Option<String>,
    focus: Option<FocusJson>,
    ready: Vec<ReadyJson>,
    blocked: Vec<BlockedJson>,
    integrity: Vec<IntegrityJson>,
    drift: DriftJson,
    hint: Option<String>,
}

impl OrientJson {
    /// The resolved-project orient view.
    pub(crate) fn resolved(
        project: NodeRefJson,
        vision: Option<String>,
        focus: Option<FocusJson>,
        model: &Rollup,
        findings: &[IntegrityFinding],
    ) -> Self {
        Self {
            schema: ORIENT_SCHEMA,
            project: Some(project),
            vision,
            focus,
            ready: model.ready.iter().map(Into::into).collect(),
            blocked: model.blocked.iter().map(Into::into).collect(),
            integrity: findings
                .iter()
                .filter(|f| f.is_error)
                .map(|f| IntegrityJson {
                    severity: "error",
                    code: f.code.to_string(),
                    who: f.who.clone(),
                    detail: f.detail.clone(),
                })
                .collect(),
            drift: (&model.drift).into(),
            hint: None,
        }
    }

    /// A no-project fallback envelope: empty data, a `hint` naming the fix.
    pub(crate) fn fallback(hint: String) -> Self {
        Self {
            schema: ORIENT_SCHEMA,
            project: None,
            vision: None,
            focus: None,
            ready: Vec::new(),
            blocked: Vec::new(),
            integrity: Vec::new(),
            drift: DriftJson { tracked: false },
            hint: Some(hint),
        }
    }
}
