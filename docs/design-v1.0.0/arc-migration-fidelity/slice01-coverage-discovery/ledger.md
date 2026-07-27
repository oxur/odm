# Slice 01 (Migration Fidelity): Coverage discovery

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row must reach ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). **Read-only slice** — mints nothing, changes no schema. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | A `coverage` module exists in `odm-migrate`, builds, and is reachable from a read-only `migrate --coverage` mode | `cargo build -p odm-migrate` → exit 0 AND `odm migrate --coverage --help` shows the flag | serious | arc-plan | open | | Sibling to `--dry-run`; confirm flag name vs `odm-command-inventory.md`. |
| F-2 | Source enumeration + classification covers **all** docs roots (`design-v1.0.0/`, `design/`, `dev/`, research) — every `.md` gets a class | `cargo test -p odm-migrate coverage_classify` → ok AND report's total-source count == `find docs -name '*.md' \| wc -l` | serious | design-notes | open | | Classes: project-plan/arc-plan/slice-doc/ledger/cc-prompt/cdc-verification/closing-report/odd/dev/research/other. |
| F-3 | **doc-coverage** detector: every source doc is matched to a node or reported uncovered (heuristic — coordinates for corpus, number/title for ODDs) | `cargo test -p odm-migrate coverage_doc_coverage` → ok; report lists uncovered by class | serious | audit/MF-1 | open | | ≈211 uncovered expected (supporting docs + dev + research); reconcile the number. |
| F-4 | **representation** detector: arc/slice **dirs** vs **nodes** gap reported, missing units named | `cargo test -p odm-migrate coverage_representation` → ok; report shows 6 arc nodes vs 11 dirs, the 5 named (store-home, release-hardening, llm-command-surface, arc07, arc08) | serious | audit/MF-5 | open | | Root cause is `arc_in_scope` A1–A6 (selfhost.rs). |
| F-5 | **stub-body** detector: nodes with ≤ 1 non-blank body line are listed (tombstones excluded) | `cargo test -p odm-migrate coverage_stubs` → ok; report stub count | serious | audit | open | | ≈44 expected (6 arc + 38 slice); exclude the 1 `retired:` tombstone. |
| F-6 | **provenance-absence** detector: nodes with no `provenance` field are listed | `cargo test -p odm-migrate coverage_provenance_absence` → ok; report count | correctness | design-notes | open | | Currently expected = all 60 (provenance lands in s03); the detector is the durable check. |
| F-7 | `migrate --coverage` is **read-only**: it writes/mints no node and leaves the store unchanged | run `odm migrate --coverage`; `git -C .worktrees/odm status --porcelain` empty AND `! grep -rnE 'persist\|write\|mint' <coverage module>` (no store-write calls in the module) | serious | slice-doc | open | | The read-only guarantee is the safety of running it on the live corpus. |
| F-8 | `coverage-report.md` is produced and its headline counts reconcile with the audit ballpark (or divergence is explained) | file exists in slice dir; counts within expected ranges, each divergence noted in the report | serious | slice-doc | open | | This report IS the arc's work-list. |
| F-9 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the new module | `cargo clippy -p odm-migrate --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' <coverage module>` AND `cargo llvm-cov -p odm-migrate --summary-only …` → module line ≥ 90% | polish | CLAUDE.md | open | | |

## What Worked

_(At slice close. Patterns that made the slice close cleanly.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 9. Done: _n_. Deferred: _n_. No-op: _n_. On close, bubble up to `../arc-plan.md`
(Arc Ledger MF-1/MF-5) per LEDGER-DISCIPLINE v2.0 §A / PM Part IV.
