# Slice 07 (Migration Fidelity): Live repair run

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. **This is a
> LIVE, class-(b) slice** — the committed store state *is* the evidence, so most rows reach
> `reproduced`/`reconciled` here (unlike the fixture-only slices). CC runs + commits (`attested`); CDC
> reproduces the store-state invariants by direct read of the committed store; `check`/`orient`/`rollup`
> runs attested → reproduced on re-run. **Snapshot/revert + dry-run-first are HARD gates.** Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **Pre-flight**: s06 doc-comment corrected (CDC v2.5); `1.0.x` green; store worktree **clean**; before-manifest captured | The `self_host_inner` comment no longer claims the order is correctness-load-bearing; `git -C .worktrees/odm status --porcelain` empty; manifest records node count / id set / per-node body-hash / `source` count / schema spread / `context.json` + the **known-good SHA** | serious | CDC v2.5 + slice-doc | open | | Baseline: 60 nodes / 0 source / all v1.0 / 44 stubs / 6 arcs / ctx→`01KWXM…`. Re-measure at run time. |
| F-2 | **Dry-run first, live store untouched by it**: `odm migrate <plan> --dry-run` previewed + adjudicated **before** firing | Preview shows reconciled/created/skipped counts, **0 drift errors**; counts reconciled against the before-manifest (≈47 plan nodes reconciled, 6 arcs + slices created); `git status` still clean after the dry-run | serious | slice-doc (HARD) | open | | Expect: 44 stubs repaired + faithful backfilled, project + retired excluded. A surprise here stops the run before any mutation. |
| F-3 | **Live run fired as one revertible commit** | `odm migrate <plan>` run for real; store committed as a **single** commit atop the captured SHA; `git reset --hard <SHA>` documented as the undo | serious | slice-doc | open | | The whole mutation is one revert away. Commit message names slice + before/after deltas. |
| F-4 | **Zero stub bodies remain** | Every plan node (arc/slice/project) has `is_stub_body == false`; the 44 stubs (6 arc + 38 slice) now carry verbatim source bodies | serious | MF-2 | open | | The core repair outcome. |
| F-5 | **Every arc/slice node source-bearing; project + retired excluded** | Every arc/slice node carries `source.paths`; the **project** node carries **no** 1:1 `source` (ODD-0025 §2.3); the **retired/tombstone** node (`design-notes.md` §1) excluded | serious | ODD-0025 §2.2/§2.3 / MF-3 | open | | The exact exclusions s06 built + CDC-verified, now on the real corpus. |
| F-6 | **Full representation** — all plan arcs + slices present | Coverage set-difference over `source.paths` = **0 uncovered arcs/slices** (`odm migrate --coverage`); all **12** plan-tree arcs represented; the 6 previously-missing arcs + their slices imported | serious | MF-5 / ODD-0025 §5 | open | | Arc/slice scope only — design/research/loose-doc coverage is s08 (F-11). |
| F-7 | **Every body-hash passes; schema `v1.1`** | A second `reconcile`/`migrate` surfaces **0** `BodyHashMismatch`; migrated nodes stamped `schema: */v1.1` | serious | ODD-0025 §2.1 | open | | A live faithful-node gate failure is a finding (F-11), not a suppress. |
| F-8 | **`odm check` green; `orient`/`rollup` reproduce** | `odm check` exit 0 (validate + reconcile); `orient` + `rollup` regenerate **byte-stable** on re-run | serious | slice-doc | open | | Mixed v1.1 (plan) / v1.0 (design/research) store is valid — `v1.0 < CURRENT` is not a schema error. |
| F-9 | **`context.json` re-pointed** correctly | Post-run `context.json` reflects the intended operator focus (preserved or deliberately advanced), not incidentally clobbered | serious | slice-doc | open | | Baseline ctx → arc `01KWXM…`. State the intended post-run value + why. |
| F-10 | **Re-run idempotent** | A second `odm migrate <plan>` → 0 reconciled / 0 created; node count + id set unchanged | serious | s05/s06 | open | | Confirms the live flow is safely re-runnable — the property s07's safety leans on. |
| F-11 | **Rollback & findings discipline; no silent scope** | Any gate failure → **revert to SHA + file finding**, not forced; any live gate failure adjudicated (drift finding), not suppressed; design/research `source` + loose-doc coverage (14 nodes + ~211 docs) explicitly **deferred to s08**, disclosed not dropped | correctness | LEDGER-DISCIPLINE / operator "let it crash then recover" | open | | The spine of the first live mutation. Hidden failure is the only unacceptable outcome. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<impl-SHA>` (`release/1.0.x`) + store commit `<store-SHA>`
(`odm` branch) on `<date>`. Verified by: `<CDC/session>`.
Rows: 11. Done: _n_. Deferred: _n_. No-op: _n_. On close, bubble up to `../arc-plan.md`:
**MF-2/MF-3/MF-5 → done** (live-corpus outcome reproduced, §B); residual design/research-`source` +
loose-doc coverage handed to **s08**.
