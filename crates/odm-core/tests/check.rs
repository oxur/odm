//! Tests for the pure structural validator (`odm_core::check`).

use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::check::{Violation, check};
use odm_core::frontmatter::{Edges, Frontmatter, SupersedeKind, Supersedes};
use odm_core::{Id, NodeType, Origin};

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 24).unwrap()
}

fn node(id: Id, number: u32, name: &str) -> Frontmatter {
    Frontmatter::new(id, number, NodeType::Slice, name, day(), day(), Origin::Planned)
}

fn id(s: &str) -> Id {
    Id::from_str(s).unwrap()
}

const A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FA0";
const B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB0";
const C: &str = "01ARZ3NDEKTSV4RRFFQ69G5FC0";
const MISSING: &str = "01ARZ3NDEKTSV4RRFFQ69G5FZZ";

#[test]
fn clean_corpus_has_no_findings() {
    let parent = node(id(A), 1, "Parent");
    let mut child = node(id(B), 2, "Child");
    child.edges_mut().part_of = Some(id(A));
    assert!(check(&[parent, child]).is_empty());
}

#[test]
fn empty_name_is_a_missing_field() {
    let fm = node(id(A), 1, "   "); // whitespace-only
    let findings = check(&[fm]);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].violation, Violation::MissingField { field: "name" });
    assert_eq!(findings[0].node, id(A));
}

#[test]
fn dangling_part_of_is_flagged() {
    let mut fm = node(id(A), 1, "Orphan");
    fm.edges_mut().part_of = Some(id(MISSING));
    let findings = check(&[fm]);
    assert_eq!(
        findings,
        vec![odm_core::check::Finding {
            node: id(A),
            number: 1,
            name: "Orphan".to_string(),
            violation: Violation::DanglingPartOf { target: id(MISSING) },
        }]
    );
}

#[test]
fn dangling_edge_is_flagged() {
    let mut fm = node(id(A), 1, "Node");
    fm.edges_mut().supersedes =
        Some(Supersedes { node: id(MISSING), kind: SupersedeKind::Obsoletes });
    let findings = check(&[fm]);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].violation,
        Violation::DanglingEdge { edge: "supersedes", target: id(MISSING) }
    );
}

#[test]
fn self_supersede_is_flagged() {
    let mut fm = node(id(A), 1, "Node");
    fm.edges_mut().supersedes = Some(Supersedes { node: id(A), kind: SupersedeKind::Updates });
    let findings = check(&[fm]);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].violation, Violation::SelfSupersede);
}

#[test]
fn supersession_cycle_is_flagged_once() {
    // A supersedes B, B supersedes A — a 2-cycle.
    let mut a = node(id(A), 1, "A");
    a.edges_mut().supersedes = Some(Supersedes { node: id(B), kind: SupersedeKind::Obsoletes });
    let mut b = node(id(B), 2, "B");
    b.edges_mut().supersedes = Some(Supersedes { node: id(A), kind: SupersedeKind::Obsoletes });

    let cycles: Vec<_> = check(&[a, b])
        .into_iter()
        .filter(|f| matches!(f.violation, Violation::SupersessionCycle { .. }))
        .collect();
    assert_eq!(cycles.len(), 1, "a cycle is reported exactly once");
    if let Violation::SupersessionCycle { cycle } = &cycles[0].violation {
        assert_eq!(cycle.len(), 2);
        assert!(cycle.contains(&id(A)) && cycle.contains(&id(B)));
    }
}

#[test]
fn terminating_supersession_chain_is_clean() {
    // A -> B -> C, terminating. No cycle, all refs resolve.
    let mut a = node(id(A), 1, "A");
    a.edges_mut().supersedes = Some(Supersedes { node: id(B), kind: SupersedeKind::Updates });
    let mut b = node(id(B), 2, "B");
    b.edges_mut().supersedes = Some(Supersedes { node: id(C), kind: SupersedeKind::Updates });
    let c = node(id(C), 3, "C");
    assert!(check(&[a, b, c]).is_empty());
}

#[test]
fn multiple_findings_are_deterministic_by_id() {
    // Two orphans; findings come back in id order (A before B).
    let mut a = node(id(A), 1, "A");
    a.edges_mut().part_of = Some(id(MISSING));
    let mut b = node(id(B), 2, "B");
    b.edges_mut().part_of = Some(id(MISSING));
    let findings = check(&[b, a]); // pass out of order
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].node, id(A));
    assert_eq!(findings[1].node, id(B));
}

#[test]
fn all_edge_kinds_are_link_checked() {
    use odm_core::frontmatter::{Dependency, TornEdge};
    let mut fm = node(id(A), 1, "Node");
    let edges = Edges {
        depends_on: vec![Dependency::Bare(id(MISSING))],
        blocked_by: vec![id(MISSING)],
        verifies: vec![id(MISSING)],
        consumes: vec![id(MISSING)],
        affects: vec![id(MISSING)],
        tears: vec![TornEdge {
            edge: Dependency::Qualified { node: id(MISSING), satisfied_at: "tested".into() },
            because: "assumed".into(),
        }],
        ..Edges::default()
    };
    *fm.edges_mut() = edges;
    let dangling = check(&[fm])
        .into_iter()
        .filter(|f| matches!(f.violation, Violation::DanglingEdge { .. }))
        .count();
    assert_eq!(dangling, 6, "all six edge kinds with a missing target are flagged");
}

// ----- T-1/T-2/T-3 (arc05 slice05): stale-doc-vs-decision (C5) ---------------

fn dm(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// A node with `created` fixed early and an explicit `updated` date.
fn doc_updated(id_s: &str, number: u32, name: &str, updated: NaiveDate) -> Frontmatter {
    Frontmatter::new(
        id(id_s),
        number,
        NodeType::Note,
        name,
        dm(2026, 6, 1),
        updated,
        Origin::Planned,
    )
}

fn stale_findings(nodes: &[Frontmatter]) -> Vec<odm_core::check::Finding> {
    check(nodes).into_iter().filter(|f| matches!(f.violation, Violation::StaleDoc { .. })).collect()
}

#[test]
fn check_flags_stale_doc_after_decision() {
    // Decision A (updated 06-25) `affects` doc B (updated 06-22): A moved after B.
    let mut a = doc_updated(A, 1, "Decision", dm(2026, 6, 25));
    a.edges_mut().affects = vec![id(B)];
    let b = doc_updated(B, 2, "Doc", dm(2026, 6, 22));

    let stale = stale_findings(&[a, b]);
    assert_eq!(stale.len(), 1);
    let f = &stale[0];
    // Subject is B — the potentially-stale doc.
    assert_eq!(f.node, id(B));
    assert_eq!(f.name, "Doc");
    match &f.violation {
        Violation::StaleDoc {
            decision,
            decision_number,
            decision_name,
            decision_updated,
            doc_updated,
        } => {
            assert_eq!(*decision, id(A));
            assert_eq!(*decision_number, 1);
            assert_eq!(decision_name, "Decision");
            assert_eq!(*decision_updated, dm(2026, 6, 25));
            assert_eq!(*doc_updated, dm(2026, 6, 22));
        }
        other => panic!("expected StaleDoc, got {other:?}"),
    }
}

#[test]
fn check_fresh_doc_not_flagged() {
    // B (updated 06-25) is at-or-after its governing decision A (updated 06-20):
    // no false positive.
    let mut a = doc_updated(A, 1, "Decision", dm(2026, 6, 20));
    a.edges_mut().affects = vec![id(B)];
    let b = doc_updated(B, 2, "Doc", dm(2026, 6, 25));
    assert!(stale_findings(&[a, b]).is_empty());
}

#[test]
fn check_stale_doc_same_day_not_flagged() {
    // Decision and doc edited the same day → `>` (not `>=`) → not flagged
    // (day-granularity: sub-day ordering is not tracked).
    let mut a = doc_updated(A, 1, "Decision", dm(2026, 6, 24));
    a.edges_mut().affects = vec![id(B)];
    let b = doc_updated(B, 2, "Doc", dm(2026, 6, 24));
    assert!(stale_findings(&[a, b]).is_empty());

    // A dangling `affects` target is not this pass's concern (link-integrity's).
    let mut c = doc_updated(C, 3, "Decision2", dm(2026, 6, 25));
    c.edges_mut().affects = vec![id(MISSING)];
    assert!(stale_findings(&[c]).is_empty());
}

// ----- D-6 (arc05 slice06): dangling reenter_when -> a check finding ---------

#[test]
fn check_flags_dangling_reenter_when() {
    use odm_core::desired::{DesiredFact, ProbeSpec, ShellExpect};
    use odm_core::frontmatter::Deferral;

    let fact = DesiredFact {
        id: "present".to_string(),
        describe: "a declared fact".to_string(),
        probe: ProbeSpec::Shell {
            run: "true".to_string(),
            inputs: Vec::new(),
            expect: ShellExpect { exit: 0, stdout_contains: None },
        },
    };
    // A deferred node whose reenter_when references a fact it does NOT declare.
    let dangling = doc_updated(A, 1, "Parked", dm(2026, 6, 20))
        .with_desired_facts(vec![fact.clone()])
        .with_deferred(Some(Deferral {
            because: "waiting".to_string(),
            reenter_when: "no-such-fact".to_string(),
        }));
    let findings: Vec<_> = check(&[dangling])
        .into_iter()
        .filter(|f| matches!(f.violation, Violation::DanglingReenterWhen { .. }))
        .collect();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].node, id(A));
    match &findings[0].violation {
        Violation::DanglingReenterWhen { reenter_when } => assert_eq!(reenter_when, "no-such-fact"),
        other => panic!("expected DanglingReenterWhen, got {other:?}"),
    }

    // A deferred node whose reenter_when DOES resolve → no finding.
    let ok =
        doc_updated(B, 2, "Parked2", dm(2026, 6, 20)).with_desired_facts(vec![fact]).with_deferred(
            Some(Deferral { because: "waiting".to_string(), reenter_when: "present".to_string() }),
        );
    assert!(
        check(&[ok])
            .into_iter()
            .all(|f| !matches!(f.violation, Violation::DanglingReenterWhen { .. }))
    );
}

// ----- arc06 slice03 (ODD-0020): content-level validity (schema + per-type) ---

mod content {
    use super::*;
    use odm_core::check::content_validity;
    use odm_core::desired::{DesiredFact, ProbeSpec, ShellExpect};
    use odm_core::frontmatter::{Deferral, Supersedes as CoreSupersedes};
    use odm_core::schema::{SchemaMarker, SchemaVersion};

    fn odd(id_s: &str, number: u32, name: &str) -> Frontmatter {
        Frontmatter::new(id(id_s), number, NodeType::Odd, name, day(), day(), Origin::Planned)
    }

    fn slice(id_s: &str, number: u32, name: &str) -> Frontmatter {
        Frontmatter::new(id(id_s), number, NodeType::Slice, name, day(), day(), Origin::Planned)
    }

    fn a_fact() -> DesiredFact {
        DesiredFact {
            id: "up".to_string(),
            describe: "service up".to_string(),
            probe: ProbeSpec::Shell {
                run: "true".to_string(),
                inputs: Vec::new(),
                expect: ShellExpect { exit: 0, stdout_contains: None },
            },
        }
    }

    #[test]
    fn check_flags_wrong_type_field() {
        // `desired_facts` on an odd (work-only field on a document) → Error.
        let bad_odd = odd(A, 1, "Doc").with_desired_facts(vec![a_fact()]);
        let findings = content_validity(&[bad_odd]);
        assert!(
            findings.iter().any(|f| matches!(
                &f.violation,
                Violation::FieldNotValidForType {
                    field: "desired_facts",
                    node_type: NodeType::Odd
                }
            )),
            "desired_facts on odd flagged: {findings:?}"
        );

        // `supersedes` on a slice (document-only field on a work node) → Error.
        let mut bad_slice = slice(B, 2, "Work");
        bad_slice.edges_mut().supersedes =
            Some(CoreSupersedes { node: id(C), kind: SupersedeKind::Obsoletes });
        assert!(
            content_validity(&[bad_slice]).iter().any(|f| matches!(
                &f.violation,
                Violation::FieldNotValidForType { field: "supersedes", node_type: NodeType::Slice }
            )),
            "supersedes on slice flagged"
        );

        // A valid odd (supersedes is fine on a document) and a valid slice
        // (desired_facts/deferred are fine on work) → no field-validity findings.
        let mut good_odd = odd(A, 1, "Doc");
        good_odd.edges_mut().supersedes =
            Some(CoreSupersedes { node: id(B), kind: SupersedeKind::Obsoletes });
        let good_slice =
            slice(B, 2, "Work").with_desired_facts(vec![a_fact()]).with_deferred(Some(Deferral {
                because: "waiting".to_string(),
                reenter_when: "up".to_string(),
            }));
        assert!(
            content_validity(&[good_odd, good_slice])
                .iter()
                .all(|f| !matches!(f.violation, Violation::FieldNotValidForType { .. })),
            "valid nodes produce no field-validity finding"
        );
    }

    #[test]
    fn unknown_newer_schema_is_reported_error() {
        // A node stamped with a newer schema than this binary supports → reported.
        let newer = odd(A, 1, "Doc").with_schema(SchemaMarker {
            node_type: NodeType::Odd,
            version: SchemaVersion { major: 1, minor: 1 },
        });
        let findings = content_validity(&[newer]);
        assert!(
            findings
                .iter()
                .any(|f| matches!(&f.violation, Violation::UnsupportedSchema { schema } if schema == "odd/v1.1")),
            "newer schema reported: {findings:?}"
        );

        // The current schema (v1.0) and an unversioned (v0.1) node are fine.
        let current = odd(B, 2, "Now").with_schema(SchemaMarker::current(NodeType::Odd));
        let legacy = odd(C, 3, "Old"); // no schema ⇒ v0.1
        assert!(
            content_validity(&[current, legacy])
                .iter()
                .all(|f| !matches!(f.violation, Violation::UnsupportedSchema { .. })),
            "current + legacy schema are supported"
        );
    }
}
