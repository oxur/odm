---
id: 01KYNDTQ6SXTKPJ2GN9SXYEMMN
number: 58837407
type: slice
schema: slice/v1.1
name: Slice 07 (Migration Fidelity) — Live repair run (plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc-migration-fidelity/slice07-live-run/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
---
# Slice 07 (Migration Fidelity) — Live repair run (plan-of-record)

> Refs: `../arc-plan.md` v2.4/v2.5 (s06 closed + CDC PASS; the doc-comment finding this slice opens
> with); `slice06-live-run-capability/{closing-report,cdc-verification}.md` (the CDC-verified capability
> this fires); ODD-0025 §2.1 (the hard body-hash gate), §2.2 (`source` on every migrated node), §2.3
> (synthesis/project exclusion), §2.8 (update-in-place). `depends_on:` s06 (the invocable, one-policy,
> `--dry-run`-able `odm migrate` flow).
>
> **This is the arc's FIRST live mutation.** Every slice before it was fixture-only. It fires the
> s06-built, CDC-verified flow on the **real** `.worktrees/odm` store — behind a snapshot/revert guard
> and a dry-run-first protocol. The capability is verified; this slice is about running it *safely* and
> *proving the live outcome*.

## Goal

Bring odm's own self-hosted corpus from **44 stubs / 0 provenance / 6 of 12 arcs** to a faithful,
fully-represented, source-bearing plan graph — by running `odm migrate` on the live store — and
**prove** the outcome. **Done when** the live store has **zero stub bodies**, **every arc/slice node
carries `source`** (the synthesis project node excluded, the retired node excluded), **all 12 plan-tree
arcs and every plan slice are represented**, **every body-hash re-verifies**, `odm check` is green,
`orient`/`rollup` reproduce, and `context.json` is correctly pointed — all committed as **one
revertible commit**, with any gate failure rolled back to the captured known-good SHA rather than
forced.

## Baseline (live store, measured 2026-07-28, `git -C .worktrees/odm`)

60 nodes: 1 project, 6 arc, 40 slice, 9 design, 5 research. **0 source-bearing.** All `schema: */v1.0`.
**44 stubs** (6 arc + 38 slice — every present arc is a stub; only the project and 1 slice are
non-stub). `context.json → arc 01KWXMBBTKNA3A0QC3SWPHBNAX`. Plan tree (`docs/design-v1.0.0`): 12
arc-plans, 49 slice-docs, 1 project-plan. CC re-captures this as the before-manifest at run time (the
store may have moved) and adjudicates the dry-run against it.

## Scope

**In:**

- **Open with the s06 doc-comment correction** (CDC v2.5 finding). `self_host_inner`'s doc comment
  calls the repair→import order a load-bearing *correctness* invariant; s06 proved it isn't (both
  orderings converge — correctness rests on `reconcile_source`, run both ways). Correct the comment to
  say what's true (order is for composability + report attribution) before touching the live store. A
  one-line-ish doc fix on `release/1.0.x`; no behavior change.
- **Snapshot / revert guard (HARD).** Confirm `release/1.0.x` green and the `.worktrees/odm` store
  worktree clean (`git status --porcelain` empty). Capture the current store HEAD SHA and a
  before-manifest (node count, id set, per-node body-hash, `source` count, schema spread,
  `context.json`). This SHA is the known-good state the whole run reverts to on any failure.
- **Dry-run first (HARD).** Build `odm` from `1.0.x`; run `odm migrate <plan-root> --dry-run` against
  the live store. Inspect the preview — reconciled / created / skipped counts, **zero** drift errors —
  and adjudicate it against the before-manifest *before* firing (expect ≈47 plan nodes reconciled: 44
  stubs repaired + faithful backfilled, project + retired excluded; the 6 missing arcs + their slices
  created). Confirm the dry-run left the live store **byte-identical** (`git status` still clean).
- **Fire the run, as one revertible commit.** Run `odm migrate <plan-root>` for real; commit the store
  worktree as a **single** commit atop the captured SHA (message names the slice + the before/after
  deltas). The whole mutation is one `git reset --hard <SHA>` away from undo.
- **Verify the live outcome** (the heart of the slice — see Verification).
- **Re-point `context.json`** as the run dictates (operator focus preserved or advanced deliberately,
  not incidentally clobbered).

**Out:**

- **The 14 design/research nodes' `source` backfill, and loose-doc coverage** (the ~211 uncovered
  docs). `discover()` structurally produces only project/arc/slice PlanNodes, so `repair`/`self_host`
  cannot reach design/research nodes — and `validate` does **not** require `source`, so those
  source-less nodes do **not** fail `check`. Their `source` + the coverage-into-`check` enforcement is
  **s08**. This slice leaves a deliberate, disclosed interim: a store with `v1.1` source-bearing plan
  nodes alongside untouched `v1.0` design/research nodes (a mixed store is valid — `v1.0 < CURRENT` is
  not a schema error).
- The **project node's synthesis re-cast** (s09 — this slice only *excludes* it from 1:1 `source`).
- The **arc-close reconcile demonstration** (s10). `artifact`-node minting (s08).

## Verification

The store state **is** the evidence (a live, class-(b) row — LEDGER-DISCIPLINE v2.0 §B; this is where
MF-2/MF-3/MF-5 finally reproduce at arc scale, not on fixtures). After the committed run:

- **`odm check` exit 0** — `validate` then `reconcile` both green.
- **Zero stub bodies** among plan nodes (`is_stub_body` false for every arc/slice/project node).
- **Every arc/slice node carries `source.paths`**; the **project node carries no 1:1 `source`**
  (ODD-0025 §2.3); the **retired/tombstone node** (`design-notes.md` §1) is **excluded**.
- **Full representation**: all 12 plan-tree arcs + every plan slice resolve to a node — coverage
  set-difference over `source.paths` = **0 uncovered arcs/slices** (`odm migrate --coverage`); the 6
  previously-missing arcs and their slices are present.
- **Every body-hash re-verifies**: a second `reconcile`/`migrate` surfaces **0** `BodyHashMismatch`;
  migrated nodes stamped `schema: */v1.1`.
- **`orient`/`rollup` reproduce** deterministically (byte-stable on re-run).
- **Re-run idempotent**: a second `odm migrate` reports 0 reconciled / 0 created; node count + id set
  unchanged.

CDC reproduces the store-state invariants by **direct read** of the committed store (source presence,
0 stubs, schema, project source-less, coverage set-difference — recomputing the normalize+hash gate
independently in the CDC VM); the `check`/`orient`/`rollup` runs are attested-by-CC → reproduced on
re-run / CI.

## Rollback & findings discipline (this slice's spine)

**Let it crash, then recover cleanly.** Any gate failure — a real `BodyHashMismatch` on a node the
corpus calls "faithful" (a body hand-edited away from its source, or a moved source path), an
unexpected create, `check` red, a non-idempotent re-run — **stops the run**: revert the store commit to
the captured SHA (known-good), file the failure as a **finding**, and do not force past it. A live
gate failure is a *discovery about the corpus*, surfaced and adjudicated (fix the source mapping, or
accept + document the drift), **never suppressed** to make the run "pass." Hidden failure is the only
unacceptable outcome.

## Exit

`ledger.md` closed; CDC-verified against the committed live store. The self-hosted corpus is faithful,
source-bearing, and fully represented — odm can now credibly claim to self-host (the P-12 DoD path).
On close, bubble up to `../arc-plan.md`: **MF-2/MF-3/MF-5 move `planned → done`** (their live-corpus
outcome is now reproduced, per §B), with the residual design/research-`source` + loose-doc coverage
gap explicitly handed to **s08**.
