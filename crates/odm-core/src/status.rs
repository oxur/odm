//! The multi-gate, evidence-tagged status vector (ODD-0013 §5.1, §2.3).
//!
//! Status is a **vector over named gates**, never a scalar (ODD-0001 D1): each
//! gate a node has reached carries its own record. A record tags *how well* the
//! gate is known with an [`Evidence`] level (§4.4 / 0001-D3), whose total order
//! `asserted < attested < reproduced < reconciled` slice 04 consumes when it
//! min-propagates confidence along dependency chains.
//!
//! This module is the **recording** half: it models the vector, validates a
//! gate against the type's [gate-set](crate::gates::GateSet), and records a
//! reach. It serializes to the §2.3 `status:` shape (a map of gate → record),
//! so it operates on the existing on-disk format rather than redefining it.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::gates::{GateSet, UnknownGate};

/// How well a reached gate is known. Ordered least- to most-confident; the
/// derived ordering is therefore the canonical total order
/// `Asserted < Attested < Reproduced < Reconciled`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Evidence {
    /// Claimed, no verification (the least-confident default).
    #[default]
    Asserted,
    /// Someone else's verification, relayed.
    Attested,
    /// Independently reproduced.
    Reproduced,
    /// Reconciled against observed reality.
    Reconciled,
}

impl Evidence {
    /// The canonical lowercase name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Evidence::Asserted => "asserted",
            Evidence::Attested => "attested",
            Evidence::Reproduced => "reproduced",
            Evidence::Reconciled => "reconciled",
        }
    }
}

/// The record for one reached gate: when, by whom (optional), at what evidence
/// level, and — optionally — the date each evidence level was *first* reached.
///
/// `reached`/`by`/`evidence` are the current reach (overwritten on a raise, as
/// before). `evidence_dates` is the durable verification-latency signal
/// (`workbench/forecasting-telemetry.md` §6): the first-reached date *per*
/// level, preserved across raises so an `attested → reproduced` transition's
/// timing is not lost. It is omitted on the wire when empty, so nodes written
/// before this field existed round-trip byte-identically.
///
/// Evidence **regression** (recording a lower level after a higher one) is out
/// of scope here — `evidence_dates` records first-reach per level only; it does
/// not model or flag a downgrade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateRecord {
    /// The date the gate was reached (the current reach; overwritten on a raise).
    pub reached: NaiveDate,
    /// Who recorded reaching it, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    /// How well the reach is known (defaults to `asserted` when absent).
    #[serde(default)]
    pub evidence: Evidence,
    /// The date each evidence level was *first* reached for this gate, in
    /// evidence order. Populated by [`Status::set_gate`]; omitted when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub evidence_dates: BTreeMap<Evidence, NaiveDate>,
}

/// A node's status: the gates it has reached, each with its record.
///
/// Serializes transparently as a map of gate name → [`GateRecord`], matching
/// the §2.3 `status:` block.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Status {
    gates: BTreeMap<String, GateRecord>,
}

impl Status {
    /// An empty status (no gates reached).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records that `gate` was reached, validating it against `gate_set` first.
    /// Re-recording a gate overwrites `reached`/`by`/`evidence` (e.g. raising the
    /// evidence level) but **preserves** the per-level first-reached dates: the
    /// reached level's date is recorded only if that level has not been seen
    /// before, so a raise keeps earlier levels' dates and re-recording the same
    /// level keeps its original date.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownGate`] if `gate` is not in `gate_set`.
    pub fn set_gate(
        &mut self,
        gate_set: &GateSet,
        gate: &str,
        by: Option<String>,
        evidence: Evidence,
        reached: NaiveDate,
    ) -> Result<(), UnknownGate> {
        if !gate_set.contains(gate) {
            return Err(UnknownGate {
                gate: gate.to_string(),
                allowed: gate_set.sequence().to_vec(),
            });
        }
        let record = self.gates.entry(gate.to_string()).or_insert_with(|| GateRecord {
            reached,
            by: by.clone(),
            evidence,
            evidence_dates: BTreeMap::new(),
        });
        // First-reach per level: never overwrite an earlier level's date.
        record.evidence_dates.entry(evidence).or_insert(reached);
        // The current reach overwrites, as before.
        record.reached = reached;
        record.by = by;
        record.evidence = evidence;
        Ok(())
    }

    /// The record for `gate`, if it has been reached.
    #[must_use]
    pub fn gate(&self, gate: &str) -> Option<&GateRecord> {
        self.gates.get(gate)
    }

    /// Whether `gate` has been reached.
    #[must_use]
    pub fn has_reached(&self, gate: &str) -> bool {
        self.gates.contains_key(gate)
    }

    /// The reached gates and their records, in gate-name order.
    pub fn reached(&self) -> impl Iterator<Item = (&str, &GateRecord)> {
        self.gates.iter().map(|(name, record)| (name.as_str(), record))
    }

    /// The number of reached gates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.gates.len()
    }

    /// Whether no gate has been reached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.gates.is_empty()
    }
}
