# Slice 13 (Migration Fidelity): Live reconcile + vision mint

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. **This slice
> mutates the live `odm` store** (reconcile all drift + the vision mint) as **one revertible commit** after
> a clean, adjudicated dry-run. **Snapshot/revert + dry-run-first are HARD gates.** Live rows are
> class-(b) — the committed store is the evidence; CDC reproduces by direct read. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **Pre-flight**: `release/1.0.x` green (s12 in); `odm` store clean; **known-good SHA `2fc25f5` + before-manifest captured** | `make check` green; `git -C .worktrees/odm status --porcelain` empty; record SHA + sha256 composite over all node files | serious (revert anchor) | s07 protocol | open | | The revert target if any gate fails. |
| F-2 | **Dry-run adjudicated**: every change is a body re-snapshot to current source, or a moved-path rewrite (0017/0018 → `04-accepted/`); the vision mint = 1 create (the 1:1 `project-plan` node) + the `#1000` re-cast; **0 id/schema/edge drift, 0 unexpected create, no re-mint**; store fingerprint stable after dry-run | `odm <reconcile/migrate> --dry-run` → preview; diff the reconcile set against the live drift scan + the 2 moved; re-hash store = unchanged | serious (the fire gate) | s07/s10 adjudicate | open | | Reconciles are **expected** — the gate is *adjudication*, not "0 changes". The count is the dry-run's, not fixed. |
| F-3 | **Reconcile every drifted node**: each drifted non-stub re-snapshotted to current source (body matches after), `id`/`edges`/`status` preserved | direct read of the committed store: recompute the body-hash gate against current sources → **0 drifted** remain | serious (MF-9 fidelity) | ODD-0025 §2.9 | open | | ~17 body-drifted at scan time; whatever is drifted at fire time. |
| F-4 | **Moved-source re-discovery live**: ODD-0017/0018 re-found by identity; `source.paths` rewritten to `04-accepted/`-relative and resolving; body re-snapshot | direct read: 0017/0018 nodes' `source.paths` point at `04-accepted/…` (relative, canonical), resolve, body matches | serious | s11 L-8b / s12 F-2 | open | | |
| F-5 | **Vision minted**: a faithful **1:1 `project-plan` node** (hard-gated) + `#1000` re-cast as an **editorial-merge synthesis superseding it**, recorded attestation, bidirectional lineage clean | direct read: `#1000` `source.synthesis: editorial-merge`, attestation present, `supersedes` → the 1:1 node; `check`'s lineage rule clean | serious (MF-7 live) | s12 `apply_project_vision` | open | | The 1 legitimate create in the dry-run. |
| F-6 | **No collateral; idempotent**: reconciled nodes changed body (+ moved `source.paths`) **only** (ids/schema/edges/status intact); already-faithful nodes + retired untouched; re-run 0/0; `orient`/`rollup` byte-stable | direct read: diff shows only body/path lines on reconciled nodes; 2× `orient`/`rollup` identical; second run 0 reconciled / 0 created | serious | slice-doc | open | | |
| F-7 | **`check` green + guard intact**: `odm check` exit 0 (coverage still 371/… enforcing); the s10-it1 `absolute-source-path` rule green (all rewritten paths relative) | `odm check` exit 0; 0 `absolute-source-path` findings; 0 uncovered | serious | s09/s10-it1 | open | | |
| F-8 | **One revertible commit; rollback discipline; no downstream/new-capability pulled forward** | single `odm`-branch commit atop `2fc25f5`; `git reset --hard 2fc25f5` documented; no arc-close/P-12 run; no L-8a; only thin live-invocation wiring, no s12-scope capability | serious | LEDGER-DISCIPLINE / operator | open | | A capability gap here is an **s12** fix. |
| F-9 | **No model drift**: 1:1 rule + migration-time-only gate unchanged; reconcile only sets a body from its current source; ODD amend-not-work-around if any line needed | cross-read: gate still migration-time-only; identity preserved on every reconcile | correctness | ODD-0025 | open | | |
| F-10 | **Clippy clean; no `unsafe`; coverage ≥ 90% on any code touched** | clippy exit 0; `! grep unsafe`; `llvm-cov` ≥ 90% on changed (mostly a run + thin wiring) | polish/correctness | CLAUDE.md | open | | |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at store commit `<SHA>` (`odm` branch, atop known-good `2fc25f5`) on `<date>`.
Verified by: `<CC then CDC>`. Rows: 10. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`. Undo:
`git reset --hard 2fc25f5`. On close, bubble up to `../arc-plan.md`: MF-7 done (vision live), MF-9 fidelity
true on the live corpus; **the arc-close is next** (MF-9 composition + P-12 demo + final reconcile-and-freeze).
