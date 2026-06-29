//! The index→graph adapter (ODD-0014 §2.4): feed the existing odm-core graph and
//! satisfaction engines from index records, **without parsing frontmatter**.
//!
//! `NodeGraph::build` and `Satisfaction::compute` both consume `&[Frontmatter]`,
//! and `compute` needs evidence *levels* — which the record now carries
//! (slice04). Rather than add index-native constructors to odm-core (a wider
//! seam), this **reconstructs `Frontmatter`s from records** (the inverse of
//! slice02's `map_edges` plus a status rebuild) and feeds the existing builders
//! **unchanged** — so no graph or satisfaction logic is re-derived (slice04
//! closing-report sketch; G-1 decision).
//!
//! Fidelity is exact for the graph + satisfaction (id, type, edges + qualifiers,
//! reached gates + evidence) **and**, since slice06, for `rollup`'s provenance
//! (`origin`) and `check`'s recomposition (`decomposed`) — the record carries
//! both. The reconstructed frontmatter is "the full frontmatter projection minus
//! the body" (§3.5): it has no body, so `orient`'s vision still does one targeted
//! `store.load(project)`.

use odm_core::frontmatter::{
    Dependency, Edges, Frontmatter, SupersedeKind as CoreSupersedeKind, Supersedes, TornEdge,
};
use odm_core::gates::GateSets;

use crate::record::{EdgeKind, EdgeQualifier, IndexRecord, SupersedeKind};

/// Reconstructs the corpus `Frontmatter`s from index records, faithfully enough
/// to feed `NodeGraph::build` / `Satisfaction::compute` (id, type, edges, status
/// with evidence). `gates` validates the reconstructed status (a node's reached
/// gate is set only if its type has a configured gate-set; a gate the current
/// config no longer knows is skipped — best-effort, matching how satisfaction
/// already reads only configured gates).
///
/// The result is **id-ordered** (records are id-sorted), matching `load_all`.
#[must_use]
pub fn frontmatters_from_records(records: &[IndexRecord], gates: &GateSets) -> Vec<Frontmatter> {
    records.iter().map(|r| frontmatter_from_record(r, gates)).collect()
}

/// Reconstructs one `Frontmatter` from a record.
fn frontmatter_from_record(record: &IndexRecord, gates: &GateSets) -> Frontmatter {
    // `created` is derivable from the ULID (the store derives the path the same
    // way). `origin` round-trips from the record (slice06) — the rollup's
    // provenance view reads it.
    let created = record.id.created_at().date_naive();
    let mut fm = Frontmatter::new(
        record.id,
        record.number,
        record.node_type,
        record.title.clone(),
        created,
        record.updated,
        record.origin,
    )
    .with_edges(edges_from_record(record));

    // Rebuild the status vector: each reached gate at its recorded evidence. The
    // reach *date* is unused by graph/satisfaction, so the record's `updated`
    // stands in. A gate whose type has no configured gate-set is skipped.
    if let Some(gate_set) = gates.for_type(record.node_type) {
        for gate in &record.gates {
            // Ignore an unknown-gate error (config drift) — best-effort fidelity.
            let _ =
                fm.status_mut().set_gate(gate_set, &gate.gate, None, gate.evidence, record.updated);
        }
    }

    // Re-affirm the decomposition assertion (slice06) — `check`'s recomposition
    // reads it. `affirm_decomposed` re-sorts/dedups, but the record's children
    // are already sorted+deduped, so this round-trips identically.
    if let Some(decomposed) = &record.decomposed {
        fm.affirm_decomposed(decomposed.children.clone(), decomposed.on);
    }
    fm
}

/// Reconstructs a node's [`Edges`] from its [`EdgeRef`](crate::record::EdgeRef)s —
/// the inverse of slice02's `map_edges`, preserving each kind's qualifier.
fn edges_from_record(record: &IndexRecord) -> Edges {
    let mut edges = Edges::default();
    for edge in &record.edges {
        match edge.kind {
            EdgeKind::PartOf => edges.part_of = Some(edge.target),
            EdgeKind::DependsOn => {
                let dep = match &edge.qualifier {
                    Some(EdgeQualifier::SatisfiedAt(gate)) => {
                        Dependency::Qualified { node: edge.target, satisfied_at: gate.clone() }
                    }
                    _ => Dependency::Bare(edge.target),
                };
                edges.depends_on.push(dep);
            }
            EdgeKind::BlockedBy => edges.blocked_by.push(edge.target),
            EdgeKind::Verifies => edges.verifies.push(edge.target),
            EdgeKind::Consumes => edges.consumes.push(edge.target),
            EdgeKind::Affects => edges.affects.push(edge.target),
            EdgeKind::Supersedes => {
                let kind = match &edge.qualifier {
                    Some(EdgeQualifier::Supersede(SupersedeKind::Updates)) => {
                        CoreSupersedeKind::Updates
                    }
                    _ => CoreSupersedeKind::Obsoletes,
                };
                edges.supersedes = Some(Supersedes { node: edge.target, kind });
            }
            EdgeKind::Tears => {
                let because = match &edge.qualifier {
                    Some(EdgeQualifier::Because(text)) => text.clone(),
                    _ => String::new(),
                };
                edges.tears.push(TornEdge { edge: Dependency::Bare(edge.target), because });
            }
        }
    }
    edges
}
