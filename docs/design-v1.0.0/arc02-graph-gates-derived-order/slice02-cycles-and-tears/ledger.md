# Slice 02 (Arc 02): Cycle detection + tears

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | Kahn detects a cycle in the ordering DAG and names its members | `cargo test -p odm-graph detect_cycle` → ok | serious | 0013 §4.2 | open | | |
| H-2 | Acyclic graph reports no cycle | `cargo test -p odm-graph acyclic_no_cycle` → ok | correctness | 0013 §4.2 | open | | |
| H-3 | A `tears` marker removes the named `depends_on` from ordering (breaks the cycle) | `cargo test -p odm-graph tear_breaks_cycle` → ok | serious | 0013 §4.3 | open | | |
| H-4 | A tear requires a rationale; a tear without one is rejected | `cargo test -p odm-graph tear_requires_rationale` → ok | correctness | 0013 §4.3 | open | | |
| H-5 | Cycle-without-tear yields a hard error (consumed by `check` v2) | `cargo test -p odm-graph cycle_without_tear_errors` → ok | serious | 0013 §4.2/§4.3 | open | | |
| H-6 | All active tears are enumerable (so assumed deps stay visible) | `cargo test -p odm-graph list_active_tears` → ok | correctness | 0013 §4.3 | open | | |
| H-7 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-graph --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-graph/src` | serious | CLAUDE.md | open | | |
| H-8 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-graph --summary-only --ignore-filename-regex '(odm-core|odm-store|odm-cli)/'` → **line** ≥ 90% (target 95%) | correctness | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at `<SHA>` on `<date>`. CDC: `<name>` (cargo rows via CI/local 1.85+).
Total rows: 8. Done: _. Deferred: _. No-op: _.
