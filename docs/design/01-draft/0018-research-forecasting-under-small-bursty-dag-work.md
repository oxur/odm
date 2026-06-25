---
number: 18
title: "Research — Forecasting under small, bursty, DAG-structured work"
author: "topological sort"
component: All
tags: [research, forecasting, telemetry, monte-carlo, pert, evidence, pm-skill]
created: 2026-06-24
updated: 2026-06-24
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# Research — Forecasting under small, bursty, DAG-structured work

> Research ODD gating **arc A8 (forecasting)**. Graduated from
> `workbench/forecasting-research.md`. Companion to the design thread in
> `workbench/forecasting-telemetry.md` (the method) and built on **ODD-0016**
> (which already adjudicated the estimation / velocity / WSJF / CCPM / CHAOS
> priors). Where this ODD and 0016 touch the same ground, 0016 governs the
> estimation-evidence priors; this ODD extends them toward odm's specific data
> regime.
>
> **Method.** Five parallel web-research sweeps (throughput/MC vs estimates;
> small-N/bursty statistics; Monte-Carlo PERT over a network; two-clock/flow
> metrics; Goodhart/metric-gaming), then in-context synthesis + adversarial
> weighing. Claims tagged **[E]** (empirical / peer-reviewed / large-dataset /
> formal) or **[P]** (practitioner-lore / consultative / weak-or-absent controlled
> evidence); contested claims carry their critique. Sources at the end.
>
> **Stance carried from 0016:** trust the formal/empirical; story points,
> velocity, WSJF, CCPM, and CHAOS figures already rejected; dependency-ordering +
> reference-class + throughput Monte-Carlo already endorsed. This ODD *extends*
> those toward our regime; it does not re-litigate them.

## 0. Verdict (bottom line up front)

The method is **directionally sound and evidence-aligned in its parts — with one
big exception that is ours to own: the two-clock separation is an untested
hypothesis, not an established technique.** The small-sample, bursty regime is a
real and repeatedly-flagged threat, but there are principled tools for forecasting
honestly inside it. Net: **proceed, but reframe.**

- **What holds (build on it):** Monte-Carlo over the *real DAG* is the correct fix
  for merge bias (which odm's slice→arc network is full of); empirically-fit,
  *censoring-corrected* distributions over guesses; Bayesian hierarchical
  reference-classes for small-N; survival analysis for in-progress slices; and the
  metadata-never-a-target guardrail. All evidence-backed.
- **The honest correction:** the *dominant evidence-based school forecasts total
  cycle time directly* and captures wait implicitly; **no located evidence shows
  that forecasting active and wait separately beats forecasting the total.** Our
  two-clock idea is plausible and well-motivated but **unvalidated** — a gap we
  would be *contributing to*, not citing. So: build the total-cycle forecast as the
  **baseline/control**, build the two-clock model alongside it, and let our own
  event log adjudicate which is better-calibrated. That is the scientifically honest
  framing and a perfect use of the ground-truth data we already collect.
- **The threat to respect:** Monte-Carlo throughput forecasting stabilizes only at
  **~20+ completed items** [E]; below ~10 the small-sample interval methods
  under-cover (false confidence) [E]. We will be in that danger zone for a long
  time. Honest, *self-widening* intervals are mandatory, not optional.

## 1. What the evidence supports (build on these)

1. **Monte-Carlo over the real network is the right tool, because of merge bias.**
   Classic PERT/critical-path is **optimistically biased by its authors' own
   admission** [E] (Malcolm et al. 1959), because completion is the *max* over
   converging paths (E[max] ≥ max E[·], Jensen) and PERT ignores near-critical
   parallel paths. "Merge bias" is *the* dominant reason PERT underestimates, grows
   with the number of near-critical converging paths, and **only Monte-Carlo
   (recomputing the whole network each iteration) captures it** [E] (Mosaic WP1087;
   IEEE 9259326; MacCrimmon & Ryavek 1962). **Directly relevant:** odm's graph is
   merge-heavy (many slices converge on an arc), so a deterministic critical-path
   forecast *would* read optimistic — MC is not gold-plating, it's the correction.

2. **Empirically-fit, calibrated distributions beat assumed ones.** The leading
   scholarly fix for PERT — **"PERT 21"** (Trietsch & Baker, *IJPM* 2012) — is a
   **lognormal-core distribution, empirically validated on real project data**
   (Trietsch et al., *EJOR* 2012, 24 projects) [E]. This is exactly our
   "measure, don't guess" stance, and it names the distribution family (right-skewed
   / lognormal) for durations that can only stretch.

3. **Bayesian hierarchical reference-classes are the correct small-N technique.**
   With little data, the only lever is structure: **partial pooling shrinks each
   reference class toward the global mean in proportion to how little data it has**
   [E] (Gelman-lineage; rstanarm). It's an established approach for *sparse /
   intermittent / cold-start* series [E] — a close analogue of bursty slice
   arrivals. Caveat: Bayesian honesty is only as good as the prior [E], though
   pooling buffers prior sensitivity [E]. This is how we forecast a slice-type with
   five examples without pretending to precision.

4. **Survival analysis is the right frame for in-progress work.** "Time to
   completion" with open slices is a **right-censored** problem; survival methods use
   the partially-observed durations instead of discarding them [E]. Kaplan–Meier
   (assumption-light, nonparametric) is safer at small N than Cox (which inherits
   sample-size limits) [E].

5. **Throughput Monte-Carlo beats estimates where it's been tested — above a data
   floor.** The one peer-reviewed head-to-head (Fraunhofer/ACM SAC 2021) found
   ~32% MMRE (delivery) and ~20% (effort) for Monte-Carlo vs **134% for developer
   estimates** [E] — but explicitly only **once ≥ ~20 historical data points** were
   available [E]. Single study, single project: real but not settled.

6. **The metadata-never-a-target guardrail is well-grounded.** Goodhart (1975) and
   Campbell (1979) are the mechanism [E]; Campbell's nuance is decisive for us —
   corruption scales with the *degree a metric drives decisions*, so a measure used
   purely for **monitoring/insight is far less corruptible than one used as a
   target**. DORA's own guidance ("metrics are signals, not goals; measure
   processes, not individuals") [E] and the SPACE framework ("no single metric;
   activity is the most-misused dimension; team-level only") [E] are direct
   endorsements. Worker-surveillance research adds: targeting / monitoring
   individuals erodes trust and induces gaming [E/P] → **no per-actor scores, ever.**

7. **Our "fewer robust covariates beat many fragile" instinct is validated.** The
   "10 events per variable" rule is folklore [E] (bias persists past EPV=150);
   worse, **shrinkage/penalization itself destabilizes at tiny N**, so "the honest
   move is fewer covariates, not better regularization" [E]. Direct support for a
   pre-registered short-list over a kitchen sink.

## 2. The small-sample / bursty reality (the threat to respect)

8. **Small N under-covers — in the dangerous direction.** Naive bootstrap
   prediction intervals are **systematically too narrow (false confidence) at small
   N**, with N≈10 the line below which behavior departs sharply from theory [E].
   Distribution-free **conformal prediction** gives finite-sample coverage — but
   *only under exchangeability*, which **non-stationary/bursty time-ordered data
   violates** [E]; it needs adaptive/weighted variants, and small calibration sets
   need a beta-correction (SSBC) or the "90% interval" silently isn't 90% [E].

9. **Bursty clustering is a Hawkes (self-exciting) phenomenon — but un-fittable at
   our scale.** Reliable Hawkes estimation needs thousands-to-100k events; parametric
   fits on little data produce spurious "apparent criticality" [E]. **Use it as the
   conceptual reason intervals must widen, not as a fitted model.**

10. **Steady-state tools are the wrong tools.** Steady-state queueing results are
    inappropriate for finite-horizon, time-varying systems [E]; stability must hold
    *instantaneously*, not on average (a burst can overload a system whose long-run
    utilization < 1) [E]. **Little's Law nuance:** the law itself is general (Stidham
    1972: the finite-window form needs only that the averages converge) [E], but the
    *steady-state* form Throughput = WIP / CycleTime requires arrival≈departure
    equilibrium [P-operationalization] — exactly what gated, hours-to-weeks-idle work
    breaks. **Use the finite-window/arrival-rate framing; never the steady-state
    form.**

11. **Observed durations are biased and censored.** The "Parkinson effect" (work
    expands; early finishes hidden) and rounding contaminate raw durations, so naive
    empirical fits are biased unless corrected for the censoring of early finishes
    [E] (EJOR 2012). And measured active/wait is a **noisy proxy** — "active" states
    get recorded as active overnight/weekends; blocks go unmarked [E] (ASOS).
    **Two design consequences:** (a) derive active-work from **git commit
    timestamps** (precise) rather than day-granularity gate dates — which our design
    already does; (b) treat open slices as **right-censored** (survival), and
    correct for early-finish censoring before fitting.

12. **Cross-slice duration correlation understates tails if ignored.** Independence
    assumptions in network simulation **systematically underestimate tail risk**;
    shared "risk drivers" induce correlation (one study: +15–30% duration when
    modeled) [E/P, figure illustrative]. **Relevant:** a hard project stretch or a
    busy human period correlates many slices at once — independent sampling would
    understate the P80/P90. Model via shared-driver / correlated sampling.

## 3. The unvalidated core — two clocks (own it)

The most important finding is an **absence**: a sweep aimed squarely at the
two-clock hypothesis found **no primary or peer-reviewed evidence that forecasting
active-work and wait-time separately is more accurate than forecasting total cycle
time directly** [evidence gap, honestly flagged]. The prevailing evidence-based
school (Vacanti / Actionable Agile) deliberately forecasts the **empirical total**
and lets wait fall out implicitly.

The *motivation* for separating is strong — Reinertsen's well-supported point that
**invisible queues dominate product-development time** [E] — and mixture/bimodal
modeling is established in adjacent domains (call-center waits, travel-time). But
"separate-then-forecast improves software-work prediction" is, on the located
evidence, **an untested hypothesis we would be contributing, not citing.**

**Design consequence (the honest experiment):** build the **total-cycle empirical
Monte-Carlo as the baseline/control**, build the **two-clock model alongside**, and
let our own ground-truth event log adjudicate via held-out calibration. If the split
wins, we've shown something new; if it doesn't, the baseline is a perfectly good
forecaster. Either way we don't *assume* the headline — we test it. This also
hedges: the baseline works even if the two-clock idea doesn't pan out.

## 4. The most defensible concrete method

A composite, each piece earning its place from §1–§2:

1. **Engine:** Monte-Carlo over the *real* dependency DAG (captures merge bias §1.1),
   ~10k iterations, **correlated sampling** via shared drivers (§2.12), not
   independent draws.
2. **Node durations:** empirically-fit, **right-skewed / lognormal-core** (§1.2),
   **censoring-corrected** (§2.11), drawn from **Bayesian hierarchical
   reference-classes** that pool sparse slice-types toward the global (§1.3).
3. **In-progress slices:** modeled as **right-censored** via survival methods
   (§1.4), not dropped.
4. **Two clocks as hypothesis, not assumption:** model active and wait separately
   *and* total directly; keep whichever is better-calibrated on held-out slices
   (§3).
5. **Intervals:** honest, wide, **self-widening when the reference class is thin**;
   coverage via Bayesian credible intervals and/or **adaptive/weighted conformal
   with finite-sample (beta) correction** (§2.8); never a bare date, always a range
   with confidence.
6. **Two outputs, never merged:** active-work effort (tighter) and calendar
   completion (wide, latency-dominated) — and say which is which.

## 5. Failure modes & guardrails

- **Below ~20 completed slices, don't pretend.** State the data floor; until then,
  forecasts are wide priors, explicitly labelled low-confidence (§1.5, §2.8).
- **Never the steady-state Little's-Law form; never steady-state queueing** for
  gated bursty work (§2.10).
- **Correct observed durations for censoring/Parkinson before fitting** (§2.11);
  derive active time from commit timestamps, not gate dates.
- **Model cross-slice correlation** or knowingly understate the tails (§2.12).
- **Cap covariates hard** (pre-registered short-list); fewer-robust > many-fragile
  (§1.7); beware endogeneity (don't double-count the latency clock as "complexity").
- **Metadata, never targets; team/process, never per-actor** (§1.6).
- **Calibrate against our own actuals and report calibration** (PERT-21 discipline):
  a forecast that isn't checked against what happened is assertion, not measurement.

## 6. Implications for the plan (A7 / A8)

- **A8 earns its research-gate** — confirmed; this ODD is that gate. The statistics
  (hierarchical pooling, survival censoring, correlated MC, conformal correction)
  are non-trivial and warranted the cited pass before code.
- **A7 must capture what A8 needs:** precise (commit-timestamp) active spans;
  open-slice censoring flags; the covariates of `forecasting-telemetry.md` §5; and
  enough history that reference classes aren't empty.
- **Add a baseline:** the total-cycle empirical Monte-Carlo is the control A8 builds
  *first*; the two-clock model is the experiment measured against it.
- **Self-host dividend:** by the time A1–A6 land, odm's own build history (arcs 1–6)
  is the first reference corpus — small, but real, and ours to calibrate on.

---

## Sources

**Throughput / MC vs estimates:** Fraunhofer/ACM SAC 2021 (Tamrakar & Jørgensen),
https://dl.acm.org/doi/10.1145/3412841.3442030 ;
https://publica.fraunhofer.de/entities/publication/b1e86bea-570a-4284-9d2c-e1309567a2da
· Jellyfish story-points dataset, https://jellyfish.co/blog/do-story-points-work/
· https://medium.com/agileinsider/story-points-or-story-count-f4b81d556fa4
· #NoEstimates, https://www.infoq.com/articles/book-review-noestimates/ ;
https://t2informatik.de/en/blog/the-idea-of-no-estimates/
· Vacanti, *Actionable Agile Metrics*, https://actionableagile.com/books/aamfp/
· Little's Law: https://colinsalmcorner.com/littles-law-doesnt-work/ ;
https://www.polaris-flow-dispatch.com/p/a-brief-history-of-littles-law ;
Stidham 1972, https://pubsonline.informs.org/doi/epdf/10.1287/opre.22.2.417

**Small-N / bursty statistics:** bootstrap small-N coverage,
https://www.tandfonline.com/doi/full/10.1080/03610918.2026.2641163 ;
https://www.fharrell.com/post/bootcal/ · hierarchical pooling,
https://cran.r-project.org/web/packages/rstanarm/vignettes/pooling.html ·
intermittent-demand HB, https://arxiv.org/html/2511.12749v1 · survival analysis,
https://www.publichealth.columbia.edu/research/population-health-methods/time-event-data-analysis ;
https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8478547/ · Hawkes,
https://arxiv.org/abs/1403.5227 ; https://arxiv.org/pdf/1308.6756 ·
non-stationary queueing, https://arxiv.org/pdf/1701.05443 · EPV folklore,
https://link.springer.com/article/10.1186/s12874-016-0267-3 · sample-size sizing,
https://academic.oup.com/ejcts/article/67/5/ezaf142/8120086 · conformal,
https://arxiv.org/pdf/1604.04173 ; SSBC, https://arxiv.org/html/2509.15349v1

**Monte-Carlo PERT / merge bias:** Mosaic WP1087,
https://mosaicprojects.com.au/WhitePapers/WP1087_PERT.pdf · original PERT 1959,
https://www.mosaicprojects.com.au/PDF-Gen/PM-History_PERT-Original_Paper.pdf ·
merge-event bias, https://ieeexplore.ieee.org/document/9259326/ · stochastic
critical path, https://onlinelibrary.wiley.com/doi/10.1155/2014/547627 · risk-driver
correlation, http://www.projectrisk.com/schedule_risk_analysis_using_risk_drivers.html
· PERT 21, https://faculty.tuck.dartmouth.edu/images/uploads/faculty/principles-sequencing-scheduling/PERT21.pdf
· Parkinson-lognormal, https://faculty.tuck.dartmouth.edu/images/uploads/faculty/principles-sequencing-scheduling/ModelingActivityTimes.pdf
· CCPM critique, https://www.pmi.org/learning/library/critical-chain-management-research-literature-5508

**Two-clock / flow metrics:** flow efficiency,
https://resources.kanban.university/flow-efficiency-a-great-metric-you-probably-arent-using/
· Reinertsen queues, https://agility-at-scale.com/principles/product-economics/ ·
ASOS empirical flow-efficiency, https://medium.com/asos-techblog/our-survey-says-uncovering-the-real-numbers-behind-flow-efficiency-e54f136b1fab
· work item age, https://www.55degrees.se/post/what-is-work-item-age · CFD,
https://businessmap.io/kanban-resources/kanban-analytics/cumulative-flow-diagram

**Goodhart / metric-gaming:** Goodhart's Law,
https://en.wikipedia.org/wiki/Goodhart%27s_law · Campbell's Law,
https://en.wikipedia.org/wiki/Campbell%27s_law · DORA signals-not-goals,
https://bryanfinster.com/whitepapers/dora-metrics · SPACE framework,
https://queue.acm.org/detail.cfm?id=3454124 · less-flawed metrics,
https://pmc.ncbi.nlm.nih.gov/articles/PMC10591122/ · estimate inflation,
https://www.mountaingoatsoftware.com/blog/how-to-prevent-estimate-inflation
