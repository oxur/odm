# Slice 02 closing report — `git`-worktree plumbing + `odm store init` bootstrap

> **Arc:** Store Home & `init` (`arc-store-home`) · **Slice:** 02 · **Feeds:** SH-2
> **Realizes:** ODD-0022 §4.3 (bootstrap) + §5 (the ratified `git` shell-out exception)
> **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (L-1…L-15)
> **Implemented by:** CC · **Date:** 2026-07-26 · **Branch:** `sh-slice02-init-bootstrap`
> (off the slice-01 tip) · **Evidence class:** attested-by-CC (local 1.85+, **git 2.39.5**);
> cargo rows reproduce on CI.

## What shipped

`odm store init` stands up the store home end to end:

```
$ odm store init
✓ store init: /repo/.worktrees/odm on orphan branch "odm" — the store is live

$ ls -A .worktrees/odm        $ cat odm.toml
.git  config.toml  nodes/      [store]
                               worktree_base = ".worktrees"
                               worktree_name = "odm"
                               branch_name = "odm"
```

- **`odm-store/src/worktree.rs`** — the one module that runs the `git` binary. `git worktree
  add --orphan` (≥ 2.42), or the two-step fallback below it; the version → argv mapping is a
  pure function, so it is unit-tested without running anything.
- **`odm-store/src/init.rs`** — arm detection and the four bootstrap steps in ODD-0022 §4.3's
  order: worktree + orphan branch, then the `[store]` locator, then `config.toml` + empty
  `nodes/`, then the gitignore entry. The locator is written *after* the worktree exists, so a
  failed `init` never leaves an `odm.toml` pointing at nothing.
- **`odm-cli/src/store_cmd.rs`** — the `store` group with `init`: `--worktree`, `--branch`,
  `--dry-run`, `--yes`, `--json`. Born under `odm store` per ODD-0023 rather than as a
  top-level command to be renamed later. Nothing else from that reorg is here.

## Two bugs the slice found, both real

**1. An orphan branch is invisible to a refs lookup.** The prompt specifies bootstrap detection
as "no `refs/heads/<branch>` and no `refs/remotes/*/<branch>`". That is correct for every state
*except the one `init` itself creates*: `git checkout --orphan` leaves the branch **unborn** —
no commit, therefore no ref — so a refs-only check does not see the home that was just made,
and a second `init` walks straight into creating it again. Verified directly: after bootstrap,
`git for-each-ref refs/heads` lists only `main`, while `git branch --show-current` in the
worktree says `odm`.

`detect` now takes the store root as well and treats **an existing store directory as
"exists locally"**. The directory is the reliable signal during the unborn window; the ref is
reliable afterwards. Each alone has a blind spot, so both are checked.

**2. The fallback path was not equivalent to the modern one.** `git checkout --orphan`
deliberately **keeps the working tree and index** — it is built for "start a new branch from
these files". So on git < 2.42 the bootstrap left the *code branch's entire checkout* sitting
in the store, staged for its first commit. `git worktree add --orphan` creates an empty tree,
so the two paths produced different stores — precisely what a fallback must not do. Fixed with
a third step, `git rm -rf --ignore-unmatch --quiet .`, which clears index and tree together.

Caught by running the thing rather than by the tests: my integration test asserted the store
*contained* `config.toml` and `nodes/`, and never that it contained **nothing else**. The
assertion is now there — the same both-halves discipline slice 01 used for "nodes land in the
store *and* not at the repo root", which I failed to apply here first time.

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **55 binaries ok, 0 failed** (+9 unit, +8 end-to-end) |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added |

**The end-to-end tests drive the built binary against a real git**, because the point of the
slice is a subprocess: an in-process test would exercise everything except the part that can
fail. Assertions are made against **git itself** — `branch --show-current`, and `merge-base
main odm` *failing* as the proof of disjoint history — so a bug in odm's reporting cannot mask
a bug in what it created.

**L-14, the scope boundary:** `Command::new` in `crates/*/src/` is `worktree.rs` (twice) and
`odm-reconcile/src/shell.rs` (the pre-existing shell *probe*). Steady state is still entirely
`gix`; the subprocess exists at `init` and nowhere else.

**Back-compat:** odm's own repo has no `[store]`, and `odm check` is green at 60 nodes,
unchanged.

## ⚠ The git version this was built on

**Local git is 2.39.5 — below the 2.42 boundary.** So the **fallback is the path actually
exercised here**, and the modern `--orphan` arm is verified only at the argv level. That is the
weaker half of L-1/L-2, and it is the half CI must cover: **CI needs a git ≥ 2.42** for the
modern path to be executed at all. If CI also runs an older git, better still — the two arms
must produce identical stores, which is exactly the equivalence bug #2 above violated.

## The two carried items

1. **`.odm/context.json` now follows the resolved store root** (L-12) — it names node ids, so
   it belongs with the nodes, as the `.odm/` index already does. Tested both ways: present
   under the store, absent at the invocation root.
2. **`ROLLUP.md` stays at the repo/invocation root** (L-13) — decision recorded, no code
   change. It is a projection *out* of the store for a code-branch reader; putting it on the
   orphan branch would hide it from everyone working on code.

## For the arc to consider

- **The `--json` shape** is `mode` / `dry_run` / `store_root` / `branch` / `worktree` /
  `git_version`. `worktree` currently duplicates `store_root` — they are the same path, named
  twice because they are different *concepts* (the git worktree, and odm's store root). Slice
  03 should decide whether that redundancy earns its place once attach/sync add modes.
- **`--yes` is accepted and unused.** `init` asks nothing today, so there is nothing to
  confirm; the flag exists for symmetry with the other mutators and for the moment attach/sync
  needs it. Flagged rather than silently no-op.
- **`init` does not commit.** The branch stays unborn until odm's first write. That is
  deliberate — it keeps the history disjoint by construction — but it means a freshly
  bootstrapped store has an untracked `config.toml`, which slice 03's sync will have to reason
  about.

## Silent-drop diff

None. Everything the cc-prompt scoped in shipped. Everything scoped out — attach/ff-sync
(slice 03), `rename` (04), corpus migration (RH C-5), the `node` group and top-level renames
(RH C-4) — is absent.

## Ledger

- **`SH-2`** — ready to close: **attested** on this report; **reproduced** when CI runs the
  cargo rows green **on a git ≥ 2.42**.
- **L-1…L-15** all `done`; see `ledger.md` for per-row evidence.
