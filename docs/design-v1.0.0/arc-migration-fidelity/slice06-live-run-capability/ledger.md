# Slice 06 (Migration Fidelity): Live-run capability

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. CC fills at
> `attested`; CDC reproduces (CI / local 1.85+; CDC sandbox has no 1.85+ cargo → cargo rows
> `attested`→`reproduced`-on-CI). **Fixture-only — no live-store mutation.** Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **Two-path fix**: `self_host`'s `to_populate` transition is now **gated** (`verify_body_hash`) and **excludes the project node** — one `source`-population policy, not two | `cargo test -p odm-migrate` → a drifted transition body → `BodyHashMismatch` (not stamped); the project node gets no `source` via the transition | serious | CDC v2.1 finding | open | | Route through `repair()`'s reconcile logic, or remove the transition's own backfill. |
| F-2 | **`odm migrate` runs the full reconcile flow** (repair→import), not `self_host()` alone; **`--dry-run`-able** | `cargo test -p odm-cli` → `odm migrate <self-host tree>` repairs+backfills then imports; `--dry-run` previews (repair/backfill/import counts) and writes nothing | serious | arc-plan v2.2 | open | | Entry point is `odm migrate` (self-host was folded into it, C-5), **not a new verb**. `repair()` had no caller outside tests. Confirm flag surface vs inventory + `odm migrate -h`. |
| F-3 | The command runs the flow in the **correct order** — reconcile existing (repair stubs + gated faithful-backfill) **before** importing missing arcs | `cargo test -p odm-cli` end-to-end → order asserted (existing nodes carry `source` before import matches by source) | serious | slice-doc | open | | So the import's source-key finds already-reconciled nodes. |
| F-4 | **End-to-end fixture flow**: stubs repaired (verbatim body + `source`), faithful backfilled (gated), missing arcs/slices imported, `v1.1` stamped, `source`/`author`/`version` populated | `cargo test -p odm-cli` on a fixture corpus (stubs + faithful + missing arcs) → all of the above green | serious | MF-2/MF-3/MF-5 | open | | The whole capability, exercised as one flow. |
| F-5 | **Project node excluded from 1:1 `source` by *every* path** (repair *and* the transition) | `cargo test -p odm-migrate` → after the full flow on a fixture with a synthesis-shaped project node, that node carries **no** 1:1 `source` | serious | ODD-0025 §2.3 / CDC v2.1 | open | | The exact case the v2.1 finding flagged. s09 synthesizes it. |
| F-6 | **`--dry-run` mutates nothing** — preview only | `cargo test -p odm-cli` → store byte-identical before/after a `--dry-run` of the full flow | serious | slice-doc | open | | The safety the live run (s07) leans on. |
| F-7 | **Re-run idempotent** — a second run mints no duplicate and re-stamps nothing needlessly (source-keyed, s05) | `cargo test` → run twice; node count + ids unchanged on the second | serious | s05 | open | | Confirms the flow is safely re-runnable end-to-end. |
| F-8 | **No live-store mutation** — fixtures/`TempDir` only; `.worktrees/odm` untouched | `git -C .worktrees/odm status --porcelain` empty; 0 live nodes gain `source:`; store hash unchanged | serious | slice-doc (Out) | open | | The live run is s07. |
| F-9 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) on changed modules | `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `! grep -RnE '\bunsafe\b' <changed>`; `cargo llvm-cov` ≥ 90% | polish | CLAUDE.md | open | | Target 95%. |
| F-10 | **No decided-model drift** — one gated `source` policy; project/synthesis excluded per ODD-0025 §2.3; `source` the identity key; no stored hash | cross-read code vs ODD-0025 §2.1/§2.3/§2.8 + CDC v2.1 finding | correctness | LEDGER-DISCIPLINE (spec-keeping) | open | | Any needed deviation → amend, don't work around. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 10. Done: _n_. Deferred: _n_. No-op: _n_. On close, bubble up to `../arc-plan.md`
(v2.1 + v2.2 findings resolved; s07 fires the flow live) per LEDGER-DISCIPLINE v2.0 §A.
