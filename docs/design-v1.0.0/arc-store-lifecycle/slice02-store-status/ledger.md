# Slice 02 (Store Lifecycle): `store status`

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence strength `asserted < attested < reproduced <
> reconciled`; a `done` row reaches ≥ `reproduced`. Code/fixtures class-(a): CDC reproduces by direct read;
> runtime (`cargo`/`clippy`/a live `store status` run) attested→CI / operator. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **`store status` exists + reports the pending delta**: `odm store status` shows the node delta (created/modified/removed, by type) in odm terms — **the same `delta::compute` that `commit` uses** | fixture: mutate a node → `store status` → the delta shows (by type, matching `commit`'s summary format) | serious (the read half) | arc-plan SL-2 | open | | Reuses `odm_store::delta::compute` + `NodeDelta::summary()` — any divergence from `commit`'s delta is a bug. |
| F-2 | **Clean store → "nothing to commit"**: a clean worktree reports "nothing to commit" (exit 0, not an error, not silence) | run on a clean store → clear message, exit 0 | serious | slice-doc | open | | Matches `commit`'s clean-path behavior. |
| F-3 | **Ahead/behind reported**: when the store branch has unpushed local commits → "N commits ahead"; when upstream has moved on → "upstream is ahead"; when synced → "up to date" | fixture or manual: local branch with unpushed commits → `store status` → "N commits ahead" | serious (the new dimension over commit) | arc-plan SL-2 | open | | Reuses `init.rs`'s `Ancestry`/`SyncAction`/`sync_action()` plumbing via `worktree::rev_parse`, `is_ancestor`, `count_commits` — but does NOT fetch (D-1). |
| F-4 | **No upstream → graceful**: when no remote-tracking branch is configured, reports "no upstream configured" (not an error) | run on a store with no remote → clear message, exit 0 | serious | slice-doc | open | | The `SyncAction::NoUpstream` arm. |
| F-5 | **`--json`**: `{clean, branch, delta, upstream: {ref, ahead, behind, action} | null, store_root}` | `--json` parses + fields correct on dirty, clean, ahead, and no-upstream runs | serious (LLM ergonomics) | ODD-0023 | open | | Shape mirrors `commit`'s JSON where they overlap (same `branch`, `delta`); adds `upstream`. |
| F-6 | **Read-only — no mutation**: the store worktree and orphan branch are **bit-identical** before and after `status` — no commit, no fetch, no index write, no worktree write | fixture: snapshot the store state (e.g. `HEAD` oid + worktree file hashes) before and after `store status` → identical | serious (the distinguishing constraint) | slice-doc | open | | Enforced by construction: `status` calls `delta::compute` + read-only ancestry queries; never calls `commit_all`, `fetch`, or any write method. Verified by asserting no new commits + no file changes after the call. |
| F-7 | **No model drift**: CLI + `odm-store` plumbing only; no node-schema change; ODD-0022 amended only if a line is genuinely needed | cross-read: diff scope is CLI + store; ODD cited if touched | correctness | ODD-0022 | open | | |
| F-8 | **Clippy clean; no `unsafe`; new code covered** | clippy `-D warnings` exit 0; `! grep unsafe`; fixtures cover dirty/clean/ahead/no-upstream/json | polish | CLAUDE.md | open | | |
