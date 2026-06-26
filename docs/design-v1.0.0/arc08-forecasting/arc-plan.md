# Arc 08 — Forecasting (plan-of-record)

> Refs: **ODD-0018** (the research gate — the defensible method, failure modes, and
> guardrails; *read it before slicing this arc*); `workbench/forecasting-telemetry.md`
> §7 (the corrected method); `project-plan.md` §4. `depends_on:` **A7** (the event log,
> the two clocks, the covariate records) + **A2** (the dependency DAG to simulate over).
>
> **Status:** scoped, not started — **post-MVP extension arc, design-ahead**, and the
> *furthest* out. Slice altitude deliberately coarse: A8's statistics are
> research-gated (ODD-0018), so the slice breakdown is a **shape, not a detailed plan**
> — the detail is written when the arc becomes active and the A7 corpus exists to fit
> against. Manufacturing fine detail now would be the "plan written far ahead" failure
> (PROJECT-MANAGEMENT Part VII).
>
> **The honest framing (ODD-0018 verdict).** The two-clock *separation* is an
> **untested hypothesis**, not an established technique. A8 therefore builds the
> **total-cycle empirical forecast as the baseline/control** *first*, the two-clock
> model *alongside*, and lets odm's own event log adjudicate by held-out calibration.
> The DAG engine is well-supported (Monte-Carlo fixes **merge bias**, which slice→arc
> graphs have); the small-sample/bursty regime is the real threat.

## Capability

Turn A7's measured substrate into **honest, evidence-based forecasts** — the
management-abstraction layer Agile's velocity promised, on ground truth instead of
guesses. **Monte-Carlo over the real dependency DAG** with **measured,
censoring-corrected, lognormal-core** node-time distributions drawn from **Bayesian
hierarchical reference classes** (pooling sparse slice-types toward the global), with
**correlated sampling** (shared drivers, not independent draws). Forecast **two
outputs, never merged** — active-work effort (tighter) and calendar completion (wide,
latency-dominated) — as **self-widening ranges with confidence, never bare dates**,
explicitly low-confidence below the **~20-completed-slice data floor**. Everything
stays **metadata, never a target**. Proposed new crate: **`odm-forecast`** (atop
`odm-telemetry`).

## Exit criteria (arc acceptance)

- A **total-cycle empirical Monte-Carlo** forecast over the DAG (the control) produces
  calibrated ranges-with-confidence on the A7 corpus.
- Node-time distributions are **empirically fit, right-skewed/lognormal-core, and
  censoring-corrected**; in-progress slices enter as right-censored, not dropped.
- Reference classes use **Bayesian hierarchical pooling** (sparse classes shrink toward
  the global); the pre-registered covariate short-list drives bucketing; covariate
  count is capped (small-N overfitting guardrail, ODD-0018 §1.7).
- Simulation uses **correlated sampling** (shared drivers); the independence trap is
  avoided (ODD-0018 §2.12).
- The **two-clock model is built and measured against the total-cycle control** on
  held-out slices; whichever is better-calibrated is kept — the split is *tested, not
  assumed*.
- Forecasts are **ranges with confidence that widen honestly** when the reference class
  is thin; below the data floor they are labelled low-confidence; **no bare date is
  ever emitted**; calibration is reported against actuals (PERT-21 discipline).
- No steady-state Little's-Law / steady-state-queueing forms are used for the bursty
  regime (ODD-0018 §2.10). No per-actor scoring exists.

## Slices (shape only — detail deferred to build time per ODD-0018)

> Coarse and provisional; expand into real slices when the arc is active and the A7
> corpus exists. Listed to mark the shape, not to commit detail far ahead.

1. **slice01 — reference classes + distribution fitting.** Hierarchical pooling over
   the pre-registered covariates; censoring-corrected lognormal-core fits; the
   data-floor honesty (low-confidence below ~20).
2. **slice02 — Monte-Carlo DAG engine.** Simulate completion over the real DAG with
   correlated sampling; merge-bias-correct by construction; the total-cycle control.
3. **slice03 — two-clock experiment + calibration.** Build the active/wait split,
   forecast both clocks, and calibrate split-vs-total on held-out slices; report which
   wins; emit the two outputs.
4. **slice04 — forecast surfacing + export.** Ranges-with-confidence in `orient`/rollup
   and as an export projection (ODD-0017); never a bare date; calibration visible.

## Arc Ledger

> Per LEDGER-DISCIPLINE v2.0 §B (Option A). Opens with class-(b) composition rows from
> the capability; class-(a) slice rows are **provisional** (the slice set firms up at
> build time per ODD-0018) and accrue as slices close, with class-(c) bubble-up.
> **Class-(b) rows are reproduced at arc scale**, and per ODD-0018 include a
> calibration check against actuals. Design-ahead: all rows `open`.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| A-1 | slice01 (reference classes + distribution fitting) closed | ptr: slice01 `cdc-verification.md` | correctness | arc-plan | open | | attested; slice set provisional |
| A-2 | slice02 (Monte-Carlo DAG engine — the total-cycle control) closed | ptr: slice02 `cdc-verification.md` | correctness | arc-plan | open | | attested; provisional |
| A-3 | slice03 (two-clock experiment + calibration) closed | ptr: slice03 `cdc-verification.md` | correctness | arc-plan | open | | attested; provisional |
| A-4 | slice04 (forecast surfacing + export) closed | ptr: slice04 `cdc-verification.md` | correctness | arc-plan | open | | attested; provisional |
| A-5 | **Compose:** the total-cycle empirical Monte-Carlo (the control) produces calibrated ranges-with-confidence over the A7 corpus | arc-scale demo + calibration vs actuals | serious | arc-plan / ODD-0018 | open | | reproduce at arc scale |
| A-6 | **Compose:** node-time distributions are empirically-fit, lognormal-core, censoring-corrected; in-progress slices enter right-censored | arc-scale demo | serious | arc-plan / ODD-0018 §1.2,§2.11 | open | | reproduce at arc scale |
| A-7 | **Compose:** reference classes use Bayesian hierarchical pooling; covariate count capped (small-N overfitting guardrail) | arc-scale demo | serious | arc-plan / ODD-0018 §1.3,§1.7 | open | | reproduce at arc scale |
| A-8 | **Compose:** the two-clock split is **built and measured against the total-cycle control** on held-out slices; the better-calibrated is kept (tested, not assumed) | arc-scale demo: held-out calibration of both models | serious | arc-plan / ODD-0018 §3 | open | | the central experiment |
| A-9 | **Compose:** forecasts are ranges-with-confidence that widen honestly when thin; low-confidence below the ~20-slice floor; **no bare date emitted**; calibration reported | arc-scale demo | serious | arc-plan / ODD-0018 §5 | open | | reproduce at arc scale |
| A-10 | **Compose:** no steady-state Little's-Law / steady-state-queueing forms used for the bursty regime; **no per-actor scoring** | arc-scale audit | serious | arc-plan / ODD-0018 §2.10 | open | | reproduce at arc scale |
| A-11 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | open | | accrues as slices close |

## Dependencies

Consumes: **A7**'s event log + two clocks + covariate records (the entire input), and
**A2**'s DAG (the network to simulate). Leans on the A7 self-host corpus (odm's own
arc-1–6 history) as the first reference data. Adjacent: the interop **export** arc can
project forecasts outward once both exist. Nothing downstream depends on A8 within the
known roadmap — it is a leaf.

## Open design questions (resolve in slice docs, against ODD-0018)

- **Does the two-clock split beat the total-cycle control?** The central empirical
  question; A8 is designed to *answer* it, not assume it.
- **Latency model**: is inter-slice latency forecastable at all, or reported as a wide
  historical-gap prior (it tracks human availability, not the work)?
- **Interval method**: Bayesian credible intervals vs adaptive/weighted conformal with
  finite-sample (beta) correction — and the exchangeability violation under
  non-stationarity (ODD-0018 §2.8).
- **Crate placement**: `odm-forecast` (proposed) atop `odm-telemetry`.

## Method

Ledger per slice; CC implements, CDC verifies; five-iteration cap. **Re-read ODD-0018
before slicing** — it is the research gate, and its failure-modes/guardrails section is
the acceptance backbone above. Slice closes bubble up here; the arc closes with a
`closing-report.md` + composition check, and (per ODD-0018) a calibration report: a
forecast that isn't checked against what happened is assertion, not measurement.

## Version History

### v1.1 — 2026-06-26
Added the **`## Arc Ledger`** section per LEDGER-DISCIPLINE v2.0 §B, matching arc04–07.
Class-(a) slice rows are flagged **provisional** (the slice set is shape-only until the
arc is built per ODD-0018); class-(b) composition rows carry the ODD-0018 method +
calibration. Pure addition — the v1.0 body is unchanged. Surfaced by: the
ledger-discipline upgrade applied to the extension arcs.

### v1.0 — 2026-06-26
Initial arc-plan at shape-only altitude, framed by ODD-0018's verdict, as part of the
post-arc6 project-plan extension (project-plan v1.2). Slice detail deliberately deferred
to build time — A8's statistics are research-gated and want the real A7 corpus to fit
against; far-ahead detail would rot (PROJECT-MANAGEMENT Part VII).
