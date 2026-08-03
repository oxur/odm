---
id: 01KZ321H1H9CYGR5CWEET1QRTF
number: 535447600
type: artifact
schema: artifact/v1.1
name: 'Closing Report — Slice 04 (Store Lifecycle): commit/index consistency'
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-lifecycle/slice04-commit-index-consistency/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-03
edges:
  part_of: 01KZ321DTAZRXCZBCA7XNKZ35K
---
# Closing Report — Slice 04 (Store Lifecycle): commit/index consistency

> Verified by: CC (this session). F-1…F-6 attested (real end-to-end `odm-cli` integration tests against a
> bootstrapped orphan-branch store; cross-read confirms `delta.rs` untouched). Closed 2026-08-02 on
> `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative.

**The fix.** `odm-store/src/git.rs::commit_all` builds the commit tree by walking the filesystem directly
(`write_tree`/`write_tree_excluding`) and never touched the on-disk git index at all — a deliberate choice
for `Repo::is_clean`/`Repo::tree_delta`'s race-free worktree-tree-vs-`HEAD` comparison, which needs no index
file. The side effect the cc-prompt diagnosed: because the index is never *updated* either, it is left at
its pre-commit state after every `store commit`, so a plain `git status` compares that stale index to the
new `HEAD` and reports the just-committed files as staged-deletions plus untracked — the commit itself is
correct, only the index misleads.

Added `Repo::sync_index_to_tree(&self, tree: ObjectId)`, called once at the end of `commit_all` (after
`commit_as` succeeds), using `gix`'s own `Repository::index_from_tree(&tree)` — which builds a
`gix_index::File` already pointed at the real on-disk index path — followed by
`.write(gix::index::write::Options::default())`. Reuses the exact `tree: ObjectId` `commit_all` already
computed and committed; no second tree build, no second exclude pass.

**F-1 (the acceptance anchor).** `commit_leaves_a_raw_git_status_clean` reproduces the operator's live
symptom end to end: a real bootstrapped orphan-branch store (`init::bootstrap`, the same path `odm store
init` takes), a seeded project→arc→slice, `store commit` through the real CLI dispatch, then a raw `git
status --porcelain` on the store worktree — empty.

**F-2 (no regression to commit content).** Every pre-existing `store_commit.rs` test — commit persistence
onto the orphan branch with the code branch untouched, the auto-summary message and its `-m` override, the
no-op-when-clean path, `--dry-run` writing nothing, `--json` shape on both a real and a no-op run, and
modified/removed delta classification — passes **unmodified**. This is not incidental: `sync_index_to_tree`
runs strictly after `commit_as` returns and touches only the index file, so nothing upstream of it (the
delta computation, the message, the commit object itself) could have been perturbed. The `odm-store` crate's
own suite (`gix_stage_commit_and_status`, `commit_all_honours_the_worktrees_own_gitignore`,
`commit_skips_empty_subdirectories`) is likewise green with no changes.

**F-3 (the `.odm/` exclusion, protected by construction).** `a_gitignored_odm_cache_never_lands_in_the_index`
seeds a `.odm/index` cache file (the same shape `odm check`/`list` leave behind) into a real bootstrapped
store — whose `.gitignore`, scaffolded at bootstrap, already carries ODD-0022's `/.odm/*` exclude — commits,
and asserts `git ls-files` (which reads the index, not the worktree or `HEAD`) never lists it, alongside a
clean `git status --porcelain`. This holds by construction, not merely by the test passing today:
`sync_index_to_tree` builds the index from the identical `tree` object `write_tree_excluding` already
filtered through the gitignore exclude stack, so there is no second, independently-maintained filtering pass
that could regress separately from the commit's own exclusion — a future change to the exclude logic can
only go wrong once, not twice.

**F-4 (idempotent).** `a_second_no_op_commit_leaves_the_index_clean` — SL-1's existing no-op path (a clean
worktree short-circuits before `commit_all` is even called) means a second `store commit` never re-runs the
index sync at all; the test confirms `git status --porcelain` stays empty across both commits regardless.

**F-5 (the race-free design untouched).** `git diff --stat -- crates/odm-store/src/delta.rs` is empty —
zero lines touched, not just "logically unaffected." `Repo::is_clean`/`Repo::tree_delta`, which `delta.rs`
and `store commit`'s auto-summary both call, are unmodified; `sync_index_to_tree` is a new private method
reachable only from `commit_all`, after the commit succeeds. `gix_stage_commit_and_status` (odm-store)
exercises `is_clean()` before and after two real commits and continues to pass unmodified — direct evidence
the index sync introduced no new dependency into odm's own status path.

**F-6.** `make format` (no functional diff), `make lint` (clippy `-D warnings` + rustfmt check, clean), and
`make test` (full workspace — every crate, every doctest) all green.

## Module doc comment updated

`git.rs`'s top-of-file doc comment previously stated flatly that the store "never goes through the on-disk
index," which was true of the *read* side (building what to commit) but became misleading once s04 added a
*write*-side index sync. Reworded to separate the two: odm's own status stays a race-free tree comparison
that never reads the index (unchanged); `commit_all` now writes the index once, after committing, purely
for standard git tooling's benefit.

## Scope discipline

Diff: `odm-store/src/git.rs` (the `sync_index_to_tree` method + its call site + the module/method doc
comments), `odm-cli/tests/store_commit.rs` (three new tests + two new fixture helpers). No change to
`delta.rs`, `commit_all`'s commit-content logic, the `.odm/`-exclude machinery in `write_tree_excluding`, or
any CLI-surface behavior (`store commit`'s flags, output, or JSON shape are byte-for-byte unchanged — no
test needed updating). No Cargo.toml change: the `index` gix feature was already pulled in transitively via
`excludes`, which `write_tree` already depended on for the ODD-0022 gitignore check.

## Iterations

One pass. The `gix` API (`Repository::index_from_tree` + `gix_index::File::write`) matched the cc-prompt's
own hint closely enough that no exploration/rework cycle was needed; the implementation compiled cleanly on
the first attempt and all new + pre-existing tests passed on the first run. Well inside the five-iteration
cap.

## Bubble-up → `../arc-plan.md`

- **Slice 04 done**, delivering ledger row **SL-6**: after `store commit`, a raw `git status` is clean —
  the root cause of the "phantom staged-deletion" confusion that recurred through the 2026-08-02 session is
  closed. This also removes the last practical gap in **SL-4** ("no raw git needed for the normal
  lifecycle") — SL-4's own criterion is about odm's commands not requiring raw git, but the operator (or
  any tool, CI check, or habit) *will* glance at raw `git status` from time to time, and that view now
  agrees with reality instead of contradicting it.
- **What implementing it revealed that the arc-plan didn't need to anticipate further**: the fix was as
  small and well-bounded as the cc-prompt scoped it — no design fork, no new dependency, no touch to the
  race-free status design. The one thing worth naming for future arc-plan readers: the module doc comment
  in `git.rs` had an assertion ("never goes through the on-disk index") that was true when written but
  became a stale absolute once s04 added a narrow, deliberate exception on the write side. Updated in place
  rather than left to mislead the next reader — worth a general note that "never does X" doc comments are
  worth re-reading whenever a slice touches the same file, even when the slice's own scope is narrow.
- **Silent-drop check:** all 6 ledger rows closed done, none deferred or no-op. No rows dropped.
- **s02/s03 stay paused** per the arc's current scoped-run status; this slice does not change that.
