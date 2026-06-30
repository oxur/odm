# CDC Verification — Arc 04 / Slice 08: Benchmark harness (the arc capstone)

> Independent verification of CC's closed ledger (impl + close on
> `arc04-slice08-benchmark`, commit `b95ebd2`), per LEDGER-DISCIPLINE v2.0 (slice scale,
> §A). CDC reproduces structural rows here; the *measured numbers* route to CI / a local
> 1.85+ run (and are machine-specific by nature — the **scaling shape** is the durable
> claim, not the absolute ms).

## Environment constraint (disclosed)

No 1.85+ toolchain in the sandbox (apt cargo 1.75); CC ran the harness on rustc 1.95.0,
Apple Silicon. Two distinct things route differently:

- **Structural rows** (generator exists + seeded + deterministic, harness present, no
  `unsafe`, diff scope, ODD-0014 `[P]`→`[E]` promotion, text-search left `[P]`): reproduced
  by CDC on the branch (below).
- **Measured rows** (P-2…P-6 absolute ms): held **attested pending CI**. The absolutes are
  machine-tagged context; CDC verifies the *internal consistency of the scaling shape* from
  the recorded table and routes the re-run to CI. The durable `[E]` is the shape, so "CDC
  reproduces" here means *reproduces the shape*, not the literal ms.

## Row dispositions

**Row count:** 8 opened, 8 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **P-1** — `odm_index::synth::generate_corpus` exists (`src/synth.rs`, `pub mod synth` in
  `lib.rs`); a **dependency-free** seeded SplitMix64 PRNG → Crockford ULIDs (no `rand`).
  The test (`tests/synth.rs`) asserts determinism *two ways*: same seed → identical ids
  **and** identical `(id, meta_hash)` projection across two corpora (stat fields differ by
  write-time; *meaning* does not — a sharper determinism check than id-equality alone).
  Realism asserted (1 project, `arcs ≥ 1 ∧ slices > arcs` forest, edges, reached gates,
  some `decomposed`, all three origins) **and the real pipeline runs** over it
  (`frontmatters_from_records` → `NodeGraph::build` → `Satisfaction::compute` → `next`).
  `_minimal_does_not_panic` covers n=1/2. ✔ — this measures the real engine, not an empty
  model.
- **P-7** — ODD-0014 promotion verified in `…incremental-indexing-and-caching.md`: the
  index-engine claims are `[E]` (snapshot size/load 24 MB / 128 ms; warm avoids re-parse /
  7.6× cold at 100k; eager recompute acceptable), and crucially the `[E]` sentence **carries
  the `lstat`-sweep-is-O(corpus) nuance inside it** — so the tag does not overclaim. The
  text-search "linear scan fast enough" claim is **left `[P]`** (lines 482, 501–502),
  explicitly flagged not-measured. `benchmark-results.md` records table + environment +
  verdict. ✔
- **P-8 (no `unsafe`)** — `grep -RnE '\bunsafe\b' crates/odm-index/src` → empty. ✔
- **Diff scope** — `git show --stat b95ebd2`: only the generator (`synth.rs`), the harness
  (`benches/index_bench.rs`, `harness = false`), `Cargo.toml`, `lib.rs`, the synth test, the
  docs, and the ODD-0014 promotion. **No engine source touched** — "reuse the engine;
  generator + harness are the new code" is honoured structurally. `FORMAT_VERSION` stays
  `3` (no format change this slice — correct). ✔

**Attested by CC (local rustc 1.95.0), pending CI:**

- **P-2…P-6 (the numbers)** — cold 78.5 / 682 / 17,072 ms; warm-no-change 32 / 329 /
  2,256 ms; warm-delta ≈ no-change; load 1.3 / 12.7 / 128 ms; index 247 KB / 2.5 / 24.9 MB;
  consumer ≈ reconcile. **The scaling shape is internally consistent** (CDC checked the
  table): warm/cold ratio grows 2.4→2.1→**7.6×** (P-3); warm-delta ≈ warm-no-change at every
  scale ⇒ delta marginal cost ≈ 0 (P-4); load linear + sub-second (P-5); consumer ≈ warm
  reconcile ⇒ eager rebuild in the noise (P-6). → **shape accepted; absolutes PENDING CI.**
- **P-8 (cargo)** — `cargo bench` runs (release); clippy `-D warnings` → exit 0
  (`needless_range_loop` fixed); generator `synth.rs` line coverage 100%; workspace green.
  → **PENDING CI.**

## Rulings on CC's flagged items

1. **Honest reframing of P-3/P-4 (the `lstat` sweep is O(corpus); "flat in corpus size"
   holds only for the *delta's marginal cost*, not absolute warm time).** **Accepted — and
   this is the slice earning its keep.** The ledger's optimistic phrasing ("warm scales with
   the delta, not the corpus") was *measured against data and corrected*, not rubber-stamped.
   The win is a **constant** (warm avoids read+parse+hash), which is why it pulls to 7.6× at
   100k — not a change in the sweep's asymptotic class. Recording that nuance *inside* the
   `[E]` claim is exactly the calibration ODD-0014 §4 demanded ("validate before declaring
   victory"). This is measure-don't-assume working as designed.
2. **Text-search "linear scan fast enough" NOT promoted (stays `[P]`).** **Accepted —
   calibrated.** This benchmark measures the index engine, not body search; promoting only
   what was measured is the correct discipline. The claim is flagged for a separate
   text-search benchmark, not silently carried.
3. **Generator lives in the library (`pub mod synth`), not feature-gated.** **Accepted,
   with the flag noted.** The P-1 verify is a plain `cargo test -p odm-index`, which must
   compile without extra flags, so library code is the pragmatic seam; it is documented
   under `#![deny(missing_docs)]`, and `odm-index` is **internal** (the published surface is
   `oxur-odm`), so the compiled-surface cost is negligible. *Tiny optional refinement (not
   required):* if `odm-index` ever needs to slim its non-test build, `synth` could move
   behind a `testutil`/`bench` feature — but that complicates the plain-cargo-test verify,
   and the flag as raised is sufficient for an internal crate.

## One decision CDC examined beyond the flags

**Steady-state warm stamp (measurement-validity choice).** The harness stamps the index in
the far future so unchanged files are non-racy and the warm sweep is `lstat`-only.
**Accepted — the right call, and well disclosed.** This measures the *normal* steady state
(index written after the last edit); conflating the racy same-tick re-hash *fallback* (a
correctness path, slice03's concern) with the warm-win would understate the win in a way
that doesn't reflect normal operation. *Scope note for honesty:* this means the benchmark
does **not** measure worst-case warm (all-racy → re-hash everything ≈ cold). That path's
cost is bounded *above* by the already-measured cold build and is the rare same-tick case,
so the omission is sound — but the `[E]` "warm 7.6× cold" claim is scoped to the steady
state, which `benchmark-results.md` states. No change required; flagged so the claim's
boundary is on the record.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — the seeded generator + the recorded 1k/10k/100k table + the
  ODD-0014 `[P]`→`[E]` promotion; slice07's deferred question settled with data (eager
  rebuild does not dominate; **no caching built**, §2.4 stands).
- **Silent-drop diff honest?** ✔ — 8/8; the O(corpus)-sweep nuance, the steady-state stamp,
  the cold super-linearity, and the not-promoted text-search claim are all disclosed, not
  buried.
- **Findings + arc-plan?** ✔ — `arc-plan.md` v1.14: A-8 open→attested (→ `done` on CI
  reproduce), A-13 satisfied. Standing arc/project finding recorded: if a corpus ever
  outgrows an O(corpus) `lstat` sweep per command, the lever is a **watcher / dir-mtime
  shortcut, not derived-artifact caching** (the consumer-read data shows caching saves
  ~nothing). No CDC plan-keeping fix needed — body lines accurate, status convention
  applied correctly.

## Verdict

**Arc 04 / Slice 08 CDC-verified on structure; the scaling shape is internally consistent;
all flags + the steady-state-stamp decision ruled; the absolute numbers + cargo/coverage
rows are attested pending CI.** The benchmark did its real job: it *corrected* the ledger's
optimistic warm-scaling phrasing against data (the `lstat` sweep is O(corpus); the win is a
constant), promoted only what was measured (`[E]` for the index engine; text-search left
`[P]`), and settled slice07's question — the eager in-memory rebuild does not dominate, so
no caching was built and §2.4 stands.

**All 8 Arc 04 slices are delivered.** The next CDC act is the **arc-close**: the
recomposition / silent-drop check across slices 01–08 (every opened criterion delivered, no
capability dropped between slice boundaries) **+** the class-(b) compose rows (A-9…A-13)
reproduced at *arc* scale (never inherited from the slices), after which **Arc 04 closes**
(the index/cache capability lands) and bubbles up to `project-plan.md`. Standing: the
attested cargo/number rows across slices 01–08 flip `attested`→`reproduced` on CI green.

CDC: planning thread, 2026-06-29. Iterations used: 1.
