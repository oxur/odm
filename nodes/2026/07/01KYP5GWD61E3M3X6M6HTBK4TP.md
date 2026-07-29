---
id: 01KYP5GWD61E3M3X6M6HTBK4TP
number: 557733600
type: artifact
schema: artifact/v1.1
name: 'Slice 04 (Arc 05): drift in `rollup` / `orient`'
created: 2026-07-01
updated: 2026-07-01
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice04-drift-in-rollup-orient/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKZ6VDE3MPF3HS94DB
---
# Slice 04 (Arc 05): drift in `rollup` / `orient`

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Retires the A3 drift
> placeholder (Q-A3-2); on-demand store-read reconcile, no index caching.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| S-1 | `odm_core::rollup::Drift` is fleshed from the empty A3 slot into a **data projection**: counts + per-node **drifted** and **errored** entries, each carrying identity (node number/name, `fact_id`, `describe`) + `expected`/`observed`; plain data, **no `odm-reconcile` dependency in `odm-core`** | `cargo test -p odm-core drift_projection_shape` → ok AND `! grep -nE "odm.reconcile|odm_reconcile" crates/odm-core/Cargo.toml crates/odm-core/src` | serious | 0013 §? / Q-A3-2 / arc-plan | done | `cargo test -p odm-core drift_projection_shape` → ok (1 passed): `Drift { holds, drifted: Vec<DriftedFact>, errored: Vec<ErroredFact> }`, each entry carrying `node_id`/`number`/`name`/`fact_id`/`describe` + (`expected`/`observed` \| `reason`); `is_clean`/`is_empty`. `grep -rnE "odm.reconcile\|odm_reconcile" crates/odm-core/Cargo.toml crates/odm-core/src` → **none** (doc prose reworded to "the `reconcile` crate" so the guard is a true tripwire). **attested** | odm-core owns the *shape*; odm-cli fills it. Kept `#[non_exhaustive]` (additive) with a `Drift::new` constructor; entries are plain structs. |
| S-2 | The reconcile report carries **render-identity** (node number/name + fact `describe`), joined **once** inside `run_corpus` from the loaded frontmatters — resolving slice03's double-load seam; `reconcile`/`rollup`/`orient` render with **no second `load_all`** | `cargo test -p odm-reconcile report_carries_identity` → ok AND `! grep -nE "load_all" crates/odm-cli/src/reconcile.rs` (the reconcile render no longer re-loads) | serious | slice03 bubble-up (double-load) | done | `cargo test -p odm-reconcile report_carries_identity` → ok (1 passed): `NodeReport` gains `number`/`name`, `FactResult` gains `describe`, joined once in `run_node` from the frontmatter it already holds. `grep -nE "load_all" crates/odm-cli/src/reconcile.rs` → **none** (reconcile.rs renders from the enriched report; slice03's second `load_all` removed — the `NodeView`/`HashMap` join deleted). **attested** | Resolved slice03's double-load. reconcile.rs simplified as the optional cleanup suggested (flagged: done). |
| S-3 | `odm rollup` runs an **on-demand reconcile** (store-read `run_corpus`), projects it into the model's `Drift`, and the drift section renders **real drift** (identity + expected/observed); a **clean** corpus → "no drift" (no fabricated data); the "not yet tracked (A5)" string is **gone** | `cargo test -p odm-cli rollup_drift_reported` + `rollup_clean_no_drift` → ok AND `! grep -rnE "not yet tracked \(A5\)" crates/odm-cli/src crates/odm-core/src` | serious | Q-A3-2 / 0013 §6 / arc-plan A-10 | done | `cargo test -p odm-cli rollup_drift_reported` → ok (1 passed): a drifting fact renders `#7 DB layer / db-up — the prod DB answers` + `expected:`/`observed:`, and an erroring fact renders `couldn't check` + `reason:`. `cargo test -p odm-cli rollup_clean_no_drift` → ok (1 passed): clean corpus → "No drift". `grep -rnE "not yet tracked \(A5\)" crates/odm-cli/src crates/odm-core/src` → **none**. **attested** | Replaces the A3 placeholder. `Rollup::assemble(...).with_drift(compute_drift(store)?)` — drift from a separate store-read reconcile; the index (which lacks `desired_facts`) is untouched. |
| S-4 | `odm orient` renders **real drift** in its drift section (same shared helper); clean → no drift; the "not yet tracked (A5)" placeholder is **gone** | `cargo test -p odm-cli orient_drift_reported` + `orient_clean_no_drift` → ok | serious | Q-A3-2 / 0013 §4.1 / arc-plan A-10 | done | `cargo test -p odm-cli orient_drift_reported` → ok (1 passed): orient's DRIFT section shows `✗ #7 DB layer / db-up: expected …, observed …`. `cargo test -p odm-cli orient_clean_no_drift` → ok (1 passed): clean → "no drift"; placeholder gone. Same shared helper as rollup (S-6). **attested** | orient's section #5 (vision → focus → ready/blocked → integrity → **drift**). |
| S-5 | Drift is computed **on-demand (store-read `run_corpus`), never cached in the index** — no `desired_facts`/drift added to `IndexRecord`; the A4 adapter-fidelity invariant stays honored by non-triggering | `! grep -rnE "desired_facts|Drift|drift" crates/odm-index/src` (no drift/facts in the index) AND the rollup/orient drift path calls `Runner::run_corpus` (store) | serious | arc-plan v1.5/v1.6 / A4 invariant | done | `grep -rnE "desired_facts" crates/odm-index/src` → **none** (no facts in the index; the two `drift` matches from `\|Drift\|drift` are pre-existing *unrelated* prose — "config drift", "recomposition drift" — see Notes). The rollup/orient drift path calls `crate::reconcile::compute_drift` → `Runner::run_corpus` (store). No `IndexRecord`/adapter/`FORMAT_VERSION` change. **attested** | The settled resolution: drift is derived + time-varying → reconcile's job, not index state. **Verify-amend flag:** the `Drift\|drift` alternation in the Verify matches pre-A5 prose in odm-index and is not a meaningful tripwire; the load-bearing check is `desired_facts` (clean) + the compute path calling `run_corpus`. |
| S-6 | A **shared `odm-cli` helper** computes drift (run_corpus → project to `odm_core::Drift`) **once**, used by **both** `rollup` and `orient` (no copy-paste of the compute/project/join) | `grep -nE "fn .*drift" crates/odm-cli/src` shows one shared projector called by both `rollup` and `orient` | serious | slice03 bubble-up / D-3 single-source | done | `grep -nE "fn .*drift" crates/odm-cli/src` → one projector `reconcile::compute_drift` (reconcile.rs:69) called by both `rollup.rs` (`.with_drift(crate::reconcile::compute_drift(store)?)`) and `orient.rs` (same); the per-command `render_drift`/DRIFT-section functions only *render* the shared projection. **attested** | The two views cannot drift from each other (same helper), and the compute/project/join lives in one place. |
| S-7 | `--json` `rollup/v1` + `orient/v1` carry the **populated** `drift` projection — **additive** (the slot already existed empty; no version bump); clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line) for the touched `odm-cli`/`odm-core`/`odm-reconcile` paths | `cargo test -p odm-cli rollup_json_includes_drift` + `orient_json_includes_drift` → ok AND `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-cli/src crates/odm-core/src crates/odm-reconcile/src` AND `cargo llvm-cov --summary-only -p odm-cli -p odm-core -p odm-reconcile` → **line** ≥ 90% | serious | A3 schema-marker convention / CLAUDE.md | done | `cargo test -p odm-cli rollup_json_includes_drift` + `orient_json_includes_drift` → ok: `drift` carries `{tracked:true, counts:{holds,drifted,errored}, drifted:[…], errored:[…]}`; `tracked` retained (additive — the shape-lock test updated from `["tracked"]` to `["counts","drifted","errored","tracked"]`, no version bump). `cargo clippy --all-targets --all-features -- -D warnings` → exit 0; `grep -RnE '\bunsafe\b' crates/odm-cli/src crates/odm-core/src crates/odm-reconcile/src` → none; `cargo llvm-cov --summary-only -p odm-cli -p odm-core -p odm-reconcile` → line (touched): reconcile.rs 96.30%, rollup.rs(cli) 98.65%, orient.rs 95.57%, json.rs 94.96%, desired.rs 100%, rollup.rs(core) 98.73%, runner.rs 97.14% (all ≥ 90). Full workspace green (41 suites). **attested** | Additive: `tracked` kept, projection fields added alongside. |

## What Worked

- **The layering fell out cleanly.** `odm-core` owns the `Drift` *shape* (plain
  data, no `reconcile`-crate dep); `odm-cli` runs the probes and projects into it.
  `Rollup::with_drift` (construct-complete builder) kept `assemble` a pure function
  and untouched for every existing caller.
- **Enriching the report resolved slice03's seam *and* fed slice04.** Carrying
  number/name/`describe` on `NodeReport`/`FactResult` (joined once in `run_node`)
  meant `reconcile`, `rollup`, and `orient` all render with **no** second load —
  slice03's flagged double-load is gone, and `compute_drift` is a trivial fold.
- **One shared projector = two views that can't diverge.** `compute_drift` is the
  single compute/project; `rollup` and `orient` only differ in how they *render*
  the same `Drift`.
- **Additive JSON was genuinely backward-compatible.** The A3 `drift:{tracked}`
  slot became `{tracked, counts, drifted, errored}` — `tracked` retained, no
  version bump; the shape-lock test's key set was the one line that changed.
- **The touched-path coverage gate caught a slice02 gap:** `desired.rs` had been
  ~85% (the `file` spec was only measured via `odm-reconcile` before); measuring
  odm-core here surfaced it, fixed with a parse test for `default_true` + null-sha.

## Closure

Closed at commit `bcde1ea` on 2026-07-01. Verified by: CC (self-attested); CDC to
reproduce. Rows: 7. Done: 7. Deferred: 0. No-op: 0.

> Per LEDGER-DISCIPLINE v2.0 §A, all evidence is **attested** (CC built and ran
> every cargo/grep row on a local 1.95 toolchain). Independent **reproduction**
> (CI / local 1.85+) elevates to `reproduced` in `cdc-verification.md`.
> **Branching note:** slices 01–03 had not merged to `main`, so this slice
> branched off `arc05-slice03-reconcile-command` (per the prompt's fallback) —
> rebase onto `main` once 01–03 merge.
