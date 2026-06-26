# Slice 01 (Arc 04): Index record + snapshot persistence

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row must reach
> ≥ `reproduced` at slice scale. CC fills evidence at `attested` per commit; CDC
> reproduces (CI / local 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | The `odm-index` crate exists, is a workspace member, and builds | `cargo build -p odm-index` → exit 0 | serious | arc-plan | open | | New crate; add to `[workspace] members`. |
| F-2 | `IndexRecord` carries identity (`id`, `rel_path`), stat fields (`mtime_secs`, `mtime_nsec`, `size`, `inode`, `mode`), both fingerprints (`content_hash`, `meta_hash`), and extracted metadata (`node_type`, gate/state, `tags`, `edges`, `title`, `updated`) | `cargo test -p odm-index index_record_shape` → ok | serious | 0014 §3.1 | open | | The type only; population from files is slice02. |
| F-3 | The snapshot header carries magic, format-version, hash-algo id, index-timestamp, record count, and a trailing checksum over the body | `cargo test -p odm-index snapshot_header_fields` → ok | serious | 0014 §3.1 | open | | index-timestamp feeds slice03's racy `>=` test. |
| F-4 | A set of records round-trips: `encode ∘ decode = identity` (via postcard) | `cargo test -p odm-index snapshot_roundtrip` (proptest) → ok | serious | 0014 §3.3 | open | | postcard dep added; stable wire format. |
| F-5 | The snapshot is persisted atomically by reusing `odm_store::atomic::write` (temp + fsync + rename + dir-fsync) — not a reimplemented writer | `cargo test -p odm-index snapshot_atomic_persist` → ok AND `grep -n 'odm_store::atomic::write\|atomic::write' crates/odm-index/src` → match | serious | 0014 §3.3 / reuse | open | | Reuse, don't reimplement (the store already does the full sequence). |
| F-6 | A bad checksum or a magic/format-version mismatch is detected on load and returned as a typed "rebuild needed" outcome — never a silent bad parse | `cargo test -p odm-index corrupt_or_version_mismatch_signals_rebuild` → ok | serious | 0014 §3.1/§4 | open | | Self-healing; the index carries no authority. |
| F-7 | Hashing reuses the workspace `sha2`; no new hash dependency added (xxh3 noted as a deferred perf option) | `grep -nE 'sha2' crates/odm-index/Cargo.toml` → match AND `! grep -nE 'xxhash\|twox\|blake3' crates/odm-index/Cargo.toml` | polish | 0014 §3.1 | open | | Crypto hash is fine for a derived cache; fast-hash is a later perf tweak. |
| F-8 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for `odm-index` | `cargo clippy -p odm-index --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src` AND `cargo llvm-cov -p odm-index --summary-only --ignore-filename-regex '(odm-core\|odm-store\|odm-graph\|odm-cli)/'` → **line** ≥ 90% | serious | CLAUDE.md | open | | No `unsafe` (rkyv/mmap deferred, so none needed). |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Rows: 8.)_
