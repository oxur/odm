# Slice 01 (Store Lifecycle) — `store commit`

> Refs: `../arc-plan.md` (SL-1) · **ODD-0022** (the store model: orphan branch, odm owns it) ·
> `crates/odm-store/src/{git,worktree,init}.rs` (the git plumbing `init` already uses to commit) ·
> `crates/odm-cli/src/{lib.rs,store_cmd.rs}` (the `store` group). `depends_on:` arc-store-home.
>
> **The near-term need:** the arc-migration-fidelity freeze mutates the store worktree with no odm verb to
> persist it. This slice adds that verb.

## Goal

Add `odm store commit` — persist the store worktree's pending node changes as a commit on the orphan
branch, odm-aware and idempotent, so the operator never drops to raw git. **Done when** `odm store commit`
stages and commits the pending node changes on the store branch with an **auto-summary message** (node
delta: N created / modified / removed, by type) overridable by `-m`; is a **clean no-op** (exit 0, clear
message) when there's nothing to commit; supports **`--dry-run`** (show the delta + the message, write
nothing) and **`--json`**; and reuses `odm-store`'s existing git plumbing rather than shelling out anew.

## Why

`store init` makes the *first* commit when it bootstraps the orphan branch, but nothing commits **after**:
`migrate`/`node` ops write files into `.worktrees/odm` and stop. Persisting them means raw
`git -C .worktrees/odm add -A && git commit` — which breaks the ODD-0022 promise that odm owns the orphan
branch and the operator never touches it by hand. `store commit` closes the `init → mutate → ??? ` gap and
makes the freeze flow (`migrate → store status → store commit`) fully odm-native.

## Scope

**In (`release/1.0.x` code + fixtures):**

- **The command** (`lib.rs` `StoreCommand::Commit` + `store_cmd.rs`): `odm store commit [-m <msg>]
  [--dry-run] [--json]`. Stages the store worktree's pending changes and commits on the **orphan branch**
  (never the code branch). Reuse `odm-store`'s git plumbing (`git.rs`/`worktree.rs` already commit for
  `init`) — extend it with a "commit the current worktree state" entry point if one isn't exposed.
- **Auto-summary message** (D-1): derived from the pending **node delta** — count added / modified / removed
  node files and classify by `type` (e.g. `store: +2 slice, ~3 arc, ~2 design, -1 project (retired)`), so
  the message reads in odm terms, not `git status` porcelain. `-m "<msg>"` overrides. *(D-1: git-delta node
  counts in v1; a richer migrate-aware summary — "reconciled / minted / collapsed" — would need `migrate`
  to hand off a pending-summary; recommend the git-delta version now, flag the handoff as a future
  enhancement.)*
- **Idempotent no-op** (F-3): a clean worktree → exit 0 with "nothing to commit" (never an error, never an
  empty commit).
- **`--dry-run`** (F-4): render the delta + the message, write nothing.
- **`--json`** (F-5): `{committed: bool, sha, branch, message, delta: {created, modified, removed, by_type}}`.
- Honor the store discipline: commit lands on the orphan branch in the worktree; the code branch is
  untouched (the worktree is gitignored from the code checkout — ODD-0022 / `init`'s `GITIGNORE_ENTRY`).

**Out:**

- **`store status`** (s02) and **`store sync`** (s03) — this arc's later slices. *(status shares s01's
  delta computation — factor it reusably, but don't build status here.)*
- **`migrate --commit`** convenience + a migrate→commit summary handoff — future (keep migrate mutating,
  commit persisting; the freeze protocol wants the verify gate *between* them).
- **`auto_stage_git`** — stays dormant (a 0.3.x semantic); `commit` is explicit.
- Any node-model change; any history rewrite.

## Verification

Fixture, class-(a) — `TempDir` store + a real orphan-branch worktree; runtime attested→CI. After the change:
a fixture mutates a node then `store commit` → the orphan branch gains exactly one commit whose message
carries the node delta; a second `store commit` is a no-op (exit 0, "nothing to commit", no new commit);
`--dry-run` writes no commit but reports the delta; `-m` overrides the message; `--json` shape validates.
CDC reproduces by reading the command + fixtures, and (operator) a real `store commit` of the frozen
migration store lands one commit on `odm`.

## Rollback & findings discipline

Fixture + a live commit are both reversible (`git reset` on the orphan branch). Amend-don't-work-around: if
the store model needs a line (unlikely — this is plumbing ODD-0022 already implies), amend ODD-0022 cited.
Flag D-1 (auto-summary content) + any decision on what's staged (node files + `config.toml`; confirm the
`.odm/` index's tracked/ignored status) + the commit author identity. Five-iteration cap.

## Exit

`ledger.md` closed; CDC-verified. `odm store commit` persists the store natively; the freeze-commit step is
no longer raw git. On close, bubble up to `../arc-plan.md`: SL-1 done; s02 (`store status`) next.
