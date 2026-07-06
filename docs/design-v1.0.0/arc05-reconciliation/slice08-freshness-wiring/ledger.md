# Slice 08 (Arc 05): freshness on every command + honest staleness

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. **Arc capstone** — wires
> slice07's freshness mechanism into every command + settles the two open loose ends; the
> arc-close runs next.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| L-1 | `reconcile_views` runs the **incremental** path (slice07) — orient/rollup get fresh input-derived drift at near-zero cost; **bare `odm`/orient runs ZERO volatile probes** (the slice04 regression dissolved); an input-derived fact whose input changed *is* re-probed | `cargo test -p odm-cli orient_runs_zero_volatile_probes` + `orient_reprobes_changed_input` → ok (a counting volatile probe stays at 0 on bare orient) | serious | ODD-0019 §3.2 / slice04 finding | open | | The headline: freshness for free, regression gone. `reconcile_views` → `reconcile_incremental` (not full `run_corpus`). |
| L-2 | `odm reconcile` (explicit) runs the **full** path — re-probes all input-derived **and volatile** facts, refreshing + re-stamping the snapshot | `cargo test -p odm-cli reconcile_command_runs_full_refreshes_volatile` → ok (volatile counting probe increments; `last_checked` re-stamped) | serious | ODD-0019 §3.4 (explicit refresh) | open | | The sanctioned "refresh volatile now" verb. `odm reconcile` → `reconcile_full`. |
| L-3 | **Honest-staleness rendering**: rollup/orient render volatile facts as **"last checked Xm ago"** (from `last_checked`), input-derived as fresh; a never-checked volatile fact reads "not yet checked — run `odm reconcile`" (no fabricated freshness); `--json` carries staleness **additively** (no schema bump) | `cargo test -p odm-cli {rollup,orient}_render_volatile_staleness` + `orient_never_checked_volatile` + `{rollup,orient}_json_staleness_additive` → ok AND `grep -rnE "rollup/v1\|orient/v1" crates/odm-cli/src` unchanged | serious | ODD-0019 §3.4 | open | | The honest half of the model — say "last checked", never pretend fresh. Additive like slice04/06. |
| L-4 | The drift snapshot lives at a **`.odm/drift` default path** and `.odm/` is **gitignored** (never truth, never committed) | `cargo test -p odm-cli drift_snapshot_default_path` → ok AND `grep -nE "^\.odm/\|/.odm/" .gitignore` | serious | ODD-0019 §3.3 / 0013 (`.odm/` gitignored) | open | | Mirrors the index's `.odm/` home; the snapshot is derived, never committed. |
| L-5 | **`ROLLUP.md` early-cutoff is drift-aware** (settles slice04 finding #2): a **drift change with no corpus change** → `ROLLUP.md` **regenerates**; no drift change + no corpus change → **skipped** (byte-identical). A persisted rollup can no longer hide stale drift | `cargo test -p odm-cli rollup_regenerates_on_drift_change` + `rollup_skips_when_drift_and_corpus_unchanged` → ok | serious | slice04 finding #2 / A4 slice07 cutoff | open | | The cutoff now keys on the drift projection too (incremental drift is cheap — slice07). Closes the "cutoff hides stale drift" gap. |
| L-6 | **Deferred-`next` decision settled — `next` stays graph-pure**: `next` answers graph-readiness (deps+gates) and is **unaffected** by a `deferred` marker; deferred is surfaced only in the reconcile-aware views (rollup/orient). Documented as a deliberate layering boundary (closes the slice06 limitation as **decided-not-withheld**) | `cargo test -p odm-cli next_unaffected_by_deferred_marker` → ok AND the rationale is in the slice06/08 docs + a code comment on the `next` reader | correctness | slice06 limitation / ODD-0019 | open | | Withholding would need indexing the marker (rejected invariant/`FORMAT_VERSION`) or `next` running a reconcile (re-introduces the orient regression). Graph-ready ≠ parked; both surfaces are correct. |
| L-7 | **No `odm-index` change**; clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line) for the touched `odm-cli`/`odm-reconcile` paths | `git diff --stat -- crates/odm-index/` shows **no** change AND `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-cli/src crates/odm-reconcile/src` AND `cargo llvm-cov --summary-only -p odm-cli -p odm-reconcile` → **line** ≥ 90% | serious | A4 invariant / CLAUDE.md | open | | Wiring slice — the freshness home is the drift snapshot, not the index. A5's zero-index-change streak closes the arc intact. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 7. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
On close → **the A5 arc-close** (composition check A-1…A-13; class-(b) rows reproduced at arc scale).
