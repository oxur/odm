# Slice 05.1 (Arc 02): Evidence-transition dates

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration
> cap. CDC-authored acceptance rows; CC fills Status/Evidence/Notes per commit.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | `GateRecord` carries an optional per-`Evidence`-level first-reached date map; `reached`/`by`/`evidence` unchanged | `cargo test -p odm-core gate_record_evidence_dates` → ok | serious | telemetry §6 / 0013 §2.3 | todo | | |
| H-2 | `set_gate` records the reached level's date on first reach | `cargo test -p odm-core set_gate_records_level_date` → ok | serious | telemetry §6 | todo | | |
| H-3 | Raising evidence **preserves** earlier levels' dates (no overwrite) — set `attested@D1` then `reproduced@D2` ⇒ `{attested:D1, reproduced:D2}` | `cargo test -p odm-core raise_preserves_prior_level_dates` → ok | serious | telemetry §6 / 0001 D3 | todo | | The point of the slice. |
| H-4 | Re-recording the **same** level keeps its original first-reached date — `attested@D1` then `attested@D3` ⇒ `attested:D1` | `cargo test -p odm-core resetting_same_level_keeps_first_date` → ok | correctness | telemetry §6 | todo | | |
| H-5 | Empty transition history round-trips byte-identically and the field is **omitted when empty** (back-compat with arc01/02 nodes) | `cargo test -p odm-core back_compat_no_evidence_dates_roundtrip` + proptest `parse ∘ emit = identity` → ok | serious | 0013 §2.3 | todo | | |
| H-6 | Existing consumers unaffected — `reached`/`evidence`/terminal-gate/satisfaction semantics unchanged | `cargo test -p odm-core` (all prior slice03/04 tests green) | serious | regression guard | todo | | |
| H-7 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src` | serious | CLAUDE.md | todo | | |
| H-8 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-core --summary-only --ignore-filename-regex '(odm-graph\|odm-store\|odm-cli)/'` → **line** ≥ 90% | correctness | CLAUDE.md | todo | | Run from a clean `target/llvm-cov-target`. |

## What Worked

_(CC fills at close.)_

## Closure

_(CC fills at close: commit, totals, deferrals. CDC verifies cargo rows via CI /
local 1.85+ before slice06 opens.)_
