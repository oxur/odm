---
number: 22
title: "The odm store home — a dedicated orphan branch in a git worktree"
author: "Duncan McGreggor"
component: All
tags: [change-me]
created: 2026-07-26
updated: 2026-07-26
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# The odm store home — a dedicated orphan branch in a git worktree

> **Drafting note.** Drafted by CDC 2026-07-26 from Duncan's design decision, as the *foundation*
> for `odm init` and the reworked `migrate`/`self-host` (RH C-5). Sequenced **before** the C-5
> work so migrate writes into the settled home. Research/design step — not yet implementation.

## 1. Decision (summary)

Give odm's store a **dedicated home**: a **git worktree** holding a **single orphan branch**
(default name `odm`), separate from the repo's code branches. All odm commands resolve the store
(`nodes/`, `odm.toml`, gate-sets) to that worktree. **`init`** creates the home; **`migrate` /
`self-host`** populate it. The odm "database" thus lives in **one canonical location, decoupled
from the code history** — which removes the cross-branch synchronisation problem entirely.

## 2. Problem

odm's store lives at `<repo-root>/nodes/` **on whatever branch is checked out**
(`StoreConfig::load` walks cwd → git-repo-root → user config; the `nodes/` root is the repo root).
Because the odm doc collection is effectively a **database**, tracking it on the working branch —
across multiple versions/projects in one repo — turns planning into a **database-synchronisation
problem across branches**: the same logical DB forks with every branch, and merges/rebases of code
drag planning state with them. In practice this has been a nightmare to manage.

## 3. Current state (what exists / what doesn't)

- **Resolution:** `odm-store/config.rs` — `odm.toml` found via cwd → repo-root (first `.git`) →
  user config; store root is the repo root; `nodes/YYYY/MM/<ULID>.md` under it.
- **Config schema:** `odm.toml` has `docs_directory`, `dev_directory`, `preserve_dustbin_structure`,
  `auto_stage_git`, `[gates.*]`, `[display]`. **No store-location field** — the nodes root is
  implicit (repo root). Adding a `[store]` section is a clean, additive change.
- **Git:** via **`gix`**, pure-Rust, **"no shelling out" (ODD-0013 Q-2)**. Today `git.rs` only does
  init/open + worktree-vs-HEAD tree comparison (the clean check). **No worktree or orphan-branch
  creation exists.**
- **No `init` command.** Subcommands today: `new/list/show/rename/retire/supersede/use/context/
  check/reconcile/orient/…`.

## 4. Design

### 4.1 The home = one orphan branch in a worktree

- **Orphan branch** (`git checkout --orphan` semantics): a branch with **no shared history** with
  the code branches. The planning DB has its own lineage — it can't be accidentally merged into a
  feature branch, and code merges never move nodes. One branch, one database, one history.
- **Worktree:** a git worktree lets the `odm` branch be **checked out simultaneously** with the
  code branch, in a separate directory. You work on code in the main tree while odm operates on the
  `odm` worktree — no branch-switching, no stashing. The **branch name is reused as the worktree
  name** (both default `odm`, both overridable).
- **One canonical branch for the whole repo's planning**, holding every version/project as *nodes*
  within it — not a branch per version. That is what dissolves the multi-version sync pain: there
  is nothing to sync because there is one DB.

### 4.2 Store resolution (config)

Add a `[store]` section to `odm.toml`, e.g.:

```toml
[store]
worktree = ".odm-worktree"   # dir of the worktree (relative to repo root, or absolute)
branch   = "odm"             # the orphan branch checked out there
```

Commands resolve the store root to the **worktree directory** instead of the repo root. The
existing cwd → repo-root → user-config search still finds `odm.toml`; the new `[store]` pointer
then redirects the *store root* to the worktree. **Open:** whether `odm.toml` lives in the main
tree (pointing at the worktree), inside the worktree, or both (§6).

### 4.3 `odm init`

`odm init` (idempotent):

1. Creates the **worktree** (dir name = branch name; overridable via `--worktree` / `--branch`
   flags or `[store]` config).
2. Creates the **orphan branch** in it (default `odm`).
3. **Writes the chosen names to `odm.toml`** (`[store]`), so later commands resolve — and any
   later rename updates this.
4. Scaffolds the initial store: `odm.toml` (gate-sets, display), an empty `nodes/`.
5. Ensures the worktree dir is **git-ignored in the code branch** so it is never committed there.

### 4.4 `migrate` / `self-host` against the home

Both write into the **worktree/orphan branch**, not the working branch. This is the reason this
ODD **precedes RH C-5**: the reworked derivation (fold + names + dates) should target the settled
store location, so migrate is built once against the final home.

### 4.5 Rename

Because the store "directory" is derived from **worktree-dir + branch-name**, renaming either must
**update `[store]` in config atomically** (and the git worktree/branch itself). A dedicated
`odm rename-store` (name TBD) command — distinct from the node `rename`.

### 4.6 Migrating existing repos (incl. odm itself)

odm's own `nodes/` currently sit on the working branch. Adopting this means **migrating odm's
corpus onto the orphan `odm` branch** — a dogfooding step. Provide a migration mode: move `nodes/`
(and `odm.toml`) into the worktree/orphan branch, write `[store]`, drop the working-branch copy.
Backward-compat for other `nodes/`-on-branch repos follows the same path.

## 5. The `gix` tension (key implementation risk)

ODD-0013 Q-2 committed odm to **`gix`, no shelling out**. But **worktree creation and orphan
branches are advanced operations** `gix` may not yet expose (its worktree support is maturing;
today odm only uses tree comparison). Options, to resolve early:

- **(a)** Confirm `gix` can `worktree add` + create an orphan branch; if so, implement natively.
- **(b)** If not, scope a **narrow exception to Q-2**: shell out to `git` **for `init`/rename only**
  (one-time setup, not a hot path), keeping all steady-state reads/writes on `gix`. Document the
  exception.
- **(c)** Defer until `gix` supports it.

**Recommendation:** verify `gix`'s worktree/orphan capability as the *first* research task; let the
answer pick (a) vs (b). `git worktree add --orphan` exists in git ≥ 2.42; older git needs a
two-step (`worktree add --detach` + `checkout --orphan`) — relevant if (b).

## 6. Open questions

- **Worktree location convention** — a repo subdir (`.odm-worktree/`, gitignored) vs a sibling
  (`../<repo>-odm/`)? Subdir is self-contained; sibling avoids any nesting oddities.
- **`odm.toml` location** — main tree (points at the worktree) vs in the worktree vs both. A
  pointer in the main tree is the most discoverable for a fresh `odm` invocation from the code tree.
- **Sharing** — the orphan branch *is* the DB, so it should be **pushable/pullable** so a team
  shares one store. Define push/pull semantics (and how a teammate's `init` attaches to an existing
  remote `odm` branch rather than creating a fresh orphan).
- **Idempotence / safety** — `init` on an existing home is a no-op; guard against clobbering an
  existing `odm` branch or worktree.
- **Interaction with the index** — the `.odm/` stat-cache (A4) lives with the store; confirm it
  rides in the worktree and doesn't collide with the worktree-dir name.

## 7. Boundaries / non-goals

- Not a general multi-repo or server sync layer — one repo, one orphan branch, git as the transport.
- Does not change the node model, ids, gates, or the `nodes/YYYY/MM/<ULID>.md` layout — only *where*
  that tree is rooted.
- `init` scaffolds; it does not import — importing is `migrate`/`self-host` (§4.4).

## 8. Version history

### v0.1 — 2026-07-26
Initial draft. Records the decision (dedicated orphan `odm` branch in a worktree as the store home),
the resolution/config change (`[store]`), the `init`/rename/migrate shape, the `gix`-vs-shell-out
tension (the key risk), and the open questions. Foundation for `odm init` and RH C-5's migrate
rework. Surfaced by: Duncan's multi-version DB-sync pain + the C-3 DATE-column investigation.
