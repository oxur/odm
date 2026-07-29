# Slice 09 (Migration Fidelity): Coverage enforcement *capability*

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced` at its scale.
> **This slice is fixture-only — no live mutation.** Capability rows are fixture-proven
> (`attested` → CI); CDC reproduces by reading the code + fixtures and on CI. There is **no live
> class-(b) row** here (the live mint + backfill are s10). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **`artifact` node type exists + validates**: `NodeType::Artifact` variant + `artifact/v1.0` schema marker + per-type field validity (ODD-0020 model) | `cargo test -p odm-core` → an `artifact` node round-trips parse/emit; an invalid per-type field is a `check` Error; unknown-schema handling per 0020 | serious | ODD-0025 §2.5 / ODD-0013 §2.2 | open | | Documented in 0013 v2.4 / 0020 v1.2; this slice lands the **code** variant. American spelling `artifact`. |
| F-2 | **Artifact containment = nearest modeled scale** (§2.5): a per-slice artifact is `part_of` its slice; an arc-/chunk-level artifact is `part_of` its **arc**; no `chunk`/`step` node scale | `cargo test` → fixture with a per-slice `ledger.md` and an arc-level report → the former `part_of` the slice node, the latter `part_of` the arc node | serious | ODD-0025 §2.5 | open | | Chunk-level (`cN-*`) attaches to the arc, not a new scale. |
| F-3 | **Discovery reaches + mints the artifact family, mint-all** (§2.6): `ledger`/`cc-prompt`/`cdc-verification`/`closing-report`/ADR/amendment/UAT + **every report incl. `coverage-report.md`** minted as `artifact`, 1:1 body + `source`, hard-gated | `cargo test` → fixture plan-set with supporting docs → each minted `artifact`; no doc exempted; body-hash gate passes on the faithful bodies | serious | ODD-0025 §2.6 (F10 mint-all) | open | | No ignore rule — a report node's body just updates on regen (§2.1 stores no hash). |
| F-4 | **Design/research family reachable + `source` backfilled** (MF-3 residual, F7): the 14 `design`/`research` nodes acquire a `source` sub-map; discovery no longer structurally excludes that type family | `cargo test` → fixture with a `design` + a `research` doc → both migrate with `source`, body-hash-gated; containment **optional** (top-level allowed) | serious | arc-plan MF-3 / F7 | open | | Today `discover()` can't reach this family — that's the gap this closes. |
| F-5 | **Doc-coverage wired into `odm check` as an Error** (MF-1/MF-6): any `.md` under the scan root with no covering node fails `check`; built on s08's portable relative `source.paths` key | `cargo test` → seed an uncovered `.md` → `check` returns Error naming it; cover it → green; result identical from two `TempDir` roots | serious (loud-hole guard) | ODD-0025 §5 / arc MF-6 | open | | Rule implemented + fixture-proven here; **live activation is s10** (pre-mint the live corpus has uncovered docs by construction). |
| F-6 | **coverage.rs Finding 2 fixed**: `representation()` counts a **named** arc (and its slices) as represented via the s05 name-derived key, not only `arc_coordinate()`; summary reads a truthful 12/12 | `cargo test` → fixture with a named arc that **has** a node → `representation()` reports it represented (0 missing), not "N−4/N" | correctness (report clarity) | CDC v2.8 Finding 2 | open | | The 4 named arcs currently always read unrepresented — the "8/12" gap. |
| F-7 | **coverage.rs Finding 3 fixed**: `provenance_absence` no longer scans the stale stored-`provenance:` key; retargeted to **`source:` presence** (or retired for the source-presence check) — decided + justified | `cargo test` → a node **with** `source:` is not flagged; a node missing `source:` is; no scan for `provenance:` remains | correctness | CDC v2.8 Finding 3 | open | | Per §2.0 `provenance` is derived-only, never stored — scanning for it always misfires. |
| F-8 | **Optional containment honored** (§2.7): the doc-coverage / `orphan` check does **not** flag a legitimately top-level `design`/`research`/`artifact` node as an orphan | `cargo test` → a top-level doc node → not reported as an orphan or uncovered | correctness | ODD-0025 §2.7 | open | | s01 confirmed the "14 floaters" are mostly the intended shape. |
| F-9 | **No live mutation; downstream not pulled forward**: no `odm`-branch commit; s10 (live mint/backfill), s11 (synthesis/L-8b), s12 (reconcile) untouched; ODD amendments implemented-against, not re-litigated | `git -C .worktrees/odm status` clean after the slice; grep: no synthesis/reconcile code added; ODD-0025/0013/0020 unedited unless a new line was genuinely needed (then amended, cited) | correctness | LEDGER-DISCIPLINE / operator | open | | The whole live run is s10 by design — this row guards the boundary. |
| F-10 | **Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) changed modules, target 95%; no model drift** | `clippy` exit 0; `! grep unsafe` on changed files; `llvm-cov` ≥ 90%; cross-read: `source` still the identity axis, body-hash gate unchanged, `artifact` per ODD-0025 §2.5 — amend-not-work-around if a model line is needed | polish/correctness | CLAUDE.md / ODD-0025 | open | | Same bar as s08. |

## What Worked

_(At slice close. Patterns that made the slice close cleanly.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` (`release/1.0.x`, capability) on `<date>`.
Verified by: `<CC then CDC>`. Rows: 10. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
On close, bubble up to `../arc-plan.md`: enforcement capability lands; **s10 (coverage live run)
unblocked + next**; MF-1/MF-6 → "capability done, live-pending".
