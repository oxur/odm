---
id: 01KYP5FT9E3DRJ7J9FFJVQZKEH
number: 587304700
type: artifact
schema: artifact/v1.1
name: 'Slice 01 (Migration Fidelity): Coverage discovery'
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice01-coverage-discovery/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SABSXWZ58M8TQK3G3
---
# Slice 01 (Migration Fidelity): Coverage discovery

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row must reach ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). **Read-only slice** — mints nothing, changes no schema. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | A `coverage` module exists in `odm-migrate`, builds, and is reachable from a read-only `migrate --coverage` mode | `cargo build -p odm-migrate` → exit 0 AND `odm migrate --coverage --help` shows the flag | serious | arc-plan | done | `crates/odm-migrate/src/coverage.rs` @ `50b91c2`; `cargo build -p odm-migrate` exit 0; `odm migrate --coverage --help` prints the `--coverage` flag with its description | Sibling to `--dry-run`, `conflicts_with_all` the other derivation flags. Flag name confirmed against `odm-command-inventory.md` (no existing `--coverage` use; none of the shipped verbs claim it). |
| F-2 | Source enumeration + classification covers **all** docs roots (`design-v1.0.0/`, `design/`, `dev/`, research) — every `.md` gets a class | `cargo test -p odm-migrate coverage_classify` → ok AND report's total-source count == `find docs -name '*.md' \| wc -l` | serious | design-notes | done | `cargo test -p odm-migrate --test coverage coverage_classify` → ok (1 passed); live run: report "Source docs: 326 total" == `find docs -name '*.md' \| wc -l` = 326 | 11 classes: project-plan/arc-plan/slice-doc/ledger/cc-prompt/cdc-verification/closing-report/odd/dev/research/other, `DocClass::all()` asserted duplicate-free by a unit test. |
| F-3 | **doc-coverage** detector: every source doc is matched to a node or reported uncovered (heuristic — coordinates for corpus, number/title for ODDs) | `cargo test -p odm-migrate coverage_doc_coverage` → ok; report lists uncovered by class | serious | audit/MF-1 | done | `cargo test -p odm-migrate --test coverage coverage_doc_coverage` → ok; live run: `coverage-report.md` §1, 266 uncovered across 10 classes, each entry carries a basis string | 266 uncovered, not ≈211 — reconciled in `coverage-report.md`'s "Reconciliation vs. the audit" (scope the audit excluded + a classification difference + session growth since the audit; no unexplained residual). |
| F-4 | **representation** detector: arc/slice **dirs** vs **nodes** gap reported, missing units named | `cargo test -p odm-migrate coverage_representation` → ok; report shows 6 arc nodes vs 11 dirs, the 5 named (store-home, release-hardening, llm-command-surface, arc07, arc08) | serious | audit/MF-5 | done | `cargo test -p odm-migrate --test coverage coverage_representation` → ok; live run: `coverage-report.md` §2, 6/12 arc dirs represented, missing 6 named (the audit's 5 + `arc-migration-fidelity` itself, shaped after the audit); 39/44 slice dirs, missing 5 named | Root cause is `arc_in_scope` A1–A6 (selfhost.rs) — unchanged by design; representation is derived from the classified `arc-plan`/`slice-doc` docs, not a second independent directory walk, so it needs no assumption about where `design-v1.0.0/` sits. |
| F-5 | **stub-body** detector: nodes with ≤ 1 non-blank body line are listed (tombstones excluded) | `cargo test -p odm-migrate coverage_stubs` → ok; report stub count | serious | audit | done | `cargo test -p odm-migrate --test coverage coverage_stubs` → ok; live run: `coverage-report.md` §3 lists exactly 44 (6 arc + 38 slice) | Exact match to the audit's live-verified 44; no drift since. |
| F-6 | **provenance-absence** detector: nodes with no `provenance` field are listed | `cargo test -p odm-migrate coverage_provenance_absence` → ok; report count | correctness | design-notes | done | `cargo test -p odm-migrate --test coverage coverage_provenance_absence` → ok; live run: `coverage-report.md` §4 lists all 60 | Checked against the node's **emitted** YAML (`document.emit()`, scanned for a `provenance:` line in the frontmatter block) rather than a typed accessor — durable across the s02 typing change, per the module's design note; a unit test (`coverage_provenance_absence`) exercises both the absent and present cases. |
| F-7 | `migrate --coverage` is **read-only**: it writes/mints no node and leaves the store unchanged | run `odm migrate --coverage`; `git -C .worktrees/odm status --porcelain` empty AND `! grep -rnE 'persist\|write\|mint' <coverage module>` (no store-write calls in the module) | serious | slice-doc | done | Ran `./target/release/odm migrate docs --coverage` from `release/1.0.x` against the live 60-node store: `git -C .worktrees/odm status --porcelain` empty before and after; `find .worktrees/odm/nodes -name '*.md' \| sort \| xargs md5 \| md5` identical before/after (`40aecffaa88be3aa8764d9423594c8b0`); `grep -rnE 'persist\|write\|mint' crates/odm-migrate/src/coverage.rs` matches only doc-comment prose (`persists`/`written`/`mint` inside `///` lines), no call sites; `coverage_is_read_only` integration test asserts a byte-identical store snapshot | The live run above is the real safety demonstration; the integration test is the regression guard. |
| F-8 | `coverage-report.md` is produced and its headline counts reconcile with the audit ballpark (or divergence is explained) | file exists in slice dir; counts within expected ranges, each divergence noted in the report | serious | slice-doc | done | `docs/design-v1.0.0/arc-migration-fidelity/slice01-coverage-discovery/coverage-report.md` committed; stub-body count matches exactly, arc/slice representation and doc-coverage counts diverge from the audit's ballpark with the divergence explained in the report's "Reconciliation vs. the audit" section (fully accounted for, no unexplained residual) | This report IS the arc's work-list for s02–s07. |
| F-9 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the new module | `cargo clippy -p odm-migrate --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' <coverage module>` AND `cargo llvm-cov -p odm-migrate --summary-only …` → module line ≥ 90% | polish | CLAUDE.md | done | `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, zero warnings; `grep -nE '\bunsafe\b' crates/odm-migrate/src/coverage.rs` → no matches; `cargo llvm-cov -p odm-migrate --summary-only` → `coverage.rs` 296/18 lines missed = 93.92% (≥ 90% floor, short of the 95% stretch target) | `cargo fmt --all` applied; `cargo test --workspace` green (no failures, workspace-wide). |

## What Worked

- **Deriving the representation detector from the already-classified doc list** (rather than a second, blind filesystem walk rooted at a guessed "plan-set root") caught its own bug immediately: a first pass hardcoded walking `docs_root` directly, which silently found 0 arc dirs against the real corpus (arcs sit under `design-v1.0.0/`, an assumption the general docs-tree API must not make). Re-deriving from `DocClass::ArcPlan`/`DocClass::SliceDoc` entries fixed it and removed the redundant walk — simpler *and* more correct.
- **Running the tool against the live corpus before writing the report** surfaced exact, verifiable divergences from the audit (the audit's exact-basename tally vs. this detector's chunk-variant folding; the corpus's growth since the audit ran) rather than requiring hand-reconciliation after the fact — the numbers were self-consistent (the four detector totals summed to the reported grand total) on the first real run.
- Reusing `selfhost::parse_prefix`/`arc_number`/`slice_number` (bumped to `pub(crate)`) instead of re-deriving the coordinate parse meant the representation and doc-coverage matchers could never silently drift from `self_host`'s own numbering — a single source of truth for "what number does this directory mint."

## Closure

Closed at commit `50b91c2` on 2026-07-27. Verified by: CC (this session); CDC
reproduction pending. Rows: 9. Done: 9. Deferred: 0. No-op: 0. On close, bubble up
to `../arc-plan.md` (Arc Ledger MF-1/MF-5) per LEDGER-DISCIPLINE v2.0 §A / PM Part IV.
