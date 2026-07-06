# Slice 01 (Arc 06): `migrate` importer core

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). Five-iteration cap. **Arc opener** — the importer core, tested on fixtures; slice02
> runs it on odm's real docs.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M-1 | `odm-migrate` crate + `odm migrate <legacy-path> [--dry-run]` command exist; workspace member (`[workspace.dependencies]` + `[workspace.lints]`, no version literals); builds; depends on `odm-core` + `odm-store` | `cargo build -p odm-migrate` → ok AND `grep -nE "\.workspace = true" crates/odm-migrate/Cargo.toml` AND `! grep -nE '= "[0-9]' crates/odm-migrate/Cargo.toml` AND `cargo test -p odm-cli migrate_command_exists` → ok | serious | arc-plan / CLAUDE.md | open | | Mirrors A4 slice01 creating `odm-index`. Added to `[workspace] members` + publish order. |
| M-2 | The legacy → new **mapping** is faithful: legacy `number` → preserved as the node's `number` + a fresh **ULID** identity; `state` scalar → the **`odd` gate-set position**; `supersedes`/`superseded-by` pair → a `supersedes` **edge** (with kind); `title`/`author`/`created`/`updated`/`tags`/`component` carried; `NodeType::Odd`; state-directory dropped | `cargo test -p odm-migrate maps_legacy_fields_to_node` → ok (a fixture ODD → a node with number preserved, gate at the mapped position, supersedes edge, type odd) | serious | arc-plan / 0013 §9 | open | | Pin the exact per-`state` → `odd`-gate mapping against the `odd` gate-set. Legacy number preserved is also the idempotence key (M-3). |
| M-3 | **Idempotent** describe-or-create: re-running `migrate` creates **nothing** new — a legacy doc whose `number` already exists as an `odd` node is **skipped**, not duplicated | `cargo test -p odm-migrate migrate_is_idempotent` → ok (migrate a fixture corpus, then migrate again → second run creates 0, reports skipped) | serious | arc-plan (idempotence key, v1.2) | open | | ULID identity is fresh (can't be re-minted) → idempotence keys on the preserved legacy `number`, not the id. |
| M-4 | **`--dry-run`** reports the plan (per-doc create/skip + counts) and writes **nothing** (no `nodes/` created, no git change) | `cargo test -p odm-migrate migrate_dry_run_writes_nothing` → ok (dry-run over a fixture → 0 nodes on disk; plan printed) | serious | arc-plan / 0013 §9 | open | | Also the reflexive-safety primitive slice03 leans on (preview before the self-host cutover). |
| M-5 | **Never-delete / supersede-not-delete**: no legacy file is removed or mutated; dustbin docs (`rejected`/`withdrawn`/`superseded`) import as superseded/retired nodes (git preserves history), legacy files intact | `cargo test -p odm-migrate migrate_never_deletes_legacy` + `dustbin_imports_as_superseded` → ok (after migrate, every legacy fixture file still exists; a dustbin doc → a superseded/retired node) | serious | arc-plan / 0013 §9 / supersede-not-delete | open | | The importer reads legacy + writes new only. Git is the history; dustbin is a node state, not a file deletion. |
| M-6 | Malformed/edge legacy input (missing `number`, unknown `state`, dangling `supersedes` target) → a **clear reported error / skip, never a panic or silent drop** | `cargo test -p odm-migrate migrate_malformed_reports_not_panics` → ok (an edge-case fixture → a positioned error / a reported skip, migrate continues or fails cleanly) | serious | project error convention (CLAUDE.md) | open | | A silent drop here would corrupt the self-host corpus later — the importer must be loud. |
| M-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for `odm-migrate` (+ any `odm-cli` migrate-command path) | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-migrate/src` AND `cargo llvm-cov --summary-only -p odm-migrate` → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 7. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
