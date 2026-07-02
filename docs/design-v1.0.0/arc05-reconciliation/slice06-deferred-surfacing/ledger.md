# Slice 06 (Arc 05): deferred surfacing + re-entry predicate (Q-A3-1)

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Cashes Q-A3-1; fills the
> defined-but-empty A3 `Deferred` slot. Read from the reconcile store-overlay (no index change).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| D-1 | A **`deferred` frontmatter marker** exists: `deferred: { because, reenter_when: <fact_id> }` where `reenter_when` references one of the node's own `desired_facts`; it round-trips; **absent ⇒ not deferred** (default) | `cargo test -p odm-core deferred_marker_round_trip` → ok | serious | Q-A3-1 / 0013 §? / arc-plan | done | `cargo test -p odm-core deferred_marker_round_trip` → ok (1 passed): `Deferral { because, reenter_when }` on `Frontmatter` (`with_deferred`/`deferred()`), round-trips; a node with it emits `deferred:` and re-parses equal; **absent ⇒ `None`** and no `deferred:` key on emit. **attested** | Reuses slice01 `desired_facts` (DRY). `#[serde(default, skip_serializing_if = "Option::is_none")]` for the absent default (YAML-additive). Proptest `modeled` list extended with `deferred`. |
| D-2 | The **re-entry predicate is reconcile-evaluated**: `reenter_when` resolves to the node's `desired_fact` outcome — `Holds` → **ready to re-enter**; `Drifted`/`Error` → **still deferred** ("waiting on `<describe>`") | `cargo test -p odm-cli deferred_ready_when_reenter_fact_holds` + `deferred_waiting_when_reenter_fact_drifts` → ok | serious | Q-A3-1 / 0001 E5 / arc-plan | done | `cargo test -p odm-cli deferred_ready_when_reenter_fact_holds` → ok (1 passed): `reenter_when → "back"` (shell `true`, Holds) → rollup renders "ready to re-enter". `cargo test -p odm-cli deferred_waiting_when_reenter_fact_drifts` → ok (1 passed): `reenter_when → "back"` (shell `false`, Drifts) → "waiting on the blocker is cleared". Evaluated by `project_deferred` over `run_corpus` (same store-read path as drift). **attested** | Same probe path drift uses. The re-entry predicate being a probe is the A5 ethos applied to parked work. |
| D-3 | `odm rollup` **fills the A3 `Deferred` slot** (empty until now): deferred nodes with `because` + re-entry status; **clean (none deferred) → no section, no fabricated data** | `cargo test -p odm-cli rollup_surfaces_deferred` + `rollup_no_deferred_section_when_none` → ok | serious | Q-A3-1 / 0013 §6 | done | `cargo test -p odm-cli rollup_surfaces_deferred` → ok (1 passed): a `## Deferred` section lists the node (`#3 Blocked work`) with `because` + re-entry status. `cargo test -p odm-cli rollup_no_deferred_section_when_none` → ok (1 passed): none deferred → **no** `## Deferred` section. `odm_core::rollup::Deferred` reshaped `{ nodes: Vec<DeferredNode { …, because, reentry } > }` + `Rollup::with_deferred`; injected via the shared `reconcile_views`. **attested** | Was always-empty in A3; now filled. Shared projector with orient (D-6). |
| D-4 | `odm orient` surfaces deferred (same shared projector); clean → no deferred section | `cargo test -p odm-cli orient_surfaces_deferred` + `orient_no_deferred_when_none` → ok | serious | Q-A3-1 / 0013 §4.1 | done | `cargo test -p odm-cli orient_surfaces_deferred` → ok (1 passed): orient's `DEFERRED` section shows `#2 Parked … (waiting on …)`. `cargo test -p odm-cli orient_no_deferred_when_none` → ok (1 passed): none deferred → **no** `DEFERRED` section. Same shared `reconcile_views` projector as rollup. **attested** | orient gains a deferred view (section #6) alongside its drift section (#5). |
| D-5 | Deferred (marker + predicate) is read from the **reconcile store-overlay, not the index** — **no** `IndexRecord`/adapter/`FORMAT_VERSION` change; the A4 adapter-fidelity invariant stays honored **by non-triggering** | `git diff --stat -- crates/odm-index/` shows **no** change AND the deferred projector reads from the `compute_drift`/`run_corpus` store path (not `index_frontmatters` records) | serious | A4 invariant (`arc04 closing-report` #2) / ODD-0019 | done | `git diff --stat -- crates/odm-index/` → **empty** (no change under `crates/odm-index/`). The deferred projector reads the marker off `NodeReport` (enriched in `run_node` from the store frontmatter) via `reconcile_views`/`run_corpus` — never `index_frontmatters`. No `IndexRecord`/adapter/`FORMAT_VERSION` change. **attested** | Deferred is reconcile-bound → lives with reconcile, not the fast index model. Marker-indexing rejected (slice-doc). |
| D-6 | `--json` (`rollup/v1` + `orient/v1`) carries the `deferred` slot **additively** (empty/absent in A3 → populated; **no version bump**); a **dangling `reenter_when`** (referencing a non-existent fact) is a **`check` finding** (link-integrity), not a panic | `cargo test -p odm-cli {rollup,orient}_json_includes_deferred` + `cargo test -p odm-core check_flags_dangling_reenter_when` → ok AND `grep -rnE "rollup/v1\|orient/v1" crates/odm-cli/src` unchanged | serious | A3 schema convention / slice05 `check_*` pattern | done | `cargo test -p odm-cli rollup_json_includes_deferred` + `orient_json_includes_deferred` → ok: `deferred:[{node_id,number,name,because,reentry:{status,waiting_on?}}]`; `rollup/v1` slot populated (was empty array), `orient/v1` gains the `deferred` key — both **additive** (shape-lock tests updated; no version bump). `cargo test -p odm-core check_flags_dangling_reenter_when` → ok (1 passed): a `reenter_when` not among the node's `desired_facts` → `Violation::DanglingReenterWhen` (Warning via `violation_severity`), never a panic. `grep -rnE "rollup/v1\|orient/v1" crates/odm-cli/src` → `ROLLUP_SCHEMA`/`ORIENT_SCHEMA` unchanged. **attested** | Additive like slice04's drift slot. Dangling reuses slice05's `check_*` + `violation_severity`. |
| D-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the touched `odm-core`/`odm-cli`/`odm-reconcile` paths | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src crates/odm-reconcile/src` AND `cargo llvm-cov --summary-only -p odm-core -p odm-cli -p odm-reconcile` → **line** ≥ 90% | serious | CLAUDE.md | done | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0; `grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src crates/odm-reconcile/src` → none; `cargo llvm-cov --summary-only -p odm-core -p odm-cli -p odm-reconcile` → line (touched): frontmatter.rs 99.47%, rollup.rs(core) 98.81%, check.rs 99.31%, runner.rs 97.18%, reconcile.rs 95.62%, rollup.rs(cli) 98.73%, orient.rs 95.42%, json.rs 95.54%, commands.rs 91.67% (all ≥ 90). Full workspace green (42 suites). **attested** | |

> **Known limitation (documented, out of scope — slice-doc):** deferred nodes are *surfaced*
> but **not withheld from `next`** (that needs the index-backed graph reader to see the marker
> → indexing it → the A4 invariant + a `FORMAT_VERSION` bump). A deferred, dep-ready node may
> still appear in `next`. Flagged as a scoped follow, not a silent drop.

## What Worked

- **The slice04/05 patterns composed exactly.** Deferred is "drift, again": a
  plain-data `odm-core` projection (`Deferred`/`DeferredNode`/`Reentry`), a
  `Rollup::with_deferred` builder, a store-read projector, and an additive JSON
  slot. The dangling-`reenter_when` finding is "stale-doc, again": a `check_*`
  pass + a `violation_severity → Warning` arm. Nothing new had to be invented.
- **Reusing `desired_facts` for the re-entry predicate (a `fact_id` reference)**
  kept one probe model — the marker is tiny (`{because, reenter_when}`), and
  "can this re-enter?" is answered by the *same* reconcile that answers drift.
- **One `run_corpus`, two views.** Refactoring `compute_drift` → `reconcile_views`
  (project both `Drift` and `Deferred` from a single corpus run) means adding
  deferred did **not** double the probe cost per command — important given the
  slice04 "orient runs probes" finding.
- **The enriched `NodeReport` paid off again.** Carrying the `deferred` marker on
  the report (as slice04 did for identity/`describe`) let the projector read
  everything from one store pass — no second load.

## Closure

Closed at commit `7ce7f0d` on 2026-07-02. Verified by: CC (self-attested); CDC to
reproduce. Rows: 7. Done: 7. Deferred: 0. No-op: 0.

> Per LEDGER-DISCIPLINE v2.0 §A, all evidence is **attested** (CC built and ran
> every cargo/grep row on a local 1.95 toolchain). Independent **reproduction**
> (CI / local 1.85+) elevates to `reproduced` in `cdc-verification.md`.
> **Branching note:** slices 01–05 had not merged to `main`, so this slice
> branched off `arc05-slice05-affects-stale-doc` (per the prompt's fallback) —
> rebase onto `main` once 01–05 merge.
