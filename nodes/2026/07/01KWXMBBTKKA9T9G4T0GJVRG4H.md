---
id: 01KWXMBBTKKA9T9G4T0GJVRG4H
number: 1503
type: slice
schema: slice/v1.1
name: '`odm reconcile` (on demand)'
created: 2026-06-30
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice03-reconcile-command/slice-doc.md
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
# Slice 03 (Arc 05): `odm reconcile` (on demand)

> Plan-of-record for A5 slice03. slice01 landed the model + trait + shell probe; slice02
> added the `file` probe + the probe-runner (`CorpusReport` / `OutcomeCounts`, store-read).
> This slice puts a **command** on top: `odm reconcile` runs the corpus runner and reports
> drift — human + `--json` (`reconcile/v1`), with severities + exit codes consistent with
> `check`. No rollup/orient wiring, no scheduling (slices 04 / 07).

## Goal

`odm reconcile` makes drift *visible and actionable* from the CLI: run every node's
`desired_facts`, render what diverged, and exit with a code a CI gate can act on. This is the
user-facing face of the marquee state-drift killer (ODD-0001 C2).

## Scope — in

1. **The `odm reconcile` command** (`odm-cli`): invoke `odm_reconcile::Runner::run_corpus`
   (the slice02 store-read runner), render the `CorpusReport`. Output to stdout (a *report*,
   not a generated file — `ROLLUP.md` is rollup's job; drift-in-rollup is slice04).
2. **Human output**: per **drifted** fact — node (number + name), `fact_id`, `describe`,
   and `expected` vs `observed`; per **probe-error** fact — the same identity + the `reason`,
   rendered at a *distinct* severity. A clean corpus prints a plain "no drift" line — **no
   fabricated data** (the rollup ethos). Use `oxur_cli::common::output::{success, error,
   warning, info}` + `oxur_cli::table` (CLAUDE.md conventions).
3. **`--json`**: a 1:1 `Serialize` projection of the report under a **`reconcile/v1`** schema
   marker (consistent with A3's `check/v1` / `rollup/v1` / `orient/v1`). This requires
   `ProbeOutcome` + the report types to gain `Serialize` — **deliberately omitted in slice02
   (no wire contract yet); added here**, additive-stable (the serde-evolution caution:
   explicit, always-serialized fields). Proposed shape:
   `{ schema:"reconcile/v1", counts:{holds,drifted,errored}, nodes:[{node_id, number, name,
   results:[{fact_id, describe, outcome:{kind:"holds"|"drifted"|"error", …}}]}] }`.
4. **Severity + exit-code mapping** — a **pure function of `OutcomeCounts`**, consistent with
   `check`:
   - all holds (or no facts) → **clean**, exit `0`, "no drift".
   - any **Drifted** fact → **Error** severity, exit **non-zero** (the finding reconcile
     exists for).
   - any **probe Error** ("couldn't check") → **Warning** severity, **always surfaced**;
     exit non-zero **only under `--strict`** (mirrors `check`'s Error/Warning + `--strict`).

## The one design decision to ratify — how loud is "couldn't check"?

The three-way outcome we protected since slice01 (`Holds`/`Drifted`/`Error`) forces an honest
question at the exit code: **a probe `Error` means we could not confirm reality — should that
fail a reconcile by default, or only under `--strict`?**

**Decision (CDC, recommended): `check`-consistent — drift is an Error (fails), a probe-error
is a Warning (surfaced always; fails only under `--strict`).** Rationale: it keeps the
slice02 distinction meaningful at the exit code (confirmed-diverged ≠ couldn't-check), reuses
the `check` mental model the user already has, and lets a CI gate opt into "couldn't-check is
also a failure" with `--strict`. **Ratification point (Duncan / CC may override):** in some
ops contexts "couldn't reach the prod DB to check" is *as* alarming as confirmed drift —
if you want couldn't-check to fail by default, say so and the default flips (the severity
distinction in the output stays either way). I recommend the `--strict`-gated default for
consistency; flag if you disagree.

## Scope — out (named, not dropped)

- **Drift in `rollup`/`orient`** (replace the A3 "not yet tracked (A5)" placeholder) —
  **slice04**. Note the carried slice04 open question (arc-plan v1.5): the rollup is a hot
  *index* view but reconcile reads the *store*; slice04's resolution is that **rollup runs an
  on-demand corpus reconcile and folds the result in — it does NOT cache drift in the
  index** (keeps drift truth in reconcile; keeps the A4 invariant un-triggered).
- **Scheduled reconcile** (`--schedule`) — **slice07**.
- **`affects` / stale-doc check** — **slice05**; **deferred surfacing** — **slice06**.

## Invariant note (A4 adapter-fidelity)

slice03 adds **no new index reader**: the command calls `Runner::run_corpus` (store-read,
from slice02). The carried adapter-fidelity invariant therefore stays **honored by
non-triggering** — no `IndexRecord`/adapter/`FORMAT_VERSION` change. (No dedicated ledger row
needed this slice; recorded here so the non-trigger is explicit, not assumed.)

## Verification approach

Integration tests against a temp store (the established `odm-cli` test pattern: seed nodes,
run the command, assert stdout + exit code), plus a unit test on the counts→exit/severity
mapping:

- clean corpus → "no drift", exit 0; drifted fact → reported with identity + expected/observed,
  exit non-zero; probe-error → surfaced distinctly, exit 0 without `--strict` and non-zero with.
- `--json` validates against `reconcile/v1` (shape + marker); 1:1 with the human model.
- the mapping is a pure function of `OutcomeCounts`.
- clippy `-D warnings`; no `unsafe`; coverage ≥ 90%.

Cargo rows are CC-`attested` and route to CI / a local 1.85+ run.

## Exit criteria

Ledger rows R-1…R-6 reach a final status: `odm reconcile` reports drift (clean → no-drift
exit 0; drift → non-zero; probe-error → surfaced, `--strict`-gated); `--json` emits
`reconcile/v1` (with `Serialize` added additively); the exit/severity map is a pure function
of `OutcomeCounts` consistent with `check`; gates pass.
