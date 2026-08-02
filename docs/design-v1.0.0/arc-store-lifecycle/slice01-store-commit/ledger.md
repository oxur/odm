# Slice 01 (Store Lifecycle): `store commit`

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence strength `asserted < attested < reproduced <
> reconciled`; a `done` row reaches ≥ `reproduced`. Code/fixtures class-(a): CDC reproduces by direct read;
> runtime (`cargo`/`clippy`/a live commit) attested→CI / operator. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **`store commit` exists + persists**: `odm store commit` stages the worktree's pending node changes + commits on the **orphan branch** (not the code branch) | fixture: mutate a node → `store commit` → orphan branch has +1 commit containing the change | serious (the gap) | operator | open | | Reuse `odm-store` git plumbing (`git.rs`/`worktree.rs`); extend with a commit-current-worktree entry if needed. |
| F-2 | **Auto-summary message + `-m` override**: default message = node delta (created/modified/removed, by type); `-m` overrides | run with no `-m` → message reads the delta in odm terms; `-m "x"` → "x" | serious (odm-aware, not porcelain) | slice-doc D-1 | open | | **D-1 flag:** git-delta counts in v1; migrate-aware summary is a future handoff. |
| F-3 | **Idempotent no-op when clean**: a clean worktree → exit 0, "nothing to commit", **no empty commit** | run twice: second run makes no commit, exits 0 | serious | slice-doc | open | | Never errors on clean; never an empty commit. |
| F-4 | **`--dry-run`**: renders the delta + message, writes no commit | dry-run on a dirty store → reports, orphan branch HEAD unchanged | serious | house contract | open | | |
| F-5 | **`--json`**: `{committed, sha, branch, message, delta:{created,modified,removed,by_type}}` | `--json` parses + fields correct on a real + a no-op run | serious (LLM ergonomics) | ODD-0023 | open | | |
| F-6 | **Store discipline honored**: commit lands on the orphan branch in the worktree; the code branch is untouched; no history rewrite | direct read: the code checkout has no new commit; the orphan branch does; existing history intact | serious (ODD-0022) | ODD-0022 | open | | Confirm what's staged (node files + `config.toml`; the `.odm/` index tracked-vs-ignored) — flag. |
| F-7 | **No model drift**: CLI + `odm-store` git plumbing only; no node-schema change; ODD-0022 amended only if a line is genuinely needed | cross-read: diff scope is CLI + store; ODD cited if touched | correctness | ODD-0022 | open | | Also flag: the commit author identity (ambient git vs odm-stamped). |
| F-8 | **Clippy clean; no `unsafe`; new code covered** | clippy `-D warnings` exit 0; `! grep unsafe`; fixtures cover commit / no-op / dry-run / json | polish | CLAUDE.md | open | | |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Verified by: `<CC then CDC>`. Rows: 8. On close, bubble up to `../arc-plan.md`: SL-1
done (`store commit` native); s02 (`store status`) next.
