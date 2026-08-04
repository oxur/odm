# Slice 03 (Store Lifecycle): `store sync`

> **Arc:** Store Lifecycle · **Branch:** `release/1.0.x` · **Status:** planned
>
> **Depends on:** s01 (`store commit`, done) · s02 (`store status`, done) ·
> arc-store-home (done) · ODD-0022 (the store discipline)
>
> **Unblocks:** SL-3 (the arc ledger's sync row) · SL-4 (no raw git — the
> composition) · CDC's ability to clone the odm store branch from GitHub ·
> the operator's ability to share the store with any other clone.

## Goal

Add `odm store sync` — a standalone verb that pushes and pulls the orphan-branch
store to/from its configured remote, honouring ODD-0022's discipline: **ff-only
pull, normal push, divergence stops** (never merge, never rebase, never rewrite
history other clones already hold). The lifecycle after this slice reads
`migrate → store status → store commit → store sync` — fully odm-native, no raw
git for the normal flow.

## Why

The store branch has never been pushed to GitHub because there is no odm verb to
do it. The operator drops to `git -C .worktrees/odm push origin odm-store` — a
workaround that violates the store-home promise (ODD-0022: *odm owns the orphan
branch; you never touch it by hand*) and that CDC cannot reproduce in the cloud
container without instructions that belong inside the tool. `store sync` closes
the last gap in the store's git lifecycle and is the gate blocking CDC from
cloning the store.

## Scope

### In

- `StoreCommand::Sync` — a new CLI subcommand (`odm store sync`).
- **Pull (ff-only):** when the upstream has commits the local branch lacks,
  fast-forward the local branch to match. Never create a merge commit.
- **Push:** when the local branch has committed work the upstream lacks, push it.
  Never force-push.
- **Divergence stops:** when both sides have commits the other lacks, report the
  situation with a clear affordance and change nothing. No merge, no rebase.
- **No-op cases:** already up-to-date → exit 0 with a clear message; no upstream
  configured → exit 0, not an error (same as `status`'s F-4).
- `--dry-run`: fetch + compare, report what *would* happen, but don't merge or
  push.
- `--json`: structured output (LLM-ergonomics contract).
- `worktree::push()` — a new function in `odm-store`, following `fetch()`'s
  pattern.

### Out

- **Merge, rebase, force-push** — any history-rewriting resolution. These
  violate ODD-0022 and are never performed.
- **Interactive conflict resolution** — `sync` stops and reports; the operator
  (or a future command) decides how to reconcile.
- **Auto-fetch on `status`** — `status` stays pure read-only (s02 D-1 stands).
- **Remote configuration** — `sync` uses `DEFAULT_REMOTE` (`origin`), the same
  remote `init`'s sync arm uses. Configuring a different remote is out of scope.
- **`store push` / `store pull` as separate verbs** — D-1 below explains why.

## Design decisions

### D-1: Single bidirectional verb

`store sync` is one verb whose behaviour depends on the ancestry state, not two
separate `push`/`pull` commands. The `SyncAction` decision table already models
all five cases: `FastForward` (pull), `LocalAhead` (push), `UpToDate` (no-op),
`Diverged` (stop), `NoUpstream` (report). A single verb that does the right
thing based on the state is simpler to teach, simpler to script, and mirrors the
mental model `init`'s sync arm established.

### D-2: Fetch always (even dry-run)

`store sync` fetches before comparing, including under `--dry-run`, for the same
reason `init::sync()` does and `status` deliberately does not: **a dry run that
reports "up to date" against stale remote-tracking refs is a lie** — the
operator acts on the preview, and a preview that diverges from the real run is
worse than none. Fetching updates remote-tracking refs only; it never moves the
local branch or writes into the worktree.

### D-3: Dirty-worktree handling (flag; CC decides)

The intended lifecycle is `commit → sync`, so the worktree is normally clean
by the time `sync` runs. But if it isn't: a **push** only sends committed work,
so a dirty worktree doesn't matter for push; a **pull** (ff-merge) could
conflict with uncommitted changes, and `git merge --ff-only` will abort if it
does. The recommendation is to check `is_clean()` and refuse the pull with
guidance ("commit your changes first — `odm store commit`, then retry") rather
than letting git's error message surface. But this is a design point, not a hard
requirement — CC may follow `init::sync()`'s precedent (which does not check
dirty) if there's a good reason. **Flag the choice in the ledger's Notes, don't
silently adopt either path.**

## Verification

Per-row in `ledger.md` (F-1…F-8). The real teeth: a **round-trip against a
bare-repo remote** (the same `TempDir` fixture pattern s02 used for the
ahead/behind rows) — push local commits, fetch from the other side, confirm the
commit is there; pull upstream commits, confirm the local branch advanced.
Divergence: push one commit locally and one on the bare repo, confirm `sync`
stops and changes nothing.

## Rollback

CLI and `odm-store` plumbing. If `sync` lands wrong: revert the commits on
`release/1.0.x`, rebuild. The store's orphan branch is never touched by this
slice's *code* changes — only by running the command against a real store, which
is an operator action outside the code rollback boundary.

## Exit

SL-3 done. The lifecycle reads `init → mutate → status → commit → sync` — four
of the four added verbs are native. SL-4 (the composition row: "no raw git")
becomes testable.
