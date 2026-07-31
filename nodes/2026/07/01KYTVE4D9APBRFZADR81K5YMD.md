---
id: 01KYTVE4D9APBRFZADR81K5YMD
number: 543313400
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 13 (Migration Fidelity): Live reconcile + vision mint'
created: 2026-07-30
updated: 2026-07-30
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice13-live-reconcile/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-31
edges:
  part_of: 01KYSX4R1GVV7BMQ6B853FAQ7T
---
# CC Prompt — Slice 13 (Migration Fidelity): Live reconcile + vision mint

Fire s12's reconcile capability + the vision mint on the **live `.worktrees/odm` corpus** — the arc's
final live mutation, the run that produces the shippable, faithful store. **One revertible commit** behind
the snapshot → dry-run → adjudicate → fire → verify protocol. After s13 CDC-closes, the **arc-close**
(MF-9 composition + the P-12 self-host demo) runs — no more slices.

> **Start condition:** on `release/1.0.x` with **s12 merged + green**. **This slice mutates the live `odm`
> store** — reconcile all drift + the vision mint — but only as one revertible commit after a clean,
> **adjudicated** dry-run. **Snapshot SHA `2fc25f5` + dry-run-first are HARD gates.** Reconciles + the
> vision mint are **expected**; the gate is **adjudication**, not "0 changes": every change must be a body
> re-snapshot to the current source, a moved-path rewrite (0017/0018), or the vision mint's single create —
> **nothing else** (0 id/schema/edge drift, 0 unexpected create, no re-mint). Any other deviation → **stop,
> flag CDC, do not fire.**

## Read first

1. `slice13-live-reconcile/ledger.md` (10 rows) — the spec of "done."
2. `slice-doc.md` (esp. the dry-run adjudication gate); **s07** `slice07-live-run/` + **s10**
   `slice10-coverage-live-run/` (the live-mutation protocol you reuse) and their `cdc-verification.md`s.
3. **s12** `slice12-reconcile-capability/` — the capability you fire: `reconcile_source`'s re-snapshot +
   moved-source re-discovery, and `synthesis::apply_project_vision`. **ODD-0025 §2.9** (reconcile policy).
4. **The drift this reconciles** (a fresh scan found ~17 body-drifted + 2 moved — but **the dry-run is the
   authority**, since the corpus keeps being edited): the `arc-plan.md` arc node, slice10's slice-doc +
   ledger nodes, ODD-0025 #25, a dozen older design/research ODDs, and moved ODD-0017/0018.
5. **The live store:** nodes under `.worktrees/odm/nodes/`; sources under the `release/1.0.x` docs tree;
   known-good SHA `2fc25f5`.

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first (mostly a run + thin CLI wiring this slice).
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **Pre-flight** (F-1). `release/1.0.x` green (s12 in) + `.worktrees/odm` clean. **Capture the known-good
   SHA (`2fc25f5`) + a before-manifest** (sha256 composite over all node files). Revert anchor.
2. **Wire the live reconcile invocation** if s12 left it library-only (a `--reconcile` surface or the
   `migrate` path). Thin CLI bit — if it's more than wiring, that's an **s12** fix; flag CDC.
3. **Dry-run + adjudicate** (F-2). Preview. Confirm every change is a body re-snapshot to current source,
   or a moved-path rewrite (0017/0018 → `04-accepted/`), or the vision mint's **single create** (the 1:1
   `project-plan` node). Confirm **0 id/schema/edge drift, 0 unexpected create, no re-mint**, store
   fingerprint unchanged after the dry-run. **Adjudicate before firing.**
4. **Fire as one revertible commit** (F-3…F-8). Reconcile all drift (re-snapshot bodies; re-discover
   0017/0018 by identity + rewrite `source.paths` to `04-accepted/`-relative); mint the vision
   (`apply_project_vision`: faithful 1:1 `project-plan` node + `#1000` re-cast as editorial-merge synthesis
   superseding it, attestation + lineage). One commit on `odm` atop `2fc25f5`; document
   `git reset --hard 2fc25f5`.
5. **Verify on the committed store** (F-3…F-7). **0 drifted** remain (recompute the gate vs current
   sources); 0017/0018 `source.paths` are `04-accepted/`-relative + resolve; `#1000` is the synthesis with
   attestation + `supersedes` → the 1:1 node, lineage clean; no collateral (only body/path changed on
   reconciled nodes, ids/schema/edges/status intact); `odm check` exit 0 + 0 `absolute-source-path`
   findings; `orient`/`rollup` byte-stable ×2; re-run idempotent (0/0).

## Constraints (flag, don't silently change)

- **Snapshot SHA + dry-run-first non-negotiable.** Reconciles/mint are expected — adjudicate; any *other*
  deviation (unexpected create/modify, id/schema/edge change on a reconciled node, body-hash failure after
  re-snapshot, `check` not green, non-idempotent re-run) → **revert to `2fc25f5` + finding**, not forced.
- **No new capability.** s12 is the capability; only the thin live-invocation wiring is in-bounds. A gap is
  an **s12** fix.
- **The gate stays migration-time-only** — reconcile re-establishes fidelity, it does not become a
  continuous check.
- Don't run the **arc-close** (MF-9 composition, the P-12 demo, the final reconcile-and-freeze, the
  `closing-report.md`) — that is the formal arc-close after this slice CDC-closes. Don't start **L-8a**.
- No `unsafe`; typed errors; any code touched clippy-clean.

## Deliverables

The committed reconciled + vision-minted store (`odm` branch, one commit atop `2fc25f5`); any thin CLI
wiring on `release/1.0.x`; `ledger.md` evidence per row (`attested`/`reproduced` — cite the adjudication
diff, counts, the undo SHA, the vision node's synthesis/attestation/supersedes); `closing-report.md` —
per-row walk, the before/after (drift count → 0, the vision mint, the 2 moved paths corrected), the dry-run
adjudication record, any findings, **plus the v2.0 Bubble-up** (did s13 make the corpus faithful + land the
vision; what the run revealed; the silent-drop diff vs In/Out; confirm the arc-close is next). Branches:
`odm` (store) + `release/1.0.x` (wiring, if any).

## Working agreement

Amend don't work around; flag every deviation; five-iteration cap. Your `done` is proposed-done — CDC
reproduces the store-state rows by direct read (0 drift, the vision synthesis + lineage, moved-path
correctness, no collateral) + CI. On close, bubble up to `../arc-plan.md` (MF-7 done; MF-9 fidelity live;
the arc-close next).
