//! Tests for the index→graph adapter (Arc 04 slice05, G-1).
//!
//! The fidelity guard: a graph + satisfaction built from adapter-reconstructed
//! frontmatters behaves **identically** to one built from the corpus
//! frontmatters. Test name carries the ledger Verify substring
//! (`index_graph_adapter_equals_frontmatter_graph`).

use chrono::NaiveDate;
use odm_core::frontmatter::{
    Dependency, Document, Frontmatter, SupersedeKind as CoreSupersedeKind, Supersedes, TornEdge,
};
use odm_core::gates::{GateSet, GateSets};
use odm_core::graph::NodeGraph;
use odm_core::satisfaction::Satisfaction;
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use odm_index::{build_records, frontmatters_from_records};
use odm_store::Store;
use tempfile::TempDir;

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 26).unwrap()
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

/// `slice` gate-set, for setting a node's status in the test corpus.
fn slice_gates() -> GateSet {
    GateSet::new(vec!["planned".to_string(), "built".to_string(), "tested".to_string()])
}

#[test]
fn index_graph_adapter_equals_frontmatter_graph() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    std::fs::write(dir.path().join("odm.toml"), CONFIG).unwrap();

    // A corpus exercising the graph + evidence-leveled satisfaction:
    //   P(project) ← A(arc) ← Early(slice), Late(slice, depends_on Early@built)
    //   Soft(slice) reached `tested` only at `attested`; Reader depends_on Soft.
    let mut real: Vec<Frontmatter> = Vec::new();

    let p = Id::new();
    real.push(Frontmatter::new(p, 1, NodeType::Project, "P", day(), day(), Origin::Planned));

    let a = Id::new();
    let mut af = Frontmatter::new(a, 2, NodeType::Arc, "A", day(), day(), Origin::Planned);
    af.edges_mut().part_of = Some(p);
    real.push(af);

    let early = Id::new();
    let mut ef =
        Frontmatter::new(early, 3, NodeType::Slice, "Early", day(), day(), Origin::Planned);
    ef.edges_mut().part_of = Some(a);
    ef.status_mut().set_gate(&slice_gates(), "tested", None, Evidence::Reconciled, day()).unwrap();
    real.push(ef);

    let late = Id::new();
    let mut lf = Frontmatter::new(late, 4, NodeType::Slice, "Late", day(), day(), Origin::Planned);
    lf.edges_mut().part_of = Some(a);
    lf.edges_mut()
        .depends_on
        .push(Dependency::Qualified { node: early, satisfied_at: "built".to_string() });
    real.push(lf);

    let soft = Id::new();
    let mut sf = Frontmatter::new(soft, 5, NodeType::Slice, "Soft", day(), day(), Origin::Planned);
    sf.status_mut().set_gate(&slice_gates(), "tested", None, Evidence::Attested, day()).unwrap();
    real.push(sf);

    // Reader exercises every remaining edge kind (so the adapter's
    // `edges_from_record` is fully covered): depends_on + blocked_by + verifies
    // + consumes + affects + a tear (of the Soft dep).
    let reader = Id::new();
    let mut rf =
        Frontmatter::new(reader, 6, NodeType::Slice, "Reader", day(), day(), Origin::Planned);
    {
        let e = rf.edges_mut();
        e.depends_on.push(Dependency::Bare(soft));
        e.blocked_by.push(early);
        e.verifies.push(early);
        e.consumes.push(soft);
        e.affects.push(p);
        e.tears.push(TornEdge { edge: Dependency::Bare(soft), because: "assume Soft".to_string() });
    }
    real.push(rf);

    // A supersedes edge (Updates kind) and a gate-less node type (`note` has no
    // configured gate-set → the adapter's status rebuild skips it).
    let superseder = Id::new();
    let mut xf = Frontmatter::new(
        superseder,
        7,
        NodeType::Slice,
        "Superseder",
        day(),
        day(),
        Origin::Planned,
    );
    xf.edges_mut().supersedes = Some(Supersedes { node: early, kind: CoreSupersedeKind::Updates });
    real.push(xf);

    real.push(Frontmatter::new(
        Id::new(),
        8,
        NodeType::Note,
        "Note",
        day(),
        day(),
        Origin::Planned,
    ));

    // Persist, then reconstruct from the built index records.
    for fm in &real {
        store.persist(&Document::new(fm.clone(), "# n\n")).unwrap();
    }
    real.sort_by_key(Frontmatter::id); // id order, so both graphs share insertion order
    let records = build_records(&store).unwrap();
    let synth = frontmatters_from_records(&records, &gates());

    // The reconstruction must recover the same id set.
    let real_ids: Vec<Id> = real.iter().map(Frontmatter::id).collect();
    let synth_ids: Vec<Id> = synth.iter().map(Frontmatter::id).collect();
    assert_eq!(real_ids, synth_ids, "same nodes, same order");

    let real_graph = NodeGraph::build(&real);
    let synth_graph = NodeGraph::build(&synth);
    let real_sat = Satisfaction::compute(&real, &gates(), Evidence::Reproduced);
    let synth_sat = Satisfaction::compute(&synth, &gates(), Evidence::Reproduced);

    // The ready frontier (incl. soft-sat flags) is identical.
    assert_eq!(
        real_graph.next(&real_sat),
        synth_graph.next(&synth_sat),
        "the ready frontier is identical"
    );

    // Per-node blocked reasons (evidence-leveled satisfaction) are identical.
    for &id in &real_ids {
        assert_eq!(
            real_graph.blocked(id, &real_sat),
            synth_graph.blocked(id, &synth_sat),
            "blocked({id}) identical"
        );
    }

    // The topological order is identical.
    assert_eq!(
        real_graph.topological_order(&[]),
        synth_graph.topological_order(&[]),
        "topological order identical"
    );

    // The containment children (part_of) round-trip — the tree the rollup uses.
    assert_eq!(real_graph.children(a), synth_graph.children(a));
}
