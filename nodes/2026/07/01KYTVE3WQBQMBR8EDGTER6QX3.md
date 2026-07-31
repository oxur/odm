---
id: 01KYTVE3WQBQMBR8EDGTER6QX3
number: 575161900
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 12 (Migration Fidelity): Reconcile capability'
created: 2026-07-29
updated: 2026-07-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice12-reconcile-capability/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-31
edges:
  part_of: 01KYSX4R1GPDKH7M5ANCTDND04
---
# CC Prompt — Slice 12 (Migration Fidelity): Reconcile capability

Build the **reconcile** operation — the one that makes odm's corpus *re-faithful* after its sources
legitimately changed — and wire s11's synthesis into the project-vision **apply** path. **All in code,
fixture-proven, with the live `.worktrees/odm` store untouched.** The live close (fire reconcile + the
vision mint) is **s13**; the arc-close (MF-9 composition + the P-12 acceptance demo) follows s13.

> **Start condition:** on `release/1.0.x` (green, s11 merged). **Fixture only — this slice writes no
> `odm`-branch commit and fires nothing on the live store.** `reconcile_source` today *skips* a drifted
> non-stub (records it in `drifted`, defers the fix here); this slice builds the fix.

## Read first

1. `slice12-reconcile-capability/ledger.md` (10 rows) — the spec of "done."
2. `slice-doc.md` (esp. **Why** + the In/Out split); **ODD-0025** §2.1 (the body-hash gate is
   **migration-time only** — nothing re-verified later), §2.8 (update-in-place repair), §2.3 (the project
   *synthesis* node stays excluded from 1:1).
3. **The drift this reconciles** (context — the *live* run is s13, not here): the s08 CDC verification
   (active arc node), the s10 verification (ODD-0013/0020 declined-backfill), s11's `closing-report.md`
   (0013/0017/0018 moved `01-draft/`→`04-accepted/` + `state:` changed).
4. **The code you change:**
   - `crates/odm-migrate/src/mapping.rs::reconcile_source` — the `drifted`-skip path becomes an
     **update-in-place re-snapshot**; add the **moved-source re-discovery** (re-find by identity, rewrite
     `source.paths` via `fidelity::relativize`, then re-snapshot).
   - `crates/odm-migrate/src/synthesis.rs` + wherever the project node is built (`replan.rs`) — the
     **vision-apply** path (fixture only).

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `03-error-handling.md`, `05-type-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **Re-snapshot mode** (F-1). A drifted non-stub is **updated in place**: body ← current source,
   **`id`/`edges`/`status` preserved**, §2.1 gate re-passes. Keep the s04 **stub-repair** path distinct
   from this **non-stub re-snapshot** (F-5).
2. **Moved-source re-discovery** (F-2). When stored `source.paths` no longer resolves but the source moved,
   re-find it **by identity** (number/coordinate), rewrite `source.paths` to the **s08-relative** new path
   (`fidelity::relativize`), then re-snapshot. Handle a node needing *both* (moved path + changed body).
3. **The living-plan-node policy** (F-3). Decide + implement how a node whose source is a still-changing
   doc reconciles — **recommended: reconcile-to-current, no special exclusion** (the gate is migration-time
   only, so inter-reconcile drift is by-design invisible to `check`; at arc-close the source is stable).
   **Record the decision; amend ODD-0025 (cited) if it needs a line** — don't work around.
4. **Vision-apply path** (F-4). Using s11's `synthesis`, wire the path that re-casts the project node as an
   **editorial-merge synthesis superseding a 1:1 `project-plan` node**. **Fixture only — no live mint.**
5. **Resolve the ODD-0025 §4 decision** (F-7). Either amend §4 (`artifact` stamps the shared `v1.1`) —
   noting that, because ODD-0025 *is* node #25's source, this makes #25 a re-snapshot target for **s13** —
   or record an explicit deferral with rationale. **Don't leave it dangling.**
6. **Fixtures** (F-1…F-6): drifted-non-stub re-snapshot (id/edges preserved); moved-source re-discovery;
   living-plan-node; vision-apply; stub-vs-non-stub; idempotence + dry-run-safety.

## Constraints (flag, don't silently change)

- **No live store mutation. No `odm`-branch commit. Fire nothing on `.worktrees/odm`.** The live reconcile
  + vision mint are **s13**; the MF-9 composition + P-12 demo + arc-close are **post-s13**.
- **The gate stays migration-time-only** (§2.1) — reconcile *re-establishes* fidelity, it does **not** make
  the gate a continuous `check`. Reconcile only ever sets a body *from* its current source, identity
  preserved. **Amend ODD-0025 (cited), don't work around** if the re-snapshot semantics or living-node
  policy need a model line.
- Don't pull **s13** (the live run) or **L-8a** (post-1.0 design-corpus migration) forward.
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables

The code change on `release/1.0.x` (re-snapshot mode + moved-source re-discovery + the vision-apply path +
the living-node policy) and any ODD-0025 amendment (the §4 resolution + a model line if the policy needs
one); `ledger.md` evidence per row (`attested` → CI — cite test names, exits); `closing-report.md` —
per-row walk, the living-plan-node decision + its rationale, the §4 resolution, any ODD amendment, **plus
the v2.0 Bubble-up** (did s12 land the reconcile capability; what it revealed; the silent-drop diff vs
In/Out; confirm s13 next + the arc-close after). Branch: `release/1.0.x` only.

## Working agreement

Amend don't work around; flag every deviation; five-iteration cap. The live run is **already carved out to
s13**, so the capability should fit one context — but if not, split (land re-snapshot + moved-source
re-discovery first; split the vision-apply and/or the policy) and flag CDC. Your `done` is proposed-done —
CDC reproduces the code + fixtures + CI. On close, bubble up to `../arc-plan.md` (MF-9 mechanism ready; s13
next; arc-close after).
