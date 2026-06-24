//! Decomposition/recomposition integrity (ODD-0013 §4.5).
//!
//! Pure structural analysis over a parsed corpus. Two pieces:
//!
//! - [`Recomposition`] — the containment **forest** derived from reverse
//!   `part_of`: a parent's complete child set, the single resolved parent of
//!   each non-root node, and the roots. This is the "recompose the whole from
//!   the parts" view.
//! - [`integrity`] — the structural [`Finding`]s the forest and the recorded
//!   gates *prove*: orphans, undeveloped stubs, drift against a
//!   `decomposed: complete` assertion, and advancing a parent to done without
//!   that assertion.
//!
//! **Structural only.** Every finding is a fact the data proves. The tool
//! deliberately does **not** guess semantically missing or excess scope ("did
//! you forget a slice?") — that is a human judgement, and faking it would be
//! confabulation (§4.5). There is, by design, no "missing scope" variant in
//! [`Issue`]; what the engine offers instead is a *cheap, drift-guarded* review
//! of the decomposition a human affirms.
//!
//! Like [`crate::check`], this performs no I/O and knows nothing about the CLI:
//! it takes a slice of [`Frontmatter`] and returns findings in a deterministic
//! order. `check` v2 (slice06) aggregates these predicates alongside the v1
//! structural checks.

use std::collections::{BTreeMap, BTreeSet};

use crate::frontmatter::Frontmatter;
use crate::gates::{GateSet, GateSets};
use crate::{Id, NodeType};

/// The containment forest of a corpus, derived from reverse `part_of`.
///
/// `part_of` is a single-parent relation, so the reverse is a forest: each node
/// is either a **root** (no `part_of` declared) or resolves to exactly one
/// parent. A node whose `part_of` names an id absent from the corpus is neither
/// a root nor a resolved child — it is an *orphan* (and a dangling edge, which
/// `check` v1 also reports); see [`integrity`].
#[derive(Debug, Clone)]
pub struct Recomposition {
    /// parent → its children, each child list sorted by id.
    children: BTreeMap<Id, Vec<Id>>,
    /// child → its single resolved parent.
    parent: BTreeMap<Id, Id>,
    /// Nodes with no `part_of` declared (the tops of the forest), sorted by id.
    roots: Vec<Id>,
}

impl Recomposition {
    /// Builds the forest from a corpus. Only `part_of` edges whose target is in
    /// the corpus form parent/child links; an unresolved `part_of` leaves the
    /// node out of both the child map and the roots (it surfaces as an orphan).
    #[must_use]
    pub fn build(nodes: &[Frontmatter]) -> Self {
        let ids: BTreeSet<Id> = nodes.iter().map(Frontmatter::id).collect();
        let mut children: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
        let mut parent: BTreeMap<Id, Id> = BTreeMap::new();
        let mut roots: Vec<Id> = Vec::new();

        // Process in id order so child lists and roots are deterministic.
        let mut ordered: Vec<&Frontmatter> = nodes.iter().collect();
        ordered.sort_by_key(|fm| fm.id());

        for fm in &ordered {
            match fm.edges().part_of {
                None => roots.push(fm.id()),
                Some(p) if ids.contains(&p) => {
                    children.entry(p).or_default().push(fm.id());
                    parent.insert(fm.id(), p);
                }
                // Declared but unresolved: an orphan, not a root.
                Some(_) => {}
            }
        }

        Self { children, parent, roots }
    }

    /// A parent's complete child set (reverse `part_of`), sorted by id. Empty if
    /// `parent` has no children (or is not in the corpus).
    #[must_use]
    pub fn children(&self, parent: Id) -> &[Id] {
        self.children.get(&parent).map_or(&[], Vec::as_slice)
    }

    /// The single resolved containment parent of `child`, or `None` if `child`
    /// is a root or its declared parent does not resolve.
    #[must_use]
    pub fn parent(&self, child: Id) -> Option<Id> {
        self.parent.get(&child).copied()
    }

    /// The roots of the forest (nodes with no `part_of` declared), sorted by id.
    #[must_use]
    pub fn roots(&self) -> &[Id] {
        &self.roots
    }
}

/// A structural decomposition/recomposition problem found by [`integrity`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The node the finding concerns.
    pub node: Id,
    /// The node's human number (for display).
    pub number: u32,
    /// The node's name (for display).
    pub name: String,
    /// What is wrong.
    pub issue: Issue,
}

/// The kind of recomposition problem.
///
/// Every variant is a fact the containment tree or the recorded gates *prove*.
/// There is deliberately no variant claiming semantically missing or excess
/// scope — that is a human judgement the tool does not fake (§4.5).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Issue {
    /// A non-root work node (`arc`/`slice`) with no resolvable containment
    /// parent — recomposition is not total.
    Orphan,
    /// A parent-capable node (`project`/`arc`) advanced past its initial
    /// (planning) gate while it has zero children: an undeveloped stub.
    UndevelopedStub {
        /// The highest gate the stub has reached (the evidence it is "working").
        gate: String,
    },
    /// The node's current children differ from the set it affirmed via
    /// `decomposed: complete` — the assertion needs re-affirmation.
    DecompositionDrift {
        /// Children present now but not in the affirmed set (sorted by id).
        added: Vec<Id>,
        /// Children in the affirmed set but absent now (sorted by id).
        removed: Vec<Id>,
    },
    /// A parent-capable node reached its terminal ("done") gate without ever
    /// affirming `decomposed: complete`.
    AdvancedWithoutDecomposition,
}

/// Analyzes a corpus for decomposition/recomposition integrity, returning all
/// structural findings in a deterministic order (by node id, then a stable
/// per-node check order).
///
/// `gates` supplies the per-type gate sequences used to judge "advanced" and
/// "done". A node whose type has no configured gate-set is exempt from the
/// stub and advance-without-decomposition checks (advancement is unjudgeable),
/// but still participates in orphan and drift detection.
#[must_use]
pub fn integrity(nodes: &[Frontmatter], gates: &GateSets) -> Vec<Finding> {
    let recomp = Recomposition::build(nodes);

    let mut ordered: Vec<&Frontmatter> = nodes.iter().collect();
    ordered.sort_by_key(|fm| fm.id());

    let mut findings = Vec::new();
    for fm in &ordered {
        check_orphan(fm, &recomp, &mut findings);
        check_decomposition(fm, &recomp, gates, &mut findings);
    }
    findings
}

/// Builds a finding for `fm` with the given issue.
fn finding(fm: &Frontmatter, issue: Issue) -> Finding {
    Finding { node: fm.id(), number: fm.number(), name: fm.name().to_string(), issue }
}

/// Orphan: a non-root work node with no resolvable parent. A `project` is the
/// root of the work tree; document nodes (`odd`/`adr`/`note`) may stand alone,
/// so neither is ever an orphan.
fn check_orphan(fm: &Frontmatter, recomp: &Recomposition, findings: &mut Vec<Finding>) {
    let requires_parent = fm.node_type().is_work() && fm.node_type() != NodeType::Project;
    if requires_parent && recomp.parent(fm.id()).is_none() {
        findings.push(finding(fm, Issue::Orphan));
    }
}

/// The undeveloped-stub, advance-without-decomposition, and drift checks, all of
/// which apply only to **parent-capable** nodes (those whose type admits
/// children — `project`/`arc`).
fn check_decomposition(
    fm: &Frontmatter,
    recomp: &Recomposition,
    gates: &GateSets,
    findings: &mut Vec<Finding>,
) {
    if fm.node_type().valid_child_types().is_empty() {
        return; // not parent-capable (slice / document): nothing to decompose
    }

    let kids = recomp.children(fm.id());

    // Advancement is only judgeable with a configured gate-set.
    if let Some(gset) = gates.for_type(fm.node_type()) {
        // H-4: advanced past planning with zero children → undeveloped stub.
        if kids.is_empty() {
            if let Some(gate) = highest_reached_beyond_initial(fm, gset) {
                findings.push(finding(fm, Issue::UndevelopedStub { gate }));
            }
        }
        // H-7: reached the terminal gate ("done") without affirming the
        // decomposition.
        let done = gset.terminal().is_some_and(|t| fm.status().has_reached(t));
        if done && fm.decomposed().is_none() {
            findings.push(finding(fm, Issue::AdvancedWithoutDecomposition));
        }
    }

    // H-6: an affirmed decomposition that no longer matches the children.
    if let Some(decomp) = fm.decomposed() {
        let affirmed: BTreeSet<Id> = decomp.children.iter().copied().collect();
        let current: BTreeSet<Id> = kids.iter().copied().collect();
        let added: Vec<Id> = current.difference(&affirmed).copied().collect();
        let removed: Vec<Id> = affirmed.difference(&current).copied().collect();
        if !added.is_empty() || !removed.is_empty() {
            findings.push(finding(fm, Issue::DecompositionDrift { added, removed }));
        }
    }
}

/// The highest gate `fm` has reached that is past the initial (index-0,
/// planning) gate — i.e. evidence the node is "working" or beyond — or `None`
/// if it has only reached the initial gate (or none).
fn highest_reached_beyond_initial(fm: &Frontmatter, gset: &GateSet) -> Option<String> {
    gset.sequence()
        .iter()
        .enumerate()
        .rfind(|&(i, gate)| i > 0 && fm.status().has_reached(gate))
        .map(|(_, gate)| gate.clone())
}
