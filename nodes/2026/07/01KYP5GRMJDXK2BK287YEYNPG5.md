---
id: 01KYP5GRMJDXK2BK287YEYNPG5
number: 517113600
type: artifact
schema: artifact/v1.1
name: 'Closing report — Arc 04 / Slice 07: Early-cutoff invalidation'
created: 2026-06-29
updated: 2026-06-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc04-index-cache/slice07-early-cutoff/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTK1FH17VB4TDJDYQQ7
---
# Closing report — Arc 04 / Slice 07: Early-cutoff invalidation

> Cash the two-fingerprint split: a **body-only edit** refreshes the index record
> but recomputes nothing downstream. Modest slice — early cutoff for a *batch CLI*
> is the persisted-artifact skip + the meta-changed signal (ODD-0014 §2.4), **not**
> lazy/persistent graph caching. Branch: `arc04-slice07-early-cutoff` (not `main`).

## What shipped

1. **Delta meta-changed vs. body-only (E-1).** `Delta` gains `meta_changed: Vec<Id>`
   (a subset of `changed`). A new `note_change(delta, prior, updated)` helper at **both**
   CHANGED arms of `reconcile` (cheap-signal and racy-hash-mismatch) pushes to `changed`
   always, and to `meta_changed` only when the freshly-built record's `meta_hash` differs
   from the prior's. Body-only = `changed` minus `meta_changed`.
2. **`Snapshot::meta_fingerprint` + `odm rollup` early-cutoff (E-2/E-3).** A new method
   hashes each record's `(id, meta_hash)` in id order into a 32-byte semantic fingerprint.
   `odm rollup` stamps it in the generated header (`fingerprint=<hex>`); a later run
   recomputes the current corpus's fingerprint from the reconciled index and, if it
   matches the one in the existing `ROLLUP.md`, **leaves the file untouched**. Any
   `meta_hash` change / new / deleted node moves the fingerprint and forces a regenerate.
3. **Record still refreshes (E-4).** A body-only edit still rebuilds the record via the
   warm path (`content_hash` + stat updated); only the *derived* `ROLLUP.md` regenerate is
   skipped — the §2.4 distinction.

## Per-row ledger walk (6 rows)

- **E-1** — `delta_distinguishes_meta_changed_from_body_only` → ok. A longer-body edit
  (same frontmatter) lands in `changed` only; a renamed node (title is a meta field) lands
  in both `changed` and `meta_changed`.
- **E-2** — `rollup_skips_on_body_only_change` → ok. After a body-only edit, the second
  `rollup` reports "unchanged … skipped" and `ROLLUP.md` is byte-identical.
- **E-3** — `rollup_regenerates_on_meta_change` → ok. Three triggers each assert "wrote" +
  changed bytes: a gate change, a new node, a deleted node. Edge/origin/decomposed changes
  ride the identical `meta_hash`→fingerprint mechanism.
- **E-4** — `body_only_edit_refreshes_record` → ok. `content_hash` differs, `meta_hash`
  unchanged; id in `changed`, not in `meta_changed`.
- **E-5** — readers unchanged: no reader source touched (the diff is `warm.rs`,
  `snapshot.rs`, `rollup.rs` + tests/docs); slice05/06 reader suites stay green.
- **E-6** — clippy `-D warnings` exit 0; no `unsafe`; coverage odm-index **95.18%** /
  odm-cli **93.68%** line; workspace test green.

## Decisions / deviations flagged (not buried)

- **Fingerprint over `(id, meta_hash)`, not just `meta_hash`.** Including the id makes the
  fingerprint robust to a delete+add that happens to produce the same `meta_hash` multiset
  — the node set is part of the semantic state. ULIDs render fixed-width (26 chars), so
  id + hash needs no separator.
- **Skip compares the *stamped* fingerprint, not reconcile bookkeeping.** Comparing the
  actual semantic state in the existing file (vs. trusting the delta) is robust to history
  — a `ROLLUP.md` deleted-and-stale, hand-edited, or pre-slice07 (no stamp) safely
  regenerates (the `None` fingerprint never matches). Flagged because it's a deliberate
  "compare state, not events" choice.
- **`rollup` now reconciles directly** (for the snapshot's records) rather than calling
  `commands::index_frontmatters` (which hides the snapshot). It still feeds the unchanged
  `frontmatters_from_records` → `Rollup::assemble`. Minor duplication of the
  reconcile-then-adapter line, taken so `rollup` can read the records' `meta_hash`es.
- **In-memory caching deliberately not built** (0014 §2.4). If slice08's benchmark shows
  the per-invocation graph rebuild dominates, that's the place to revisit — flagged there,
  not pre-empted here.

## Uncertainties / things CDC should look at

- **Cargo/coverage rows are CC-`attested`.** Reproduce on CI (or a clean 1.85 floor); the
  sandbox ran rustc 1.95.0. Coverage from `cargo llvm-cov 0.6.21` with other workspace
  crates ignored via `--ignore-filename-regex`.
- **Worth a second look:** the existing `rollup_command_regenerates_idempotently` test
  (run twice → identical bytes) now exercises the *skip* path on its second run rather
  than a re-write; it still passes (byte-identical), and the slice07 tests add the
  explicit skip/regenerate coverage.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

Applied to `arc-plan.md`:

- **A-7** → `done`-on-attested: slice07 closed (early-cutoff: delta meta-changed signal +
  `ROLLUP.md` persisted-artifact skip; in-memory readers unchanged per §2.4).
- **A-11 (self-heal compose row)** is unaffected — the fingerprint stamp lives *in*
  `ROLLUP.md` (a derived artifact), not in the index; a missing/corrupt index still
  rebuilds and the rollup regenerates.
- **Only slice08 (the 100k benchmark) remains** in Arc 04. The arc's exit criteria are all
  in except the benchmark's `[P]`→`[E]` promotion; slice08 measures the finished system
  and can settle whether deeper caching is ever warranted.

No new arc-level finding (A-N) is raised: the slice landed within scope; the
two-fingerprint foundation was already built, so this was a read-and-skip, not a redesign.

## Iterations

One pass. No spec amendment needed; the slice closed within scope.
