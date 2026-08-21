# Slice 02 (Store Lifecycle) -- CDC Verification: `store status`

> Backfill verification, 2026-08-21. This closes the formal gap called out in
> the arc close: slice 02 was implemented and CC-attested on 2026-08-03, but no
> `cdc-verification.md` was written at the time.
>
> Verification target: `release/1.0.x` at `505cba6` (current checkout). Original
> slice close commit: `a623635` (`Close SL slice 02: store status`).

## Verdict

**PASS.** The slice delivered `odm store status` as specified. All eight ledger
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

The `store_status` target is the direct verification for this slice. The
`store_sync` target was run alongside it because `status` and `sync` share the
same ancestry/action plumbing, and the Store Lifecycle close depends on the two
commands agreeing about upstream state.

## Ledger Walk

### F-1 -- Status exists and reports the pending delta

**Status: reproduced.** `store_status.rs` includes
`dirty_store_shows_the_pending_delta` and
`dirty_store_matches_commits_own_delta_computation`. The second fixture compares
`store status --json` against `store commit --dry-run --json` on the same dirty
worktree, proving `status` reports the same node delta `commit` would write.

### F-2 -- Clean store reports "nothing to commit"

**Status: reproduced.** `clean_store_reports_nothing_to_commit` verifies
`clean: true` in JSON, and `clean_store_reports_in_plain_text_too` verifies the
plain-text affordance.

### F-3 -- Ahead/behind reported

**Status: reproduced with a structural cross-check.** The fixture
`local_ahead_of_upstream_reports_the_count` creates a real bare-repo upstream,
pushes/fetches it, adds two unpushed local commits, and verifies
`action: "local-ahead"`, `ahead: 2`, and `behind: 0`. The synced case is covered
by `synced_store_reports_up_to_date`.

The broader upstream-state classification is also structurally verified in
`crates/odm-cli/src/store_cmd.rs`: `status()` computes both `ahead` and
`behind`, builds the same `init::Ancestry` used by `sync`, and passes it through
`init::sync_action`. The upstream-ahead action path is exercised by the sibling
`store_sync` fixture `upstream_ahead_pulls_and_fast_forwards`, which uses the
same ancestry/action plumbing and a real bare-repo remote.

### F-4 -- No upstream is graceful

**Status: reproduced.** `no_upstream_is_not_an_error` verifies `upstream: null`
and exit 0; `no_upstream_reports_in_plain_text_too` verifies the plain-text
message.

### F-5 -- JSON shape

**Status: reproduced.** `json_shape_is_correct` parses the JSON shape and
asserts `clean`, `branch`, `store_root`, `delta`, and `upstream`. Other fixtures
exercise the upstream object shape in local-ahead and synced states.

### F-6 -- Read-only, no mutation

**Status: reproduced.** `status_never_mutates_the_store_or_the_orphan_branch`
snapshots the store HEAD, `git status --porcelain`, and node path listing before
running `store status --json`, then verifies all three are unchanged. It also
reruns `status` and checks byte-identical JSON output, which catches hidden fetch
or mutation side effects.

### F-7 -- No model drift

**Status: structurally verified.** The slice close commit touches only
`odm-cli/src/lib.rs`, `odm-cli/src/store_cmd.rs`,
`odm-cli/tests/store_status.rs`, and Store Lifecycle planning artifacts. No
node schema, frontmatter model, or ODD-0022 model change was introduced.

### F-8 -- Clippy clean, no unsafe, covered

**Status: reproduced for coverage fixtures; source shape verified.** The
targeted fixtures passed on 2026-08-21. A source read of the slice's changed Rust
files found no `unsafe` block introduced by the status implementation. Full
workspace lint/test was not rerun in this backfill; the original close records
`make lint` and `make test` green, and the current targeted tests still pass.

## Bubble-Up Check

Slice 02 delivered the read half of the Store Lifecycle capability: a native
`odm store status` command that reports pending node deltas and upstream
position without mutating the store. The arc-plan already incorporated this
slice in SL-2; no new scope or sequencing change is required.

Silent-drop diff: specified `store status` capability versus delivered
capability matches. The only formal gap was the missing CDC verification file;
this backfill closes it.

## What Worked

The strongest fixture is the status-vs-commit delta comparison. It prevents
`status` from becoming a parallel, drift-prone implementation of what `commit`
would write, and it is exactly the kind of cross-command invariant this arc
needed.
