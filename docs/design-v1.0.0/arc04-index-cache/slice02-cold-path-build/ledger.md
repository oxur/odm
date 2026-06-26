# Slice 02 (Arc 04): Cold-path build

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). Reproduced here on a local **1.95.0** toolchain (the 1.85+ floor is met), so the
> cargo rows are CC-`attested` pending the independent CI gate. Five-iteration cap (closed
> in 1).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| B-1 | A cold build walks `nodes/` (reusing odm-store's walk, not a re-derived traversal) and produces one `IndexRecord` per node file; a missing/empty `nodes/` yields an empty record set (no error) | `cargo test -p odm-index cold_build_one_record_per_file` + `cold_build_empty_corpus` → ok AND grep shows the store walk is reused | serious | arc-plan / 0014 §2.1 | done (attested) | `d03d5d0`; both tests → 2 passed. Factored `Store::node_paths()` out of `load_all`; `build_records` calls `store.node_paths()` (`build.rs:78`). `load_all` now parses what `node_paths` returns (odm-store suite still green). | The `nodes/YYYY/MM` traversal lives once, in odm-store. |
| B-2 | Each record's stat fields (`mtime_secs`, `mtime_nsec`, `size`, `inode`, `mode`) come from `lstat` of the file | `cargo test -p odm-index cold_build_stat_fields` → ok | serious | 0014 §3.1 | done (attested) | `d03d5d0`; `cold_build_stat_fields_from_lstat` → 1 passed (size/inode/mode compared to `symlink_metadata`; mtime > 0). | `symlink_metadata` (lstat); mtime via portable `modified()`; inode/mode Unix (`0` elsewhere). |
| B-3 | `content_hash` = SHA-256 over the raw file bytes (the input fingerprint) | `cargo test -p odm-index cold_build_content_hash` → ok | serious | 0014 §2.5/§3.1 | done (attested) | `d03d5d0`; `cold_build_content_hash_is_sha256_of_bytes` → 1 passed (matches an independent SHA-256 of the file bytes; differs from `meta_hash`). | Reuses `hash::sha256`; bytes already read for parsing. |
| B-4 | Extracted metadata is populated from the parsed `Document`: `node_type`, `gates` (reached gate names from `status().reached()`), `tags`, `title` (= `name()`), `updated` | `cargo test -p odm-index cold_build_metadata_fields` → ok | serious | 0014 §3.1 / slice01 | done (attested) | `d03d5d0`; `cold_build_metadata_fields_from_document` → 1 passed (node_type/title/updated/tags; `gates == ["built","planned"]`, gate-name sorted). | `gates` = reached-gate set (slice01 finding #1). |
| B-5 | Domain `Edges` are mapped to the index's `EdgeRef`/`EdgeKind` across all edge kinds present | `cargo test -p odm-index cold_build_edge_mapping` → ok | serious | slice01 bubble-up #3 | done (attested) | `d03d5d0`; `cold_build_edge_mapping_all_kinds` (9 edges, all kinds) + `..._supersede_updates_and_qualified_tear` → 2 passed. | **Decision: full qualifier fidelity.** `EdgeRef` enriched with `qualifier: Option<EdgeQualifier>` preserving `depends_on.satisfied_at`, supersede-kind, and tear `because` — so slice04's index-backed graph-build + `orient` need not re-read frontmatter (the arc's "stop re-walking" goal). Extends slice01's `EdgeRef`; format-version stays 1 (no on-disk index exists — the crate is not wired into any command yet). |
| B-6 | `meta_hash` = SHA-256 over a canonical, deterministic encoding of the extracted metadata; identical metadata ⇒ identical `meta_hash` across runs | `cargo test -p odm-index meta_hash_deterministic` → ok | serious | 0014 §2.5 | done (attested) | `d03d5d0`; `meta_hash_deterministic_across_runs` → 1 passed (two builds, equal `meta_hash`). | **Field set:** `node_type, gates, tags, edges, title` (tags+edges sorted; gates already sorted). **Excludes** stat fields, `content_hash`, and **`updated`** (bookkeeping) so a body-only edit leaves `meta_hash` unchanged — the property slice05's early cutoff needs. Encoded via postcard, then SHA-256. |
| B-7 | The cold build assembles records + `index_timestamp` (= now at build) + count and persists via slice01's `Snapshot::persist` — not a reimplemented writer | `cargo test -p odm-index cold_build_persists_snapshot` → ok AND grep shows slice01's persist reused | serious | 0014 §3.2 / reuse | done (attested) | `d03d5d0`; `cold_build_persists_snapshot` → 1 passed (2 records, `index_timestamp > 0`, file written); `build` returns `Snapshot::new(now, records)`; persist via `Snapshot::persist` (`build.rs` references `Snapshot` ×6). | `now_unix_secs()` stamps the build; persist is slice01's (which wraps `odm_store::atomic::write`). |
| B-8 | A built-then-loaded index round-trips: cold-build → persist → load → identical records, and `Load::Loaded` (not `RebuildNeeded`) | `cargo test -p odm-index cold_build_then_load_roundtrip` → ok | serious | integration | done (attested) | `d03d5d0`; `cold_build_then_load_roundtrip` → 1 passed (`Load::Loaded(back)` with `back == built`). | End-to-end over a small synthetic corpus (gates, tags, edges, two nodes). |
| B-9 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for `odm-index` | `cargo clippy -p odm-index --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src` AND `cargo llvm-cov -p odm-index --summary-only …` → **line** ≥ 90% | serious | CLAUDE.md | done (attested) | `d03d5d0`; clippy → exit 0; `unsafe` grep → no matches; `cargo llvm-cov -p odm-index` → **line 98.68%** (build.rs 98.59%). `fmt --check` clean; full workspace `cargo test` → 246 passed. | No `unsafe`. |

## What Worked

- **`Store::node_paths()` was the right factor.** One walk seam now serves both
  `load_all` (parse-all) and the index (stat+hash+parse); the B-1 reuse grep is a
  real invariant, not a comment. odm-store's own suite stayed green — the refactor
  preserved `load_all`'s exact contract.
- **`build_records` (deterministic) split from `build` (stamps `now`)** made every
  field assertion reproducible while keeping the real "stamp at build" behaviour;
  the proptest/round-trip tests never fight the clock.
- **The two-fingerprint split paid off concretely.** `content_hash` over raw bytes
  vs. `meta_hash` over the *semantic* metadata (excluding `updated`/stat) is what
  lets slice05 do early cutoff; choosing the meta_hash field set deliberately here
  (not "hash the whole record") is the load-bearing decision.
- **A transient proptest failure caught a genuine trap** — `skip_serializing_if`
  on a postcard field desyncs the stream. The fix (always serialize the qualifier)
  + the committed regression seed mean it can't silently come back.

## Closure

Closed at commit `d03d5d0` on 2026-06-26 (CC implementation). CDC verification:
pending (cargo rows via CI / local 1.85+ — `attested` → `reproduced`). On close, CC
bubbles up to `arc-plan.md` (Arc Ledger A-2) per LEDGER-DISCIPLINE v2.0 §A. Rows: 9.
Done: 9. Deferred: 0. No-op: 0.
