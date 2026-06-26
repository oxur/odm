# CC Prompt — Slice 03 (Arc 04): Warm-path change detection

Make subsequent runs cheap *and correct*: load the snapshot, `lstat`-compare each file,
touch only the delta. This is **the correctness core of A4** — the racy-git case is where
"stat-only" is a real bug, so the `>=` content-hash fallback is non-negotiable.

> **Start condition:** slice02 (cold-path build) CDC-verified / CI-green — `build`,
> `build_record`, `Store::node_paths`, and slice01's `Snapshot` (load/persist/
> `index_timestamp`) exist. If slice02 isn't in, hold.

## Read first
1. `slice03-warm-path-detection/ledger.md` (10 rows).
2. `slice-doc.md` (same dir) and `../arc-plan.md` (Arc Ledger A-3; the A4 capability).
3. **ODD-0014 §3.2** (the warm-path algorithm — implement it), **§2.3** (the racy-git
   lesson + the same-size-edit defense), **§4** (guardrails: stat-only is a correctness
   bug; nanosecond mtime is *not* a correctness signal).
4. The pieces you reuse: `crates/odm-index/src/build.rs` (`build`, `build_record` —
   make it `pub(crate)`-visible to the warm path) and `snapshot.rs` (`Snapshot::load`/
   `persist`/`new`, `index_timestamp`, `Load`).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `02-api-design.md`, `05-type-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence at `attested`; write the
  **Bubble-up to the arc** section at close).

## Task (the ODD-0014 §3.2 algorithm)
1. **Load** the snapshot; `RebuildNeeded` → full cold rebuild via slice02 `build` (not an
   error).
2. **Classify each file** over `Store::node_paths` against its cached record:
   NEW (insert) · CHANGED if `size`∨`mtime_secs`∨`mode` differs (re-read+re-hash+re-parse
   via `build_record`) · **RACILY CLEAN if `mtime_secs >= index_timestamp`** → content-hash;
   differs ⇒ CHANGED, else clean · else CLEAN (skip; reuse the record, no re-parse).
3. **DELETED:** any cached id absent from the walk → remove its record.
4. **Same-size-edit defense:** on write, zero the recorded `size` of still-racy entries
   (git's belt-and-suspenders, §2.3).
5. **Re-stamp + persist:** on any change, `index_timestamp = now`, persist via
   `Snapshot::persist`; **no change → no rewrite**.
6. **Return a delta** (new/changed/deleted/clean) for slice04/05.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Reuse, not reimplement:** `build_record` for re-parsing, `build` for the rebuild
  path, `Snapshot::load`/`persist` for I/O, `node_paths` for the walk. The warm path adds
  *classification*, not new copies.
- **Stat-only is a correctness bug:** never skip the `>=` racy content-hash. Whole-second
  mtime + size + mode is the cheap signal; the content hash is the authority. No
  nanosecond-mtime correctness dependence.
- **No record-shape change** (FORMAT_VERSION stays 1); if one proves necessary, flag it
  as a version bump (the slice02 watch-item).
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test -p odm-index` + clippy + coverage; `ledger.md` evidence per row (at
`attested`); `closing-report.md` — per-row walk **plus the v2.0 Bubble-up to the arc**
(did slice03 deliver its piece; what did it reveal; the silent-drop diff). Feature branch
(`arc04-slice03-warm-path-detection`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration cap;
your `done` is *proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local
1.85+). On close, bubble up to `arc-plan.md` (A-3) per LEDGER-DISCIPLINE v2.0 §A.
