# Slice 04 (Arc 02): Derived order & satisfaction

> Per LEDGER_DISCIPLINE. Final status + evidence (commit SHA + Verify output)
> before advancing. Compile/test rows reproduced by CDC in CI or a local 1.85+
> toolchain (the Cowork sandbox has none). Five-iteration cap. Rows H-6…H-10 are
> the evidence-leveled-satisfaction work (ODD-0013 §4.4).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | Topological order over `depends_on ∪ consumes` (Kahn); acyclic ⇒ valid order | `cargo test -p odm-graph topo_order` → ok | serious | 0013 §4.1 | done | `547b5f2`: `topo_order` → 1 passed. Dependency-before-dependent order; a cycle → `Err(Cycle)`. | |
| H-2 | `next` = deps satisfied ∧ no active `blocked_by` ∧ not complete | `cargo test -p odm-graph next_ready_frontier` → ok | serious | 0013 §4.1, 0001 B2 | done | `547b5f2`: `next_ready_frontier` → 1 passed (+ `next_excludes_complete_and_blocked`). | Active block = blocked-by edge to a not-complete node. |
| H-3 | `blocked X` lists each unsatisfied dependency + why | `cargo test -p odm-graph blocked_reasons` → ok | correctness | 0013 §4.1, 0001 B3 | done | `547b5f2`: `blocked_reasons` → 1 passed. `Unsatisfied`/`SoftSatisfied`/`ExternallyBlocked` reasons. | |
| H-4 | `path X [Y]` returns the dependency chain / critical path | `cargo test -p odm-graph path_chain` → ok | correctness | 0013 §4.1 | done | `547b5f2`: `path_chain` → 1 passed. `path(X,None)` = longest dep chain; `path(X,Some Y)` = a path or `None`. | |
| H-5 | Satisfaction: edge satisfied iff target reached `satisfied_at` (default terminal) gate | `cargo test -p odm-core satisfaction_gate` → ok | serious | 0013 §4.4 | done | `547b5f2`: `satisfaction_gate` → 1 passed. `Some(evidence)` iff target reached the gate (satisfied_at, else terminal). | |
| H-6 | Evidence ordering total + correct: `asserted < attested < reproduced < reconciled` | `cargo test -p odm-core evidence_ordering` → ok | serious | 0013 §4.4 / 0001 D3 | done | `547b5f2`: `evidence_ordering` → 1 passed (sort yields canonical order). | Canonical `Evidence` from slice03. |
| H-7 | **Min-propagation**: a node's effective evidence = min over its transitive dependency path | `cargo test -p odm-graph evidence_min_propagation` (proptest …) → ok | serious | 0013 §4.4 | done | `547b5f2`: `evidence_min_propagation` → 1 passed (unit) + `min_propagation_is_monotone` (128-case proptest: chain effective = min link; lowering a link lowers it). | |
| H-8 | Threshold (default `reproduced`) configurable via `odm.toml`; below-threshold ⇒ soft-satisfied | `cargo test -p odm-core satisfaction_threshold` (default + override) → ok | serious | 0013 §4.4 | done | `547b5f2`: `satisfaction_threshold` → 1 passed. Absent → `reproduced`; `threshold="attested"` override; `is_soft`; bad level → error. | |
| H-9 | `next` flags soft-satisfied deps; `blocked X` names the low-evidence dep + how to raise it | `cargo test -p odm-graph soft_satisfied_surfacing` (…) → ok | serious | 0013 §4.4, 0001 F2/G3 | done | `547b5f2`: `soft_satisfied_surfacing` → 1 passed; `SoftSatisfied{dep, evidence, threshold}` (threshold = how to raise). CLI renders `⚠ dep … evidence=…` (H-12). | |
| H-10 | Soft-satisfied does NOT block: `next` still lists the node (visibility, not gating) | `cargo test -p odm-graph soft_satisfied_not_blocking` → ok | correctness | 0013 §4.4 | done | `547b5f2`: `soft_satisfied_not_blocking` → 1 passed. Below-threshold dep stays in `next`, flagged via `Ready.soft`. | |
| H-11 | Staleness guard: advancing a node with an unsatisfied `depends_on` warns | `cargo test -p odm-core staleness_guard` → ok | serious | 0013 §4.4, 0001 B3 | done | `547b5f2`: `staleness_guard` → 1 passed. `staleness_on_advance` → `Some(Staleness)` when deps unsatisfied, else `None` (non-fatal). | |
| H-12 | `--json` for `next`/`blocked`/`path` is stable + carries the evidence level | `cargo test -p odm-cli json_schema_derived_order` (snapshot) → ok | correctness | 0013 §7 | done | `547b5f2`: `json_schema_derived_order` → 1 passed. `next` JSON carries `effective_evidence` + soft evidence; `blocked` carries `evidence`+`threshold`; `path` carries the chain. | |
| H-13 | Clippy clean (`-D warnings`); no `unsafe`; no panics on public paths | `cargo clippy -p odm-graph -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-graph/src crates/odm-core/src` | serious | CLAUDE.md / rust-guidelines | done | `547b5f2`: clippy (graph+core, also cli) → exit 0; unsafe grep → no match; odm-graph stays domain-agnostic. | Public paths return `Result`/`Option`; no `unwrap`/`expect` in src. |
| H-14 | Coverage ≥ 90% (target 95%) for the new graph/satisfaction code | `cargo llvm-cov -p odm-graph -p odm-core --summary-only --ignore-filename-regex '(odm-store|odm-cli)/'` → **line** ≥ 90% (target 95%) | correctness | CLAUDE.md | done | `547b5f2`: TOTAL **line 96.59%**, region 95.93% (order.rs 96.35%, satisfaction.rs 92.94%, graph.rs 95.35% line). | Run from a clean `target/llvm-cov-target`. |
| H-15 | Typed `Status` (slice03) wired into `Frontmatter`, replacing the preserved-unknown-key passthrough; satisfaction reads it | `cargo test -p odm-core frontmatter_roundtrip status_typed_field` → ok AND `grep -qE 'status' crates/odm-core/src/frontmatter.rs` (typed field present) | serious | slice03 deferral / 0013 §2.3 | done | `547b5f2`: `status_typed_field` → 1 + `frontmatter_roundtrip` → 1 passed; `status` is a typed field (grep present). `Satisfaction::compute` reads `fm.status()`. | Skip-serialized when empty; canonical-order snapshot unaffected. |
| H-16 | arc01 `unknown_keys_preserved` updated **2→1** (status now a known typed field, not preserved-unknown — expected, disclosed change to the closed test) | `cargo test -p odm-core unknown_keys_preserved` → ok (asserts 1 remaining preserved key, not 2) | serious | slice03 deferral | done | `547b5f2`: `unknown_keys_preserved` → 2 passed; the round-trip test now asserts `unknown_key_count == 1` (only `desired_facts`) and `status().has_reached("built")`. | Disclosed 2→1 change to the closed arc01 test. |

## What Worked

- **Generic engine, domain types at the edge.** Derived order, the ready
  frontier, blocked reasons, paths, and min-propagation all live in `odm-graph`
  over `Graph<N,E>` with a caller-supplied confidence level `L: Ord` — no domain
  vocabulary (H-6 substring grep stays clean). The domain mapping (`Evidence`,
  gates, `EdgeKind`) lives in `odm-core::satisfaction`; `NodeGraph` bridges them.
- **Threshold split from satisfaction.** "Satisfied" (target reached its gate)
  and "fully satisfied" (evidence ≥ threshold) are distinct: below-threshold is
  *soft-satisfied* — surfaced on `next` (`Ready.soft`) and named by `blocked`
  (`SoftSatisfied{dep, evidence, threshold}`), but never withholds a node. This
  kept H-2/H-9/H-10 cleanly separable.
- **Min-propagation as a weakest-link fold** over the transitive dep set,
  proved monotone by a 128-case proptest (H-7) rather than only example tests.
- **Cycle-first topo.** `topological_order` runs `detect_cycle` first and
  returns `Err(Cycle)` before peeling, so the acyclic path never needs a panic
  fallback (the `debug_assert!` is belt-and-suspenders only).
- **In-process CLI tests** via `dispatch` (odm-cli is library-only) carried the
  JSON-schema row (H-12) without a test-only binary.

## Closure

Closed at commit `547b5f2` on `2026-06-24`. CDC verification: pending
(compile/test rows reproduced by CDC in CI or a local 1.85+ toolchain — the
Cowork sandbox has none). All `done` states are *proposed done* pending that
independent verification. Total rows: 16. Done: 16. Deferred: 0. No-op: 0.

**Flagged for CDC.** (1) *Tears carry no rationale*: frontmatter `tears:
Vec<Dependency>` has no rationale field, so `Satisfaction` passes empty tears at
the CLI level — tear-aware ordering is exercised in the graph layer only;
rationale-bearing tears are deferred to the arc that needs them. (2) *arc01 test
changed 2→1* (H-16): wiring typed `Status` into `Frontmatter` turned `status`
from a preserved-unknown key into a known field, a disclosed change to a
closed slice's test. (3) `serde_norway` chosen over the stale `serde_yaml`
family (both >12mo) — typed round-trip probed (H-15).
