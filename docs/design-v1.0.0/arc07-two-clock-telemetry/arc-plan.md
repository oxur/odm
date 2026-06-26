# Arc 07 — Two-clock telemetry (plan-of-record)

> Refs: **ODD-0018** (forecasting research gate — what A7 must collect for A8);
> `workbench/forecasting-telemetry.md` (the method — §5 covariates, §5b CDC
> instrumentation, §6 schema); ODD-0013 (node/gate/edge schema); `project-plan.md`
> §4. `depends_on:` **A2** (gates + evidence + DAG) + **A3** (rollup/orient surface
> to render into); *matures with* **A5** (richer reconciled evidence), but does not
> require it.
>
> **Status:** scoped, not started — **post-MVP extension arc, design-ahead.** A7
> builds after A1–A6 land and odm self-hosts (per arc06's close note). Slice
> breakdown at one-line altitude; per-slice doc sets written when the arc becomes
> active. Drafted ahead of the normal sequencing at operator request (project-plan
> v1.2).
>
> **The measurement layer, not forecasting.** A7 turns the data odm already produces
> (gates, the evidence event log, git, the ledger / five-doc set) into a derived,
> queryable telemetry stream + a first descriptive view. Forecasting is **A8** (gated
> by ODD-0018). A7 is independently valuable: real flow-efficiency / cycle-time /
> rework telemetry with zero forecasting — a clean demoable waypoint and the first
> self-host dogfood (odm measuring its own arc-1–6 history).

## Capability

Build the **two-clock telemetry substrate**: derive a per-node/per-slice **event
log** from gate transitions (`reached`/`by`/`evidence` + `evidence_dates`) and git
(commit timestamps, diff stats via `gix`); decompose elapsed time into the two
clocks — **active-work** (within a slice) vs **inter-slice latency** (human-gated
waiting) — with the plan/implement/verify phase split and right-censoring for
in-progress slices; **compute and annotate** a set of **language-agnostic**
covariates onto nodes (git + graph + ledger + gate-log + calendar; *no* code-AST /
language-specific metrics); make the **CDC verification** emit *structured*
machine-readable outcomes rather than only prose; and surface it all as the
status×gate **evidence matrix** plus descriptive telemetry (flow-efficiency,
cycle-time distribution, rework, drift) in `orient`/rollup. Everything is **metadata,
never a target; team/process, never per-actor** (Goodhart). Proposed new crate:
**`odm-telemetry`** (derived analytics over `odm-store` + `odm-core` + `odm-index` +
`gix`).

## Exit criteria (arc acceptance)

- A node resolves to its implementation commits (the `branch` link), and the two
  recorded fields (`branch`, `iterations`) round-trip in frontmatter (back-compat,
  omit-when-empty). `evidence_dates` already landed (arc02 slice05.1).
- `odm` emits a node's **event log** (gate + commit events, time-ordered) as `--json`
  — derived, rebuildable, no new source of truth.
- The **two clocks** are computed per slice — active-work span, inter-slice latency,
  the plan/implement/verify phase split — with open slices flagged right-censored;
  corpus-level distributions available.
- The CDC verification emits a **structured record** (`iterations_used`,
  rows-by-disposition, `deviations_flagged`, `uncertainties_named`, evidence-at-close,
  verdict); the computable fields are odm-derived, not hand-keyed.
- The **language-agnostic covariate set** (forecasting-telemetry §5) is computed and
  annotated onto nodes, tagged leading vs lagging; pre-registered short-list flagged;
  no per-actor scoring exists.
- The **evidence matrix** renders (oxur-cli table, copy-pasteable) and a descriptive
  telemetry summary appears in `orient`/rollup — **descriptive only, no forecasts**.
- Pointed at odm's *own* arc-1–6 history, the telemetry produces a sane first
  reference corpus (the self-host dogfood).

## Slices (dependency-ordered, one-line scope)

1. **slice01 — telemetry schema + slice↔git linkage.** The keystone: add `branch`
   (auto-set when work starts) + `iterations` (promoted from the CDC count); resolve
   node → commit set, with a post-merge fallback (first-parent range or cache-at-close).
   — `odm-telemetry` / `odm-core`.
2. **slice02 — event-log derivation (read model).** Merge gate transitions +
   `evidence_dates` with git events (timestamps, diff stats via `gix`), keyed by
   `branch`; emit time-ordered, `--json`; pure derivation.
3. **slice03 — structured CDC-verification emission.** Make the verification a
   machine-readable record (counts/enums, not only prose); odm computes the computable
   fields, captures the asserted ones. Feeds slice05's lagging covariates; seeds the
   A6 PM-skill template.
4. **slice04 — two-clock decomposition.** From the event log: active-work span
   (first→last branch commit), inter-slice latency (gaps), plan/implement/verify phase
   split (gate actor + timestamps), right-censoring flag; corpus distributions.
5. **slice05 — covariate collection & annotation.** Compute + annotate the §5
   language-agnostic covariates (git: churn/files/scatter/fixups/hotspots/entropy;
   graph: edge degree, **soft-satisfied-deps-at-start**, tears, topo position; ledger:
   row count, significance mix; gate-log: iterations, verification latency,
   evidence-at-close; calendar: elapsed-since-close, burst/drought); leading vs
   lagging tagged; never per-actor.
6. **slice06 — evidence matrix + telemetry surfacing.** Render the status×gate
   evidence matrix (oxur-cli table) + flow-efficiency / cycle-time / rework / drift
   summaries in `orient`/rollup; terminal + `--json`/markdown export. Descriptive only.

## Arc Ledger

> Per LEDGER-DISCIPLINE v2.0 §B (Option A: the arc ledger lives here in `arc-plan.md`
> and closes in the companion `closing-report.md`). Opens with the class-(b)
> composition rows stated up front from the capability; accrues class-(a) slice-closed
> rows and class-(c) bubble-up rows as slices close. **Class-(b) rows are reproduced at
> the arc scale — an end-to-end demonstration, never inherited from the slices.**
> Design-ahead: all rows `open` until the arc is built.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| A-1 | slice01 (telemetry schema + slice↔git linkage) closed | ptr: slice01 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-2 | slice02 (event-log derivation) closed | ptr: slice02 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-3 | slice03 (structured CDC-verification emission) closed | ptr: slice03 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-4 | slice04 (two-clock decomposition) closed | ptr: slice04 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-5 | slice05 (covariate collection & annotation) closed | ptr: slice05 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-6 | slice06 (evidence matrix + telemetry surfacing) closed | ptr: slice06 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-7 | **Compose:** a node resolves to its implementation commits (the `branch` link); `branch`/`iterations` round-trip (back-compat, omit-when-empty) | arc-scale demo: annotate a node, list its commits; round-trip a pre-field node | serious | arc-plan | open | | reproduce at arc scale |
| A-8 | **Compose:** `odm` emits a node's time-ordered event log (gate + commit events) as `--json`, derived and rebuildable | arc-scale demo: event log over a real slice | serious | arc-plan | open | | reproduce at arc scale |
| A-9 | **Compose:** the two clocks compute per slice — active-work span, inter-slice latency, plan/implement/verify phases — with open slices right-censored | arc-scale demo: clocks over the arc1–6 corpus | serious | arc-plan | open | | reproduce at arc scale |
| A-10 | **Compose:** the CDC verification emits a structured record (`iterations_used`, rows-by-disposition, deviations/uncertainties counts, evidence-at-close, verdict); computable fields odm-derived | arc-scale demo: a structured verification record | serious | arc-plan / forecasting-telemetry §5b | open | | reproduce at arc scale |
| A-11 | **Compose:** the language-agnostic covariate set is computed + annotated, leading/lagging tagged; **no per-actor scoring exists** (Goodhart) | arc-scale demo: covariate record per node; audit → no per-actor score | serious | arc-plan / ODD-0018 | open | | reproduce at arc scale |
| A-12 | **Compose:** the status×gate evidence matrix renders (copy-pasteable) + descriptive telemetry in `orient`/rollup — **no forecasts** | arc-scale demo: render the matrix + telemetry summary | serious | arc-plan | open | | reproduce at arc scale |
| A-13 | **Compose:** pointed at odm's own arc1–6 history, the telemetry yields a sane first reference corpus (the self-host dogfood) | arc-scale demo: telemetry over odm's own build | serious | arc-plan | open | | reproduce at arc scale |
| A-14 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | open | | accrues as slices close |

## Dependencies

Consumes: **A2**'s gate/evidence model + DAG + `evidence_dates`; **A3**'s rollup/orient
surface to render into; the store + `gix` (commit timestamps, diff stats); the ledger /
five-doc set (row counts, the structured CDC record). *Matures with* **A5** (reconciled
evidence enriches the lagging covariates) but does not require it. Leaves for **A8**:
the complete event log + covariate records + the two clocks — A8 forecasts over them
(ODD-0018). Independent of A4 (telemetry does not require the index, though it will read
it once present).

## Open design questions (resolve in slice docs)

- **`branch` auto-set trigger** (slice01): on first `set-gate` to an active gate? on an
  explicit `start`? on branch creation? Affects every active-span measurement.
- **Post-merge branch resolution** (slice01): cache-at-close vs first-parent merge range.
- **Structured CDC schema** (slice03): exact field set + which odm computes vs the CDC
  asserts — this is also the A6 PM-skill template's spine.
- **Pre-registered covariate short-list** (slice05): commit forecasting-telemetry §5's
  short-list up front; everything else earns its way in from data (ODD-0018 §1.7,
  small-N overfitting guardrail).
- **Crate placement**: `odm-telemetry` (proposed) vs fold into `odm-index` — decide at
  slice01.

## Method

Ledger per slice; CC implements, CDC verifies; cargo rows via CI / local 1.85+;
five-iteration cap. Slice closes bubble up to this arc-plan; the arc closes with its own
`closing-report.md` + composition check. **On arc close, odm measures its own build** —
and **A8 (forecasting)** becomes buildable on the collected substrate.

## Version History

### v1.1 — 2026-06-26
Added the **`## Arc Ledger`** section per LEDGER-DISCIPLINE v2.0 §B (class-(a)
slice-closed, class-(b) composition, class-(c) bubble-up rows), matching the arc04–06
ledgers. Pure addition — the v1.0 body is unchanged. Surfaced by: the ledger-discipline
upgrade (v1→v2.0) now applied to the extension arcs.

### v1.0 — 2026-06-26
Initial arc-plan, distilled to one-line altitude from
`workbench/a7-telemetry-arc-breakdown.md` and ODD-0018, as part of the post-arc6
project-plan extension (project-plan v1.2). No slices started; design-ahead per *plan
late, plan deep* — detail lands when the arc becomes active.
