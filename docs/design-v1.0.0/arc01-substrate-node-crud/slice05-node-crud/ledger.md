# Slice 05 (Arc 01): Node CRUD commands

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| K-1 | `new <type> <name>` mints a ULID, allocates the next human number, persists | `cargo test -p odm-cli new_persists` (assert_cmd) → ok | serious | 0013 §7 | open | | |
| K-2 | `new` is idempotent describe-or-create: re-running describes, does not duplicate | `cargo test -p odm-cli new_idempotent` → ok | serious | 0025 §4c / 0013 §7 | open | | |
| K-3 | `list` shows nodes; filters by type / tag / component | `cargo test -p odm-cli list_filters` → ok | correctness | 0013 §7 | open | | |
| K-4 | `show X` renders node + edges + way-finding in one call | `cargo test -p odm-cli show_node` → ok | correctness | 0013 §7 | open | | |
| K-5 | `rename` changes `name` only — `id` and on-disk path unchanged | `cargo test -p odm-cli rename_keeps_id_and_path` → ok | serious | 0013 §2.1/§6 | open | | |
| K-6 | `retire X --because` marks withdrawn/removed; file preserved (git), not deleted | `cargo test -p odm-cli retire_preserves_file` → ok | serious | 0001 (supersede-don't-delete) | open | | |
| K-7 | `supersede X --with Y --kind obsoletes\|updates` records the lineage edge | `cargo test -p odm-cli supersede_with_kind` → ok | correctness | 0013 §3 | open | | |
| K-8 | `use [project\|arc] X` sets current context; `context` shows it (no `--project`/`--arc` needed after) | `cargo test -p odm-cli context_use_and_show` → ok | serious | 0025 §4a | open | | |
| K-9 | `--dry-run` on mutators writes nothing; `--yes` runs non-interactively | `cargo test -p odm-cli dry_run_and_yes` → ok | correctness | 0013 §7 | open | | |
| K-10 | `--json` on queries has a stable, documented schema | `cargo test -p odm-cli json_schema_crud` (snapshot) → ok | correctness | 0013 §7 | open | | |
| K-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% | `cargo clippy -p odm-cli --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-cli --summary-only` ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at `<SHA>` on `<date>`. CDC: `<name>` (cargo rows via CI/local 1.85+).
Total rows: 11. Done: _. Deferred: _. No-op: _.
