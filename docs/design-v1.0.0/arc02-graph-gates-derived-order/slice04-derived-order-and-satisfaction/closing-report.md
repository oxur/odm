# Closing Report — Slice 04 (Arc 02): Derived order & satisfaction

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 05 opens.

- **Implementation commit:** `547b5f2`.
- **Branch:** `arc02-slice04-derived-order-and-satisfaction` (based on
  `arc02-slice03-gates`; not pushed; not merged to `main`).
- **Scope delivered:** the *consuming* half of evidence-leveled status —
  derived-order queries (`topological_order`, `next`, `blocked`, `path`,
  `min_evidence`) as a domain-agnostic engine in odm-graph; edge satisfaction +
  threshold + soft classification + staleness guard in `odm-core::satisfaction`;
  the `NodeGraph` bridge; `odm next|blocked|path` (+ `--json`) in odm-cli. Also
  the deferred slice03 integration: typed `Status` wired into `Frontmatter`.
- **Result:** 16 rows, all `done`. 0 deferred, 0 no-op. CDC added H-15
  (status-wiring) and H-16 (arc01 `unknown_keys` 2→1) during review.
- **Aggregate gates:** `cargo test` (graph/core/cli) → all pass; clippy
  `-D warnings` (graph+core+cli) → exit 0; no `unsafe`; H-6 domain-agnostic grep
  clean; fmt clean; coverage TOTAL line 96.59% / region 95.93%.

## Per-row walk

| ID | Status | Evidence (re-runnable at `547b5f2`) |
|----|--------|-------------------------------------|
| H-1 | done | `cargo test -p odm-graph topo_order` → 1 passed; dependency-before-dependent; cycle → `Err(Cycle)`. |
| H-2 | done | `cargo test -p odm-graph next_ready_frontier` → 1 (+ `next_excludes_complete_and_blocked`); deps-satisfied ∧ no active block ∧ not complete. |
| H-3 | done | `cargo test -p odm-graph blocked_reasons` → 1; `Unsatisfied`/`SoftSatisfied`/`ExternallyBlocked`. |
| H-4 | done | `cargo test -p odm-graph path_chain` → 1; `path(X,None)` = longest dep chain, `path(X,Some Y)` = a path or `None`. |
| H-5 | done | `cargo test -p odm-core satisfaction_gate` → 1; satisfied iff target reached `satisfied_at` (else terminal). |
| H-6 | done | `cargo test -p odm-core evidence_ordering` → 1; `asserted < attested < reproduced < reconciled`. |
| H-7 | done | `cargo test -p odm-graph evidence_min_propagation` → 1 + `min_propagation_is_monotone` (128-case proptest). |
| H-8 | done | `cargo test -p odm-core satisfaction_threshold` → 1; default `reproduced`, `odm.toml` override, bad level → error. |
| H-9 | done | `cargo test -p odm-graph soft_satisfied_surfacing` → 1; `SoftSatisfied{dep, evidence, threshold}` names how to raise. |
| H-10 | done | `cargo test -p odm-graph soft_satisfied_not_blocking` → 1; below-threshold dep stays in `next`, flagged via `Ready.soft`. |
| H-11 | done | `cargo test -p odm-core staleness_guard` → 1; `staleness_on_advance` → `Some` when deps unsatisfied, else `None`. |
| H-12 | done | `cargo test -p odm-cli json_schema_derived_order` → 1; `next`/`blocked`/`path` JSON carry the evidence level. |
| H-13 | done | clippy (graph+core+cli) → exit 0; `! grep '\bunsafe\b' …/src` → no match; H-6 grep clean. |
| H-14 | done | `cargo llvm-cov -p odm-graph -p odm-core …` → TOTAL line 96.59% / region 95.93% (from a clean `target/llvm-cov-target`). |
| H-15 | done | `cargo test -p odm-core status_typed_field … frontmatter_roundtrip` → pass; `status` is a typed field; `Satisfaction::compute` reads `fm.status()`. |
| H-16 | done | `cargo test -p odm-core unknown_keys_preserved` → pass; round-trip now asserts `unknown_key_count == 1`. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **Domain-agnostic engine vs. domain mapping.** All derived-order logic
   (`order.rs`) lives in odm-graph over `Graph<N,E>` with a caller-supplied
   confidence level `L: Ord` — no domain vocabulary, so the H-6 substring grep
   stays clean. The domain mapping (`Evidence`, gate semantics, `EdgeKind`)
   lives in `odm-core::satisfaction`, and `NodeGraph` re-exports the order types
   (`Block`, `Ready`, `SoftDep`, `Cycle`, `Tear`) so odm-cli does **not** depend
   on odm-graph directly. Flagging the re-export as the intended dependency seam.

2. **Tears carry no rationale (deferred).** Frontmatter `tears: Vec<Dependency>`
   has no rationale field, while the graph layer's `Tear` does. `Satisfaction`
   therefore passes **empty tears** at the CLI/core level; tear-aware ordering is
   exercised only in the odm-graph tests. Rationale-bearing tears at the domain
   level are deferred to the arc that needs them (recomposition/integrity).
   Flagging the gap so CDC doesn't read empty-tears as a bug.

3. **arc01 test changed 2→1 (H-16, disclosed).** Wiring typed `Status` into
   `Frontmatter` (the slice03 deferral) turns `status` from a preserved-unknown
   key into a known typed field, lowering the `unknown_keys_preserved` count from
   2 to 1. This is an expected, disclosed change to a closed slice's test — the
   integration the slice03 closing report (decision 1) explicitly deferred here.

4. **In-process CLI tests, not `assert_cmd`.** odm-cli is library-only; the
   JSON-schema row (H-12) is driven through `dispatch(cli, root, out, err)`
   rather than spawning a binary, consistent with the slice05 in-process pivot.

5. **`BTreeMap` status ordering carries forward.** As in slice03, on-disk gates
   serialize in name order, not configured-sequence order — deterministic
   round-trip; sequence order is available from the `GateSet` for display.

## Uncertainties named

- **`min_evidence` is whole-subtree, not per-target-path.** It folds the minimum
  over the node's entire transitive satisfied-dep set (weakest-link). For the
  "critical path evidence" framing this is the conservative reading; if a future
  slice wants per-path evidence it will need a path-parameterized variant.
- **`satisfaction.rs` residue (~7%).** Uncovered lines are mostly the
  empty-tears/edge-case branches; TOTAL line 96.59% clears the 90% bar (95%
  target) comfortably.
- **Sandbox has no Rust toolchain**, so all cargo evidence was produced on the
  local dev host; CDC should reproduce on CI / a local 1.85+ run, from a clean
  `target/llvm-cov-target` (stale artifacts give bogus coverage numbers).
