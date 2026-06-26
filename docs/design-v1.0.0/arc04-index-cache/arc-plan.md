# Arc 04 — Index & cache (plan-of-record)

> Refs: ODD-0014 (the cited research pass — *the* design source for this arc),
> ODD-0013 §6.1 (index/cache mini-infra) + Q-9; ODD-0015 A4 row; `project-plan.md` §2.
> `depends_on:` A1–A3 (the store, the graph build, and the rollup/orient consumers
> that will read the index instead of re-walking).
>
> **Status:** planned, not started. Per *plan late, plan deep*, this arc-plan carries
> the slice breakdown at one-line altitude; the per-slice doc sets are written when the
> arc becomes active.

## Capability

A read-acceleration **mini-infra** — the `odm-index` crate — that makes "which files
define which nodes?" and metadata filter/sort fast at scale **without a database, an
FTS engine, or a daemon** (the "as little infra as possible — i.e. none" line). It is
a persisted, sorted, derived **stat-cache** under `.odm/` (gitignored, never truth):
the *first* run pays a full scan + hash + parse and persists; *subsequent* runs
`lstat`-compare and touch only the delta. It is **racy-git-correct** (size + conditional
content-hash, never stat-only), **self-healing** (corrupt/missing ⇒ rebuild from the
node files), and it feeds `list` / `orient` / the graph build so they stop re-walking
the tree. This is the A4 capability the arc's slices must compose into.

## Exit criteria (arc acceptance)

- First run builds + persists the index; subsequent runs cost scales with the *delta*,
  not the corpus.
- Change detection is **correct under the racy-git case** (an in-place same-size edit
  within the cache-write tick is caught via the `mtime >= index-timestamp` content-hash
  fallback) — verified by a test that would fail under a stat-only shortcut.
- A missing or corrupt index is silently rebuilt; the index carries no authority.
- `list`/`orient`/graph-build read the index, not a fresh walk; behavior is identical
  to the full-scan baseline (the index is an accelerator, not a semantic change).
- A benchmark over synthetic 1k/10k/100k corpora promotes ODD-0014's `[P]` performance
  claims to `[E]`.

## Slices (dependency-ordered, one-line scope)

1. **slice01 — index record + snapshot persistence.** The `IndexRecord` shape
   (id, rel_path, stat fields, content_hash, meta_hash, extracted metadata) + a
   versioned header (magic, format-version, hash-algo, index-timestamp, count, trailing
   checksum); atomic write (temp + rename + fsync of file *and* dir); `postcard` or
   `bincode 2`. — `odm-index`.
2. **slice02 — cold-path build.** `walkdir` the `nodes/` tree; `lstat` + read + hash +
   parse frontmatter per file; build records; persist. O(corpus), paid once.
3. **slice03 — warm-path change detection (the racy-correct delta).** Load + verify
   header/checksum; `lstat`-compare on size/mtime/mode; the `>=` racy test → content-hash
   fallback; deletion detection; re-stamp + persist on change. The correctness core.
4. **slice04 — in-memory filter/sort + wire consumers.** Build type/tag/state/edge maps
   on load; point `list`, `orient`, and the graph build at the index instead of a fresh
   walk; confirm identical behavior to the full-scan baseline.
5. **slice05 — early-cutoff invalidation.** Distinguish `content_hash` (did the file
   change?) from `meta_hash` (did its *meaning* change?); a body-only change updates the
   record but recomputes nothing downstream.
6. **slice06 — benchmark harness.** Synthetic 1k/10k/100k corpora; measure
   cold/warm/load latency; promote the 0014 `[P]` claims to `[E]`; record the numbers.

## Dependencies

Consumes: A1's store + frontmatter parse, A2's graph build, A3's rollup/orient (the
consumers rewired in slice04). Leaves for later: nothing blocks A5/A6 on A4 — A4 is a
performance arc, independently shippable.

## Open design questions (resolve in slice docs; ODD-0014 §4 guardrails)

- **rkyv + mmap (zero-copy):** deferred — attractive but adds layered `unsafe` and
  per-major format breaks, and load-latency dominance is **unmeasured**. Start with
  postcard/bincode-2; revisit only if slice06's benchmark demands it.
- **Sharding:** deferred — stay single-file until measured load/rewrite cost bites;
  then shard a *snapshot* by ULID id-prefix (fixed count, never `hash mod N`).
- **Filesystem watcher (ODD-0014 §5):** out of arc scope — correctness must rest on the
  stat-walk; a watcher is opt-in acceleration only, a later add if interactive latency
  ever matters.
- **Text search:** linear scan (grep-style) is the dependency-free default; a hand-rolled
  inverted index only if a real large-corpus body-search requirement appears — never an
  FTS engine dependency.

## Method

Ledger per slice; CC implements, CDC verifies every row; cargo rows via CI / local
1.85+. Five-iteration cap. Slice closes bubble up to this arc-plan (Part IV); the arc
closes with its own `closing-report.md` + composition check (Part V).

## Version History

### v1.0 — 2026-06-26
Initial arc-plan, drafted from ODD-0014 (research) + the ODD-0015 A4 row, as part of
the project-plan synthesis session. No slices started; breakdown at one-line altitude
per *plan late, plan deep*.
