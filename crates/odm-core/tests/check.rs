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
    use odm_core::frontmatter::Dependency;
    let mut fm = node(id(A), 1, "Node");
    let edges = Edges {
        depends_on: vec![Dependency::Bare(id(MISSING))],
        blocked_by: vec![id(MISSING)],
        verifies: vec![id(MISSING)],
        consumes: vec![id(MISSING)],
        affects: vec![id(MISSING)],
        tears: vec![Dependency::Qualified { node: id(MISSING), satisfied_at: "tested".into() }],
        ..Edges::default()
    };
    *fm.edges_mut() = edges;
    let dangling = check(&[fm])
        .into_iter()
        .filter(|f| matches!(f.violation, Violation::DanglingEdge { .. }))
        .count();
    assert_eq!(dangling, 6, "all six edge kinds with a missing target are flagged");
}
