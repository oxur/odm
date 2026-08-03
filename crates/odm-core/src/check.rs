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
//! 5. **Dangling `reenter_when`** (arc05 slice06, Q-A3-1) — a `deferred` node
//!    whose `reenter_when` references a `desired_fact` it does not declare (see
//!    [`Violation::DanglingReenterWhen`]).
//!
//! [`content_validity`] adds **content-level** families that need the actual
//! frontmatter (which the index omits), so the CLI runs them over
//! store-loaded nodes, not the fast index-backed [`check`]: **per-type
//! field-validity** ([`Violation::FieldNotValidForType`]), the
//! **schema-version marker** ([`Violation::UnsupportedSchema`]), **source-path
//! portability** ([`Violation::AbsoluteSourcePath`]), and — ODD-0026 §2.1,
//! arc-store-as-source slice02 — **authored/source consistency**
//! ([`Violation::InconsistentAuthoredSource`]): `source` is not in the index
//! at all, so this check needs the same store-loaded pass `AbsoluteSourcePath`
//! does.
//!
//! Graph-level checks (cycles-without-tears, out-of-order/staleness,
//! recomposition, below-threshold satisfaction) are deliberately **not** here;
//! they are `check` v2 (Arc 02), which adds validators alongside these without
//! rewriting them. New checks should be added as a `check_*` helper that pushes
//! onto the findings vector — see [`check`].

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;

use crate::Id;
use crate::Origin;
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
    /// A node's `deferred.reenter_when` references a `desired_fact` id that the
    /// node does not declare (arc05 slice06, Q-A3-1). The node can never be shown
    /// ready to re-enter, so the reference should be fixed — but the node is still
    /// validly parked, so this is advisory (a Warning), not a structural error.
    DanglingReenterWhen {
        /// The unresolved `reenter_when` fact id.
        reenter_when: String,
    },
    /// A field is present that is **not valid for the node's type** (arc06 slice03,
    /// ODD-0020 §2): the per-type schema contract is violated — e.g. `desired_facts`
    /// on a `design`, or `supersedes` on a `slice`. A structural contract violation,
    /// so it is an Error (see [`content_validity`]).
    FieldNotValidForType {
        /// The offending field name (e.g. `"desired_facts"`).
        field: &'static str,
        /// The node's type, on which that field is not valid.
        node_type: crate::NodeType,
    },
    /// A node is stamped with a schema **newer** than this binary supports (arc06
    /// slice03, ODD-0020 §5): e.g. a `design/v1.1` node read by a `v1.0` binary. A
    /// reported condition — the reader cannot fully understand it — never a silent
    /// misparse.
    UnsupportedSchema {
        /// The offending schema marker (e.g. `"design/v1.1"`).
        schema: String,
    },
    /// A `source.paths` entry is not repo-content-root-relative — absolute, or
    /// anchored at a `.worktrees/` superproject root instead of the content
    /// root (arc-migration-fidelity s08 F-1's portability invariant).
    /// **Unconditional** — unlike coverage, there is no legitimate absolute
    /// `source.paths` case, so this needs no config gate (s10 iteration 1: the
    /// durable enforcement s08's data-only rewrite lacked, added after a
    /// regression reintroduced 4 absolute paths and slipped through `check`
    /// green because nothing enforced the invariant).
    AbsoluteSourcePath {
        /// The offending path, exactly as stored.
        path: std::path::PathBuf,
    },
    /// `origin: authored` and `source` disagree about whether this node is
    /// authored (ODD-0026 §2.1): `origin: authored` with no `source`, or a
    /// `source` that isn't the authored shape (`class` other than
    /// `"authored"`, or a non-empty `paths`/`migrated_by`/`migrated_on` — data
    /// an authored node, never migrated, cannot legitimately carry); or
    /// conversely `source.class == "authored"` on a node whose `origin` isn't
    /// `authored`. "Authored" is a provenance *value*, never the absence of
    /// one, and the two signals must always agree.
    InconsistentAuthoredSource {
        /// What specifically disagrees.
        reason: &'static str,
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
    check_reenter_when(&ordered, &mut findings);

    findings
}

/// Builds a finding for `fm` with the given violation.
fn finding(fm: &Frontmatter, violation: Violation) -> Finding {
    Finding { node: fm.id(), number: fm.number(), name: fm.name().to_string(), violation }
}

/// Content-level validity (arc06 slice03, ODD-0020): the checks that need the
/// node's **actual frontmatter content** — per-type field-validity and the
/// schema-version marker — which the derived index deliberately omits. The CLI
/// therefore runs this over **store-loaded** frontmatters (mirroring A5's
/// store-read for drift/deferred), separately from the index-backed [`check`].
///
/// Findings are returned id-ordered for stable output.
#[must_use]
pub fn content_validity(nodes: &[Frontmatter]) -> Vec<Finding> {
    let mut ordered: Vec<&Frontmatter> = nodes.iter().collect();
    ordered.sort_by_key(|fm| fm.id());

    let mut findings = Vec::new();
    for fm in &ordered {
        check_field_validity(fm, &mut findings);
        check_schema_version(fm, &mut findings);
        check_source_paths(fm, &mut findings);
        check_authored_source(fm, &mut findings);
    }
    findings
}

/// Per-type field-validity (ODD-0020 §2, carve-out per v1.4). The type-specific
/// fields fall into two buckets: **work-only** (`desired_facts`, `deferred` —
/// probeable/parked work) and **document-only** (`supersedes`, `affects` —
/// document lineage/governance). A field from the wrong bucket for the node's
/// type is a contract violation — **except** a work node carrying
/// `source.synthesis` (arc-migration-fidelity s13, ODD-0025 §2.3's project-
/// vision re-cast): structurally a document-lineage record wearing a work
/// node's type, so `supersedes`/`affects` are valid on it precisely because
/// it *is* a synthesis, not despite its `node_type`.
fn check_field_validity(fm: &Frontmatter, findings: &mut Vec<Finding>) {
    let ty = fm.node_type();
    if ty.is_document() {
        // Work-only fields are not valid on a document node.
        if !fm.desired_facts().is_empty() {
            findings.push(finding(
                fm,
                Violation::FieldNotValidForType { field: "desired_facts", node_type: ty },
            ));
        }
        if fm.deferred().is_some() {
            findings.push(finding(
                fm,
                Violation::FieldNotValidForType { field: "deferred", node_type: ty },
            ));
        }
    }
    if ty.is_work() {
        // Document-only fields are not valid on a work node — unless that
        // work node has been re-cast as a synthesis, in which case the
        // lineage edges are exactly what it's supposed to carry.
        let is_synthesis = fm.source().is_some_and(|s| s.synthesis.is_some());
        if !is_synthesis && !fm.edges().supersedes.is_empty() {
            findings.push(finding(
                fm,
                Violation::FieldNotValidForType { field: "supersedes", node_type: ty },
            ));
        }
        if !is_synthesis && !fm.edges().affects.is_empty() {
            findings.push(finding(
                fm,
                Violation::FieldNotValidForType { field: "affects", node_type: ty },
            ));
        }
        // `author`/`version` are document-node fields (ODD-0025 §2.2: "Both are
        // document-node fields", referring to the two just introduced) —
        // meaningless on a work node. `source`, by contrast, is explicitly
        // "every migrated node carries a source sub-map" (§2.2, no type
        // restriction) — a self-hosted arc/slice is a migrated *node* too, so
        // `source` stays valid on work nodes; only `author`/`version` are
        // flagged here.
        if fm.author().is_some() {
            findings.push(finding(
                fm,
                Violation::FieldNotValidForType { field: "author", node_type: ty },
            ));
        }
        if fm.version().is_some() {
            findings.push(finding(
                fm,
                Violation::FieldNotValidForType { field: "version", node_type: ty },
            ));
        }
    }
}

/// Portability of `source.paths` (arc-migration-fidelity s08 F-1, enforced as
/// an invariant since s10 iteration 1): every stored path must be
/// repo-content-root-relative. Unconditional — there is no legitimate
/// absolute `source.paths` case, so (unlike the `[coverage]` scan) this needs
/// no config gate.
fn check_source_paths(fm: &Frontmatter, findings: &mut Vec<Finding>) {
    let Some(source) = fm.source() else {
        return;
    };
    for path in &source.paths {
        if !is_portable_source_path(path) {
            findings.push(finding(fm, Violation::AbsoluteSourcePath { path: path.clone() }));
        }
    }
}

/// Whether a stored `source.paths` entry is portable: not absolute, and not
/// anchored at a `.worktrees/` superproject root (which would bake in the
/// worktree name — the same portability bug an absolute path is, one anchor
/// level up; arc-migration-fidelity s08 F-1).
fn is_portable_source_path(path: &std::path::Path) -> bool {
    if path.is_absolute() {
        return false;
    }
    !matches!(
        path.components().next(),
        Some(std::path::Component::Normal(first)) if first == ".worktrees"
    )
}

/// Schema-version validity (ODD-0020 §5): a node stamped with a schema newer than
/// this binary supports is a reported error, never a silent misparse.
fn check_schema_version(fm: &Frontmatter, findings: &mut Vec<Finding>) {
    if let Some(marker) = fm.schema() {
        if marker.version.is_newer_than_current() {
            findings.push(finding(fm, Violation::UnsupportedSchema { schema: marker.to_string() }));
        }
    }
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
    for s in &edges.supersedes {
        dangling(fm, ids, "supersedes", s.node, findings);
    }
    for target in edges.tears.iter().map(|t| dependency_target(&t.edge)) {
        dangling(fm, ids, "tears", target, findings);
    }
}

/// `origin: authored` and `source` must agree (ODD-0026 §2.1): "authored" is
/// a provenance *value*, never the absence of `source`, so an authored node
/// still carries `source: { class: "authored" }` — with no `paths` and no
/// `migrated_by`/`migrated_on` (there was no migration to record). Every
/// *other* check keeps firing on an authored node exactly as on a migrated
/// one; this is the only rule that treats the two differently, and it treats
/// them as a **consistency requirement** in both directions, not a
/// suppression: a mismatch either way is reported, never silently accepted.
fn check_authored_source(fm: &Frontmatter, findings: &mut Vec<Finding>) {
    if fm.origin() == Origin::Authored {
        let Some(source) = fm.source() else {
            findings.push(finding(
                fm,
                Violation::InconsistentAuthoredSource {
                    reason: "origin: authored requires a source record (class: \"authored\")",
                },
            ));
            return;
        };
        if !source.is_authored() {
            findings.push(finding(
                fm,
                Violation::InconsistentAuthoredSource {
                    reason: "origin: authored requires source.class == \"authored\"",
                },
            ));
        }
        if !source.paths.is_empty() {
            findings.push(finding(
                fm,
                Violation::InconsistentAuthoredSource {
                    reason: "an authored node carries no source.paths — it was never migrated",
                },
            ));
        }
        if source.migrated_by.is_some() || source.migrated_on.is_some() {
            findings.push(finding(
                fm,
                Violation::InconsistentAuthoredSource {
                    reason: "an authored node carries no migrated_by/migrated_on — there was no migration",
                },
            ));
        }
    } else if fm.source().is_some_and(crate::frontmatter::Source::is_authored) {
        findings.push(finding(
            fm,
            Violation::InconsistentAuthoredSource {
                reason: "source.class == \"authored\" requires origin: authored",
            },
        ));
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

/// Supersession-lineage integrity (ODD-0025 §2.3, s11 F-2): no self-supersede,
/// no cycles. A node may now supersede **many** targets (a synthesis), so the
/// relation is a general directed graph, not a single-successor chain — cycle
/// detection walks it with an explicit-stack DFS (not recursion, since the
/// relation depth is bounded only by corpus size, not by any model invariant).
fn check_supersession(ordered: &[&Frontmatter], findings: &mut Vec<Finding>) {
    // Map each node to every node it supersedes.
    let mut succ: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
    for fm in ordered {
        for s in &fm.edges().supersedes {
            if s.node == fm.id() {
                findings.push(finding(fm, Violation::SelfSupersede));
            } else {
                succ.entry(fm.id()).or_default().push(s.node);
            }
        }
    }

    let by_id: BTreeMap<Id, &Frontmatter> = ordered.iter().map(|fm| (fm.id(), *fm)).collect();
    let mut reported: BTreeSet<Id> = BTreeSet::new();
    // Nodes whose whole subtree has already been fully explored (from any
    // start) — never re-walked, so the overall cost stays linear in edges.
    let mut finished: BTreeSet<Id> = BTreeSet::new();

    for &start in succ.keys() {
        if finished.contains(&start) {
            continue;
        }
        // Each stack frame is (node, index of the next child to visit).
        // `on_path` is the current DFS path — a child already on it is a
        // back-edge, i.e. a cycle.
        let mut stack: Vec<(Id, usize)> = vec![(start, 0)];
        let mut on_path: Vec<Id> = vec![start];
        let mut on_path_set: BTreeSet<Id> = BTreeSet::from([start]);

        while let Some(&mut (node, ref mut next_child)) = stack.last_mut() {
            let children = succ.get(&node).map_or(&[][..], Vec::as_slice);
            if let Some(&child) = children.get(*next_child) {
                *next_child += 1;
                if on_path_set.contains(&child) {
                    let at = on_path.iter().position(|&id| id == child).unwrap_or(0);
                    let cycle: Vec<Id> = on_path[at..].to_vec();
                    let key = cycle.iter().copied().min().unwrap_or(child);
                    if reported.insert(key) {
                        // Attribute the finding to the smallest-id node in the cycle.
                        if let Some(fm) = by_id.get(&key) {
                            findings.push(finding(fm, Violation::SupersessionCycle { cycle }));
                        }
                    }
                } else if !finished.contains(&child) {
                    stack.push((child, 0));
                    on_path.push(child);
                    on_path_set.insert(child);
                }
            } else {
                stack.pop();
                on_path.pop();
                on_path_set.remove(&node);
                finished.insert(node);
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

/// Dangling-`reenter_when` (arc05 slice06, Q-A3-1): a node's `deferred` marker
/// references a `reenter_when` fact id the node does not declare. Advisory — the
/// node is still validly parked, but its re-entry predicate can never resolve.
fn check_reenter_when(ordered: &[&Frontmatter], findings: &mut Vec<Finding>) {
    for fm in ordered {
        let Some(deferral) = fm.deferred() else {
            continue;
        };
        let declared = fm.desired_facts().iter().any(|fact| fact.id == deferral.reenter_when);
        if !declared {
            findings.push(finding(
                fm,
                Violation::DanglingReenterWhen { reenter_when: deferral.reenter_when.clone() },
            ));
        }
    }
}
