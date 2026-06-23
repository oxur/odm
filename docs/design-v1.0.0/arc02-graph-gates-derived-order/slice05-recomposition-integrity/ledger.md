# Slice 05 (Arc 02): Decomposition/recomposition integrity

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | Reverse-`part_of` enumerates a parent's complete child set | `cargo test -p odm-core recompose_children` → ok | serious | 0013 §4.5 | open | | |
| H-2 | Every non-root node resolves to exactly one parent (total, unambiguous) | `cargo test -p odm-core single_parent_total` → ok | serious | 0013 §4.5 | open | | |
| H-3 | Orphan detection: a non-root node with no resolvable parent is flagged | `cargo test -p odm-core detect_orphan` → ok | serious | 0013 §4.5 | open | | |
| H-4 | No-stub: a `project`/`arc` advanced into a working/complete gate with zero children is flagged | `cargo test -p odm-core detect_undeveloped_stub` → ok | correctness | 0013 §4.5 | open | | |
| H-5 | `decomposed: complete` assertion recorded on a parent | `cargo test -p odm-core decomposed_assertion` → ok | correctness | 0013 §4.5 | open | | |
| H-6 | Guard: children added/removed after `decomposed: complete` flags for re-affirmation | `cargo test -p odm-core decomposed_drift_guard` → ok | serious | 0013 §4.5 | open | | |
| H-7 | Guard: a parent advanced toward done WITHOUT `decomposed: complete` is flagged | `cargo test -p odm-core advance_without_decomposition` → ok | correctness | 0013 §4.5 | open | | |
| H-8 | Semantic missing-scope detection is NOT attempted (documented non-goal; no false "missing scope" claims) | `cargo test -p odm-core no_semantic_scope_guessing` (asserts the API only reports structural facts) → ok | correctness | 0013 §4.5 | open | | |
| H-9 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src` | serious | CLAUDE.md | open | | |
| H-10 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-core --summary-only --ignore-filename-regex '(odm-graph|odm-store|odm-cli)/'` → **line** ≥ 90% (target 95%) | correctness | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at `<SHA>` on `<date>`. CDC: `<name>` (cargo rows via CI/local 1.85+).
Total rows: 10. Done: _. Deferred: _. No-op: _.
