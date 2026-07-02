# Slice 07 (Arc 05): incremental drift — probes-as-rules + the `.odm/` drift snapshot

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. **The freshness core**
> (ODD-0019); library mechanism — slice08 wires it into every command + renders staleness.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| K-1 | The probe model gains optional **`inputs`** (paths/globs) + a **class**: `file` probe → input-derived (its `path` is the input); `shell` + `inputs` → input-derived; `shell` w/o `inputs` → **volatile**. Additive round-trip (absent ⇒ default) | `cargo test -p odm-core probe_inputs_round_trip` + `probe_class_from_inputs` → ok | serious | ODD-0019 §3.1/§3.5 | open | | Additive to slice01 (`skip_serializing_if`, internally-tagged; proptest `modeled` extended). Volatile-by-default for `shell` w/o inputs = honest staleness (ODD-0019). |
| K-2 | An input's **fingerprint is racy-correct** — size + mtime + a **conditional content-hash** on the racy `mtime >= snapshot-stamp` window (ODD-0014 discipline, **never stat-only**), reusing the file probe's `hex_sha256`; a **same-tick, same-size** input edit is **caught** (would be missed stat-only) | `cargo test -p odm-reconcile input_fingerprint_catches_racy_same_size_edit` → ok | serious | ODD-0014 §3.2 / ODD-0019 §3.2 / 0001-C2 | open | | The A4/slice03 racy test, applied to a probe input. A missed input change = a stale cached outcome = the C2 silent-staleness failure — so racy-correctness is load-bearing, not polish. |
| K-3 | A **persisted `.odm/` drift snapshot** exists — own `MAGIC` (`ODMDRIFT`) + format-version + checksum, **atomic write via `odm_store::atomic::write`**; per fact: last `ProbeOutcome` + (input fingerprint \| last-checked timestamp); round-trips; **corrupt/missing → rebuild** (self-heal) | `cargo test -p odm-reconcile drift_snapshot_round_trip` + `drift_snapshot_corrupt_rebuilds` → ok | serious | ODD-0019 §3.3 / ODD-0014 (snapshot discipline) | open | | Reuses the A4 header discipline (versioned, checksummed, atomic) — a **separate** artifact from the index snapshot (its own `MAGIC`). Never the `IndexRecord`. |
| K-4 | **Incremental reconcile**: an input-derived fact whose inputs are **unchanged** reuses the **cached outcome** (early cutoff, **no re-probe**); **changed** inputs → **re-probe**; the snapshot is updated + persisted atomically | `cargo test -p odm-reconcile incremental_skips_unchanged_reprobes_changed` → ok (a side-effecting/counting probe proves "did not run" on unchanged) | serious | ODD-0019 §3.2 / §4 (verifying traces) | open | | Cost proportional to the change. The "did not run" assertion (not just "same result") is what proves the cutoff — mirrors A4 slice08's warm-skip proof. |
| K-5 | **Volatile facts are not run** on the incremental path — their last `ProbeOutcome` is carried and a **last-checked timestamp** stamped; they refresh only on an explicit full run | `cargo test -p odm-reconcile volatile_not_run_incrementally_stamps_last_checked` → ok | serious | ODD-0019 §3.1/§3.4 | open | | Honest staleness's data half (the *rendering* is slice08). Bare-`odm`-runs-zero-volatile is delivered end-to-end in slice08; here the snapshot carries the stamp. |
| K-6 | **No `odm-index` coupling**: the drift snapshot is a separate `.odm/` artifact; the input fingerprint reuses the ODD-0014 *pattern* + odm-reconcile's `hex_sha256`, **not** odm-index's record-coupled warm-path; **no `IndexRecord`/adapter/`FORMAT_VERSION` change** — A4 invariant untouched | `git diff --stat -- crates/odm-index/` shows **no** change (or only unrelated) AND `! grep -nE "IndexRecord\|index_frontmatters\|FORMAT_VERSION" crates/odm-reconcile/src` | serious | A4 invariant / ODD-0019 §7 | open | | Keeps A5's zero-index-change streak. DRY note (flagged, not acted): two thin racy-fingerprint users (index warm-path + this) — don't extract a shared primitive for two (YAGNI). |
| K-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the new `odm-reconcile`/`odm-core` paths | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-reconcile/src crates/odm-core/src` AND `cargo llvm-cov --summary-only -p odm-reconcile -p odm-core` → **line** ≥ 90% | serious | CLAUDE.md | open | | |

> **Slice boundary (not a row):** `reconcile_views` (slice06) still calls the full
> `run_corpus` this slice — **slice08 flips it to the incremental path** + runs it before
> every command + renders "last checked Xm ago". slice07 is the library mechanism, tested
> directly.

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 7. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
