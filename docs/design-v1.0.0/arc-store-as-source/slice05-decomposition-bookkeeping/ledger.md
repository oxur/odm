# Slice 05 ledger — decomposition bookkeeping: consistency + auto-recompose

Per `LEDGER-DISCIPLINE.md` §A. Code slice: rows are grep/test-verifiable. CC fills
Evidence at the commit each is met (strength `attested`); CDC reproduces.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **One** shared "decomposition children" definition (work-typed reverse-`part_of`) exists in `odm-core` and is called by **both** `check_decomposition` and `node decomposed` | grep: both call the shared helper; unit test asserts identical sets for a mixed-child-type parent | serious | slice-doc (a) | open | | the divergence root |
| F-2 | MF `#58837400` reports **0** decomposition-drift after `odm node decomposed 58837400` | real-store (or fixture) `check`: no `DecompositionDrift` for MF | serious | 2026-08-02 bug | open | | acceptance anchor |
| F-3 | A parent with a non-work (artifact/note) child affirms cleanly — the artifact is neither required in the affirmation nor flagged as drift | fixture unit test | correctness | slice-doc (a) | open | | |
| F-4 | `migrate` auto-recomposes an affirmed parent whose children were **re-minted to new ids but are the same logical set** — no manual re-affirm, no residual drift | re-mint fixture; post-`migrate` `check` clean; no manual step | serious | slice-doc (b) | open | | Duncan's "same children as before" guard |
| F-5 | **The seam holds:** a genuinely-new work child under an affirmed parent **still** produces a `DecompositionDrift` finding (migrate does NOT auto-bless completeness) | add-child fixture; drift finding present after `migrate` | serious | slice-doc (out) | open | | protects spec-keeping |
| F-6 | No regression: `make check` / the test suite is green; existing decomposition/recompose tests pass | `make check` exit 0 | correctness | standing | open | | |

## What Worked

_(At slice close.)_

## Closure

Closed at commit <SHA> on <date>. Verified by: <name/session>.
Rows: 6. Done: _. Deferred: _. No-op: _.
