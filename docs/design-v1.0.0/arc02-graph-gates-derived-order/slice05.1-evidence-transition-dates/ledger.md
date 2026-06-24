# Slice 05.1 (Arc 02): Evidence-transition dates

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration
> cap. CDC-authored acceptance rows; CC fills Status/Evidence/Notes per commit.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | `GateRecord` carries an optional per-`Evidence`-level first-reached date map; `reached`/`by`/`evidence` unchanged | `cargo test -p odm-core gate_record_evidence_dates` → ok | serious | telemetry §6 / 0013 §2.3 | done | `5c238a8`: `gate_record_evidence_dates` → 1 passed. `evidence_dates: BTreeMap<Evidence, NaiveDate>` added; reached/by/evidence assert identical to before. | |
| H-2 | `set_gate` records the reached level's date on first reach | `cargo test -p odm-core set_gate_records_level_date` → ok | serious | telemetry §6 | done | `5c238a8`: `set_gate_records_level_date` → 1 passed. First reach → `{level: reached}` via `entry().or_insert()`. | |
| H-3 | Raising evidence **preserves** earlier levels' dates (no overwrite) — set `attested@D1` then `reproduced@D2` ⇒ `{attested:D1, reproduced:D2}` | `cargo test -p odm-core raise_preserves_prior_level_dates` → ok | serious | telemetry §6 / 0001 D3 | done | `5c238a8`: `raise_preserves_prior_level_dates` → 1 passed. Both levels' dates survive; `evidence`/`reached` follow the raise. | The point of the slice. |
| H-4 | Re-recording the **same** level keeps its original first-reached date — `attested@D1` then `attested@D3` ⇒ `attested:D1` | `cargo test -p odm-core resetting_same_level_keeps_first_date` → ok | correctness | telemetry §6 | done | `5c238a8`: `resetting_same_level_keeps_first_date` → 1 passed. `or_insert` keeps D1; `reached`/`by` follow the latest call. | |
| H-5 | Empty transition history round-trips byte-identically and the field is **omitted when empty** (back-compat with arc01/02 nodes) | `cargo test -p odm-core back_compat_no_evidence_dates_roundtrip` + proptest `parse ∘ emit = identity` → ok | serious | 0013 §2.3 | done | `5c238a8`: `back_compat_no_evidence_dates_roundtrip` → 1 (pre-field YAML re-emits byte-identical, no `evidence_dates` key) + `evidence_dates_roundtrip_identity` (128-case proptest, populated field round-trips). | |
| H-6 | Existing consumers unaffected — `reached`/`evidence`/terminal-gate/satisfaction semantics unchanged | `cargo test -p odm-core` (all prior slice03/04 tests green) | serious | regression guard | done | `5c238a8`: `cargo test --workspace` → all green (status/satisfaction/graph/check/recompose/frontmatter unregressed). | |
| H-7 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src` | serious | CLAUDE.md | done | `5c238a8`: clippy → exit 0; unsafe grep → no match; `cargo fmt --check` clean. | |
| H-8 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-core --summary-only --ignore-filename-regex '(odm-graph\|odm-store\|odm-cli)/'` → **line** ≥ 90% | correctness | CLAUDE.md | done | `5c238a8`: TOTAL **line 97.88%** / region 97.28% (status.rs line 94.23%); from a clean `target/llvm-cov-target`. | Run from a clean `target/llvm-cov-target`. |

## What Worked

- **`entry().or_insert()` is exactly the first-reach semantics.** One line —
  `record.evidence_dates.entry(evidence).or_insert(reached)` — gives "record on
  first reach, never overwrite," covering H-2/H-3/H-4 with no branching. The
  current-reach fields (`reached`/`by`/`evidence`) are then overwritten as
  before, so the raise behavior is unchanged.
- **`skip_serializing_if = "BTreeMap::is_empty"` buys back-compat for free.** A
  node written before this field has no `evidence_dates` key; `#[serde(default)]`
  parses it to an empty map, and skip-when-empty omits it on emit — so the bytes
  are identical (H-5 proves it against a literal pre-field YAML).
- **`BTreeMap<Evidence, _>` keys serialize in evidence order**, deterministic and
  human-meaningful (asserted → reconciled), with `Evidence`'s existing lowercase
  string Serialize used as the map key — no new wire vocabulary.
- **Pure additive change.** No public signature changed; the one `GateRecord`
  literal construction site (in `set_gate`) is the only code touched besides the
  field. Every slice03/04/05 consumer compiled and passed untouched (H-6).

## Closure

Closed at `5c238a8` on `2026-06-24`. CDC: pending (cargo rows reproduced by CDC
in CI or a local 1.85+ toolchain). All `done` states are *proposed done* pending
that independent verification. Total rows: 8. Done: 8. Deferred: 0. No-op: 0.

**Flagged for CDC.** (1) Evidence **regression** is recorded as a first-reach,
not flagged: if a caller records a *lower* level after a higher one (e.g.
`reproduced` then `attested`), `evidence_dates` gains the lower level's date and
`evidence` follows it down — the slice models first-reach per level only and does
**not** detect or prevent a downgrade (explicitly out of scope; A7 owns
regression semantics). (2) `reached` still **overwrites** on a raise (unchanged
slice03 behavior); the durable per-level history now lives in `evidence_dates`,
so `reached` is "current reach," not "first reach of the gate." (3) The signal is
recorded but **not consumed** — latency math and rollup surfacing are arc A7.
