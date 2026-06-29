# Slice 08 (Arc 04): Benchmark harness

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. The arc capstone:
> closes A-13, last slice before the arc-close.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| P-1 | A **seeded** synthetic-corpus generator produces reproducible corpora at 1k / 10k / 100k nodes with realistic frontmatter (`part_of` forest, edges, gates+evidence, varied `origin`, some `decomposed`) | `cargo test -p odm-index synthetic_corpus_is_seeded_and_realistic` → ok | serious | 0014 §4 | open | | Same seed → identical corpus; exercises the real build/adapter/graph paths, not empty nodes. |
| P-2 | **Cold build** latency is measured at 1k/10k/100k and is ~`O(corpus)` (≈ linear in node count) | harness run → recorded table; ratio 1k→10k→100k ≈ linear | serious | 0014 §2.1 | open | | The pay-once baseline. |
| P-3 | **Warm no-change** reconcile at 100k is **dramatically faster than cold build** (lstat sweep, no re-parse) — asserted as a ratio, not just an absolute | harness run → warm-no-change ≪ cold (record the ratio) | serious | 0014 §2.1 (the incremental win) | open | | The headline win: the warm path doesn't re-parse. |
| P-4 | **Warm small-delta** (1 changed file) cost is **flat in corpus size** (≈ no-change + one re-parse) across 1k→100k — delta-cost, not corpus-cost | harness run → small-delta time ~constant across scales | serious | 0014 §2.1 | open | | The "cost scales with the delta" claim (A-9 echoes this at arc scale). |
| P-5 | **Snapshot load** (decode) at 100k is sub-second; the on-disk size is recorded | harness run → load time + file size at 100k recorded; sub-second | serious | 0014 §1 (`[P]`→`[E]`) | open | | Validates "single snapshot file fine to tens of MB / sub-second load." |
| P-6 | A **consumer read** at 100k (`reconcile`→adapter→graph→`check`/`next`) is measured; the verdict on slice07's deferred question (is eager in-memory rebuild acceptable at 100k, or does it dominate?) is **recorded** | harness run → consumer-read time at 100k recorded + a written verdict | serious | slice07 deferred Q / 0014 §2.4 | open | | Measures + records only — does **not** build caching (out of scope; a post-benchmark decision if it dominates). |
| P-7 | The measured table is recorded durably (numbers + toolchain/machine context), and ODD-0014's `[P]` performance claims are promoted to `[E]` — the **scaling** as the durable claim, absolutes as context | `grep -nE '\[E\]' docs/design/06-final/0014-*.md` shows the promoted claims AND a benchmark-results record exists | serious | 0014 §4 / arc-plan A-13 | open | | A-13's deliverable. Don't pin a literal-ms `[E]`; assert scaling, record absolutes + environment. |
| P-8 | The harness builds + runs (release); clippy `-D warnings`; no `unsafe`; the generator (lib code) coverage ≥ 90% (line); benches excluded from the coverage gate | `cargo bench`/harness → runs; `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src` AND `cargo llvm-cov --summary-only` (generator) → **line** ≥ 90% | serious | CLAUDE.md | open | | Harness mechanism CC's choice (benches/criterion, feature-gated test, or examples bin). |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Rows: 8. On close: A-13 satisfied; ALL 8 slices delivered → the arc-close runs.)_
