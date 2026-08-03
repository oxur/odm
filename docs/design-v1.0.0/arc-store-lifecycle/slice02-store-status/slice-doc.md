# Slice 02 (Store Lifecycle) — `store status`

> Refs: `../arc-plan.md` (SL-2) · **ODD-0022** (the store model: orphan branch, odm owns it) ·
> `crates/odm-store/src/delta.rs` (the node delta s01 factored for reuse) ·
> `crates/odm-store/src/init.rs` (`Ancestry`, `SyncAction`, `sync_action()` — the remote-tracking
> plumbing `init`'s sync arm already uses) ·
> `crates/odm-store/src/git.rs` (`Repo`, `tree_delta`, `is_clean`) ·
> `crates/odm-cli/src/store_cmd.rs` (the `store` dispatch — `commit` is the sibling).
> `depends_on:` s01 (`store commit` — factored `delta.rs`).
>
> **The read half of the same coin as commit.** s01 computes the delta and persists it; this slice
> computes the delta and *reports* it — plus adds the remote-tracking dimension (ahead/behind vs.
> the configured upstream) that `commit` doesn't need but `sync` (s03) will.

## Goal

Add `odm store status` — a **read-only** view of the store's git state, reporting two dimensions:

1. **The pending node delta** that `store commit` would write: created / modified / removed node
   files, classified by type, in odm terms — the same `delta::compute` that `commit` uses, rendered
   the same way. When clean: "nothing to commit", not an error.
2. **Ahead/behind vs. the configured upstream** (if any): how many local commits are unpushed, and
   whether the upstream has moved on, using the `Ancestry` / `SyncAction` plumbing `init.rs` already
   has. When there is no upstream: say so, don't error.

**Done when** `odm store status` reports both dimensions; is a clean no-op (exit 0, "nothing to
commit, up to date") on a clean, synced store; supports `--json`; and never mutates the store or
the orphan branch.

## Why

The store lifecycle gap is `init → mutate → ??? → commit → sync`. s01 filled `commit`; this fills
`status` — the read step that lets the operator (and CDC) **see** what `commit` would do and where
the branch stands relative to its remote, before running either. Without it, the only way to check
is `git -C .worktrees/odm status` + `git -C .worktrees/odm log --oneline origin/odm-store..HEAD`
— raw git, breaking the ODD-0022 promise. It also gives `store sync` (s03) a check the user can
run first.

## Scope

**In (`release/1.0.x` code + fixtures):**

- **The command** (`lib.rs` `StoreCommand::Status` + `store_cmd.rs`): `odm store status [--json]`.
  Read-only; never commits, never writes to the worktree, never touches the orphan branch.
- **Pending delta** (F-1/F-2): reuse `odm_store::delta::compute` — the exact same computation
  `commit` uses. Render the same odm-aware summary (`NodeDelta::summary()`). Clean worktree:
  "nothing to commit" (not an error, not silence).
- **Ahead/behind** (F-3/F-4): reuse the `Ancestry` / `SyncAction` / `sync_action()` plumbing from
  `init.rs`. The sync arm of `store init` already computes this via `worktree::rev_parse`,
  `worktree::is_ancestor`, `worktree::count_commits` — `status` calls the same path but **does not
  fetch** (read-only; `sync` fetches). Report: "N commits ahead", "upstream is ahead", "up to
  date", or "no upstream configured" — never an error for any of these states.
  *(D-1: whether `status` should fetch before comparing. Recommend **no** — `status` is read-only
  and side-effect-free; a fetch modifies `.git`'s remote-tracking refs. If the user wants a fresh
  comparison, `store sync` will fetch first. Flag this for the operator.)*
- **`--json`** (F-5): `{clean, branch, delta: NodeDelta, upstream: {ref, ahead, behind, action},
  store_root}` — or `upstream: null` when there is no upstream. Shape mirrors `commit`'s JSON
  where the two overlap (same `branch`, `delta` fields); adds the `upstream` dimension.
- **No mutation** (F-6): no commit, no fetch, no index write, no worktree write. The store and the
  orphan branch must be **bit-identical** before and after `status`. *(This is the one constraint
  that distinguishes status from commit and sync.)*

**Out:**

- **`store commit`** (s01, done) and **`store sync`** (s03, next) — this slice reads, they write.
- **`--dry-run`** — `status` is inherently dry; there is nothing to suppress.
- **A `--fetch` flag** — if the operator later wants "status that fetches first", it's a flag on
  this command or a behavior of `sync`; not built here (per D-1).
- **Colorized/rich terminal output** — keep it clean and readable (match `commit`'s `term::success`
  / `term::info` pattern), but no progress bars, spinners, or table formatting. Future polish, not
  this slice.
- Any node-model change; any history rewrite.

## Verification

Fixture, class-(a) — `TempDir` store + a real orphan-branch worktree (reuse s01's fixture setup);
runtime attested→CI. After the change:

- Dirty a node → `store status` → the delta shows (by type).
- Clean store → `store status` → "nothing to commit".
- With a remote configured and local ahead → `store status` → "N commits ahead".
- No upstream → `store status` → "no upstream" (not an error).
- Before and after: the store worktree and orphan branch are unchanged (no mutation).

CDC reproduces by staging the musl binary + the store worktree and running `odm store status` in
the cloud container (s05 established the binary-access workflow for exactly this).

## Design decisions

- **D-1 (fetch before compare?):** Recommend **no** — `status` is read-only and should have zero
  side effects; `sync` fetches. The tradeoff: the upstream comparison uses whatever the last fetch
  left behind, so it can be stale. An explicit `--fetch` flag is the future escape hatch if needed,
  but it changes the read-only contract, so it's out of scope here. Flag for operator decision.

## Rollback & findings discipline

Pure read — nothing to roll back. Amend-don't-work-around: if the store model needs a line
(unlikely — this is plumbing ODD-0022 already implies), amend ODD-0022 cited. Flag D-1 + any
deviation from the delta `commit` uses (they should be identical; any divergence is a bug, not a
feature). Five-iteration cap.

## Exit

`ledger.md` closed; CDC-verified. `odm store status` reports the store's read state natively; the
lifecycle is `init → mutate → status → commit → sync`. On close, bubble up to `../arc-plan.md`:
SL-2 done; s03 (`store sync`) next.
