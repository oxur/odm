# Slice 06 (Arc 01): `check` v1 + link-integrity

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| L-1 | `check` flags a node missing a required field for its type | `cargo test -p odm-cli check_missing_field` (assert_cmd) → ok | serious | 0013 §2.3 | open | | |
| L-2 | Link-integrity: a dangling `part_of` reference is flagged | `cargo test -p odm-cli check_dangling_part_of` → ok | serious | 0011 R6 / 0001 A3 | open | | |
| L-3 | Link-integrity: a dangling `supersedes`/edge reference is flagged | `cargo test -p odm-cli check_dangling_edge` → ok | serious | 0011 R6 | open | | |
| L-4 | Supersession-chain integrity: a self-supersede or a cyclic chain is flagged | `cargo test -p odm-cli check_supersession_chain` → ok | serious | 0013 §3 | open | | |
| L-5 | A clean corpus passes with exit `0` | `cargo test -p odm-cli check_clean_passes` → ok | serious | 0013 §7 | open | | |
| L-6 | Exit codes: `0` clean, `1` violations, `2` usage error | `cargo test -p odm-cli check_exit_codes_v1` → ok | serious | 0013 §7 | open | | |
| L-7 | Every finding names the exact fix command (errors-as-affordances) | `cargo test -p odm-cli check_errors_name_fix_v1` → ok | serious | 0013 §7 / 0001 | open | | |
| L-8 | `--json` report with a stable, documented schema | `cargo test -p odm-cli check_json_v1` (snapshot) → ok | correctness | 0013 §7 | open | | |
| L-9 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% | `cargo clippy -p odm-cli -p odm-core --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-cli -p odm-core --summary-only --ignore-filename-regex '(odm-store|odm-graph)/'` → **line** ≥ 90% (target 95%) | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at `<SHA>` on `<date>`. CDC: `<name>` (cargo rows via CI/local 1.85+).
Total rows: 9. Done: _. Deferred: _. No-op: _. **Arc 01 complete on close.**
