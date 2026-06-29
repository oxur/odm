# Closing report — Arc 04 / Slice 05: Index→graph adapter + graph readers (partial)

> CC implementation closing report. Status: **proposed-done (`attested`) for the
> adapter + derived-order readers** (G-1, G-2, G-6, G-7); **`check`/`rollup`/`orient`
> (G-3/G-4/G-5) deferred to a continuation** (a record enrichment the slice plan
> overlooked). Impl commit `89a2223`; docs commit follows.

## What shipped, what's deferred, and the finding

The slice's **crux** — the index→graph adapter — is delivered and proven, and the
derived-order readers (`next`/`blocked`/`path`) now read the index. But wiring the
remaining consumers surfaced a concrete blocker the slice plan did not anticipate:

> **The composed views + `check` read frontmatter fields the index record does not
> carry.** `Rollup::assemble`'s provenance view reads `fm.origin()`; `check`'s
> `recompose::integrity` reads `fm.decomposed()`. Neither is in `IndexRecord`. The
> slice-doc's adapter-reconstruction list — *id, number, type, name, edges, status* —
> omitted both. The adapter can reconstruct a **graph-faithful** frontmatter (which is
> all `next`/`blocked`/`path` need), but not a **rollup-/check-faithful** one.

So `rollup`/`orient`/`check` cannot be made identical-to-baseline off the index without
**a further record enrichment** (`origin` + `decomposed`, `FORMAT_VERSION 2 → 3`) plus a
small `check`-aggregate refactor (`&[Document]` → `&[Frontmatter]`). That is a coherent,
separable chunk — its own pass — so I delivered the adapter + readers clean and flagged
the rest, rather than half-wire the composed views against fields that aren't there.

## Per-row ledger walk (7 rows)

- **G-1 — done.** `frontmatters_from_records` synthesizes frontmatters from records
  (inverse `map_edges` + status rebuild) and feeds the *unchanged* `NodeGraph::build` /
  `Satisfaction::compute`. The fidelity test asserts the adapter graph == the frontmatter
  graph on the ready frontier, per-node blocked (evidence-leveled satisfaction),
  topological order, and containment — over a corpus with every edge kind.
- **G-2 — done.** `Derived::load` is index-backed (reconcile → adapter → graph +
  satisfaction); `next`/`blocked`/`path` produce the baseline-correct derived order.
- **G-3 — deferred.** `check` needs `decomposed` (+ the aggregate refactor). Re-entry in
  the continuation.
- **G-4 — deferred.** `rollup` needs `origin`. Re-entry in the continuation.
- **G-5 — deferred.** `orient` composes G-4 (origin) + G-3 (decomposed) + the targeted
  vision-body load. Gated on both.
- **G-6 — done** for the wired consumers (graph readers + `list`): reconcile-then-read
  freshness verified.
- **G-7 — done.** clippy `-D warnings` → exit 0; no `unsafe`; line coverage **odm-index
  98.19%** (adapter.rs 95.83%) **/ odm-cli 94.22%**; fmt clean; workspace `cargo test` →
  266 passed.

## Decisions / deviations flagged (not buried)

1. **Adapter shape (G-1): synthesize frontmatters.** Per the slice-doc's recommended
   seam and slice04's sketch — reconstruct `Frontmatter`s and feed the existing engines,
   zero odm-core change. The fidelity test is the guard. The synth frontmatter is
   graph-faithful but **not** a full node: it carries a placeholder `origin`, no
   `decomposed`, no body — exactly the fields the deferred consumers need (the finding).
2. **The second split.** slice04 split (a) from (b+c); slice05 now splits (b-graph) from
   (b-check + c). Two splits in the consumer-wiring is the signal that **A4 under-scoped
   the "wire all consumers off the index" work**: each consumer reads a different slice of
   the frontmatter, so the record must grow per consumer (slice04: `number`/`component`
   for `list`; slice05-cont: `origin`/`decomposed` for rollup/check). Flagged as the
   arc-level finding (bubble-up).
3. **`aggregate` will need `&[Frontmatter]`** (not `&[Document]`) to be index-backed —
   it only uses frontmatters + `store.path_of`. A bounded refactor, noted for the
   continuation (G-3).

## Uncertainties / things CDC should look at

- **Two defensive arms uncovered in `adapter.rs`** (the `_ =>` fallbacks for a
  malformed depends_on/supersede qualifier) — unreachable through `build_one` (which
  always sets the right qualifier); named, adapter.rs is otherwise 95.83% line.
- **The continuation's enrichment is a `FORMAT_VERSION 3` bump** — the third in the
  arc (v2 was slice04). Cheap via the self-heal, but worth the operator noting the
  cadence: each consumer-wiring step has needed a record field the plan didn't list.
- **`orient`'s targeted body load** (G-5) is still unbuilt; the design (one
  `store.load(project_id)` for the vision body, bodies-stay-out-of-index) is unchanged
  from slice04's plan.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

- **Did slice05 deliver seams b+c?** Partially: **seam b's graph half** — the adapter
  (the crux) + the derived-order readers — is delivered and proven. **Seam b's `check`
  and seam c (composed views)** are deferred, blocked on a record enrichment the plan
  did not anticipate. Disclosed deferral, not a silent drop.
- **What did it reveal the arc-plan didn't anticipate?**
  - **The record must carry `origin` (rollup) and `decomposed` (check)** to back those
    consumers identically — the adapter can't conjure fields the record never stored.
    **Arc-plan input:** A-4 cannot close until a continuation enriches the record
    (`FORMAT_VERSION 3`) + refactors `aggregate`, then wires rollup/orient/check.
    **A-11** (index-backed graph matches baseline) is **satisfied for the derived-order
    readers** now (G-1/G-2); the composed/check half follows the continuation.
  - **"Wire all consumers off the index" is ~3 slices, not 1** (slice04 list; slice05
    graph readers; a continuation for rollup/orient/check) — because each consumer reads
    a different frontmatter projection, so the record grows per consumer. A genuine
    scoping correction for the arc.
- **Slice-scale silent-drop diff:** G-1/G-2/G-6/G-7 delivered in full; G-3/G-4/G-5 are
  disclosed deferrals with concrete re-entry conditions (the `origin`+`decomposed`
  enrichment + aggregate refactor). 4/7 done, 3 deferred-with-reason. No row softened.

## Iterations

One (for the delivered scope). The split was decided once the `origin`/`decomposed` gap
was confirmed by reading the consumer code (`Rollup::assemble` provenance,
`recompose::integrity`); not a five-iteration-cap trigger.
