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
4. **slice04 — enrich record + in-memory filter/sort + wire consumers.** Enrich
   `IndexRecord` so `gates` carries per-gate **evidence** (not just reached names) —
   `FORMAT_VERSION 1 → 2` (an old index → `RebuildNeeded(VersionMismatch)` → cold
   rebuild, via slice01's self-heal) — so the graph build can compute evidence-leveled
   satisfaction off the index. Build type/tag/gate/edge maps on load (filter "by gate"
   = by reached gate — odm has no scalar state, per the slice01 bubble-up, v1.2); an
   index→graph adapter feeds the DAG + satisfaction from index records (no frontmatter
   parse). Point `list`, the graph readers (`next`/`blocked`/`path`/`check`), and the
   composed views (`rollup`/`orient`) at the index (via `reconcile`-then-read);
   `orient` loads only the *current project's* `Document` for the vision body (one
   targeted load — the index deliberately carries no bodies, 0014 §3.5). Confirm
   identical behavior to the full-scan baseline. *(Large slice — split seam, if needed
   at execution: enrich+maps+`list` | graph adapter+readers | composed views.)*
5. **slice05 — early-cutoff invalidation.** Distinguish `content_hash` (did the file
   change?) from `meta_hash` (did its *meaning* change?); a body-only change updates the
   record but recomputes nothing downstream.
6. **slice06 — benchmark harness.** Synthetic 1k/10k/100k corpora; measure
   cold/warm/load latency; promote the 0014 `[P]` claims to `[E]`; record the numbers.

## Arc Ledger

> Per LEDGER-DISCIPLINE v2.0 §B (Option A: the arc ledger lives here in `arc-plan.md`
> and closes in the companion `closing-report.md`). Opens now with the class-(b)
> composition rows stated up front from the capability; accrues class-(a) slice-closed
> rows and class-(c) bubble-up rows as slices close. **Class-(b) rows are reproduced at
> the arc scale — an end-to-end demonstration, never inherited from the slices.**

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| A-1 | slice01 (record + persistence) closed | ptr: slice01 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report `69952ce` (8/8); CDC-verified on structure (`slice01-record-persistence/cdc-verification.md`); cargo rows pending CI | → `done` when slice01 reproduces (CI green). **Convention:** Status ∈ open/done/deferred/no-op; the evidence *strength* lives in the Evidence column. |
| A-2 | slice02 (cold-path build) closed | ptr: slice02 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report `d03d5d0` (9/9, line cov 98.68%); awaiting CDC reproduce (`attested → reproduced`) | → `done` when slice02 reproduces (CI green). |
| A-3 | slice03 (warm-path racy-correct delta) closed | ptr: slice03 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report `e53bc44` (10/10, line cov 98.36%); awaiting CDC reproduce (`attested → reproduced`) | → `done` when slice03 reproduces (CI green). The racy `>=` content-hash fallback (the correctness core) is in. |
| A-4 | slice04 (filter/sort + wire consumers) closed | ptr: slice04 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-5 | slice05 (early-cutoff invalidation) closed | ptr: slice05 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-6 | slice06 (benchmark) closed | ptr: slice06 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-7 | **Compose:** first run builds + persists; a subsequent run touches only the delta (cost scales with the change, not the corpus) | arc-scale demo: cold run, then warm run over an unchanged-but-one corpus; observe delta-only work | serious | arc-plan | open | | reproduce at arc scale |
| A-8 | **Compose:** change detection is racy-git-correct end-to-end — a same-tick, same-size in-place edit is caught (would fail under a stat-only path) | arc-scale demo: craft the racy case; warm run detects it | serious | arc-plan | open | | reproduce at arc scale |
| A-9 | **Compose:** a missing/corrupt index self-heals (rebuilt from node files; carries no authority) | arc-scale demo: delete/corrupt `.odm/` index; next run rebuilds; results identical | serious | arc-plan | open | | reproduce at arc scale |
| A-10 | **Compose:** `list`/`orient`/graph-build read the index and match the full-scan baseline behavior | arc-scale demo: same outputs index-backed vs. forced full-scan | serious | arc-plan | open | | reproduce at arc scale |
| A-11 | **Compose:** the 100k-node benchmark promotes ODD-0014's `[P]` perf claims to `[E]` | arc-scale demo: run slice06's harness; record the numbers | serious | arc-plan | open | | reproduce at arc scale |
| A-12 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | open | | accrues as slices close |

Closes in `arc04-index-cache/closing-report.md`: the per-row walk + composition verdict,
independently gated (fresh context / operator). A failed class-(b) row spawns a
**remediation slice** (not a re-pass) via the plan-change discipline.

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

### v1.6 — 2026-06-29
**slice04 scope resolved (CDC planning + Duncan).** A design fork surfaced while
drawing up slice04: the index carries edges + reached-gate *names* but **not** per-gate
evidence (the graph commands need it for evidence-leveled satisfaction) or bodies
(`orient` needs the vision body). Decision (Duncan, Option A): **enrich `IndexRecord`
with per-gate evidence → `FORMAT_VERSION 1 → 2`**, and wire *all* consumers — `list`,
the graph readers, and the composed views — off the index, with `orient` doing one
targeted project-body load. The slice04 body line is updated accordingly. Kept as **one
slice** (not split) to avoid a mid-sequence renumber that would break the `A-10`/`A-12`
arc-ledger back-references in slices 01–03's closed bubble-ups; a split seam is named in
the slice-doc as the execution-time fallback. The slice02 `FORMAT_VERSION` freeze
watch-item is **discharged here** (the bump the watch anticipated). Surfaced by: CDC
planning of slice04, not a slice bubble-up.

### v1.5 — 2026-06-26
**slice03 bubble-up** (A-3 attested; A-12 accrual). slice03 delivered the warm path
(load → lstat-classify → racy `>=` content-hash → delta → persist-on-change) with no
silent drops; the racy correctness core is in. Three findings dispositioned:
1. **Delta shape fixed:** `{ rebuilt, new[], changed[], deleted[], clean: count }`.
   **slice05 input:** early cutoff reads `delta.changed` (diff each changed record's
   `meta_hash` vs. prior to decide downstream recompute) + `delta.deleted`; `new`
   always recomputes; `clean` is a no-op count. A5/reconcile can reuse the same Delta.
2. **The index now has a maintained warm path** (`reconcile`). **slice04 / A-10 input:**
   the consumers (`list`/`orient`/graph-build) read a *reconciled* snapshot — call
   `reconcile` before reading, not just cold `build`.
3. **A `node_paths` double-stat micro-cost** (warm stats for the cheap signal;
   `build_one` re-stats on NEW/CHANGED). Negligible; **slice06 benchmark** confirms it
   does not matter at 100k.
Surfaced by: the slice03 closing-report bubble-up section.

### v1.4 — 2026-06-26
**slice02 bubble-up** (A-2 attested; A-12 accrual). slice02 delivered the cold path
(walk → stat+hash+parse → records → persisted snapshot) with no silent drops. Three
findings dispositioned, none forcing a re-break:
1. **`EdgeRef` qualifier fidelity resolved (the slice-doc's open question):** `EdgeRef`
   was enriched to carry `depends_on.satisfied_at`, supersede-kind, and tear `because`.
   **slice04 input:** the index-backed graph-build reads satisfaction (`satisfied_at`)
   and `orient`'s active-tears (`because`) straight from the index — it does **not**
   re-read frontmatter for ordering/satisfaction (strengthens A-10). Format-version
   stayed 1 (no on-disk index exists; the crate is wired into no command yet — a bump
   is required if any command persists an index before slice04).
2. **`meta_hash` field set fixed:** `node_type, gates, tags, edges, title` (sorted),
   **excluding `updated` + stat fields**. **slice05 input:** early cutoff compares this
   exact set; a body-only edit (content_hash differs, meta_hash same) recomputes nothing.
3. **postcard format-evolution note:** `serde` `skip_serializing_if` desyncs a
   non-self-describing stream — index record/format fields must stay always-serialized;
   any future "optional" field is a **format-version bump**, never a skip.
Surfaced by: the slice02 closing-report bubble-up section.

### v1.3 — 2026-06-26
**CDC verification of slice01** (plan-keeping). Two reconciliations: (a) propagated v1.2
finding #1 into the **body** — slice04's "type/tag/state/edge maps" → "type/tag/**gate**/
edge maps" (the plan-change discipline is change-the-body *and* log it, not log-only);
(b) **normalized the arc-ledger status convention** — A-1's Status was `attested` (an
evidence *strength*); set it to `open` with the strength in the Evidence column, so
A-2…A-6 inherit the right shape (`done` requires ≥ `reproduced`). Surfaced by: CDC
verification (`slice01-record-persistence/cdc-verification.md`), not a slice bubble-up.

### v1.2 — 2026-06-26
**slice01 bubble-up** (A-1 attested; A-12 accrual). slice01 delivered its assigned
piece (the record + snapshot format + persistence + self-heal) with no silent drops.
Three findings dispositioned for downstream slices, none forcing a slice re-break:
1. ODD-0014's generic `state: State` field has **no odm scalar** — status is a
   multi-gate vector — so the record stores `gates: Vec<String>` (the reached-gate
   set). **slice04 input:** the filter/sort "by state" affordance is "by reached
   gate"; build the maps on the gate set.
2. The `HashAlgo` enum + 1-byte on-disk algo id make the ODD-0014-recommended
   xxh3 fingerprint swap a **format-versioned, non-breaking change** — the hook is
   in place if slice06's benchmark shows hashing dominates (F-7 deferred it).
3. The index **owns its `EdgeKind`** (mirrors, not reuses, `odm_core`'s) so the
   wire format is governed by the snapshot format-version; **slice02** maps domain
   edges → `EdgeRef` on populate.
Surfaced by: the slice01 closing-report bubble-up section.

### v1.1 — 2026-06-26
Added the **`## Arc Ledger`** section (class-(a) slice-closed, class-(b) composition,
class-(c) bubble-up rows) when arc04 became the active work, per LEDGER-DISCIPLINE v2.0
§B. Pure addition — the v1.0 body is unchanged. Surfaced by: the ledger-discipline
upgrade (v1→v2.0), not a slice bubble-up.

### v1.0 — 2026-06-26
Initial arc-plan, drafted from ODD-0014 (research) + the ODD-0015 A4 row, as part of
the project-plan synthesis session. No slices started; breakdown at one-line altitude
per *plan late, plan deep*.
