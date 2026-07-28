# Slice 03 (Migration Fidelity): Fidelity core

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. CC fills
> evidence at `attested` per commit; CDC reproduces (CI / local 1.85+; the CDC sandbox has no
> 1.85+ cargo, so cargo rows are `attested`→`reproduced`-on-CI). **Fixture-only** — no live-store
> mutation. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | `source` (typed sub-struct: `paths`/`class`/`normalization`/`migrated_by`/`migrated_on`), `author: Option<String>`, `version: Option<String>` exist on `Frontmatter`, in a documented canonical slot | `cargo build -p odm-core` exit 0; grep `frontmatter.rs` for the three fields + a canonical-order comment | serious | ODD-0025 §2.2 | open | | `author` moves off `extra` (was `insert_extra` at `mapping.rs:229`). |
| F-2 | Round-trip `parse ∘ emit = identity` holds with the new fields | `cargo test -p odm-core` round-trip/proptest → ok | serious | 0013 §2.3 invariant | open | | Extend the existing proptest, don't fork it. |
| F-3 | Per-type validity: `author`/`version`/`source` valid on document nodes; a `check` finding on a work node | `cargo test -p odm-core` validity test → ok (both valid + invalid cases) | correctness | ODD-0020 §2 | open | | Shared-core + per-type-validity, not N structs. |
| F-4 | `selfhost.rs` imports the **verbatim source body** — the stub synthesis (`format!("# {}\n", fm.name())`, ~line 201) is gone | grep `selfhost.rs` — no `format!("# {}` stub; a test asserts imported body == fixture source body (post-normalize) | serious | ODD-0025 §2.1 / design-notes root cause | open | | arc/slice bodies come from `arc-plan.md`/`slice-doc.md` under `PlanNode.source`. |
| F-5 | `mapping.rs` imports the ODD body verbatim and types `author`/`version` onto the new fields | `cargo test -p odm-migrate` mapping test → ok | serious | ODD-0025 §2.1/§2.4 | open | | mapping.rs is already ~faithful; align + type the fields. |
| F-6 | **Hard body-hash gate** (both importers): `sha256(normalize(src)) == sha256(normalize(node))`, `normalize = trim+lf`; mismatch is a hard `MigrateError` | `cargo test -p odm-migrate` gate **pass-case** (faithful body ok) **and fail-case** (mutated body → err) → ok | serious | ODD-0025 §2.1 | open | | Hard-fail, not a per-doc skip. |
| F-7 | `source` sub-map populated at migration (paths/class/normalization=`trim+lf`/tool+version/date) | test: a fixture-migrated node carries a populated `source` | serious | ODD-0025 §2.2 | open | | `PlanNode.source` already holds the path (`selfhost.rs:130`). |
| F-8 | `author`/`version` preserved from source frontmatter onto the typed fields | test: a fixture ODD with `author`/`version` → node carries them | correctness | ODD-0025 §2.2 | open | | Not git-derived (git returns the migrator). |
| F-9 | ODD-0025 §4 amendments **applied**: ODD-0013 §2.3 (`source`/`author`/`version`), §2.2 (`artifact` named for s05), §9 (new migration semantics); ODD-0020 §2/§4 (schema-minor note) | grep the ODDs for the added fields/sections | serious | ODD-0025 §4 | open | | Doc edits; the `artifact` **NodeType enum + minting** are s05, not here. |
| F-10 | **No live-store mutation** — tests run on fixtures/temp stores only; the `.worktrees/odm` corpus is untouched | tests use `TempDir`/fixtures; `git -C .worktrees/odm status --porcelain` empty after the test run | serious | slice-doc (Out) | open | | Live re-migration is s04. |
| F-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) on the changed modules | `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `! grep -RnE '\bunsafe\b' <changed>`; `cargo llvm-cov` module line ≥ 90% | polish | CLAUDE.md | open | | Target 95%. |
| F-12 | **No decided-model drift** — the implementation matches ODD-0025 (normalize = `trim+lf`; the stored record is named `source` not `provenance`; no stored body hash) | cross-read the code against ODD-0025 §2.0–§2.2 | correctness | LEDGER-DISCIPLINE (spec-keeping) | open | | Any needed deviation → amend ODD-0025, don't work around. |

## What Worked

_(At slice close. Patterns that made the slice close cleanly.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 12. Done: _n_. Deferred: _n_. No-op: _n_. On close, bubble up to `../arc-plan.md`
(MF-2/MF-3 plan against this capability) per LEDGER-DISCIPLINE v2.0 §A / PM Part IV.
