# CC Prompt — Slice 05.1 (Arc 02): Evidence-transition dates

Make `GateRecord` keep the date each evidence level was *first* reached, so the
verification-latency signal survives an evidence raise. A small, contained,
back-compatible schema extension — the **recording** only; consumption is arc A7.

> **Start condition:** slice03 CDC-closed (`GateRecord`, `Status::set_gate`,
> `Evidence` exist — they do). Independent of slice04/05; lands before slice06.
> Else hold.

## Read first
1. `slice05.1-evidence-transition-dates/ledger.md` (8 rows).
2. `slice-doc.md` (same dir).
3. `crates/odm-core/src/status.rs` — the current `GateRecord` / `Status::set_gate`.
4. `docs/design/01-draft/0013-odm-architecture-design.md` §2.3, §4.4; and
   `workbench/forecasting-telemetry.md` §6 (why this is captured now, not in A7).

## Load skills
- **rust-guidelines** (`11-anti-patterns.md`, `05-type-design.md`, `02-api-design.md`).
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
- Add an **optional** per-evidence-level first-reached date map to `GateRecord`
  (shape your choice — e.g. `BTreeMap<Evidence, NaiveDate>`). Keep `reached`,
  `by`, `evidence` exactly as they are.
- In `Status::set_gate`: on first reach, record the level's date; on a **raise**,
  insert the new level's date and **preserve** earlier levels' dates (do not
  overwrite). Re-recording the same level keeps its original first-reached date.
- Serialize in canonical field order; **omit the field when empty** so arc01/02
  nodes with no transition history round-trip byte-identically.

## Constraints
- **Back-compat is non-negotiable:** a node written before this slice must parse,
  and emitting it must not invent the field. `parse ∘ emit = identity`.
- **No consumer breakage:** `reached` / `evidence` / terminal-gate / satisfaction
  semantics unchanged; existing slice03/04 tests stay green.
- Record **first-reach per level only**; evidence **regression** is out of scope
  (note it, don't model it). No `unsafe`; typed errors; coverage ≥ 90%.

## Deliverables
Green test / clippy / coverage; `ledger.md` evidence per row; `closing-report.md`.
Feature branch (`arc02-slice05.1-evidence-dates`); not `main`.

## Working agreement
Amend, don't work around; flag every deviation rather than burying it;
five-iteration cap; your `done` is *proposed done* → CDC verifies (cargo rows via
CI / local 1.85+).
