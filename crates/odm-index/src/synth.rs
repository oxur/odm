//! Seeded synthetic-corpus generation for the Arc 04 benchmark harness
//! (slice08, ODD-0014 §4: "validate on a synthetic 100k corpus before declaring
//! victory").
//!
//! This is **test/benchmark support that happens to live in the library** so both
//! the `benches/index_bench.rs` harness and the unit test that guards
//! reproducibility can call it. It builds a *realistic* corpus — a `part_of`
//! forest (project → arcs → slices), `depends_on`/`blocked_by` edges, reached
//! gates with evidence, varied `origin`, and some affirmed `decomposed` — so the
//! benchmark exercises the **real** build / adapter / graph paths, not empty
//! nodes.
//!
//! Determinism is the contract: the same `seed` yields a byte-identical corpus
//! (identical ids ⇒ identical filenames ⇒ identical content). To avoid a `rand`
//! dependency and to keep ids reproducible (the store derives each node's path
//! from its id), ids are minted from a seeded [`SplitMix64`](Rng) PRNG rendered
//! into a valid Crockford-base32 ULID string — not [`Id::new`], which is
//! clock+entropy based.

use std::collections::HashMap;

use chrono::NaiveDate;
use odm_core::frontmatter::{Dependency, Document, Frontmatter};
use odm_core::gates::GateSet;
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;

/// The gate configuration the synthetic corpus is built against. The generator
/// writes this to `<root>/odm.toml` so a consumer read (`reconcile` → adapter →
/// graph → `check`/`next`) finds the gate-sets it needs.
pub const GATE_CONFIG: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

/// Crockford base32 (ULID alphabet — excludes `I`, `L`, `O`, `U`).
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// A tiny deterministic PRNG (SplitMix64) — seeded, dependency-free, so the same
/// seed produces an identical corpus.
struct Rng(u64);

impl Rng {
    /// Seeds the generator.
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The next pseudo-random `u64` (SplitMix64).
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A pseudo-random value in `0..n` (caller guarantees `n > 0`).
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// Mints a deterministic, valid [`Id`] from the PRNG: a 26-char Crockford ULID
/// whose first character is `0..=7` (so the 130-bit base32 string stays within
/// the 128-bit ULID range and always parses).
fn det_id(rng: &mut Rng) -> Id {
    let mut s = String::with_capacity(26);
    s.push(CROCKFORD[(rng.next_u64() % 8) as usize] as char);
    for _ in 0..25 {
        s.push(CROCKFORD[(rng.next_u64() % 32) as usize] as char);
    }
    s.parse().expect("a constructed-valid ULID string")
}

/// The fixed creation/updated date stamped on every synthetic node.
fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 26).unwrap()
}

/// Cycles `origin` so provenance has all three buckets populated.
fn origin_for(i: usize) -> Origin {
    match i % 3 {
        0 => Origin::Planned,
        1 => Origin::Discovered,
        _ => Origin::Amendment,
    }
}

/// Writes a reproducible synthetic corpus of `n` nodes (`n >= 1`) into `store`,
/// plus an `odm.toml` ([`GATE_CONFIG`]). The shape: node 0 is the project; the
/// next `max(1, n/50)` are arcs (`part_of` the project); the rest are slices
/// (`part_of` a pseudo-random arc, some with a `depends_on`/`blocked_by` edge to
/// an earlier slice). About two-thirds of nodes reach a gate at a cycling
/// evidence level; even-indexed arcs affirm `decomposed` over their children.
///
/// Returns the node ids in creation order. The **same `seed` yields the same
/// ids** (and thus an identical corpus).
///
/// # Panics
///
/// Panics if `n == 0`, or if a node file cannot be persisted (the benchmark
/// harness has no error path — a failed write is a setup bug, not a result).
#[must_use]
pub fn generate_corpus(store: &Store, n: usize, seed: u64) -> Vec<Id> {
    assert!(n >= 1, "a corpus needs at least the project node");
    std::fs::write(store.root().join("odm.toml"), GATE_CONFIG).expect("write odm.toml");

    let slice_gates = GateSet::new(vec!["planned".into(), "built".into(), "tested".into()]);
    let parent_gates = GateSet::new(vec![
        "planned".into(),
        "in-progress".into(),
        "complete".into(),
        "verified".into(),
    ]);
    let evidences =
        [Evidence::Asserted, Evidence::Attested, Evidence::Reproduced, Evidence::Reconciled];

    let mut rng = Rng::new(seed);
    let ids: Vec<Id> = (0..n).map(|_| det_id(&mut rng)).collect();

    // ~2% of nodes are arcs, at least one, never more than leaves room for.
    let num_arcs = (n / 50).clamp(1, n.saturating_sub(1).max(1));
    let first_slice = num_arcs + 1; // index of the first slice node

    let mut nodes: Vec<Frontmatter> = Vec::with_capacity(n);
    let mut children: HashMap<usize, Vec<Id>> = HashMap::new();
    let mut ev = 0usize;

    for (i, &id) in ids.iter().enumerate() {
        let number = (i + 1) as u32;
        let origin = origin_for(i);

        if i == 0 {
            // The project root.
            nodes.push(Frontmatter::new(
                id,
                number,
                NodeType::Project,
                "Project",
                day(),
                day(),
                origin,
            ));
        } else if i < first_slice {
            // An arc, contained by the project.
            let mut fm = Frontmatter::new(id, number, NodeType::Arc, "Arc", day(), day(), origin);
            fm.edges_mut().part_of = Some(ids[0]);
            children.entry(0).or_default().push(id);
            if i % 2 == 0 {
                fm.status_mut()
                    .set_gate(&parent_gates, "planned", None, evidences[ev % 4], day())
                    .expect("valid arc gate");
                ev += 1;
            }
            nodes.push(fm);
        } else {
            // A slice, contained by a pseudo-random arc.
            let arc_idx = 1 + rng.below(num_arcs);
            let mut fm =
                Frontmatter::new(id, number, NodeType::Slice, "Slice", day(), day(), origin);
            fm.edges_mut().part_of = Some(ids[arc_idx]);
            children.entry(arc_idx).or_default().push(id);

            // ~1/3 depend on an earlier slice; ~1/10 are externally blocked by one.
            if i > first_slice {
                let span = i - first_slice;
                if rng.below(3) == 0 {
                    fm.edges_mut()
                        .depends_on
                        .push(Dependency::Bare(ids[first_slice + rng.below(span)]));
                }
                if rng.below(10) == 0 {
                    fm.edges_mut().blocked_by.push(ids[first_slice + rng.below(span)]);
                }
            }

            // ~2/3 reach a gate at a cycling evidence level.
            if rng.below(3) != 0 {
                let gate = if rng.below(2) == 0 { "planned" } else { "built" };
                fm.status_mut()
                    .set_gate(&slice_gates, gate, None, evidences[ev % 4], day())
                    .expect("valid slice gate");
                ev += 1;
            }
            nodes.push(fm);
        }
    }

    // Even-indexed arcs affirm their decomposition (exercises `decomposed`).
    for (arc_idx, node) in nodes.iter_mut().enumerate().take(first_slice) {
        if arc_idx >= 1 && arc_idx % 2 == 0 {
            if let Some(kids) = children.get(&arc_idx) {
                node.affirm_decomposed(kids.clone(), day());
            }
        }
    }

    for fm in nodes {
        let body = if fm.node_type() == NodeType::Project {
            format!("# Vision\n\nSynthetic benchmark corpus ({n} nodes).\n")
        } else {
            format!("# {} #{}\n\nSynthetic body.\n", fm.name(), fm.number())
        };
        store.persist(&Document::new(fm, body)).expect("persist synthetic node");
    }

    ids
}
