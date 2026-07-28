# Slice 08 (Migration Fidelity): Source-path portability

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. Capability
> rows are fixture-proven (`attested` → CI); the live-rewrite rows are class-(b) (the committed store is
> the evidence — CDC reproduces by direct read). **Snapshot/revert + dry-run-first are HARD gates**
> (this rewrites all 61 just-migrated nodes). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **Relative-to-content-root storage**: `source.paths` stored as `docs/…`, never absolute, never worktree-prefixed | `cargo test -p odm-migrate` → a migrated node's `source.paths[0]` starts `docs/` (no leading `/`, no `.worktrees/`) | serious | CDC v2.8 F1 / v2.9 | open | | Anchor = git toplevel of the plan tree, **not** the superproject root. Seam: `resolve()`/`body_source_path`/`build_source`. |
| F-2 | **Canonical form**: forward slashes, no `./`/`..`, no trailing slash; stored form invariant to how the plan-root arg was spelled; a **decided, tested case rule** | `cargo test` → same node from arg `docs/…`, `./docs/…/`, and an absolute arg all store the identical path string | serious | slice-doc | open | | macOS case-insensitive vs Linux-CI case-sensitive — pick store-as-disk-compare-exact **or** case-fold, justify, test. |
| F-3 | **One shared anchor, write == resolve**: the stored relative path resolves back to an absolute for `reconcile_source`'s `fs::read`; write and read go through one function | `cargo test` → a node with a relative `source.paths` still passes the body-hash gate (the source file is read via anchor+relative) | serious | slice-doc | open | | Write and read cannot drift — one anchor, both directions. |
| F-4 | **Transition-safe matching (re-mint guard)**: an **absolute-stored** node and a **relative-discovered** lookup canonicalize to the **same** `by_source` key → updated in place, **not** re-minted | `cargo test` → a store seeded with absolute `source.paths` (the s07 shape) re-migrated → each node **matched + rewritten to relative**, **0 created** | serious | s05 transition / slice-doc | open | | The exact hazard: a naive rewrite would miss `by_source` and mint 61 duplicates. |
| F-5 | **Cross-checkout determinism** (the point): two stores from the same docs at different absolute roots → identical `source.paths` + `by_source` outcomes | `cargo test` → build over two `TempDir` roots; assert byte-identical `source.paths` and no duplicate on cross-root re-run | serious | slice-doc (why) | open | | The property absolute violated; the reason this slice exists. |
| F-6 | **Coverage set-difference stable across roots** | `cargo test` → doc-coverage (source-based) yields the same 0-uncovered from either root | serious | ODD-0025 §5 | open | | So s09's coverage-in-`check` is CI-portable. |
| F-7 | **Live rewrite**: 61 nodes absolute→relative, **bodies/ids/schema unchanged, 0 re-mint**, one revertible commit | dry-run previews **61 rewrites / 0 creates / 0 body changes**, store byte-identical after; fire → single commit atop captured SHA; `git reset --hard <SHA>` documented | serious | slice-doc | open | | **HARD gate:** any create in the dry-run → stop (F-4 wrong), don't fire. |
| F-8 | **Post-rewrite verification**: paths relative+canonical; 0 stubs; 61 source-bearing; project+retired excluded; bodies/ids/schema intact; `check` green; `orient`/`rollup` byte-stable; **cross-root re-run idempotent** | direct read of the committed store + a re-run from a different checkout path → 0 reconciled / 0 created | serious | slice-doc | open | | The live proof that portability holds and nothing else moved. |
| F-9 | **Rollback & findings discipline; no silent scope** | any gate failure → revert to SHA + finding, not forced; s09 items (artifact mint, check-wiring, coverage.rs fixes, design/research `source`) **not** pulled forward | correctness | LEDGER-DISCIPLINE / operator | open | | The re-mint case is the one to watch. |
| F-10 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) changed modules; **no model drift** | clippy exit 0; `! grep unsafe`; `llvm-cov` ≥ 90%; cross-read: `source` still the identity axis (now portable), gate unchanged, no stored hash; ODD-0025 amended not worked-around if a path-form line is needed | polish/correctness | CLAUDE.md / ODD-0025 | open | | Target 95%. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<impl-SHA>` (`release/1.0.x`) + store commit `<store-SHA>`
(`odm` branch) on `<date>`. Verified by: `<CDC/session>`.
Rows: 10. Done: _n_. Deferred: _n_. No-op: _n_. On close, bubble up to `../arc-plan.md`
(CDC v2.8 Finding 1 resolved; s09 coverage enforcement unblocked + next).
