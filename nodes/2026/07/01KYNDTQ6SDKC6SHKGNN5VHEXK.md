---
id: 01KYNDTQ6SDKC6SHKGNN5VHEXK
number: 7773604
type: slice
schema: slice/v1.1
name: Slice 04 — `odm store rename` (slice-doc / plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/slice04-store-rename/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KYNDTQ6SHYYMCPX4JN86ZJ6F
status:
  built:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  planned:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  tested:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
---
# Slice 04 — `odm store rename` (slice-doc / plan-of-record)

> **Arc:** Store Home & `init` (`arc-store-home`) · **Ledger:** SH-4 · **Realizes:** ODD-0022 §4.5.
> **The last slice of the arc.** **Slottable** — depends only on slice 02's `worktree.rs` shell-out
> module + slice 01's `[store]` locator; independent of slice 03. On close, the arc reaches its
> composition (SH-5) — SH-6 (the dogfood cutover) still rides RH C-5.

## Goal

Let an operator **rename the store's home** — the worktree directory and/or the orphan branch — with
the `[store]` locator kept in lockstep, so `odm` keeps resolving. Because the store root is *derived*
from `worktree-dir` + `branch-name` (slice 01), renaming either without updating the locator would make
the corpus unreachable; this slice makes the two move together, safely.

After this slice: `odm store rename --worktree <new>` and/or `--branch <new>` moves the worktree
(`git worktree move`) and/or renames the local branch (`git branch -m`), rewrites `[store]` to match,
and `odm list`/`check` operate at the new location — with the old path/branch gone.

## Command placement

`odm store rename`, under the `store` group slice 02 created (ODD-0023). **Distinct from `odm node
rename`** (rename a node's name) — the three-tier surface keeps them unambiguous; no collision. Bare
positional convention: `odm store rename <new>` renames **both** worktree and branch to `<new>` (they
default equal, ODD-0022 §4.1); `--worktree` / `--branch` rename them independently.

## Scope

**In:**

- **Worktree-dir rename** — `git worktree move <old_path> <new_path>` (git ≥ 2.17; not `--orphan`-gated).
  The worktree, its working tree, and git's worktree metadata all move together.
- **Local branch rename** — `git branch -m <old> <new>` (renames the branch checked out in the
  worktree; preserves its upstream config).
- **Atomic locator update** — rewrite `[store]` (`worktree_name` / `branch_name`) so resolution
  (`StoreHome::resolve`) finds the store at the new location. **Ordering/safety invariant:** apply the
  git ops **first**, write the locator **last**, and write it to match the **observed** git state — so
  the locator can never point at a store that isn't there. A git failure before the locator write leaves
  everything as it was (old locator still valid); a partial failure leaves the locator mirroring what
  actually happened, plus a clear report of what succeeded and the manual fix. The store is **always
  resolvable** after the command, success or not.
- `--dry-run` (report old→new for worktree / branch / locator, touch nothing), `--yes`, `--json`
  (report the rename: old/new worktree, branch, store_root).

**Out (deferred / not this slice):**

- **Renaming the *published* branch across a team.** `git branch -m` is **local**; it does not rename
  `origin/<branch>`. If the branch has an upstream / exists on a remote, this slice **warns** that the
  remote keeps the old name and the rename is local-only (renaming the shared DB's branch is a
  coordination event — push-new + delete-old on the remote — and is **out of scope**, YAGNI like the
  §6 shared-branch deferrals). It does not attempt remote renames.
- **Repair / `--force`** of a broken half-rename beyond the "locator mirrors reality" guarantee above.
- Any change to the node model, gates, edges, or the corpus — this only moves *where the tree is rooted*.

## §5 boundary

`git worktree move` and `git branch -m` are worktree/branch administration `gix` 0.66 doesn't expose —
they belong to the **same ratified ODD-0022 §5 exception** as `init` (setup/rename only), and live in
`worktree.rs`. Steady-state node reads/writes stay on `gix` (the L-14/L-15 boundary). This is rename,
which §5 named explicitly — no new widening.

## Safety cases to get right

- **Target already exists** — the new worktree dir or the new branch already exists → **stop, don't
  clobber**, name the collision.
- **No-op** — renaming to the current name is a clean no-op with a clear message.
- **Dirty store** — uncommitted changes in the worktree: `git worktree move` preserves the working
  tree; if git refuses (lock/dirty), surface git's reason rather than forcing.
- **Published branch** — the local-only warning above.

## Verification approach

- **Unit:** the plan builder (old→new for worktree/branch), the collision/no-op decisions, and the
  "locator mirrors observed git state" mapping — pure, no git.
- **Integration** (`assert_cmd`, scratch repo bootstrapped by `odm store init`):
  - rename **worktree only** → `.worktrees/<new>` exists, `.worktrees/odm` gone, branch unchanged,
    `[store].worktree_name` updated, `odm list`/`check` green at the new path.
  - rename **branch only** → worktree on `<new>` branch, `[store].branch_name` updated, resolution green.
  - rename **both** (bare positional) → both move, locator matches, green.
  - **collision** (target worktree/branch exists) → stops, nothing changed.
  - **published branch** (bootstrap → push to a bare remote → rename `--branch`) → warns local-only,
    the remote still has the old name.
  - `--dry-run` touches nothing; `--json` reports old/new.
- **Scope grep:** the rename git ops are in `worktree.rs` only; steady-state stays gix.

## Exit criteria

- `odm store rename` moves the worktree and/or renames the local branch and keeps `[store]` in lockstep;
  the store is **always resolvable** after the command (the locator-mirrors-reality invariant holds,
  including on partial failure).
- Collisions and no-ops handled; the published-branch case warns rather than desyncing silently;
  remote renames explicitly out of scope.
- `--dry-run`/`--json` correct; steady-state stays on gix.
- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- All SH-4 `ledger.md` rows reach a final status; bubble-up to `arc-plan.md` (SH-4; anything the
  arc-plan didn't anticipate; silent-drop diff).
- **Arc close becomes available:** with SH-1…SH-4 closed and the three-mode `init` demo (SH-5)
  reproducible, `arc-store-home/closing-report.md` can be written — noting **SH-6 (dogfood cutover)
  remains reproduced jointly with RH C-5**, so the arc closes *code-complete + compose-verified* with
  SH-6 explicitly pending the C-5 re-self-host, not silently dropped.
