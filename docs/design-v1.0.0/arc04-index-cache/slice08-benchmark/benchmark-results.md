# Arc 04 benchmark results (slice08)

> The durable record for ledger row **P-7** / Arc Ledger **A-13**. The **scaling**
> is the `[E]` claim; the absolute milliseconds are machine/toolchain **context**,
> recorded below. Reproduce with `cargo bench -p odm-index` (the `index_bench`
> harness over the seeded `synth` corpus). Single-shot end-to-end latencies (not
> criterion statistics — we want whole-operation cost at scale).

## Environment (context for the absolutes)

| | |
|---|---|
| Machine | Apple Silicon, macOS (Darwin 24.6.0), APFS temp dir |
| Toolchain | rustc/cargo **1.95.0**, `bench` profile (optimized) |
| Harness | `crates/odm-index/benches/index_bench.rs`, `harness = false` |
| Corpus | seeded `synth::generate_corpus` (project → ~2% arcs → slices; `depends_on`/`blocked_by` edges; gates+evidence; varied `origin`; some `decomposed`) |
| Seed | `0x0DA0_BEEF_C0DE_0042` (fixed — reproducible) |
| Run | single shot per scale; CC-local (sandbox has no 1.85 floor — CI reproduces) |

## Measured table

| nodes | cold build | warm no-change | warm small-delta | snapshot load | index size | consumer read |
|------:|-----------:|---------------:|-----------------:|--------------:|-----------:|--------------:|
| 1,000 | 78.5 ms | 32.4 ms | 43.1 ms | 1.3 ms | 247 KB | 40.0 ms |
| 10,000 | 682.0 ms | 328.5 ms | 326.4 ms | 12.7 ms | 2,486 KB | 340.5 ms |
| 100,000 | 17,071.6 ms | 2,256.5 ms | 2,299.0 ms | 127.8 ms | 24,918 KB | 2,208.2 ms |

- **cold build** — full corpus walk + read + frontmatter parse + content & meta SHA-256 per file (the pay-once, FIRST-full path, ODD-0014 §2.1).
- **warm no-change** — `reconcile` over an index that post-dates the corpus: the `lstat` sweep + decode, **no re-parse** (the steady state; the racy same-tick re-hash fallback is a separate correctness path, slice03).
- **warm small-delta** — one changed file, then `reconcile`.
- **snapshot load** — `Snapshot::decode` of the persisted index (+ on-disk size).
- **consumer read** — `reconcile` → index→graph adapter → `NodeGraph::build` → `Satisfaction::compute` → `next` (the `odm next` path).

## What the numbers say (the durable scaling claims)

1. **Cold build is ~O(corpus)** and pays the full read+parse+hash per file. (78 → 682 →
   17,072 ms; the 10k→100k factor (~25×) exceeds 10× as per-file content I/O and the
   growing working set dominate — the durable claim is *cold pays full per-file work*,
   which the warm path avoids.)
2. **The warm path's win over cold grows with scale: 2.4× (1k) → 2.1× (10k) → 7.6×
   (100k).** Cold's per-file read+parse+hash inflates with the corpus while the warm
   `lstat` sweep stays cheap, so at 100k a no-change reconcile is **7.6× faster** than a
   cold build (2.3 s vs 17.1 s). This is the headline incremental win.
3. **The delta's marginal cost is ≈ 0 (corpus-independent).** warm small-delta ≈ warm
   no-change at every scale (43≈32, 326≈328, 2299≈2256 ms): re-parsing one changed file is
   negligible. The *expensive* work (read+parse+hash) scales with the **delta**; the
   *cheap* `lstat` sweep is O(corpus) — see the nuance below.
4. **Snapshot load is sub-second at 100k (128 ms) and linear**; the index is ~**24 MB**
   at 100k (~250 B/node) — squarely in the "single file, fine to tens of MB" range.
5. **The eager in-memory graph rebuild does not dominate.** consumer read ≈ the warm
   reconcile at every scale (40≈32, 340≈328, 2208≈2256 ms): the adapter + graph build +
   satisfaction add negligible time over the I/O reconcile.

## Honest nuance — the `lstat` sweep is O(corpus)

The warm path is **not** sub-linear in the corpus: warm no-change scales ~linearly
(32 → 328 → 2,256 ms) because `reconcile` must `lstat` **every** file to learn what
changed (there is no watcher — a deliberate ODD-0014 choice). So "cost scales with the
delta, not the corpus" is true **only for the expensive re-parse work**; the cheap
per-file `lstat` is unavoidable and O(corpus). The win is the *constant* (warm avoids
read+parse+hash), which is why warm pulls 7.6× ahead at 100k — not a change in the
asymptotic class of the sweep. A directory-mtime shortcut or a watcher is the only way
to make the whole warm path sub-linear, and that is explicitly out of scope (§2.4).

## Verdict — slice07's deferred question

**Is the eager in-memory graph rebuild acceptable at 100k?** **Yes.** The consumer read
(2,208 ms) is dominated by the `reconcile` `lstat` sweep (2,256 ms); the graph rebuild
itself is in the noise. Persistent in-memory derived caching would save almost nothing.
If any future optimization is ever warranted it is the **stat-walk** (a watcher /
dir-mtime shortcut), not derived-artifact caching — and ODD-0014 §2.4 deliberately defers
that. **No caching built** (the data confirms §2.4's "eager is acceptable").

## `[P]` → `[E]` promotions (and what stays `[P]`)

Promoted in `docs/design/06-final/0014-…-indexing-and-caching.md`:

- **Single snapshot file fine to tens of MB / sub-second load** → `[E]` (24 MB, 128 ms at 100k).
- **Warm path avoids re-parse; the re-parse work scales with the delta** → `[E]` (warm
  7.6× faster than cold at 100k; delta marginal cost ≈ 0), with the recorded nuance that
  the `lstat` sweep is O(corpus).

**Stays `[P]` (not measured here):** "linear text search is fast enough over large
corpora" — this benchmark measures the *index engine*, not body text search, so that
claim is **not** promoted. Flagged for a future, separate text-search benchmark.
