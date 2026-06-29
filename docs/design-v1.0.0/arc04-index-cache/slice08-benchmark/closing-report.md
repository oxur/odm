# Closing report — Arc 04 / Slice 08: Benchmark harness (the arc capstone)

> Measure the index engine at 1k/10k/100k, record the numbers, promote ODD-0014's
> index-engine `[P]` claims to `[E]`, and settle slice07's deferred question (is the
> eager in-memory rebuild acceptable at 100k?). Branch: `arc04-slice08-benchmark`
> (not `main`). **All 8 Arc 04 slices delivered on close — the arc-close runs next.**

## What shipped

1. **Seeded synthetic-corpus generator (P-1)** — `odm_index::synth::generate_corpus`,
   library code (so the harness *and* the reproducibility test share it). A dependency-free
   SplitMix64 PRNG rendered into valid Crockford ULID strings yields deterministic ids; the
   corpus is a realistic `part_of` forest (project → ~2% arcs → slices) with
   `depends_on`/`blocked_by` edges, reached gates with cycling evidence, all three origins,
   and some affirmed `decomposed`.
2. **The harness (P-2…P-6, P-8)** — `crates/odm-index/benches/index_bench.rs`, a
   `harness = false` release binary (no criterion — we want whole-operation latencies). It
   drives the **unchanged** engine: cold `build`, warm `reconcile` (no-change + 1-file
   delta), `Snapshot::load`, and a consumer read (`reconcile`→adapter→`NodeGraph`→
   `Satisfaction`→`next`). Steady state is modelled by stamping the index after the corpus
   so the warm sweep is `lstat`-only (the racy re-hash path is slice03's correctness
   concern, not the warm-win measurement).
3. **The record + promotion (P-7)** — `benchmark-results.md` (table + environment +
   analysis + verdict); ODD-0014's index-engine `[P]` claims promoted to `[E]`.

## Per-row ledger walk (8 rows)

- **P-1** — `synthetic_corpus_is_seeded_and_realistic` + `_minimal_does_not_panic` → ok.
  Same seed → identical ids; a different seed differs; realism + the real pipeline asserted.
- **P-2** — cold = 78.5 / 682.0 / 17,071.6 ms; ~O(corpus) (per-file cost rises at 100k).
- **P-3** — warm no-change = 32.4 / 328.5 / 2,256.5 ms; **7.6× faster than cold at 100k**
  (the win grows with scale).
- **P-4** — warm small-delta ≈ warm no-change at every scale ⇒ the **delta's marginal cost
  is ≈ 0**; the absolute warm time tracks the O(corpus) sweep (recorded nuance).
- **P-5** — load = 1.3 / 12.7 / 127.8 ms (sub-second at 100k); index 247 KB / 2.49 MB /
  24.9 MB ("tens of MB").
- **P-6** — consumer read ≈ the reconcile at every scale ⇒ the eager rebuild does **not**
  dominate; verdict recorded; no caching built.
- **P-7** — `benchmark-results.md` recorded; 0014 promoted to `[E]` (snapshot size/load;
  warm-avoids-reparse scaling; eager-acceptable), text-search left `[P]`.
- **P-8** — `cargo bench` runs; clippy `-D warnings` clean; no `unsafe`; generator
  `synth.rs` line coverage **100%**; benches excluded from the gate.

## Decisions / deviations flagged (not buried)

- **The generator lives in the library (`pub mod synth`), not behind a feature gate.** The
  ledger's P-1 verify is a plain `cargo test -p odm-index …`, which must compile without
  extra flags — so the generator is normal lib code (fully documented under
  `#![deny(missing_docs)]`). It is benchmark/test support; flagged because it adds a small
  surface to an otherwise lean crate (odm-index is internal; the published surface is
  `oxur-odm`).
- **Steady-state warm stamp (a measurement-validity choice).** The harness stamps the index
  in the far future so unchanged files are non-racy and the warm sweep is `lstat`-only —
  the normal case (index written after the last edit). Measuring the racy same-tick path
  would conflate the correctness fallback (re-hash everything) with the warm win. Flagged
  + documented in the harness and results.
- **Honest reframing of P-3/P-4 (amend, don't fudge).** The data shows the warm path's win
  over cold is ~2× at 1k but **7.6× at 100k** — "dramatic at 100k" holds (P-3), but the
  small-scale ratio is modest, and the `lstat` **sweep is O(corpus)** (no watcher), so
  "flat in corpus size" (P-4) is true only for the *delta's marginal cost*, not the
  absolute warm time. Recorded precisely rather than asserting the optimistic phrasing.
- **Text-search claim NOT promoted.** This benchmark measures the index engine, not body
  search; ODD-0014's "linear scan fast enough" stays `[P]`, flagged for a separate
  text-search benchmark. (Calibrated: promote only what was measured.)

## Uncertainties / things CDC should look at

- **Numbers are CC-`attested`, single-shot, machine-tagged** (Apple Silicon, macOS, rustc
  1.95.0, APFS tempdir). The **scaling** is the durable `[E]`; CDC reproduces the *shape*
  (warm ≪ cold at 100k; delta marginal cost ≈ 0; load sub-second; consumer ≈ reconcile) on
  CI — the absolute ms will differ by machine.
- **Cold's 10k→100k super-linearity (~25×).** Attributed to per-file content I/O + working
  set growth (warm, which skips content, stays ~linear). Worth a confirming look; it does
  not affect the warm-win or load conclusions.
- **100k corpus setup is slow** (100k fsync'd writes dominate wall-clock); the *measured*
  operations are fast. The harness accepts scale args (`cargo bench -p odm-index -- 1000
  10000`) for quick reproduction.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

Applied to `arc-plan.md`:

- **A-8** → `done`-on-attested: slice08 closed (harness + recorded table + `[P]`→`[E]`).
- **A-13 (compose: benchmark promotes `[P]`→`[E]`)** is satisfied — its deliverable is the
  recorded `benchmark-results.md` + the 0014 promotion.
- **Finding for the arc-close (and the project):** the warm path's win is a *constant*
  (avoids re-parse), not a sub-linear sweep — the `lstat` sweep is O(corpus) because odm
  has no watcher (a deliberate §2.4 choice). If a corpus ever grows past where an O(corpus)
  `lstat` sweep per command is acceptable, the lever is a watcher / dir-mtime shortcut, **not**
  in-memory derived caching (which the consumer-read measurement shows would save ~nothing).
  This is the post-arc / A4-follow optimization, scoped but not built.

**All 8 Arc 04 slices are now delivered.** The arc-close is next: recomposition /
silent-drop check across slices 01–08, the class-(b) compose rows (A-9…A-13) reproduced at
arc scale, then Arc 04 closes (the index/cache capability lands) and bubbles up to the
project plan.

## Iterations

One pass. No spec amendment to the criteria was needed; the only in-flight correction was
the precise reading of P-3/P-4 against the data (recorded, not fudged) and a
`needless_range_loop` clippy fix in the generator.
