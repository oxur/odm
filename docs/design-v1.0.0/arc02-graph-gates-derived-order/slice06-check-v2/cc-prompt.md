# CC Prompt — Slice 06 (Arc 02): `check` v2

Make `odm check` the single mechanical gate that aggregates every graph-level
invariant — the command the framework's prose rules collapse into. Closes Arc 02.

> **Start condition:** arc02 slices 01–05 + arc01 slice06 (`check` v1) CDC-closed.
> Else hold.

## Read first
1. `slice06-check-v2/ledger.md` (10 rows).
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §7, §4.3, §4.4, §4.5.

## Load skills
- **rust-guidelines** (`11-anti-patterns.md`, `14-cli-tools/03-error-handling.md`
  for exit codes, `14-cli-tools/02-argument-parsing.md`).
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
Aggregate over the whole graph: schema + link-integrity (v1), cycles-without-tears,
out-of-order/staleness, recomposition (orphan/stub/decomposition drift), and
below-threshold (soft-satisfied) dependencies. Add:
- Exit codes `0`/`1`/`2`; a `--strict`/CI mode that promotes warnings to failures.
- **Errors-as-affordances**: every finding names the exact command to resolve it.
- `--json` report with a stable schema.

## Constraints
- Consume the predicates from slices 02/04/05 — don't reimplement them.
- `reconcile`/desired-fact drift is NOT in scope (Arc A5 extends `check` later).
- No `unsafe`; typed errors; coverage ≥ 90%; `assert_cmd` for exit-code tests.

## Deliverables
Green test/clippy/coverage; `ledger.md` evidence per row; `closing-report.md`.
Feature branch; not `main`.

## Working agreement
Amend don't work around; five-iteration cap; proposed-done → CDC via CI/local 1.85+.
On close, **Arc 02 is complete**.
