---
id: 01KYP5G2RT3R6QRRN07JCSZB4J
number: 513839300
type: artifact
schema: artifact/v1.1
name: 'cc-prompt — Release Hardening: arc close (RH-6 / RH-7 compose + closing-report + bubble-up)'
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/cc-prompt-rh-arc-close.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# cc-prompt — Release Hardening: arc close (RH-6 / RH-7 compose + closing-report + bubble-up)

> **Arc:** Release Hardening (UAT) · **Task:** close the arc — write
> `arc-release-hardening/closing-report.md`, flip the compose rows, and bubble up to `project-plan.md`.
> **All chunks are done** (RH-1…RH-10; C-7 retired into C-5). **CDC has reproduced the RH-6 compose**
> (evidence below) and RH-7 is already attested via the C-5 cutover. **An independent fresh-context
> arc-gate will run after you write the report** (the A4/A5/arc-store-home pattern) — write it to be
> reviewed by a context that has never seen this arc.

## What's already true (don't re-do — cite it)

- **Chunks closed** (each with a CDC verification on the real store): **C-1** theming (RH-1), **C-2**
  type taxonomy (RH-2), **C-3** `list` overhaul (RH-3), **C-4** command surface + ODD-0023 reorg
  (RH-4), **C-5** fold + dates + names + vision + the SH-6 cutover (RH-5), **C-8** normalized status
  (RH-9), **C-6** `validate` hardening — G-2/G-3/L-3b (RH-10). **C-7** retired into C-5.
- **RH-7 (reflexive) — attested:** the C-5 cutover re-derived the corpus into the store home,
  `check`-green at 60 with `design`/`research` types + de-numbered names.

## RH-6 compose — CDC reproduced it (cite this run)

Run on the self-hosted store at `release/1.0.x` @ `839fa26`, the **whole RH surface in one pass**,
every chunk's work visible and coherent:

- **`orient`** — VISION renders (L-3a), CURRENT FOCUS = arc #1600 (the C-5 focus fix), READY lists
  `design` nodes (C-2 type), INTEGRITY ok.
- **`project`** (C-4 rename of `context`) and **`chain`** (C-4 rename of `path`) resolve.
- **`validate`** (pure) exit 0 with the two C-6/G-3 undecomposed warns; **`check`** (validate +
  reconcile, C-4's inversion) exit 0.
- **`odm node list`** (C-4 reorg) renders `DATE│TYPE│STATUS│NAME-tree│ID` on the C-1 warm-orange theme
  (C-1), branch-and-leaf tree + de-numbered names (C-3), **real** dates 2026-06-20… (C-5/F-20), clean
  names — no `(plan-of-record)` (C-5/F-18), `design`/`research`/`project`/`slice` types (C-2), and the
  normalized `active`/`done` STATUS (C-8).

That is RH-6 — *"the self-hosted CLI is coherent + themed + UAT-validated end-to-end."* Reproduced.
(CDC will re-run it as part of the arc-gate; you may cite it as reproduced.)

## Write `arc-release-hardening/closing-report.md`

Per `PROJECT-MANAGEMENT.md` (arc close). Include:

1. **What the arc set out to do** — UAT-driven v1.0.0 hardening of the self-hosted CLI (theming,
   types, naming, `list`, the command surface, the fold + cutover, check-hardening), triaged surface
   vs. model.
2. **Per-chunk walk** — one line per RH-1…RH-10 (+ C-7 retired), each pointing at its
   `cdc-verification.md` / closing report, with the headline outcome.
3. **Composition verdict** — **RH-6** reproduced (the compose run above), **RH-7** attested (C-5
   re-derivation), **RH-8** bubble-up dispositioned. **Durable `reproduced` (CI) rides the push** —
   state it honestly (origin is SSH; the branch is unpushed).
4. **The findings ledger closed out** — every F-row and the routed L/G rows dispositioned (done /
   routed / deferred). Note the **defects the arc itself surfaced and fixed** (the C-5 dogfood trio;
   the C-8 read-back that delivered L-1; the C-6 premise corrections) as part of the honest record.
5. **Carried forward** (recorded, not dropped): `store status` (ODD-0023 §7, non-blocking); the
   shared-`context.json` two-level model (post-1.0); `#1600` should not affirm decomposition until A6
   closes (C-6's `decomposition-drift`); L-8b pre-release ODD reconciliation; the LLM arc's L-1
   remainder (`--json` unreached gates + satisfaction/weakest-link).

## Bubble up to `project-plan.md`

- **§2a** — Release Hardening row → **closed/done** (all chunks + compose).
- **§3** — RH status → closed; the surface is **settled**.
- **Sequencing** — per §2a, the order is **RH (now closed) → LLM command surface → A6 resumes at
  slice05 (PM-skill) + slice06**. Note that C-8 already delivered the LLM arc's blocking slice (L-1),
  so that arc is materially lighter (see its v1.2). A6 slice05/06 now target the settled surface.
- **Version-history entry** naming RH's close and what it unblocks.
- **Ledger:** RH does not have its own project P-row (it's a v1.0.x hardening arc, not an A-numbered
  DoD arc) — but record its close in P-13 (bubble-up dispositioned) and note the DoD-surface is now
  settled for the P-7 (situational-awareness) demo at project close.

## Ledger + method

- Flip **RH-6 → reproduced** (CDC), **RH-7 → reproduced** (attested via C-5, `check`-green),
  **RH-8 → done**. The class-(a) rows (RH-1…RH-5, RH-9, RH-10) are already attested.
- One commit on `release/1.0.x` (the arc lives there). After you write it, **CDC runs the independent
  fresh-context arc-gate** (whole-arc review against the running binary) — that's the final close gate,
  as arc-store-home had. A PASS / PASS-WITH-NOTES from that gate closes the arc.
- **Not in this task:** any new code. This is the close — synthesis + bubble-up only.
