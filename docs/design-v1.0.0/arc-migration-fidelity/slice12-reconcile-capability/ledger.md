# Slice 12 (Migration Fidelity): Reconcile capability

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. **Fixture only —
> no live store mutation** (the live close is s13; the arc-close follows). Capability rows are
> fixture-proven (`attested` → CI). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **Re-snapshot mode**: a drifted non-stub is **updated in place** (body ← current source, `id`/`edges`/`status` preserved, §2.1 gate re-passes) — no longer skipped-and-recorded | `cargo test` → seed a node whose source drifted → reconcile updates its body to match, id/edges/status unchanged; the `drifted`-skip path no longer leaves it unfixed | serious | s06 `reconcile_source` / ODD-0025 §2.8 | open | | Keep s04 stub-repair distinct from this non-stub re-snapshot. |
| F-2 | **Moved-source re-discovery**: a node whose stored `source.paths` no longer resolves but whose source moved is re-found **by identity** and its `source.paths` rewritten to the **s08-relative** new path, then re-snapshot | `cargo test` → seed a node whose source moved (`01-draft/`→`04-accepted/` shape) → reconcile finds it by number, updates `source.paths` (relative, canonical), re-snapshots body | serious | s11 L-8b moves / s08 | open | | One node can need both a moved path and a changed body (0013/0017/0018). |
| F-3 | **Living-plan-node policy decided + implemented**: how a node whose source is a still-changing doc reconciles — reconcile-to-current, no special exclusion (gate is migration-time-only, §2.1); decision recorded, ODD-0025 amended if a line is needed | `cargo test` → a living-source node reconciles to current without reject; doc/ODD records the policy + rationale | serious | s08 open question | open | | The arc node is faithful once reconciled at arc-close (source then stable); inter-reconcile drift is by-design invisible to `check`. |
| F-4 | **Vision-apply path fixture-proven** (MF-7 live half, built here): the project node re-casts as an **editorial-merge synthesis** superseding a **1:1 `project-plan` node**, using s11's `synthesis` | `cargo test` (`TempDir`) → the apply path yields the synthesis + the faithful 1:1 node + the supersede edge; **no live write** | serious | MF-7 / s11 | open | | s13 fires it live. |
| F-5 | **Stub-repair vs non-stub re-snapshot are distinct + both correct** | `cargo test` → a stub node → s04 repair path; a drifted non-stub → re-snapshot path; neither silently does the other | correctness | ODD-0025 §2.8 | open | | |
| F-6 | **Reconcile is idempotent + dry-run-safe**: a second reconcile of an already-faithful node is a no-op; `--dry-run` writes nothing | `cargo test` → re-snapshot twice → second run 0 changes; dry-run → store fingerprint unchanged | serious | s07/s08 protocol | open | | The property s13's dry-run gate relies on. |
| F-7 | **ODD-0025 §4 decision resolved**: either amend §4 (`artifact` stamps the shared `v1.1`) — making #25 a re-snapshot target for s13 — or an explicit recorded deferral; not left dangling | direct review: §4 amended (dated version-history entry) **or** a recorded decision to defer with rationale | correctness | s09 CDC LOW follow-up | open | | Amending §4 drifts node #25 (ODD-0025 is its source) → s13 re-snapshots it. |
| F-8 | **No live store mutation; downstream not pulled forward**: `.worktrees/odm` untouched; the live reconcile + vision mint stay s13; the MF-9 composition + P-12 demo + arc-close stay post-s13; L-8a not started | `git -C .worktrees/odm status` clean; no reconcile/vision fired live; no composition/P-12 run | serious | LEDGER-DISCIPLINE / operator | open | | This slice writes no `odm`-branch commit. |
| F-9 | **No model drift; amend-not-work-around**: the 1:1 rule + the migration-time-only gate unchanged; re-snapshot re-establishes fidelity, it does not make the gate continuous; any model line (re-snapshot semantics, living-node policy) is an ODD edit, cited | cross-read: gate still migration-time-only; reconcile only ever sets a body *from* its current source, identity preserved | correctness | ODD-0025 | open | | |
| F-10 | **Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) changed modules, target 95%** | clippy exit 0; `! grep unsafe`; `llvm-cov` ≥ 90% | polish/correctness | CLAUDE.md | open | | Same bar as s08–s11. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` (`release/1.0.x`) on `<date>`. Verified by: `<CC then CDC>`.
Rows: 10. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`. On close, bubble up to `../arc-plan.md`: MF-9's
mechanism ready; **s13 (live reconcile + vision mint) next**; the arc-close (MF-9 composition + P-12 demo)
follows s13.
