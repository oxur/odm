# Slice 04 cc-prompt — `store commit` leaves the git index consistent

**You are CC.** Real toolchain, with tests. This is a **remediation of SL-1**
(`store commit`) — a small, well-bounded fix. Read `slice-doc.md` + `ledger.md`
(F-1…F-6) first. F-1 is the acceptance anchor.

## The bug (confirmed live 2026-08-02)

`odm-store/src/git.rs::commit_all` writes the commit tree directly from the working
directory and creates the commit via `commit_as`, and it **never updates the git
index.** (This was a deliberate "race-free, no index file" choice for `odm store
status`, which compares the worktree tree to HEAD's tree.) The side effect: after
every `store commit`, the on-disk index is stale relative to the new HEAD, so a plain
`git status` reports the just-committed node files as `deleted` (staged) +
`untracked`. The commit is correct — HEAD has the full tree — but the index misleads,
and it spooked the operator into a false "corrupted store" diagnosis twice this
session.

## What to build

After `commit_all` writes the commit, **sync the store worktree's git index to the
new HEAD tree** so `git status` reads clean. In `gix`, build an index state from the
committed tree and persist it to the store worktree's index file (e.g.
`repo.index_from_tree(&tree_id)` → write, or the equivalent current gix API — pick
the right one and keep it to the store worktree, not the code worktree). Do this for
a real commit; under `--dry-run`, write nothing (index included).

## Constraints — do not regress

- **Commit content unchanged** (F-2): what gets committed, the auto-summary, the
  no-op-when-clean, `--dry-run`/`--json` — all SL-1 behavior stays. SL-1's tests must
  pass unmodified.
- **`.odm/` exclusion holds** (F-3): the ODD-0022 gix-exclude in `write_tree` keeps
  derived caches (`.odm/index`, `.odm/drift`) out of the commit — they must also stay
  **out of the index** after the sync.
- **`delta.rs` untouched** (F-5): the odm-aware status/summary still derives from the
  worktree-tree-vs-HEAD comparison. The index sync is *additive* — it must not become
  a new dependency of odm's own status path, and must not reintroduce the race the
  no-index design avoided.

## The test that matters (F-1)

On a real orphan-branch fixture (SL-1's discipline): `migrate`/dirty a node →
`store commit` → assert **`git status --porcelain` is empty** and the index matches
HEAD. Add it alongside the existing SL-1 fixtures. This is the assertion SL-1 lacked
(it verified the commit content, not the resulting `git status`).

## Definition of done

F-1…F-6 reach a final status; a real `store commit` leaves `git status` clean;
`make check` green. Close with the per-row ledger walk + a bubble-up to the arc (did
this deliver a clean index; anything SL-4 / ODD-0022 should note; the silent-drop
diff).
