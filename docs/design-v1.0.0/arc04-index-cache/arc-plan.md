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
4. **slice04 — enrich record + maps + wire `list` (seam a, delivered).** Enrich
   `IndexRecord` so `gates` carries per-gate **evidence** (+ `number`/`component`) —
   `FORMAT_VERSION 1 → 2` (an old index → `RebuildNeeded(VersionMismatch)` → cold
   rebuild, via slice01's self-heal); in-memory type/tag/gate/edge maps on load; `list`
   index-backed (`reconcile`-then-read), `--json` stays `load_all` (the §3.5 filter/sort
   boundary). — `odm-index` + `odm-cli`. *(Was the whole consumer-wiring; split at the
   named seam — see v1.7/v1.8. Seams b+c → slice05.)*
5. **slice05 — index→graph adapter + derived-order readers (delivered).** The adapter
   reconstructs `Frontmatter`s from index records (no parse) → feeds the *unchanged*
   `NodeGraph::build`/`Satisfaction::compute`; `next`/`blocked`/`path` read the index
   (`reconcile`-then-read), identical to baseline. — `odm-index` + `odm-cli`.
   *(`check`/`rollup`/`orient` need record fields the adapter lacked — `origin` +
   `decomposed` — split to slice06; see v1.9.)*
6. **slice06 — enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient` (slice05
   continuation).** Enrich `IndexRecord` with `origin` (rollup provenance) + `decomposed`
   (check recomposition) — `FORMAT_VERSION 2 → 3` (old index self-heals); refactor
   `check`'s `aggregate` to take `&[Frontmatter]`; wire `check`/`rollup`/`orient` off the
   index (`reconcile`-then-read); `orient` loads only the current project's `Document`
   for the vision body (0014 §3.5). Identical to the full-scan baseline. **Closes A-4 +
   A-5; satisfies the consumers-read-the-index compose row (A-12).** *(The complete
   remaining field gap is `origin`+`decomposed` — grep-verified; no third surprise.)*
7. **slice07 — early-cutoff invalidation** *(was slice05→06)*. Distinguish `content_hash`
   (did the file change?) from `meta_hash` (did its *meaning* change?); a body-only
   change updates the record but recomputes nothing downstream.
8. **slice08 — benchmark harness** *(was slice06→07)*. Synthetic 1k/10k/100k corpora;
   measure cold/warm/load latency; promote the 0014 `[P]` claims to `[E]`; record the
   numbers.

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
| A-4 | slice04 (seam a: enrich + maps + index-backed `list`) closed | ptr: slice04 `cdc-verification.md` | correctness | arc-plan | open | seam (a) attested: CC closing-report `2dafaa1` (7/10); CDC-verified on structure (`slice04-enrich-and-wire/cdc-verification.md`); cargo rows pending CI. **Seams (b)+(c) now delivered** — slice06 wired the graph readers + composed views (closing-report this slice). | → `done` when seam (a) reproduces (CI green); functionally closed by slice06. |
| A-5 | slice05 (index→graph adapter + derived-order readers) closed | ptr: slice05 `cdc-verification.md` | correctness | arc-plan | open | adapter + derived-order attested: CC closing-report `89a2223` (4/7 — G-1 adapter, G-2 `next`/`blocked`/`path`, G-6, G-7); CDC-verified on structure (`slice05-graph-adapter-and-views/cdc-verification.md`); cargo rows pending CI. **The deferred `check`/`rollup`/`orient` are now wired** (slice06, A-6). | → `done` when it reproduces (CI green); the deferral it carried is closed by slice06. |
| A-6 | slice06 (enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient`; slice05 continuation) closed | ptr: slice06 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report this slice (8/8; line cov odm-index 97.49% / odm-cli 93.63%); `FORMAT_VERSION 2 → 3` (v2 self-heals, no migration code); `aggregate` takes `&[Frontmatter]`; shared `index_frontmatters` feeds `check`/`rollup`/`orient` + `Derived`; one targeted `store.load` for `orient`'s vision body. Awaiting CDC reproduce. | attested-on-close. Carries G-3/G-4/G-5 from slice05. **Closes A-4 + A-5; makes A-12 satisfiable** (every read path index-backed). |
| A-7 | slice07 (early-cutoff invalidation) closed | ptr: slice07 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report this slice (6/6; line cov odm-index 95.18% / odm-cli 93.68%); `Delta` gains `meta_changed`; `Snapshot::meta_fingerprint` stamps `ROLLUP.md`; `odm rollup` skips on a body-only edit, regenerates on meta-change/new/deleted; in-memory readers unchanged (§2.4). Awaiting CDC reproduce. | → `done` when slice07 reproduces (CI green). Only **slice08** (benchmark) remains in the arc. |
| A-8 | slice08 (benchmark) closed | ptr: slice08 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-9 | **Compose:** first run builds + persists; a subsequent run touches only the delta (cost scales with the change, not the corpus) | arc-scale demo: cold run, then warm run over an unchanged-but-one corpus; observe delta-only work | serious | arc-plan | open | | reproduce at arc scale |
| A-10 | **Compose:** change detection is racy-git-correct end-to-end — a same-tick, same-size in-place edit is caught (would fail under a stat-only path) | arc-scale demo: craft the racy case; warm run detects it | serious | arc-plan | open | | reproduce at arc scale |
| A-11 | **Compose:** a missing/corrupt index self-heals (rebuilt from node files; carries no authority) | arc-scale demo: delete/corrupt `.odm/` index; next run rebuilds; results identical | serious | arc-plan | open | | reproduce at arc scale |
| A-12 | **Compose:** `list`/`orient`/graph-build read the index and match the full-scan baseline behavior | arc-scale: the **adapter-fidelity chain** (synth frontmatter == parsed for every field each consumer reads — graph G-1, origin+decomposed V-3) ∘ the *unchanged* shared `aggregate`/`assemble`/`integrity`, **+** each consumer's correctness + idempotence (warm == cold) | serious | arc-plan | open | **Satisfiable as of slice06 (A-6):** every read path — `list`, derived-order, `check`, `rollup`, `orient` — is index-backed. **No live full-scan-vs-index diff: the `load_all` path is removed once a consumer is wired** (v1.12) — equivalence rests on the adapter-fidelity chain, not an A/B demo. | reproduce at arc scale |
| A-13 | **Compose:** the 100k-node benchmark promotes ODD-0014's `[P]` perf claims to `[E]` | arc-scale demo: run slice08's harness; record the numbers | serious | arc-plan | open | | reproduce at arc scale |
| A-14 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | open | | accrues as slices close |

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

### v1.13 — 2026-06-29
**slice07 closed (A-7 attested; early-cutoff in).** Cashed the two-fingerprint split:
`reconcile`'s `Delta` gains `meta_changed` (the semantic subset of `changed` — a
`meta_hash` compare at the CHANGED arm), and `odm rollup` skips regenerating `ROLLUP.md`
when the corpus is semantically unchanged — via a `Snapshot::meta_fingerprint` (hash over
each record's `(id, meta_hash)`) stamped in the generated header. A body-only edit
refreshes the index record but regenerates nothing downstream; any meaning-change / new /
deleted node regenerates. In-memory readers unchanged (0014 §2.4 — no lazy/persistent
graph caching). 6/6 rows attested; line cov odm-index 95.18% / odm-cli 93.68%; clippy
`-D warnings` clean; no `unsafe`. No new arc-level finding. **Only slice08 (the 100k
benchmark) remains.** CC closing-report: `slice07-early-cutoff/closing-report.md`.
Cargo/coverage rows reproduce via CI.

### v1.12 — 2026-06-29
**CDC verification of slice06 — A-12 verify-method corrected (plan-keeping).** slice06
wired `check`/`rollup`/`orient` off the index, which *removed* their `load_all` paths —
so A-12's original Verify ("forced-full-scan vs. index diff") is now **impossible**.
Corrected A-12's Verify to the real reproduction: the **adapter-fidelity chain** (synth
frontmatter == parsed for every field each consumer reads — graph G-1 + origin/decomposed
V-3) ∘ the unchanged shared `aggregate`/`assemble`/`integrity`, plus each consumer's
correctness + idempotence. **Standing note:** with no A/B safety net, the adapter-fidelity
tests are the load-bearing equivalence guarantee — any future consumer reading a new
field must extend the adapter *and* its fidelity test. Surfaced by: CC's slice06 flag #1
+ CDC verification.

### v1.11 — 2026-06-29
**slice06 closed (A-6 attested; consumer-wiring done).** Enriched `IndexRecord` with
`origin` + `decomposed` (the grep-verified two-field gap — `Decomposition` mirrored
index-side to dodge postcard `skip_serializing_if` desync), bumped `FORMAT_VERSION 2 → 3`
(v2 self-heals, no migration code), extended the adapter (provenance + recomposition
fidelity), refactored `check`'s `aggregate` to `&[Frontmatter]`, and wired `check`/
`rollup`/`orient` off the shared `index_frontmatters` (reconcile-then-read) — `orient`
keeping one targeted `store.load` for the vision body (§3.5). 8/8 rows attested; line cov
odm-index 97.49% / odm-cli 93.63%; clippy `-D warnings` clean; no `unsafe`. **A-4 + A-5
close** (their deferred seams delivered) and **A-12 is satisfiable** (every read path
index-backed, matches baseline). No new arc-level finding. CC closing-report:
`slice06-views-and-check/closing-report.md`. Cargo/coverage rows reproduce via CI.

### v1.10 — 2026-06-29
**Renumber executed for the slice05 split (CDC, per v1.9's operator request; Duncan's
v1.8 scheme).** **slice06 = the continuation** (enrich `origin`+`decomposed` →
`FORMAT_VERSION 2 → 3`, `aggregate` refactor, wire `check`/`rollup`/`orient`);
early-cutoff → **slice07**; benchmark → **slice08**. Arc Ledger: class-(a) now A-1…A-8
(A-6 new = slice06); compose **old A-8…A-12 → A-9…A-13**; bubble-up **old A-13 → A-14**.
**CDC owns the proximate miss** v1.9 names diplomatically: the slice05 doc's
adapter-field list was *mine* and overlooked `origin`+`decomposed`. **Bounded + frozen:**
the complete remaining gap is grep-verified to be exactly those two fields, so this is
the **last insert — arc04 is frozen at 8 slices** and the renumber-bridges stop here.
> **Bridge note (consolidated — supersedes v1.8's; live table is authoritative).** The
> *consumers-read-the-index* compose row (cited **A-10** in v1.4/v1.5/v1.7, **A-11** in
> v1.8/v1.9) is now **A-12**. The *bubble-up accrual* row (A-12 pre-v1.8, **A-13** in
> v1.8/v1.9) is now **A-14**. IDs **A-1…A-5 are unchanged.** Resolve any other old A-N
> citation by its *description* against the current table, not its number.
>
> **Process note:** two renumber-bridges in one arc is the number-as-identity brittleness
> odm itself exists to kill — harmless in the markdown-bootstrap phase, impossible once
> self-hosting (A6). A standing reminder of *why* we're building this. If a third insert
> ever arises, switch to stable row IDs rather than a third bridge.
Surfaced by: CDC execution of the v1.9 operator request.

### v1.9 — 2026-06-29
**slice05 partial bubble-up + a second split** (A-5 adapter+derived-order attested; A-13
accrual). slice05 delivered its **crux — the index→graph adapter** (G-1, synthesize
frontmatters → feed the existing graph/satisfaction unchanged) **+ the derived-order
readers** `next`/`blocked`/`path` (G-2), index-backed and identical to baseline (commit
`89a2223`). **Deferred: `check` (G-3), `rollup` (G-4), `orient` (G-5)** to a continuation.
**The arc-level finding:** *wiring all consumers off the index is ~3 slices, not 1* —
each consumer reads a different frontmatter projection, so the **record must grow per
consumer**. slice04 added `number`/`component` for `list`; **the composed views + `check`
read `origin` (rollup provenance) and `decomposed` (`check` recomposition), which the
record does not carry.** The slice plan's adapter-reconstruction list
(id/number/type/name/edges/status) overlooked both. **Operator action requested:** scope
the continuation — enrich the record with `origin` + `decomposed` (`FORMAT_VERSION 2 →
3`, self-heal handles old indexes) + refactor `aggregate` to `&[Frontmatter]`, then
source-swap `rollup`/`orient`/`check` to reconcile→adapter→model. **A-4 stays open**
until it lands; **A-11** (index-backed graph == baseline) is **satisfied for the
derived-order readers** now. Surfaced by: the slice05 closing-report bubble-up.

### v1.8 — 2026-06-29
**Renumber executed (CDC, per v1.7's operator request; scheme chosen by Duncan).** The
slice04 split is realized: **slice05 = the continuation** (index→graph adapter + graph
readers + composed views, seams b+c); the former early-cutoff/benchmark shift to
**slice06 / slice07**. The Arc Ledger was renumbered sequentially: class-(a) is now
A-1…A-7 (A-5 new = slice05); compose rows shifted **old A-7…A-11 → A-8…A-12**; bubble-up
**old A-12 → A-13**.
> **Bridge note (reference translation).** Earlier version-history entries (v1.4, v1.5,
> v1.7) and slices 02/03's closed bubble-ups cite **pre-renumber** IDs. To translate any
> reference written before v1.8: **add +1 to any ledger ID ≥ A-7.** So the old "A-10"
> (the *consumers-read-the-index* compose row) is now **A-11**, and the old "A-12"
> (bubble-up accrual) is now **A-13**. IDs A-1…A-6 are unaffected by the translation
> (A-5/A-6's *text* was relabeled to the renumbered slices, but those IDs predate the
> shift).
Surfaced by: CDC execution of the v1.7 operator request.

### v1.7 — 2026-06-29
**slice04 seam-(a) bubble-up + split request** (A-4 seam-a attested; A-12 accrual).
v1.6 kept slice04 as one slice with a named split seam as the execution-time fallback;
**that fallback fired.** CC delivered **seam (a) — enrich + maps + index-backed `list`**
(rows I-1…I-5, I-9-`list`, I-10; commit `2dafaa1`) and deferred **(b) index→graph
adapter + graph readers** (I-6, I-7) and **(c) composed views** (I-8) to a continuation.
Per the prompt, CC did **not** self-name a `04b`. **Operator action requested: renumber
the continuation** (e.g. insert a slice for b+c and push the current slice05/06 down, or
a `slice04-cont`) — A-4 stays `open` until it lands. Findings dispositioned:
1. **`list` needs `number` + `component`** in the record (beyond per-gate evidence) for
   its table + filters — added in seam (a). **A-10 input:** index-backed `list` table +
   filters match baseline.
2. **`list --json` §3.5 boundary:** the full-node dump (`origin`/`reserved`/`retired`)
   stays `load_all` — the index is the filter/sort accelerator, not a full-node store.
   **Open for ratification:** accept the boundary (recommended) vs. grow the record for
   full `--json` parity. **A-10 input:** "identical to baseline" holds for the human
   table + filters; full-node `--json` parity is a separate call.
3. **The index→graph adapter (seam b) is a slice's worth on its own** — synthesize
   `Frontmatter`s from records → feed the existing `NodeGraph`/`Satisfaction` (no parse,
   no re-derive). Sketch in the slice04 closing report for the continuation to build on;
   the record now carries the evidence it needs.
Surfaced by: the slice04 seam-(a) closing-report bubble-up.

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
