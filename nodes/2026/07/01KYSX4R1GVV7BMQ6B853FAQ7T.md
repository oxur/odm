---
id: 01KYSX4R1GVV7BMQ6B853FAQ7T
number: 58837413
type: slice
schema: slice/v1.1
name: Slice 13 (Migration Fidelity) — Live reconcile + vision mint (plan-of-record)
created: 2026-07-30
updated: 2026-07-30
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice13-live-reconcile/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-30
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
---
# Slice 13 (Migration Fidelity) — Live reconcile + vision mint (plan-of-record)

> Refs: `../arc-plan.md` (s13 row; MF-7 live half; MF-9 compose) · **s12** (the reconcile capability +
> `synthesis::apply_project_vision` this fires) · **ODD-0025** §2.9 (reconcile re-establishes fidelity to
> a changed source; the living-plan-node policy), §2.3 (synthesis) · **s07/s10** (the live-mutation
> protocol this reuses). `depends_on:` s12.
>
> **The arc's final live mutation — the run that produces the shippable, faithful corpus.** It fires s12's
> reconcile capability + the vision mint on the live `.worktrees/odm` store, behind the snapshot → dry-run
> → adjudicate → fire → verify protocol. After s13 CDC-closes, the **arc-close** (MF-9 composition + the
> P-12 self-host acceptance demonstration) runs — no more slices.

## Goal

Make odm's own corpus **fully faithful again** after this arc's own edits, and land the modeled project
vision. **Done when** every drifted node is re-snapshotted to its current source (bodies match; the two
L-8b-moved ODDs re-discovered by identity and their `source.paths` rewritten to the new `04-accepted/`
location); the project vision is minted as an **editorial-merge synthesis superseding a 1:1 `project-plan`
node** (s12's `apply_project_vision`, live); `odm check` is green; `orient`/`rollup` are byte-stable; the
run is idempotent and collateral-free — all as **one revertible commit** atop known-good `2fc25f5`.

## Why

s12 built reconcile and proved it on fixtures; s13 is where it earns its place — the corpus has drifted
during the arc's own work, and "odm self-hosts *faithfully*" (MF-9, the P-12 DoD) requires that drift
closed on the live store before the arc can claim it. A fresh live scan (2026-07-29) finds **19 nodes
needing reconcile**: **17 body-drifted** (the active `arc-plan.md` arc node; the slice10 slice-doc + ledger
nodes edited during their own slices; ODD-0025 #25, drifted by s12's own §2.9/§4 edit; and a dozen older
design/research ODDs whose sources were edited post-migration) and **2 moved-source** (ODD-0017/0018,
whose files L-8b relocated `01-draft/`→`04-accepted/`). This is exactly the "sources legitimately changed"
case §2.9 defines — not corruption; the adjudication step confirms it before firing.

**Live-run, capability-already-proven** — the arc's discipline (s07 after s06, s10 after s09): s12's
reconcile + vision-apply are fixture-verified; s13 fires them on the store that ships, behind the same
hard gates s07/s08/s10 used, because this run rewrites odm's own normative-ODD nodes for the last time.

## Scope

**In (one revertible commit on the `odm` branch, atop known-good `2fc25f5`):**

- **Reconcile every drifted node** to its current source (re-snapshot the body in place, `id`/`edges`/
  `status` preserved, gate re-passes) — the ~17 body-drifted nodes from the live scan, and re-discover the
  **2 moved** ODDs (0017/0018) by identity, rewriting their `source.paths` to the portable `04-accepted/`
  relative form. *(Count is the dry-run's to confirm, not a fixed contract — the corpus keeps being edited;
  s13 reconciles whatever is drifted at fire time.)*
- **Mint the project vision** (MF-7 live half). Establish the **1:1 `project-plan` node** (faithful body
  from `project-plan.md`, hard-gated) and re-cast the project node (`#1000`) as an **editorial-merge
  synthesis superseding it**, via s12's `synthesis::apply_project_vision`, with a recorded attestation and
  the tooling-guaranteed bidirectional lineage.
- **Wire the live reconcile invocation** if not already exposed (s12 built the library capability; s13
  fires it — via the `migrate`/reconcile path or a `--reconcile` surface, whichever s12 left; a thin CLI
  bit, flagged if it's more).
- **The full s07 live protocol.** Confirm `release/1.0.x` green (s12 in) + the `odm` store clean; capture
  the **known-good SHA `2fc25f5` + before-manifest**; **`--dry-run`** → **adjudicate** (every change is a
  body re-snapshot to current source or a moved-path rewrite; the vision mint is 1 create — the 1:1
  `project-plan` node — plus the `#1000` re-cast; **0 id/schema/edge changes on reconciled nodes**, **0
  unexpected creates**, store fingerprint stable after the dry-run) → **fire as one revertible commit** →
  **verify**.

**Out:**

- **The arc-close → after s13** (a formal CDC/gate step, not a slice): the **MF-9 composition check** (do
  the twelve/thirteen slices compose into "odm self-hosts faithfully"?) + the **P-12 self-host acceptance
  demonstration** (a fresh session orients fully from `odm orient` alone on the reconciled corpus) + the arc
  `closing-report.md` + the bubble-up to `project-plan.md`. **Note (disclosed):** nodes whose sources are
  edited *during* the close — the `arc-plan.md` arc node especially, plus s13's own close docs — will show
  minor re-drift after this run; per §2.9 that is by-design invisible to `check`, and the arc-close should
  end with a **final reconcile-and-freeze pass** once the plan docs stop moving, so the shipped corpus is
  genuinely faithful.
- **L-8a** (migrate the whole design corpus into nodes) — post-1.0 follow-on.
- Any new capability — s12 is closed; a capability gap here is an **s12** fix, not new s13 scope (only the
  thin live-invocation wiring is in-bounds).

## Verification

Live, class-(b) — the committed store is the evidence; CDC reproduces by direct read + independent
recomputation (the s07/s08/s10 method). After the committed run:

- **0 drifted nodes** remain (recompute the body-hash gate against current sources — every source-bearing
  node faithful); the 2 moved ODDs carry `04-accepted/`-relative `source.paths` that resolve.
- **The vision:** `#1000` is an editorial-merge synthesis with a recorded attestation and a `supersedes`
  edge to a faithful 1:1 `project-plan` node; the bidirectional-lineage `check` rule is clean.
- **No collateral:** reconciled nodes changed body (and, for the 2 moved, `source.paths`) **only** —
  ids/schema/edges/status intact; nodes that were already faithful untouched; retired node excluded.
- `odm check` **exit 0**; `orient`/`rollup` byte-stable ×2; a re-run is **idempotent** (0 reconciled / 0
  created); the absolute-source-path guard (s10 it1) stays green (all rewritten paths relative).

`check`/`orient`/`rollup`/`migrate` runtime rows attested-by-CC → CI; CDC reproduces store state (0 drift,
the vision synthesis + lineage, moved-path correctness, no collateral) by direct read.

## Rollback & findings discipline

Same spine as s07/s10. **Hard gates:** snapshot SHA + before-manifest are non-negotiable; **dry-run first**.
The dry-run's changes are **expected** (reconciles + the vision mint) — the gate is *adjudication*: every
reconcile is a body re-snapshot to the current source (or a moved-path rewrite), the only create is the
1:1 `project-plan` node, and **nothing else** changes (0 id/schema/edge drift, no unexpected create, no
re-mint). Any deviation — an unexpected create/modify, an id/schema change on a reconciled node, a
body-hash failure after re-snapshot, `check` not green, a non-idempotent re-run — **reverts to `2fc25f5` +
finding**, not forced. Investigate-before-firing; clear every deviation before the irreversible step.

## Exit

`ledger.md` closed; CDC-verified against the committed store. odm's own corpus is faithful end-to-end and
the modeled vision is live. On close, bubble up to `../arc-plan.md`: MF-7 done (vision live); MF-3/MF-9's
fidelity is now true on the live corpus; **the arc-close is next** — MF-9 composition + the P-12 self-host
acceptance demonstration + the final reconcile-and-freeze — after which **Migration Fidelity closes** and
the self-host DoD is demonstrable.
