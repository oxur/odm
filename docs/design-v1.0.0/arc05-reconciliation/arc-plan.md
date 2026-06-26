# Arc 05 — Reconciliation (plan-of-record)

> Refs: ODD-0013 §5.2 (desired-state facts + probes) + §4.4 (the `affects` edge /
> evidence-leveled satisfaction) + §3 (the `affects` edge); ODD-0015 A5 row + §5
> (E5 deferred re-entry, C5 stale-doc); ODD-0001 C2 (the prod-DB 503), C5;
> `project-plan.md` §2; Arc 03 `arc-plan.md` Q-A3-1 + Q-A3-2 (this arc cashes both).
> `depends_on:` A2 (the gate/evidence model) + A3 (the rollup/orient views that drift
> and deferred surface into).
>
> **Status:** planned, not started. Slice breakdown at one-line altitude; per-slice doc
> sets written when the arc becomes active.

## Capability

Drift detection for plans — the **marquee state-drift killer** (ODD-0001 C2, the
prod-DB 503). Nodes declare `desired_facts`; a pluggable **`Probe`** trait diffs
*declared* desired state against *observed reality* and reports **drift**, on demand
and on a schedule. Reconciliation is honest **only about tracked facts** (Terraform's
lesson, lifted to plan state), so the tool nudges enumerating integration- and
program-level facts. Drift folds into the generated rollup/orient — **replacing the A3
"not yet tracked (A5)" placeholder** (Q-A3-2) — and this arc is also where the
**`affects` edge + stale-doc-vs-decision check** (C5) and **deferred-node surfacing +
re-entry predicate** (Q-A3-1, deferred from A3) land. The `odm-reconcile` crate.

## Exit criteria (arc acceptance)

- A node can declare `desired_facts`; `odm reconcile` runs their probes and reports
  drift (declared-vs-observed), with a non-zero/flagged result when reality diverges.
- The first probe impls exist: a **shell** probe (run a command, compare exit/stdout)
  and a **file** probe (the legacy checksum/mtime detector repurposed).
- Drift surfaces in `rollup`/`orient` — the A3 placeholder is gone, replaced by real
  tracked-fact drift (and "no drift" when clean), with no fabricated data.
- The `affects` edge powers a stale-doc-vs-committed-decision check folded into `check`.
- **Deferred nodes are surfaced with their checkable re-entry predicate** — the Q-A3-1
  deferral is cashed (representation + surfacing + predicate evaluation).
- `reconcile --schedule` supports recurring drift checks.

## Slices (dependency-ordered, one-line scope)

1. **slice01 — `desired_facts` schema + `Probe` trait.** Frontmatter `desired_facts`
   (id, describe, probe spec); the `Probe` trait + result model; the **shell** probe as
   first impl. — `odm-core` / `odm-reconcile`.
2. **slice02 — file probe + probe execution.** The legacy checksum/size/mtime detector
   repurposed as a `file` probe; probe-runner that executes a node's facts and collects
   results.
3. **slice03 — `odm reconcile` (on demand).** Diff declared vs observed across the
   corpus; report drift (human + `--json`, per the slice04-A3 schema convention); exit
   codes / severities consistent with `check`.
4. **slice04 — drift in rollup/orient.** Replace the A3 "not yet tracked (A5)"
   placeholder with real drift in the `Rollup` model + the orient view; "no drift" when
   clean (no fabricated data).
5. **slice05 — `affects` edge + stale-doc-vs-decision check (C5).** A decision/doc node
   `affects` the docs it touches; `check` flags a doc that contradicts a committed
   decision.
6. **slice06 — deferred surfacing + re-entry predicate (Q-A3-1).** A `deferred`
   representation carrying a checkable re-entry condition (a `desired_fact`/probe);
   surfaced in rollup/orient; predicate evaluated by the reconciler. Fills the
   defined-but-empty A3 slot.
7. **slice07 — scheduled reconcile.** `reconcile --schedule` for recurring drift checks;
   drift folded into the rollup on a cadence.

## Arc Ledger

> Per LEDGER-DISCIPLINE v2.0 §B (Option A: opens here, closes in the companion
> `closing-report.md`). Class-(b) composition rows stated up front from the capability;
> class-(a) slice-closed and class-(c) bubble-up rows accrue as slices close. **Class-(b)
> rows are reproduced at arc scale — an end-to-end demonstration, never inherited.**

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| A-1 | slice01 (desired_facts + Probe trait + shell probe) closed | ptr: slice01 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-2 | slice02 (file probe + probe execution) closed | ptr: slice02 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-3 | slice03 (`odm reconcile` on demand) closed | ptr: slice03 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-4 | slice04 (drift in rollup/orient) closed | ptr: slice04 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-5 | slice05 (`affects` edge + stale-doc check) closed | ptr: slice05 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-6 | slice06 (deferred surfacing + re-entry predicate) closed | ptr: slice06 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-7 | slice07 (scheduled reconcile) closed | ptr: slice07 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-8 | **Compose:** a declared `desired_fact` whose reality diverges is detected and reported as drift by `odm reconcile` | arc-scale demo: declare a fact, diverge reality, observe drift | serious | arc-plan / 0001-C2 | open | | reproduce at arc scale |
| A-9 | **Compose:** both probe kinds work end-to-end — a **shell** probe and a **file** probe | arc-scale demo: one of each, exercised | serious | arc-plan | open | | reproduce at arc scale |
| A-10 | **Compose:** drift surfaces in `rollup`/`orient` — the A3 "not yet tracked (A5)" placeholder is gone, replaced by real drift (and "no drift" when clean) | arc-scale demo: rollup/orient before vs. after a divergence | serious | arc-plan / Q-A3-2 | open | | reproduce at arc scale |
| A-11 | **Compose:** the `affects` edge powers a stale-doc-vs-committed-decision finding in `check` | arc-scale demo: a doc contradicting a committed decision → flagged | serious | arc-plan / 0001-C5 | open | | reproduce at arc scale |
| A-12 | **Compose:** deferred nodes are surfaced with a checkable re-entry predicate (the Q-A3-1 deferral cashed) | arc-scale demo: a deferred node + its predicate surfaced in rollup/orient | serious | arc-plan / Q-A3-1 | open | | reproduce at arc scale |
| A-13 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | open | | accrues as slices close |

Closes in `arc05-reconciliation/closing-report.md`: per-row walk + composition verdict,
independently gated. A failed class-(b) row spawns a **remediation slice**, not a re-pass.

## Dependencies

Consumes: A2's gate/evidence + satisfaction model, A3's rollup model + orient view +
the deferred/drift slots left defined-but-empty. Leaves for later: A6's self-host can
run reconcile on odm's own corpus once it lands.

## Open design questions (resolve in slice docs)

- **Probe safety/sandboxing.** A shell probe runs arbitrary commands — scope the trust
  model (probes are author-declared, run locally) and whether any guardrails are needed.
- **Schedule mechanism.** In-tool scheduler vs. emit-for-cron/CI — decide in slice07;
  lean toward the latter (files-are-the-source ethos, no daemon).
- **Program-level acceptance facts.** ODD-0015 A5 calls for program-level acceptance
  (the MVP DoD as tracked facts) — fold into slice01's `desired_facts` model or a thin
  layer above it; settle when slice01 is planned.
- **Deferred representation.** The exact schema for `deferred` (status variant vs.
  marker + predicate) is the Q-A3-1 question A3 deliberately left open until "the
  schema/metadata firms up" — settle it in slice06, not before.

## Method

Ledger per slice; CC implements, CDC verifies; cargo rows via CI / local 1.85+;
five-iteration cap. Slice closes bubble up to this arc-plan; the arc closes with its own
`closing-report.md` + composition check.

## Version History

### v1.1 — 2026-06-26
Added the **`## Arc Ledger`** section per LEDGER-DISCIPLINE v2.0 §B (the arc ledger opens
with the arc-plan, which already exists). Pure addition — the v1.0 body is unchanged.
Surfaced by: the ledger-discipline upgrade (v1→v2.0), not a slice bubble-up.

### v1.0 — 2026-06-26
Initial arc-plan, drafted from ODD-0013 §5.2/§4.4, the ODD-0015 A5 row, and the
Arc 03 deferrals (Q-A3-1 deferred-surfacing, Q-A3-2 drift placeholder) that this arc
cashes. No slices started; one-line altitude per *plan late, plan deep*.
