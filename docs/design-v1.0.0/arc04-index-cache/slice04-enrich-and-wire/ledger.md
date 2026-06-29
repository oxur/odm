# Slice 04 (Arc 04): Enrich record + wire consumers

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. **Large slice** — split
> seam (enrich+maps+list | adapter+readers | composed views) if it won't hold one context.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| I-1 | `IndexRecord.gates` carries per-gate **evidence** (gate name + `Evidence` level), not just names | `cargo test -p odm-index gates_carry_evidence` → ok | serious | arc-plan v1.6 / 0014 §3.1 | open | | The minimum satisfaction needs; no reached-dates (A7). |
| I-2 | `FORMAT_VERSION` is `2`; an on-disk v1 index loads as `RebuildNeeded(VersionMismatch)` → cold rebuild (reuse slice01 self-heal; no migration code) | `cargo test -p odm-index v1_index_triggers_rebuild` → ok AND `grep -n 'FORMAT_VERSION: u16 = 2' crates/odm-index/src/snapshot.rs` | serious | arc-plan v1.6 / slice02 watch-item | open | | Discharges the slice02 FORMAT_VERSION freeze. |
| I-3 | `build_one` populates per-gate evidence (cold + warm); `meta_hash` now covers gate evidence (an evidence change invalidates downstream) and still excludes `updated`/stat | `cargo test -p odm-index build_one_evidence` + `meta_hash_tracks_evidence` → ok | serious | slice02 B-4/B-6 | open | | Evidence is semantic for the graph ⇒ correctly in meta_hash. |
| I-4 | In-memory maps build on load from the records: `type→ids`, `tag→ids`, `gate→ids`, edge adjacency; no disk after load, no FTS | `cargo test -p odm-index inmemory_maps_built` → ok | serious | 0014 §3.5 | open | | Built once on load; a few MB at 10k–100k. |
| I-5 | `list` reads the index maps (filter by type/tag/gate/component); output identical to the `load_all` baseline | `cargo test -p odm-cli list_index_backed_matches_baseline` → ok | serious | arc-plan / 0014 §3.5 | open | | "by gate" = by reached gate (slice01 finding #1). |
| I-6 | An index→graph adapter builds the `NodeGraph` + `Satisfaction` inputs from index records (edges+qualifiers+evidence) — **no frontmatter parse**; the graph equals one built from frontmatters | `cargo test -p odm-index index_graph_adapter_equals_frontmatter_graph` → ok | serious | 0014 §2.4 | open | | **The crux.** Feeds existing odm-core graph/satisfaction; does not re-derive them. |
| I-7 | `next` / `blocked` / `path` / `check` read the index-backed graph; output identical to the `load_all` baseline | `cargo test -p odm-cli graph_readers_index_backed_match_baseline` → ok | serious | 0013 §4.1 / arc-plan A-10 | open | | Evidence-leveled satisfaction (min-prop, soft-sat) reproduced off the index. |
| I-8 | `rollup` / `orient` compose over the index-backed model; `orient` loads only the current project's `Document` for the vision body (one targeted load, not a walk); output identical to baseline | `cargo test -p odm-cli rollup_orient_index_backed_match_baseline` → ok | serious | 0013 §6/§4.1 / arc-plan A-10 | open | | Bodies stay out of the index (0014 §3.5) — one `store.load(project)`. |
| I-9 | Consumers `reconcile` (warm path) before reading, so a stale index is freshened first; an edit is reflected without a manual rebuild | `cargo test -p odm-cli consumers_reconcile_before_read` → ok | serious | slice03 finding #2 | open | | reconcile-then-read across the wired commands. |
| I-10 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-index + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Rows: 10.)_
