# CDC Verification — Arc 02 / Slice 01: Graph construction + reverse edges

> Independent verification of CC's closed ledger (impl `f2f74f0`; closed `1d382fe`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here. First slice of Arc 02.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 8 opened, 8 addressed. No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
H-6 — `odm-graph` is domain-agnostic: the literal grep is clean **and** the stronger
invariant holds — `odm-graph` has **no dependency on `odm-core`** (verified in its
Cargo.toml + src); `Graph<N, E>` is generic with `EdgeKind` living in `odm-core`;
ordering DAG = `DependsOn ∪ Consumes` (`ordering_successors`); no `unsafe` (H-7
half). → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
H-1 (graph_build), H-2 (reverse = transpose proptest), H-3 (ordering-DAG
membership), H-4 (part_of tree), H-5 (adjacency by kind), H-7 (clippy), H-8 (line
97.67%). → **PENDING CI.**

## Rulings on CC's flagged items

1. **Generic engine + `EdgeKind` in `odm-core` (H-6 satisfied structurally).**
   **Accepted — exemplary.** The engine is domain-agnostic *by construction*; there
   is no domain vocabulary to leak. Verified the real invariant (no `odm-core` dep
   in `odm-graph`). *CC's grep-sharpness flag is valid:* H-6's
   `grep -REiq 'project|arc|slice|odd|adr|gate'` is substring + case-insensitive, so
   it would false-positive on `std::sync::Arc`, "aggregate", "search". CC avoided
   those incidentally; the durable guard is the **dependency-direction check**
   (`odm-graph` must not depend on `odm-core`), which I reproduced. *Forward note:*
   future domain-agnostic checks should use that dependency check, not a name-grep.
2. **Build is total; dangling edges skipped (`check` owns ref-resolution).**
   **Accepted.** Clean separation: the graph is built from *resolvable* edges;
   `check` v1 (arc01 slice06) flags dangling refs. *Interplay note for slice 04:*
   `next`/satisfaction run on the built graph (resolvable edges only), so a dangling
   `depends_on` won't gate readiness — but `check` flags it first, so a corpus is
   corrected before `next` is trusted. Slice 04 should keep this division in mind.
3. **Single-parent enforced by the schema (`part_of: Option<Id>`).** **Accepted** —
   matches Q-4 (containment is a single-parent tree); recomposition stays
   unambiguous.
4. **No cycle/acyclicity handling yet.** **Accepted** — correctly scoped to slice 02
   (cycles + tears); a cyclic corpus building fine here is expected.

## Verdict

Arc 02 / Slice 01 **CDC-verified on structure; all items accepted; cargo rows
pending CI.** On CI green, it closes and slice 02 (cycles + tears) opens. The graph
engine's domain-agnostic foundation is in place.

CDC: planning thread, 2026-06-22. Iterations used: 1.
