# Closing report — Arc 04 / Slice 03: Warm-path change detection

> CC implementation closing report. Status: **proposed-done** (`attested`) → CDC
> reproduces the cargo rows via CI / a local 1.85+ toolchain (`attested →
> reproduced`). Impl commit `e53bc44`; docs commit (this report + ledger +
> arc-plan bubble-up) follows.

## What this slice built

The warm path: `reconcile` loads the prior snapshot, `lstat`-compares each node
file against its cached record, and touches only the delta — the
ODD-0014 §3.2 algorithm, **correct under the racy-git case**. It is the
correctness core of A4: a file racy w.r.t. the index stamp is content-hashed
(the hash is the authority), never trusted on stat alone.

## Per-row ledger walk (10 rows)

- **W-1 — done (attested).** `RebuildNeeded` (missing/corrupt/stale) → full cold
  rebuild via slice02 `build`, reported as `rebuilt` with every node `new`. Not an
  error.
- **W-2 — done (attested).** A non-racy unchanged file is CLEAN: the cached record
  is reused (no `build_one`), byte-identical, and `index_timestamp` is preserved.
- **W-3 — done (attested).** A cheap-signal change (size/mtime/mode) re-parses via
  `build_one` and re-hashes.
- **W-4 — done (attested).** A file with no cached record is inserted (`new`).
- **W-5 — done (attested).** A cached record with no file on the walk is removed
  (`deleted`); the index is the authoritative file list.
- **W-6 — done (attested).** The racy case: a same-size in-place edit with the
  cheap signal reset to match is caught by the content hash (`changed`, not
  `clean`) — a test that fails under stat-only; an unchanged racy file stays clean.
- **W-7 — done (attested).** The same-size-edit defense zeroes the recorded `size`
  of entries still racy w.r.t. the new stamp; durable across reload.
- **W-8 — done (attested).** On change: re-stamp `index_timestamp = now` + persist;
  the reloaded snapshot equals the returned one. No change: `index_timestamp`
  preserved (no rewrite).
- **W-9 — done (attested).** Returns a `Delta` (new/changed/deleted id sets + clean
  count + `rebuilt`); verified over a mixed corpus.
- **W-10 — done (attested).** clippy `-D warnings` → exit 0; no `unsafe`; line
  coverage **98.36%**; `FORMAT_VERSION` unchanged (= 1); `fmt --check` clean; full
  workspace `cargo test` → 257 passed.

## Decisions / deviations flagged (not buried)

1. **W-9 — Delta shape decided: id sets for new/changed/deleted, count for clean.**
   slice05's early cutoff acts on *what moved* (changed → diff `meta_hash`; deleted
   → invalidate; new → compute), so those are id sets; clean is the large
   do-nothing majority whose ids carry no downstream signal, so it is a count.
   Plus `rebuilt: bool`. Recorded on W-9; ratify against slice04/05 needs.
2. **Cache keyed by `rel_path`, not a re-parsed filename-stem id.** ODD-0014 §3.2
   says "id_from_path"; in odm a node's filename *is* its id (`path_of(id)`), so
   `rel_path` identity == record identity and avoids parsing the stem. Equivalent,
   and it matches how the record already links id↔path. Flagged as an
   implementation choice.
3. **`build_one` exposed (not `build_record`).** The cc-prompt said "make
   `build_record` pub(crate)-visible." I instead factored and exposed `build_one`
   (the full read+stat+parse+assemble) because the warm path needs the *whole*
   per-file build for NEW/CHANGED, not just the assemble-from-pieces step. Same
   reuse intent, better seam — `build_records` now also calls it, so there is one
   per-file builder. Flagged as a deliberate divergence from the prompt's wording.
4. **Racy-CHANGED re-reads the file once more.** The racy branch reads bytes to
   hash; on a hash mismatch it calls `build_one`, which reads again — two reads for
   the rare *racy-and-actually-changed* file. The common racy-clean case reads once
   and never parses (the win is preserved). Chose simplicity over threading the
   already-read bytes into `build_one`; flagged.
5. **No record-shape change → `FORMAT_VERSION` stays 1** (W-10 / the slice02
   watch-item). The freeze-or-bump obligation still lands on slice04 (the first
   command that *persists* an index).

## Uncertainties / things CDC should look at

- **Two warm.rs lines uncovered (97.30% line)** — the `WarmError::Stat` / `::Read`
  error-closure bodies (an `lstat`/`read` failing on a file the walk just listed: a
  permissions/race case, hard to provoke portably). Named, not padded around;
  crate line coverage is 98.36%.
- **The "no rewrite" proof is by proxy** (`index_timestamp` preserved on a
  no-change run), not by observing the absence of a write syscall. A re-stamp only
  happens on the persist path, so a preserved stamp implies no persist — a sound
  proxy, but flagged as not a direct filesystem observation.
- **`snapshot.rs:259` (the slice01 `CountMismatch` guard)** remains the one
  uncovered line there (unchanged; unreachable via the public API).

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

- **Did slice03 deliver its assigned piece of the A4 capability?** Yes. The arc
  needs "subsequent runs `lstat`-compare and touch only the delta … racy-git-
  correct." This slice is exactly that warm path, with the racy `>=` content-hash
  fallback as the non-negotiable correctness core and the same-size-edit defense on
  write. It does **not** wire consumers (slice04) or consume `meta_hash` for early
  cutoff (slice05) — its assigned scope.
- **What did it reveal the arc-plan didn't anticipate?**
  - The **Delta shape** is now fixed (decision 1) — **slice05 input:** early cutoff
    reads `delta.changed` (diff each changed record's `meta_hash` vs. the prior to
    decide whether downstream recompute is needed) and `delta.deleted`; `new` always
    recomputes; `clean` (count) is a no-op. A5/reconcile can consume the same Delta.
  - The index now has a **maintained warm path**, so **slice04** can call
    `reconcile` (not just cold `build`) before reading — the consumers see an
    incrementally-updated snapshot. **Arc-plan input for A-10:** the index `list`/
    `orient`/graph-build read is over a reconciled snapshot.
  - **A `node_paths` double-stat micro-cost** (warm stats for the cheap signal;
    `build_one` re-stats on NEW/CHANGED). Negligible, noted for slice06's benchmark
    to confirm it doesn't matter at 100k.
- **Slice-scale silent-drop diff (scope-as-specified vs. scope-as-delivered):**
  none. Every "In" item in `slice-doc.md` shipped (load + rebuild fallback, the four
  classifications, deletion, the racy hash fallback, the same-size-edit defense,
  re-stamp/persist-on-change with no-change-no-rewrite, the returned delta); every
  "Out" item (early-cutoff consumption, consumer wiring, benchmark, watcher) was
  held out. No row softened; 10 opened, 10 dispositioned.

## Iterations

One. In-slice corrections before close: an unused-variable warning (a test binding
dropped) — no logic changes. The racy and defense behaviours landed correct on the
first implementation, verified by deterministic explicit-mtime tests.
