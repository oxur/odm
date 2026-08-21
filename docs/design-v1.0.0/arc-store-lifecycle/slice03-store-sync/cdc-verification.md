# Slice 03 (Store Lifecycle) -- CDC Verification: `store sync`

> Backfill verification, 2026-08-21. This closes the formal gap called out in
> the arc close: slice 03 was implemented and CC-attested on 2026-08-03, but no
> `cdc-verification.md` was written at the time.
>
> Verification target: `release/1.0.x` at `505cba6` (current checkout). Original
> slice close commit: `bcae229` (`Close SL slice 03: store sync`).

## Verdict

**PASS.** The slice delivered `odm store sync` as specified. All eight ledger
rows are closed with reproducible evidence; no rows were dropped.

## Row Count

- Opening ledger rows: F-1 through F-8 (8 rows).
- Closing report rows addressed: F-1 through F-8.
- CDC result: 8/8 reproduced or structurally verified; 0 deferred; 0 no-op; 0
  silent drops.

## Evidence Reproduced

Command run on 2026-08-21:

```console
$ cargo test --test store_status --test store_sync
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The `store_sync` target is the direct verification for this slice. It uses real
temporary git repositories and bare-repo remotes; it does not mock push, pull, or
divergence.

## Ledger Walk

### F-1 -- Pull, fast-forward only

**Status: reproduced.** `upstream_ahead_pulls_and_fast_forwards` advances a real
bare-repo remote via an independent clone, runs `store sync --json`, and verifies
the local HEAD matches the remote branch tip. `pull_reports_in_plain_text_too`
covers the text path.

The slice also fixed a real bug discovered by this fixture: `merge_ff_only`
previously depended on unset upstream-tracking config. The implementation now
passes the upstream ref explicitly to `git merge --ff-only <ref>`.

### F-2 -- Push without force

**Status: reproduced.** `local_ahead_pushes_to_the_remote` creates local commits,
runs `store sync --json`, and verifies the bare remote's branch ref becomes the
local commit. The implementation path calls `git push <remote> <branch>` and has
no force flag.

### F-3 -- Divergence stops and changes nothing

**Status: reproduced.** `diverged_store_changes_nothing` advances both local and
remote independently, snapshots both tips, runs `store sync --json`, verifies
`action: "diverged"`, and verifies both tips are unchanged. The command reports
the affordance without merge, rebase, push, or pull.

### F-4 -- No-op cases

**Status: reproduced.** `up_to_date_is_a_no_op` covers the synced case and
`no_upstream_is_not_an_error` covers no upstream. Both exit 0 and report a clear
outcome.

### F-5 -- Dry run previews without mutating

**Status: reproduced.** The dry-run fixtures cover both pull and push previews.
Each fixture verifies the relevant side is unchanged after the dry run, then runs
a real sync and confirms the previewed action actually happens.

### F-6 -- JSON shape

**Status: reproduced.** `json_shape_is_correct` and the branch-specific fixtures
verify the JSON shape across push, pull, up-to-date, diverged, no-upstream, and
dry-run states.

### F-7 -- No model drift

**Status: structurally verified.** The slice close commit touches CLI dispatch,
`store_cmd.rs`, `store_sync.rs`, and the necessary git plumbing in
`odm-store/src/{init,worktree}.rs`. No node schema or frontmatter model changed.
No ODD-0022 amendment was needed: the implementation enforces the existing
orphan-branch, fast-forward-only, divergence-stops discipline.

### F-8 -- Clippy clean, no unsafe, covered

**Status: reproduced for coverage fixtures; source shape verified.** The
targeted fixtures passed on 2026-08-21. A source read of the slice's changed Rust
files found no `unsafe` block introduced by the sync implementation. Full
workspace lint/test was not rerun in this backfill; the original close records
`make lint` and `make test` green, and the current targeted tests still pass.

## Bubble-Up Check

Slice 03 delivered the Store Lifecycle sync capability: a native `odm store
sync` command that pushes, pulls by fast-forward only, and stops cleanly on
divergence. This made the arc composition row SL-4 testable and, with the later
set-remote slice, completed the native store git lifecycle.

Silent-drop diff: specified `store sync` capability versus delivered capability
matches. The only formal gap was the missing CDC verification file; this
backfill closes it.

## What Worked

The real bare-repo fixtures did more than prove the new command; they exercised a
dormant `init::sync` fast-forward path and exposed the implicit-upstream bug.
That is a good pattern for future git-facing slices: run the actual git side
effect, not only the pure decision table.
