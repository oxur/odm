# Slice 06 (Arc 02): `check` v2

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | `check` aggregates schema + link-integrity (v1) on the whole graph | `cargo test -p odm-cli check_schema_and_links` → ok | serious | arc01 slice06 | open | | |
| H-2 | `check` fails on a cycle-without-tear; passes once torn | `cargo test -p odm-cli check_cycle_requires_tear` → ok | serious | 0013 §4.3 | open | | |
| H-3 | `check` reports out-of-order / staleness | `cargo test -p odm-cli check_staleness` → ok | serious | 0013 §4.4 / 0001 B3 | open | | |
| H-4 | `check` reports recomposition violations (orphan/stub/decomposition drift) | `cargo test -p odm-cli check_recomposition` → ok | serious | 0013 §4.5 | open | | |
| H-5 | `check` reports below-threshold (soft-satisfied) dependencies | `cargo test -p odm-cli check_soft_satisfied` → ok | serious | 0013 §4.4 / 0001 F2 | open | | |
| H-6 | Exit codes: `0` clean, `1` violations, `2` usage error | `cargo test -p odm-cli check_exit_codes` → ok | serious | 0013 §7 | open | | |
| H-7 | `--strict`/CI mode promotes warnings (staleness, soft-satisfaction) to failures | `cargo test -p odm-cli check_strict_mode` → ok | correctness | 0013 §4.4 | open | | |
| H-8 | Every finding names the exact fix command (errors-as-affordances) | `cargo test -p odm-cli check_errors_name_fix` → ok | serious | 0013 §7 / 0001 | open | | |
| H-9 | `--json` report with a stable, documented schema | `cargo test -p odm-cli check_json_schema` (snapshot) → ok | correctness | 0013 §7 | open | | |
| H-10 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% | `cargo clippy -p odm-cli -p odm-core --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-cli -p odm-core --summary-only --ignore-filename-regex '(odm-store|odm-graph)/'` → **line** ≥ 90% (target 95%) | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at `<SHA>` on `<date>`. CDC: `<name>` (cargo rows via CI/local 1.85+).
Total rows: 10. Done: _. Deferred: _. No-op: _. **Arc 02 complete on close.**
