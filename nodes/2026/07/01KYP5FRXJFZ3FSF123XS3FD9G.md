---
id: 01KYP5FRXJFZ3FSF123XS3FD9G
number: 58837410
type: slice
schema: slice/v1.1
name: Slice 10 (Migration Fidelity) — Coverage live run (plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice10-coverage-live-run/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
---
# Slice 10 (Migration Fidelity) — Coverage live run (plan-of-record)

> Refs: `../arc-plan.md` (s10 row; MF-1/MF-3/MF-5/MF-6; F7/F10) · **s09** (the enforcement capability
> this fires) · **s07** `slice07-live-run/` (the live-mutation protocol this reuses) · **s08**
> `slice08-source-path-portability/` (the portable `source.paths` key + the s07→s08 known-good tip) ·
> ODD-0025 §2.5/§2.6/§2.7 (artifact type, mint-all, optional containment). `depends_on:` s09.
>
> **The live corrective run for coverage — the arc's second live mutation** (the s06/s07 shape: s09
> built the capability fixture-only; s10 fires it). It mints the supporting-doc corpus, backfills the
> design/research `source`, and turns coverage enforcement **on** against odm's own store.

## Goal

Make "no file left behind" **true and enforced on the live `.worktrees/odm` corpus**: mint an `artifact`
node for every supporting doc (mint-all, ODD-0025 §2.6), backfill `source` onto the 14 design/research
nodes, and **activate the doc-coverage check** so `odm check` fails on any uncovered `.md` — all as one
revertible commit behind the snapshot → dry-run → adjudicate → fire → verify protocol. **Done when** the
live store carries a faithful, body-hash-gated `artifact` node for every supporting `.md` (incl. every
report and `coverage-report.md`), the 14 design/research nodes carry `source`, the doc-coverage gate is
**on** and `odm check` is **green** (0 uncovered), `orient`/`rollup` are byte-stable, the run is
idempotent, and project + retired remain untouched.

## Why

s09 proved the enforcement machinery on fixtures but deliberately did **not** touch the live store,
because activating the check before the nodes exist would make `odm check` red on the real corpus (~211
uncovered docs per s01's inventory). s10 is where the two halves meet: the mint **and** the gate flip
land **together**, atomically, so the store moves from "coverage green because the gate is off" to
"coverage green because every doc is now covered" with **no red window**. This is the moment MF-1/MF-6
go from "capability exists" to "property enforced" — the arc's fourth faithfulness property
(no-file-left-behind) becomes a check that fires, not a claim that's trusted.

## Scope

**In (one revertible commit on the `odm` branch, atop the s08 known-good tip `7226797`):**

- **Mint the artifact corpus (mint-all, §2.6).** Every supporting `.md` under the doc-coverage scan root
  — `ledger` / `cc-prompt` / `cdc-verification` / `closing-report`, ADRs, amendments, UAT docs, **and
  every generated report including `coverage-report.md`** — gets a faithful 1:1 `artifact` node, body
  hard-gated (§2.1), `part_of` its nearest modeled scale (§2.5: per-slice → slice; arc-/chunk-level →
  arc). No exemption, no ignore rule.
- **Backfill the design/research `source` (MF-3 residual, F7).** The 14 `design`/`research` nodes acquire
  their `source` sub-map via s09's extended discovery, body-hash-gated, containment optional (§2.7) —
  built on s08's portable relative key.
- **Activate the doc-coverage gate.** Flip doc-coverage **on** in the live operational gate-set
  (`.worktrees/odm/config.toml`, per the C-5 operational-config split) so `odm check` treats an
  uncovered `.md` as an **Error** — green only because the mint now covers everything.
- **The full s07 live protocol.** Confirm `release/1.0.x` green + the `odm` store worktree clean;
  capture the **known-good SHA + before-manifest**; **`--dry-run`** → **adjudicate** (the creates match
  s01's supporting-doc inventory; the modifies are exactly the 14 design/research nodes; **0 unexpected
  change** to the 62 existing source-bearing nodes, project, or retired; store fingerprint stable after
  the dry-run) → **fire as one revertible commit** → **verify**.

**Out:**

- The enforcement **capability** itself (the `artifact` type, discovery reach, check rule, coverage.rs
  Findings 2–3) — that is **s09**, merged first on `release/1.0.x`. s10 writes no capability code beyond
  what wiring the live gate-set requires; a capability gap found here is an s09 fix, not new s10 scope.
- **Synthesis + L-8b** — **s11**.
- **The arc-close reconcile run** — **s12**, including the **living-doc-drift reconcile** the s08 CDC
  verification surfaced (the active arc node's body lags its living `arc-plan.md`; s12 must treat a
  legitimate source change as re-snapshot, and decide whether a living-plan arc node is reconciled
  on-edit or excluded like the project synthesis node). **Note:** this s10 mint will itself create the
  drift's mirror at scale — every minted report node is a faithful snapshot the instant it's written,
  and any later edit re-opens the same gap; s12 owns the reconcile, s10 only records it.
- Any change to what `source` means or to the body-hash gate (unchanged).

## Verification

Live, class-(b) — the committed store is the evidence; CDC reproduces by direct read + independent
recomputation (the s07/s08 method). After the committed run:

- **Coverage:** doc-coverage set-difference = **0 uncovered** `.md` under the scan root; the gate is
  **on**; `odm check` **exit 0** with doc-coverage enforcing.
- **Mint fidelity:** every minted `artifact` node has a body that byte-matches its source under
  `normalize` (trim+lf); `part_of` resolves to the correct nearest modeled scale; `coverage-report.md`
  has a node (no exemption).
- **Design/research:** the 14 nodes now carry `source`; none is flagged orphan (optional containment).
- **No collateral:** the 62 pre-existing source-bearing nodes, project (`#1000`), and retired (`#1605`)
  are **unchanged** (spot-checked by last-touch commit); ids/bodies/schema of existing nodes untouched;
  `orient`/`rollup` byte-stable across 2 runs; a re-run is **idempotent** (0 created / 0 reconciled) and
  cross-root stable (the s08 key).

`check`/`orient`/`rollup`/`migrate` runtime rows are attested-by-CC → CI (macOS binaries); CDC
reproduces store state (mint count, body-hash faithfulness on a sample+, containment, exclusions,
0-uncovered) by direct read.

## Rollback & findings discipline

Same spine as s07/s08. **Hard gates:** snapshot SHA + before-manifest are non-negotiable; **dry-run
first**. The dry-run's **creates are expected** here (unlike s08) — the gate is *adjudication*, not
"0 creates": the created set must match s01's supporting-doc inventory (count + identity), the modified
set must be exactly the 14 design/research nodes, and **nothing else** may change. Any deviation —
an unexpected create/modify, a body-hash failure on a mint, a re-mint of an existing node, `check` not
reaching green after the mint, a non-idempotent re-run — **reverts to the captured SHA and becomes a
finding**, not forced. Investigate-before-firing (the s07/s08 discipline), and clear every deviation
before the irreversible step.

## Exit

`ledger.md` closed; CDC-verified against the committed store. Coverage is enforced on odm's own corpus;
"no file left behind" is now a check that fires. On close, bubble up to `../arc-plan.md`: MF-1/MF-6
**done** (enforced live); MF-3 **done** for the full criterion (design/research `source` landed);
**s11 (synthesis + L-8b) unblocked and next**; the living-doc-drift reconcile confirmed routed to s12.
