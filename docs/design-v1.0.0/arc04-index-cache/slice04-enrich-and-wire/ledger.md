# Slice 04 (Arc 04): Enrich record + wire consumers

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). Reproduced here on a local **1.95.0** toolchain. Five-iteration cap.
>
> **SPLIT AT THE NAMED SEAM (per the cc-prompt's size guidance).** This slice is large;
> it is delivered as **seam (a) = enrich + maps + `list`** (rows I-1…I-5, the `list`
> part of I-9, I-10). The remaining seams — **(b) index→graph adapter + graph readers**
> (I-6, I-7) and **(c) composed views** (I-8) plus the graph/composed part of I-9 — are
> **deferred to a renumbered continuation slice**, routed via the bubble-up (the prompt
> said: don't invent a `04b` name; flag for proper renumber). See the closing report.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| I-1 | `IndexRecord.gates` carries per-gate **evidence** (gate name + `Evidence` level), not just names | `cargo test -p odm-index gates_carry_evidence` → ok | serious | arc-plan v1.6 / 0014 §3.1 | done (attested) | `2dafaa1`; `gates_carry_evidence_and_build_one_evidence` → 1 passed. New `GateState { gate, evidence }`; record `gates: Vec<GateState>`. | No reached-dates (A7). |
| I-2 | `FORMAT_VERSION` is `2`; an on-disk v1 index loads as `RebuildNeeded(VersionMismatch)` → cold rebuild (reuse slice01 self-heal; no migration code) | `cargo test -p odm-index v1_index_triggers_rebuild` → ok AND `grep -n 'FORMAT_VERSION: u16 = 2' crates/odm-index/src/snapshot.rs` | serious | arc-plan v1.6 / slice02 watch-item | done (attested) | `2dafaa1`; `v1_index_triggers_rebuild` → 1 passed (forged v1 file → `VersionMismatch{found:1}` → `reconcile` rebuilds cold); grep → `snapshot.rs:40`. | Discharges the slice02 FORMAT_VERSION freeze. |
| I-3 | `build_one` populates per-gate evidence (cold + warm); `meta_hash` now covers gate evidence and still excludes `updated`/stat | `cargo test -p odm-index build_one_evidence` + `meta_hash_tracks_evidence` → ok | serious | slice02 B-4/B-6 | done (attested) | `2dafaa1`; both → 2 passed (`build_one` sets evidence; raising a gate's evidence flips `meta_hash`). | meta_hash also excludes the new `number`/`component` (display/filter, not graph-meaning) — flagged. |
| I-4 | In-memory maps build on load: `type→ids`, `tag→ids`, `gate→ids`, edge adjacency; no disk after load, no FTS | `cargo test -p odm-index inmemory_maps_built` → ok | serious | 0014 §3.5 | done (attested) | `2dafaa1`; `inmemory_maps_built` → 1 passed (type/tag/gate lookups + forward edge adjacency). `maps.rs` `IndexMaps::build`. | maps.rs 100% line. |
| I-5 | `list` reads the index (filter by type/tag/gate/component); output identical to the `load_all` baseline | `cargo test -p odm-cli list_index_backed_matches_baseline` → ok | serious | arc-plan / 0014 §3.5 | done (attested) | `2dafaa1`; `list_index_backed_matches_baseline` → 1 passed (warm read == cold-built; type filter narrows; number order). Existing `cli.rs` list tests still green. | **Scope decision:** the human **table + filters** are index-backed (§3.5); `list --json` (full-node dump with `origin`/`reserved`/`retired`) stays `load_all` — the index is the filter/sort accelerator, not a full-node store. Flagged. |
| I-6 | An index→graph adapter builds the `NodeGraph` + `Satisfaction` inputs from index records (edges+qualifiers+evidence) — **no frontmatter parse**; the graph equals one built from frontmatters | `cargo test -p odm-index index_graph_adapter_equals_frontmatter_graph` → ok | serious | 0014 §2.4 | deferred | | **Seam (b) — continuation.** Re-entry: a renumbered continuation slice. Design sketched (synthesize `Frontmatter`s from records → feed existing `NodeGraph::build`/`Satisfaction::compute`); see closing report. The record now carries the evidence the adapter needs (I-1/I-3 done). |
| I-7 | `next` / `blocked` / `path` / `check` read the index-backed graph; output identical to baseline | `cargo test -p odm-cli graph_readers_index_backed_match_baseline` → ok | serious | 0013 §4.1 / arc-plan A-10 | deferred | | **Seam (b) — continuation.** Re-entry: after I-6; renumbered continuation. |
| I-8 | `rollup` / `orient` compose over the index-backed model; `orient` loads only the current project's `Document`; output identical to baseline | `cargo test -p odm-cli rollup_orient_index_backed_match_baseline` → ok | serious | 0013 §6/§4.1 / arc-plan A-10 | deferred | | **Seam (c) — continuation.** Re-entry: after I-6/I-7; renumbered continuation. |
| I-9 | Consumers `reconcile` (warm path) before reading, so a stale index is freshened first | `cargo test -p odm-cli consumers_reconcile_before_read` → ok | serious | slice03 finding #2 | done (attested) | `2dafaa1`; `consumers_reconcile_before_read` → 1 passed (a node added after the first `list` appears on the next `list`, no manual rebuild). | **For `list` (seam a).** The same reconcile-then-read wraps the graph/composed consumers in the continuation (b/c). |
| I-10 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-index + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | done (attested) | `2dafaa1`; clippy → exit 0; `unsafe` grep → no matches; line **odm-index 98.53% / odm-cli 93.82%**; `fmt --check` clean; full workspace `cargo test` → 263 passed. | Applies to the seam-(a) scope delivered. |

## What Worked

- **The format bump was as cheap as the slice02 watch-item promised.** Bumping
  `FORMAT_VERSION → 2` plus slice01's existing `RebuildNeeded(VersionMismatch)`
  self-heal = old indexes rebuild cold, zero migration code. The forge-a-v1-file
  test exercises it end-to-end.
- **`GateState` was a small, contained enrichment** — `reached()` already yields
  `(name, &GateRecord)`, so `build_one` just maps `record.evidence` in.
- **Filtering the reconciled records directly for `list`** kept "identical to
  baseline" trivially true; `IndexMaps` is delivered + tested as the reusable
  filter primitive the graph adapter (seam b) and hot-path filters will lean on.
- **Recognising the split early** (the `list --json` full-node fork + the size of
  the adapter) kept seam (a) clean rather than half-landing (b)/(c) under budget.

## Closure

Closed (seam a) at commit `2dafaa1` on 2026-06-29. CDC verification: pending (cargo
rows via CI / local 1.85+ — `attested → reproduced`). Rows: 10. **Done: 7** (I-1…I-5,
I-9, I-10). **Deferred: 3** (I-6, I-7, I-8 → renumbered continuation, seams b+c). No-op:
0. On close, CC bubbles up to `arc-plan.md` (A-4) per LEDGER-DISCIPLINE v2.0 §A,
**requesting the operator renumber the continuation** (do not self-name `04b`).
