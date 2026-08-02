---
id: 01KYP5H4YXAJAKPGHMNA7K6WNX
number: 22
type: design
schema: design/v1.1
name: The odm store home — a dedicated orphan branch in a git worktree
created: 2026-07-26
updated: 2026-07-31
tags:
- change-me
component: All
author: Duncan McGreggor
version: '1.0'
origin: planned
reserved: false
source:
  paths:
  - docs/design/04-accepted/0022-the-odm-store-home-a-dedicated-orphan-branch-in-a-git-worktree.md
  class: odd
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-01
status:
  accepted:
    reached: 2026-07-26
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-26
  draft:
    reached: 2026-07-26
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-26
  revised:
    reached: 2026-07-26
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-26
  under-review:
    reached: 2026-07-26
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-26
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
worktree_base = ".worktrees"   # base dir for all worktrees (the common convention), relative to repo root
worktree_name = "odm"          # odm's worktree subdir -> .worktrees/odm
branch_name   = "odm"          # the orphan branch; kept equal to worktree_name by default to avoid confusion
```

Commands resolve the store root to `<repo>/<worktree_base>/<worktree_name>` (default **`.worktrees/odm`**) instead of the repo root. The
existing cwd → repo-root → user-config search still finds `odm.toml`; the new `[store]` pointer
then redirects the *store root* to the worktree. **Two configs (resolved).** The code-branch **`odm.toml`** is the *locator* — committed, minimal, just `[store]` (where the home is). The *operational* config — gate-sets, display, `docs_directory`, author — lives **inside the store** as **`config.toml`** (not another `odm.toml`: redundant given the branch), so it is **versioned and shared with the data it governs** and cannot drift from it. Load is two-stage: find `odm.toml` -> `[store]` -> resolve `.worktrees/odm` -> load that store's `config.toml`.

**The `[legacy]` sub-table (v1.1, arc-migration-fidelity s13/s14).** A pre-split (odm 0.3.x)
`odm.toml` carries `docs_directory`/`dev_directory` (and a few settings the v1.0 crates don't yet
consume) inline, with no `[store]` section at all. `odm store init`'s bootstrap arm ports those
keys forward into the fresh store's `config.toml`, under `[legacy]`, as a preserved historical
record — never acted on directly, just carried so a later migration pass has it rather than losing
or re-deriving it. `migrate` reads `docs_directory`/`dev_directory` from the **top-level** key
first, falling back to the same key under `[legacy]` — so a store whose only source for either is
the ported-forward block still resolves correctly without the operator re-typing the modern key.
`[legacy].additional_paths` is a distinct, **live** key (not a ported-forward record): the operator
names extra un-typed legacy directories (research notes, brainstorm sessions, chat logs) on
`migrate --all`'s command line, and they are unioned in, sorted, deduplicated, and written back
here so a forgotten re-pass still covers them. **This lives in the operational config the code
actually reads** (`config.toml` inside the store, once one exists — `StoreHome::resolve`'s
`operational_path`) — a `[legacy]` block sitting only in the code-branch `odm.toml` (the locator)
is invisible to `migrate` the moment a store `config.toml` exists, since the locator stops being
the operational file at that point. A v1.0+ config that wants `additional_paths` re-runs to stay
idempotent must carry `[legacy]` in the file the code resolves, not the locator.

### 4.3 `odm init`

`odm init` — **three modes, chosen by detection:** **bootstrap** (no `odm` branch anywhere → create the orphan, below), **attach** (a remote `odm` branch exists → check it out into the worktree, *do not* re-orphan — see §6 sharing), and **sync** (a store already exists **locally** → `fetch` + `merge --ff-only` from `origin/odm`: a clean fast-forward freshens the store; a **divergence** (unpushed local commits + advanced upstream) or a missing upstream **warns and stops**, leaving merge-vs-rebase to the operator — never auto-rebasing a shared branch; `--force`/`--overwrite` deferred, §6). The **bootstrap** steps:

1. Creates the **worktree** (dir name = branch name; overridable via `--worktree` / `--branch`
   flags or `[store]` config).
2. Creates the **orphan branch** in it (default `odm`).
3. **Writes the chosen names to `odm.toml`** (`[store]`), so later commands resolve — and any
   later rename updates this.
4. Scaffolds the store *inside the worktree*: **`config.toml`** (gate-sets, display, `docs_directory`, author — the operational config) + an empty `nodes/`. (The code-branch `odm.toml` holds only `[store]`, per step 3.)
5. Ensures **`/.worktrees/`** is **git-ignored in the code branch** so the worktree is never staged there.

**Attach** (a branch already exists — the fresh-clone / teammate case): `init` *checks out*, it does not scaffold. If the branch is only on the remote (`origin/odm`), fetch and create a local tracking branch; then ensure `.worktrees/` (create + gitignore if missing) and `git worktree add .worktrees/odm odm`. The store — `config.toml` + `nodes/` — arrives **with** the branch, so nothing is (re)written or re-orphaned; on a fresh clone the code-branch `odm.toml` locator is already present. **No branch anywhere → fall through to bootstrap.**

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
into the worktree/orphan branch. **Split the existing `odm.toml`:** its operational settings (gate-sets, display, `docs_directory`, ...) become **`config.toml` in the store**; a minimal `odm.toml` with just `[store]` stays committed on the code branch as the locator.
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

**Resolution — research 2026-07-26 (gix 0.66; gitoxide `crate-status.md`): Option (b).** gix *can* write refs + commits (branch / orphan-root creation is feasible natively) and *can* check out an index into a directory, but **linked-worktree creation/registration (`git worktree add`) is explicitly _not implemented_ — listed as planned (`create, move, remove, repair` unchecked).** So `init` cannot stand up the worktree through gix. Option (a) native is impossible in 0.66; (c) defer is an unbounded wait on upstream. **Adopt (b):** shell out to `git` for the one-time `init`/rename worktree setup only — `git worktree add --orphan <dir>` (git ≥ 2.42; two-step `worktree add --detach` + `checkout --orphan` for older git) — while **all steady-state reads/writes stay on gix**. Consequence: `init`/rename acquire a `git` **binary** dependency (setup only, not the runtime hot path). A narrow, documented exception to ODD-0013 Q-2 — **ratified by the operator 2026-07-26** (`git` for worktree setup only; all steady-state on gix).

## 6. Open questions

- **Worktree location — RESOLVED (2026-07-26):** adopt the common **`.worktrees/`** convention — the odm worktree at **`<repo>/.worktrees/odm`** (`worktree_base` + `worktree_name`), branch `odm` kept equal to the worktree name by default. `init` gitignores `/.worktrees/`. Fits alongside a user's other worktrees. *(Nested-in-repo caveat: tools that recurse the tree will descend into it unless ignored — the gitignore covers the git side.)*
- **Config placement — RESOLVED (2026-07-26):** two files. `odm.toml` (locator — just `[store]`) committed on the **code branch**; `config.toml` (operational — gate-sets, display, `docs_directory`, author) **in the store**, versioned/shared with the data. Two-stage load — see §4.2.
- **Sharing — RESOLVED (2026-07-26):** the `odm` branch *is* the shared DB. A teammate's `init` **attaches** to an existing `odm`/`origin/odm` (checks it out) rather than forking a fresh orphan (which would split the DB into unrelated histories — the sync nightmare, reintroduced). See the §4.3 three-way detection. **Push/pull:** the `odm` branch is a **normal branch** — `git push`/`pull` *is* the team-sync mechanism (one branch, on purpose); an odm-wrapped `sync`/`push`/`pull` convenience is **deferred** (YAGNI) until a real need appears.
- **Idempotence / sync — RESOLVED (2026-07-26):** `init` on an existing **local** store does a **fast-forward-only sync** (`fetch` + `merge --ff-only` from `origin/odm`), not a bare no-op — a clean FF freshens the store; a **divergence or missing upstream warns and stops**. **No auto-rebase or auto-merge:** the `odm` branch is shared/published, so rewriting (rebase) or silently merging its history would corrupt teammates' clones — FF-only is the only safe *automatic* sync; divergence resolution is the operator's call. `--force` (non-destructive re-init), `--overwrite` (destructive), and repair of partial/broken states (branch present, worktree gone) are **deferred** (YAGNI) until concrete use cases define them.
- **Interaction with the index** — the `.odm/` stat-cache (A4) lives with the store; confirm it
  rides in the worktree and doesn't collide with the worktree-dir name.

## 7. Boundaries / non-goals

- Not a general multi-repo or server sync layer — one repo, one orphan branch, git as the transport.
- Does not change the node model, ids, gates, or the `nodes/YYYY/MM/<ULID>.md` layout — only *where*
  that tree is rooted.
- `init` scaffolds; it does not import — importing is `migrate`/`self-host` (§4.4).

## 8. Version history

### v1.1 — 2026-07-31
§4.2: documents the `[legacy]` sub-table `odm store init` began writing in
arc-migration-fidelity s13 (docs_directory/dev_directory ported forward from
a pre-split odm.toml, never previously recorded here) and the new
`[legacy].additional_paths` key `migrate --all` reads/writes (s14) — the
persistent sweep-and-remember list for un-typed legacy directories beyond
docs_directory/dev_directory. Also records the resolution to a real defect
s14 found and fixed: a `[legacy]` block the operator had added to the
code-branch `odm.toml` (the locator) was invisible to `migrate`, since a
store `config.toml` — the actual file `StoreHome::resolve` treats as
operational once it exists — takes precedence over the locator entirely.
Surfaced by: arc-migration-fidelity s14 (operator-surfaced defects against
`migrate --all`).

### v0.7 — 2026-07-26
§4.3/§6 idempotence refined: `init` on an existing local store does a **fast-forward-only sync** (`pull --ff-only` from `origin/odm`) instead of a bare no-op — a useful re-init that cannot corrupt the shared DB. **No auto-rebase/auto-merge** on the shared `odm` branch (rewriting or merging published history breaks teammates' clones); a divergence or missing upstream warns and stops, resolution left to the operator. `--force`/`--overwrite`/repair still deferred. Surfaced by: operator sync-on-init idea + shared-branch safety analysis.

### v0.6 — 2026-07-26
§6 **sharing resolved — all opens closed; ready for Accepted.** `init`'s **attach** mode (§4.3) checks out an existing local/remote `odm` branch rather than re-orphaning (forking would reintroduce the sync nightmare); the store's `config.toml`+`nodes/` ride *with* the branch, so attach never re-scaffolds. Team sync = plain `git push`/`pull` of the one `odm` branch; an odm-wrapped sync command is deferred (YAGNI). Surfaced by: operator articulation of the init flow.

### v0.5 — 2026-07-26
§6 idempotence resolved: `init` on an existing **local** store warns + **no-ops**; `--force`/`--overwrite` deferred until real use cases (YAGNI). §4.3 now states the **three-way detection** — bootstrap / attach (remote `odm` exists) / no-op (local store exists) — which ties idempotence to the last remaining open, sharing (§6). Surfaced by: operator idempotence decision.

### v0.4 — 2026-07-26
§6 config-placement resolved: **two configs.** `odm.toml` = the locator (`[store]`, committed on the code branch); `config.toml` = the operational config (gate-sets/display/`docs_directory`/author) **inside the store**, so it is versioned and shared with the data it governs (cannot drift). §4.2/§4.3/§4.6 updated; two-stage load (locator -> store path -> `config.toml`). Surfaced by: operator config-placement decision.

### v0.3 — 2026-07-26
§4.2 config settled + §5 ratified. Store location adopts the common `.worktrees/` convention: `[store]` = `worktree_base` / `worktree_name` / `branch_name` (default `.worktrees/odm`, branch `odm`, kept equal). The §6 worktree-location question is resolved; `init` gitignores `/.worktrees/`. The Q-2 shell-out exception (git for `init`/rename setup only) is **operator-ratified**. Surfaced by: operator config decision.

### v0.2 — 2026-07-26
§5 resolved by research (gix 0.66 / gitoxide `crate-status.md`): worktree *creation* is not implemented in gix (ref+commit writing and index-checkout are), so store-home `init`/rename **shell out to `git` for worktree setup only** — a scoped, documented exception to ODD-0013 Q-2 (needs operator ratification); gix stays the engine for every steady-state op. Surfaced by: the CDC gix-capability research.

### v0.1 — 2026-07-26
Initial draft. Records the decision (dedicated orphan `odm` branch in a worktree as the store home),
the resolution/config change (`[store]`), the `init`/rename/migrate shape, the `gix`-vs-shell-out
tension (the key risk), and the open questions. Foundation for `odm init` and RH C-5's migrate
rework. Surfaced by: Duncan's multi-version DB-sync pain + the C-3 DATE-column investigation.
