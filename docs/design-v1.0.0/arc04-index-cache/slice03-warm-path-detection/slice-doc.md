# Slice 03 (Arc 04) — Warm-path change detection (plan-of-record)

> Refs: ODD-0014 §3.2 (the warm-path algorithm — *the* spec for this slice), §2.3 (the
> racy-git correctness lesson), §0 (the convergent stat-cache finding), §4 (guardrails:
> stat-only is a correctness bug; nanosecond mtime is not a correctness signal); arc04
> `arc-plan.md` slice03 + Arc Ledger A-3. `depends_on:` slice01 (`Snapshot` load/persist
> + `index_timestamp`), slice02 (`build_record` per-file builder + `build` for the
> rebuild path; `Store::node_paths`).
>
> **Why this slice exists:** slice02 builds the whole index every run (O(corpus)). This
> slice makes the *subsequent* runs cheap: load the snapshot, `lstat`-compare each file,
> and touch only the delta — re-reading/re-parsing just what changed. It is **the
> correctness core of A4**: the one place where "stat-only" is a real bug (the racy-git
> case), so the racy `>=` content-hash fallback is non-negotiable, not a perf tradeoff.

## Goal

A warm reconcile that updates an existing snapshot from a corpus walk at delta cost,
**correctly** under the racy-git case. **Done when** an unchanged corpus is a no-op
(stat-only, no re-parse, no rewrite); new/changed/deleted files are detected and the
snapshot updated; a same-tick, same-size in-place edit (the racy case) is caught via the
content-hash fallback (a test that would fail under a stat-only path); and the reconcile
returns a delta the downstream slices can consume.

## Scope

**In** (the ODD-0014 §3.2 algorithm):

- **Load + rebuild fallback.** `Snapshot::load`; a `RebuildNeeded` (corrupt / version /
  missing) → a **full cold rebuild** (reuse slice02's `build`), not an error.
- **Per-file classification** against the cached record, over `Store::node_paths`:
  - **NEW** (no cached record) → read + hash + parse + insert (reuse slice02's
    `build_record`);
  - **CHANGED** (cheap signal: `size` ∨ `mtime_secs` ∨ `mode` differs) → re-read +
    re-hash + re-parse + update;
  - **RACILY CLEAN** (`mtime_secs >= index_timestamp`) → **content-hash the file**; if it
    differs from the cached `content_hash` → CHANGED, else clean. *The correctness core.*
  - **CLEAN** (cheap signal matches, `mtime_secs < index_timestamp`) → skip; record
    reused, **not** re-read or re-parsed.
- **DELETED detection:** any cached id absent from the walk → record removed.
- **Same-size-edit defense** (ODD-0014 §2.3, git's belt-and-suspenders): on write, zero
  the recorded `size` of still-racy entries so a future same-tick same-size edit forces a
  cheap mismatch.
- **Re-stamp + persist:** on any change, set `index_timestamp = now` (just before write)
  and persist via slice01's `Snapshot::persist`; **no change → no rewrite**.
- **Return a delta** (new / changed / deleted / clean) — the signal slice04 (consumers)
  and slice05 (early cutoff) build on.

**Out:** early-cutoff *downstream* invalidation (diffing `meta_hash` to skip graph/rollup
recompute) — slice05; pointing `list`/`orient`/graph-build at the index — slice04; the
benchmark — slice06; the optional filesystem watcher — deferred (arc-plan open Qs).

## Design notes (settle here)

- **Reuse the per-file builder.** Re-parsing a changed/new file must call slice02's
  `build_record` (make it `pub(crate)`-visible to the warm path) — do **not** duplicate
  the stat + hash + parse logic. Same discipline as slice02's `node_paths` reuse.
- **Whole-second mtime + size + mode is the cheap signal; the content hash is the
  authority** (ODD-0014 §2.3/§4). **Do not** rely on nanosecond mtime for correctness
  (Linux default-off; exotic/network FS unreliable) — at most an opportunistic extra
  dirty hint.
- **No record-shape change** → `FORMAT_VERSION` stays 1 (the freeze-or-bump obligation
  still lands on slice04, the first persisting *command*). If slice03 finds it must
  change the record, that is a version bump — flag it.
- **Delta shape (open):** what the reconcile returns (counts vs. the changed/deleted id
  sets). Decide against what slice04/05 need; record it on the delta row.

## Verification

`cargo test -p odm-index` green (clean no-op, new/changed/deleted, **the racy same-tick
same-size case**, same-size-edit defense, re-stamp+persist, no-change-no-rewrite, delta);
clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line) for `odm-index`. Rows in
`ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+ — `attested` →
`reproduced`). Subsequent runs cost the delta, not the corpus, and are racy-correct.
slice04 can point the consumers at the (now incrementally-maintained) index. Bubble up to
`arc-plan.md` (A-3) at close.
