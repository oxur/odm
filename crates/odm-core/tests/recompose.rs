//! Tests for decomposition/recomposition integrity (`odm_core::recompose`).

use std::collections::BTreeSet;
use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::frontmatter::Frontmatter;
use odm_core::gates::GateSets;
use odm_core::recompose::{Issue, Recomposition, integrity};
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 24).unwrap()
}

const CONFIG: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]

[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]

[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

fn gates() -> GateSets {
    GateSets::from_toml_str(CONFIG).unwrap()
}

fn id(tag: char) -> Id {
    Id::from_str(&format!("01ARZ3NDEKTSV4RRFFQ69G5F{tag}0")).unwrap()
}

fn node(tag: char, number: u32, ty: NodeType) -> Frontmatter {
    Frontmatter::new(id(tag), number, ty, tag.to_string(), day(), day(), Origin::Planned)
}

/// A node of `ty` whose `part_of` points at `parent`.
fn child(tag: char, number: u32, ty: NodeType, parent: char) -> Frontmatter {
    let mut fm = node(tag, number, ty);
    fm.edges_mut().part_of = Some(id(parent));
    fm
}

/// Records that `fm` reached `gate` (at `reproduced`) using `gates`.
fn reach(fm: &mut Frontmatter, gates: &GateSets, gate: &str) {
    let gset = gates.for_type(fm.node_type()).unwrap();
    fm.status_mut().set_gate(gset, gate, None, Evidence::Reproduced, day()).unwrap();
}

// ----- H-1: reverse-part_of enumerates a parent's complete child set --------

#[test]
fn recompose_children() {
    // project P <- arc Q <- slices X, Y.
    let p = node('P', 1, NodeType::Project);
    let q = child('Q', 2, NodeType::Arc, 'P');
    let x = child('X', 3, NodeType::Slice, 'Q');
    let y = child('Y', 4, NodeType::Slice, 'Q');

    let recomp = Recomposition::build(&[p, q, x, y]);

    assert_eq!(recomp.children(id('P')), &[id('Q')]);
    // The full child set, sorted by id (X before Y).
    assert_eq!(recomp.children(id('Q')), &[id('X'), id('Y')]);
    // A leaf has no children.
    assert!(recomp.children(id('X')).is_empty());
}

// ----- H-2: every non-root node resolves to exactly one parent (total) ------

#[test]
fn single_parent_total() {
    let p = node('P', 1, NodeType::Project);
    let q = child('Q', 2, NodeType::Arc, 'P');
    let x = child('X', 3, NodeType::Slice, 'Q');
    let y = child('Y', 4, NodeType::Slice, 'Q');
    let corpus = vec![p, q, x, y];
    let all: BTreeSet<Id> = corpus.iter().map(Frontmatter::id).collect();

    let recomp = Recomposition::build(&corpus);

    // Exactly one root: the project.
    assert_eq!(recomp.roots(), &[id('P')]);

    // Every non-root resolves to exactly one parent, and recomposition is total:
    // roots ∪ resolved-children == the whole corpus, partitioned (no overlap).
    let roots: BTreeSet<Id> = recomp.roots().iter().copied().collect();
    let resolved: BTreeSet<Id> =
        all.iter().copied().filter(|&n| recomp.parent(n).is_some()).collect();
    assert!(roots.is_disjoint(&resolved));
    assert_eq!(&roots | &resolved, all);

    // Each child appears under exactly one parent (single-parent is structural:
    // `part_of` is an Option, so two parents are unrepresentable).
    for &n in &resolved {
        let parent = recomp.parent(n).unwrap();
        assert_eq!(recomp.children(parent).iter().filter(|&&c| c == n).count(), 1);
    }
}

// ----- H-3: orphan detection ------------------------------------------------

#[test]
fn detect_orphan() {
    let gates = gates();

    // A slice with no part_of, and a slice whose part_of does not resolve, are
    // both orphans. A root project and a standalone note are not.
    let lonely_slice = node('X', 1, NodeType::Slice); // no part_of
    let dangling_slice = child('Y', 2, NodeType::Slice, 'Z'); // Z absent
    let project = node('P', 3, NodeType::Project); // root: not an orphan
    let note = node('N', 4, NodeType::Note); // standalone doc: not an orphan
    // arc-migration-fidelity s09 F-8: a legitimately top-level `artifact` is
    // not an orphan either (ODD-0025 §2.7, optional containment).
    let artifact = node('A', 5, NodeType::Artifact);

    let findings = integrity(&[lonely_slice, dangling_slice, project, note, artifact], &gates);
    let orphans: BTreeSet<Id> =
        findings.iter().filter(|f| f.issue == Issue::Orphan).map(|f| f.node).collect();

    assert_eq!(orphans, [id('X'), id('Y')].into_iter().collect());
}

// ----- H-4: undeveloped-stub detection --------------------------------------

#[test]
fn detect_undeveloped_stub() {
    let gates = gates();

    // An arc advanced to "in-progress" with zero children → stub.
    let mut working = node('Q', 1, NodeType::Arc);
    reach(&mut working, &gates, "in-progress");

    // An arc still only "planned" with zero children → NOT a stub (planning a
    // not-yet-decomposed arc is legitimate).
    let mut planned = node('R', 2, NodeType::Arc);
    reach(&mut planned, &gates, "planned");

    // An arc advanced with a child → NOT a stub.
    let mut developed = node('S', 3, NodeType::Arc);
    reach(&mut developed, &gates, "in-progress");
    let kid = child('X', 4, NodeType::Slice, 'S');

    let findings = integrity(&[working, planned, developed, kid], &gates);
    let stubs: Vec<&odm_core::recompose::Finding> =
        findings.iter().filter(|f| matches!(f.issue, Issue::UndevelopedStub { .. })).collect();

    assert_eq!(stubs.len(), 1);
    assert_eq!(stubs[0].node, id('Q'));
    assert_eq!(stubs[0].issue, Issue::UndevelopedStub { gate: "in-progress".to_string() });
}

#[test]
fn detect_undeveloped_stub_ignores_document_family_children() {
    let gates = gates();

    // An arc advanced to "in-progress" with only an `artifact` child (no
    // slice) is still a stub -- attaching a supporting doc is not
    // "developing" the work (arc-migration-fidelity s10).
    let mut working = node('Q', 1, NodeType::Arc);
    reach(&mut working, &gates, "in-progress");
    let doc = child('A', 2, NodeType::Artifact, 'Q');

    let findings = integrity(&[working, doc], &gates);
    assert!(
        findings.iter().any(|f| matches!(f.issue, Issue::UndevelopedStub { .. })),
        "a document-only child does not count as development: {findings:?}"
    );
}

// ----- H-5: the decomposed-complete assertion is recorded -------------------

#[test]
fn decomposed_assertion() {
    let mut arc = node('Q', 1, NodeType::Arc);
    assert!(arc.decomposed().is_none());

    // Affirm against two children (passed unsorted) — recorded sorted + dated.
    arc.affirm_decomposed(vec![id('Y'), id('X')], day());
    let decomp = arc.decomposed().expect("affirmed");
    assert_eq!(decomp.on, day());
    assert_eq!(decomp.children, vec![id('X'), id('Y')]);
}

// ----- H-6: drift guard -----------------------------------------------------

#[test]
fn decomposed_drift_guard() {
    let gates = gates();

    // Arc Q affirmed complete against child X only.
    let mut q = node('Q', 1, NodeType::Arc);
    q.affirm_decomposed(vec![id('X')], day());
    let x = child('X', 2, NodeType::Slice, 'Q');
    // Y was added under Q *after* the assertion (drift: added Y).
    let y = child('Y', 3, NodeType::Slice, 'Q');

    let findings = integrity(&[q.clone(), x, y], &gates);
    let drift = findings
        .iter()
        .find(|f| matches!(f.issue, Issue::DecompositionDrift { .. }))
        .expect("drift flagged");
    assert_eq!(drift.issue, Issue::DecompositionDrift { added: vec![id('Y')], removed: vec![] });

    // Now affirm against {X, Z} but only X is present → Z removed.
    let mut q2 = node('Q', 1, NodeType::Arc);
    q2.affirm_decomposed(vec![id('X'), id('Z')], day());
    let x2 = child('X', 2, NodeType::Slice, 'Q');
    let findings = integrity(&[q2, x2], &gates);
    let drift = findings
        .iter()
        .find(|f| matches!(f.issue, Issue::DecompositionDrift { .. }))
        .expect("drift flagged");
    assert_eq!(drift.issue, Issue::DecompositionDrift { added: vec![], removed: vec![id('Z')] });
}

// ----- s10: document-family children don't count as decomposition drift ----

#[test]
fn decomposed_drift_guard_ignores_document_family_children() {
    let gates = gates();

    // Arc Q affirmed complete against its one slice, X. An `artifact` and a
    // `note` are later attached (part_of Q, ODD-0025 §2.5's containment) --
    // real containment, but not *work* decomposition, so this must not read
    // as drift (arc-migration-fidelity s10, surfaced live: a mint that
    // attaches supporting docs to an already-`decomposed`-affirmed arc was
    // incorrectly flagged before this fix).
    let mut q = node('Q', 1, NodeType::Arc);
    q.affirm_decomposed(vec![id('X')], day());
    let x = child('X', 2, NodeType::Slice, 'Q');
    let artifact = child('A', 3, NodeType::Artifact, 'Q');
    let note = child('N', 4, NodeType::Note, 'Q');

    let findings = integrity(&[q, x, artifact, note], &gates);
    assert!(
        !findings.iter().any(|f| matches!(f.issue, Issue::DecompositionDrift { .. })),
        "document-family children are not work decomposition: {findings:?}"
    );

    // But a genuinely new *slice* still triggers drift alongside them.
    let mut q2 = node('Q', 1, NodeType::Arc);
    q2.affirm_decomposed(vec![id('X')], day());
    let x2 = child('X', 2, NodeType::Slice, 'Q');
    let y2 = child('Y', 5, NodeType::Slice, 'Q');
    let artifact2 = child('A', 3, NodeType::Artifact, 'Q');

    let findings = integrity(&[q2, x2, y2, artifact2], &gates);
    let drift = findings
        .iter()
        .find(|f| matches!(f.issue, Issue::DecompositionDrift { .. }))
        .expect("a genuinely new slice still flags drift");
    assert_eq!(drift.issue, Issue::DecompositionDrift { added: vec![id('Y')], removed: vec![] });
}

// ----- H-7: advance-toward-done without the assertion -----------------------

#[test]
fn advance_without_decomposition() {
    let gates = gates();

    // Arc Q reached terminal "verified" with a child but no `decomposed`.
    let mut q = node('Q', 1, NodeType::Arc);
    reach(&mut q, &gates, "verified");
    let x = child('X', 2, NodeType::Slice, 'Q');

    let findings = integrity(&[q, x], &gates);
    assert!(
        findings
            .iter()
            .any(|f| f.node == id('Q') && f.issue == Issue::AdvancedWithoutDecomposition),
        "done arc without `decomposed` is flagged"
    );

    // Once affirmed (matching the actual child), the flag clears.
    let mut q = node('Q', 1, NodeType::Arc);
    reach(&mut q, &gates, "verified");
    q.affirm_decomposed(vec![id('X')], day());
    let x = child('X', 2, NodeType::Slice, 'Q');

    let findings = integrity(&[q, x], &gates);
    assert!(
        !findings.iter().any(|f| f.issue == Issue::AdvancedWithoutDecomposition),
        "an affirmed, matching decomposition clears the flag"
    );
}

// ----- H-8: no semantic missing-scope guessing ------------------------------

#[test]
fn no_semantic_scope_guessing() {
    let gates = gates();

    // A well-formed tree: project P (root) <- arc Q <- slice X, where Q is
    // "done", has exactly one slice child, and has affirmed its decomposition
    // against that child. A *human* might suspect an arc with a single slice is
    // under-scoped — but the tool makes NO such claim: there is no "missing
    // scope" finding, so a structurally-sound parent is clean.
    let mut p = node('P', 1, NodeType::Project); // root, only planning
    p.affirm_decomposed(vec![id('Q')], day()); // G-3: affirmed, so it is silent
    let mut q = child('Q', 2, NodeType::Arc, 'P');
    reach(&mut q, &gates, "verified");
    q.affirm_decomposed(vec![id('X')], day());
    let x = child('X', 3, NodeType::Slice, 'Q');

    // Every *structural* obligation is discharged, so anything reported here
    // could only come from the tool second-guessing the plan's semantics.
    let findings = integrity(&[p, q, x], &gates);
    assert!(findings.is_empty(), "structurally sound, got: {findings:?}");
}

// ----- G-3: a parent with children must affirm its decomposition ------------

#[test]
fn undecomposed_parent_with_children_is_reported() {
    // A parent that has decomposed at all, but never affirmed the result.
    let p = node('P', 1, NodeType::Project);
    let q = child('Q', 2, NodeType::Arc, 'P');
    let findings = integrity(&[p, q], &gates());

    let issues: Vec<&Issue> = findings.iter().map(|f| &f.issue).collect();
    assert!(
        issues.iter().any(|i| matches!(i, Issue::UndecomposedParent { children: 1 })),
        "the project is reported with its child count: {issues:?}"
    );
}

#[test]
fn a_childless_parent_is_not_undecomposed() {
    // Nothing has been decomposed yet, so there is nothing to affirm. (A
    // childless parent that has *advanced* is an undeveloped stub — a different
    // finding, tested above.)
    let p = node('P', 1, NodeType::Project);
    let findings = integrity(&[p], &gates());
    assert!(
        !findings.iter().any(|f| matches!(f.issue, Issue::UndecomposedParent { .. })),
        "no children, nothing to affirm: {findings:?}"
    );
}

#[test]
fn an_affirmed_parent_is_not_reported() {
    let mut p = node('P', 1, NodeType::Project);
    let q = child('Q', 2, NodeType::Arc, 'P');
    p.affirm_decomposed(vec![id('Q')], day());
    let findings = integrity(&[p, q], &gates());
    assert!(
        !findings.iter().any(|f| matches!(f.issue, Issue::UndecomposedParent { .. })),
        "affirmed, so silent: {findings:?}"
    );
}

#[test]
fn a_done_parent_reports_the_stronger_finding_only() {
    // Both rules describe the same gap. A done parent is the more serious case,
    // and reporting both would count one gap twice.
    let mut p = node('P', 1, NodeType::Project);
    let q = child('Q', 2, NodeType::Arc, 'P');
    let g = gates();
    reach(&mut p, &g, "verified");
    let findings = integrity(&[p, q], &g);

    let n_advanced =
        findings.iter().filter(|f| matches!(f.issue, Issue::AdvancedWithoutDecomposition)).count();
    let n_undecomposed =
        findings.iter().filter(|f| matches!(f.issue, Issue::UndecomposedParent { .. })).count();
    assert_eq!(n_advanced, 1, "the terminal-gate finding fires: {findings:?}");
    assert_eq!(n_undecomposed, 0, "and not the broader one as well: {findings:?}");
}

#[test]
fn a_slice_is_never_an_undecomposed_parent() {
    // Slices admit no children, so the rule does not apply to them at all.
    let p = node('P', 1, NodeType::Project);
    let q = child('Q', 2, NodeType::Arc, 'P');
    let x = child('X', 3, NodeType::Slice, 'Q');
    let findings = integrity(&[p, q, x], &gates());
    assert!(
        !findings
            .iter()
            .any(|f| f.node == id('X') && matches!(f.issue, Issue::UndecomposedParent { .. })),
        "not parent-capable: {findings:?}"
    );
}

#[test]
fn a_mismatched_decomposition_still_drifts_not_merely_undecomposed() {
    // An affirmation that no longer matches its children is drift — a *wrong*
    // assertion, which is worse than an absent one and must not be masked by
    // the new rule.
    let mut p = node('P', 1, NodeType::Project);
    let q = child('Q', 2, NodeType::Arc, 'P');
    let r = child('R', 3, NodeType::Arc, 'P');
    p.affirm_decomposed(vec![id('Q')], day());
    let findings = integrity(&[p, q, r], &gates());

    assert!(
        findings.iter().any(|f| matches!(f.issue, Issue::DecompositionDrift { .. })),
        "the added child drifts the affirmation: {findings:?}"
    );
    assert!(
        !findings.iter().any(|f| matches!(f.issue, Issue::UndecomposedParent { .. })),
        "and it is not also reported as unaffirmed: {findings:?}"
    );
}
