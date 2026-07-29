---
id: 01KYP5GW1X8AHFPDJPJ7585JSM
number: 582446700
type: artifact
schema: artifact/v1.1
name: 'CDC Verification — Arc 05 / Slice 04: drift in `rollup` / `orient`'
created: 2026-07-02
updated: 2026-07-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice04-drift-in-rollup-orient/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKZ6VDE3MPF3HS94DB
---
# CDC Verification — Arc 05 / Slice 04: drift in `rollup` / `orient`

> Independent verification of CC's closed ledger (impl + close on
> `arc05-slice04-drift-in-rollup-orient`, commits `bcde1ea` + `c1e9c05`), per
> LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo rows
> route to CI / a local 1.85+ run.

## Environment constraint (disclosed)

CDC sandbox has no 1.85+ toolchain; CC built + ran on local 1.95.0 (attested → reproduced on
CI). Branch cut off `arc05-slice03-…` (01–03 unmerged) — rebase onto `main` when they merge.

## Row dispositions

**Row count:** 7 opened, 7 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **S-1** — `odm_core::rollup::Drift { holds, drifted: Vec<DriftedFact>, errored:
  Vec<ErroredFact> }` (`rollup.rs:164`), entries carry identity + expected/observed | reason;
  `Rollup::with_drift` builder (`:364`) keeps `assemble` pure. `grep odm_reconcile
  crates/odm-core/Cargo.toml` → **none** (layering held). ✔
- **S-2** — `NodeReport.number/name` + `FactResult.describe` on the runner report
  (`runner.rs:27,39`); `grep load_all crates/odm-cli/src/reconcile.rs` → **none** (slice03's
  second load removed — the double-load seam is resolved, not just noted). ✔
- **S-3** — `rollup` renders real drift via `compute_drift`; `rollup_drift_reported` +
  `rollup_clean_no_drift` present; `grep "not yet tracked (A5)"` → **none**. ✔
- **S-4** — `orient` renders real drift (same helper); `orient_drift_reported` +
  `orient_clean_no_drift` present; placeholder gone. ✔
- **S-5** — `grep desired_facts crates/odm-index/src` → **none** (no facts in the index; the
  `Drift|drift` matches are pre-existing unrelated prose, as CC's Verify-amend flag notes);
  drift path is `compute_drift → run_corpus` (store). A4 invariant un-triggered. ✔
- **S-6** — one `compute_drift` projector (`reconcile.rs:69`) called by both `rollup` and
  `orient`; per-command code only *renders*. The two views cannot diverge. ✔
- **S-7 (no `unsafe`)** — grep empty across the three crates. ✔

**Attested by CC (local 1.95.0), pending CI:** clippy `-D warnings` → 0; touched-path line
coverage ≥ 94% (incl. a fixed slice02 `desired.rs` gap); `--json` additive; 41 suites green.
→ **PENDING CI.**

## Rulings on CC's flagged decisions

1. **`Rollup::with_drift` construct-complete builder** (slice-doc left the seam to CC).
   Accepted — keeps `assemble` pure and injects drift as data; the recommended
   construct-complete shape, no post-mutation. ✔
2. **S-5 Verify-amend flag** (the `Drift|drift` alternation matches unrelated pre-A5
   odm-index prose — "config drift"/"recomposition drift"). **Accepted, and correct.** My
   Verify was too broad; the load-bearing check is `desired_facts` (clean) + the compute path
   calling `run_corpus`, both of which hold. Good catch — an over-broad grep is a weak
   tripwire, and CC flagged rather than silently satisfied it.
3. **Fixed a slice02 `desired.rs` coverage gap in passing** (only ever measured via
   odm-reconcile before). Accepted — a real gap, honestly disclosed, fixed.

## The two arc-level findings (well-captured; CDC ruling)

CC surfaced two findings and recorded them in arc-plan v1.7. Both are real and correctly
routed. My rulings:

- **Finding (1): `orient` (bare `odm`) now runs every probe on each invocation** — a
  regression against the A3 "bare `odm` is the one cheap way-finding call" ethos. **This is a
  real design tension, and I own the planning miss:** my slice-doc wired drift into `orient`
  without flagging that `orient` is the *cheap, always-on* command, so running author-declared
  shell probes on every bare `odm` invocation is both a latency cost and a **trust-surface
  change** (every `odm` now auto-spawns declared commands once facts exist). slice04 delivered
  exactly the A-10 spec ("drift in rollup/orient"); the tension is a plan-level refinement the
  spec didn't anticipate, not a slice defect. **Currently low-risk** (odm's own corpus
  declares ~no facts yet, so `orient` runs ~zero probes today), but it becomes a live footgun
  the moment facts are declared. CC's proposed fix — a cached `.odm/` drift snapshot
  (+timestamp) that `orient` *reads*, refreshed only by `reconcile`/`rollup`/scheduled — is
  the right shape and belongs in slice07. **Disposition raised to the operator** (below) — it
  touches the trust model + the orient-is-cheap invariant, so the *urgency* (accept interim
  vs. pull the fix forward) is his call.
- **Finding (2): the `ROLLUP.md` early-cutoff keys on the corpus meta-fingerprint, which
  doesn't cover reality** — committed drift can go stale, and `compute_drift` runs probes even
  when the cutoff then skips the write. **Accepted** — the same cached-snapshot mechanism
  dissolves both (drift state carries its own freshness; the cutoff no longer gates a probe
  run). Routed to slice07/arc-close.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-4: real drift in both views; A-10 **mechanism-complete**
  (correctly marked "reproduce at arc-scale," not inherited).
- **Silent-drop diff honest?** ✔ — 7/7; the two findings, the Verify-amend, and the
  repurposed placeholder tests are all disclosed.
- **Findings + arc-plan?** ✔ — CC propagated the bubble-up itself (A-4 + A-10 note + v1.7),
  including both findings. Exemplary plan-keeping.

## Verdict

**Arc 05 / Slice 04 CDC-verified on structure; all flags ruled; cargo rows pending CI.** The
marquee drift-killer is now visible where users look: `rollup` and `orient` render real drift
(honest "no drift" when clean), the layering held (`odm-core` owns the shape, `odm-cli`
computes it), and the slice03 double-load is genuinely resolved. **One design tension needs
an operator call** (Finding 1: bare `odm` auto-running probes) — raised below; it does not
block slice05 (`affects` edge, an independent concern). A-4 attested-on-close; flips `done`
on CI green. **A5 at 4/7.**

CDC: planning thread, 2026-07-01. Iterations used: 1.
