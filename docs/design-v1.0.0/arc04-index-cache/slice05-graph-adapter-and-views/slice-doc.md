# Slice 05 (Arc 04) — Index→graph adapter + wire graph readers & composed views (plan-of-record)

> The **slice04 continuation** (seams b+c, deferred from slice04 — see arc-plan v1.7/v1.8).
> Refs: ODD-0014 §2.4 (the index feeds the graph build), §3.5 (bodies stay out); ODD-0013
> §4.1/§4.4 (derived-order + evidence-leveled satisfaction), §6 (rollup); arc04
> `arc-plan.md` slice05 + Arc Ledger A-5 (closes A-4, strengthens A-11). `depends_on:`
> slice04 (the enriched record — per-gate evidence + `EdgeRef` qualifiers — and the
> `reconcile` freshen path), slices 01–03 (the index).
>
> **Why this slice exists:** slice04 made the index *carry* what the graph needs and wired
> `list`. This slice wires the **graph commands and composed views** — the heavy
> consumers — so they read the index instead of full-scanning the corpus. The crux is the
> **index→graph adapter**: reconstruct what `NodeGraph::build` / `Satisfaction::compute`
> consume from index records, with no frontmatter parse.

## Goal

The graph commands and composed views read the index. **Done when** an index→graph
adapter feeds the DAG + evidence-leveled satisfaction from index records (no frontmatter
parse); `next`/`blocked`/`path`/`check` and `rollup`/`orient` read the index (via
`reconcile`-then-read) with **output identical to the full-scan baseline**; and `orient`
loads only the current project's `Document` for the vision body.

## Scope

**In:**

- **Index→graph adapter.** `NodeGraph::build` and `Satisfaction::compute` both take
  `&[Frontmatter]`, and `compute` needs evidence **levels** (which the record now
  carries — no dates needed). So the adapter **reconstructs `Frontmatter`s from index
  records** (id, number, type, name, edges from `EdgeRef`+qualifiers, status from
  `gates`+evidence) and feeds the *existing* `build`/`compute` **unchanged** — no
  re-derivation of graph/satisfaction logic. (Design note below: synthesize vs. an
  index-native constructor.)
- **Wire the graph readers** (each `reconcile`-then-read): `next`, `blocked`, `path`,
  `check` — `check` includes the evidence-leveled satisfaction path (min-propagation,
  soft-satisfaction) reproduced off the index.
- **Wire the composed views** (each `reconcile`-then-read): `rollup`, `orient` — over
  the index-backed model; `orient` additionally loads **only the current project's
  `Document`** for the vision body (one targeted load — the index carries no bodies,
  §3.5).
- **Identical-to-baseline.** Each wired consumer's output equals its `load_all` output,
  asserted by test (the index is an accelerator, not a semantic change).

**Out:** early-cutoff *downstream* invalidation (using `meta_hash`/the delta) — slice06;
the benchmark — slice07; `list` (done in slice04); rkyv/mmap/sharding/watcher — deferred.

## Design notes (settle here)

- **Adapter shape (open):** **synthesize `Frontmatter`s from records** (CC's slice04
  sketch — zero change to odm-core, the smaller seam; *recommended*) **vs.** give
  `NodeGraph`/`Satisfaction` index-native constructors. Pick the smaller, clearer seam;
  the fidelity test (G-1: adapter graph == frontmatter graph) is the guard either way.
- **Reuse, not reimplement:** feed the existing `NodeGraph::build` / `Satisfaction::
  compute`; compose over the existing `Rollup`/orient models; freshen via slice03's
  `reconcile`. The adapter is the only substantial new code.
- **Bodies stay out of the index** (§3.5): `orient`'s vision is the one targeted
  `store.load(project)`.

## Verification

`cargo test -p odm-index -p odm-cli` green (adapter graph == frontmatter graph; each
graph reader + composed view index-backed == baseline; orient targeted body load;
reconcile-then-read freshness); clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line)
for odm-index + odm-cli. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+). All consumers read
the index; the graph commands compute satisfaction off the enriched record. **On close,
slice04's A-4 closes (seam a+b+c delivered) and the arc's A-11 compose row is satisfiable.**
slice06 can layer early-cutoff; slice07 benchmarks. Bubble up to `arc-plan.md` (A-5).
