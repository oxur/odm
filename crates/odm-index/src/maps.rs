//! In-memory filter/sort indexes built from the snapshot records (ODD-0014 §3.5)
//! — metadata filtering and sorting **without a database or an FTS engine**.
//!
//! [`IndexMaps::build`] turns a record set into a handful of `BTreeMap`s — by
//! type, by tag, by reached gate, plus forward edge adjacency — so a consumer
//! can answer "which nodes are slices / tagged `store` / have reached `built`?"
//! and walk the dependency edges with no disk access and no re-parse. Built once
//! on load; a few MB at 10k–100k nodes.

use std::collections::BTreeMap;

use odm_core::{Id, NodeType};

use crate::record::{EdgeRef, IndexRecord};

/// The in-memory filter/sort maps over a loaded record set.
#[derive(Debug, Clone, Default)]
pub struct IndexMaps {
    by_type: BTreeMap<NodeType, Vec<Id>>,
    by_tag: BTreeMap<String, Vec<Id>>,
    by_gate: BTreeMap<String, Vec<Id>>,
    edges: BTreeMap<Id, Vec<EdgeRef>>,
}

impl IndexMaps {
    /// Builds the maps from a record set. Ids appear in each bucket in the
    /// records' own order (id-sorted, as the snapshot is), so results are
    /// deterministic.
    #[must_use]
    pub fn build(records: &[IndexRecord]) -> Self {
        let mut maps = Self::default();
        for record in records {
            maps.by_type.entry(record.node_type).or_default().push(record.id);
            for tag in &record.tags {
                maps.by_tag.entry(tag.clone()).or_default().push(record.id);
            }
            for gate in &record.gates {
                maps.by_gate.entry(gate.gate.clone()).or_default().push(record.id);
            }
            maps.edges.insert(record.id, record.edges.clone());
        }
        maps
    }

    /// The ids of nodes of `node_type`.
    #[must_use]
    pub fn ids_by_type(&self, node_type: NodeType) -> &[Id] {
        self.by_type.get(&node_type).map_or(&[], Vec::as_slice)
    }

    /// The ids of nodes carrying `tag`.
    #[must_use]
    pub fn ids_by_tag(&self, tag: &str) -> &[Id] {
        self.by_tag.get(tag).map_or(&[], Vec::as_slice)
    }

    /// The ids of nodes that have reached `gate`.
    #[must_use]
    pub fn ids_by_gate(&self, gate: &str) -> &[Id] {
        self.by_gate.get(gate).map_or(&[], Vec::as_slice)
    }

    /// The outgoing edges of `node` (forward adjacency).
    #[must_use]
    pub fn edges_of(&self, node: Id) -> &[EdgeRef] {
        self.edges.get(&node).map_or(&[], Vec::as_slice)
    }
}
