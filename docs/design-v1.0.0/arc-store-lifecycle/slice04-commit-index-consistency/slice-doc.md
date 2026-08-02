# Slice 04 — `store commit` leaves the git index consistent

<!-- Name/title carries no document-role metadata, per ODD-0013 §2.1. -->

**Arc:** Store Lifecycle · **Kind:** code · **A remediation of SL-1** (runs now,
operator call 2026-08-02; s02/s03 stay paused) · **Opened:** 2026-08-02 ·
**Origin:** the 2026-08-02 store-commit finding

## Goal

After `odm store commit`, the git **index** must reflect the new HEAD, so a plain
`git status` on the store worktree reads **clean**. Today it does not — and that gap
is the root cause of the "phantom staged-deletion" confusion that recurred through
this session (a store that looked corrupted after every commit but never was).

## The bug (root cause — confirmed live 2026-08-02)

`odm-store/src/git.rs::commit_all` builds the commit tree directly from the working
directory and writes the commit object with `commit_as`, and — by design, for a
race-free `odm store status` — **never goes through the on-disk index.** But it also
never *updates* the index after committing. So the index is left at the pre-commit
state, and a raw `git status` compares that stale index to the new HEAD and reports
the **just-committed** node files as `deleted` (staged) + `untracked`. The commit
itself is correct (HEAD has everything); only the index is stale.

Verified on the live store: `store commit` produced `24f1037` with the full tree,
yet `git status` showed 25 "staged deletions" of files present in HEAD and the
worktree, with `git diff --cached` = the entire committed content. The operator
cleared it with `git reset` (index → HEAD). This slice makes that manual step
unnecessary.

## Scope — in

- After `commit_all` writes the commit, **sync the git index to the new HEAD tree**
  (gix: build the index state from the committed tree and persist it to the store
  worktree's index file), so `git status` is clean immediately after `store commit`.
- A test that asserts a **clean `git status --porcelain`** (empty) after
  `store commit` — the exact assertion SL-1's tests lacked (they checked the commit
  *content*, not the resulting working-tree/index state).

## Scope — out

- **No change to the commit content** — SL-1's behavior (what gets committed, the
  auto-summary, the no-op-when-clean, `--dry-run`/`--json`) is unchanged; its tests
  must still pass.
- **No change to the odm-aware delta/status** (`delta.rs`, the worktree-tree-vs-HEAD
  comparison). The index sync is *additive* — it keeps the index consistent for
  standard git tooling; it does **not** reintroduce an index dependency into odm's
  own race-free status computation. The `.odm/`-exclude (ODD-0022) must not regress.
- Not retroactively repairing an already-stale index from a prior commit (the
  operator's one-time `git reset` handles the current one). This is forward-looking.

## Verification approach

Real-orphan-branch fixture (matching SL-1's discipline): `migrate`/dirty a node →
`store commit` → assert `git status --porcelain` is empty and the index matches HEAD;
confirm the commit content is right (SL-1 tests green); confirm a `.odm/` cache is
still excluded from both the commit and the index; confirm a second `store commit`
on a clean store is a no-op leaving a clean index.

## Exit criteria

`git status` is clean immediately after `store commit`; the commit content and the
odm-aware delta are unchanged; `.odm/` exclusion holds; `make check` green. After
this lands, the SL-4 "no raw git needed" promise actually holds when a user *does*
glance at raw git — which they will.
