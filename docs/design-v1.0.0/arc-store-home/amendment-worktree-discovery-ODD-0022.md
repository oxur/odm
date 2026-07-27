# Amendment to ODD-0022 — worktree-general store discovery & project scoping

> **Amends** ODD-0022 (*The odm store home*, Accepted v1.0) · **Arc:** arc-store-home (reopened for
> its expanded feature set) · **Target:** `release/1.0.x` · **Author:** CDC, from Duncan's design
> (2026-07-27) · **Status:** draft for operator review.
>
> **What this changes, in one line:** ODD-0022 gave the store *one canonical home*; this amendment
> makes that home **reachable from any worktree** and adds the **project-scoping** model that the
> multi-project / per-line-worktree layout needs. It **expands** §2/§4.1 (which already declared the
> one-branch-many-projects vision) and **supersedes the resolution mechanics of §4.2** (single-checkout
> assumption). §4.3–§4.6, §5 (the ratified `git` shell-out), and §7 stand unchanged except where noted.

## 1. Why the original resolution is incomplete

ODD-0022 §4.2 resolves the store root to `<repo>/<worktree_base>/<worktree_name>`, where `<repo>` is
"the repo root — first ancestor with a `.git` — that holds the found `odm.toml`." That is correct for a
**single checkout**. It breaks the moment the store must be reached from a **sibling worktree**, which is
exactly the layout §2/§4.1 promised ("multiple versions/projects in one repo," "one canonical branch…not
a branch per version").

Verified against the code and real git (2026-07-27):

- `StoreHome::resolve` (`odm-store/src/home.rs`) finds `odm.toml` at cwd first, else at the nearest
  ancestor with a `.git`, and anchors `store_root` at **that directory**.
- A **linked worktree** carries its own committed `odm.toml`, and its `.git` is a gitlink *file* that
  `config::repo_root`'s `.exists()` check still stops at. So from `.worktrees/1.1.x/`, resolution yields
  `.worktrees/1.1.x/.worktrees/odm` — a **nested path that does not exist** — instead of the one shared
  store at the top.
- Consequence today: the store resolves **only** from the primary checkout. With `main` at the top and
  all work in sibling worktrees (the adopted layout), *nothing* resolves the corpus. This is the gap the
  reopened arc closes.

The fix is to stop anchoring on "wherever an `odm.toml` was found" and anchor on **what git already knows
about the repository** — the common dir — so worktree placement becomes irrelevant to discovery.

## 2. Amended design

### 2.1 Discovery anchor — the git common dir (supersedes §4.2's anchor)

Store discovery is a **two-step** resolution:

**Step A — find the repo and the store (branch-agnostic, cwd-agnostic).**

1. From cwd, ask git for the common dir: `git rev-parse --git-common-dir` → the shared `.git`. Its parent
   is the **primary worktree root** (the top-level checkout). This resolves identically from the primary,
   from any linked worktree, and from the store worktree — verified reachable even from a worktree created
   *outside* the repo tree (`/tmp/...`). *(If cwd is not inside any git repo, `--repo=<path>` is required
   and supplies the primary explicitly.)*
2. Read the **primary worktree's** `odm.toml` `[store]` (defaults `worktree_base=.worktrees`,
   `worktree_name=odm`) → `store_root = <primary>/<worktree_base>/<worktree_name>` (default
   `<primary>/.worktrees/odm`).
3. Load the store's `config.toml` — the existing two-stage load, now anchored at the primary, so it is the
   same store from everywhere.

This keeps the ratified boundary: `--git-common-dir` and `git worktree list` go through the **`git`
shell-out already sanctioned by §5** for worktree operations; all steady-state reads/writes stay on `gix`.
gix 0.66 still cannot create worktrees, but reading the common dir is a plain `rev-parse` — a shell-out in
the same narrow, documented class as `init`/rename.

### 2.2 `top_level_branch` — assertion, not discovery

Add `top_level_branch` to the primary's `odm.toml` (default `main`). Its job is **orientation and
validation, not discovery** — discovery uses git's *primary worktree* (structural, branch-agnostic).
`top_level_branch` lets odm (a) warn when the primary is checked out to something other than the declared
trunk, and (b) give the propagation queue a named integration target. Making it configurable is what keeps
the model general for repos whose trunk is not `main`.

### 2.3 Project scoping — 1:1 project ↔ branch ↔ worktree

Per the operator's model, `release/1.0.x`, `release/1.1.x`, and `project/bitubardos` are **projects of
equal standing**, each on its own branch+worktree, all descending from `main` as the common trunk. Git
*ancestry* (bitubardos ← 1.1.x ← main) is orthogonal to project *standing* (all peers). The mapping is
**1:1**: one project node ↔ one branch ↔ one worktree.

**Step B — determine project scope.**

1. Map cwd → its worktree's branch via `git worktree list --porcelain` (the authoritative path↔branch map;
   dir names need not match branch names — `1.0.x`↔`release/1.0.x`, `bitubardos`↔`project/bitubardos`).
2. Map branch → project: the project node whose **`branch` attribute** equals that branch.
3. Default scope by where you stand:
   - **In a project worktree** → scope defaults to that project. Override with `--project=<name|path>`.
   - **In the primary (main) or the store worktree** → there is no single project; **read/query commands
     default to the whole-corpus aggregate**, and **mutators require `--project`**. (This is the one
     refinement to the operator's draft rule: "from main, `--project` required" is right for `node new`,
     but too strict for `orient`/`list`/`rollup`, where the cross-project view is the point of the hub.)
   - **Outside the repo** → `--repo` (Step A) *and* `--project` both required for project-scoped ops.

`--project` resolves **name-first** (via the registry below), falling back to a path. `--repo` overrides
the primary. Both are the explicit escape hatches for the non-default cases.

### 2.4 The project registry — derived, with an explicit override

Rather than a per-worktree `[projects.*]` section (which would re-introduce the N-copies-one-truth drift
the store-home split killed), the registry is **derived and single-sourced**:

- Each **project node** carries a `branch` attribute (its line's branch).
- `git worktree list --porcelain` supplies the live branch↔path map.
- odm **reconciles** the two — a natural `odm-reconcile` probe: green iff every active project worktree
  maps to exactly one project node and back. Drift (a project with no worktree, or a worktree with no
  project) is a reported finding, not a silent gap.

An explicit `[projects.<name>]` block (in the **store's `config.toml`**, the metadata authority — *not*
per-worktree `odm.toml`) is supported only as an **override** for cases the derivation can't infer.

### 2.5 Store branch default: `odm` → `odm/store`

Decouple the two names that ODD-0022 §4.1 kept equal by default: **worktree dir stays `odm`**, the
**branch becomes `odm/store`** (namespaced, like `release/…` and `project/…`). In code this splits the
single shared `default_store_name()` into `default_worktree_name() = "odm"` and
`default_branch_name() = "odm/store"`. Resolution uses `worktree_name` for the *directory*
(`.worktrees/odm`) and `branch_name` only for git operations, so the store path is unchanged. The live
change is a `git branch -m odm odm/store` plus a locator/​`config.toml` update — check whether the existing
`store rename` (ODD-0022 §4.5) covers a branch-only rename before leaning on it.

### 2.6 The main-anchor commit

For the primary (currently `main`, frozen on 0.3.5) to serve as the anchor, it needs a small **structural**
commit — code stays 0.3.5:

- `.worktrees/` added to `main`'s `.gitignore` (0.3.5 predates the rule the release lines carry; without
  it the worktree home shows as untracked at the top — already observed).
- optionally `top_level_branch` and a minimal `[store]` in `main`'s `odm.toml`, though the defaults make
  both unnecessary for resolution to work.

### 2.7 Concurrency (named, minimally scoped for now)

The store is a **single checkout of one branch** (`odm/store`). Concurrent writers from two worktrees
(operator in `1.1.x`, an agent in `bitubardos`) race on the same working tree + index. For this arc the
position is **serialize corpus writes**; a real locking/queueing story is deferred to a later arc (and is
adjacent to the propagation queue). Named here so it is disclosed, not discovered.

## 3. Interaction with ODD-0022's open items

- §6 "Interaction with the index" — the `.odm/` stat-cache follows the store, so it is anchored the same
  way (primary → `<store_root>/.odm`), reachable identically from every worktree. No separate discovery.
- §4.3–§4.6 (`init`/attach/sync, migrate, rename, adopt) are unchanged in shape; they gain the
  `worktree_name`≠`branch_name` split (§2.5) and, where they resolve the store, the common-dir anchor
  (§2.1) in place of the found-`odm.toml` anchor.

## 4. Boundaries / non-goals (unchanged from §7, restated for this amendment)

- Does **not** change the node model, ids, gates, or the `nodes/YYYY/MM/<ULID>.md` layout — only *how the
  one store is discovered* and *how a project scope is chosen*.
- Does **not** introduce multiple stores — the decision stands: **one shared corpus** for all projects
  (multi-store is explicitly deferred).
- Does **not** build the propagation queue — that is a **separate `1.0.x` arc** that consumes this arc's
  branch-map substrate (project nodes + `branch` attribute + inter-project edges).

## 5. Proposed slice mapping (for the reopened arc-plan; sizing is the arc-plan's call)

- **slice05** — the `odm/store` rename + the `worktree_name`/`branch_name` default split (§2.5).
- **slice06** — common-dir discovery (§2.1) + `top_level_branch` (§2.2) + the main-anchor (§2.6).
- **slice07** — project scoping (§2.3): `git worktree list` mapping, the `--project`/`--repo` surface, the
  hub read-vs-mutate default, and the derived registry + reconcile probe (§2.4).

## 6. Version history

### v0.1 — 2026-07-27
Initial amendment draft. Anchors store discovery on the git common dir (supersedes ODD-0022 §4.2's
single-checkout resolution), adds `top_level_branch`, the 1:1 project↔branch scoping model with the
read-vs-mutate hub asymmetry, the derived project registry (reconcile-able, override in store
`config.toml`), the `odm`→`odm/store` branch-default split, the main-anchor commit, and a named
concurrency position. Surfaced by: the multi-project / per-line-worktree setup (2026-07-27) and the
verified failure of §4.2 resolution from a linked worktree.
