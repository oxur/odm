# Slice 02 (Arc 02): Cycle detection + tears

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | Kahn detects a cycle in the ordering DAG and names its members | `cargo test -p odm-graph detect_cycle` → ok | serious | 0013 §4.2 | done | `b7514cf`: `detect_cycle` → 2 passed (`..._names_members` + `..._excludes_innocent_downstream`). Kahn detects; a DFS back-edge walk names the precise members (a 1→2→1 cycle reports {1,2}, not the downstream 3). | |
| H-2 | Acyclic graph reports no cycle | `cargo test -p odm-graph acyclic_no_cycle` → ok | correctness | 0013 §4.2 | done | `b7514cf`: `acyclic_no_cycle` → 1 passed. A chain 1→2→3 → `detect_cycle` returns `None`. Also: non-ordering-kind edges never form a cycle. | |
| H-3 | A `tears` marker removes the named `depends_on` from ordering (breaks the cycle) | `cargo test -p odm-graph tear_breaks_cycle` → ok | serious | 0013 §4.3 | done | `b7514cf`: `tear_breaks_cycle` → 1 passed. 1↔2 cycle; a `Tear(2,1,…)` excludes that edge → `detect_cycle` returns `None`. Self-loop tear also covered. | |
| H-4 | A tear requires a rationale; a tear without one is rejected | `cargo test -p odm-graph tear_requires_rationale` → ok | correctness | 0013 §4.3 | done | `b7514cf`: `tear_requires_rationale` → 1 passed. `Tear::new` with `""`/`"   "` → `Err(MissingRationale)`; a real rationale → `Ok`. | Rationale enforced at construction (whitespace-only rejected). |
| H-5 | Cycle-without-tear yields a hard error (consumed by `check` v2) | `cargo test -p odm-graph cycle_without_tear_errors` → ok | serious | 0013 §4.2/§4.3 | done | `b7514cf`: `cycle_without_tear_errors` → 1 passed. With no tears, `detect_cycle` returns `Some(Cycle)`; `Cycle<N>` is a typed `std::error::Error` whose `Display` names the members — check v2 (slice06) surfaces it as a failure. | |
| H-6 | All active tears are enumerable (so assumed deps stay visible) | `cargo test -p odm-graph list_active_tears` → ok | correctness | 0013 §4.3 | done | `b7514cf`: `list_active_tears` → 1 passed. `active_tears` returns the tears naming a real ordering edge; a tear of a non-existent edge is excluded. | |
| H-7 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-graph --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-graph/src` | serious | CLAUDE.md | done | `b7514cf`: clippy → exit 0; unsafe grep → no match. H-6 domain-agnostic grep also clean. | |
| H-8 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-graph --summary-only --ignore-filename-regex '(odm-core|odm-store|odm-cli)/'` → **line** ≥ 90% (target 95%) | correctness | CLAUDE.md | done | `b7514cf`: TOTAL **line 93.58%**, region 93.91% (cycle.rs 91.08% line, lib.rs 100%). | Residual gap is the unreachable `extract_cycle` fallback + the duplicate-queue guard — see uncertainty. Run from a clean `target/llvm-cov-target`. |

## What Worked

- **Kahn for the verdict, DFS for the witness.** Kahn (in-degree peeling) gives
  the clean yes/no the doc names — but its leftover set over-includes nodes
  *downstream* of a cycle. Following the detection with a DFS back-edge walk over
  the un-emitted nodes names the *precise* cycle members (the
  `..._excludes_innocent_downstream` test pins this). Two simple standard passes
  beat one clever one.
- **The engine stays generic; the caller owns "ordering".** `detect_cycle` takes
  `ordering_kinds: &[E]` and `tears: &[Tear<N>]`, so odm-graph never names a
  domain edge — H-6 holds by construction, and the same primitive will serve
  `next`/`blocked` (slice04) by passing the same kind set.
- **Rationale enforced at the type boundary.** `Tear::new` is the only
  constructor and it rejects an empty rationale, so an un-justified tear is
  *unrepresentable* — H-4 is a type invariant, not a runtime check callers might
  forget.
- **A cycle is a typed `Error`.** Making `Cycle<N>: std::error::Error` (with a
  member-naming `Display`) means slice06's `check` consumes it directly as a hard
  failure — no stringly-typed hand-off.

## Closure

Closed at `b7514cf` on 2026-06-24. CDC: _pending_ (cargo rows via CI / local
1.85+; sandbox has no toolchain). Total rows: 8. Done: 8. Deferred: 0. No-op: 0.
