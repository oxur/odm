# CC Prompt — Slice 04 (Arc 05): drift in `rollup` / `orient`

Fourth slice of A5. slices 01–03 built the model, both probes, the runner, and standalone
`odm reconcile`. **Retire the A3 placeholder**: `rollup` and `orient` stop printing "not yet
tracked (A5)" and render **real drift** from an on-demand reconcile. No `affects`/deferred/
scheduled (slices 05–07).

> **Start condition:** slices 01–03 CDC-verified. Branch off `main`:
> **`arc05-slice04-drift-in-rollup-orient`** (not `main`). If 01–03 unmerged, branch off
> slice03's branch and flag for rebase (as prior slices did).

## Read first
1. `slice04-drift-in-rollup-orient/ledger.md` (7 rows) + `slice-doc.md` (same dir) —
   especially **"The layering constraint"** and **"drift cannot come from the index."**
2. `../arc-plan.md` — A5 capability, Arc Ledger (this slice closes **A-4** and satisfies
   the **A-10** compose row: "drift surfaces in rollup/orient, placeholder gone"), v1.5/v1.6
   (the settled store-read/no-index-cache resolution + the slice03 render-identity seam).
3. The A3 placeholders you're replacing: `crates/odm-core/src/rollup.rs` (`Drift {}` at ~155,
   the `Rollup` model + `assemble`), `crates/odm-cli/src/orient.rs` (drift section #5, "not
   yet tracked (A5)"), `crates/odm-cli/src/rollup.rs` (index-backed model build via
   `odm_index::reconcile` + the drift-section renderer).
4. slice02/03 API: `odm_reconcile::{Runner, CorpusReport, NodeReport, OutcomeCounts}`;
   `crates/odm-cli/src/reconcile.rs` (the render + `verdict`).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design, error handling, layering.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc** at close; **propagate the bubble-up into `arc-plan.md` (A-4 row +
  version entry) yourself**, as in slices 02–03).

## Task
1. **`Drift` model** (`odm-core`, S-1): flesh the empty `Drift {}` into a data projection —
   counts + per-node drifted/errored entries with identity (number/name, `fact_id`,
   `describe`) + `expected`/`observed`. **Plain data; add NO `odm-reconcile` dep to
   `odm-core`** (dependency runs odm-reconcile → odm-core only).
2. **Report identity** (`odm-reconcile`, S-2): enrich `run_corpus`'s report with node
   number/name + fact `describe`, joined once from the frontmatters it already loads —
   resolving slice03's double-load. (Optionally simplify `reconcile.rs` to drop its second
   `load_all`; flag if you do.)
3. **Wire `rollup`** (S-3): run an on-demand `Runner::run_corpus` (store-read), project →
   `odm_core::Drift`, inject into the model; drift section renders real drift; clean → "no
   drift" (no fabricated data); the "not yet tracked (A5)" string is gone.
4. **Wire `orient`** (S-4): same, via the shared helper.
5. **Shared helper** (S-6): one `odm-cli` function computes+projects drift, called by both
   `rollup` and `orient` (no copy-paste; the two views can't diverge).
6. **`--json`** (S-7): `rollup/v1` + `orient/v1` carry the populated `drift` — **additive**
   (the slot existed empty in A3; no version bump).
7. **Gates** (S-7): clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If the `Drift` shape or the `assemble` seam forces a wider
  change than the ledger implies, raise it.
- **Drift is store-read + on-demand — never index-cached** (S-5). Do NOT add `desired_facts`
  or drift to `IndexRecord`. The model stays index-backed; drift comes from `run_corpus`.
  This keeps the A4 invariant un-triggered.
- **Layering:** `odm-core` must not depend on `odm-reconcile`. The projection lives in
  `odm-cli`.
- **Render convention:** `writeln!` + `tabled` (odm-cli has **no** `oxur-cli` dep — the
  slice03 finding). Do not introduce `oxur_cli`.
- **No fabricated data:** clean corpus → an honest "no drift", never invented rows.
- **Stay in scope:** no `affects`/deferred/scheduled; no index changes.

## Deliverables
The `Drift` model + report-identity + wired `rollup`/`orient` + `--json`, with `ledger.md`
evidence per row (`attested`); a `closing-report.md` — per-row walk **plus the Bubble-up to
the arc** (did slice04 deliver A-4 + satisfy A-10; what it reveals for slice05/06; the
silent-drop diff) — **and propagate that into `arc-plan.md` (A-4 + a version entry)**. Feature
branch `arc05-slice04-drift-in-rollup-orient`; not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-4) per LEDGER-DISCIPLINE v2.0 §A.
