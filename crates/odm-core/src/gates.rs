//! Per-node-type gate-sets, loaded from `odm.toml` (ODD-0013 §5.1).
//!
//! Each node type defines an ordered sequence of gates in
//! `[gates.<type>] sequence = [...]`. The last gate in a type's sequence is its
//! **terminal** gate — the default satisfaction target used by slice 04.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::NodeType;

/// The ordered gates for a single node type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateSet {
    sequence: Vec<String>,
}

impl GateSet {
    /// Creates a gate-set from an ordered sequence of gate names.
    #[must_use]
    pub fn new(sequence: Vec<String>) -> Self {
        Self { sequence }
    }

    /// The gates, in order.
    #[must_use]
    pub fn sequence(&self) -> &[String] {
        &self.sequence
    }

    /// Whether `gate` is part of this set.
    #[must_use]
    pub fn contains(&self, gate: &str) -> bool {
        self.sequence.iter().any(|g| g == gate)
    }

    /// The terminal (last) gate, or `None` if the sequence is empty.
    #[must_use]
    pub fn terminal(&self) -> Option<&str> {
        self.sequence.last().map(String::as_str)
    }
}

/// The gate-sets for every configured node type.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GateSets {
    by_type: BTreeMap<NodeType, GateSet>,
}

impl GateSets {
    /// Loads gate-sets from an `odm.toml` string. Only the `[gates.<type>]`
    /// tables are read; any other top-level keys are ignored.
    ///
    /// # Errors
    ///
    /// Returns [`GateConfigError::Toml`] if the string is not valid TOML or the
    /// `[gates]` shape is wrong, or [`GateConfigError::UnknownType`] if a key
    /// under `[gates]` is not a known node type.
    pub fn from_toml_str(toml_str: &str) -> Result<Self, GateConfigError> {
        let raw: RawConfig =
            toml::from_str(toml_str).map_err(|e| GateConfigError::Toml(e.to_string()))?;
        let mut by_type = BTreeMap::new();
        for (key, set) in raw.gates {
            let node_type =
                key.parse::<NodeType>().map_err(|_| GateConfigError::UnknownType(key.clone()))?;
            by_type.insert(node_type, GateSet::new(set.sequence));
        }
        Ok(Self { by_type })
    }

    /// The gate-set configured for `node_type`, if any.
    #[must_use]
    pub fn for_type(&self, node_type: NodeType) -> Option<&GateSet> {
        self.by_type.get(&node_type)
    }

    /// The terminal gate for `node_type`, if it is configured and non-empty.
    #[must_use]
    pub fn terminal(&self, node_type: NodeType) -> Option<&str> {
        self.for_type(node_type).and_then(GateSet::terminal)
    }

    /// The number of node types with a configured gate-set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_type.len()
    }

    /// Whether no gate-sets are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_type.is_empty()
    }
}

/// Deserialization shape for the `[gates]` table of `odm.toml`.
#[derive(Deserialize)]
struct RawConfig {
    #[serde(default)]
    gates: BTreeMap<String, RawGateSet>,
}

#[derive(Deserialize)]
struct RawGateSet {
    sequence: Vec<String>,
}

/// An error loading gate-sets from config.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GateConfigError {
    /// The TOML could not be parsed, or the `[gates]` shape was wrong.
    #[error("invalid gate config: {0}")]
    Toml(String),
    /// A key under `[gates]` is not a known node type.
    #[error("unknown node type in [gates]: {0:?}")]
    UnknownType(String),
}

/// The error returned when recording a gate that is not in the type's set.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown gate {gate:?}; expected one of {allowed:?}")]
pub struct UnknownGate {
    /// The rejected gate name.
    pub gate: String,
    /// The gates that would have been accepted.
    pub allowed: Vec<String>,
}
