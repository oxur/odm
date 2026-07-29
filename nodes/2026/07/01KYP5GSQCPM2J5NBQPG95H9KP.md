---
id: 01KYP5GSQCPM2J5NBQPG95H9KP
number: 564300500
type: artifact
schema: artifact/v1.1
name: Arc 05 — Reconciliation — closing report
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKQ3QE7FGTM80MYNDH
---
# Arc 05 — Reconciliation — closing report

> The arc-level close for A5, per PROJECT-MANAGEMENT.md Part V + LEDGER-DISCIPLINE v2.0 §B.
> Distinct from the eight per-slice closing-reports. Written by CDC (who ran the per-slice
> verifications); the independent **arc-gate review** is recorded at the foot (a fresh-context
> subagent, per §B's doer ≠ gatekeeper rule).
>
> **Evidence register.** Unlike A4 (which closed CI-pending), **A5 closes with CI green on
> `release/1.0.x` (operator-confirmed, 2026-07-06)** — so the slices' cargo/coverage/number
> rows and the class-(b) executable reproductions are **`reproduced`**, not attested-pending.
> CDC additionally reproduced the composition **structurally** in-session (the end-to-end
> tests exist and span the arc — below). Git note: work lives on `release/1.0.x` (main was
> reset onto `release/0.3.x`, the pre-rebuild import); future branches cut from `release/1.0.x`.

## 1. Capability — restated, and the verdict

**Capability (from `arc-plan.md` / `project-plan.md` A5):** drift detection for plans — the
**marquee state-drift killer** (ODD-0001 C2, the prod-DB-503). Nodes declare `desired_facts`;
a pluggable `Probe` trait diffs *declared* desired state against *observed reality* and
reports **drift**. Reconciliation is honest only about tracked facts; drift folds into the
generated `rollup`/`orient` (replacing the A3 placeholder); the `affects` edge powers a
stale-doc-vs-decision check; deferred nodes surface with a checkable re-entry predicate. Per
the **v1.8 redirection (ODD-0019)**, freshness is **incremental and automatic on every
command** (riding the A4 stat-cache), not scheduled.

**Verdict: delivered.** All five arc exit criteria are met by composed, end-to-end demos
wired through the real commands, CI-green. The freshness redirection — the arc's defining
mid-course change — landed as ODD-0019 and is realized end-to-end. No capability gap.

## 2. Slice walk (8 slices — matches the breakdown; no arc-scale silent drop)

| Slice | Scope delivered | Outcome | Pointer |
|-------|-----------------|---------|---------|
| 01 desired_facts + Probe + shell probe | frontmatter `desired_facts`; three-way `ProbeOutcome` (Holds/Drifted/Error); shell probe (exec-directly, `Error≠Drift`); `odm-reconcile` crate | **delivered** | `slice01-…/cdc-verification.md` |
| 02 file probe + probe-runner | `file` probe (exists/sha256/size); per-node/corpus runner (store-read); drift/error distinct in `OutcomeCounts` | **delivered** | `slice02-…/cdc-verification.md` |
| 03 `odm reconcile` | the command; human + `--json` `reconcile/v1`; `check`-consistent severity via pure `verdict(OutcomeCounts)`; `--strict`-gated probe-error | **delivered** | `slice03-…/cdc-verification.md` |
| 04 drift in rollup/orient | `Drift` projection (odm-core owns shape); shared `compute_drift`; real drift in both views; A3 placeholder gone | **delivered** | `slice04-…/cdc-verification.md` |
| 05 `affects` + stale-doc (C5) | `Violation::StaleDoc` (temporal, never semantic); Warning-tier; `violation_severity` helper | **delivered** | `slice05-…/cdc-verification.md` |
| 06 deferred surfacing + re-entry (Q-A3-1) | `deferred{because, reenter_when}`; re-entry = a reconcile-evaluated fact; `reconcile_views` (one run → drift+deferred) | **delivered** | `slice06-…/cdc-verification.md` |
| 07 incremental drift core (ODD-0019) | probe `inputs`/volatile classes; racy-correct input fingerprint; persisted `.odm/` drift snapshot; incremental reconcile | **delivered** | `slice07-…/cdc-verification.md` |
| 08 freshness wiring + honest staleness | `reconcile_views`→incremental (bare `odm` = zero volatile probes); `odm reconcile`→full; "last checked Xm ago"; drift-aware `ROLLUP.md` cutoff; `next` graph-pure | **delivered** (capstone) | `slice08-…/cdc-verification.md` |

**Arc-scale silent-drop check:** 8 slices specified, 8 delivered, 8 CDC-verifications on disk.
**One ledger-shape defect found and fixed** (not a slice drop): the arc grew to 8 slices but
carried 7 class-(a) rows — slice08's closure had been folded into the compose rows. Fixed by
appending slice08 as stable **A-14** (per v1.10's stable-ID decision, not a renumber). The
original slice07 ("scheduled reconcile") was **superseded** by the v1.8 freshness model, not
dropped — a tracked plan-change (arc-plan v1.8; the demotion is recorded). No silent drop.

## 3. Arc Ledger — per-row walk

### Class-(a): slices closed (A-1…A-7 + A-14) — reproduced (CI green)

All eight point to a closed `cdc-verification.md`; CI green on `release/1.0.x` flips them from
attested to **reproduced**. Strength: `reproduced`. ✔ (Row IDs are non-contiguous by design —
A-14 is slice08's stable-ID class-(a) row; identity ≠ order.)

### Class-(b): slices compose — reproduced at arc scale (A-8…A-12)

Per §B, reproduced end-to-end, never inherited. CDC reproduced each structurally (the wired
command exercises the composed behavior); CI green is the executable reproduction.

- **A-8 — a declared fact whose reality diverges is detected + reported by `odm reconcile`.**
  `reconcile_drift_reported_nonzero` (drift → exit 1, identity + expected/observed) +
  `reconcile_command_runs_full_refreshes_volatile` (full path re-probes + stamps). The C2
  loop, end-to-end. ✔
- **A-9 — both probe kinds end-to-end.** `shell_probe_*` (8 cases incl. holds/drift/error) +
  `file_probe_*` (6 cases), both evaluated through the reconcile paths. ✔
- **A-10 — drift surfaces in `rollup`/`orient`; A3 placeholder gone.** `rollup_drift_reported`
  + `orient_drift_reported`; honest "no drift" when clean; `grep "not yet tracked (A5)"` → none. ✔
- **A-11 — `affects` powers a stale-doc finding in `check`.** Arc-scale (wired `odm check`):
  `check_stale_doc_is_warning` + `check_json_includes_stale_doc` (`odm-cli/tests/cli.rs`) —
  `A affects B` ∧ `A.updated > B.updated` → a `stale-doc` Warning through the command
  (`--strict`-gated). (The `odm-core` unit `check_flags_stale_doc_after_decision` backs the
  finding logic; the CLI tests are the end-to-end demo.) ✔
- **A-12 — deferred nodes surface with a checkable re-entry predicate.**
  `rollup_surfaces_deferred` + `orient_surfaces_deferred`; re-entry reconcile-evaluated
  (ready/waiting). ✔

*Freshness overlay (the v1.8 capability, verified across A-8/A-10/A-12):* bare `odm` runs the
**incremental** path — `orient_runs_zero_volatile_probes` proves zero volatile probes (the
slice04 regression dissolved); volatile facts render honest "last checked Xm ago"; the
`ROLLUP.md` cutoff is drift-aware. ✔

### Class-(c): bubble-up findings dispositioned (A-13)

Every slice bubble-up routed via the arc-plan version history (v1.2…v2.3). Three carried
follow-ups dispositioned (see §5). No finding undisposed. ✔

## 4. Accumulated arc-plan change log (drift from the original plan, in one place)

A5 shipped at `arc-plan.md` **v2.3**, from v1.0. The substantive drift, all tracked:

- **The v1.8 freshness redirection (the big one).** Mid-arc, Duncan redirected the freshness
  architecture: scheduled/manual reconcile is too laggy and *continually misses items* — so
  drift rides the A4 stat-cache, incremental on every command. Captured as **ODD-0019** +
  arc-plan v1.8; the original slice07 ("scheduled reconcile") was superseded by two slices
  (07 probes-as-rules, 08 wiring). This is the arc's defining decision.
- **The store-vs-index boundary (slice02) held all eight slices.** Every consumer reads
  facts/drift/deferred from the store/reconcile-overlay, never the index — the A4
  adapter-fidelity invariant stayed **honored by non-triggering** the entire arc (every slice
  carried an empty-`git-diff` guard row). The one design temptation (index the deferred
  marker) was considered and rejected.
- **Two probe classes + honest staleness (ODD-0019).** input-derived (stat-cached, always
  fresh) vs volatile (honest "last checked Xm ago", explicit refresh) — the operator's
  ratified choice.

## 5. Bubble-up to the project

1. **Did A5 deliver its capability as `project-plan.md` defined it?** **Yes** — the
   reconciliation capability (desired_facts + probes; drift in rollup/orient replacing the A3
   placeholder; `affects`/stale-doc; deferred/re-entry) is delivered, *and* the freshness
   model makes it fresh on every command — a capability the roadmap line didn't specify but
   ODD-0019 scoped and delivered.
2. **What did A5 reveal that the project plan did not anticipate?** No roadmap re-scope. Three
   findings carried forward, none forcing a project-plan change:
   (a) **two corpus reads per bare command** (incremental probe-load + a render-identity
   reload) — a post-arc/A6 optimization (unify the loads, or the snapshot carries identity);
   (b) **`CLAUDE.md` doc-drift** — its "CLI output uses `oxur_cli`" claim is stale (odm-cli
   uses `tabled`+`writeln!`, no oxur-cli dep) → reconcile `CLAUDE.md` (a doc-keeping fix,
   good to fold into A6's doc work);
   (c) **volatile-re-entry behavior change** — a deferred node with a *volatile* re-entry
   predicate now needs an explicit `reconcile` to flip "waiting→ready" (input-derived
   re-entries stay auto-fresh); the honest consequence of the freshness model, recorded.
3. **Silent-drop diff at arc scale, rolled to the project:** none. The one ledger-shape defect
   (missing slice08 class-(a) row) was found + fixed at close; the superseded scheduled-slice
   is a tracked plan-change, not a drop.

**Standing arc→project status:** A5 is independently shippable and composes onto A4 (the two
arcs share the stat-cache spine — ODD-0019). It does not block A6. Project DoD row **P-5
("A5 closed + composed")** flips to `done` (CI green). **MVP (A1–A3) + A4 + A5 are now in;
only A6 (migrate + self-host + PM-skill) remains for the v1.0.0 DoD.**

## 6. Cross-scale trending (what recurred across slices — §B)

Two patterns recurred, both healthy signals:

- **CC caught imprecisions in CDC's ledger rows twice** (the `oxur_cli` render convention,
  slice03; the odm-core-vs-odm-cli severity-test crate, slice05) — a good doer↔planner check.
  Lesson recorded: CDC should name a row's Verify crate by *where the behavior lives*, not
  where it's conceptually "about."
- **The slice04→05→06 pattern (plain-data projection + `Rollup::with_*` builder + store-read
  projector + additive JSON) composed with nothing new invented** for three consecutive
  slices — a sign the arc's seams were right. `reconcile_views` (one `run_corpus`, two views)
  was the keystone that later let slice08 flip the whole arc to incremental at one seam.

## 7. Closure

**Composition verdict: delivered — the slices compose into the reconciliation capability,
fresh on every command.** Slices: 8 (matches the breakdown, after the A-14 reconciliation).
Class-(b) rows A-8…A-12: reproduced at arc scale (end-to-end command demos, CI green).
Findings dispositioned: all (A-13). No arc-scale silent drop. A failed class-(b) row would
spawn a remediation slice (none needed).

Gate reviewed by: **independent fresh-context subagent** (recorded below) — per §B, the one
who performed the composition does not sign it off.

CDC: planning thread, 2026-07-06.

---

## Independent arc-gate review

**Reviewer:** fresh-context subagent (read-only; no role in the composition). **Verdict:
PASS-WITH-NOTES.**

The gate verified each cited demo against the source, not this report's summary:

- **Slice count / ledger shape (PASS).** 8 `cdc-verification.md` on disk; Arc Ledger rows
  A-1…A-14 all `done`; the stable-ID append is complete (A-1…A-7 + A-14 = 8 class-(a) rows,
  every slice covered). The self-disclosed ledger-shape defect was correctly repaired, not
  papered over.
- **Class-(b) A-8…A-12 (PASS).** Every cited test exists and is a genuine command-level
  end-to-end demo (invokes `run(dir, &["reconcile"|"rollup"|"orient"|"check"])`, asserts exit
  codes + stdout, uses counting probes) — not slice-local units mislabeled as composition.
  `orient_runs_zero_volatile_probes` asserts a **true zero** count. "not yet tracked (A5)" is
  gone from `crates/`.
- **Zero-index-change (PASS).** No A5 slice commit touched `crates/odm-index/`;
  `odm-reconcile/src` carries none of the index identifiers. Invariant honored by
  non-triggering the whole arc.
- **Bubble-up honesty (PASS).** All three carried follow-ups verified real and disclosed
  (the `CLAUDE.md` oxur-cli drift confirmed against `odm-cli/Cargo.toml`); the superseded
  scheduled-slice07 is a tracked plan-change (arc-plan v1.8), not a drop.

**Gate corrections, dispositioned:**

1. **A-9 shell count "9" → 8.** **Applied** (above).
2. **A-11 cited the `odm-core` unit, not the wired `check` demo.** **Applied** — the compose
   row now cites `check_stale_doc_is_warning` + `check_json_includes_stale_doc` (odm-cli), with
   the core unit noted as backing the logic. (The arc-plan A-11 row updated likewise.)
3. **P-5 wording ("flips to `done`") vs the still-`open` project-plan row.** **Resolved** —
   P-5 is flipped to `done` as part of this close (project-plan v1.6), so the claim is now
   true, not aspirational.
4. **A-13 version-range mismatch (v2.2 vs v2.3).** **Applied** — harmonized to v2.3 in the
   arc-plan A-13 row.

The gate confirmed the substance (real composition, honest ledger, no drops); the four
corrections were count/citation accuracy, none inflating capability. Arc close stands.
