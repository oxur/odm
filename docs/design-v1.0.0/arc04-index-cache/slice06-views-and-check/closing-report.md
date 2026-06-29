# Closing report — Arc 04 / Slice 06: Enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient`

> The slice05 continuation. Finishes Arc 04's consumer-wiring: the two record
> fields the composed views + `check` needed (`origin`, `decomposed`), and the
> three remaining read paths off the index. **On close, A-4 + A-5 close and
> A-12 is satisfiable.** Branch: `arc04-slice06-views-and-check` (not `main`).

## What shipped

1. **Record enrichment + format bump (V-1/V-2).** `IndexRecord` gains
   `origin: Origin` and `decomposed: Option<Decomposition>`. `Decomposition` is an
   **index-owned mirror** of `odm_core`'s — `odm_core`'s `children` carries
   `skip_serializing_if`, which silently desyncs the non-self-describing postcard
   stream (the exact failure slice02 hit on `EdgeRef`). `FORMAT_VERSION 2 → 3`; an
   on-disk v2 index self-heals through slice01's load path (`RebuildNeeded(VersionMismatch)`
   → cold rebuild) with **no migration code**. `build_one` populates both; the
   `meta_hash`'s `MetaInput` now covers `origin` (provenance) + `decomposed`
   (recomposition) while still excluding `updated`/stat/display fields.
2. **Adapter extension (V-3).** `frontmatters_from_records` reconstructs `origin`
   (into `Frontmatter::new`) and re-affirms `decomposed` (`affirm_decomposed`), so a
   synthesized frontmatter is faithful for `recompose::integrity` + `Rollup::assemble`
   provenance — not just the graph/satisfaction shape slice05 proved.
3. **`aggregate` refactor + three wirings (V-4/V-5/V-6/V-7).** `aggregate` takes
   `&[Frontmatter]` (the minimal seam — it no longer loads the corpus itself). A
   shared `commands::index_frontmatters(store, gates)` (reconcile → adapter) feeds
   `check`, `rollup`, `orient`, and folds in `Derived::load` (the graph readers).
   `orient` resolves its project off the index and does **one** targeted
   `store.load(project_id)` for the vision body (§3.5).

## Per-row ledger walk (8 rows)

- **V-1** — `record_carries_origin_decomposed` + `v2_index_triggers_rebuild` → ok;
  `grep 'FORMAT_VERSION: u16 = 3'` → `snapshot.rs:41`. The stale slice04
  `v1_index_triggers_rebuild` had its now-false `FORMAT_VERSION == 2` guard removed
  (the live guard moved to `v2_…`); it still proves an even-older format self-heals.
- **V-2** — `build_one_origin_decomposed` + `meta_hash_tracks_decomposed_and_origin`
  → ok. The meta-hash flips on an origin change *and* on a decomposition change; the
  pre-existing `meta_hash_tracks_evidence` still proves body/stat/`updated` edits don't.
- **V-3** — `adapter_reconstructs_origin_decomposed` → ok. Asserts
  `Rollup::assemble(real).provenance == assemble(synth).provenance` **and**
  `recompose::integrity(real) == integrity(synth)` over a corpus engineered to contain
  a real `DecompositionDrift` (a positive signal, not empty==empty). slice05's
  `index_graph_adapter_equals_frontmatter_graph` still passes.
- **V-4** — `check_index_backed_matches_baseline` → ok: warm == cold-built, and the
  output surfaces `decomposition-drift` (proves `decomposed` flows) + `orphan`.
- **V-5** — `rollup_index_backed_matches_baseline` → ok: warm == cold-built, and a
  `Discovered` node lands under `### Discovered` (position-checked) — `origin` carried.
- **V-6** — `orient_index_backed_matches_baseline` → ok: warm == cold-built, the
  vision **body** appears (only via the one targeted load), focus = the arc, orphan in
  INTEGRITY.
- **V-7** — `view_consumers_reconcile_before_read` → ok: a freshly-persisted slice
  appears in `rollup`/`orient` and a fresh orphan in `check` with no manual rebuild.
- **V-8** — clippy `-D warnings` exit 0 (one `doc_lazy_continuation` fixed); no
  `unsafe`; coverage **odm-index 97.49%** / **odm-cli 93.63%** line; workspace test green.

## Decisions / deviations flagged (not buried)

- **`Decomposition` is mirrored, not reused.** Embedding `odm_core::Decomposition`
  (with its `skip_serializing_if` on `children`) would desync postcard. This is the
  same call as slice02's `EdgeRef`; flagged here because it means the index has a
  second small type duplication on purpose. The adapter round-trips it identically
  because the record's children are already sorted+deduped (so `affirm_decomposed`'s
  re-sort is a no-op).
- **"Identical to baseline" is established structurally, surfaced semantically.**
  There is no longer a `load_all` code path for `check`/`rollup`/`orient` to diff
  against literally — they are *all* index-backed now. The equivalence is guaranteed
  because every path calls the same `aggregate`/`assemble`/`integrity`; the only
  variable (the frontmatter source) is proven equivalent by the odm-index adapter
  fidelity tests. The odm-cli tests therefore assert the wired output is *correct*
  (the origin/decomposed-driven findings surface) + idempotent (warm == cold-built),
  which is the substance of the guarantee. Calibrated honestly: these are not
  byte-diffs against a parallel full-scan run.
- **Double gate-config read.** `check` reads `odm.toml` once for the adapter and
  `aggregate` reads it again internally. Kept for the *minimal seam* (aggregate stays
  self-contained); the file is tiny and the read is not on the hot path. Noted in case
  a future slice wants to thread `GateSets` through.
- **The gap was exactly `origin`+`decomposed`.** No consumer needed a third record
  field — consistent with the slice05 CDC grep. No silent additions.

## Uncertainties / things CDC should look at

- **Cargo/coverage rows are CC-`attested`.** Reproduce on CI (or a clean 1.85 floor);
  the sandbox ran 1.95.0. The coverage numbers are from `cargo llvm-cov 0.6.21` with
  the other workspace crates ignored via `--ignore-filename-regex`.
- **Worth a second look:** the `orient` project-resolution branch now matches by id
  over the index frontmatters (was: over `docs`). Confirm the stale-`ctx.project`
  fall-through still behaves (a ctx pointing at a now-deleted node → fallback). The
  existing `orient.rs` tests cover the no-project / pick-project paths and pass.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

Applied to `arc-plan.md`:

- **A-6** → `done`-on-attested: slice06 closed; carries G-3/G-4/G-5; `FORMAT_VERSION
  2 → 3`; consumer-wiring complete.
- **A-4 + A-5 close.** Their deferred seams — slice04 seam (b) graph readers + (c)
  composed views, and slice05's `check`/`rollup`/`orient` deferral — are now all
  delivered and proven. Their rows are updated to note the closure lands with slice06.
- **A-12 (arc-scale compose) is satisfiable.** Every read path — `list`, the
  derived-order readers (`next`/`blocked`/`path`), `check`, `rollup`, `orient` — is
  index-backed and matches baseline behavior. The arc-scale *demo* (forced full-scan
  vs. index, identical outputs) remains for the arc-close compose pass; the capability
  it depends on is in.

No new arc-level finding (A-N) is raised: the slice landed within the v1.10 scope and
the grep-verified two-field gap held.

## Iterations

One pass. No spec amendment needed; the slice closed within scope (the only
in-flight correction was dropping the stale `FORMAT_VERSION == 2` assertion in the
slice04 test, which the bump made false).
