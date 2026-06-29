# CC Prompt — Slice 06 (Arc 04): Enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient`

The slice05 continuation. Finish the consumer-wiring: enrich the record with the **two**
fields the composed views + `check` need (`origin`, `decomposed`), extend the adapter,
and wire `check`/`rollup`/`orient` off the index. **On close, A-4 and A-5 close.**

> **Start condition:** slice05 (adapter + derived-order readers) CDC-verified / CI-green —
> `frontmatters_from_records`, the index-backed `Derived`, and `reconcile`-then-read
> exist. If not in, hold.

## Read first
1. `slice06-views-and-check/ledger.md` (8 rows).
2. `slice-doc.md` (same dir) and `../arc-plan.md` (slice06 + Arc Ledger A-6; the split
   history v1.9/v1.10 — note the live ledger is authoritative per the bridge note).
3. The slice05 `cdc-verification.md` (the gap is grep-verified to be exactly
   `origin`+`decomposed` — no third field).
4. Reuse points: `odm-index` `adapter::frontmatters_from_records` (extend it), `build_one`,
   `Snapshot` (FORMAT_VERSION + self-heal); `odm-core` `recompose::integrity`,
   `Rollup::assemble`, the `Origin`/`Decomposition` types; `odm-cli` `check`'s `aggregate`,
   `rollup`/`orient`, the `Derived`/reconcile-then-read wrapper.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `02-api-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence at `attested`; write the
  **Bubble-up to the arc** section at close).

## Task
1. **Enrich + bump:** add `origin: Origin` + `decomposed: Option<Decomposition>` to
   `IndexRecord`; `FORMAT_VERSION 2 → 3` (a v2 index → `RebuildNeeded(VersionMismatch)` →
   cold rebuild via slice01 self-heal — no migration code). Populate in `build_one`;
   extend `meta_hash` to cover `decomposed` + `origin` (still exclude `updated`/stat +
   display fields).
2. **Extend the adapter:** `frontmatters_from_records` reconstructs `origin` +
   `decomposed` so a synthesized `Frontmatter` is faithful for `recompose::integrity` +
   `Rollup::assemble` provenance. Extend the slice05 fidelity test to cover them.
3. **`aggregate` refactor:** make `check`'s `aggregate` take `&[Frontmatter]` (adapter
   output) instead of loading the corpus itself — the minimal seam.
4. **Wire the three** (each `reconcile`-then-read): `check` (recomposition off
   `decomposed`), `rollup` (provenance off `origin`), `orient` (rollup model + check
   integrity + one targeted `store.load(project)` for the vision body).
5. **Identical-to-baseline:** each wired consumer's output equals its `load_all` output.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Reuse, not reimplement:** extend the slice05 adapter; feed the existing
  `recompose`/`Rollup`/orient; freshen via `reconcile`. Don't re-derive any of them.
- **Bodies stay out of the index** (§3.5): only `orient`'s vision triggers the one
  targeted `store.load(project)`.
- The gap is **exactly** `origin`+`decomposed` (grep-verified) — if you find a consumer
  needs a *third* record field, **stop and flag it** (it would contradict the slice05
  CDC finding), don't silently add it.
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test -p odm-index -p odm-cli` + clippy + coverage; `ledger.md` evidence per
row (at `attested`); `closing-report.md` — per-row walk **plus the v2.0 Bubble-up to the
arc** (note that **A-4 + A-5 now close** and A-12 is satisfiable; the consumer-wiring is
done). Feature branch (`arc04-slice06-views-and-check`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration cap;
your `done` is *proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local
1.85+). On close, bubble up to `arc-plan.md` (A-6) per LEDGER-DISCIPLINE v2.0 §A.
