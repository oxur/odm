# CC Prompt — Slice 02 (Store Lifecycle): `store status`

Add `odm store status` — a **read-only** view of the store's git state: the pending node delta
(what `commit` would write) **and** ahead/behind vs. the configured upstream. The read half of s01
(`store commit`); reuses its delta computation and `init`'s remote-tracking plumbing.

> **Start condition:** on `release/1.0.x`, green. `store commit` landed in s01; `delta.rs` is
> already factored for reuse; `init.rs` has the `Ancestry`/`SyncAction`/`sync_action()` plumbing.
> This slice adds the status verb that reads both.

## Read first

1. `slice02-store-status/ledger.md` (8 rows) + `slice-doc.md` (esp. **D-1** — no-fetch decision).
2. **ODD-0022** (the store model: orphan branch, odm owns it, never rewrite history).
3. **Code — the delta (reuse):** `crates/odm-store/src/delta.rs` — `compute()` + `NodeDelta` +
   `summary()`. This is what `commit` calls; `status` calls it identically.
4. **Code — the remote tracking (reuse):** `crates/odm-store/src/init.rs` — `Ancestry` (lines
   165–178), `SyncAction` (lines 180–207), `sync_action()` (lines 218–229). The sync arm of
   `store init` calls these via `worktree::rev_parse`, `worktree::is_ancestor`,
   `worktree::count_commits`. `status` uses the same path.
5. **Code — the sibling:** `crates/odm-cli/src/store_cmd.rs` — `commit()` (lines 453–525) and
   `CommitJson` (lines 421–432). The `status` handler follows the same structure; the JSON shape
   mirrors where they overlap.

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `03-error-handling`, `05-type-design`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **The command** (F-1/F-2). `StoreCommand::Status { json: bool }` + the `store_cmd.rs` handler.
   Resolve the store home → open the repo → compute the delta via `delta::compute` → compute the
   ancestry via the `init.rs` plumbing. Report both. Clean store: "nothing to commit" (exit 0). No
   `--dry-run` flag — `status` is inherently dry.

2. **Pending delta** (F-1). Call `odm_store::delta::compute(&repo, &store_root)` — the exact same
   function `commit` calls. Render with `NodeDelta::summary()`. Any divergence from `commit`'s
   delta is a bug, not a feature — they must be identical.

3. **Ahead/behind** (F-3/F-4). Reuse `init.rs`'s plumbing:
   - Resolve the upstream ref: `{DEFAULT_REMOTE}/{branch}` (same as `init`'s sync arm).
   - `worktree::rev_parse` local and upstream.
   - `worktree::is_ancestor` + `worktree::count_commits` → `Ancestry`.
   - `sync_action(Some(ancestry))` → `SyncAction`.
   - When `rev_parse` fails for the upstream (no remote, no tracking branch): `sync_action(None)` →
     `SyncAction::NoUpstream` → report "no upstream configured", **not** an error.

   **D-1 (no fetch):** `status` does **not** call `worktree::fetch`. It reads whatever the last
   fetch left behind. This keeps `status` pure read-only (no `.git` mutation). If the operator later
   wants a "fetch then status", it's `store sync`'s job or a future `--fetch` flag.

   **Note:** the ancestry plumbing currently lives as module-level items inside `init.rs` (`Ancestry`,
   `SyncAction`, `sync_action()`) and is already `pub` + exported via `lib.rs`. The `worktree`
   module functions (`rev_parse`, `is_ancestor`, `count_commits`) are `pub(crate)`. If any of these
   need broader visibility for `store_cmd.rs` to call them for `status`, re-export appropriately —
   keep the logic in `odm-store`, not the CLI.

4. **`--json`** (F-5). Shape:
   ```json
   {
     "clean": false,
     "branch": "odm-store",
     "store_root": ".worktrees/odm",
     "delta": { "created": 2, "modified": 1, "removed": 0, "by_type": {"slice": {"created": 2, ...}, ...} },
     "upstream": {
       "ref": "origin/odm-store",
       "action": "local-ahead",
       "ahead": 3,
       "behind": 0
     }
   }
   ```
   When there is no upstream: `"upstream": null`. The `delta` field is `NodeDelta` (already
   `Serialize`). `clean` is the top-level boolean (replaces `commit`'s `committed`). Mirror
   `commit`'s JSON conventions: always-present fields (no `skip_serializing_if` on `clean`,
   `branch`, `delta`).

5. **Read-only invariant** (F-6). `status` must never call `commit_all`, `fetch`, or any write.
   The fixture asserts: `HEAD` oid before == after; no new commits; worktree files unchanged.

6. **Fixtures** (F-1…F-6): dirty-store-shows-delta; clean-is-nothing-to-commit;
   ahead-shows-count; no-upstream-is-not-an-error; json-shape-correct; read-only-invariant.
   *(For the ahead/behind fixtures: you'll need a repo with a remote-tracking branch. Consider
   a `TempDir` bare repo as "upstream" + the worktree with unpushed commits — same fixture
   pattern `init`'s tests use for the sync arm.)*

## Constraints (flag, don't silently change)

- **Read-only.** Never commit, fetch, write to the index, or write to the worktree. No side effects.
- **Same delta as `commit`.** Call `delta::compute` identically; do not compute a different delta.
- **Git ops live in `odm-store`**, not the CLI (the CLI orchestrates).
- Don't build `store sync` (s03) — but confirm the ancestry plumbing is reusable for it (it is;
  `sync` will add the fetch + the actual ff/push).
- No `unsafe`; typed errors; clippy `-D warnings` clean; cover the new code.

## Deliverables

The `store status` command + fixtures on `release/1.0.x`; any `odm-store` re-exports added;
`ledger.md` evidence per row; `closing-report.md` — the walk, D-1 disposition, the v2.0 bubble-up
(SL-2 done; s03 next). Branch: `release/1.0.x` only.

## Working agreement

Amend don't work around; flag deviations; five-iteration cap. Your `done` is proposed-done — CDC
reproduces the command + fixtures + CI, and stages the musl binary to run `odm store status` live in
the cloud container (the s05-established workflow). On close, bubble up to `../arc-plan.md`.
