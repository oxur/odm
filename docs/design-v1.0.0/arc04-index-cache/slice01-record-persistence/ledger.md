# Slice 01 (Arc 04): Index record + snapshot persistence

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row must reach
> ≥ `reproduced` at slice scale. CC fills evidence at `attested` per commit; CDC
> reproduces (CI / local 1.85+). Reproduced here on a local **1.95.0** toolchain
> (the 1.85+ floor is met), so the cargo rows are CC-`attested` pending the
> independent CI gate. Five-iteration cap (closed in 1).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | The `odm-index` crate exists, is a workspace member, and builds | `cargo build -p odm-index` → exit 0 | serious | arc-plan | done (attested) | `69952ce`; `cargo build -p odm-index` → Finished, exit 0. Added to `[workspace] members`; `postcard` added to `[workspace.dependencies]`. | New crate. |
| F-2 | `IndexRecord` carries identity (`id`, `rel_path`), stat fields (`mtime_secs`, `mtime_nsec`, `size`, `inode`, `mode`), both fingerprints (`content_hash`, `meta_hash`), and extracted metadata (`node_type`, gate/state, `tags`, `edges`, `title`, `updated`) | `cargo test -p odm-index index_record_shape` → ok | serious | 0014 §3.1 | done (attested) | `69952ce`; `index_record_shape_carries_all_fields` → 1 passed (asserts every field group). Type in `record.rs`. | gate/state realised as `gates: Vec<String>` (reached-gate set) — odm has no single `state`; flagged in closing report. Population is slice02. |
| F-3 | The snapshot header carries magic, format-version, hash-algo id, index-timestamp, record count, and a trailing checksum over the body | `cargo test -p odm-index snapshot_header_fields` → ok | serious | 0014 §3.1 | done (attested) | `69952ce`; `snapshot_header_fields_present` → 1 passed (header() fields; on-disk MAGIC/version/algo at fixed offsets; trailing checksum present; timestamp+count round-trip). | index-timestamp feeds slice03's racy `>=` test. |
| F-4 | A set of records round-trips: `encode ∘ decode = identity` (via postcard) | `cargo test -p odm-index snapshot_roundtrip` (proptest) → ok | serious | 0014 §3.3 | done (attested) | `69952ce`; `snapshot_roundtrip_encode_decode_identity` (proptest, 0–20 arbitrary records) → 1 passed. postcard dep added; fixed-prefix + body + checksum layout. | Stable wire format. |
| F-5 | The snapshot is persisted atomically by reusing `odm_store::atomic::write` (temp + fsync + rename + dir-fsync) — not a reimplemented writer | `cargo test -p odm-index snapshot_atomic_persist` → ok AND `grep -n 'odm_store::atomic::write\|atomic::write' crates/odm-index/src` → match | serious | 0014 §3.3 / reuse | done (attested) | `69952ce`; `snapshot_atomic_persist_and_reload` → 1 passed (persist creates dirs, file exists, reloads identically); grep → `snapshot.rs:278 odm_store::atomic::write(path, &bytes)?`. | Reuse, not reimplement. |
| F-6 | A bad checksum or a magic/format-version mismatch is detected on load and returned as a typed "rebuild needed" outcome — never a silent bad parse | `cargo test -p odm-index corrupt_or_version_mismatch_signals_rebuild` → ok | serious | 0014 §3.1/§4 | done (attested) | `69952ce`; `corrupt_or_version_mismatch_signals_rebuild` (decode-level: BadChecksum/TooShort/BadMagic/VersionMismatch/UnknownHashAlgo/Missing) + `..._through_load` (on-disk corrupt file → `Load::RebuildNeeded(BadChecksum)`) → 2 passed. | `RebuildReason` typed enum; `Load::RebuildNeeded`. Self-healing; index carries no authority. |
| F-7 | Hashing reuses the workspace `sha2`; no new hash dependency added (xxh3 noted as a deferred perf option) | `grep -nE 'sha2' crates/odm-index/Cargo.toml` → match AND `! grep -nE 'xxhash\|twox\|blake3' crates/odm-index/Cargo.toml` | polish | 0014 §3.1 | done (attested) | `69952ce`; `sha2.workspace = true` present; xxhash/twox/blake3 grep → no matches (exit 1). `HashAlgo::Sha256`; xxh3 documented as deferred in `snapshot.rs`. | Crypto hash fine for a derived cache. |
| F-8 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for `odm-index` | `cargo clippy -p odm-index --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src` AND `cargo llvm-cov -p odm-index --summary-only --ignore-filename-regex …` → **line** ≥ 90% | serious | CLAUDE.md | done (attested) | `69952ce`; clippy `-p odm-index --all-targets -- -D warnings` → exit 0; `unsafe` grep → no matches (exit 1); `cargo llvm-cov -p odm-index` → **line 98.78%**. `fmt --check` clean; full workspace `cargo test` → 234 passed. | No `unsafe` (rkyv/mmap deferred). |

## What Worked

- **Reusing `odm_store::atomic::write` was a clean seam.** The index needed
  exactly the store's crash-safe write sequence; `persist` is three lines over it,
  and the F-5 grep proves there is no second implementation to drift.
- **A fixed-offset binary header + a single postcard body + a trailing checksum**
  gave clean, *typed* detection of every corruption mode (F-6) without any
  manual framing of the records: magic/version/algo are checked at known offsets
  before any deserialize, and a bad body is a postcard error mapped to
  `RebuildReason::Decode`. The same shape git's index uses.
- **`Load::RebuildNeeded` as a first-class non-error outcome** (vs. an error
  variant) models the self-healing contract directly — slice02/03 will `match`
  and rebuild, and a missing file is just `Missing`, not an error to special-case.
- **The `u128 → Ulid → Id::from_str` proptest strategy** gave deterministic,
  shrinkable arbitrary ids despite `Id` exposing no numeric constructor — keeping
  the round-trip property reproducible.
- **Owning the index's `EdgeKind`** (mirroring, not reusing, `odm_core`'s) keeps
  the wire format under the snapshot format-version's control — the right
  coupling for a versioned cache.

## Closure

Closed at commit `69952ce` on 2026-06-26 (CC implementation). CDC verification:
pending (cargo rows via CI / local 1.85+ — `attested` → `reproduced`). On close,
CC bubbles up to `arc-plan.md` (Arc Ledger row A-1) per LEDGER-DISCIPLINE v2.0 §A.
Rows: 8. Done: 8. Deferred: 0. No-op: 0.
