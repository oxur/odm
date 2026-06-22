# CC Prompt — Slice 05 (Arc 02): Decomposition/recomposition integrity

Make decomposition structural and checkable: total recomposition, orphan/stub
detection, and a drift-guarded `decomposed: complete` assertion. **Do NOT attempt
automatic semantic missing-scope detection** — that's a human judgement.

> **Start condition:** arc02 slice01 (graph + `part_of` tree) CDC-closed. Else hold.

## Read first
1. `slice05-recomposition-integrity/ledger.md` (10 rows).
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §4.5 (spec); ODD-0001 E4.

## Load skills
- **rust-guidelines** (`11-anti-patterns.md`, `05-type-design.md`).
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
- Reverse-`part_of`: enumerate a parent's complete child set; every non-root node
  resolves to exactly one parent (total, unambiguous).
- Orphan detection; no-stub detection (project/arc advanced with zero children).
- `decomposed: complete` assertion + guard: children changed after assertion, or a
  parent advanced toward done without it → flag for re-affirmation.

## Constraints
- **Structural only.** Report facts the graph proves; never *guess* missing/excess
  scope (H-8). No `unsafe`; typed errors; coverage ≥ 90%.

## Deliverables
Green test/clippy/coverage; `ledger.md` evidence per row; `closing-report.md`.
Feature branch; not `main`.

## Working agreement
Amend don't work around; five-iteration cap; proposed-done → CDC via CI/local 1.85+.
