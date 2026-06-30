# Slice 03 (Arc 05): `odm reconcile` (on demand)

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Wires the slice02 runner
> to a CLI command; no rollup/orient/schedule (slices 04 / 07).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| R-1 | `odm reconcile` runs the corpus runner and reports drift to stdout; a **clean** corpus (all holds / no facts) prints a plain "no drift" line — **no fabricated data** — and exits `0` | `cargo test -p odm-cli reconcile_clean_reports_no_drift_exit_0` → ok | serious | arc-plan slice03 / 0001-C2 | done | `cargo test -p odm-cli reconcile_clean_reports_no_drift_exit_0` → ok (1 passed): a node whose fact holds and a corpus with no declared facts both print a plain `reconcile: no drift …` line and exit `0`. **attested** | Report to stdout, not a generated file (that's rollup, slice04). Renders via plain `writeln!` to the out buffer — matching `check`/`rollup`/`orient`, which use `writeln!`+`tabled`, not `oxur_cli` (odm-cli has no `oxur-cli` dep). Flagged (slice-doc named `oxur_cli`). |
| R-2 | A **drifted** fact is reported with its identity (node number + name, `fact_id`, `describe`) and `expected` vs `observed`, and the command exits **non-zero** | `cargo test -p odm-cli reconcile_drift_reported_nonzero` → ok | serious | arc-plan slice03 / 0001-C2 | done | `cargo test -p odm-cli reconcile_drift_reported_nonzero` → ok (1 passed): a `false`-exit-1 fact against expected exit 0 → output carries `#7`, `"DB layer"`, `db-up`, the `describe`, and `expected:`/`observed:` lines; exit `1`. **attested** | The finding reconcile exists for — must be unmissable + machine-detectable (exit code). Identity (number/name/describe) is joined from the store docs (the slice02 report carries ids only — see bubble-up). |
| R-3 | A **probe `Error`** ("couldn't check") is surfaced **distinctly from drift** (Warning severity), never swallowed; exit stays `0` **without** `--strict` and is **non-zero with** `--strict` | `cargo test -p odm-cli reconcile_probe_error_surfaced_warning` + `reconcile_strict_fails_on_probe_error` → ok | serious | slice01/02 `Error ≠ Drifted` / arc-plan | done | `cargo test -p odm-cli reconcile_probe_error_surfaced_warning` → ok (1 passed): a missing-binary fact → `[error]` line + `reason:` + `couldn't-check` in the header, exit `0`. `cargo test -p odm-cli reconcile_strict_fails_on_probe_error` → ok (1 passed): same corpus with `--strict` → exit `1`. **attested** | Implemented the recommended `--strict`-gated default (not flipped to fail-by-default). Mirrors `check`'s Error/Warning + `--strict`. Error stays distinct from drift in both output (`[error]` vs `[drift]`) and exit semantics. |
| R-4 | The exit-code + severity mapping is a **pure function of `OutcomeCounts`** (clean / drift / error), consistent with `check`'s conventions | `cargo test -p odm-cli reconcile_exit_severity_is_pure_fn_of_counts` (a table: counts → (severity, exit, exit-under-strict)) → ok | serious | slice02 G-6 / arc-plan | done | `cargo test -p odm-cli reconcile_exit_severity_is_pure_fn_of_counts` → ok (1 passed): a table over `OutcomeCounts` asserts `verdict(counts)` → `(severity, exit, strict_exit)` for clean / holds-only / drift / probe-error / both — no I/O. drift→error(1,1); probe-error→warning(0,1); clean→(0,0); drift dominates when both present. **attested** | CC flagged in slice02 that this is a pure function — pinned as `fn verdict(&OutcomeCounts) -> Verdict` with a `#[cfg(test)]` unit test. |
| R-5 | `odm reconcile --json` emits a 1:1 projection of the report under a **`reconcile/v1`** schema marker; `ProbeOutcome` + the report types gain `Serialize` (added here, **additive-stable** — explicit, always-serialized fields) | `cargo test -p odm-cli reconcile_json_schema` → ok AND `grep -rnE "reconcile/v1" crates/odm-cli/src` AND `grep -nE "Serialize" crates/odm-reconcile/src` | serious | A3 schema-marker convention / slice02 flag | done | `cargo test -p odm-cli reconcile_json_schema` (+ `reconcile_json_clean_is_ok`) → ok: `--json` emits `{schema:"reconcile/v1", ok, counts:{holds,drifted,errored}, nodes:[{node_id,number,name,results:[{fact_id,describe,outcome:{kind,…}}]}]}`. `grep -rnE "reconcile/v1" crates/odm-cli/src` → `RECONCILE_SCHEMA`. `grep -nE "Serialize" crates/odm-reconcile/src` → `ProbeOutcome` (tagged `kind`, additive) + `FactResult`/`NodeReport`/`CorpusReport`/`OutcomeCounts`. **attested** | Consistent with `check/v1`/`rollup/v1`/`orient/v1`. The slice02-deferred `Serialize` lands here under a versioned contract; `ProbeOutcome` is internally tagged on `kind` (additive-only). |
| R-6 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the slice's `odm-cli` reconcile path + any `odm-reconcile` additions | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-cli/src` AND `cargo llvm-cov --summary-only -p odm-cli -p odm-reconcile` → **line** ≥ 90% | serious | CLAUDE.md | done | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0; `grep -RnE '\bunsafe\b' crates/odm-cli/src` → none; `cargo llvm-cov --summary-only -p odm-cli -p odm-reconcile` → line: `odm-cli/src/reconcile.rs` **97.10%**; `odm-reconcile` file.rs 97.01% / runner.rs 96.92% / shell.rs 100% (all ≥ 90). Full workspace `cargo test` green (41 suites). **attested** | |

> **Invariant note (no row needed):** slice03 adds **no new index reader** — the command
> calls `Runner::run_corpus` (store-read). The A4 adapter-fidelity invariant stays honored
> by non-triggering; no `IndexRecord`/adapter/`FORMAT_VERSION` change. (slice-doc "Invariant
> note".)

## What Worked

- **`check` was a ready-made template.** Mirroring its exit model (errors fail;
  warnings fail only under `--strict`; `return commands::check(...)` from
  dispatch) made the severity/exit semantics a known quantity — reconcile is
  `check` with drift=error, probe-error=warning. Reusing the mental model is the
  whole point of the slice-doc's ratified decision.
- **Pure `verdict(&OutcomeCounts)` factored the policy out of the I/O.** R-4's
  "pure function" pinned as a `#[cfg(test)]` table test; the command is a thin
  shell around it. The counts→(severity, exit, strict_exit) table is the spec.
- **Internally-tagged `Serialize` on `ProbeOutcome`** gave the `{kind, …}` JSON
  shape for free and matches the additive-evolution discipline already used for
  `ProbeSpec` — one consistent serde idiom across the model.
- **One enriched `NodeView` feeds both renderings.** Human and `--json` derive
  from the same joined view (the D-3 ethos: the two outputs cannot drift).

## Closure

Closed at commit `266fc38` on 2026-06-30. Verified by: CC (self-attested); CDC to
reproduce. Rows: 6. Done: 6. Deferred: 0. No-op: 0.

> Per LEDGER-DISCIPLINE v2.0 §A, all evidence is **attested** (CC built and ran
> every cargo/grep row on a local 1.95 toolchain). Independent **reproduction**
> (CI / local 1.85+) elevates to `reproduced` in `cdc-verification.md`.
> **Branching note:** slices 01–02 had not merged to `main`, so this slice
> branched off `arc05-slice02-file-probe-and-runner` (per the prompt's fallback)
> — rebase onto `main` once 01/02 merge.
