---
id: 01KZ1YCPRDNKE49ZW62QEA4YJJ
number: 522918400
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 01 (Store Lifecycle): `store commit`'
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-lifecycle/slice01-store-commit/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK41J56PMJ2JV7QDB6T8
---
# CC Prompt — Slice 01 (Store Lifecycle): `store commit`

Add `odm store commit` — persist the store worktree's pending node changes as a commit on the orphan
branch, odm-aware and idempotent, so the operator never drops to raw git. **The near-term need:** the
arc-migration-fidelity freeze mutates the store worktree and there is no odm verb to commit it.

> **Start condition:** on `release/1.0.x`, green. `store init` already makes the *first* commit; nothing
> commits after. This slice adds the commit verb.

## Read first

1. `slice01-store-commit/ledger.md` (8 rows) + `slice-doc.md` (esp. **D-1** — the auto-summary message).
2. **ODD-0022** (the store model: orphan branch, odm owns it, never rewrite history).
3. **Code:** `crates/odm-store/src/git.rs` + `worktree.rs` (the git plumbing `init` already uses to
   commit — `init.rs` runs `git commit -q -m "initial"` at bootstrap; reuse/extend that path); the store
   resolution (`StoreHome`); `crates/odm-cli/src/lib.rs` (`StoreCommand` enum — add `Commit`) +
   `store_cmd.rs` (the `store` dispatch).

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `03-error-handling`, `05-type-design`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **The command** (F-1). `StoreCommand::Commit { message: Option<String>, dry_run: bool, json: bool }` +
   the `store_cmd.rs` handler. Resolve the store home, stage the worktree's pending changes, commit on the
   **orphan branch** (never the code branch). Reuse `odm-store`'s git plumbing; add a
   "commit the current worktree state" entry point on `odm-store` if one isn't exposed (keep git ops in
   `odm-store`, not the CLI).
2. **Auto-summary message** (F-2, D-1). Default = the pending **node delta**: count added/modified/removed
   node files, classify by `type` (parse the changed nodes' frontmatter), render in odm terms (e.g.
   `store: +2 slice, ~3 arc, ~2 design, -1 project (retired)`). `-m` overrides. **Recommend the git-delta
   version; flag** the richer migrate-aware summary (reconciled/minted/collapsed) as a future migrate→commit
   handoff, don't build it.
3. **Idempotent no-op** (F-3). Clean worktree → exit 0, "nothing to commit", **no empty commit**.
4. **`--dry-run`** (F-4) writes no commit; **`--json`** (F-5) emits
   `{committed, sha, branch, message, delta:{created,modified,removed,by_type}}`.
5. **Store discipline** (F-6). Commit lands on the orphan branch in the worktree; the code checkout is
   untouched; no history rewrite. **Confirm + flag** what's staged (node files + `config.toml`; the `.odm/`
   index's tracked-vs-ignored status) and the **commit author identity** (ambient git vs odm-stamped).
6. **Fixtures** (F-1…F-5): commit-persists; no-op-when-clean; dry-run-writes-nothing; `-m` override; json
   shape.

## Constraints (flag, don't silently change)

- **Orphan branch only.** Never commit to or touch the code branch; never rewrite existing history.
- **Git ops live in `odm-store`**, not the CLI (the CLI orchestrates).
- **`auto_stage_git` stays dormant** — `commit` is explicit; don't wire auto-commit.
- Don't build `store status`/`store sync` (later slices) — but factor the node-delta computation reusably
  (status needs it).
- No `unsafe`; typed errors; clippy `-D warnings` clean; cover the new code.

## Deliverables

The `store commit` command + fixtures on `release/1.0.x`; any `odm-store` entry point added; `ledger.md`
evidence per row; `closing-report.md` — the walk, D-1 + the staged-set/author decisions, the v2.0 bubble-up
(SL-1 done; s02 next). Branch: `release/1.0.x` only.

## Working agreement

Amend don't work around; flag deviations; five-iteration cap. Your `done` is proposed-done — CDC reproduces
the command + fixtures + CI, and (operator) a real `store commit` of the frozen migration store lands one
commit on `odm`. On close, bubble up to `../arc-plan.md`.
