# Slice 06 (Arc 04) — Enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient` (plan-of-record)

> The **slice05 continuation** (the deferred G-3/G-4/G-5 — see arc-plan v1.9/v1.10).
> Refs: ODD-0013 §4.5 (recomposition / `decomposed`), §6 (rollup + provenance/`origin`),
> §4.1 (orient); ODD-0014 §3.5 (structured metadata in, body out); arc04 `arc-plan.md`
> slice06 + Arc Ledger A-6 (closes A-4 + A-5, satisfies A-12). `depends_on:` slice05
> (the adapter + the index-backed `Derived`/reconcile-then-read), slice04 (the enriched
> record + maps).
>
> **Why this slice exists:** slice05 wired the graph readers but `check`/`rollup`/`orient`
> read two frontmatter fields the record doesn't carry — `decomposed` (check's
> recomposition) and `origin` (rollup's provenance view). This slice enriches the record
> with **exactly those two** (grep-verified as the complete remaining gap — no third
> field), refactors `check`'s aggregate to accept the adapter's output, and wires the
> three. **On close it finishes the consumer-wiring: A-4 and A-5 close.**

## Goal

The composed views + `check` read the index. **Done when** `IndexRecord` carries
`origin` + `decomposed` (`FORMAT_VERSION 2 → 3`, old index self-heals), the adapter
reconstructs them so a synthesized `Frontmatter` is faithful for recomposition +
provenance, and `check`/`rollup`/`orient` read the index (`reconcile`-then-read) with
**output identical to the full-scan baseline** — `orient` loading only the current
project's `Document` for the vision body.

## Scope

**In:**

- **Record enrichment + format bump.** Add `origin: Origin` and `decomposed:
  Option<Decomposition>` to `IndexRecord`. `FORMAT_VERSION 2 → 3` — an on-disk v2 index →
  `RebuildNeeded(VersionMismatch)` → cold rebuild (slice01 self-heal; no migration code).
  Populate in `build_one` (cold + warm). `meta_hash` covers **`decomposed`** (graph-
  semantic — recomposition findings) and **`origin`** (the provenance view); still
  excludes `updated`/stat + display fields.
- **Extend the adapter.** `frontmatters_from_records` reconstructs `origin` + `decomposed`
  too, so a synthesized `Frontmatter` is faithful for `recompose::integrity` (check) and
  `Rollup::assemble`'s provenance view (the slice05 G-1 fidelity test extends to cover
  them).
- **`aggregate` refactor.** Refactor `check`'s `aggregate` to take `&[Frontmatter]` (the
  adapter output) rather than loading the corpus itself — the minimal seam so `check`
  rides the index-backed graph.
- **Wire the three** (each `reconcile`-then-read): `check` (incl. recomposition off
  `decomposed`), `rollup` (provenance off `origin`), `orient` (composes the rollup model
  + check integrity, **and** loads only the current project's `Document` for the vision
  body — one targeted load, §3.5).
- **Identical-to-baseline.** Each wired consumer's output equals its `load_all` output,
  asserted by test.

**Out:** early-cutoff *downstream* invalidation — slice07; the benchmark — slice08;
`reserved`/`retired`/`desired_facts` (no current consumer reads them; if A5 reconcile
later needs `desired_facts` in the index, that's an A5 concern).

## Design notes (settle here)

- **The gap is closed, not chased.** `origin` + `decomposed` is the *complete* remaining
  set (grep-verified across `rollup`/`recompose`/`orient` in slice05 CDC). After this,
  the record is "the full frontmatter projection **minus the body**" — §3.5-consistent.
- **Reuse, not reimplement:** extend the slice05 adapter; feed the existing
  `recompose::integrity` / `Rollup::assemble` / orient; freshen via `reconcile`. The
  enrichment + the `aggregate` signature change are the only substantial edits.
- **`FORMAT_VERSION 3` is free** (slice01 self-heal routes a stale v2 index to a cold
  rebuild) — same as the v1→v2 bump in slice04.

## Verification

`cargo test -p odm-index -p odm-cli` green (enrich + v3 self-heal; adapter reconstructs
origin/decomposed; `check`/`rollup`/`orient` index-backed == baseline; orient targeted
body load; reconcile-then-read); clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line)
for odm-index + odm-cli. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+). **All consumers read
the index — A-4 and A-5 close; the arc's A-12 (consumers-read-the-index) compose row is
satisfiable.** slice07 layers early-cutoff; slice08 benchmarks. Bubble up to
`arc-plan.md` (A-6) at close.
