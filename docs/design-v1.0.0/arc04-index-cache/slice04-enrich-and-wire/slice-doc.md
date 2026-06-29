# Slice 04 (Arc 04) — Enrich record + wire consumers (plan-of-record)

> Refs: ODD-0014 §3.5 (in-memory filter/sort, no FTS; bodies stay out), §2.4 (the index
> feeds the graph build), §3.1 (record fields); arc04 `arc-plan.md` slice04 (v1.6) + Arc
> Ledger A-4 / A-10. `depends_on:` slice01 (`Snapshot`/`Load` + the version-mismatch
> self-heal), slice02 (`build_one`, `meta_hash`, `EdgeRef` qualifiers), slice03
> (`reconcile`). Discharges the slice02 `FORMAT_VERSION` freeze watch-item.
>
> **Why this slice exists:** A4's point is to stop the consumers re-walking + re-parsing
> the whole corpus. slices 01–03 built and maintain the index, but **nothing reads it
> yet**. This slice makes `list`, the graph readers, and the composed views read the
> index — which first requires the record to carry what they need (per-gate **evidence**
> for satisfaction; the index keeps no bodies, so `orient` does one targeted body load).
>
> **Size note (large slice).** If it exceeds one context, the pre-named split seam is:
> (a) enrich + maps + `list` · (b) index→graph adapter + readers (`next`/`blocked`/
> `path`/`check`) · (c) composed views (`rollup`/`orient`). A continuation routes via
> the bubble-up; do not improvise a `04b` bisection name.

## Goal

The index becomes the read path. **Done when** `IndexRecord.gates` carries per-gate
evidence (`FORMAT_VERSION → 2`, old index self-heals to a rebuild); in-memory type/tag/
gate/edge maps build on load; an index→graph adapter feeds the DAG + evidence-leveled
satisfaction from index records; and `list`, `next`/`blocked`/`path`/`check`, and
`rollup`/`orient` read the index (via `reconcile`-then-read) with **output identical to
the full-scan baseline** — `orient` loading only the current project's body for vision.

## Scope

**In:**

- **Record enrichment + format bump.** `gates: Vec<String>` (reached names) →
  per-gate **evidence** (gate name + `Evidence` level) — the minimum the satisfaction
  model needs (no reached-dates; those are A7 telemetry). `FORMAT_VERSION 1 → 2`; an
  on-disk v1 index → `RebuildNeeded(VersionMismatch)` → cold rebuild (reuse slice01's
  self-heal — already built). Populate in `build_one` (cold + warm get it). `meta_hash`
  now covers the gate **evidence** too (an evidence change invalidates downstream;
  still excludes `updated`/stat).
- **In-memory maps** (0014 §3.5), built on load from the records: `type → ids`,
  `tag → ids`, `gate → ids`, and edge adjacency. No disk after load; no FTS.
- **Index→graph adapter.** Build the `NodeGraph` + `Satisfaction` inputs from index
  records (edges + qualifiers + per-gate evidence) — equivalent to building them from
  frontmatters, with **no frontmatter parse**.
- **Wire the consumers** (each via `reconcile`-then-read, so a stale index is freshened
  first — slice03 finding #2):
  - `list` → the in-memory maps (filter by type/tag/gate/component);
  - `next` / `blocked` / `path` / `check` → the index-backed graph;
  - `rollup` / `orient` → the index-backed model; `orient` additionally loads **only the
    current project's `Document`** for the vision body (one targeted load, not a walk).
- **Identical-to-baseline.** Each wired consumer's output equals its full-scan
  (`load_all`) output, asserted by test.

**Out:** early-cutoff *downstream* invalidation (using `meta_hash`/the delta to skip
recompute) — slice05; the benchmark — slice06; rkyv/mmap/sharding/watcher — deferred.

## Design notes (settle here)

- **Reuse, not reimplement:** `build_one` (populate), `reconcile` (freshen), the
  `Rollup`/`orient` models (compose) — the adapter feeds existing odm-core graph/
  satisfaction, it does not re-derive them.
- **The format bump is cheap by design:** slice01's `RebuildNeeded(VersionMismatch)` →
  cold rebuild already handles a stale-format index; bumping to 2 just routes old
  indexes through it. No migration code.
- **Bodies stay out of the index** (0014 §3.5): `orient`'s vision is the one place a
  body is needed, handled by a single targeted `store.load(project_id)` — not by
  carrying bodies in the record.

## Verification

`cargo test -p odm-index -p odm-cli` green (enrich + format-bump self-heal; maps;
adapter graph == frontmatter graph; each consumer index-backed == baseline; orient
targeted body load; reconcile-then-read freshness); clippy `-D warnings`; no `unsafe`;
coverage ≥ 90% (line) for odm-index + odm-cli. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+). The consumers read
the index instead of full-scanning the corpus; the graph commands compute satisfaction
off the enriched record. slice05 can layer early-cutoff on the delta. Bubble up to
`arc-plan.md` (A-4; strengthens A-10) at close.
