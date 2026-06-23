# Slice 03 (Arc 02): Gates, status & evidence recording

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | `Evidence` enum has a total order `asserted < attested < reproduced < reconciled` | `cargo test -p odm-core evidence_total_order` → ok | serious | 0013 §4.4 / 0001 D3 | open | | |
| H-2 | Per-type gate-sets load from `odm.toml` `[gates.<type>]` | `cargo test -p odm-core gate_sets_from_config` → ok | serious | 0013 §5.1 | open | | |
| H-3 | `set-gate` records `{reached, by, evidence}` on the node's status vector | `cargo test -p odm-core set_gate_records_evidence` → ok | serious | 0013 §5.1 / 0001 D3 | open | | |
| H-4 | A gate not in the node type's set is rejected | `cargo test -p odm-core set_gate_rejects_unknown` → ok | correctness | 0013 §5.1 | open | | |
| H-5 | Terminal-gate accessor returns the last gate in a type's sequence | `cargo test -p odm-core terminal_gate` → ok | correctness | 0013 §4.4 | open | | |
| H-6 | Status is a vector (multiple gates independently set), not a scalar | `cargo test -p odm-core status_is_multigate` → ok | serious | 0013 §5.1 / 0001 D1 | open | | |
| H-7 | Default evidence is `asserted` when not specified (least-confident default) | `cargo test -p odm-core evidence_default_asserted` → ok | correctness | 0013 §4.4 | open | | |
| H-8 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src` | serious | CLAUDE.md | open | | |
| H-9 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-core --summary-only --ignore-filename-regex '(odm-graph|odm-store|odm-cli)/'` → **line** ≥ 90% (target 95%) | correctness | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at `<SHA>` on `<date>`. CDC: `<name>` (cargo rows via CI/local 1.85+).
Total rows: 9. Done: _. Deferred: _. No-op: _.
