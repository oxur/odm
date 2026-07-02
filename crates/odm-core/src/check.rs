//! Structural validation of a node corpus — `check` v1.
//!
//! This is a **pure** function over already-parsed [`Frontmatter`]: it takes a
//! slice of nodes and returns the structural [`Finding`]s. It performs no I/O
//! and knows nothing about the CLI — the command layer loads the corpus, maps
//! findings to fix affordances, and chooses an exit code.
//!
//! v1 covers three structural families:
//!
//! 1. **Required-field completeness** — per-type required fields are present and
//!    non-empty.
//! 2. **Link-integrity** — every edge reference (`part_of`, `depends_on`,
//!    `blocked_by`, `verifies`, `consumes`, `affects`, `supersedes`, `tears`)
//!    resolves to a node in the corpus (no dangling refs).
//! 3. **Supersession-chain integrity** — no node supersedes itself, and the
//!    `supersedes` relation has no cycles.
//! 4. **Stale-doc-vs-decision** (arc05 slice05, ODD-0001 C5) — a doc governed by
//!    a committed decision (`A affects B`) whose decision was `updated` after it
//!    (`A.updated > B.updated`) is flagged as potentially stale. Structural +
//!    temporal only, never semantic (see [`Violation::StaleDoc`]).
//!
//! Graph-level checks (cycles-without-tears, out-of-order/staleness,
//! recomposition, below-threshold satisfaction) are deliberately **not** here;
//! they are `check` v2 (Arc 02), which adds validators alongside these without
//! rewriting them. New checks should be added as a `check_*` helper that pushes
//! onto the findings vector — see [`check`].

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;

use crate::Id;
use crate::frontmatter::Frontmatter;

/// A single structural problem found by [`check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The node the finding concerns.
    pub node: Id,
    /// The node's human number (for display).
    pub number: u32,
    /// The node's name (for display).
    pub name: String,
    /// What is wrong.
    pub violation: Violation,
}

/// The kind of structural violation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Violation {
    /// A required field is absent or empty.
    MissingField {
        /// The field name (e.g. `"name"`).
        field: &'static str,
    },
    /// `part_of` references a node id that is not in the corpus.
    DanglingPartOf {
        /// The unresolved target id.
        target: Id,
    },
    /// An edge other than `part_of` references a node id not in the corpus.
    DanglingEdge {
        /// The edge name (e.g. `"depends_on"`, `"supersedes"`).
        edge: &'static str,
        /// The unresolved target id.
        target: Id,
    },
    /// A node's `supersedes` edge points at itself.
    SelfSupersede,
    /// The `supersedes` relation forms a cycle through these nodes (in order).
    SupersessionCycle {
        /// The ids forming the cycle.
        cycle: Vec<Id>,
    },
    /// A doc may be **stale** relative to a committed decision that governs it:
    /// the decision `A` (which declares `A affects B`) was `updated` *after* the
    /// doc `B` it governs (`A.updated > B.updated`), so `B` may not reflect `A`
    /// (ODD-0001 C5). The subject [`Finding`] is `B` (the potentially-stale doc);
    /// this carries the governing decision `A` and both dates.
    ///
    /// **Structural + temporal, never semantic.** The `affects` edge *is* the
    /// author's assertion that `A` is a committed decision governing `B`; this
    /// check never inspects content or claims `B` *contradicts* `A` — it flags
    /// *potential* staleness for a human to judge (the same boundary
    /// recomposition-integrity draws: no automatic semantic detection).
    ///
    /// **Day granularity:** `updated` is a [`NaiveDate`], so the comparison is
    /// `>` (strictly later day), not `>=` — a decision and a doc edited on the
    /// **same day** are not distinguished and are **not** flagged (avoids nagging
    /// on a coordinated same-day edit; sub-day ordering is simply not tracked).
    StaleDoc {
        /// The governing decision node (`A`).
        decision: Id,
        /// `A`'s human number.
        decision_number: u32,
        /// `A`'s name.
        decision_name: String,
        /// When `A` (the decision) was last updated.
        decision_updated: NaiveDate,
        /// When `B` (this doc) was last updated.
        doc_updated: NaiveDate,
    },
}

/// Validates the structure of a node corpus, returning all findings.
///
/// Findings are returned in a deterministic order (by node id, then by a stable
/// per-node check order), so callers and snapshots see stable output.
#[must_use]
pub fn check(nodes: &[Frontmatter]) -> Vec<Finding> {
    let ids: BTreeSet<Id> = nodes.iter().map(Frontmatter::id).collect();
    let mut findings = Vec::new();

    // Process nodes in id order for deterministic output.
    let mut ordered: Vec<&Frontmatter> = nodes.iter().collect();
    ordered.sort_by_key(|fm| fm.id());

    for fm in &ordered {
        check_required_fields(fm, &mut findings);
        check_link_integrity(fm, &ids, &mut findings);
    }
    check_supersession(&ordered, &mut findings);
    check_stale_docs(&ordered, &mut findings);

    findings
}

/// Builds a finding for `fm` with the given violation.
fn finding(fm: &Frontmatter, violation: Violation) -> Finding {
    Finding { node: fm.id(), number: fm.number(), name: fm.name().to_string(), violation }
}

/// Required-field completeness. Per-type required fields live in
/// [`required_fields`]; v1 requires a non-empty `name` for every type. (This is
/// the extension point for v2 type-specific requirements.)
fn check_required_fields(fm: &Frontmatter, findings: &mut Vec<Finding>) {
    for &field in required_fields(fm) {
        let present = match field {
            "name" => !fm.name().trim().is_empty(),
            _ => true,
        };
        if !present {
            findings.push(finding(fm, Violation::MissingField { field }));
        }
    }
}

/// The required (must be present and non-empty) fields for a node's type.
///
/// v1 requires `name` for all types. Add type-specific entries here as the
/// model grows (this keeps the rule data-driven and v2-extensible).
fn required_fields(_fm: &Frontmatter) -> &'static [&'static str] {
    &["name"]
}

/// Link-integrity: every edge reference resolves to a node in the corpus.
fn check_link_integrity(fm: &Frontmatter, ids: &BTreeSet<Id>, findings: &mut Vec<Finding>) {
    let edges = fm.edges();

    if let Some(parent) = edges.part_of {
        if !ids.contains(&parent) {
            findings.push(finding(fm, Violation::DanglingPartOf { target: parent }));
        }
    }

    for target in edges.depends_on.iter().map(dependency_target) {
        dangling(fm, ids, "depends_on", target, findings);
    }
    for &target in &edges.blocked_by {
        dangling(fm, ids, "blocked_by", target, findings);
    }
    for &target in &edges.verifies {
        dangling(fm, ids, "verifies", target, findings);
    }
    for &target in &edges.consumes {
        dangling(fm, ids, "consumes", target, findings);
    }
    for &target in &edges.affects {
        dangling(fm, ids, "affects", target, findings);
    }
    if let Some(s) = &edges.supersedes {
        dangling(fm, ids, "supersedes", s.node, findings);
    }
    for target in edges.tears.iter().map(|t| dependency_target(&t.edge)) {
        dangling(fm, ids, "tears", target, findings);
    }
}

/// Pushes a [`Violation::DanglingEdge`] if `target` is not in `ids`.
fn dangling(
    fm: &Frontmatter,
    ids: &BTreeSet<Id>,
    edge: &'static str,
    target: Id,
    findings: &mut Vec<Finding>,
) {
    if !ids.contains(&target) {
        findings.push(finding(fm, Violation::DanglingEdge { edge, target }));
    }
}

/// The target id of a dependency edge (bare or qualified).
fn dependency_target(dep: &crate::frontmatter::Dependency) -> Id {
    match dep {
        crate::frontmatter::Dependency::Bare(id) => *id,
        crate::frontmatter::Dependency::Qualified { node, .. } => *node,
    }
}

/// Supersession-chain integrity: no self-supersede, no cycles.
fn check_supersession(ordered: &[&Frontmatter], findings: &mut Vec<Finding>) {
    // Map each node to the node it supersedes (its single lineage successor).
    let mut succ: BTreeMap<Id, Id> = BTreeMap::new();
    for fm in ordered {
        if let Some(s) = &fm.edges().supersedes {
            if s.node == fm.id() {
                findings.push(finding(fm, Violation::SelfSupersede));
            } else {
                succ.insert(fm.id(), s.node);
            }
        }
    }

    // Detect cycles in the supersedes relation. Walk from each node; if we
    // return to a node already on the current path, that path segment is a
    // cycle. Report each distinct cycle once (keyed by its smallest id).
    let by_id: BTreeMap<Id, &Frontmatter> = ordered.iter().map(|fm| (fm.id(), *fm)).collect();
    let mut reported: BTreeSet<Id> = BTreeSet::new();

    for &start in succ.keys() {
        let mut path: Vec<Id> = Vec::new();
        let mut seen: BTreeSet<Id> = BTreeSet::new();
        let mut cur = start;
        loop {
            if seen.contains(&cur) {
                // Found a cycle: the segment of `path` from `cur` onward.
                let at = path.iter().position(|&id| id == cur).unwrap_or(0);
                let cycle: Vec<Id> = path[at..].to_vec();
                let key = cycle.iter().copied().min().unwrap_or(cur);
                if reported.insert(key) {
                    // Attribute the finding to the smallest-id node in the cycle.
                    if let Some(fm) = by_id.get(&key) {
                        findings.push(finding(fm, Violation::SupersessionCycle { cycle }));
                    }
                }
                break;
            }
            seen.insert(cur);
            path.push(cur);
            match succ.get(&cur) {
                Some(&next) => cur = next,
                None => break, // chain terminates — good
            }
        }
    }
}

/// Stale-doc-vs-decision (ODD-0001 C5): for each `A affects B` where the
/// governing decision `A` was `updated` strictly after the doc `B`
/// (`A.updated > B.updated`), flag `B` as potentially stale relative to `A`.
///
/// Structural + temporal, **never semantic** — see [`Violation::StaleDoc`] for
/// the `affects`-is-the-commitment reasoning and the `NaiveDate` day-granularity
/// limitation (`>` not `>=`). A dangling `affects` target is not this pass's
/// concern (link-integrity already reports it); an unresolved target is skipped.
fn check_stale_docs(ordered: &[&Frontmatter], findings: &mut Vec<Finding>) {
    let by_id: BTreeMap<Id, &Frontmatter> = ordered.iter().map(|fm| (fm.id(), *fm)).collect();
    for &decision in ordered {
        for &doc_id in &decision.edges().affects {
            let Some(&doc) = by_id.get(&doc_id) else {
                continue; // dangling `affects` — reported by link-integrity
            };
            if decision.updated() > doc.updated() {
                findings.push(finding(
                    doc,
                    Violation::StaleDoc {
                        decision: decision.id(),
                        decision_number: decision.number(),
                        decision_name: decision.name().to_string(),
                        decision_updated: decision.updated(),
                        doc_updated: doc.updated(),
                    },
                ));
            }
        }
    }
}
