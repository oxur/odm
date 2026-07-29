---
id: 01KYP5G62XF2K9TTWMGFSGN49F
number: 560947500
type: artifact
schema: artifact/v1.1
name: 'cc-prompt — arc-store-home slice 04: `odm store rename`'
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/slice04-store-rename/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SDKC6SHKGNN5VHEXK
---
# cc-prompt — arc-store-home slice 04: `odm store rename`

> **Arc:** Store Home & `init` (`arc-store-home`) · **Slice:** 04 (**last**) · **Feeds:** SH-4 ·
> **Realizes:** ODD-0022 §4.5. **Builds on** slice 01 (`[store]` locator + `StoreHome::resolve`) and
> slice 02 (`worktree.rs`, the git shell-out). **Slottable** — independent of slice 03. On close, the
> arc reaches its composition (SH-5); SH-6 (dogfood cutover) still rides RH C-5.

## Goal

Add **`odm store rename`**: move the store's worktree directory and/or rename its local orphan branch,
keeping the `[store]` locator in lockstep so `odm` never loses the corpus. The store root is derived
from `worktree-dir` + `branch-name` (slice 01), so a rename must move the git side **and** the locator
together — always leaving the store resolvable.

## Command placement

`odm store rename`, under the `store` group (ODD-0023), **distinct from `odm node rename`**. Surface:

- `odm store rename <new>` — bare positional renames **both** worktree and branch to `<new>` (they
  default equal, ODD-0022 §4.1).
- `--worktree <new>` / `--branch <new>` — rename them independently.
- `--dry-run`, `--yes`, `--json`.

## Changes

### 1. Git ops (in `worktree.rs`, the one shell-out module — §5 covers `init`/**rename**)

- **Worktree move**: `git worktree move <old_path> <new_path>` (git ≥ 2.17, not `--orphan`-gated).
- **Branch rename**: `git branch -m <old> <new>` — renames the branch checked out in the worktree;
  preserves its upstream config. **Local only.**
- Route both through the existing `run()` helper; map git's stderr into `StoreError::Git`.

### 2. The rename flow (`init.rs` or a sibling, calling `worktree.rs`)

Order for the **always-resolvable** invariant (L-5):

1. Validate: resolve the current store (must exist); compute targets; **collision check** (new
   worktree dir or new branch already exists → stop, name it) and **no-op** (new == current → clean
   message, done).
2. Apply the git op(s): worktree move, then branch rename (whichever were requested).
3. **Write the locator LAST, to mirror the *observed* git state** — read back where the worktree is and
   what the branch is called, and write `[store]` to match reality (not merely the intended target). So
   even a partial failure leaves the locator pointing at a store that exists; report exactly what
   succeeded and the manual fix if a step failed.
   - A git failure **before** any change → nothing moved, old locator still valid, plain error.

### 3. Published-branch guard (L-6)

If `--branch` (or the bare form) renames a branch that has an upstream or exists on a remote, **warn**:
the rename is **local-only** — `origin/<old>` keeps the old name; renaming the shared DB's branch is a
team coordination event (push-new + delete-old) and is **out of scope**. Do **not** attempt any remote
rename. Proceed with the local rename after the warning.

### 4. CLI (`store_cmd.rs`)

Wire `rename` into the `store` group. `--dry-run` prints old→new (worktree / branch / locator) and
touches nothing; `--json` reports `{ old_worktree, new_worktree, old_branch, new_branch, store_root }`.
Data → stdout, diagnostics → stderr; errors-as-affordances on collisions and the published-branch warn.

## Scope boundary (out)

- **Renaming the published/remote branch** across a team — deferred (coordination event; §6-style YAGNI).
- **`--force`/repair** beyond the locator-mirrors-reality guarantee — deferred.
- No node-model / gate / edge / corpus change — only *where the tree is rooted*.
- **Steady-state stays on gix** — rename git is `init`/rename-time only (L-11).

## Acceptance / ledger (SH-4, `ledger.md` L-1…L-12)

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- Unit: plan/target builder, collision + no-op decisions, the "locator mirrors observed git state" map.
- Integration (`assert_cmd`, scratch repo via `odm store init`):
  - rename **worktree only** → new path exists, old gone, `[store].worktree_name` updated, `list`/`check`
    green at the new path.
  - rename **branch only** → worktree on the new branch, `[store].branch_name` updated, resolution green,
    upstream preserved.
  - rename **both** (bare positional) → both move, locator matches.
  - **collision** → stops, nothing changed; **no-op** → clean message.
  - **published branch** (bootstrap → push to a bare remote → `--branch` rename) → warns local-only,
    `origin/<old>` still present.
  - `--dry-run` touches nothing; `--json` old/new correct.
- `grep` confirms the rename git ops live only in `worktree.rs` (L-11).

## Method

One branch (`sh-slice04-store-rename`, off the slice-03 tip or `release/1.0.x` — settle at start); one
mergeable diff; five-iteration cap. CC implements on local 1.85+ **with a real git**. CDC verifies
(`cdc-verification.md`); cargo rows attested → CI (the slice-02 two-arm git matrix runs the store suite).
On close, bubble up to `arc-store-home/arc-plan.md` (SH-4; anything unanticipated; silent-drop diff),
**then write `arc-store-home/closing-report.md`** — the arc closes code-complete + compose-verified
(SH-5 reproducible), with **SH-6 (dogfood cutover) explicitly pending RH C-5** (recorded, not dropped),
and a project bubble-up to `project-plan.md` (P-14).
