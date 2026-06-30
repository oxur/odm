# Slice 03 (Arc 05): `odm reconcile` (on demand)

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Wires the slice02 runner
> to a CLI command; no rollup/orient/schedule (slices 04 / 07).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| R-1 | `odm reconcile` runs the corpus runner and reports drift to stdout; a **clean** corpus (all holds / no facts) prints a plain "no drift" line — **no fabricated data** — and exits `0` | `cargo test -p odm-cli reconcile_clean_reports_no_drift_exit_0` → ok | serious | arc-plan slice03 / 0001-C2 | open | | Report to stdout, not a generated file (that's rollup, slice04). Uses `oxur_cli` output helpers + `table`. |
| R-2 | A **drifted** fact is reported with its identity (node number + name, `fact_id`, `describe`) and `expected` vs `observed`, and the command exits **non-zero** | `cargo test -p odm-cli reconcile_drift_reported_nonzero` → ok | serious | arc-plan slice03 / 0001-C2 | open | | The finding reconcile exists for — must be unmissable + machine-detectable (exit code). |
| R-3 | A **probe `Error`** ("couldn't check") is surfaced **distinctly from drift** (Warning severity), never swallowed; exit stays `0` **without** `--strict` and is **non-zero with** `--strict` | `cargo test -p odm-cli reconcile_probe_error_surfaced_warning` + `reconcile_strict_fails_on_probe_error` → ok | serious | slice01/02 `Error ≠ Drifted` / arc-plan | open | | The `--strict`-gated default (recommended) — ratification point in `slice-doc.md` (Duncan/CC may flip couldn't-check to fail-by-default). Mirrors `check`'s Error/Warning + `--strict`. |
| R-4 | The exit-code + severity mapping is a **pure function of `OutcomeCounts`** (clean / drift / error), consistent with `check`'s conventions | `cargo test -p odm-cli reconcile_exit_severity_is_pure_fn_of_counts` (a table: counts → (severity, exit, exit-under-strict)) → ok | serious | slice02 G-6 / arc-plan | open | | CC flagged in slice02 that this is a pure function — pin it as one (testable in isolation, no I/O). |
| R-5 | `odm reconcile --json` emits a 1:1 projection of the report under a **`reconcile/v1`** schema marker; `ProbeOutcome` + the report types gain `Serialize` (added here, **additive-stable** — explicit, always-serialized fields) | `cargo test -p odm-cli reconcile_json_schema` → ok AND `grep -rnE "reconcile/v1" crates/odm-cli/src` AND `grep -nE "Serialize" crates/odm-reconcile/src` | serious | A3 schema-marker convention / slice02 flag | open | | Consistent with `check/v1`/`rollup/v1`/`orient/v1`. The slice02-deferred `Serialize` lands here under a versioned contract. |
| R-6 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the slice's `odm-cli` reconcile path + any `odm-reconcile` additions | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-cli/src` AND `cargo llvm-cov --summary-only -p odm-cli -p odm-reconcile` → **line** ≥ 90% | serious | CLAUDE.md | open | | |

> **Invariant note (no row needed):** slice03 adds **no new index reader** — the command
> calls `Runner::run_corpus` (store-read). The A4 adapter-fidelity invariant stays honored
> by non-triggering; no `IndexRecord`/adapter/`FORMAT_VERSION` change. (slice-doc "Invariant
> note".)

## What Worked

_(At slice close. Patterns that made the slice close cleanly.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 6. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
