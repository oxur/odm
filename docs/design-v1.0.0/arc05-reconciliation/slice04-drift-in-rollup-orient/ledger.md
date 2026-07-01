# Slice 04 (Arc 05): drift in `rollup` / `orient`

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Retires the A3 drift
> placeholder (Q-A3-2); on-demand store-read reconcile, no index caching.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| S-1 | `odm_core::rollup::Drift` is fleshed from the empty A3 slot into a **data projection**: counts + per-node **drifted** and **errored** entries, each carrying identity (node number/name, `fact_id`, `describe`) + `expected`/`observed`; plain data, **no `odm-reconcile` dependency in `odm-core`** | `cargo test -p odm-core drift_projection_shape` → ok AND `! grep -nE "odm.reconcile|odm_reconcile" crates/odm-core/Cargo.toml crates/odm-core/src` | serious | 0013 §? / Q-A3-2 / arc-plan | open | | odm-core owns the *shape*; odm-cli fills it. Keep the layering (odm-reconcile → odm-core, never the reverse). Additive-friendly (the A3 `Drift {}` was `#[non_exhaustive]`). |
| S-2 | The reconcile report carries **render-identity** (node number/name + fact `describe`), joined **once** inside `run_corpus` from the loaded frontmatters — resolving slice03's double-load seam; `reconcile`/`rollup`/`orient` render with **no second `load_all`** | `cargo test -p odm-reconcile report_carries_identity` → ok AND `! grep -nE "load_all" crates/odm-cli/src/reconcile.rs` (the reconcile render no longer re-loads) | serious | slice03 bubble-up (double-load) | open | | odm-reconcile already loads frontmatters in `run_corpus` — enrich the report there. Optional: simplify slice03's `reconcile.rs` to use it (flag if done). |
| S-3 | `odm rollup` runs an **on-demand reconcile** (store-read `run_corpus`), projects it into the model's `Drift`, and the drift section renders **real drift** (identity + expected/observed); a **clean** corpus → "no drift" (no fabricated data); the "not yet tracked (A5)" string is **gone** | `cargo test -p odm-cli rollup_drift_reported` + `rollup_clean_no_drift` → ok AND `! grep -rnE "not yet tracked \(A5\)" crates/odm-cli/src crates/odm-core/src` | serious | Q-A3-2 / 0013 §6 / arc-plan A-10 | open | | Replaces the A3 placeholder. Drift comes from a separate store-read reconcile (the model is index-backed but the index lacks `desired_facts`). Shared helper (S-6). |
| S-4 | `odm orient` renders **real drift** in its drift section (same shared helper); clean → no drift; the "not yet tracked (A5)" placeholder is **gone** | `cargo test -p odm-cli orient_drift_reported` + `orient_clean_no_drift` → ok | serious | Q-A3-2 / 0013 §4.1 / arc-plan A-10 | open | | orient's section #5 (vision → focus → ready/blocked → integrity → **drift**). |
| S-5 | Drift is computed **on-demand (store-read `run_corpus`), never cached in the index** — no `desired_facts`/drift added to `IndexRecord`; the A4 adapter-fidelity invariant stays honored by non-triggering | `! grep -rnE "desired_facts|Drift|drift" crates/odm-index/src` (no drift/facts in the index) AND the rollup/orient drift path calls `Runner::run_corpus` (store) | serious | arc-plan v1.5/v1.6 / A4 invariant | open | | The settled resolution: drift is derived + time-varying → reconcile's job, not index state. Keeps the store-vs-index tension closed. |
| S-6 | A **shared `odm-cli` helper** computes drift (run_corpus → project to `odm_core::Drift`) **once**, used by **both** `rollup` and `orient` (no copy-paste of the compute/project/join) | `grep -nE "fn .*drift" crates/odm-cli/src` shows one shared projector called by both `rollup` and `orient` | serious | slice03 bubble-up / D-3 single-source | open | | The two views cannot drift from each other (same helper), and the identity join lives in one place. |
| S-7 | `--json` `rollup/v1` + `orient/v1` carry the **populated** `drift` projection — **additive** (the slot already existed empty; no version bump); clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line) for the touched `odm-cli`/`odm-core`/`odm-reconcile` paths | `cargo test -p odm-cli rollup_json_includes_drift` + `orient_json_includes_drift` → ok AND `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-cli/src crates/odm-core/src crates/odm-reconcile/src` AND `cargo llvm-cov --summary-only -p odm-cli -p odm-core -p odm-reconcile` → **line** ≥ 90% | serious | A3 schema-marker convention / CLAUDE.md | open | | Additive: the A3 `rollup/v1`/`orient/v1` already declared a `drift` slot (empty); populating it is backward-compatible. |

## What Worked

_(At slice close. Patterns that made the slice close cleanly.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 7. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
