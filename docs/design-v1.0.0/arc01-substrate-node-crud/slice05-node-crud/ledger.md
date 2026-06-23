# Slice 05 (Arc 01): Node CRUD commands

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| K-1 | `new <type> <name>` mints a ULID, allocates the next human number, persists | `cargo test -p odm-cli new_persists` (assert_cmd) → ok | serious | 0013 §7 | done | `f33c983`: `new_persists` → 1 passed. `new slice "Store layer"` → `created slice #1 (…ULID…)`; one `.md` persisted. Next number = max(existing)+1. | |
| K-2 | `new` is idempotent describe-or-create: re-running describes, does not duplicate | `cargo test -p odm-cli new_idempotent` → ok | serious | 0025 §4c / 0013 §7 | done | `f33c983`: `new_idempotent` → 1 passed. Second `new slice "Same"` prints `exists: slice #1 …`; node count stays 1. | Idempotency key = (type, exact name). |
| K-3 | `list` shows nodes; filters by type / tag / component | `cargo test -p odm-cli list_filters` → ok | correctness | 0013 §7 | done | `f33c983`: `list_filters` → 1 passed (type filter); tag/component filters covered in `extra::show_renders_all_fields_and_children`. | |
| K-4 | `show X` renders node + edges + way-finding in one call | `cargo test -p odm-cli show_node` → ok | correctness | 0013 §7 | done | `f33c983`: `show_node` → 1 passed. Full rendering (tags/component/part_of/supersedes/retired/children) covered by `extra` rich-fixture tests. | Way-finding = parent (`part_of`) + children (full-scan). |
| K-5 | `rename` changes `name` only — `id` and on-disk path unchanged | `cargo test -p odm-cli rename_keeps_id_and_path` → ok | serious | 0013 §2.1/§6 | done | `f33c983`: `rename_keeps_id_and_path` → 1 passed. Same single file path before/after; id unchanged; new name shown. | Path = f(id); persist rewrites in place. |
| K-6 | `retire X --because` marks withdrawn/removed; file preserved (git), not deleted | `cargo test -p odm-cli retire_preserves_file` → ok | serious | 0001 (supersede-don't-delete) | done | `f33c983`: `retire_preserves_file` → 1 passed. File still exists; frontmatter gains `retired:` + reason. | New typed `retired` field — see decision (1). |
| K-7 | `supersede X --with Y --kind obsoletes\|updates` records the lineage edge | `cargo test -p odm-cli supersede_with_kind` → ok | correctness | 0013 §3 | done | `f33c983`: `supersede_with_kind` → 1 passed. Edge recorded on Y (newer) → X with `kind: obsoletes`. | Semantics flagged in decision (2). |
| K-8 | `use [project\|arc] X` sets current context; `context` shows it (no `--project`/`--arc` needed after) | `cargo test -p odm-cli context_use_and_show` → ok | serious | 0025 §4a | done | `f33c983`: `context_use_and_show` → 1 passed. `use project`/`use arc` persist to `.odm/context.json`; `context` shows both; type mismatch rejected. | |
| K-9 | `--dry-run` on mutators writes nothing; `--yes` runs non-interactively | `cargo test -p odm-cli dry_run_and_yes` → ok | correctness | 0013 §7 | done | `f33c983`: `dry_run_and_yes` → 1 passed (new); retire/supersede dry-run covered in `extra`. `--yes` accepted; mutators run non-interactively. | See decision (3): `--yes` is an accepted affirmation (no interactive prompt yet). |
| K-10 | `--json` on queries has a stable, documented schema | `cargo test -p odm-cli json_schema_crud` (snapshot) → ok | correctness | 0013 §7 | done | `f33c983`: `json_schema_crud` → 1 passed. Asserts the exact key set + stable field values (id treated as a 26-char ULID, not pinned). | Schema = `NodeJson` (documented in `commands.rs`). |
| K-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% | `cargo clippy -p odm-cli --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-cli --summary-only --ignore-filename-regex '(odm-core|odm-store|odm-graph)/'` → **line** ≥ 90% (target 95%) | serious | CLAUDE.md | done | `f33c983`: clippy → exit 0; no `unsafe`; coverage **line 98.60%**, region 95.69% (TOTAL, odm-core/store/graph excluded). | |

## What Worked

- **A thin `odm-cli` binary made `cargo test -p odm-cli` + assert_cmd
  self-contained.** The published binary is `odm` (oxur-odm umbrella), but
  `cargo test -p odm-cli` doesn't build dependents — so assert_cmd had no `odm`
  to spawn. Adding a thin `odm-cli` bin (same `odm_cli::run`) sets
  `CARGO_BIN_EXE_odm_cli`, so the suite runs standalone, and cargo-llvm-cov
  captures the spawned binary's coverage.
- **Library fixtures for fields the CLI can't yet set.** tags/component/part_of
  have no CLI setter in this slice (linking is Arc 02), so the `show`/`list`
  rendering branches for them were unreachable end-to-end. Seeding nodes through
  `odm-core`/`odm-store` in the tests (then spawning `odm show`) exercised those
  branches and lifted line coverage to 98.6%.
- **Path = f(id) made `rename` trivially correct.** Because the path is derived
  from the immutable id, `rename` just rewrites the same file — K-5 ("id/path
  unchanged") falls out for free, verified by comparing the file path before and
  after.
- **Tying `created` to the ULID timestamp** (`Id::created_at().date_naive()`)
  kept `new` free of a separate clock read and made the persisted month and the
  `created` field consistent by construction.

## Closure

Closed at `f33c983` on 2026-06-23. CDC: _pending_ (cargo rows via CI / local
1.85+; sandbox has no toolchain). Total rows: 11. Done: 11. Deferred: 0.
No-op: 0.

> Also in this branch: a fix for the slice04 CI failure — gix's reflog write
> needs a config-based committer identity, absent in CI; `git.rs` now seeds an
> in-memory `user.name`/`user.email` after init/open (commit `d00f436`). Verified
> under an empty `HOME` with `GIT_CONFIG_NOSYSTEM=1`.
