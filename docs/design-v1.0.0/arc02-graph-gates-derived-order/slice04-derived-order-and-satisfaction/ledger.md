# Slice 04 (Arc 02): Derived order & satisfaction

> Per LEDGER_DISCIPLINE. Final status + evidence (commit SHA + Verify output)
> before advancing. Compile/test rows reproduced by CDC in CI or a local 1.85+
> toolchain (the Cowork sandbox has none). Five-iteration cap. Rows H-6…H-10 are
> the evidence-leveled-satisfaction work (ODD-0013 §4.4).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | Topological order over `depends_on ∪ consumes` (Kahn); acyclic ⇒ valid order | `cargo test -p odm-graph topo_order` → ok | serious | 0013 §4.1 | open | | |
| H-2 | `next` = deps satisfied ∧ no active `blocked_by` ∧ not complete | `cargo test -p odm-graph next_ready_frontier` → ok | serious | 0013 §4.1, 0001 B2 | open | | |
| H-3 | `blocked X` lists each unsatisfied dependency + why | `cargo test -p odm-graph blocked_reasons` → ok | correctness | 0013 §4.1, 0001 B3 | open | | |
| H-4 | `path X [Y]` returns the dependency chain / critical path | `cargo test -p odm-graph path_chain` → ok | correctness | 0013 §4.1 | open | | |
| H-5 | Satisfaction: edge satisfied iff target reached `satisfied_at` (default terminal) gate | `cargo test -p odm-core satisfaction_gate` → ok | serious | 0013 §4.4 | open | | |
| H-6 | Evidence ordering total + correct: `asserted < attested < reproduced < reconciled` | `cargo test -p odm-core evidence_ordering` → ok | serious | 0013 §4.4 / 0001 D3 | open | | |
| H-7 | **Min-propagation**: a node's effective evidence = min over its transitive dependency path | `cargo test -p odm-graph evidence_min_propagation` (proptest: inserting a lower-evidence link lowers the node's effective level) → ok | serious | 0013 §4.4 | open | | |
| H-8 | Threshold (default `reproduced`) configurable via `odm.toml`; below-threshold ⇒ soft-satisfied | `cargo test -p odm-core satisfaction_threshold` (default + override) → ok | serious | 0013 §4.4 | open | | |
| H-9 | `next` flags soft-satisfied deps; `blocked X` names the low-evidence dep + how to raise it | `cargo test -p odm-graph soft_satisfied_surfacing` (asserts the `⚠ … evidence=attested` flag + the blocked explanation) → ok | serious | 0013 §4.4, 0001 F2/G3 | open | | |
| H-10 | Soft-satisfied does NOT block: `next` still lists the node (visibility, not gating) | `cargo test -p odm-graph soft_satisfied_not_blocking` → ok | correctness | 0013 §4.4 | open | | |
| H-11 | Staleness guard: advancing a node with an unsatisfied `depends_on` warns | `cargo test -p odm-core staleness_guard` → ok | serious | 0013 §4.4, 0001 B3 | open | | |
| H-12 | `--json` for `next`/`blocked`/`path` is stable + carries the evidence level | `cargo test -p odm-cli json_schema_derived_order` (snapshot) → ok | correctness | 0013 §7 | open | | |
| H-13 | Clippy clean (`-D warnings`); no `unsafe`; no panics on public paths | `cargo clippy -p odm-graph -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-graph/src crates/odm-core/src` | serious | CLAUDE.md / rust-guidelines | open | | |
| H-14 | Coverage ≥ 90% (target 95%) for the new graph/satisfaction code | `cargo llvm-cov -p odm-graph -p odm-core --summary-only` → ≥ 90% | correctness | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at commit `<SHA>` on `<date>`. CDC verification: `<name/session>` (compile/
test rows via CI or local 1.85+). Total rows: 14. Done: _. Deferred: _. No-op: _.
