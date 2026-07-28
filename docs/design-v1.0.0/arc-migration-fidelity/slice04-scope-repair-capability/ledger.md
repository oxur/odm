# Slice 04 (Migration Fidelity): Scope + repair capability

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. CC fills
> at `attested` per commit; CDC reproduces (CI / local 1.85+; CDC sandbox has no 1.85+ cargo →
> cargo rows `attested`→`reproduced`-on-CI). **Fixture-only — no live-store mutation.** Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | The `MAX_MVP_ARC` / `arc_in_scope` cap is **removed** — `self_host` imports every arc dir (numbered A1–A8 + named) | `! grep -nE 'MAX_MVP_ARC' crates/odm-migrate/src/` (predicate deleted, not raised); a test imports a fixture with `arc07` + a named arc | serious | arc-plan v1.6 F11 | open | | Delete the predicate; don't hardcode a higher constant (no new lock-in). |
| F-2 | Named arcs (+ their slices) get a **deterministic, collision-free `number` handle** | `cargo test -p odm-migrate` named-arc test → a named arc node has a unique `number`, no collision with A1–A8's 1100–1800 | serious | arc-plan v1.6 F12 | open | | Handle only — not identity/order. State the rule in code + report. |
| F-3 | `coverage.rs`'s shared `arc_in_scope` use updated for the removed cap — representation reflects all arc dirs in-scope | `cargo test -p odm-migrate coverage*` green; a formerly out-of-scope arc now counts as representable (not silently excluded) | correctness | arc-plan v1.6 (shared predicate) | open | | The s01 detector shares the predicate (`coverage.rs:512`). |
| F-4 | **Update-in-place repair** op: a stub node is repaired with verbatim body + `source`, preserving `id`/`edges`/`status`/`number` | `cargo test -p odm-migrate` repair test asserts body==source (post-normalize), `source` populated, and id/edges/status/number **unchanged** across the rewrite | serious | ODD-0025 §2.8 | open | | Via `Store::persist` overwrite — **no `delete`**. Stub = lone-H1 (reuse s01 predicate). |
| F-5 | Repair goes through the **hard body-hash gate**; a mismatch hard-fails | `cargo test -p odm-migrate` repair gate test: faithful repair ok; a mutated body → `MigrateError::BodyHashMismatch` | serious | ODD-0025 §2.1 | open | | Reuse `fidelity::verify_body_hash` / `build_source`. |
| F-6 | **Schema-minor bump executed**: `SchemaVersion::CURRENT` = `v1.1`; new/repaired nodes stamp `<type>/v1.1` | grep `schema.rs` CURRENT == v1.1; `cargo test -p odm-core` stamps `v1.1` | serious | ODD-0020 §4 / s03 carry-forward | open | | Closes the s03 deferral. |
| F-7 | **Forward-compat**: existing `v1.0` nodes still validate; `is_newer_than_current` treats `v1.1` as current, `v1.0` as older-valid | `cargo test -p odm-core` schema tests → a `v1.0` node passes `check`; `v1.1` is current | serious | ODD-0020 §5 (forward-compat) | open | | The workspace "unsupported-newer-schema" contract must still hold. |
| F-8 | Stub detection reused/consistent with s01 (`≤ 1 non-blank body line`, tombstones excluded) | `cargo test -p odm-migrate` → the repair path targets exactly the stub set s01 counts (44 on the real shape) | correctness | s01 / design-notes F8 | open | | Don't re-derive the stub predicate. |
| F-9 | **No live-store mutation** — tests use fixtures/`TempDir` only; `.worktrees/odm` untouched | `git -C .worktrees/odm status --porcelain` empty; 0 live nodes gain `source:`; node md5-of-md5 unchanged | serious | slice-doc (Out) | open | | The live run is s05. |
| F-10 | ODD-0020 §4 records `v1.1` as current (the doc matches the code bump) | grep 0020 §4 for the v1.1-current statement | serious | ODD-0025 §4 / s03 carry-forward | open | | Doc + code in sync. |
| F-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) on changed modules | `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `! grep -RnE '\bunsafe\b' <changed>`; `cargo llvm-cov` ≥ 90% | polish | CLAUDE.md | open | | Target 95%. |
| F-12 | **No decided-model drift** — repair is update-in-place (ODD-0025 §2.8, `persist` overwrite, no `delete`); the record is `source`; no stored body hash; cap removed not raised | cross-read code vs ODD-0025 §2.8 + arc-plan v1.6 | correctness | LEDGER-DISCIPLINE (spec-keeping) | open | | Any needed deviation → amend ODD-0025 / raise to CDC, don't work around. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 12. Done: _n_. Deferred: _n_. No-op: _n_. On close, bubble up to `../arc-plan.md`
(MF-2/MF-3/MF-5 capability; schema bump closes the s03 carry-forward) per LEDGER-DISCIPLINE v2.0 §A.
