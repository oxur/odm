//! The probe-runner (slice02): execute a node's (or the whole corpus's)
//! `desired_facts` and collect per-fact outcomes.
//!
//! The runner reads frontmatter **from the store, not the index** (see the crate
//! docs): `run_corpus` calls [`Store::load_all`](odm_store::Store::load_all)
//! afresh each invocation, so a newly written fact is seen with no manual index
//! rebuild (read-through freshness).

use std::path::Path;

use odm_core::frontmatter::Document;
use odm_core::{Id, ProbeSpec};
use odm_store::{Store, StoreError};
use serde::Serialize;

use crate::file::FileProbe;
use crate::probe::{Probe, ProbeOutcome};
use crate::shell::ShellProbe;

/// One declared fact's evaluation: the fact's node-local id and the outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FactResult {
    /// The fact's node-local id (from the `desired_facts` entry).
    pub fact_id: String,
    /// The three-way outcome of probing it.
    pub outcome: ProbeOutcome,
}

/// One node's results: its id plus a [`FactResult`] per declared fact. A node
/// with no `desired_facts` yields an **empty** `results` (a no-op, not an error).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeReport {
    /// The node whose facts were run.
    pub node_id: Id,
    /// One result per declared fact, in declaration order.
    pub results: Vec<FactResult>,
}

impl NodeReport {
    /// `true` if the node declared no facts (nothing was run).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Tally of this node's outcomes by kind.
    #[must_use]
    pub fn counts(&self) -> OutcomeCounts {
        OutcomeCounts::of(self.results.iter().map(|result| &result.outcome))
    }
}

/// The corpus-wide collection: a [`NodeReport`] for every node that declared at
/// least one fact. Factless nodes contribute nothing (they have no
/// `(fact_id, outcome)` tuples).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusReport {
    /// Per-node reports, in store (creation-id) order. Only nodes with facts.
    pub nodes: Vec<NodeReport>,
}

impl CorpusReport {
    /// `true` if no node in the corpus declared any facts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Every result across the corpus, flattened to `(node_id, fact_result)`.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &FactResult)> {
        self.nodes
            .iter()
            .flat_map(|node| node.results.iter().map(move |result| (node.node_id, result)))
    }

    /// Tally of every outcome across the corpus, by kind. Drift and error stay
    /// **distinct** — slice03 maps each to its own severity / exit code.
    #[must_use]
    pub fn counts(&self) -> OutcomeCounts {
        OutcomeCounts::of(self.iter().map(|(_, result)| &result.outcome))
    }
}

/// A tally of outcomes by kind, keeping drift and error distinct (never
/// flattened to a single count).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct OutcomeCounts {
    /// Facts whose probe held.
    pub holds: usize,
    /// Facts whose probe found drift (declared ≠ observed).
    pub drifted: usize,
    /// Facts whose probe could not be evaluated.
    pub errored: usize,
}

impl OutcomeCounts {
    /// Tallies the outcomes yielded by `outcomes`.
    fn of<'a>(outcomes: impl Iterator<Item = &'a ProbeOutcome>) -> Self {
        let mut counts = OutcomeCounts::default();
        for outcome in outcomes {
            match outcome {
                ProbeOutcome::Holds => counts.holds += 1,
                ProbeOutcome::Drifted { .. } => counts.drifted += 1,
                ProbeOutcome::Error { .. } => counts.errored += 1,
            }
        }
        counts
    }

    /// Total number of facts tallied.
    #[must_use]
    pub fn total(&self) -> usize {
        self.holds + self.drifted + self.errored
    }
}

/// Executes the `desired_facts` of a node or a whole corpus, reading current
/// frontmatter from the store.
#[derive(Debug, Clone)]
pub struct Runner<'a> {
    store: &'a Store,
}

impl<'a> Runner<'a> {
    /// Builds a runner over `store`. File-probe paths resolve relative to the
    /// store root (the repo / odm working directory).
    #[must_use]
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    /// Runs one node's declared facts, in order, collecting `(fact_id, outcome)`.
    /// A factless node yields an empty [`NodeReport`] (a no-op, never an error).
    #[must_use]
    pub fn run_node(&self, document: &Document) -> NodeReport {
        let root = self.store.root();
        let node_id = document.frontmatter().id();
        let results = document
            .frontmatter()
            .desired_facts()
            .iter()
            .map(|fact| FactResult {
                fact_id: fact.id.clone(),
                outcome: evaluate_spec(&fact.probe, root),
            })
            .collect();
        NodeReport { node_id, results }
    }

    /// Runs every node's declared facts across the corpus, reading **current**
    /// frontmatter from the store (read-through freshness — a newly written fact
    /// is seen on the next call with no manual index rebuild).
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] if the corpus cannot be loaded from the store.
    pub fn run_corpus(&self) -> Result<CorpusReport, StoreError> {
        let documents = self.store.load_all()?;
        let nodes = documents
            .iter()
            .map(|document| self.run_node(document))
            .filter(|report| !report.is_empty())
            .collect();
        Ok(CorpusReport { nodes })
    }
}

/// Dispatches a [`ProbeSpec`] to its probe and evaluates it. The exhaustive
/// match is the deliberate safety net: a new `ProbeSpec` kind is a compile error
/// here until it is handled.
fn evaluate_spec(spec: &ProbeSpec, root: &Path) -> ProbeOutcome {
    match spec {
        ProbeSpec::Shell { run, expect } => ShellProbe::new(run.clone(), expect.clone()).evaluate(),
        ProbeSpec::File { path, expect } => {
            FileProbe::new(root, path.clone(), expect.clone()).evaluate()
        }
    }
}
