# Slice 06 (Arc 04): Enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient`

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. The slice05
> continuation (G-3/G-4/G-5); finishes the arc's consumer-wiring.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| V-1 | `IndexRecord` carries `origin` + `decomposed`; `FORMAT_VERSION 2 → 3`; an on-disk v2 index loads as `RebuildNeeded(VersionMismatch)` → cold rebuild (slice01 self-heal; no migration code) | `cargo test -p odm-index record_carries_origin_decomposed` + `v2_index_triggers_rebuild` → ok AND `grep -n 'FORMAT_VERSION: u16 = 3'` | serious | arc-plan v1.10 / slice05 finding | open | | The complete remaining gap (grep-verified: only these two). |
| V-2 | `build_one` populates `origin` + `decomposed` (cold + warm); `meta_hash` covers `decomposed` (recomposition) and `origin` (provenance) and still excludes `updated`/stat/display fields | `cargo test -p odm-index build_one_origin_decomposed` + `meta_hash_tracks_decomposed_and_origin` → ok | serious | slice02 B-4/B-6 | open | | Both are semantic (graph recomposition / provenance view) ⇒ correctly in meta_hash. |
| V-3 | The adapter (`frontmatters_from_records`) reconstructs `origin` + `decomposed`, so a synthesized `Frontmatter` is faithful for `recompose::integrity` + `Rollup::assemble`'s provenance | `cargo test -p odm-index adapter_reconstructs_origin_decomposed` → ok | serious | slice05 G-1 / 0014 §2.4 | open | | Extends the slice05 fidelity test (graph == baseline) to recomposition + provenance. |
| V-4 | `check`'s `aggregate` refactored to take `&[Frontmatter]`; `check` reads the index-backed graph + recomposition (`decomposed`); output identical to baseline (incl. orphan / undeveloped-stub / decomposition-drift findings + severities) | `cargo test -p odm-cli check_index_backed_matches_baseline` → ok | serious | 0013 §4.5 / arc-plan A-12 | open | | The recomposition path is what needed `decomposed`. |
| V-5 | `rollup` composes over the index-backed model; the provenance view (`origin`) matches; output identical to baseline | `cargo test -p odm-cli rollup_index_backed_matches_baseline` → ok | serious | 0013 §6 / arc-plan A-12 | open | | Provenance is what needed `origin`. |
| V-6 | `orient` composes over the index-backed model (rollup + check integrity) **and** loads only the current project's `Document` for the vision body; output identical to baseline | `cargo test -p odm-cli orient_index_backed_matches_baseline` → ok | serious | 0013 §4.1 / 0014 §3.5 | open | | One targeted `store.load(project)` — bodies stay out of the index. |
| V-7 | All three (`check`/`rollup`/`orient`) `reconcile` (warm path) before reading | `cargo test -p odm-cli view_consumers_reconcile_before_read` → ok | serious | slice03 finding #2 | open | | Same reconcile-then-read wrapper as `list`/derived-order. |
| V-8 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-index + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Rows: 8. On close: A-4 + A-5 close; consumer-wiring done.)_
