# Slice 04 closing report — `odm store rename`

> **Arc:** Store Home & `init` (`arc-store-home`) · **Slice:** 04 (**last**) · **Feeds:** SH-4
> **Realizes:** ODD-0022 §4.5 · **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (L-1…L-12)
> **Implemented by:** CC · **Date:** 2026-07-26 · **Branch:** `sh-slice04-store-rename`
> (off `release/1.0.x` @ `0818fff`) · **Evidence class:** attested-by-CC (local 1.85+, git 2.39.5);
> cargo rows reproduce on CI.

## What shipped

```
$ odm store rename planning
✓ store rename: /repo/.worktrees/odm → /repo/.worktrees/planning, branch "odm" → "planning"
```

- `odm store rename <new>` renames both halves; `--worktree` / `--branch` do them
  independently, and a flag overrides the positional per half.
- `--dry-run`, `--yes`, `--json` on every path.
- Two new git ops in `worktree.rs` (`worktree move`, `branch -m`) plus four read-backs
  (`path_of_branch`, `branch_exists`, `current_branch`, `is_published`). §5 already names
  rename, so the boundary is unchanged: **setup-time only, steady state on `gix`** (L-11).
- The decision — no-op / collision / proceed — is a **pure function**, so every branch is tested
  without a repository.

## Three things git does that the design had to absorb

I probed each before writing the flow, rather than assuming:

1. **`git worktree move` behaves like `mv`.** Given an *existing* destination directory it moves
   the worktree *into* it (`<dest>/<basename>`) and **reports success**. So git will not protect
   against a collision — odm must pre-check, which is L-7, and it must read back where the tree
   actually landed rather than trusting the path it asked for, which is L-5.
2. **`branch -m` works on an unborn branch.** A freshly bootstrapped store has no commit, so its
   branch has no ref; the rename rewrites the symbolic HEAD and works anyway. Worth confirming,
   since the alternative would have needed a special case.
3. **`worktree move` preserves uncommitted work** (L-9), verified with a real dirty file rather
   than assumed.

## The bug this slice found — a green `check` over an invisible corpus

The first cut wrote the locator only after *both* git ops succeeded. So when the worktree move
succeeded and the branch rename then failed — `git branch -m odm bad..name` — the result was:

```
worktree actually at:  .worktrees/moved
locator points at:     worktree_name = "odm"      ← a path that no longer exists
odm check:             ✓ check: ok (0 node(s), no problems)
```

**That is the worst failure this slice can produce.** Resolution finds nothing at the stale
path, self-heals an empty store there, and reports success — so the corpus is invisible *and*
odm says everything is fine. Silence, not an error.

Fixed by holding the branch result, writing the locator from the **observed** state either way,
and only then reporting the failure with what did succeed:

```
Error: the worktree moved to …/.worktrees/moved and the locator now points there, so the store
is intact and resolvable; the branch rename failed: … The branch is still "odm" — re-run
`odm store rename --branch <name>` with a valid name
```

Same scenario now: locator says `moved`, `check` reports **1 node**. Pinned by
`a_partial_failure_still_leaves_the_store_resolvable`.

This is exactly what L-5 was written to prevent, and the collision pre-check alone did not cover
it — the pre-check catches *predictable* conflicts, while this was git refusing mid-flight for a
reason no pre-check would anticipate. The ordering rule the prompt specified ("write the locator
LAST, mirroring the observed state") is what makes it recoverable; I had implemented the
"mirroring observed" half but not the "even on failure" half.

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **57 binaries ok, 0 failed** (+10 unit, +14 end-to-end) |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added |

**Every rename test ends by asserting the corpus is still reachable** — `list` shows the node,
`check` is green at the new path. Asserting the git move alone would have passed while the store
was unfindable, which is precisely the bug above.

**Back-compat:** odm's own repo has no `[store]`; `check` green at 60 nodes, unchanged. A repo
without a `[store]` gets an explanation pointing at `odm store init`, not an obscure failure.

## For the arc to consider

- **`--yes` is accepted and unused** on every `store` subcommand now (carried from slices 02–03).
  Nothing prompts, so nothing needs confirming; it is waiting for an operation that does.
- **Renaming a published branch is local-only, by design** (L-6). The warning says so and names
  why: reconciling a shared branch is push-new / delete-old / everyone re-points — a coordination
  event, not a flag.
- **`worktree_base` is not renameable.** Only the leaf directory and the branch are. Moving the
  base (`.worktrees` → something else) would be a third axis; no one has asked, and ODD-0022 §4.5
  speaks only of worktree-dir and branch-name.

## Silent-drop diff

None. Everything scoped in shipped. Scoped out and absent: remote/published branch renaming,
`--force`/repair beyond the locator-mirrors-reality guarantee, and any node-model change.

## Ledger

- **`SH-4`** — ready to close: **attested** on this report; **reproduced** when CI runs the cargo
  rows green.
- **L-1…L-12** all `done`.
- With SH-1…SH-4 closed and **SH-5** reproducible, the arc reaches its composition — see
  `arc-store-home/closing-report.md`. **SH-6 (dogfood cutover) remains open, pending RH C-5.**
