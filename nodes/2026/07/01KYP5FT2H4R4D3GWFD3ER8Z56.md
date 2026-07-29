---
id: 01KYP5FT2H4R4D3GWFD3ER8Z56
number: 594194400
type: artifact
schema: artifact/v1.1
name: Slice 01 closing report — Coverage discovery
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice01-coverage-discovery/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SABSXWZ58M8TQK3G3
---
# Slice 01 closing report — Coverage discovery

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 01 · **Feeds:** MF-1, MF-5
> **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (F-1…F-9) · **Implemented by:** CC
> **Date:** 2026-07-27 · **Branch:** `arc-migfidelity-slice01-coverage` (off `release/1.0.x`
> @ `14f098b`) · **Evidence class:** attested-by-CC (local 1.85+); cargo rows reproduce on CI/CDC.

## What shipped

A **read-only coverage/gap detector**: a `coverage` module in `odm-migrate`
(`crates/odm-migrate/src/coverage.rs`) plus a read-only `odm migrate <docs-root>
--coverage` reporting mode in `odm-cli` (sibling to `--dry-run`), run once over
odm's own `1.0.x/docs` corpus (326 files) against the live 60-node store, producing
**`coverage-report.md`** — the exact, re-runnable, four-dimension inventory this
arc's remaining slices (s02–s07) plan against instead of the audit's estimates.

Four detectors, each a count plus the offending list, every finding carrying its
matching basis:

1. **doc-coverage** — every source `.md` matched to a node, or reported uncovered.
   Structural coordinates (arc/slice directory numbering, reusing
   `selfhost::parse_prefix`/`arc_number`/`slice_number`) for the planning corpus;
   frontmatter `number` (reusing `legacy::parse_file`) for ODDs.
2. **representation** — arc/slice **directories** vs. arc/slice **nodes**, missing
   units named. Derived from the same classified doc list the doc-coverage
   detector builds (not a second, independent filesystem walk), so it makes no
   assumption about where the plan-set root sits.
3. **stub-body** — work nodes whose body is ≤ 1 non-blank line (tombstones
   excluded).
4. **provenance-absence** — nodes carrying no `provenance:` key, checked against
   the node's emitted YAML rather than a typed accessor (durable across the s02
   typing change).

Live headline counts: **326 source docs, 60 covered, 266 uncovered; 6/12 arc dirs
+ 39/44 slice dirs represented; 44 stub bodies; 60/60 nodes missing provenance.**
Every divergence from the audit's ballpark (≈44 stubs / ≈211 uncovered / 5 missing
arcs) is reconciled by name in `coverage-report.md`'s closing section — none
rounded off.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | `coverage` module + read-only `--coverage` mode | **done** | Builds; `odm migrate --coverage --help` shows the flag. Flag name checked against `odm-command-inventory.md` — unclaimed. |
| F-2 | Classification covers all docs roots | **done** | `coverage_classify` green; live total (326) == `find docs -name '*.md' \| wc -l`. |
| F-3 | doc-coverage detector | **done** | `coverage_doc_coverage` green; live report lists 266 uncovered across 10 classes, each with a basis. |
| F-4 | representation detector | **done** | `coverage_representation` green; live report: 6 arc dirs + 5 slice dirs unrepresented, all named. |
| F-5 | stub-body detector | **done** | `coverage_stubs` green; live count 44, exact match to the audit. |
| F-6 | provenance-absence detector | **done** | `coverage_provenance_absence` green; live count 60/60. |
| F-7 | `--coverage` is read-only | **done** | Live run against the real store: `git status --porcelain` empty and node-file byte hash unchanged before/after; no store-write call site in the module; `coverage_is_read_only` integration test guards the regression. |
| F-8 | `coverage-report.md` produced, reconciled | **done** | Committed in the slice dir; every divergence from the audit explained by name, not rounded off. |
| F-9 | Clippy clean, no `unsafe`, coverage ≥ 90% | **done** | Workspace-wide clippy clean; no `unsafe` in the module; `coverage.rs` at 93.92% line coverage. |

**Rows: 9. Done: 9. Deferred: 0. No-op: 0.** No silent drops — every row opened in
`ledger.md` closes here with its evidence (`ledger.md`'s Evidence column carries the
exact command output per LEDGER-DISCIPLINE v2.0 §A; not duplicated here).

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --workspace` | all green (no failures found workspace-wide) |
| `cargo test -p odm-migrate --test coverage` | 6/6 (the 5 ledger-named tests + `coverage_is_read_only` for F-7) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all` | applied, clean |
| `unsafe` in the new module | none |
| `cargo llvm-cov -p odm-migrate --summary-only` (`coverage.rs`) | 296/18 lines missed = 93.92% (≥ 90% floor) |
| Live run against `.worktrees/odm` (60 nodes) | store untouched (`git status --porcelain` empty; node-file byte hash identical before/after) |

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s01 deliver its assigned piece?** Yes — the arc-plan named s01's job as
"build the read-only doc-coverage detector (inverse `orphan`) + sibling detectors
… → the exact gap inventory (the work-list)", and that is exactly what shipped:
a re-runnable tool plus a committed inventory with 100% of its totals internally
reconciled (every per-class count sums to the reported grand total, in both the
doc-coverage and representation dimensions).

**What it revealed the arc-plan didn't anticipate:**

1. **A real classification-boundary question for s02's model.** The audit's
   manual tally bucketed chunk-scale artifacts (`cN-closing-report.md`,
   `C-N-cdc-verification.md`, `cc-prompt-cN-*.md`) as "ad-hoc/other", distinct
   from the canonical per-*slice* `ledger`/`cc-prompt`/`cdc-verification`/
   `closing-report` cohort. This detector instead folds them into their
   slice-scale sibling class, since neither has a node class yet either way and
   the total is unaffected — but s02's supporting-doc node-class design will
   have to make this same call for real: is a chunk-scale artifact `part_of` its
   *slice* (folded, as here) or does it need its own artifact granularity between
   slice and arc? **Not resolved here — flagged for s02, not silently decided.**
2. **A doc root the arc-plan's own capability statement didn't name.**
   `docs/dev/research/` (5 docs) is distinct from `docs/dev/` (26 docs) in this
   corpus, and from the *tag-based* `research` node type ODDs already carry
   (5 more, inside the `odd` class) — three different things sharing the word
   "research". The arc-plan's exit criteria ("every `.md` under `1.0.x/docs/*`
   maps to a node") already covers all three by scope, but the *name* overlap is
   worth a line in s02's model doc so a future reader doesn't conflate them.
3. **This slice's own artifacts are part of the gap it measures.** Because s01
   mints nothing, its own `arc-plan.md`/`slice-doc.md`/`ledger.md`/`cc-prompt.md`
   correctly appear as uncovered/unrepresented in its own report — a
   self-referential finding that is *correct*, not a bug, but worth naming so a
   later reader doesn't mistake it for a matcher miss.

**The slice-scale silent-drop diff:** scope-as-specified (slice-doc.md's "In")
vs. scope-as-delivered — no drops. The four detectors, the `--coverage` mode, the
tests, and the committed report are all present. The one explicit "Out" item
re-confirmed as correctly out: no minting, no schema change, no wiring into
`check`/`validate` occurred (verified live: the store's `git status` was empty
after the real run).

**Recommended arc-ledger update (MF-1, MF-5):** both remain **planned**, not
**done** — s01 built the *detector*, not the *enforced check* (MF-1: "green on
`1.0.x/docs`" wants doc-coverage wired into `check`, which is s05's job) nor the
*representation fix* (MF-5: "all arcs … represented" wants the minting s04/s05
do). s01's contribution is the exact inventory those later slices plan against;
recorded as a pointer from MF-1/MF-5 to this closing report, not as a completed
composition row.
