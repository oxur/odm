---
id: 01KYP5FRXJBKY1GHW4QF9JQZD7
number: 58837409
type: slice
schema: slice/v1.1
name: Slice 09 (Migration Fidelity) — Coverage enforcement *capability* (plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice09-coverage-enforcement/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
status:
  built:
    reached: 2026-07-29
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-29
  planned:
    reached: 2026-07-29
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-29
  tested:
    reached: 2026-07-29
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-29
---
# Slice 09 (Migration Fidelity) — Coverage enforcement *capability* (plan-of-record)

> Refs: `../arc-plan.md` (s09 row; MF-1/MF-3/MF-5/MF-6; F7/F10) · **ODD-0025** §2.5 (`artifact` type),
> §2.6 (mint-all reports), §2.7 (optional containment for doc nodes), §2.2 (`source` sub-map) · ODD-0013
> §2.2 (artifact documented, v2.4 — *code variant pending, this slice*) · ODD-0020 (`artifact/v1.0`
> named, v1.2 — *code pending, this slice*) · s05 (source-based identity + name-derived arc handle) ·
> s08 (portable relative `source.paths` — the key this check is built on). `depends_on:` s08.
>
> **Capability slice — fixture-only, no live mutation** (the s06/s07 unbundle pattern). It builds the
> machinery that makes "no file left behind" a *mechanically enforced* property; the **live mint + live
> backfill fire in the follow-on live-run slice** (proposed **s10**; see `../arc-plan.md` split note).

## Goal

Make coverage **enforceable** — turn the s01 read-only detector into a real `odm check` rule, and give
the migrator the node model + discovery reach it needs to leave no `.md` uncovered — **all in code,
fixture-proven, with zero mutation of the live `.worktrees/odm` store.** **Done when** the `artifact`
node type exists and validates; `discover()` reaches both the artifact doc family (ledger / cc-prompt /
cdc-verification / closing-report / ADR / amendment / UAT) **and** the design/research family; a seeded
uncovered `.md` makes `odm check` **fail with an Error**; coverage.rs Findings 2–3 are fixed (the
report reads a truthful 12/12, and the stale `provenance:` detector no longer misfires); and every one
of the above is proven by a `TempDir`/fixture test — **the live corpus is untouched, and turning the
enforcing check on against it is s10's job.**

## Why (the property, and why capability-first)

"No file left behind" (MF-1) is one of the arc's four faithfulness properties, and the only one still
*trusted rather than checked*: s01 built a detector that **reports** the gap, but nothing **fails** on
it. Until `odm check` errors on an uncovered doc, a new `.md` can be added and silently escape the
store — the exact class of drift the arc exists to kill. Enforcing it needs three things the store does
not have yet: a node type for the ~211 supporting docs (`artifact`, ODD-0025 §2.5), discovery that can
*reach* them (and the 14 design/research docs `discover()` structurally cannot see today), and the
coverage set-difference wired into `check` as an Error.

**Capability-first, live-run-second** is the arc's established shape (s06 built the live-run flow;
s07 fired it) and the operator's stated preference. It matters more here than anywhere: this slice
**mints a new node type and adds a check that fails on uncovered docs**. If the enforcing check went
live *before* the ~211 artifact nodes were minted, `odm check` would go **red on the live corpus** (211
uncovered docs). So the capability is proven on fixtures here; the live mint **and** the check's
live activation land together in s10, behind the snapshot → dry-run → fire protocol — the same
discipline s07/s08 used, applied to a much larger mint than a 61-path rewrite.

## Scope

**In (all code + fixtures; no live mutation):**

- **The `artifact` node type.** Add the `NodeType::Artifact` variant (`odm-core/src/node_type.rs`) +
  the `artifact/v1.0` schema marker (ODD-0020, already named) + per-type field validity (ODD-0020
  shared-core / per-type model) + the §2.5 **containment rule**: an `artifact` is `part_of` its
  **nearest *modeled* scale** — a per-slice artifact → its slice; an arc-level **or chunk-level**
  artifact → its **arc** (no `chunk`/`step` node scale, §2.5). Document-family: containment is
  **optional** (§2.7).
- **Discovery reach — artifact family.** Extend `discover()` / the migration walk to find the supporting
  docs (`ledger.md`, `cc-prompt.md`, `cdc-verification.md`, `closing-report.md`, ADR, amendment, UAT)
  and mint each as an `artifact` node with a **faithful 1:1 body** + `source` sub-map, under the §2.1
  hard body-hash gate, `part_of` its nearest modeled scale. **Mint-all, no exemption** (§2.6):
  `coverage-report.md` and every generated report included.
- **Discovery reach — design/research family.** The 14 `design`/`research` nodes carry **no `source`
  today** because `discover()` cannot reach that type family (arc-plan MF-3 note). Extend discovery /
  backfill so those nodes acquire their `source` (MF-3's residual scope, F7), body-hash-gated like any
  other. Their containment stays **optional** (§2.7) — a legitimately top-level design/research node is
  **not** an orphan.
- **Wire doc-coverage into `odm check`** (MF-1, MF-6). The s01 set-difference becomes a `check` rule
  that returns an **Error** for any `.md` under the doc-coverage scan root with no covering node. Built
  on **s08's portable relative `source.paths` key** so it resolves in CI, not just the authoring
  machine. The rule is implemented and fixture-proven here; its **activation on the live store is s10**
  (post-mint), so `check` on the real corpus is not made red by this slice.
- **coverage.rs Finding 2** (CDC v2.8) — `representation()` resolves a **named** arc's representation via
  its s05 **name-derived stable key**, not only via `arc_coordinate()`'s numbered parse, so the summary
  line reads a truthful **12/12** instead of "8/12" (the 4 named arcs — release-hardening, store-home,
  llm-command-surface, migration-fidelity — stop reading as unrepresented). Same fix for named-arc
  slices.
- **coverage.rs Finding 3** (CDC v2.8) — `provenance_absence` currently scans for a stored `provenance:`
  frontmatter line, but ODD-0025 §2.0 renamed the stored record `provenance → source` (provenance is
  **derived-only, never stored**), so the detector misfires on every node. **Retarget it to `source:`
  presence** (the real MF-3 property — "every node carries a source record"), or retire it in favor of
  the source-presence check — **decide + justify + test**.
- **Fixtures** (`TempDir`) for all of the above: artifact mint incl. nearest-scale containment and
  mint-all-incl-reports; design/research `source` backfill; the enforcing check (**seed an uncovered
  `.md` → `check` Error**; cover it → green); Finding 2 (named arc reads represented); Finding 3
  (`source`-presence check, not the stale key); optional-containment (a top-level doc node is not an
  orphan); cross-root determinism on the coverage key (built on s08).

**Out:**

- **The live run — proposed s10.** The live mint-all of `artifact` nodes on `.worktrees/odm` (~211 per
  s01's inventory), the live design/research `source` backfill, and **flipping the enforcing check on
  against the live corpus** — all behind snapshot → dry-run → adjudicate → fire → verify. This slice
  writes **no** commit to the `odm` branch.
- **Synthesis + L-8b** (old s10 → **s11** under the split).
- **The arc-close reconcile run** (old s11 → **s12**), including the **living-doc-drift reconcile** the
  s08 CDC verification surfaced (the active arc node's body lags its living `arc-plan.md`; reconcile
  must treat a legitimate source change as re-snapshot, not drift-to-reject — and decide whether a
  living-plan arc node is reconciled-on-edit or treated specially like the project synthesis node).
- Any change to what `source` **means** (still the identity axis) or the body-hash gate (unchanged). The
  ODD-0013/0020 artifact amendments are **already documented** — implement against them; if a *new*
  model line is needed, **amend, don't work around**.

## Verification

Fixture / `TempDir`, class-(a) (fixture-attested → CI). **No live class-(b) row in this slice** — the
live store is not touched. After the slice:

- `NodeType::Artifact` + `artifact/v1.0` exist and validate; an `artifact` fixture node is `part_of` its
  nearest modeled scale.
- A fixture corpus with a supporting doc + a design/research doc migrates → both are covered; a **seeded
  uncovered `.md` → `odm check` Error**, and covering it → green.
- The coverage report reads **12/12** on a fixture with a named arc; the `source`-presence check
  replaces the stale `provenance:` scan.
- Cross-root determinism holds on the coverage key (two `TempDir` roots → identical result).

CDC reproduces the code + fixtures structurally + on CI (no live store to read this slice).

## Rollback & findings discipline

Fixture-only, so there is nothing to revert on the live store. A capability gap found in review is a
normal fix-iteration (five-cap). If the capability genuinely will not fit one context even without a
live run, **split further and flag CDC** — but the live mint/backfill is already carved out to s10, so
the remainder should fit. Any discovered model gap → **amend ODD-0025/0013/0020, don't work around**.

## Exit

`ledger.md` closed; CDC-verified against the code + fixtures + CI. The store can now *mechanically
enforce* coverage: the type, the discovery reach, and the check rule all exist and are proven. On close,
bubble up to `../arc-plan.md`: the enforcement capability lands; **s10 (coverage live run) is unblocked
and next**; MF-1/MF-6 move from "planned" to "capability done, live-pending."
