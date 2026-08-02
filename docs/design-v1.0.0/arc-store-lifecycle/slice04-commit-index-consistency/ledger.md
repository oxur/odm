# Slice 04 ledger — `store commit` leaves the git index consistent

Per `LEDGER-DISCIPLINE.md` §A. Code slice (a remediation of SL-1). CC fills Evidence
at the commit each is met (`attested`); CDC reproduces. F-1 is the acceptance anchor.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | After `odm store commit`, a raw **`git status --porcelain`** on the store worktree is **empty** (index synced to the new HEAD) | fixture: `migrate`/dirty → `store commit` → assert porcelain empty | serious | 2026-08-02 finding | done | `odm-cli/tests/store_commit.rs::commit_leaves_a_raw_git_status_clean` — real bootstrapped orphan-branch store (`init::bootstrap`), seeds project→arc→slice, `store commit` via the real CLI dispatch, asserts `git status --porcelain` on the store worktree is empty. `cargo test -p odm-cli --test store_commit`: 14/14 green. | the gap SL-1's tests missed |
| F-2 | The commit **content** is unchanged — the orphan branch gains the right commit with the auto-summary; all SL-1 tests pass | run SL-1's suite; orphan branch log correct | serious | SL-1 | done | All pre-existing `odm-cli/tests/store_commit.rs` tests (commit persistence, auto-summary message, `-m` override, no-op-when-clean, `--dry-run`, `--json` shape on real + no-op runs, modified/removed delta classification, the no-`[store]`-section guard) pass **unmodified** — `sync_index_to_tree` runs strictly after `commit_as` returns, touching only the on-disk index file, never the commit object, the message, or the delta computation. `odm-store` crate suite (`gix_stage_commit_and_status` etc.) also green, incl. the pre-existing `commit_all_honours_the_worktrees_own_gitignore` and `commit_skips_empty_subdirectories` edge cases. | no behavior change to what's committed |
| F-3 | `.odm/` derived caches remain **excluded** from both the commit **and** the index (no ODD-0022 / `write_tree` gix-exclude regression) | fixture: a `.odm/index` present → absent from the commit tree and not staged after commit | correctness | ODD-0022 | done | `odm-cli/tests/store_commit.rs::a_gitignored_odm_cache_never_lands_in_the_index` — writes a `.odm/index` cache file into a real bootstrapped store (whose real `.gitignore` carries the ODD-0022 `/.odm/*` exclude), commits, asserts `git ls-files` (reads the index) does not contain `.odm/index` and `git status --porcelain` stays empty. Holds **by construction**, not just by test: `sync_index_to_tree` builds the index from the *same* `tree` object `write_tree_excluding` already filtered through the gitignore exclude stack — there is no second, independent filtering pass that could regress separately from the commit's own exclusion. | protect the s01 fix |
| F-4 | **Idempotent** — a second `store commit` on a clean store is a no-op and leaves a clean index | run twice; second is no-op, `git status` still clean | correctness | SL-1 | done | `odm-cli/tests/store_commit.rs::a_second_no_op_commit_leaves_the_index_clean` — first commit, assert clean; second `store commit` reports `nothing to commit` (SL-1's existing no-op path, which returns before ever calling `commit_all`/the index sync), assert `git status --porcelain` is still empty. | |
| F-5 | The odm-aware delta/status (`delta.rs` tree-comparison) is **unchanged** — the index sync is additive, not a new status dependency | cross-read: `delta.rs` untouched; `store commit`'s summary still derives from the tree delta | correctness | SL-5 | done | `git diff --stat -- crates/odm-store/src/delta.rs` is empty — zero lines touched. `Repo::is_clean`/`Repo::tree_delta` (the race-free worktree-tree-vs-`HEAD` comparison `delta.rs` and `store commit`'s auto-summary both call) are unmodified; `sync_index_to_tree` is a new private method called only from `commit_all`, after the commit, writing only the index file. Reproduced structurally by `gix_stage_commit_and_status` (odm-store) continuing to pass unmodified — it exercises `is_clean()` before/after two commits. | preserves the race-free design |
| F-6 | No regression: `make check` green | `make check` exit 0 | correctness | standing | done | `make format` (no functional diff) + `make lint` (clippy `-D warnings` + rustfmt check, clean) + `make test` (full workspace, all crates + doctests) all green. See Closure for the full-suite run. | |

## What Worked

- **The `gix` API had exactly the right shape already.** `Repository::index_from_tree(&tree)` builds a
  `gix_index::File` already pointed at the real on-disk index path (`git_dir().join("index")`); the only
  new call needed was `.write(gix::index::write::Options::default())`. No new dependency, no new feature
  flag — `index` was already pulled in transitively via the `excludes` feature `write_tree` already uses
  for the ODD-0022 gitignore check.
- **Reusing the same `tree: ObjectId` `commit_all` already had, rather than recomputing it,
  made F-3 hold by construction instead of by two independently-maintained filters.** Because
  `sync_index_to_tree` is handed the exact tree `commit_as` just committed — the one `write_tree_excluding`
  already filtered through the gitignore exclude stack — there is no way for the index to diverge from the
  commit's own `.odm/`-exclusion; a second, hand-rolled exclude pass over the index would have been a
  second place for that discipline to rot.
- **The pre-existing SL-1 suite was the regression net for free.** Because the index sync is strictly
  additive (new code path, appended after the commit succeeds, touching only the index file), running
  SL-1's entire test suite unmodified was sufficient evidence for F-2 — no test needed rewriting, which is
  itself a signal the change was scoped correctly.

## Closure

Closed 2026-08-02. Verified by: CC (this session) — attested for all 6 rows (real end-to-end `odm-cli`
integration tests against a bootstrapped orphan-branch store, not a synthetic fixture; cross-read confirms
`delta.rs` is byte-for-byte untouched). Full workspace `make format` + `make lint` + `make test` green (see
`closing-report.md` for the exact run). CDC reproduction is the open item. Rows: 6. Done: 6. Deferred: 0.
No-op: 0.
