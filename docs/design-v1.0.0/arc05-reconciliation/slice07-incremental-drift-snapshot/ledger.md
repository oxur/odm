# Slice 07 (Arc 05): incremental drift — probes-as-rules + the `.odm/` drift snapshot

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. **The freshness core**
> (ODD-0019); library mechanism — slice08 wires it into every command + renders staleness.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| K-1 | The probe model gains optional **`inputs`** (paths/globs) + a **class**: `file` probe → input-derived (its `path` is the input); `shell` + `inputs` → input-derived; `shell` w/o `inputs` → **volatile**. Additive round-trip (absent ⇒ default) | `cargo test -p odm-core probe_inputs_round_trip` + `probe_class_from_inputs` → ok | serious | ODD-0019 §3.1/§3.5 | done | `cargo test -p odm-core probe_inputs_round_trip` → ok (1 passed): `inputs` on the `shell` variant round-trips; absent ⇒ empty (no `inputs:` on emit). `cargo test -p odm-core probe_class_from_inputs` → ok (1 passed): `ProbeSpec::class()` — `file` → `InputDerived` (path is the input), `shell`+inputs → `InputDerived`, `shell` w/o inputs → `Volatile`; `inputs()` returns the declared paths. **attested** | Additive (`skip_serializing_if = "Vec::is_empty"`; internally-tagged; all `ProbeSpec::Shell` sites updated — the not-`#[non_exhaustive]` safety net). Volatile-by-default = honest staleness (ODD-0019). |
| K-2 | An input's **fingerprint is racy-correct** — size + mtime + a **conditional content-hash** on the racy `mtime >= snapshot-stamp` window (ODD-0014 discipline, **never stat-only**), reusing the file probe's `hex_sha256`; a **same-tick, same-size** input edit is **caught** (would be missed stat-only) | `cargo test -p odm-reconcile input_fingerprint_catches_racy_same_size_edit` → ok | serious | ODD-0014 §3.2 / ODD-0019 §3.2 / 0001-C2 | done | `cargo test -p odm-reconcile input_fingerprint_catches_racy_same_size_edit` → ok (1 passed): capture in the racy window (`captured_at == mtime`), then a **same-size, same-second** edit — stat is byte-identical, but the conditional content-hash (`mtime_secs >= captured_at`) catches it. `input_fingerprint_trusts_stat_outside_racy_window` → ok (cheap signal trusted when not racy; a delete is a change). Reuses the file probe's `hex_sha256`. **attested** | The A4/slice03 racy test, applied to a probe input — never stat-only. Per-fingerprint `captured_at` is the racy reference (decoupled from the snapshot-wide stamp). |
| K-3 | A **persisted `.odm/` drift snapshot** exists — own `MAGIC` (`ODMDRIFT`) + format-version + checksum, **atomic write via `odm_store::atomic::write`**; per fact: last `ProbeOutcome` + (input fingerprint \| last-checked timestamp); round-trips; **corrupt/missing → rebuild** (self-heal) | `cargo test -p odm-reconcile drift_snapshot_round_trip` + `drift_snapshot_corrupt_rebuilds` → ok | serious | ODD-0019 §3.3 / ODD-0014 (snapshot discipline) | done | `cargo test -p odm-reconcile drift_snapshot_round_trip` → ok (1 passed): `DriftSnapshot` encodes `MAGIC ODMDRIFT` + `SNAPSHOT_VERSION` + body + SHA-256 checksum; `encode`/`decode` and `persist`/`load` (via `odm_store::atomic::write`) round-trip; per fact: `ProbeOutcome` + (`InputFingerprint`s \| `last_checked`). `cargo test -p odm-reconcile drift_snapshot_corrupt_rebuilds` → ok (1 passed): missing / garbage / flipped-byte → `Load::RebuildNeeded` (self-heal), never a bad parse. **attested** | Mirrors the A4 header discipline; **separate** artifact (own `ODMDRIFT` magic). **Deviation (flagged):** body is **JSON**, not postcard — `ProbeOutcome` is internally-tagged and `Id` is a string-newtype, both needing a self-describing format (postcard cannot deserialize them). Same header/checksum/atomic discipline. |
| K-4 | **Incremental reconcile**: an input-derived fact whose inputs are **unchanged** reuses the **cached outcome** (early cutoff, **no re-probe**); **changed** inputs → **re-probe**; the snapshot is updated + persisted atomically | `cargo test -p odm-reconcile incremental_skips_unchanged_reprobes_changed` → ok (a side-effecting/counting probe proves "did not run" on unchanged) | serious | ODD-0019 §3.2 / §4 (verifying traces) | done | `cargo test -p odm-reconcile incremental_skips_unchanged_reprobes_changed` → ok (1 passed): a **counting `#!/bin/sh` probe** (appends a line per run) proves it — first run probes (count 1); unchanged input → **count stays 1** (cached, did-not-run); edited input → count 2; unchanged again → stays 2. `full_reprobes_input_derived_including_missing_input` → ok (Full always re-probes; a missing input is captured as absent). Persisted atomically each run. **attested** | Cost proportional to the change. The "count stays" assertion proves the cutoff (not just "same result") — mirrors A4 slice08's warm-skip proof. |
| K-5 | **Volatile facts are not run** on the incremental path — their last `ProbeOutcome` is carried and a **last-checked timestamp** stamped; they refresh only on an explicit full run | `cargo test -p odm-reconcile volatile_not_run_incrementally_stamps_last_checked` → ok | serious | ODD-0019 §3.1/§3.4 | done | `cargo test -p odm-reconcile volatile_not_run_incrementally_stamps_last_checked` → ok (1 passed): a full run checks the volatile fact (count 1) + stamps `last_checked`; an incremental run leaves the counter at 1 (**not run**) and carries the outcome + `last_checked` verbatim. **attested** | Honest staleness's data half (rendering is slice08). Here the snapshot carries the stamp; a never-checked volatile fact gets no entry until an explicit full run. |
| K-6 | **No `odm-index` coupling**: the drift snapshot is a separate `.odm/` artifact; the input fingerprint reuses the ODD-0014 *pattern* + odm-reconcile's `hex_sha256`, **not** odm-index's record-coupled warm-path; **no `IndexRecord`/adapter/`FORMAT_VERSION` change** — A4 invariant untouched | `git diff --stat -- crates/odm-index/` shows **no** change (or only unrelated) AND `! grep -nE "IndexRecord\|index_frontmatters\|FORMAT_VERSION" crates/odm-reconcile/src` | serious | A4 invariant / ODD-0019 §7 | done | `git diff --stat -- crates/odm-index/` → **empty** (no change under `crates/odm-index/`). `grep -rnE "IndexRecord\|index_frontmatters\|FORMAT_VERSION" crates/odm-reconcile/src` → **none** (the drift snapshot's version const is named `SNAPSHOT_VERSION` to keep the guard a real tripwire). The input fingerprint reuses the ODD-0014 *pattern* + odm-reconcile's own `hex_sha256`, not the index warm-path. **attested** | Keeps A5's zero-index-change streak. DRY note (flagged, not acted): two thin racy-fingerprint users (index warm-path + this) — YAGNI to extract a shared primitive for two. |
| K-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the new `odm-reconcile`/`odm-core` paths | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-reconcile/src crates/odm-core/src` AND `cargo llvm-cov --summary-only -p odm-reconcile -p odm-core` → **line** ≥ 90% | serious | CLAUDE.md | done | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0; `grep -RnE '\bunsafe\b' crates/odm-reconcile/src crates/odm-core/src` → none; `cargo llvm-cov --summary-only -p odm-reconcile -p odm-core` → line (new/touched): `snapshot.rs` 93.65%, `incremental.rs` 96.36%, `desired.rs` 100%, `file.rs` 97.01%, `runner.rs` 97.22% (all ≥ 90). Full workspace green (43 suites). **attested** | |

> **Slice boundary (not a row):** `reconcile_views` (slice06) still calls the full
> `run_corpus` this slice — **slice08 flips it to the incremental path** + runs it before
> every command + renders "last checked Xm ago". slice07 is the library mechanism, tested
> directly.

## What Worked

- **Per-fingerprint `captured_at` beat the index's single-stamp model.** The index
  keeps one `index_timestamp` and re-stamps only on change; carrying old
  fingerprints under a new stamp would lose the racy reference. Storing
  `captured_at` on each `InputFingerprint` makes an unchanged carried entry keep
  its own racy context — the racy check is a pure function of the fingerprint,
  which also made K-2 deterministic (capture with `captured_at == mtime`).
- **The counting `#!/bin/sh` script proved the cutoff.** Shell probes exec
  directly (no shell), so a side-effecting `run` string is awkward — a tempfile
  script that appends per run (slice02's signal-test pattern) gives an observable
  run count, turning "did not re-probe" into an assertion, not a hope.
- **Mirror-the-header, own-the-body.** Reusing the A4 snapshot *discipline*
  (MAGIC + version + checksum + `atomic::write`) without importing its
  record-coupled `Snapshot` kept K-6 clean. The one forced divergence — a **JSON**
  body — fell out of `ProbeOutcome` being internally-tagged (postcard can't
  deserialize that); flagged, not worked around.
- **`ProbeSpec::class()`/`inputs()` in odm-core** kept the classification in the
  domain model (one place), so the runner just asks the spec.

## Closure

Closed at commit `4cf2c82` on 2026-07-02. Verified by: CC (self-attested); CDC to
reproduce. Rows: 7. Done: 7. Deferred: 0. No-op: 0.

> Per LEDGER-DISCIPLINE v2.0 §A, all evidence is **attested** (CC built and ran
> every cargo/grep row on a local 1.95 toolchain). Independent **reproduction**
> (CI / local 1.85+) elevates to `reproduced` in `cdc-verification.md`.
> **Branching note:** slices 01–06 had not merged to `main`, so this slice
> branched off `arc05-slice06-deferred-surfacing` (per the prompt's fallback) —
> rebase onto `main` once 01–06 merge.
