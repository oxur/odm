# CC Prompt — Slice 03 (Arc 02): Gates, status & evidence recording

Make status a configurable, multi-gate, evidence-tagged vector, plus the `set-gate`
operation that advances it. This is the *recording* half; slice04 *consumes* it.

> **Start condition:** arc01 CDC-closed (the node + status schema field exist). Else hold.

## Read first
1. `slice03-gates-status-evidence/ledger.md` (9 rows).
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §5.1, §4.4, §2.3.

## Load skills
- **rust-guidelines** (`11-anti-patterns.md`, `05-type-design.md`, `02-api-design.md`).
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
- `Evidence` enum with a **total order** `asserted < attested < reproduced <
  reconciled` (canonical definition — slice04 consumes it).
- Per-node-type gate-sets from `odm.toml` (`[gates.<type>] sequence = [...]`).
- `set-gate <node> <gate> --by --evidence`: records `{reached, by, evidence}` on
  the node's status vector; rejects a gate not in the type's set; default evidence
  `asserted`.
- Terminal-gate accessor (default satisfaction target in slice04).

## Constraints
- Status is a **vector**, never a scalar (0001 D1). Evidence default is the
  *least-confident* level (`asserted`). No `unsafe`; typed errors; coverage ≥ 90%.
- Status *serialization* already exists in the arc01 schema — operate on it, don't
  redefine it.

## Deliverables
Green test/clippy/coverage; `ledger.md` evidence per row; `closing-report.md`.
Feature branch; not `main`.

## Working agreement
Amend don't work around; five-iteration cap; proposed-done → CDC via CI/local 1.85+.
