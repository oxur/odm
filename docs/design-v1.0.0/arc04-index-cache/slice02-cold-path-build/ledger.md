# Slice 02 (Arc 04): Cold-path build

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| B-1 | A cold build walks `nodes/` (reusing odm-store's walk, not a re-derived traversal) and produces one `IndexRecord` per node file; a missing/empty `nodes/` yields an empty record set (no error) | `cargo test -p odm-index cold_build_one_record_per_file` + `cold_build_empty_corpus` → ok AND `grep -nE 'odm_store::(layout\|store)\|load' crates/odm-index/src` shows the store walk is reused | serious | arc-plan / 0014 §2.1 | open | | Factor a path-yielding walk in odm-store if needed; don't re-derive `nodes/YYYY/MM`. |
| B-2 | Each record's stat fields (`mtime_secs`, `mtime_nsec`, `size`, `inode`, `mode`) come from `lstat` of the file | `cargo test -p odm-index cold_build_stat_fields` → ok | serious | 0014 §3.1 | open | | mtime_nsec recorded but not a correctness signal (slice03). |
| B-3 | `content_hash` = SHA-256 over the raw file bytes (the input fingerprint) | `cargo test -p odm-index cold_build_content_hash` → ok | serious | 0014 §2.5/§3.1 | open | | Reuses `sha2`; bytes already in hand from the read. |
| B-4 | Extracted metadata is populated from the parsed `Document`: `node_type`, `gates` (reached gate names from `status().reached()`), `tags`, `title` (= `name()`), `updated` | `cargo test -p odm-index cold_build_metadata_fields` → ok | serious | 0014 §3.1 / slice01 | open | | `gates` = reached-gate set (odm has no scalar state — slice01 finding #1). |
| B-5 | Domain `Edges` are mapped to the index's `EdgeRef`/`EdgeKind` across all edge kinds present | `cargo test -p odm-index cold_build_edge_mapping` → ok | serious | slice01 bubble-up #3 | open | | Resolve `EdgeRef` qualifier fidelity (satisfied_at / supersede-kind / because) — see slice-doc design note; record the decision here. |
| B-6 | `meta_hash` = SHA-256 over a canonical, deterministic encoding of the extracted metadata; identical metadata ⇒ identical `meta_hash` across runs | `cargo test -p odm-index meta_hash_deterministic` (repeat/proptest) → ok | serious | 0014 §2.5 | open | | The derived fingerprint slice05's early-cutoff consumes. Canonical encoding (CC's choice) must be order-stable. |
| B-7 | The cold build assembles records + `index_timestamp` (= now at build) + count and persists via slice01's `Snapshot::persist` — not a reimplemented writer | `cargo test -p odm-index cold_build_persists_snapshot` → ok AND `grep -nE 'Snapshot|persist' crates/odm-index/src` shows slice01's persist reused | serious | 0014 §3.2 / reuse | open | | Reuse, not reimplement (slice01 already wraps `odm_store::atomic::write`). |
| B-8 | A built-then-loaded index round-trips: cold-build → persist → load → identical records, and `Load::Loaded` (not `RebuildNeeded`) | `cargo test -p odm-index cold_build_then_load_roundtrip` → ok | serious | integration | open | | End-to-end over a small synthetic corpus. |
| B-9 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for `odm-index` | `cargo clippy -p odm-index --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src` AND `cargo llvm-cov -p odm-index --summary-only --ignore-filename-regex '(odm-core\|odm-store\|odm-graph\|odm-cli)/'` → **line** ≥ 90% | serious | CLAUDE.md | open | | No `unsafe` (rkyv/mmap deferred). |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Rows: 9.)_
