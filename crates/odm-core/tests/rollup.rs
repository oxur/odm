//! Tests for the rollup model (`odm_core::rollup`).
//!
//! Test names carry the substrings the slice02 ledger Verify commands filter on
//! (`rollup_model_assembles`, `rollup_tree_total`, `rollup_status_gate_order`).

use std::collections::BTreeSet;
use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::frontmatter::{Dependency, Frontmatter, TornEdge};
use odm_core::gates::GateSets;
use odm_core::rollup::{BlockReason, Rollup};
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 25).unwrap()
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

/// Records that `fm` reached `gate` at `evidence`.
fn reach(fm: &mut Frontmatter, gates: &GateSets, gate: &str, evidence: Evidence) {
    let gset = gates.for_type(fm.node_type()).unwrap();
    fm.status_mut().set_gate(gset, gate, None, evidence, day()).unwrap();
}

/// A small standing corpus: project P ← arc Q ← slices X, Y, with X depending
/// on Y. Reused across several assertions.
fn corpus() -> Vec<Frontmatter> {
    let p = node('P', 1, NodeType::Project);
    let q = child('Q', 2, NodeType::Arc, 'P');
    let mut x = child('X', 3, NodeType::Slice, 'Q');
    let y = child('Y', 4, NodeType::Slice, 'Q');
    // X depends_on Y (so X is blocked until Y is complete; Y is ready).
    x.edges_mut().depends_on.push(Dependency::Bare(id('Y')));
    vec![p, q, x, y]
}

// ----- R-1: the model assembles every section as a pure function ------------

#[test]
fn rollup_model_assembles_every_section() {
    let gates = gates();
    let model = Rollup::assemble(&corpus(), &gates, Evidence::Reproduced);

    // Tree: one root (the project).
    assert_eq!(model.tree.len(), 1, "exactly one forest root");
    assert_eq!(model.tree[0].node.id, id('P'));

    // Ready/blocked: Y (no deps) is ready; X (depends_on unsatisfied Y) is blocked.
    let ready_ids: BTreeSet<Id> = model.ready.iter().map(|r| r.node.id).collect();
    assert!(ready_ids.contains(&id('Y')), "Y has no deps and is ready");
    let blocked_ids: BTreeSet<Id> = model.blocked.iter().map(|b| b.node.id).collect();
    assert!(blocked_ids.contains(&id('X')), "X depends on unsatisfied Y, so is blocked");

    // The blocked entry names the unsatisfied edge (R-4 at the model level).
    let x_block = model.blocked.iter().find(|b| b.node.id == id('X')).unwrap();
    assert!(x_block.reasons.iter().any(|r| matches!(
        r,
        BlockReason::Unsatisfied { dep } if dep.id == id('Y')
    )));

    // Provenance covers every node (all planned here).
    assert_eq!(model.provenance.planned.len(), 4);
    assert!(model.provenance.discovered.is_empty());

    // A3 slots: drift present-but-empty, deferred empty (Q-A3-1 / Q-A3-2).
    assert_eq!(model.drift, odm_core::rollup::Drift::default(), "drift carries no data until A5");
    assert!(model.deferred.nodes.is_empty(), "no deferred surfacing until A5");

    // No active tears in this corpus.
    assert!(model.tears.is_empty());
}

// ----- R-1 (cont.): active tears carry their rationale ----------------------

#[test]
fn rollup_model_assembles_active_tears_with_rationale() {
    let gates = gates();
    let mut nodes = corpus();
    // Make Y depend on X too (a cycle X<->Y), then tear Y->X with a rationale.
    let yi = nodes.iter().position(|f| f.id() == id('Y')).unwrap();
    nodes[yi].edges_mut().depends_on.push(Dependency::Bare(id('X')));
    nodes[yi].edges_mut().tears.push(TornEdge {
        edge: Dependency::Bare(id('X')),
        because: "assume X to break the X<->Y cycle".to_string(),
    });

    let model = Rollup::assemble(&nodes, &gates, Evidence::Reproduced);
    assert_eq!(model.tears.len(), 1, "one active tear");
    let tear = &model.tears[0];
    assert_eq!(tear.from.id, id('Y'));
    assert_eq!(tear.to.id, id('X'));
    assert_eq!(tear.because, "assume X to break the X<->Y cycle");
}

// ----- R-2: the way-finding tree is total and unambiguous -------------------

#[test]
fn rollup_tree_total_single_parent_no_orphans() {
    let gates = gates();
    let nodes = corpus();
    let all: BTreeSet<Id> = nodes.iter().map(Frontmatter::id).collect();
    let model = Rollup::assemble(&nodes, &gates, Evidence::Reproduced);

    // Collect every id reachable through the tree.
    let mut seen = BTreeSet::new();
    fn walk(node: &odm_core::rollup::TreeNode, seen: &mut BTreeSet<Id>) {
        assert!(seen.insert(node.node.id), "no node appears twice in the tree");
        for c in &node.children {
            walk(c, seen);
        }
    }
    for root in &model.tree {
        walk(root, &mut seen);
    }

    // Every node in the corpus is placed exactly once (recomposition is total).
    assert_eq!(seen, all, "the tree places every node exactly once");

    // Structure: P -> Q -> {X, Y}, children sorted by id.
    let p = &model.tree[0];
    assert_eq!(p.node.id, id('P'));
    assert_eq!(p.children.len(), 1);
    let q = &p.children[0];
    assert_eq!(q.node.id, id('Q'));
    let kids: Vec<Id> = q.children.iter().map(|c| c.node.id).collect();
    assert_eq!(kids, vec![id('X'), id('Y')], "children sorted by id");
}

// ----- R-3: status vectors render in gate-sequence order, not alphabetical --

#[test]
fn rollup_status_gate_order_follows_sequence() {
    let gates = gates();
    let mut q = child('Q', 2, NodeType::Arc, 'P');
    // Reach an out-of-alphabetical-order gate set: planned + complete (skipping
    // in-progress). Alphabetical order would be complete, in-progress, planned.
    reach(&mut q, &gates, "planned", Evidence::Reproduced);
    reach(&mut q, &gates, "complete", Evidence::Asserted);
    let p = node('P', 1, NodeType::Project);

    let model = Rollup::assemble(&[p, q], &gates, Evidence::Reproduced);
    let q_node = &model.tree[0].children[0];

    // The vector is the full sequence in order, not alphabetical.
    let gate_order: Vec<&str> = q_node.status.iter().map(|g| g.gate.as_str()).collect();
    assert_eq!(gate_order, vec!["planned", "in-progress", "complete", "verified"]);

    // Reached gates carry their evidence; absent gates are not-reached (None).
    assert_eq!(q_node.status[0].evidence, Some(Evidence::Reproduced)); // planned
    assert_eq!(q_node.status[1].evidence, None); // in-progress (absent)
    assert_eq!(q_node.status[2].evidence, Some(Evidence::Asserted)); // complete
    assert_eq!(q_node.status[3].evidence, None); // verified (absent)
}

// ----- R-3 (cont.): types without a gate-set have an empty status vector ----

#[test]
fn rollup_status_gate_order_empty_for_documents() {
    let gates = gates();
    // A standalone document node (root, no gate-set configured for `note`).
    let n = node('N', 1, NodeType::Note);
    let model = Rollup::assemble(&[n], &gates, Evidence::Reproduced);
    assert_eq!(model.tree.len(), 1);
    assert!(model.tree[0].status.is_empty(), "no gate-set ⇒ empty status vector");
}
