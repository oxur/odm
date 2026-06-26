# CDC Verification — Arc 04 / Slice 03: Warm-path change detection

> Independent verification of CC's closed ledger (impl `e53bc44`; closed `699ebe7`),
> per LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here;
> cargo rows route to CI / local 1.85+. This is the correctness core of A4 — verified
> with extra care on the racy classification.

## Environment constraint (disclosed)

No 1.85+ toolchain in the sandbox; cargo executions route to CI / CC's local 1.95.0 run
(held **attested pending CI**). CDC reproduces structural rows by reading the branch
`arc04-slice03-warm-path-detection` at `e53bc44`.

## Row dispositions

**Row count:** 10 opened, 10 addressed. No silent drops. ✔

**Reproduced by CDC (structural, at `e53bc44`) — the classification read line-by-line
(`warm.rs:129–191`):**

- **W-1** — `reconcile` matches `Load::RebuildNeeded(_)` → `rebuild_cold` (reuses
  `Snapshot::load` + slice02 `build`); not an error. ✔
- **W-2** — the CLEAN arm (cheap signal matches, `mtime_secs < index_timestamp`) pushes
  the cached record without calling `build_one` — no re-read/re-parse. ✔
- **W-3** — `cheap_differs` (`size ∨ mtime_secs ∨ mode`) → `build_one` re-parse, into
  `delta.changed`. ✔
- **W-4 / W-5** — NEW (no cached record) → insert; DELETED = cache remnants after the
  walk (`cache.into_values()`). ✔
- **W-6 (correctness core)** — branch order is right: **racy (`mtime_secs >=
  index_timestamp`) is checked before any clean conclusion**, and there the **content
  hash is the authority** (`sha256(&bytes) == record.content_hash` → clean, else
  CHANGED). Tests `warm_racy_same_size_edit_caught` + `warm_racy_unchanged_stays_clean`
  present; the same-size test edits in place to equal-byte-length content and resets
  mtime so the cheap signal matches — caught only by the hash (would be CLEAN under a
  stat-only path). ✔
- **W-7** — `zero_racy_sizes(&mut next, stamp)` zeroes the recorded `size` of any record
  with `mtime_secs >= stamp`, forcing a future cheap mismatch. ✔
- **W-8** — persist iff `delta.is_changed()`: re-stamp `index_timestamp = now` + persist
  on change; keep `prior.index_timestamp` and persist nothing otherwise. ✔
- **build_one reuse** — factored to `pub(crate)` in `build.rs:97`; called by both the
  cold build and all three warm re-parse sites (NEW/CHANGED/racy-changed). One per-file
  assembler, no second copy. ✔
- **W-10** — no `unsafe` (grep empty); `FORMAT_VERSION` unchanged (`snapshot.rs:38`). ✔

**Attested by CC (local rustc 1.95.0), pending CI:** `cargo test -p odm-index` → 30
passed; full workspace → 257 passed; clippy `-D warnings` → exit 0; `fmt --check` clean;
line coverage **98.36%** (warm.rs 97.30%). → **PENDING CI** (`attested → reproduced`).

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — the warm path (load → classify → racy hash → delta →
  persist-on-change), exactly slice03's scope; early-cutoff *consumption*, consumer
  wiring, and benchmark all correctly held out.
- **Silent-drop diff honest?** ✔ — 10/10; the two uncovered warm.rs I/O-error closures
  (97.30%) named, not buried.
- **Findings dispositioned + arc-plan updated?** ✔ — arc-plan **v1.5** routes the delta
  shape → slice05, reconcile-before-read → slice04/A-10, and the double-stat micro-cost →
  slice06 benchmark. A-3 uses the `open` + strength-in-Evidence convention. No fix needed.

## Rulings on CC's flagged items

1. **Delta shape: id sets (new/changed/deleted) + clean count + `rebuilt`.** **Accepted**
   — exactly the design; slice05 acts on the changed/deleted id sets, clean carries no
   downstream signal. The note that A5/reconcile can reuse `Delta` is good forward sense.
2. **Cache keyed by `rel_path`.** **Accepted** — a node's filename *is* its id, so
   path-identity == record-identity; NEW/CHANGED/DELETED fall out of one `HashMap::remove`
   per walked file.
3. **`build_one` (not the prompt's `build_record`).** **Accepted** — naming only; the
   real per-file builder is factored + reused, which is the reuse intent.
4. **Racy-changed re-reads once more.** **Accepted** — a rare path (read-to-hash, then
   `build_one` re-reads); a minor optional optimization (thread the bytes through), not a
   correctness issue. Not worth complicating the builder API now.
5. **Two I/O-error closures uncovered; "no rewrite" via the `index_timestamp`-preserved
   proxy.** **Accepted** — named; 98.36% clears the floor; the timestamp-preservation
   proxy is a clean observable for no-rewrite (avoids flaky sleep-based mtime probing).

## CDC plan-keeping

**None needed** — bubble-up complete, downstream body lines remain accurate (v1.5
sharpens them, doesn't contradict), convention applied correctly. (Same clean state as
slice02; the discipline is settled.)

## Watch-items carried to slice04

slice04 (wire consumers + filter/sort maps) now inherits **two** accumulated obligations:
(a) the **`FORMAT_VERSION` freeze-or-bump** (it is the first slice to wire a *persisting
command*); and (b) **call `reconcile` before reading** (slice03 finding #2) so consumers
read a freshened, not stale, index.

## Verdict

**Arc 04 / Slice 03 CDC-verified on structure; all flags accepted; cargo rows pending
CI.** The racy-correct warm path is in and faithful to ODD-0014 §3.2 — subsequent runs
cost the delta, and the same-tick same-size edit (the one bug stat-only would miss) is
caught by the content hash. On CI green, A-3 flips `open → done`. slice04 (consumers +
filter/sort) is next, carrying the two watch-items above.

CDC: planning thread, 2026-06-26. Iterations used: 1.
