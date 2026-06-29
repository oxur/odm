# Slice 07 (Arc 04): Early-cutoff invalidation

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Modest slice — early
> cutoff for a *batch CLI* is the persisted-artifact skip + the signal (0014 §2.4),
> not lazy/persistent graph caching.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| E-1 | `reconcile`'s `Delta` distinguishes **meta-changed** from **body-only**: a changed record whose new `meta_hash` == the prior record's `meta_hash` is body-only (not meta-changed); the `Delta` exposes the meta-changed id set | `cargo test -p odm-index delta_distinguishes_meta_changed_from_body_only` → ok | serious | 0014 §2.4/§2.5 | open | | `reconcile` already holds prior + new record at the CHANGED arm — just compare `meta_hash`. The signal A5/A7 will also read. |
| E-2 | `odm rollup` skips regenerating `ROLLUP.md` when the corpus is **semantically unchanged** since the last generation (no meta-change, no new, no deleted) — the file is left untouched (byte-identical, mtime preserved) | `cargo test -p odm-cli rollup_skips_on_body_only_change` → ok | serious | arc-plan slice07 / 0014 §2.4 | open | | Recommended: a combined `meta_hash`-fingerprint stamped in the generated header; skip if the current corpus fingerprint matches. |
| E-3 | `odm rollup` **regenerates** `ROLLUP.md` when any `meta_hash` changed, or a node was added/deleted (a meaning-change is never missed) | `cargo test -p odm-cli rollup_regenerates_on_meta_change` → ok | serious | arc-plan slice07 | open | | Covers: gate/evidence change, edge change, origin/decomposed change, new/deleted node. The early cutoff must never skip a real change. |
| E-4 | A body-only edit still **refreshes the index record** (`content_hash` + stat updated via the warm path) — early-cutoff skips the *derived* recompute, not the record refresh | `cargo test -p odm-index body_only_edit_refreshes_record` → ok | serious | 0014 §2.4 | open | | The §2.4 distinction: record updates; downstream does not. |
| E-5 | The in-memory graph readers (`next`/`blocked`/`path`/`check`/`orient`) are **unchanged** — eager recompute per invocation (acceptable for a batch CLI, 0014 §2.4); no behavior change | `cargo test -p odm-cli` (the slice05/06 reader tests stay green) → ok | correctness | 0014 §2.4 | open | | The design boundary: persistent in-memory derived caching is out (deferred unless slice08 shows need). |
| E-6 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-index + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Rows: 6. After this: only slice08 — the benchmark — remains.)_
