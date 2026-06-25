# Slice 02 (Arc 03): Rollup generation

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+ (sandbox has no
> 1.85 toolchain). CDC-authored acceptance rows; CC fills Status/Evidence/Notes per
> commit. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| R-1 | An `odm-core` rollup model assembles tree + per-node status + ready/blocked + active-tears + origin from a loaded corpus as a **pure function** (no I/O, no cache) | `cargo test -p odm-core rollup_model_assembles` → ok | serious | 0013 §6 / arc-plan D-2,D-3 | open | | New `rollup.rs`; reuses Recomposition / next / blocked / active_tears / origin — no reimplementation. |
| R-2 | The way-finding tree is the total `part_of` forest (roots + children via reverse `part_of`); every non-root resolves to exactly one parent; no orphans/dupes | `cargo test -p odm-core rollup_tree_total` → ok | serious | 0013 §4.5/§6 | open | | Reuses `Recomposition`. |
| R-3 | Per-node status vectors render in **gate-sequence order** (the type's `odm.toml` sequence), not alphabetical; absent gates shown as not-reached | `cargo test -p odm-core rollup_status_gate_order` → ok | serious | arc-plan D-4 / slice03 CDC note | open | | Order from `GateSet::sequence()`. |
| R-4 | ready (`next`) and blocked sets are computed from the graph + satisfaction and rendered; blocked entries name their unsatisfied edges | `cargo test -p odm-cli rollup_ready_blocked` → ok | serious | 0013 §4.1/§6 | open | | Reuses `NodeGraph::next`/`blocked`. |
| R-5 | Active tears render **with their `because` rationale** | `cargo test -p odm-cli rollup_active_tears_rationale` → ok | serious | 0013 §6 / slice01 | open | | Depends on slice01's `TornEdge`. |
| R-6 | A provenance/origin view labels nodes by `origin` (planned / discovered / amendment) — the original-vs-emergent view | `cargo test -p odm-cli rollup_origin_view` → ok | serious | 0015 A3 / 0001-E2 | open | | Reuses `frontmatter.origin()`. |
| R-7 | The drift section is structurally present but reads "not yet tracked (A5)" — no fabricated data | `cargo test -p odm-cli rollup_drift_placeholder` → ok | correctness | arc-plan Q-A3-2 | open | | Drift computation is A5. |
| R-8 | No deferred-node surfacing: the rendered rollup emits no deferred section and no `deferred` status variant is introduced | `cargo test -p odm-cli rollup_omits_deferred_until_a5` → ok | polish | arc-plan Q-A3-1 | open | | Guard test — deferred + re-entry predicate land with A5. |
| R-9 | `odm rollup` regenerates `ROLLUP.md` from a **full scan**, written atomically via odm-store; idempotent (same corpus → same bytes) | `cargo test -p odm-cli rollup_command_regenerates` → ok | serious | 0013 §6/§7 / arc-plan D-2 | open | | Full-scan regenerate; no cache (A4). |
| R-10 | `ROLLUP.md` carries a "generated — do not edit (`odm rollup`)" header at repo root; `--dry-run` writes nothing | `cargo test -p odm-cli rollup_header_and_dry_run` → ok | correctness | 0013 §6 / §7 | open | | Committed shared view (see slice-doc design note). |
| R-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-core + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Total rows: 11.)_
