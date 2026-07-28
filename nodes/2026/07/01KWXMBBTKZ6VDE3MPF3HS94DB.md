---
id: 01KWXMBBTKZ6VDE3MPF3HS94DB
number: 1504
type: slice
schema: slice/v1.1
name: drift in `rollup` / `orient`
created: 2026-07-01
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc05-reconciliation/slice04-drift-in-rollup-orient/slice-doc.md
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
# Slice 04 (Arc 05): drift in `rollup` / `orient`

> Plan-of-record for A5 slice04. slices 01–03 built the model, both probes, the runner, and
> the standalone `odm reconcile`. This slice **retires the A3 placeholder**: `rollup` and
> `orient` stop printing "not yet tracked (A5)" and render **real drift** from an on-demand
> reconcile. No `affects`/deferred/scheduled (slices 05–07).

## Goal

Fold drift into the two generated views a user actually reads day-to-day. After this slice,
`odm orient` and `ROLLUP.md` show what has diverged from declared desired state — the A3
`Drift {}` slot (Q-A3-2) carries real data, and "no drift" is shown honestly when clean.

## What we're replacing (the A3 placeholder)

- `odm_core::rollup::Drift {}` — an **empty** `#[non_exhaustive]` struct in the `Rollup`
  model (`rollup.rs:155`); assemble produces it empty; the renderer prints "not yet tracked
  (A5)".
- `orient` drift section (#5, `orient.rs:265–267`) prints "not yet tracked (A5)".

## The layering constraint that shapes this slice

`Rollup::assemble(nodes, gates, threshold)` is a **pure function** and `odm-core` must **not**
depend on `odm-reconcile` (the dependency runs the other way). Probes are I/O/side-effecting.
Therefore:

- **`odm-core` owns the `Drift` *shape*** — plain data (a projection): counts + per-node
  drifted/errored entries carrying identity (node number/name, `fact_id`, `describe`) and
  `expected`/`observed`. No `odm-reconcile` types leak into `odm-core`.
- **`odm-cli` computes drift** — it depends on both, runs `Runner::run_corpus` (store-read),
  projects the `CorpusReport` → `odm_core::Drift`, and injects it into the model. A shared
  odm-cli helper does this once for **both** `rollup` and `orient`.

**Second constraint — drift cannot come from the index.** The `rollup`/`orient` *model* is
index-backed (A4/slice06), but the index deliberately does **not** carry `desired_facts`
(slice02). So drift **must** be computed from a separate **store-read** reconcile
(`run_corpus`), not from the index-backed frontmatters (which have empty `desired_facts`).
This is the settled resolution (arc-plan v1.5/v1.6): **rollup/orient run an on-demand
reconcile and fold the result in; drift is never cached in the index.** Drift is a *derived,
time-varying* fact — it belongs to reconcile, not to index state (which mirrors file state).
The A4 adapter-fidelity invariant stays **honored by non-triggering**.

## Resolving the slice03 render-identity seam

slice03 flagged a double-load: `run_corpus` loads docs to run probes, then reconcile re-loads
to join node number/name + fact describe (the report carried **ids only**). Fix here:
**enrich the runner's report** (`NodeReport`/`FactResult`) to carry render-identity, computed
**once** inside `run_corpus` from the frontmatters it already loads. Then `reconcile`,
`rollup`, and `orient` all render from the enriched report with **no second load**. (This
also lets `reconcile`'s slice03 double-load be dropped — optional cleanup, flag if done.)

## Scope — in

1. **`Drift` model** (`odm-core`): flesh the empty `Drift {}` into the data projection above
   (counts + drifted/errored entries with identity + expected/observed). Plain data; still
   additive-friendly. `odm-core` gains **no** `odm-reconcile` dependency.
2. **Report identity enrichment** (`odm-reconcile`): `run_corpus`'s report carries node
   number/name + fact `describe`, joined once from the loaded frontmatters.
3. **`odm rollup`** (`odm-cli`): run the on-demand reconcile, project → `Drift`, inject into
   the `Rollup` model; the drift section renders real drift; **clean → "no drift"** (no
   fabricated data). The A3 placeholder string is gone.
4. **`odm orient`** (`odm-cli`): the drift section renders real drift (same shared helper);
   clean → no drift; placeholder gone.
5. **`--json`** (`rollup/v1` + `orient/v1`): the previously-empty `drift` slot now carries
   the projection — **additive** (no version bump; the slot already existed, now populated).
6. A **shared odm-cli drift helper** used by both commands (factor the compute+project once).

## Scope — out (named, not dropped)

- **`affects` / stale-doc check** — **slice05**; **deferred surfacing** — **slice06**;
  **scheduled reconcile** — **slice07**.
- **Caching drift in the index** — **declined** (settled): drift is on-demand, store-read.
- **The `assemble` signature** — CC's call how drift enters the model (a param on `assemble`,
  or a compose step that sets `rollup.drift`); recommend construct-complete (pass it in)
  over post-mutation. Flag if it forces a wider change.

## Verification approach

odm-cli integration tests (seed nodes with facts, diverge reality, run the command) + an
odm-core unit test on the `Drift` projection:

- rollup: a drifted fact appears in the drift section with identity + expected/observed; a
  clean corpus → "no drift"; the "not yet tracked (A5)" string is gone.
- orient: same, in its drift section.
- `--json`: `rollup/v1` + `orient/v1` carry the populated drift projection (additive).
- the reconcile report carries identity (no second load); the shared helper feeds both.
- drift is store-read/on-demand — no `desired_facts`/drift in `odm-index` (grep).
- clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

Cargo rows are CC-`attested`, reproduced via CI / local 1.85+.

## Exit criteria

Ledger S-1…S-7 reach a final status: the `Drift` model carries a real projection (odm-core,
no odm-reconcile dep); the reconcile report carries render-identity (slice03 double-load
resolved); `rollup` and `orient` render real drift (clean → no drift; placeholder gone);
`--json` is additive; drift is on-demand store-read, never index-cached; gates pass.

> **Note — CLAUDE.md render convention (from slice03):** odm-cli renders with `writeln!` +
> `tabled` (no `oxur-cli` dep). Use that; do **not** reach for `oxur_cli` output helpers.
> (The stale `CLAUDE.md` line is queued for the arc-close doc-keeping fix.)
