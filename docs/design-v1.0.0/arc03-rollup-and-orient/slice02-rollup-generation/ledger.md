# Slice 02 (Arc 03): Rollup generation

> Per LEDGER_DISCIPLINE. Cargo rows reproduced on a local 1.95.0 toolchain (the
> 1.85+ floor is met here); CDC re-runs them via CI / a local 1.85+ toolchain for
> the independent gate. CDC-authored acceptance rows; CC filled Status/Evidence/
> Notes per commit. Five-iteration cap (closed in 1).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| R-1 | An `odm-core` rollup model assembles tree + per-node status + ready/blocked + active-tears + origin from a loaded corpus as a **pure function** (no I/O, no cache) | `cargo test -p odm-core rollup_model_assembles` → ok | serious | 0013 §6 / arc-plan D-2,D-3 | done | `4a50ac2`; `rollup_model_assembles_*` → 2 passed. `Rollup::assemble` (`crates/odm-core/src/rollup.rs`) takes `&[Frontmatter]`+`&GateSets`+`threshold`, builds nothing from disk. | Reuses Recomposition / next / blocked / active_tears / origin — no graph/recompose logic reimplemented. |
| R-2 | The way-finding tree is the total `part_of` forest (roots + children via reverse `part_of`); every non-root resolves to exactly one parent; no orphans/dupes | `cargo test -p odm-core rollup_tree_total` → ok | serious | 0013 §4.5/§6 | done | `4a50ac2`; `rollup_tree_total_single_parent_no_orphans` → 1 passed (asserts tree-placed ids == whole corpus, children id-sorted). | Reuses `Recomposition`. |
| R-3 | Per-node status vectors render in **gate-sequence order** (the type's `odm.toml` sequence), not alphabetical; absent gates shown as not-reached | `cargo test -p odm-core rollup_status_gate_order` → ok | serious | arc-plan D-4 / slice03 CDC note | done | `4a50ac2`; `rollup_status_gate_order_*` → 2 passed (sequence order asserted against a deliberately non-alphabetical reach; documents → empty vector). | Order from `GateSet::sequence()`; absent gate ⇒ `evidence: None`. |
| R-4 | ready (`next`) and blocked sets are computed from the graph + satisfaction and rendered; blocked entries name their unsatisfied edges | `cargo test -p odm-cli rollup_ready_blocked` → ok | serious | 0013 §4.1/§6 | done | `4a50ac2`; `rollup_ready_blocked_named_edges` → 1 passed (asserts `unsatisfied: #3 Early` in the Blocked section). | Reuses `NodeGraph::next`/`blocked`; ready nodes partitioned out of blocked. |
| R-5 | Active tears render **with their `because` rationale** | `cargo test -p odm-cli rollup_active_tears_rationale` → ok | serious | 0013 §6 / slice01 | done | `4a50ac2`; `rollup_active_tears_rationale_rendered` → 1 passed (asserts `because: cut the A-B cycle`). | Tears sourced via `odm_core::graph::frontmatter_tears` (slice01's `TornEdge`). |
| R-6 | A provenance/origin view labels nodes by `origin` (planned / discovered / amendment) — the original-vs-emergent view | `cargo test -p odm-cli rollup_origin_view` → ok | serious | 0015 A3 / 0001-E2 | done | `4a50ac2`; `rollup_origin_view_groups_by_provenance` → 1 passed (planned/discovered/amendment groups each carry the right node). | Reuses `frontmatter.origin()`; covers **every** node (incl. any orphan). |
| R-7 | The drift section is structurally present but reads "not yet tracked (A5)" — no fabricated data | `cargo test -p odm-cli rollup_drift_placeholder` → ok | correctness | arc-plan Q-A3-2 | done | `4a50ac2`; `rollup_drift_placeholder_no_fake_data` → 1 passed (asserts `## Drift` + `Not yet tracked (A5)`). | `Drift` is an empty `#[non_exhaustive]` slot; renderer prints the placeholder unconditionally. |
| R-8 | No deferred-node surfacing: the rendered rollup emits no deferred section and no `deferred` status variant is introduced | `cargo test -p odm-cli rollup_omits_deferred_until_a5` → ok | polish | arc-plan Q-A3-1 | done | `4a50ac2`; `rollup_omits_deferred_until_a5` → 1 passed (asserts the rendered file contains no `deferred` substring, case-insensitive). | `Deferred` slot defined but always empty; renderer emits nothing for it. |
| R-9 | `odm rollup` regenerates `ROLLUP.md` from a **full scan**, written atomically via odm-store; idempotent (same corpus → same bytes) | `cargo test -p odm-cli rollup_command_regenerates` → ok | serious | 0013 §6/§7 / arc-plan D-2 | done | `4a50ac2`; `rollup_command_regenerates_idempotently` → 1 passed (two runs → byte-identical). | Full-scan `load_all`; atomic write via `odm_store::atomic::write`; render carries no timestamp. |
| R-10 | `ROLLUP.md` carries a "generated — do not edit (`odm rollup`)" header at repo root; `--dry-run` writes nothing | `cargo test -p odm-cli rollup_header_and_dry_run` → ok | correctness | 0013 §6 / §7 | done | `4a50ac2`; `rollup_header_and_dry_run` → 1 passed (header asserted in preview + file; `--dry-run` leaves no `ROLLUP.md`). | Committed shared view at repo root (see slice-doc design note). |
| R-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-core + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | done | `4a50ac2`; clippy `--all-targets --all-features --workspace` → exit 0; `unsafe` grep → no matches (exit 1); `cargo llvm-cov` **line**: odm-core **98.69%** (rollup.rs 98.62%), odm-cli **92.82%** (rollup.rs 98.17%). | `fmt --check` also clean; full workspace `cargo test` → 201 passed. |

## What Worked

- **The model-as-packaging discipline (D-3) paid off cleanly.** `Rollup::assemble`
  is pure assembly: every section is an existing odm-core op
  (`Recomposition`, `NodeGraph::next`/`blocked`, `active_tears`, `Satisfaction`,
  `origin()`) mapped into small owned view structs. No graph or recompose logic
  was reimplemented, so the model stays a thin, single source for slice03/slice04.
- **Reading the existing `check`/`next` command paths first** fixed the section
  semantics for free: the rollup loads the corpus, builds the graph, and sources
  tears exactly as `check` does. That surfaced the one shared helper worth
  extracting (`frontmatter_tears`) and let me dedup the CLI's private copy.
- **Thinking through the render branches for coverage caught a real bug:** a
  soft-satisfied node is *ready* (soft deps never withhold it) yet
  `NodeGraph::blocked` still reports the soft dep, so a naive blocked set would
  list it in both sections. Partitioning blocked on the ready frontier is the fix
  — found before it shipped, not after.
- **Deterministic render (no timestamp)** made idempotency a one-line property to
  test and a guarantee by construction, not a hope.

## Closure

Closed at commit `4a50ac2` on 2026-06-25 (CC implementation). CDC verification:
pending (cargo rows via CI / local 1.85+). Total rows: 11. Done: 11. Deferred: 0.
No-op: 0.
