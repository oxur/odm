# odm — telemetry & forecasting (post-arc6 thread)

> **Status.** Working design note / brainstorm capture — **not yet an ODD**.
> Candidate to graduate into a research ODD once the thread firms up. Written
> 2026-06-24 (CDC + Duncan, the post-arc6 exploration thread, conversation 4 of
> the handoff map). Companion to
> `docs/dev/research/0005-odm-as-a-pm-tool-arc-6-baseline-analysis.md` (was
> `workbench/pm-tool-baseline-analysis.md`), which
> established the arc-6 baseline and the category-(c) "in-grain" menu this thread
> develops.
>
> **Sources.** Grounded in ODD-0016 (SWE PM failures & practices — the
> evidence layer), ODD-0011 (planning-system research), ODD-0013 (architecture:
> gates, evidence levels, derived order, provenance-as-derived-lineage), ODD-0017
> (interop / export-as-projection), ODD-0001 (the project-x failure modes). Crate
> facts (oxur-cli, ascii-dag) verified 2026-06-24.
>
> **Calibration up front (load-bearing).** The *rendering* and the *forecasting
> method* rest on solid ground (0016 endorses reference-class + throughput
> Monte-Carlo explicitly; the data already exists in the schema). The *bet* is
> that useful forecasts survive **small, bursty, human-gated samples**. That is an
> empirical question we cannot answer in advance — error bars will be wide early,
> and the calendar forecast may be dominated by an irreducible human-latency term.
> This note describes a promising, distinctive direction, not a proven result. We
> hold it provisionally and let the data correct us.

---

## 1. The thesis

odm can offer what Agile's velocity layer *promised* — a statistical membrane
that translates capacity into schedules-with-error-bars, freeing engineers from
"when will it be done?" — but built on **measured ground truth** instead of
**estimated proxies**. It does this because, at the close of arc 6, odm already
captures a ground-truth **event log** (git timestamps + gate-reached timestamps +
evidence-level transitions) and a **real dependency DAG**. No mainstream PM tool
has both; most have neither honestly.

The reframe that makes it click: in an LLM-driven workflow, **wall-clock time
flips role** — it stops being the thing work is *judged on* and becomes the
*richest signal we have*. Time goes from **scoreboard to telemetry.**

## 2. Agile's real invention, rebuilt on ground truth

Agile's most under-credited contribution was an **insulation layer**: by deriving
velocity from story points, it moved the "when?" question off engineers and onto a
statistical translation owned by the PM (track complexity + velocity → map to
dates with error bars → train management to accept ranges).

The flaw is the foundation. ODD-0016 is blunt: story points have weak-to-no
correlation with cycle time, no cross-team validity, and are gameable by inflation
and story-splitting (Goodhart). The membrane was real and valuable; the proxy
under it was lore.

**odm's move:** keep the membrane, replace the proxy with measurement. Forecast
from *observed* complexity and *observed* timing, not from up-front guesses. This
is the evidence-backed version of exactly what velocity was reaching for, and it
is ODD-0016 §5's one endorsed method ("count small uniform items + empirical
throughput / Monte-Carlo; never summed story points").

## 3. The two-clock model (the crux)

The interrupt-driven reality of LLM work forces a separation that conventional
tools blur. Two clocks, two different random variables, two different uses:

- **Active-work time** — within a slice: first commit → close, iterations burned,
  gate-to-gate transitions. Measures **the work**. (≈ Lean *touch time* / cycle
  time.)
- **Inter-slice latency** — the gap from one slice to the next. Measures **human
  guidance bandwidth and availability**, *not the work*. (≈ Lean *wait time*; the
  ratio touch/(touch+wait) is *flow efficiency*.)

Conflating these is what makes existing "forecasts" unreliable: a slice that took
three weeks of calendar might be 40 minutes of active work and the rest waiting for
a human to look. **The two clocks must be modelled and forecast separately**, then
recombined only at the end, explicitly.

odm's edge: it has the **ground-truth event stream** to compute both honestly. Git
commit timestamps + `status` gate `reached:` dates + evidence-level transitions
*are* the events. Jira infers cycle time from when a human dragged a card; odm
reads it from the commits and gate flips. This is the concrete cash value of
"direct access to the raw data."

## 4. The ground-truth event log (data we already have)

Per ODD-0013, "provenance" is **derived lineage** — git history + the supersedes
chain + gate-reached timestamps — *not* a stored field. The telemetry is therefore
mostly **derived, not stored**:

- **Gate transitions:** `status.<gate> = { reached: <date>, by, evidence }` already
  records *when* each gate was reached, *by whom*, and at *what evidence level*.
- **Git:** commit timestamps and per-commit diff stats on the slice branch give
  active-work spans, files touched, and churn.
- **Evidence progression:** the `asserted → attested → reproduced → reconciled`
  ladder gives a *verification latency* per gate (how long until a claim became
  trustworthy), which is itself a quality/risk signal.

## 5. Observed covariates (language-agnostic) — prediction vs post-hoc

The inversion of story points: don't *guess* complexity up front; **collect cheap,
language-agnostic covariates** and let the statistics weight them. odm must serve
Go, Rust, Python, Erlang, Lisp, Shell, and Makefile teams alike, so **no code-AST /
language-specific metrics** (no cyclomatic/cognitive parsing — those can't span
Erlang/Lisp/Makefile, and reaching into language toolchains would couple the
planning substrate to them). Only signals from **git**, odm's own **graph/ledger**,
the **gate/evidence event log**, and **calendar** context.

Each is tagged by the clock it likely informs — **→W** active-work, **→L**
inter-slice latency, **→R** rework-risk — and split **leading** (known *before* a
slice starts → can *forecast* it) vs **lagging** (known only *after* → trains the
reference class + powers post-hoc analysis). That split is load-bearing: only
leading covariates predict an unstarted slice.

### Inventory by source

- **Git / VCS (`gix`, already a dep):** churn (±lines, net) →W; files touched,
  distinct dirs/crates (scatter), hunk count →W; intra-slice commit gaps (the
  two-clock *fractally*) →W/L; commit count →W; fixup/revert commits, force-pushes
  →R; edits to *old/stable* files (hotspots) →R; temporal change-coupling →R; diff
  entropy (real vs repetitive change) →W. Exclude generated/vendored paths
  (gitattributes `linguist-generated` — itself agnostic).
- **odm graph (native, free):** edge degree (`depends_on`/`consumes`/`blocked_by`/
  `affects`) →W; **soft-satisfied deps at start** →R; downstream blast radius →W/R;
  tears attached →R; topological position, part_of depth, gate-set length →W.
- **Ledger / five-doc set (counts + prose):** ledger-row count →W; significance mix
  (more `correctness`/`serious`) →W/R; test-gated vs structural row ratio →W;
  cc-prompt / out-of-scope size →W; **flagged-deviations & uncertainties-named** →R;
  amendments raised →R.
- **Gate/evidence event log:** iterations-to-close →W/R; **verification latency**
  (`attested → reproduced`, now captured via `evidence_dates`) →L/R; evidence
  regressions →R; **evidence-at-close** →R; built→verified lag →L.
- **Calendar / sequence (the latency clock's home):** day-of-week / time-of-day,
  weekend/holiday in the gap, burst-vs-drought clustering, elapsed since prior close
  →L; prior-slice outcome (difficulty autocorrelation), clean-close streak,
  `origin: discovered/amendment` vs `planned`, backlog dwell, `deferred` re-entry
  →W/L/R.

### Leading vs lagging

- **Leading** (forecast an unstarted slice): ledger-row count, edge degree,
  soft-satisfied-deps-at-start, cc-prompt/scope size, topological position, part_of
  depth, backlog dwell, `origin`, prior-slice outcome, clean-close streak, and the
  calendar/burst context (for →L).
- **Lagging** (train the reference class / explain): iterations, churn / files /
  scatter, diff entropy, fixup-revert count, flagged-deviations &
  uncertainties-named, verification latency, evidence regressions, evidence-at-close.

### odm-native gems (no PM tool has these)

- **soft-satisfied-deps-at-start** — starting on dependencies still below the
  evidence threshold: the *503-failure covariate*, the satisfaction model predicting
  its own rework risk (leading, →R).
- **evidence-at-close** — closed at `attested` vs `reconciled`: a premature-close
  detector (lagging, →R).
- **closing-report deviation / uncertainty counts** — a near-free *struggle/surprise*
  signal that exists only because the CC/CDC process produces honest flags
  (lagging, →R).

### Pre-commit short-list (start here; earn the rest from data)

- work clock (leading): **ledger-row count + soft-satisfied-deps + edge degree**
- latency clock (leading): **elapsed-since-last-close + burst/drought**
- rework lens (lagging): **iterations × evidence-at-close**

### Selection discipline (small-N is the enemy)

With dozens of candidates and tens of slices, **overfitting is the default, not a
risk**. So: prefer a *pre-registered handful* + regularization over a kitchen sink;
beware **endogeneity** (e.g. "time at evidence level" is partly the latency clock in
disguise — don't double-count); keep every covariate **metadata, never a target**
(Goodhart); let recency-weighting absorb non-stationarity. Three robust covariates
beat fifteen fragile ones.

## 5b. The CDC verification phase as instrumentation

The CDC verification is becoming **load-bearing as a data source**, not just a
quality gate — it is where the richest lagging covariates are born (iterations,
per-row dispositions, ruled deviations, uncertainties, evidence-at-close). Two
consequences, both on-grain:

1. **Strengthen its specificity.** The verification must emit *structured*,
   machine-readable outcomes — counts and enums, not only prose — so the covariates
   are reliable rather than fragilely grep'd from free text. Candidate structured
   fields per slice: `iterations_used`, rows-by-disposition (pass / soft / deferred),
   `deviations_flagged`, `uncertainties_named`, evidence-at-close per gate, the CDC
   verdict.
2. **Mechanize collection as tool-use, not hand-work.** Per odm's whole thesis —
   turn the discipline into a tool operation. odm (a `check` / telemetry step)
   should **compute and annotate** these onto the node — churn from `gix`, counts
   from the ledger, dates from the gate log — so neither CC, CDC, nor the operator
   hand-keys them. Hand-collection is error-prone; mechanical collection is reliable
   *and* harder to game by accident. This is A7's collection layer, and it folds
   into the A6 PM-skill (the CDC-verification template becomes a structured,
   tool-backed form rather than free prose).

## 6. Metadata / schema additions

Most telemetry **derives** from gates + git; we store only what git can't
reconstruct reliably. Grounded in the *built* schema (`odm-core/src/status.rs`):
`GateRecord { reached: NaiveDate, by, evidence }` — note `reached` is **day
granularity**, and `set_gate` **overwrites** on an evidence raise.

### Decisions locked 2026-06-24

- **Active-start rule = the first commit on the slice's branch** (full-precision
  git timestamp, automatic). *Not* the `planned` gate date (that's backlog-birth, a
  different clock) and *not* a manual `started` marker (unreliable bookkeeping — the
  smell odm exists to kill). Gates give **phase boundaries + actor attribution** at
  day granularity; **git is the precision clock** for active-work spans. Two
  complementary clocks, both already emitted. (Gates stay `NaiveDate` by design.)

- **Record: `branch:` on the slice node** (the keystone). Auto-populated by odm
  when work starts — not a commit-trailer convention (discipline-dependent). Without
  it, git-derived signals (active span, diff size, files touched, inter-slice
  latency) can't be attributed to a slice, since implementation commits touch
  `crates/…`, not the slice's node file. *Caveat:* after merge + branch-delete,
  `git log <branch>` won't resolve — A7 derives-and-caches at close, or uses
  first-parent merge ranges.

- **Record: `iterations:` on the slice node** — promoted from the count the CDC
  verification already produces ("Iterations used: N"), so it's not new bookkeeping.
  **Definition (super-clear, by request — close ≠ correct):** the number of CC↔CDC
  delivery attempts taken to reach ledger close *for this slice instance* (the
  five-iteration-cap counter). It is a **difficulty/effort** signal, explicitly
  **not** a correctness signal — the closing iteration is *not* guaranteed correct,
  and odm makes no such claim. Work later found incomplete or wrong is **never**
  retro-incremented here; a later correction is its own tracked event (an amendment
  via `supersedes {kind: updates}`, a re-entry from `deferred`, or a follow-up
  slice).

- **Record: per-evidence-level first-reached dates on `GateRecord`** — the
  verification-latency / evidence-churn signal. `set_gate` overwrites today, so the
  `attested → reproduced` transition timing is lost; git reconstruction is fragile
  under squash-merge / rebase, so we **store** it (optional, omitted when empty,
  back-compatible). **Record-now-or-lose-it:** captured in **slice 05.1** so arcs
  2–6's data isn't lost. `reached`/`by`/`evidence` and all consumers unchanged.

### Three orthogonal axes (never collapse into one number)

- `iterations` → *how hard was it to close?* (effort)
- evidence level → *how well-verified was the close?* (confidence)
- **rework events** → *did the close hold up over time?* (quality)

The third is **free / derived** — a `supersedes {updates}`, a `deferred` re-entry,
or an evidence regression against a previously-closed slice. High rework against low
closing-evidence is the signal that says *we're closing slices prematurely*. odm's
evidence-vector model already killed "done means done"; this just reads it back out.

Everything else (durations, gaps, churn beyond the above, diff size, recomposition
breadth, edge degree, ledger-row count) stays **derived** — consistent with
provenance-as-derived-lineage; no truth duplicated.

## 7. Forecasting: Monte-Carlo PERT over the *real* DAG

The crescendo: odm uniquely holds **both halves** that nobody else combines —

1. the **real dependency DAG** (arc A2), and
2. **empirically-fit per-node time distributions** (from the event log, §4–5).

So odm can run **Monte-Carlo PERT on the actual graph with measured durations.**
Everyone else runs PERT/Monte-Carlo on *estimated* durations over a *hand-drawn*
network; odm runs it on *measured* ones over the *derived* DAG.

Forecast **two outputs**, and never collapse them:

- **Active-work effort** — narrow error bars; a property of the work.
- **Calendar completion** — wide error bars; dominated by the inter-slice latency
  distribution, which tracks human availability, not the code.

Stating which is which *is* the honest membrane — strictly better than a single
velocity number that silently mixes the two. Method is ODD-0016-endorsed:
reference-class buckets + throughput Monte-Carlo, not summed points.

> **Research correction (2026-06-24 — see ODD-0018, the forecasting research ODD).**
> The two-clock *separation* is an **untested hypothesis**: the dominant
> evidence-based school forecasts *total* cycle time directly and captures wait
> implicitly, and no located evidence shows decomposition forecasts better. So A8
> builds the **total-cycle empirical Monte-Carlo as the baseline/control** *first*,
> the two-clock model *alongside*, and lets our own event log adjudicate by
> held-out calibration. The DAG engine is well-supported (Monte-Carlo is the
> established fix for **merge bias**, which our slice→arc network is full of), but
> use **censoring-corrected, lognormal-core** node distributions drawn from
> **Bayesian hierarchical reference-classes** (the principled small-N tool), with
> **correlated sampling** (shared drivers), and honest **self-widening** intervals.
> Hard data floor: throughput Monte-Carlo stabilizes only at **~20+ completed
> slices** — below that, forecasts are explicitly low-confidence wide priors.

## 8. Guardrails (non-negotiable)

- **Metadata, never a target (Goodhart).** The analytics surface emits
  *distributions and forecasts*, never per-actor scores or leaderboards. There must
  be no single "velocity" number to game; the moment "slices/week" becomes a goal,
  slice-splitting and inflation follow — the exact story-point failure ODD-0016
  documents.
- **Calibrated about our own power.** Bursty + human-gated ⇒ small samples per
  reference class ⇒ wide bars early. Report uncertainty honestly; resist precision
  we don't have (ODD-0016's replication/over-claiming guardrail, turned on
  ourselves). The forecast should *widen* its own bars when the reference class is
  thin.
- **Forecast, don't promise.** Schedules are ranges with confidence, surfaced as
  such; the tool never emits a bare date.

## 9. Rendering surfaces & prior art

Two renderers for two surfaces — they are complementary, not competing:

- **Tables (`oxur-cli::table`, *our own* library, built on `tabled` 0.17).** For the
  **committed / diffed** surface: the status×gate evidence matrix, forecast
  distribution tables, the rollup. Renders to `String` (copy-pasteable by
  construction), has semantic cell-coloring hooks (`state_to_fg_color` →
  generalize to evidence/gate coloring), title/footer/spans. Stable line-diffs:
  add a node → add a line.
- **`ascii-dag` (third-party, MIT/Apache, **zero-dependency / `no_std`**, Sugiyama
  layout).** For the **on-demand display / export** surface: `odm viz` / `odm
  export` — the ODD-0017 evangelism engine, where a human absorbs DAG structure at
  a glance. Regenerated fresh, never committed/diffed (auto-layout reflows under
  change → poor as a tracked artifact).
  - **Adoption cost flagged:** its `rust-version` is **1.92**, above odm's **1.85**
    MSRV. Pulling it into core bumps the floor; alternative is feature-gating it or
    confining it to the CLI/export crate. Its zero-dep/`no_std` posture is otherwise
    unusually well-aligned with odm's minimal-infra ethos.

The two visualization and the **export** threads converge: export (ODD-0017) is
already "a renderer over the rollup," so the status matrix (table) and the DAG
(`ascii-dag`) are simply export projections in two formats. One projection layer,
multiple shapes.

**Key insight (medium enforces grain).** The plain-text constraint isn't just a
nice copy-paste property — it *selects for* stable, diffable encodings (tables) for
tracked state and *against* unstable ones (auto-laid-out node-link) for tracked
state, while still allowing the beautiful node-link form where it belongs
(on-demand). The topological order already linearizes the DAG into columns, so we
rarely *need* true graph drawing for the tracked surface anyway.

## 10. Provisional arc breakdown (dependency-ordered)

Ordered by dependency, not by estimated size — dogfooding odm's own thesis. Sizes
are deliberately *not* story-pointed; instead each arc names the observed-complexity
signals we'd expect (per §5), to be confirmed against the telemetry once it exists.

| Arc (provisional) | Deliverable | `depends_on` | Notes / expected complexity |
|---|---|---|---|
| **A7 — Two-clock telemetry** | Derive the event log from gates+git; the two-clock model (active-work vs inter-slice latency); the language-agnostic covariate collection layer (§5) that **computes & annotates** covariates onto nodes (`gix` churn, ledger counts, gate-log dates); **structured CDC-verification emission** (§5b — counts/enums, not prose); the status×gate **evidence matrix** table view; minimal metadata additions (§6). | A2 (gates+evidence+DAG), A3 (rollup/orient surface) | *consumes* richer evidence from A5 when present (soft). Derivation + the collection layer + structured CDC output + 1–2 schema fields + one table renderer → **modest-plus**. The foundation; forecasting is worthless without it. |
| **A8 — Forecasting** | Reference-class bucketing on observed complexity; empirical per-node distributions; **Monte-Carlo PERT over the derived DAG**; the two-output forecast (active effort / calendar) with honest error bars; export projection of the forecast. | A7, A2 (the DAG) | Carries the statistical modelling + Monte-Carlo + uncertainty calibration → likely the **heaviest**; a **research-gate** is warranted before code (as `odm-index` got ODD-0014). |
| **(viz/export tie-in)** | `odm viz` / `odm export` rendering of the matrix (table) and DAG (`ascii-dag`); forecast tables/exports. | A3 + the ODD-0017 export arc | Pure renderers over rollup+telemetry; slots alongside the interop export arc, not a separate engine. |

MVP of *this thread* = A7 + A8. A7 is independently valuable on its own (honest
flow-efficiency and cycle-time telemetry, no forecasting required) — a good
demoable waypoint and a natural self-hosting dogfood once arcs 1–6 land.

## 11. Open questions

- ~~**Active-start rule.**~~ **Decided 2026-06-24:** first commit on the slice's
  branch; gates give phase boundaries, git gives precision (§6).
- **Post-merge branch resolution.** `branch:` won't resolve after merge + delete —
  derive-and-cache at close, or use first-parent merge ranges? (An A7 detail.)
- **Reference-class definition.** Which covariates (§5) actually predict, and how to
  bucket with thin data? (An empirical question A7's telemetry answers before A8.)
- **Covariate pre-registration & regularization.** Which leading covariates do we
  commit to up front (the §5 short-list?) and what keeps small-N from overfitting —
  regularization, a hard cap on covariate count, recency-weighting?
- **Structured CDC-verification schema (§5b).** The exact machine-readable fields
  the verification must emit (`iterations_used`, rows-by-disposition,
  `deviations_flagged`, `uncertainties_named`, evidence-at-close, verdict) — and
  which odm computes vs which the CDC asserts. Feeds the A6 PM-skill template.
- **Latency model.** Is inter-slice latency forecastable at all, or effectively a
  wide prior set by human cadence? Possibly a renewal process; possibly just
  reported as a historical gap distribution with no pretence of prediction.
- ~~**Distribution fitting under small N.**~~ **Researched 2026-06-24**
  (ODD-0018): Bayesian hierarchical pooling + censoring-corrected
  lognormal-core + adaptive/weighted conformal with finite-sample correction;
  bootstrap under-covers below N≈10; ~20+ completed slices before MC stabilizes.
- ~~**Research ODD?**~~ **Done** — graduated to **ODD-0018** (Draft), the A8
  research gate.
- **Two-clock validation (open — ours to test).** No evidence that forecasting
  active vs wait separately beats forecasting total cycle time directly. Build the
  total-cycle Monte-Carlo as the control; test the two-clock split against it on our
  own held-out slices.
- **Self-host milestone.** Does A7 become the first post-MVP capability odm dogfoods
  on its *own* build history (which, by then, is a rich event log of arcs 1–6)?
