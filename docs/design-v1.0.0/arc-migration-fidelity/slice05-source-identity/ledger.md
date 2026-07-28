# Slice 05 (Migration Fidelity): Source-based identity

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. CC fills at
> `attested` per commit; CDC reproduces (CI / local 1.85+; CDC sandbox has no 1.85+ cargo → cargo
> rows `attested`→`reproduced`-on-CI). **Fixture-only — no live-store mutation.** Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | `self_host` idempotence keys on **`source.paths`**, not `(type, number)` | `cargo test -p odm-migrate` idempotence test → a source-bearing fixture node is matched by source (not number) on re-run; grep shows no `(type, number)` idempotence key | serious | design-notes F12 / arc-plan v1.9 | open | | The stable identity s03 gave every node. |
| F-2 | **Coordinate→source transition**: a pre-`source` node is matched by coordinate for its **one-time** `source` population, then by `source` | `cargo test -p odm-migrate` transition test: first run populates via coordinate; second run matches via source (no re-populate, no duplicate) | serious | slice-doc | open | | The corpus is pre-`source` today — the transition is required. |
| F-3 | **Re-run duplicate is impossible**: adding a named arc + re-running mints **no** duplicate (the s04 v1.8 hazard) | `cargo test -p odm-migrate` → a fixture with source-bearing named-arc nodes, add a name that sorts earlier, re-run → node count unchanged, no dup | serious | s04 CDC v1.8 finding | open | | This row *is* the reason the slice exists. |
| F-4 | **Backfill / reconcile-to-source**: a faithful non-stub node lacking `source` gets one; body unchanged; gate passes as a no-op | `cargo test -p odm-migrate` backfill test: faithful node → `source` populated, body byte-identical, `updated`/schema bumped | serious | slice-doc / ODD-0025 §2.2 | open | | Generalizes s04's `repair` beyond stubs. |
| F-5 | The gate on backfill **surfaces** a non-faithful body (a drifted node), not swallow it | `cargo test -p odm-migrate` → a non-stub node whose body ≠ source body → `MigrateError::BodyHashMismatch` | serious | ODD-0025 §2.1 | open | | Backfill enforces fidelity; it doesn't blindly stamp `source`. |
| F-6 | **Project node excluded** (synthesis — body ≠ source): not backfilled here, flagged for s08 | `cargo test -p odm-migrate` → the project node is skipped by backfill (not an error, not forced through the 1:1 gate) | correctness | ODD-0025 §2.3 (synthesis) | open | | `replan.rs::vision_from_plan` synthesizes it; s08 handles it. |
| F-7 | **Source-based coverage matching**: `coverage.rs` matches source-doc → node by `source.paths` (exact); named arcs resolve | `cargo test -p odm-migrate coverage*` → a source-bearing named-arc node resolves (no false "uncovered"); exact set-difference | serious | ODD-0025 §5 | open | | Resolves CC's s04-disclosed named-arc coverage gap. *(Split-escape: may move to s07 if s05 runs heavy.)* |
| F-8 | **Name-derived stable handle**: named-arc `number` derived from the slug (into the ≥ 1900 band, collision-handled), stable under adding arcs | `cargo test -p odm-migrate` → the handle is recomputable from the slug and unchanged when another named arc is added | correctness | s04 CDC v1.8 finding | open | | Replaces `named_arc_number(index)` (position-based). Cosmetic — nothing keys on it. |
| F-9 | `number` keys **nothing** for correctness — used only for display / CLI lookup | grep: no `(type, number)` idempotence or coverage key remains; `commands.rs`'s number lookup is display-only | serious | design-notes F12 | open | | The point of the slice: retire number-as-key entirely. |
| F-10 | **No live-store mutation** — fixtures/`TempDir` only; `.worktrees/odm` untouched | `git -C .worktrees/odm status --porcelain` empty; 0 live nodes gain `source:`; store hash unchanged | serious | slice-doc (Out) | open | | The live run is s06. |
| F-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) on changed modules | `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `! grep -RnE '\bunsafe\b' <changed>`; `cargo llvm-cov` ≥ 90% | polish | CLAUDE.md | open | | Target 95%. |
| F-12 | **No decided-model drift** — `source` is the identity key; `repair` extended (not replaced); synthesis (project node) excluded per ODD-0025 §2.3; no stored hash | cross-read code vs ODD-0025 §2.1/§2.2/§2.8/§5 + design-notes F12 | correctness | LEDGER-DISCIPLINE (spec-keeping) | open | | Any needed deviation → amend, don't work around. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 12. Done: _n_. Deferred: _n_. No-op: _n_. On close, bubble up to `../arc-plan.md`
(resolves the v1.8 pre-mint requirement at root; F12 done) per LEDGER-DISCIPLINE v2.0 §A.
