# Slice 06 (Arc 04): Enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient`

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. The slice05
> continuation (G-3/G-4/G-5); finishes the arc's consumer-wiring.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| V-1 | `IndexRecord` carries `origin` + `decomposed`; `FORMAT_VERSION 2 → 3`; an on-disk v2 index loads as `RebuildNeeded(VersionMismatch)` → cold rebuild (slice01 self-heal; no migration code) | `cargo test -p odm-index record_carries_origin_decomposed` + `v2_index_triggers_rebuild` → ok AND `grep -n 'FORMAT_VERSION: u16 = 3'` | serious | arc-plan v1.10 / slice05 finding | done | attested: both tests → ok (`enrich.rs`); `grep` → `snapshot.rs:41`. `Decomposition` is an **index-owned mirror** of `odm_core`'s (its `children` carries `skip_serializing_if`, which would desync postcard — the slice02 lesson). v2 self-heals via slice01's load path; the stale slice04 `v1_index_triggers_rebuild` had its `FORMAT_VERSION == 2` guard dropped (now carried by `v2_…`). | The complete remaining gap (grep-verified: only these two). |
| V-2 | `build_one` populates `origin` + `decomposed` (cold + warm); `meta_hash` covers `decomposed` (recomposition) and `origin` (provenance) and still excludes `updated`/stat/display fields | `cargo test -p odm-index build_one_origin_decomposed` + `meta_hash_tracks_decomposed_and_origin` → ok | serious | slice02 B-4/B-6 | done | attested: both tests → ok. `MetaInput` gained `origin` + `decomposed` (canonical builder updated); `meta_hash_tracks_…` flips the hash on an origin change AND on a decomposition change, and the existing `meta_hash_tracks_evidence` still excludes `updated`/stat/display. | Both are semantic (graph recomposition / provenance view) ⇒ correctly in meta_hash. |
| V-3 | The adapter (`frontmatters_from_records`) reconstructs `origin` + `decomposed`, so a synthesized `Frontmatter` is faithful for `recompose::integrity` + `Rollup::assemble`'s provenance | `cargo test -p odm-index adapter_reconstructs_origin_decomposed` → ok | serious | slice05 G-1 / 0014 §2.4 | done | attested: test → ok. Corpus has varied origins (Planned/Discovered/Amendment) + an engineered `DecompositionDrift`; asserts `Rollup::assemble(real).provenance == assemble(synth).provenance` **and** `recompose::integrity(real) == integrity(synth)` (a *positive* drift, not empty==empty). The slice05 `index_graph_adapter_equals_frontmatter_graph` (graph == baseline) still passes. | Extends the slice05 fidelity test (graph == baseline) to recomposition + provenance. |
| V-4 | `check`'s `aggregate` refactored to take `&[Frontmatter]`; `check` reads the index-backed graph + recomposition (`decomposed`); output identical to baseline (incl. orphan / undeveloped-stub / decomposition-drift findings + severities) | `cargo test -p odm-cli check_index_backed_matches_baseline` → ok | serious | 0013 §4.5 / arc-plan A-12 | done | attested: test → ok. `aggregate` now takes `&[Frontmatter]` (drops its own `load_all`→clone); `check` feeds it `index_frontmatters`. Test asserts warm == cold-built AND surfaces `decomposition-drift` (proves `decomposed` flows) + `orphan` (recomposition over the index graph). | The recomposition path is what needed `decomposed`. |
| V-5 | `rollup` composes over the index-backed model; the provenance view (`origin`) matches; output identical to baseline | `cargo test -p odm-cli rollup_index_backed_matches_baseline` → ok | serious | 0013 §6 / arc-plan A-12 | done | attested: test → ok. `rollup` builds `index_frontmatters` then the unchanged `Rollup::assemble`. Test asserts warm == cold-built AND a `Discovered`-origin node lands under `### Discovered` (position-checked between the Discovered and Amendment headers) — proves `origin` is carried, not defaulted. | Provenance is what needed `origin`. |
| V-6 | `orient` composes over the index-backed model (rollup + check integrity) **and** loads only the current project's `Document` for the vision body; output identical to baseline | `cargo test -p odm-cli orient_index_backed_matches_baseline` → ok | serious | 0013 §4.1 / 0014 §3.5 | done | attested: test → ok. `orient` resolves the project id off the index frontmatters, does **one** `store.load(project_id)` for the body, then composes off the index. Test asserts warm == cold-built, the vision **body** appears (only obtainable via the targeted load — bodies aren't in the index), current focus = the arc, and `orphan` surfaces in INTEGRITY. | One targeted `store.load(project)` — bodies stay out of the index. |
| V-7 | All three (`check`/`rollup`/`orient`) `reconcile` (warm path) before reading | `cargo test -p odm-cli view_consumers_reconcile_before_read` → ok | serious | slice03 finding #2 | done | attested: test → ok. After warming the index, a freshly-`persist`ed slice appears in `rollup` + `orient` and a fresh orphan appears in `check`, with **no** manual rebuild — the `reconcile` inside `index_frontmatters` (shared with `list`/derived-order) freshens first. | Same reconcile-then-read wrapper as `list`/derived-order. |
| V-8 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-index + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | done | attested: `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 (fixed one `doc_lazy_continuation` on the `index_frontmatters` doc); `grep` for `unsafe` → none; `cargo llvm-cov --summary-only` per crate (other crates ignored) → **odm-index line 97.49%**, **odm-cli line 93.63%**. Full `cargo test --workspace` green. | |

## What Worked

- **Mirror, don't embed (the slice02 lesson, applied prophylactically).** `odm_core`'s
  `Decomposition.children` carries `skip_serializing_if`, which silently desyncs the
  non-self-describing postcard stream. The index owns a parallel `Decomposition` with
  always-serialized fields — exactly the call slice02 had to make for `EdgeRef`.
- **One seam, three consumers.** Refactoring `aggregate` to take `&[Frontmatter]` and
  adding the shared `index_frontmatters(store, gates)` helper let `check`/`rollup`/
  `orient` all switch to reconcile-then-read with no re-derivation — they feed the
  *unchanged* `Rollup::assemble` / `recompose::integrity` / `NodeGraph`. `Derived::load`
  (the graph readers' wrapper) folded onto the same helper.
- **Fidelity proven at the lowest layer, surfaced at the highest.** The "identical to
  baseline" guarantee is structural: both paths call the *same* `aggregate`/`assemble`/
  `integrity`; the only variable is the frontmatter source, and the odm-index adapter
  tests (`index_graph_adapter_equals_frontmatter_graph`, `adapter_reconstructs_origin_decomposed`)
  prove the synthesized frontmatters equal the parsed ones for exactly these consumers.
  The odm-cli tests then assert the wired output surfaces the origin/decomposed-driven
  findings (drift, provenance grouping) end-to-end.
- **Bodies stayed out of the index.** `orient` is the only consumer needing a body; it
  resolves the project id off the index and does a single targeted `store.load` (§3.5).

## Closure

All 8 rows `done` at **attested** (CC); cargo/coverage rows reproduce via CI / a local
1.85+ toolchain (the sandbox runs 1.95.0, above the floor). Coverage: odm-index **97.49%**
line, odm-cli **93.63%** line. No `unsafe`; clippy `-D warnings` clean; full workspace
`cargo test` green.

**On close:** this slice finishes Arc 04's consumer-wiring (G-3/G-4/G-5 carried from
slice05). **A-4 + A-5 close** (their deferred seams (b)+(c) are now delivered), and the
arc-scale compose row **A-12** is satisfiable (every read path — `list`/derived-order/
`check`/`rollup`/`orient` — is index-backed and matches baseline). Bubbled up to
`arc-plan.md` A-6 (+ A-4/A-5 notes) per LEDGER-DISCIPLINE v2.0 §A.
