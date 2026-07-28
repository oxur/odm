---
id: 01KWXMBBTKT5WD5R931YMT2FNE
number: 1508
type: slice
schema: slice/v1.1
name: freshness on every command + honest staleness (the arc capstone)
created: 2026-07-06
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc05-reconciliation/slice08-freshness-wiring/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KWXMBBTKQ3QE7FGTM80MYNDH
status:
  built:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  planned:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  tested:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
---
# Slice 08 (Arc 05): freshness on every command + honest staleness (the arc capstone)

> Plan-of-record for A5 slice08 — the **last slice of the arc**. slice07 built the freshness
> *mechanism* (probe classes + racy input fingerprint + the `.odm/` drift snapshot +
> incremental reconcile). This slice **wires it into every command**, renders **honest
> staleness**, and settles the two loose ends the arc accumulated. After it: the A5 arc-close.

## Goal

Make drift **fresh on every command, for free** — bare `odm` (orient) runs the cheap
incremental pass (re-probing only changed inputs), volatile facts show "last checked Xm ago"
and are never auto-spawned, and `odm reconcile` is the explicit "refresh everything" verb.
This is ODD-0019 realized end-to-end, and it **dissolves the slice04 orient-runs-probes
regression**.

## Design of record: ODD-0019 §3.2 (incremental), §3.4 (honest staleness), §8 (the wiring)

CC's slice07 closing-report handed this slice a concrete plan; this doc formalizes it.

## Scope — in

1. **Wire `reconcile_views` → the incremental path** (slice07). Every command that reads
   drift/deferred (orient, rollup) now runs the **incremental** reconcile (fresh input-derived
   at near-zero cost; volatile carried from the snapshot with staleness) — **not** the full
   `run_corpus`. **This dissolves the slice04 regression:** bare `odm`/orient runs **zero
   volatile probes**.
2. **`odm reconcile` → the full path.** The explicit command runs `reconcile_full`: re-probes
   **all** input-derived **and volatile** facts, refreshing + stamping the snapshot. This is
   the sanctioned "refresh the volatile facts now" verb (ODD-0019's explicit-refresh).
3. **Honest-staleness rendering.** rollup/orient render volatile facts as **"last checked Xm
   ago"** (from the snapshot's `last_checked`), input-derived as fresh; `--json` gains the
   staleness data **additively** (no schema bump). A never-checked volatile fact reads
   "not yet checked — run `odm reconcile`" (no fabricated freshness).
4. **`.odm/drift` default path, gitignored.** The snapshot lives at a default `.odm/` path
   (like the index), and `.odm/` is gitignored (never truth, never committed).

## Settle the two loose ends (the arc's accumulated open questions)

5. **Deferred-`next` decision (slice06 limitation) — SETTLED: `next` stays graph-pure.**
   `next` answers "what is *graph*-ready?" (deps + gates); **deferred is a reconcile-layer
   status**, surfaced in the reconcile-aware views (rollup/orient), **not** in `next`.
   Withholding deferred from `next` would require either indexing the marker (the rejected
   invariant + `FORMAT_VERSION` path) or making `next` run a reconcile (re-introducing the
   very orient regression we're dissolving). So a deferred-but-dep-ready node **correctly**
   appears in `next` (graph-ready) *and* in rollup/orient's Deferred section (parked) — that
   is a **layering boundary, not an inconsistency**. Documented; the slice06 "limitation" is
   closed as **decided-not-withheld**. (Revisit only if a future need justifies indexing one
   `deferred` boolean.)
6. **`ROLLUP.md` early-cutoff (slice04 finding #2) — SETTLED: the cutoff is drift-aware.**
   A4's early-cutoff skips regenerating `ROLLUP.md` when the corpus *meta-fingerprint* is
   unchanged — but **drift can change without a corpus change** (external reality moved), so a
   persisted `ROLLUP.md` could go stale behind the cutoff. Fix: the cutoff now also keys on
   the **drift projection** (incremental drift is cheap now, slice07), so a drift change
   regenerates `ROLLUP.md`. The persisted rollup can no longer hide stale drift — the exact
   staleness A5 exists to kill.

## Scope — out (named)

- **Indexing the `deferred` marker / `next`-withholding** — rejected (item 5); a revisit, not
  this slice.
- **New probe kinds, scheduling** — out (scheduled was demoted, ODD-0019; freshness is
  incremental-on-read).
- **A4-index changes** — none; the drift snapshot is the freshness home (streak holds).

## Verification approach

`odm-cli` integration tests + `odm-reconcile` wiring tests:

- orient/rollup run the incremental path; a **counting volatile probe** proves bare `odm`
  runs it **zero** times (regression dissolved); an input-derived fact with a changed input
  *is* re-probed and fresh.
- `odm reconcile` runs the full path — the volatile counting probe increments; snapshot
  `last_checked` re-stamped.
- rollup/orient render "last checked Xm ago" for volatile; "not yet checked" when never run;
  fresh for input-derived. `--json` additive (no schema bump).
- `.odm/drift` written at the default path; `.odm/` gitignored.
- **ROLLUP cutoff drift-aware:** drift changes with no corpus change → `ROLLUP.md`
  regenerates (byte change); no drift + no corpus change → skipped (byte-identical).
- **deferred-`next`:** a deferred, dep-ready node still appears in `next` (graph-pure) *and*
  in the Deferred section — the documented boundary.
- no change under `crates/odm-index/`; clippy `-D warnings`; no `unsafe`; coverage ≥ 90%.

Cargo rows are CC-`attested`, reproduced via CI / local 1.85+.

## Exit criteria

Ledger L-1…L-7 reach a final status: every command reads drift via the incremental path
(orient runs zero volatile probes); `odm reconcile` is the full refresh; honest staleness is
rendered (+ additive `--json`); `.odm/drift` is a gitignored default; the deferred-`next`
decision is settled (graph-pure) and the `ROLLUP.md` cutoff is drift-aware; no index change;
gates pass. **On close, the A5 arc-close runs** (composition check across A-1…A-13,
class-(b) rows reproduced at arc scale).

> **Render/convention:** `writeln!` + `tabled` (no `oxur-cli` dep). Reuse slice07's
> `reconcile_incremental`/`reconcile_full` + the `DriftSnapshot`; join render-identity from
> the corpus (as slice04/06 do).
