---
id: 01KYP5GZ1VY5JN8QJ4TDQS1AX9
number: 580487800
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 08 (Arc 05): freshness on every command + honest staleness (arc capstone)'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice08-freshness-wiring/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKT5WD5R931YMT2FNE
---
# CC Prompt — Slice 08 (Arc 05): freshness on every command + honest staleness (arc capstone)

The **last slice of A5**. slice07 built the freshness mechanism; this slice **wires it into
every command**, renders **honest staleness**, and settles the arc's two loose ends. It
dissolves the slice04 orient-runs-probes regression. On close, the **A5 arc-close** runs.

> **Start condition:** slices 01–07 CDC-verified. Branch off `main`:
> **`arc05-slice08-freshness-wiring`** (not `main`). If 01–07 unmerged, branch off slice07's
> branch and flag for rebase.

## Read first
1. **ODD-0019** §3.2 (incremental), §3.4 (honest staleness + explicit refresh), §8 (the
   wiring) — the design of record.
2. `slice08-freshness-wiring/ledger.md` (7 rows) + `slice-doc.md` (same dir) — especially the
   two **settled** loose ends (deferred-`next` = graph-pure; `ROLLUP.md` cutoff = drift-aware).
3. `../arc-plan.md` — A5 capability + `## Freshness model`, Arc Ledger (this slice closes
   **A-8**, and its wiring makes the compose rows A-9…A-13 arc-reproducible).
4. slice07's API (`crates/odm-reconcile/src/incremental.rs`, `snapshot.rs`) —
   `reconcile_incremental`/`reconcile_full` + the `DriftSnapshot` you wire in. slice06's
   `reconcile_views` (`crates/odm-cli/src/reconcile.rs`) — the seam you flip. A4 slice07's
   `ROLLUP.md` early-cutoff (the meta-fingerprint skip) — you make it drift-aware.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design, error handling.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc** at close; **propagate it into `arc-plan.md` (A-8 + version entry)
  yourself**; note **all 8 slices delivered → the arc-close is next**).

## Task
1. **Wire `reconcile_views` → incremental** (L-1): orient/rollup run `reconcile_incremental`;
   bare `odm`/orient runs **zero** volatile probes (prove with a counting volatile probe);
   a changed input *is* re-probed.
2. **`odm reconcile` → full** (L-2): the explicit command runs `reconcile_full` (re-probes
   volatile + input-derived, re-stamps the snapshot).
3. **Honest staleness** (L-3): render volatile as "last checked Xm ago" (from `last_checked`),
   input-derived fresh; never-checked volatile → "not yet checked — run `odm reconcile`";
   `--json` additive (no schema bump).
4. **`.odm/drift` default path + gitignored** (L-4).
5. **`ROLLUP.md` cutoff drift-aware** (L-5): a drift change (no corpus change) regenerates
   `ROLLUP.md`; no change → still skipped. (Key the cutoff on the drift projection too.)
6. **Settle deferred-`next`** (L-6): `next` stays graph-pure (unaffected by a `deferred`
   marker); document the boundary in a code comment + the docs. Do **not** withhold deferred
   from `next` (that's the rejected index/`FORMAT_VERSION` path or the reconcile-in-`next`
   regression).
7. **Gates + no index change** (L-7): `git diff` under `crates/odm-index/` empty; clippy
   `-D warnings`; no `unsafe`; coverage ≥ 90% line.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If wiring the incremental path or the drift-aware cutoff
  needs a shape change, raise an amendment against ODD-0019.
- **The regression must actually be gone** (L-1) — assert bare `odm` runs the volatile probe
  **zero** times (counting probe), not just "it's faster."
- **No `odm-index` change** — the freshness home is the drift snapshot. Keep A5's streak.
- **`next` stays graph-pure** (L-6) — do not couple it to the reconcile/deferred layer.
- **Honest, never fabricated:** never-checked volatile says so; no invented "fresh".
- **Additive JSON:** no `rollup/v1`/`orient/v1`/`reconcile/v1` version bump for staleness.
- **Render:** `writeln!` + `tabled`.

## Deliverables
The wired incremental freshness + honest-staleness rendering + the two settled loose ends,
with `ledger.md` evidence per row (`attested`); a `closing-report.md` — per-row walk **plus
the Bubble-up to the arc** (did slice08 deliver A-8; **all 8 slices delivered → arc-close
next**; what the wiring revealed; the silent-drop diff) — **and propagate that into
`arc-plan.md` (A-8 + a version entry)**. Feature branch `arc05-slice08-freshness-wiring`;
not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-8) per LEDGER-DISCIPLINE v2.0 §A — and the **arc-close**
(recomposition + class-(b) rows reproduced at arc scale) follows.
