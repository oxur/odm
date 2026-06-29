# Closing report — Arc 04 / Slice 04: Enrich record + wire consumers (seam a)

> CC implementation closing report. Status: **proposed-done (`attested`) for seam
> (a)**; seams (b)+(c) **deferred to a renumbered continuation** (the cc-prompt's
> sanctioned split). Impl commit `2dafaa1`; docs commit (this report + ledger +
> arc-plan bubble-up) follows.

## The split — what shipped, what's deferred, and why

The cc-prompt and slice-doc both flagged slice04 as large and pre-named the split
seam: **(a) enrich + maps + `list`** · **(b) index→graph adapter + graph readers**
· **(c) composed views**. Two things made (a) the right place to stop with
iteration headroom intact:

1. **Seam (b) is the crux and is substantial** — an index→graph adapter that
   builds `NodeGraph` + `Satisfaction` from records with no frontmatter parse,
   then rewires four readers (`next`/`blocked`/`path`/`check`) with four
   identical-to-baseline tests, plus the adapter==frontmatter-graph test. (c)
   stacks `rollup`/`orient` on top.
2. **Seam (a) itself surfaced a design fork** (the `list --json` full-node
   boundary, below) worth settling and recording before piling on (b)/(c).

Delivering (a) cleanly + flagging (b)/(c) honours the prompt ("split at the named
seam … route the continuation via the bubble-up; do not invent a `04b` name —
flag it and we renumber properly"). **I did not create a `04b` directory.**

## Per-row ledger walk (10 rows)

- **I-1 — done.** `IndexRecord.gates` is `Vec<GateState>` (gate + `Evidence`).
- **I-2 — done.** `FORMAT_VERSION = 2`; a forged v1 file loads as
  `RebuildNeeded(VersionMismatch{found:1})` and `reconcile` rebuilds cold — no
  migration code (slice01 self-heal). Discharges the slice02 freeze watch-item.
- **I-3 — done.** `build_one` populates per-gate evidence; `meta_hash` covers it
  (an evidence change flips the hash), still excluding `updated`/stat.
- **I-4 — done.** `IndexMaps` builds `type→ids`/`tag→ids`/`gate→ids` + edge
  adjacency on load; no disk, no FTS. 100% line.
- **I-5 — done.** `list`'s human table is index-backed (reconcile-then-read);
  identical to baseline; filters narrow. See the fork below for `--json`.
- **I-6 — deferred (seam b).** Index→graph adapter. Re-entry: renumbered
  continuation. The record now carries the evidence the adapter needs.
- **I-7 — deferred (seam b).** `next`/`blocked`/`path`/`check` index-backed.
- **I-8 — deferred (seam c).** `rollup`/`orient` index-backed + targeted vision
  body load.
- **I-9 — done for `list`.** `list` reconciles before reading (a node added after
  the first list shows on the next, no manual rebuild). The same wrapper extends
  to the graph/composed consumers in the continuation.
- **I-10 — done (seam-a scope).** clippy `-D warnings` → exit 0; no `unsafe`; line
  coverage **odm-index 98.53% / odm-cli 93.82%**; fmt clean; workspace `cargo
  test` → 263 passed.

## Decisions / deviations flagged (not buried)

1. **Enrichment beyond "evidence-only."** The prompt scoped the record enrichment
   to per-gate evidence (what the *graph* needs). But `list` (seam a's consumer)
   needs `number` (its table column) and `component` (a filter) — neither was in
   the record. I added both. This is a necessary, §3.5-aligned enrichment for
   index-backed `list`; flagged because it exceeds the prompt's wording.
2. **`list --json` stays `load_all` (the §3.5 boundary).** The baseline
   `list --json` emits a full `NodeJson` — `origin`, `reserved`, `retired` — which
   the index deliberately does **not** carry (ODD-0014 §3.5: the index is the
   filter/sort accelerator, not a full-node store). So the human **table +
   filters** are index-backed (identical to baseline), and `--json` (a full-node
   dump, not a filter/sort op) stays `load_all`. I-5 "identical to baseline" holds
   for the table + filtering; the `--json` path is unchanged. **Operator/CDC call:**
   accept this boundary, or expand the record to carry `origin`/`reserved`/`retired`
   (a larger enrichment) so `--json` is also index-backed. I recommend the boundary
   (keeps the index lean, §3.5-true); flagged for ratification.
3. **`meta_hash` excludes `number`/`component`.** They are display/filter metadata,
   not graph/rollup *meaning*; a component-only change should not force downstream
   recompute. Consistent with the slice02 meta_hash intent (it already excludes
   `updated`). Flagged.
4. **`list` filters the reconciled records directly**, not via `IndexMaps`. Direct
   filtering is trivially identical-to-baseline; `IndexMaps` is delivered + tested
   (I-4) as the reusable filter primitive that the seam-(b) adapter and any hot
   path will use. Flagged so the CDC sees the map isn't yet on `list`'s path.

## Uncertainties / things CDC should look at

- **The renumber.** Seams (b)+(c) need a new slice number (e.g. a slice05 that
  pushes the current slice05/06 down, or a `slice04-cont`). That's an
  operator/planning call — the bubble-up requests it; I did not presume a name.
- **The adapter approach (seam b), for early validation.** Sketch: reconstruct an
  `Edges` from each record's `EdgeRef`s (inverse of slice02's `map_edges`) and a
  `Status` from its `GateState`s (via the type's `GateSet`), synthesize a
  `Frontmatter` (id/type/edges/status real; number/origin placeholder — unused by
  the graph), and feed the **existing** `NodeGraph::build` / `Satisfaction::compute`.
  No frontmatter *parse*; the graph logic is reused, not re-derived. Worth a CDC
  sanity check before the continuation builds it.
- **odm-cli now depends on odm-index** (new edge in the crate graph). Intentional
  (the CLI is the index's consumer); noted.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

- **Did slice04 deliver its assigned piece of the A4 capability?** Partially, by
  design: seam (a) — the record now carries what the consumers need (evidence +
  display/filter fields), the in-memory maps exist, and the **first consumer
  (`list`) reads the index** (reconcile-then-read). Seams (b)/(c) — the graph
  readers and composed views — are deferred to a renumbered continuation. This is
  the prompt's sanctioned split, not a silent shortfall.
- **What did it reveal the arc-plan didn't anticipate?**
  - **The record needs more than evidence to back `list`** (`number`,
    `component`) — and a **§3.5 boundary** on `list --json` (full-node fields the
    index shouldn't carry). **Arc-plan input:** A-10 ("consumers read the index …
    identical to baseline") is met for `list`'s table + filters; a full-node
    `--json` parity would require either record growth or staying `load_all`.
  - **slice04 genuinely splits** at the (b) seam — the adapter is a slice's worth
    on its own. **Arc-plan input:** the slice list needs a renumber to carry the
    continuation; A-4 stays `open` (seam a attested) until (b)+(c) land.
- **Slice-scale silent-drop diff (scope-as-specified vs. scope-as-delivered):**
  seam (a) delivered every In item it covers (enrich, format bump, maps, `list`
  index-backed + reconcile-then-read); seams (b)/(c) are **disclosed deferrals**
  with re-entry conditions (the renumbered continuation), not silent drops. 7/10
  rows done, 3 deferred-with-reason. No row softened within seam (a).

## Iterations

One (for seam a). The split was decided up front from the prompt's size guidance
and confirmed by the `list --json` fork; not a five-iteration-cap trigger.
