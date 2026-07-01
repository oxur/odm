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
| A-1 | slice01 (desired_facts + Probe trait + shell probe) closed | ptr: slice01 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report (`21cfbf1`; 7/7; cov odm-reconcile 96% / odm-core desired.rs 100%); CDC-verified on structure (`slice01-desired-facts-probe/cdc-verification.md`); three-way `ProbeOutcome` + exec-directly shell probe; cargo rows pending CI. | → `done` when slice01 reproduces (CI green). |
| A-2 | slice02 (file probe + probe execution) closed | ptr: slice02 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report (`e4ca702`; 7/7; cov odm-reconcile file.rs 97% / runner.rs 97% / shell.rs 100%); `file` probe (exists/sha256/size, `sha2` reuse) + per-node/per-corpus runner with read-through; reads facts from the **store, not the index** (G-5 grep clean — invariant honored by non-triggering); drift/error kept distinct in `OutcomeCounts`; cargo rows pending CI. Branched off `arc05-slice01-…` (slice01 unmerged); rebase on merge. | → `done` when slice02 reproduces (CI green). |
| A-3 | slice03 (`odm reconcile` on demand) closed | ptr: slice03 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report (`266fc38`; 6/6; cov odm-cli reconcile.rs 97%); `odm reconcile` renders drift (clean→no-drift exit 0; drift→exit 1; probe-error→Warning, `--strict`-gated) with `check`-consistent severity via a pure `verdict(OutcomeCounts)`; `--json` `reconcile/v1` (added `Serialize` additively, `ProbeOutcome` tagged on `kind`); store-read `run_corpus`, **no** index reader (invariant un-triggered); cargo rows pending CI. Branched off `arc05-slice02-…`; rebase on merge. | → `done` when slice03 reproduces (CI green). |
| A-4 | slice04 (drift in rollup/orient) closed | ptr: slice04 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report (`DRIFT04`; 7/7; cov touched paths ≥ 94% line); `odm_core::rollup::Drift` fleshed (plain data, no reconcile dep) + `Rollup::with_drift`; reconcile report enriched with identity (slice03 double-load resolved); `rollup`/`orient` render real drift via one shared `compute_drift` projector (store-read `run_corpus`, no index change); `--json` `drift` slot populated additively (no version bump); the "not yet tracked (A5)" placeholder is gone. cargo rows pending CI. Branched off `arc05-slice03-…`; rebase on merge. | → `done` when slice04 reproduces (CI green). |
| A-5 | slice05 (`affects` edge + stale-doc check) closed | ptr: slice05 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-6 | slice06 (deferred surfacing + re-entry predicate) closed | ptr: slice06 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-7 | slice07 (scheduled reconcile) closed | ptr: slice07 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-8 | **Compose:** a declared `desired_fact` whose reality diverges is detected and reported as drift by `odm reconcile` | arc-scale demo: declare a fact, diverge reality, observe drift | serious | arc-plan / 0001-C2 | open | | reproduce at arc scale |
| A-9 | **Compose:** both probe kinds work end-to-end — a **shell** probe and a **file** probe | arc-scale demo: one of each, exercised | serious | arc-plan | open | | reproduce at arc scale |
| A-10 | **Compose:** drift surfaces in `rollup`/`orient` — the A3 "not yet tracked (A5)" placeholder is gone, replaced by real drift (and "no drift" when clean) | arc-scale demo: rollup/orient before vs. after a divergence | serious | arc-plan / Q-A3-2 | open | mechanism-complete (slice04, `DRIFT04`): placeholder gone in both views; real drift + honest "no drift"; additive JSON. To **reproduce at arc scale** at arc-close (never inherited). | reproduce at arc scale |
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

- **Probe safety/sandboxing.** ✅ **Resolved (slice01).** Probes are **author-declared and
  run locally with the user's own privileges** — the same trust model as a git hook,
  `make`, or `build.rs` in your own repo. The MVP adds **no sandbox**; the trust boundary
  is "you ran a command written in your own node files," documented explicitly in code +
  the slice-doc. Guardrails considered and deferred (a `--no-exec`/dry-run, an allowlist)
  to an untrusted-/multi-author-corpus scenario, which is out of MVP scope.
- **Schedule mechanism.** In-tool scheduler vs. emit-for-cron/CI — decide in slice07;
  lean toward the latter (files-are-the-source ethos, no daemon).
- **Program-level acceptance facts.** ✅ **Resolved (slice01): no separate layer.**
  Program-/arc-level acceptance facts are just `desired_facts` declared on the project (or
  arc) node — the `desired_facts` model is uniform across node types. One mechanism; the
  MVP DoD can be encoded as `desired_facts` on the project node, which `reconcile` then
  checks (a dogfooding hook for A6).
- **Deferred representation.** The exact schema for `deferred` (status variant vs.
  marker + predicate) is the Q-A3-1 question A3 deliberately left open until "the
  schema/metadata firms up" — settle it in slice06, not before. (Note: a `Deferred` model
  + rollup slot **already exist** in `odm-core/rollup.rs` — slice06 fills, not greenfields.)
- **Index integration of `desired_facts` (the carried invariant's trigger).** Whether
  `reconcile` reads `desired_facts` *via the index* (consistent with A4, → IndexRecord +
  adapter + fidelity test + `FORMAT_VERSION` bump) or directly from the store is a
  **slice02/03 decision** (the probe-runner / `reconcile` are the first readers). slice01
  adds the field to the frontmatter only; the slice that first reads it off the index must
  honor the adapter-fidelity invariant below. Flagged so it is not missed.

## Invariants carried in from earlier arcs

- **Adapter-fidelity obligation (from A4, gate-elevated).** A4 wired every read consumer
  off the index, which *removed* the `load_all` full-scan path — so there is **no live
  A/B safety net**: equivalence-to-baseline rests entirely on the adapter-fidelity tests
  (`odm-index/tests/adapter.rs`), which compare index-synthesized frontmatters against
  parsed ones field-by-field. **Hard invariant for every A5 slice that adds an
  index-backed reader or makes a consumer read a *new* record field: extend the adapter
  *and* its fidelity test in the same slice.** The CLI `*_matches_baseline` idempotence
  tests pass green *tautologically* whether or not the new field is faithful, so they will
  **not** catch a regression — only the fidelity test will. A slice that reads a new field
  without extending the fidelity test is a silent weakening of the A4 guarantee, and its
  ledger must carry a row that fails if the adapter is incomplete. (Raised by A4's
  arc-gate review; see `arc04-index-cache/closing-report.md` §"Independent arc-gate
  review" correction #2.)

## Method

Ledger per slice; CC implements, CDC verifies; cargo rows via CI / local 1.85+;
five-iteration cap. Slice closes bubble up to this arc-plan; the arc closes with its own
`closing-report.md` + composition check.

## Version History

### v1.7 — 2026-07-01
**slice04 closed (A-4 attested; A-10 mechanism-complete) + bubble-up propagated.**
Retired the A3 drift placeholder (Q-A3-2): `rollup` and `orient` now render **real
drift**. `odm_core::rollup::Drift` fleshed into a plain-data projection (counts +
drifted/errored entries with identity + expected/observed | reason; **no**
`reconcile`-crate dep — layering held via `Rollup::with_drift`). The reconcile
report gained render-identity (node number/name + fact `describe`), **resolving
slice03's double-load** — `reconcile`/`rollup`/`orient` render with no second store
read. One shared `odm-cli` projector (`compute_drift` → store-read `run_corpus` →
`Drift`) feeds **both** views (they cannot diverge). `--json` `drift` slot
populated **additively** (`tracked` retained; no version bump). 7/7 attested;
clippy clean; no `unsafe`; touched-path coverage ≥ 94% line (a slice02 gap in
`desired.rs` — only ever measured via odm-reconcile — surfaced and fixed). Drift
stays **store-read/on-demand, never index-cached** (S-5); the A4 invariant is
un-triggered. **Two findings for slice07/arc-close:** (1) `orient` (bare `odm`)
now runs every probe on each invocation — a regression against the "one cheap
call" ethos; recommend slice07 cache reconcile output (`.odm/` drift snapshot +
timestamp) and have `orient` *read* cached drift, refreshing only on `reconcile`.
(2) The persisted `ROLLUP.md` early-cutoff keys on the corpus meta-fingerprint,
which does not cover reality — committed drift can go stale, and `compute_drift`
runs probes even when the cutoff then skips the write; the cached-snapshot
approach dissolves both. *Plan-keeping:* CC propagated this bubble-up here itself.
Surfaced by: slice04 close.

### v1.6 — 2026-06-30
**slice03 closed (A-3 attested) + bubble-up propagated.** Landed `odm reconcile`
(`odm-cli`): invokes the slice02 store-read `Runner::run_corpus`, renders drift to
stdout (per-fact identity + `expected`/`observed`; probe errors at a distinct
`[error]` severity), and exits `check`-consistently — drift fails (1), a probe
error is a Warning surfaced always and failing only under `--strict`. The
severity/exit policy is a **pure** `verdict(&OutcomeCounts)` (unit-tested as a
table). `--json` emits **`reconcile/v1`**; `ProbeOutcome` + the runner report
types gained `Serialize` (additive, `ProbeOutcome` internally tagged on `kind`).
6/6 rows attested; clippy clean; no `unsafe`; cov reconcile.rs 97%. **No new index
reader** — `run_corpus` reads the store; the A4 invariant stays un-triggered.
**Sharpens slice04:** (1) the report carries node/fact *ids only*, so rendering
identity (`#number name`, a fact's `describe`) requires a store join — slice03
re-loads to do it; slice04 renders the same drift and should **factor the join
once** (a reconcile-side enrich helper, or the rollup builder — which already
holds the corpus — joining), not copy it. (2) Confirms the settled drift→rollup
path: the rollup builder calls the same store-read runner on demand and folds the
counts in (no index caching). (3) The pure `OutcomeCounts`→severity mapping is
the natural input to the rollup's drift summary. **Deviation flagged:** output
uses `writeln!` (matching `check`/`rollup`/`orient` — `odm-cli` has no `oxur-cli`
dep), not the `oxur_cli` helpers the slice-doc/CLAUDE.md name. *Plan-keeping:* CC
propagated this bubble-up here itself. Surfaced by: slice03 close.

### v1.5 — 2026-06-30
**slice02 closed (A-2 attested) + bubble-up propagated.** Landed the **`file`** probe
(`ProbeSpec::File` with `exists`/`sha256`/`size`, reusing the workspace `sha2`) and the
**probe-runner** (`Runner::run_node` / `run_corpus` → `NodeReport`/`CorpusReport`, drift
and error kept distinct in `OutcomeCounts`). 7/7 rows attested; clippy clean; no `unsafe`;
cov file.rs 97% / runner.rs 97% / shell.rs 100%. The slice01 `odm-reconcile` `lib.rs` was
modularized (`probe`/`shell`/`file`/`runner`) with the public API unchanged.
**Central decision implemented as recommended (not overridden):** the runner reads
`desired_facts` from the **store, not the index** — so the carried A4 adapter-fidelity
invariant is honored **by non-triggering** (no `IndexRecord`/adapter/fidelity/`FORMAT_VERSION`
change), guarded by the G-5 grep. **Sharpens slice03:** the result model makes severity/
exit-code mapping a pure function of `OutcomeCounts`, and the `--json` shape is `CorpusReport`
→ `{nodes:[{node_id,results:[{fact_id,outcome}]}]}`; slice03 must add `Serialize` to
`ProbeOutcome`/report (left off deliberately — no wire contract yet) under a `reconcile/v1`
schema marker. **Sharpens slice04 (new open question):** the rollup is a *hot view* off the
**index**, but reconcile reads the **store** — slice04 must decide how drift reaches the
rollup without re-introducing the index-vs-store tension (recommended: rollup invokes an
on-demand corpus reconcile and folds the result in, rather than caching drift in the index);
settle it explicitly in the slice04 doc. *Plan-keeping note:* CC propagated this bubble-up
here itself (the PM Part IV step slice01 had left to CDC). Surfaced by: slice02 close.

### v1.4 — 2026-06-30
**slice01 closed (A-1 attested; arc opener in) + CDC plan-keeping.** Landed `desired_facts`
(`odm-core::desired`: `DesiredFact`/`ProbeSpec`/`ShellExpect`), the three-way
`ProbeOutcome` (`Holds`/`Drifted`/`Error`) + object-safe `Probe` trait, and the **shell**
probe in the new `odm-reconcile` crate. 7/7 rows attested; clippy clean; no `unsafe`; cov
96%/100%. CDC-verified on structure (cargo rows pending CI). Two design rulings recorded:
(1) the shell probe **execs directly** (whitespace-tokenized argv), *not* `sh -c` — derived
from the non-negotiable `Error ≠ Drifted` split (`sh -c` makes a missing binary exit 127 = a
divergence). Bounded cost: no shell metacharacters; **carry-forward** — if quoted/structured
args are ever needed, add an explicit `argv: Vec<String>` shell-probe form (not `sh -c`),
in a later slice. (2) The frontmatter `desired_facts` field *correctly* uses
`skip_serializing_if` (YAML is self-describing — the A4 no-skip lesson is **postcard**-only);
the always-serialized obligation lands on the **index record**, not the frontmatter.
**Sharpens slice02/03:** `desired_facts` is a *nested structured* field, so the first index
reader's adapter + fidelity-test extension (the carried invariant) is more than a scalar
add. *Plan-keeping note:* CC wrote the bubble-up in its closing-report but did not propagate
it here; CDC recorded A-1 + this entry (the PM Part IV slice-close arc-plan-update step).
Surfaced by: slice01 close + CDC verification.

### v1.3 — 2026-06-30
**A5 activated; slice01-relevant open questions resolved (plan-deepening).** A5 is now the
active arc (chosen over A6 after A4's CI-green close). Per *plan late, plan deep*, deepened
the plan as slice01 is drawn up: **resolved** the probe-safety question (author-declared,
local, user-privilege; no MVP sandbox) and the program-level-acceptance-facts question
(no separate layer — `desired_facts` on the project/arc node). Added a fourth open question
naming the **index-integration trigger** for the carried adapter-fidelity invariant
(slice02/03, not slice01). Recorded two **already-built** hooks found while grounding the
slice: the `affects` edge (`EdgeKind::Affects`, `frontmatter.affects`, graph + check
dangling-ref) and the `Deferred` rollup model both exist (A2/A3) — so slices 05 and 06 are
*lighter than greenfield* (they add the **semantics**, not the model). No structural
re-break; the 7-slice breakdown holds. Surfaced by: drawing up slice01.

### v1.2 — 2026-06-30
**Carried in the adapter-fidelity invariant from A4's close (project-level bubble-up).**
Added an "Invariants carried in from earlier arcs" section recording the hard obligation,
elevated by A4's independent arc-gate review, that any A5 slice adding an index-backed
reader or reading a new record field must extend the adapter + its fidelity test in the
same slice — because A4 removed the `load_all` A/B net and the CLI idempotence tests pass
tautologically. Pure addition; the v1.0/v1.1 body is unchanged. Surfaced by: A4 arc-close
+ its arc-gate review (`arc04-index-cache/closing-report.md`).

### v1.1 — 2026-06-26
Added the **`## Arc Ledger`** section per LEDGER-DISCIPLINE v2.0 §B (the arc ledger opens
with the arc-plan, which already exists). Pure addition — the v1.0 body is unchanged.
Surfaced by: the ledger-discipline upgrade (v1→v2.0), not a slice bubble-up.

### v1.0 — 2026-06-26
Initial arc-plan, drafted from ODD-0013 §5.2/§4.4, the ODD-0015 A5 row, and the
Arc 03 deferrals (Q-A3-1 deferred-surfacing, Q-A3-2 drift placeholder) that this arc
cashes. No slices started; one-line altitude per *plan late, plan deep*.
