# Slice 03 (Arc 04): Warm-path change detection

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). Reproduced here on a local **1.95.0** toolchain (the 1.85+ floor is met), so the
> cargo rows are CC-`attested` pending the independent CI gate. Five-iteration cap (closed
> in 1).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| W-1 | Warm reconcile loads the existing snapshot; a `RebuildNeeded` (corrupt / version / missing) triggers a full cold rebuild (reuse slice02 `build`), not an error | `cargo test -p odm-index warm_rebuild_on_load_failure` → ok | serious | 0014 §3.2 | done (attested) | `e53bc44`; `warm_rebuild_on_load_failure` → 1 passed (no index → `delta.rebuilt`, 2 new, persisted). `reconcile` matches `Load::RebuildNeeded(_) => rebuild_cold` (reuses `Snapshot::load` + `build`). | |
| W-2 | An unchanged file (size + mtime_secs + mode match, mtime_secs < index_timestamp) is CLEAN — skipped, its record reused, **not** re-read or re-parsed | `cargo test -p odm-index warm_clean_file_skipped_not_reparsed` → ok | serious | 0014 §3.2/§2.1 | done (attested) | `e53bc44`; → 1 passed (clean=1; new/changed/deleted empty; index_timestamp preserved ⇒ no rewrite; reused record byte-identical to prior). | The incremental win: the CLEAN arm pushes the cached record without calling `build_one`. |
| W-3 | A changed file (size ∨ mtime_secs ∨ mode differs) is re-read + re-hashed + re-parsed and its record updated | `cargo test -p odm-index warm_changed_file_updated` → ok | serious | 0014 §3.2 | done (attested) | `e53bc44`; → 1 passed (size-differing edit ⇒ `changed=[id]`; content_hash re-computed). Re-parse via slice02 `build_one` (reuse). | |
| W-4 | A NEW file (no cached record) is read + parsed + inserted | `cargo test -p odm-index warm_new_file_inserted` → ok | serious | 0014 §3.2 | done (attested) | `e53bc44`; → 1 passed (`delta.new=[added]`, 2 records). | |
| W-5 | A DELETED file (cached id absent from the walk) → its record is removed | `cargo test -p odm-index warm_deleted_file_removed` → ok | serious | 0014 §3.2 | done (attested) | `e53bc44`; → 1 passed (`delta.deleted=[gone]`; only the kept node remains). Any cached record unmatched by the walk → removed. | The index is the authoritative file list. |
| W-6 | **The racy case:** a file with `mtime_secs >= index_timestamp` is content-hashed; a same-tick same-size in-place edit (stat unchanged) is caught; identical content stays clean | `cargo test -p odm-index warm_racy_same_size_edit_caught` + `warm_racy_unchanged_stays_clean` → ok | serious | 0014 §2.3/§3.2 | done (attested) | `e53bc44`; both → 2 passed. **The same-size test edits in place to equal-byte-length content + resets mtime so the cheap signal matches** — `changed=[id]`, `clean=0`; it would be CLEAN under a stat-only path. Unchanged-racy → `clean=1`. | **The correctness core.** mtime+size+mode is the cheap signal; the hash is the authority. |
| W-7 | Same-size-edit defense: on write, the recorded `size` of still-racy entries is zeroed so a future same-tick same-size edit forces a cheap mismatch | `cargo test -p odm-index warm_racy_entries_size_zeroed_on_write` → ok | correctness | 0014 §2.3 (git defense) | done (attested) | `e53bc44`; → 1 passed (a future-mtime file ⇒ persisted record `size == 0`, durable across reload). `zero_racy_sizes` zeroes any record with `mtime_secs >= new stamp`. | Adopted (not skipped) — git's belt-and-suspenders, cheap. |
| W-8 | On any change, `index_timestamp = now` and the snapshot is persisted via slice01 `Snapshot::persist`; a no-change run does **not** rewrite | `cargo test -p odm-index warm_restamp_and_persist_on_change` + `warm_no_change_no_rewrite` → ok | serious | 0014 §3.2 / reuse | done (attested) | `e53bc44`; both → 2 passed. On change: `index_timestamp` re-stamped (≠ prior) + the persisted snapshot reloads equal to the returned one. No change: `index_timestamp` preserved (the observable proxy for "no rewrite"). | Persist iff `delta.is_changed()`. |
| W-9 | A warm reconcile returns a delta (new / changed / deleted / clean) for downstream consumers | `cargo test -p odm-index warm_returns_delta` → ok | serious | slice04/05 input | done (attested) | `e53bc44`; → 1 passed (mixed corpus: `new=[N]`, `changed=[C]`, `deleted=[D]`, `clean=1`). | **Delta shape (decided):** **id sets** for new/changed/deleted (slice05's early cutoff acts on exactly these), **count** for clean (the do-nothing majority — ids carry no downstream signal), + `rebuilt: bool`. Recorded here. |
| W-10 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for `odm-index`; no record-shape change (FORMAT_VERSION stays 1) | `cargo clippy -p odm-index --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src` AND `cargo llvm-cov -p odm-index --summary-only …` → **line** ≥ 90% AND FORMAT_VERSION unchanged | serious | CLAUDE.md / slice02 watch-item | done (attested) | `e53bc44`; clippy → exit 0; `unsafe` grep → no matches; `cargo llvm-cov -p odm-index` → **line 98.36%** (warm.rs 97.30%); `FORMAT_VERSION: u16 = 1` unchanged (`snapshot.rs:38`). `fmt --check` clean; full workspace `cargo test` → 257 passed. | No record-shape change needed; freeze/bump obligation still lands on slice04 (first persisting command). |

## What Worked

- **Factoring `build_one` out of `build_records`** gave the warm path the per-file
  re-parse seam for free — NEW/CHANGED call the *same* assembler the cold build
  uses, so a record built warm is byte-identical to one built cold. No second copy
  of stat+hash+parse.
- **Keying the cache by `rel_path`** (a node's filename is its id, so path ==
  identity) made NEW / CHANGED / DELETED fall out of one `HashMap::remove` per
  walked file, with deletions = whatever remains.
- **Setting mtimes explicitly in the tests** turned the racy `>=` cases — normally
  clock-dependent and flaky — into deterministic, reproducible assertions; the
  same-size-edit test reproduces the exact bug stat-only would miss.
- **`is_changed()` driving both persist and re-stamp** made "no change → no
  rewrite" a single branch, and `index_timestamp`-preservation a clean observable
  proxy for it (no sleep-based mtime probing).

## Closure

Closed at commit `e53bc44` on 2026-06-26 (CC implementation). CDC verification:
pending (cargo rows via CI / local 1.85+ — `attested` → `reproduced`). On close, CC
bubbles up to `arc-plan.md` (Arc Ledger A-3) per LEDGER-DISCIPLINE v2.0 §A. Rows: 10.
Done: 10. Deferred: 0. No-op: 0.
