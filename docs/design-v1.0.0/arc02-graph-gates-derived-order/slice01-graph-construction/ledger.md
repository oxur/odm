# Slice 01 (Arc 02): Graph construction + reverse edges

> Per LEDGER_DISCIPLINE. Final status + evidence before advancing; cargo rows
> reproduced in CI / local 1.85+ (sandbox has none). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | Graph builds from a node set; every node maps to exactly one graph index | `cargo test -p odm-core graph_build` → ok | serious | 0013 §4 | open | | |
| H-2 | Reverse edges derived, not stored: backlinks match forward edges | `cargo test -p odm-graph reverse_edges` (proptest: reverse adjacency == transpose of forward) → ok | serious | 0013 §3 | open | | |
| H-3 | Ordering DAG = `depends_on ∪ consumes` only (excludes part_of/verifies/supersedes/affects) | `cargo test -p odm-core ordering_dag_membership` → ok | serious | 0013 §3 | open | | |
| H-4 | `part_of` exposed as a separate single-parent tree (not in the ordering DAG) | `cargo test -p odm-core part_of_tree` → ok | correctness | 0013 §3 | open | | |
| H-5 | Forward/reverse accessors by edge kind (children/parents) | `cargo test -p odm-graph adjacency_by_kind` → ok | correctness | 0013 §4 | open | | |
| H-6 | `odm-graph` is domain-agnostic: no node-type/gate names in its source | `! grep -REiq 'project\|arc\|slice\|odd\|adr\|gate' crates/odm-graph/src` | correctness | 0013 §8 | open | | |
| H-7 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-graph -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-graph/src` | serious | CLAUDE.md | open | | |
| H-8 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-graph -p odm-core --summary-only` → ≥ 90% | correctness | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at `<SHA>` on `<date>`. CDC: `<name>` (cargo rows via CI/local 1.85+).
Total rows: 8. Done: _. Deferred: _. No-op: _.
