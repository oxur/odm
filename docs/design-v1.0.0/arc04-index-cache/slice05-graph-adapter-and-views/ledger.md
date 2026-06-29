# Slice 05 (Arc 04): Index→graph adapter + wire graph readers & composed views

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). Reproduced here on a local **1.95.0** toolchain. Five-iteration cap. The slice04
> continuation (seams b+c).
>
> **PARTIAL — A FURTHER SPLIT (flagged, not silent).** Delivered: the **adapter + the
> derived-order graph readers** (G-1, G-2, G-6, G-7). Deferred: **`check` (G-3),
> `rollup` (G-4), `orient` (G-5)** — they read frontmatter fields the index record does
> **not** carry (`origin` for rollup's provenance; `decomposed` for `check`'s
> recomposition checks). Wiring them needs a **further record enrichment** (`origin` +
> `decomposed`, `FORMAT_VERSION 3`) + a `check`-aggregate refactor — a chunk the slice
> plan's adapter-reconstruction list (id/number/type/name/edges/status) overlooked.
> Routed to a continuation via the bubble-up; see the closing report. **A-4 does not
> close yet.**

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| G-1 | An index→graph adapter builds the graph + satisfaction inputs from index records (edges+qualifiers + per-gate evidence) with **no frontmatter parse**; a graph/satisfaction built via the adapter **equals** one built from the corpus `Frontmatter`s | `cargo test -p odm-index index_graph_adapter_equals_frontmatter_graph` → ok | serious | 0014 §2.4 / slice04 finding 3 | done (attested) | `89a2223`; → 1 passed. **Shape chosen:** synthesize `Frontmatter`s from records (`adapter.rs` `frontmatters_from_records`) → feed existing `NodeGraph::build`/`Satisfaction::compute` unchanged. Fidelity asserted on ready frontier, per-node blocked (evidence-leveled), topological order, containment; corpus covers every edge kind + a gate-less type. | **The crux — delivered.** Zero odm-core change. |
| G-2 | `next` / `blocked` / `path` read the index-backed graph (`reconcile`-then-read); output identical to the `load_all` baseline | `cargo test -p odm-cli derived_order_index_backed_match_baseline` → ok | serious | 0013 §4.1 / arc-plan A-11 | done (attested) | `89a2223`; → 1 passed (`next` lists the ready node not the blocked one; `blocked` names the unsatisfied dep; `path` is the chain). `Derived::load` is index-backed; the three readers ride it. Identity rests on G-1 (graph == baseline graph). | |
| G-3 | `check` reads the index-backed graph; the evidence-leveled satisfaction path is reproduced off the index; output identical to baseline | `cargo test -p odm-cli check_index_backed_matches_baseline` → ok | serious | 0013 §4.4 / arc-plan A-11 | deferred | | **Continuation.** `check`'s recomposition (`recompose::integrity`) reads `fm.decomposed()` — **not in the record.** Re-entry: enrich the record with `decomposed` (`FORMAT_VERSION 3`) + refactor `aggregate` to take `&[Frontmatter]`, then feed synth frontmatters. (Evidence-leveled satisfaction itself is already adapter-faithful — G-1.) |
| G-4 | `rollup` composes over the index-backed model; output identical to baseline | `cargo test -p odm-cli rollup_index_backed_matches_baseline` → ok | serious | 0013 §6 / arc-plan A-11 | deferred | | **Continuation.** `Rollup::assemble`'s provenance view reads `fm.origin()` — **not in the record.** Re-entry: enrich the record with `origin`, then source-swap `rollup` to reconcile→adapter→`Rollup::assemble`. |
| G-5 | `orient` composes over the index-backed model **and** loads only the current project's `Document` for the vision body; output identical to baseline | `cargo test -p odm-cli orient_index_backed_matches_baseline` → ok | serious | 0013 §4.1 / 0014 §3.5 | deferred | | **Continuation.** Composes the rollup model (needs `origin`, G-4) **and** the `check` integrity findings (needs `decomposed`, G-3) — gated on both enrichments. The targeted `store.load(project)` for the vision body lands here. |
| G-6 | All wired consumers `reconcile` (warm path) before reading | `cargo test -p odm-cli graph_consumers_reconcile_before_read` → ok | serious | slice03 finding #2 | done (attested) | `89a2223`; → 1 passed (a node added after the first `next` appears on the next `next`, no manual rebuild). For the **wired** consumers (the graph readers + `list` from slice04). The composed/check consumers inherit the same wrapper when wired (G-3/G-4/G-5). | |
| G-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-index + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' …` AND `cargo llvm-cov --summary-only …` per crate → **line** ≥ 90% | serious | CLAUDE.md | done (attested) | `89a2223`; clippy → exit 0; `unsafe` grep → no matches; line **odm-index 98.19%** (adapter.rs 95.83%) **/ odm-cli 94.22%**; `fmt --check` clean; full workspace `cargo test` → 266 passed. | Applies to the delivered scope. |

## What Worked

- **Synthesizing `Frontmatter`s from records was the right seam** (G-1): the inverse
  of slice02's `map_edges` + a status rebuild, feeding the *unchanged* graph and
  satisfaction engines. Zero odm-core change; the fidelity test proves graph ==
  baseline graph (ready/blocked/topo/containment) directly.
- **Index-backing `Derived` once** flipped `next`/`blocked`/`path` together — they all
  ride one `load`. Every existing derived-order test stayed green (identical behaviour).
- **The G-1 corpus exercising every edge kind** caught the adapter's `edges_from_record`
  coverage to 95.83% and proved the qualifier round-trip (satisfied_at / supersede-kind
  / tear-because) survives record → adapter.
- **Recognising the second split early** — `rollup`/`check` read `origin`/`decomposed`,
  which the record lacks — kept the adapter + readers clean rather than half-wiring the
  composed views against fields that aren't there.

## Closure

Closed (partial) at commit `89a2223` on 2026-06-29. CDC verification: pending (cargo
rows via CI / local 1.85+ — `attested → reproduced`). Rows: 7. **Done: 4** (G-1, G-2,
G-6, G-7). **Deferred: 3** (G-3, G-4, G-5 → continuation; re-entry = enrich the record
with `origin` + `decomposed`, `FORMAT_VERSION 3`, + the `aggregate` refactor). No-op: 0.
**A-4 stays open** (seam b graph-half delivered; check + composed views carried forward).
On close, CC bubbles up to `arc-plan.md` (A-5) per LEDGER-DISCIPLINE v2.0 §A, requesting
the operator scope the continuation.
