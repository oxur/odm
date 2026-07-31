---
id: 01KYTVE43MWKRXZ0YJB9GH4V8D
number: 500884000
type: artifact
schema: artifact/v1.1
name: Slice 12 closing report — Reconcile capability
created: 2026-07-29
updated: 2026-07-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice12-reconcile-capability/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-31
edges:
  part_of: 01KYSX4R1GPDKH7M5ANCTDND04
---
# Slice 12 closing report — Reconcile capability

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 12 · **Feeds:** MF-9 (mechanism) ·
> **Realizes:** ODD-0025 §2.1/§2.8 (extended by new §2.9), §4 (resolved) · **Assignment:** `cc-prompt.md`
> · **Ledger:** `ledger.md` (F-1…F-10) · **Implemented by:** CC · **Date:** 2026-07-29
> **Branch:** `release/1.0.x` only — one commit, `3daf893`. **No `odm` branch commit this slice** —
> fixture-only, per the hard rule; the live close is s13. **Evidence class:** fixture-attested
> (LEDGER-DISCIPLINE v2.0 §B class-(a)) throughout; there is no class-(b) row — nothing in this slice
> touches the live store.

## What shipped

**Re-snapshot mode, both node families (F-1, F-5):** a drifted non-stub node is now **updated in
place** instead of skipped-and-recorded or hard-rejected. Design/research:
`mapping::backfill_source`'s drift branch (previously: record `Drifted`, leave the node completely
untouched — the exact ODD-0013/ODD-0020 shape s10 found live) now re-snapshots the body from the
current legacy content, reported in a new `reconciled` bucket distinct from `repaired` (stub-replace) —
same mechanism, different report bucket, never conflated. Work-tree: the existing private
`selfhost::reconcile_source` gained a `force_resnapshot: bool` — `false` preserves `repair()`'s
unchanged, still-tested policy (hard-fail on a drifted non-stub, the F-5 stub-vs-reconcile
distinctness); `true` is the new re-snapshot policy the new `reconcile()` function uses. `id`/`edges`/
`status` are preserved in both families — only `body`/`source`/`updated`/schema change.

**Moved-source re-discovery, both node families (F-2):** a node whose stored `source.paths` no longer
resolves is re-found by the corpus's own identity key — `number` for design/research
(`mapping::reconcile_source`, new), the structural `(type, number)` coordinate for work-tree nodes
(`selfhost::reconcile`, new) — and its `source.paths` rewritten to the s08-relative canonical form of
the new location. The sourceless case (`backfill_source`) already re-discovers by number for free,
since `legacy::discover` walks the *current* tree rather than remembering a path. A node needing both a
moved path and a changed body — the exact live shape ODD-0013/0017/0018 are in after s11's L-8b moves —
is handled by both functions in one pass.

**The living-plan-node policy, decided and implemented (F-3):** reconcile-to-current, no special
exclusion for a still-changing source — the slice-doc's own recommendation, adopted as drafted. Proven
by fixture (reconcile the same node twice, with its source edited in between; both runs succeed and
pick up the latest content), not just asserted. Recorded in a new ODD-0025 §2.9.

**The vision-apply path, promoted to reusable code (F-4):** `synthesis::apply_project_vision` lifts s11's
inline fixture mechanism into library code any caller (s13) can call directly — given an existing 1:1
`project-plan` node's identity/source/body and `plan_root`, it extracts the vision text
(`replan::vision_from_plan`, unchanged) and builds the editorial-merge synthesis over it. No I/O beyond
that one read; never persists.

**ODD-0025 §4 resolved (F-7):** `artifact/v1.0` corrected to the delivered `artifact/v1.1`, deferred in
s10 iteration 1 specifically because no reconcile mechanism existed yet to absorb node #25's resulting
drift — this slice building that mechanism is what makes resolving it safe now.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | Re-snapshot mode | **done** | Both families; id/edges/status preserved; fixture-proven against the s08/s10 concrete shapes. |
| F-2 | Moved-source re-discovery | **done** | Both families; the 0013/0017/0018 both-moved-and-drifted shape covered. |
| F-3 | Living-plan-node policy | **done** | Reconcile-to-current, no exclusion; proven by repeated-edit fixtures; ODD-0025 §2.9 added. |
| F-4 | Vision-apply path | **done** | `apply_project_vision`, promoted from s11's inline mechanism; fixture-proven, no live write. |
| F-5 | Stub-repair vs reconcile distinct | **done** | `force_resnapshot` flag; `repair()`'s existing tests pass unmodified. |
| F-6 | Idempotent + dry-run-safe | **done** | Both families; dry-run writes nothing, second real run is a 0-change no-op. |
| F-7 | ODD-0025 §4 resolved | **done** | Amended to `artifact/v1.1`; disclosed as an s13 re-snapshot target for node #25. |
| F-8 | No live store mutation | **done** | `.worktrees/odm` unchanged throughout, still `2fc25f5`. |
| F-9 | No model drift | **done** | 1:1 rule + migration-time-only gate unchanged; ODD-0025 §2.9 is the one model addition. |
| F-10 | Clippy/unsafe/coverage | **done** | Clean throughout; mapping.rs 92.72%, selfhost.rs 94.11%, synthesis.rs 98.93% lines. |

**Rows: 10. Done: 10. Deferred: 0. No-op: 0.** No silent drops: the live reconcile + vision mint (s13),
the MF-9 composition + P-12 demo + arc-close (post-s13), and L-8a (post-1.0) are all confirmed
untouched (F-8).

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace --all-features` | 0 failed, checked after every code checkpoint |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean (only the pre-existing, unrelated `proc-macro-error2` notice) |
| `cargo fmt --check` | clean |
| `unsafe` in every file this slice touched (9 files) | none |
| `llvm-cov --workspace` | `mapping.rs` 92.72%, `selfhost.rs` 94.11%, `synthesis.rs` 98.93% lines — all above the 90% floor |
| `.worktrees/odm` status | clean, still `2fc25f5` — untouched throughout |
| Pre-existing `repair()`/`backfill_source` idempotence tests | pass unmodified (F-5's distinctness confirmed structurally) |

## Deviations / findings (flagged, per the working agreement)

### The living-plan-node policy was adopted exactly as drafted — no deviation

The slice-doc's "recommended" policy (reconcile-to-current, no special exclusion) is what got built.
Named here only because F-3 explicitly asks the decision be *recorded*, not because anything diverged
from the plan.

### ODD-0013 §3's own text still says "single-target" — not touched, out of this slice's scope

s11 shipped `edges.supersedes` as a `Vec` in code, and ODD-0025 §4 already listed "ODD-0013 §3 (Edges):
change `supersedes` from single-target to a list" as a required amendment — but a check of ODD-0013 §3's
actual current text (not read in full this slice) suggests it may not yet reflect the list-shape
change in its own prose. This is a **discovered-but-unconfirmed** item, not verified against the ODD's
full text, and explicitly out of F-7's scope (which is only about ODD-0025 §4's `artifact/v1.0` line).
Flagged for a future slice or the operator to confirm and, if needed, amend ODD-0013 §3 directly.

### `mapping.rs`/`selfhost.rs` sit between the 90% floor and the 95% target

Both files' *overall* line coverage (92.72% / 94.11%) reflects their whole, largely pre-existing
surface, not specifically the lines this slice added — the new functions
(`reconcile_source`/`reconcile`) are exercised by dedicated fixture files with real branch coverage
(moved/drifted/both/neither, idempotence, dry-run). Not chased further to hit the 95% stretch target
for the whole file, since the shortfall is inherited baseline, not new uncovered surface — consistent
with how this same disclosure was made for `odm-cli/src/commands.rs` in s10/s11.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s12 build the reconcile capability?** Yes, at the fixture-proven, library-code level: both node
families can now re-snapshot a drifted node in place, re-discover a moved source by identity, and
reconcile a still-changing "living" source cleanly and repeatedly — and the project-vision re-cast is
now a reusable function, not an inline test mechanism. **MF-9's mechanism is ready** — the arc's
composition check (does everything actually compose into "odm self-hosts faithfully"?) and the P-12
acceptance demonstration are still ahead, gated on s13's live run producing the reconciled corpus they
demonstrate against.

**What this slice revealed that the arc-plan didn't anticipate:**

1. **The concrete live drift landscape is more varied than "some nodes drifted"** — direct inspection of
   `.worktrees/odm` (not assumed, read) found three distinct shapes needing three distinct fixture
   proofs: sourceless+moved (#13), sourced+moved+drifted (#17, #18), sourceless+not-moved+drifted (#20).
   A generic "drift happens" capability could have missed the moved-and-drifted combination if it hadn't
   been checked against the real corpus first.
2. **Moved-source re-discovery for a *sourceless* node was already solved, for free, by the existing
   identity-matching design** (`legacy::discover`/`discover` both walk the current tree). Only the
   already-*sourced* case needed genuinely new code. Worth naming as a pattern: before building a new
   capability, check whether an existing identity-based matcher already covers part of it structurally.
3. **A possible loose end surfaced but not resolved**: ODD-0013 §3's own prose may not yet reflect the
   `supersedes`→`Vec` change s11 shipped in code (see Deviations). If confirmed, this is a small,
   contained ODD-text fix, not a model or code change — routed to a future slice or the operator.

**The slice-scale silent-drop diff:** scope-as-specified vs. scope-as-delivered — no drops. Every "In"
item (F-1 through F-7) landed; every "Out" item (the live reconcile + vision mint, the MF-9 composition
+ P-12 demo + arc-close, L-8a) is confirmed untouched (F-8).

**Recommended arc-ledger update:**

- **MF-9** ("odm self-hosts *faithfully* — `check`/`orient`/`rollup` green on a corpus with real
  bodies, full coverage, source records") → mechanism **ready**, composition/demo **not yet run** (both
  require s13's reconciled corpus to demonstrate against).
- **s13 (live reconcile + vision mint) is next**: fire `reconcile`/`reconcile_source` against
  `.worktrees/odm` behind the s07/s08/s10 snapshot → dry-run → adjudicate → fire → verify protocol,
  reconciling the s08 active-arc-node, ODD-0013/0017/0018/0020 (moved and/or drifted), and node #25
  (the ODD-0025 §4 amendment's own resulting drift) — then fire the vision-synthesis mint via
  `apply_project_vision`.
- **The arc-close follows s13**: MF-9's composition check + the P-12 self-host acceptance demonstration
  + the arc `closing-report.md` + the bubble-up to `project-plan.md`.
