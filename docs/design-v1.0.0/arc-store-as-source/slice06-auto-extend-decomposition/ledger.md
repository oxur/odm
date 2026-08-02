# Slice 06 ledger — auto-extend affirmed decomposition on authored additions

Per `LEDGER-DISCIPLINE.md` §A. Code slice. CC fills Evidence at the commit each is
met (`attested`); CDC reproduces. The real-MF row (F-2) is the acceptance anchor.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | An already-affirmed parent that gains a plan-tree-declared work-child has its `decomposed` **auto-extended** by `migrate --all` — includes the new child, `check` clean, **no manual `node decomposed`** | fixture: affirm N slices, author slice N+1, `migrate --all`, assert affirmation ⊇ {N+1} and 0 drift | serious | SS5-1 | open | | the core ask |
| F-2 | **The MF transitional shape auto-heals** (affirmation carrying stale non-work children + a genuine new slice → rewritten to the current work-child set, 0 drift, no manual step) | fixture reproducing MF's shape; and the real store: reset → `migrate --all` → MF 0 drift, no affirm | serious | SS5-1 | open | | acceptance anchor |
| F-3 | A parent that **lost** a work-child still raises `DecompositionDrift` (not auto-healed) | fixture: remove/retire a work-child, `migrate --all`, assert drift flagged | correctness | scope-out | open | | removals stay surfaced |
| F-4 | A **never-affirmed** parent is **not** auto-affirmed — it stays `undecomposed-parent` | fixture: undecomposed parent with children, `migrate --all`, assert still undecomposed | serious | scope-out | open | | first affirm is a human act |
| F-5 | **Idempotent** — a second `migrate --all` after an auto-extend extends nothing | fixture: run twice, assert 0 auto-extends on the second | correctness | standing | open | | |
| F-6 | The `RECOMPOSE` report/output distinguishes **auto-extended** from **left as drift** (new outcome surfaced) | CLI output shows the auto-extend line; `--json` carries the outcome | polish | scope-in | open | | |
| F-7 | No regression: slice 05's `ReAffirmed` (identity churn) and the undeclared-addition seam still hold; `make check` green | `make check` exit 0; slice 05 tests pass | correctness | standing | open | | refines, doesn't remove, the seam |

## What Worked

_(At slice close.)_

## Closure

Closed at commit <SHA> on <date>. Verified by: <name/session>.
Rows: 7. Done: _. Deferred: _. No-op: _.
