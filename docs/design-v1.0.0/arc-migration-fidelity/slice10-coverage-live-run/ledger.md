# Slice 10 (Migration Fidelity): Coverage live run

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced` at its scale.
> **This slice mutates the live `odm` store** (a large mint + a gate flip) as **one revertible commit**
> after a clean, adjudicated dry-run. **Snapshot/revert + dry-run-first are HARD gates.** The live rows
> are class-(b) — the committed store is the evidence; CDC reproduces by direct read. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **Pre-flight**: `release/1.0.x` green (s09 merged); `odm` store worktree clean; **known-good SHA + before-manifest captured** | `make check` green; `git -C .worktrees/odm status --porcelain` empty; record SHA (`7226797`) + sha256 composite over all node files | serious (revert anchor) | s07 protocol | open | | The revert target if any gate fails. |
| F-2 | **Dry-run adjudicated**: creates = s01's supporting-doc inventory (count + identity); modifies = exactly the 14 design/research nodes; **0 unexpected change** to the 62 existing source-bearing nodes / project / retired; store fingerprint stable after dry-run | `odm migrate --dry-run` → preview counts; diff the create/modify sets against s01 `coverage-report.md` + the 14-node list; re-hash store = unchanged | serious (the fire gate) | s07/s08 adjudicate | open | | Unlike s08, **creates are expected** — the gate is *adjudication*, not "0 creates". Any set mismatch → stop + finding. |
| F-3 | **Artifact mint-all (§2.6)**: every supporting `.md` under the scan root gets a faithful 1:1 `artifact` node — `ledger`/`cc-prompt`/`cdc-verification`/`closing-report`/ADR/amendment/UAT **+ every report incl. `coverage-report.md`**; no exemption | direct read of the committed store: one `artifact` node per supporting doc; count matches s01 inventory; `coverage-report.md` has a node | serious | ODD-0025 §2.6 | open | | Mint-all, no ignore rule (§2.6). |
| F-4 | **Mint fidelity**: every minted `artifact` body byte-matches its source under `normalize` (trim+lf); §2.1 hard gate held on every mint | direct read: recompute the body-hash gate on the minted nodes (resolve relative `source.paths` → compare) | serious | ODD-0025 §2.1 | open | | 0 `BodyHashMismatch` at mint. |
| F-5 | **Containment (§2.5)**: each `artifact` is `part_of` its nearest modeled scale — per-slice → its slice, arc-/chunk-level → its arc | direct read: sample per-slice + arc-/chunk-level artifacts → correct `part_of` edge | serious | ODD-0025 §2.5 | open | | No `chunk`/`step` scale. |
| F-6 | **Design/research `source` backfilled (F7)**: the 14 `design`/`research` nodes now carry `source`; none flagged orphan (optional containment §2.7) | direct read: the 14 nodes source-bearing; `check`/orphan does not flag a top-level doc node | serious | arc-plan MF-3 / F7 | open | | Was blocked on `discover()` reach — s09 opened it. |
| F-7 | **Coverage enforced + green**: doc-coverage gate **on** in the live `.worktrees/odm/config.toml`; `odm check` **exit 0** with 0 uncovered `.md`; a seeded uncovered doc would fail | `odm check` exit 0 (coverage enforcing); `odm migrate --coverage` → 0 uncovered; (fixture/negative: seeded uncovered → Error, from s09) | serious (MF-1/MF-6) | ODD-0025 §5 / arc MF-6 | open | | Green **because** covered, not because the gate is off — the whole point. No red window (mint + flip atomic). |
| F-8 | **No collateral + idempotent**: the 62 pre-existing source-bearing nodes, project (`#1000`), retired (`#1605`) unchanged; ids/bodies/schema intact; `orient`/`rollup` byte-stable; re-run idempotent (0 created / 0 reconciled) + cross-root stable | direct read: last-touch commits of project/retired unchanged; existing-node diff = none; 2× `orient`/`rollup` identical; second `migrate` run 0/0 | serious | slice-doc | open | | The s07/s08 no-collateral guarantee, at mint scale. |
| F-9 | **One revertible commit; rollback discipline; no downstream pulled forward** | single `odm`-branch commit atop `7226797`; `git reset --hard 7226797` documented; no synthesis (s11) / reconcile (s12) work; no capability code beyond the live gate wiring | serious | LEDGER-DISCIPLINE / operator | open | | A capability gap here is an **s09** fix, not new s10 scope. |
| F-10 | **No model drift; clippy/unsafe clean on any code touched** | cross-read: `source` still the identity axis, body-hash gate unchanged, `artifact` per §2.5; clippy exit 0 + no `unsafe` on any changed file; ODD amended-not-worked-around if a line was needed | polish/correctness | CLAUDE.md / ODD-0025 | open | | s10 is mostly a live run; minimal/no new code beyond gate wiring. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at store commit `<SHA>` (`odm` branch, atop known-good `7226797`) on `<date>`.
Verified by: `<CC then CDC>`. Rows: 10. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
Undo: `git reset --hard 7226797`. On close, bubble up to `../arc-plan.md`: MF-1/MF-6 done (enforced
live); MF-3 done (full criterion); **s11 (synthesis + L-8b) next**; living-doc-drift reconcile → s12.
