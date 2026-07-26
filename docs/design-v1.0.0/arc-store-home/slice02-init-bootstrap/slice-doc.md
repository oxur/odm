# Slice 02 — `git`-worktree plumbing + `odm store init` bootstrap (slice-doc / plan-of-record)

> **Arc:** Store Home & `init` (`arc-store-home`) · **Ledger:** SH-2 · **Realizes:** ODD-0022
> §4.3 (bootstrap) + §5 (the ratified `git` shell-out exception). **Builds on slice 01** (`home.rs`
> resolution + two-config load); **load-bearing for** slice 03 (attach/ff-sync), slice 04 (rename),
> and the RH C-5 cutover (SH-6). Consumes the `branch_name` that slice 01 parsed but left unused.

## Goal

Make `odm` able to **stand up a store home from nothing**: on a repo with no `odm` branch, one
command creates the worktree + orphan branch, writes the `[store]` locator, scaffolds the store, and
gitignores the worktree — so that afterwards every odm command resolves (via slice 01) into the new
home and `check` is green there. This is the first place odm touches `git` as a **subprocess**, under
the narrow, operator-ratified ODD-0022 §5 exception (setup only; all steady-state stays on `gix`).

After this slice, on a fresh repo: `odm store init` ⇒ `.worktrees/odm/` worktree checked out to a
new orphan `odm` branch; `odm.toml` gains a `[store]` section; the store holds `config.toml` +
empty `nodes/`; `/.worktrees/` is git-ignored on the code branch; `odm new` / `odm check` then
operate in the home.

## Command placement (decide at slice start)

ODD-0022/the arc-plan wrote this as `odm init`; **ODD-0023 (Draft) places it under `odm store
init`.** Per the operator's sequencing call (inventory rewrite *before* store, so the command is
born in its final home and never renamed), this slice **births the `store` subcommand group with
`init` as its first member** — not a top-level `odm init` later moved. This is low-risk even with
ODD-0023 in Draft: the `store` group is the *undebated* part of that ODD (§8 is naming/placement
only; the §5 debatables never touch `store`), and "store commands need a home" is ODD-0023's
motivating problem (§2). The **rest** of the reorg (the `node` group, top-level renames, deprecation
aliases) is **not** in this slice — it lands in RH C-4. Only the `store` parent + `init` child are
added here. *(If the operator prefers to hold the group until ODD-0023 is Accepted, the fallback is a
top-level `odm init` with a store rename in C-4 — but that reintroduces exactly the churn the
rewrite-first ordering was chosen to avoid.)*

## Scope

**In:**

- **The `git` shell-out wrapper** (new, isolated module — e.g. `odm-store/src/worktree.rs`): the
  *only* place odm invokes the `git` binary. Creates the worktree + orphan branch:
  `git worktree add --orphan <branch> <dir>` on git ≥ 2.42; a **two-step fallback**
  (`git worktree add --detach <dir>` then `git -C <dir> checkout --orphan <branch>`) for older git.
  Detect the `git` version once; a clear, actionable error if `git` is absent or too old (name the
  fix). Ratified ODD-0022 §5 exception — scoped to setup, documented at the module head.
- **Bootstrap detection** — "**no `odm` branch anywhere**" (no local `refs/heads/<branch_name>`, no
  `refs/remotes/*/<branch_name>`). Only the bootstrap arm is implemented here; if a branch **does**
  exist, `init` **stops with a clear message** deferring to slice 03's attach/sync (no clobber, no
  re-orphan).
- **Bootstrap steps** (ODD-0022 §4.3, in order): (1) create the worktree dir (name from
  `[store].worktree_name`, default `odm`, overridable via `--worktree`/`--branch`); (2) create the
  orphan branch (`[store].branch_name`, default `odm`); (3) **write `[store]` to `odm.toml`** on the
  code branch (the locator, so later commands resolve); (4) scaffold **inside the worktree**:
  `config.toml` (operational defaults — gate-sets, display, `docs_directory`, author) + an empty
  `nodes/`; (5) ensure **`/.worktrees/`** is in the code branch's `.gitignore` (create/append,
  idempotent).
- **`odm store init` CLI wiring** — the `store` subcommand group + `init` subcommand, `--dry-run`
  (print the plan, touch nothing) and `--yes`, `--worktree`/`--branch` overrides. `--json` reports
  the created home (mode `bootstrap`, paths, branch).
- **Carried item #1 (slice-01 bubble-up) — `.odm/context.json`:** route it through the **resolved
  store root**, so context rides *with* the store exactly as the `.odm/` index does (ODD-0022 §6
  "the `.odm/` stat-cache lives with the store"). context.json names node ids — it is store state,
  not invocation state. Small, in-scope, and this is the slice that owns the call.

**Out (later slices / not this slice):**

- **attach** (existing local/remote `odm` branch) and **ff-sync** — **slice 03** (SH-3). This slice
  only bootstraps and *detects-and-stops* on an existing branch.
- **`rename`** (store rename) — **slice 04** (SH-4).
- **Migrating odm's own corpus** onto the orphan branch — **RH C-5's re-self-host / cutover**
  (SH-6). `init` scaffolds an *empty* store; it does not import (ODD-0022 §7).
- **The full C-4 command reorg** — the `node` group, `context`→`project`, `path`→`chain`,
  deprecation aliases, `--help`/`--json` rewrite. Only the `store` group + `init` are added here.
- **Steady-state git on `gix`** — unchanged. The subprocess is bootstrap-only; no read/write hot
  path gains a `git` call.

## Carried item #2 — `ROLLUP.md` placement (decision, not code, in this slice)

Slice 01 left open where `ROLLUP.md` lives once the store moves to the worktree. **Decision:**
`ROLLUP.md` stays at the **repo (invocation) root** — it is a projection *out of* the store meant to
be read/committed on the **code branch** (the human-facing "what's the plan" view, like a README);
putting it inside a gitignored worktree would hide it from everyone browsing the code branch. This
slice **records the decision** and does **not** change `rollup`'s output path (it already writes at
the invocation root). Any flat-rollup mechanics belong to the LLM arc (L-7) / the rollup owner; no
code change here beyond the recorded call.

## Verification approach

- **Unit:** `git`-version detection; the wrapper builds the correct argv for ≥ 2.42 vs the two-step
  fallback; `git`-absent / too-old produces the actionable error. Bootstrap detection classifies
  "no branch anywhere" vs "branch exists" correctly (fixtures with/without local & remote refs).
- **Integration** (scratch git repos, `assert_cmd`): `odm store init` on a fresh repo ⇒
  `.worktrees/odm/` exists; the worktree's branch is `odm` and **orphan** (disjoint history — no
  parent commit shared with the code branch); `odm.toml` has `[store]`; `config.toml` + `nodes/`
  scaffolded in the store; `/.worktrees/` in `.gitignore`. Then `odm new …` writes under the store
  and `odm check` is green there. `--dry-run` touches nothing. A second `odm store init` **does not
  re-bootstrap** — it detects the existing branch and stops with the slice-03 deferral message.
- **Scope-boundary grep:** `Command::new("git")` (or the chosen exec) appears **only** in the new
  worktree module — steady-state modules (`git.rs`, `store.rs`, `layout.rs`, `reconcile`) unchanged
  in that respect. `.odm/context.json` reads/writes go through the resolved store root.
- `odm check` green in the new home; existing no-`[store]` repos still unaffected (slice-01 regression
  intact).

## Exit criteria

- `odm store init` bootstraps a working home end-to-end (worktree + orphan branch + locator +
  scaffold + gitignore); `odm new`/`check` operate there afterwards.
- The `git` shell-out is confined to the one setup module, version-guarded, with a clear error when
  `git` is missing/old; steady-state stays on `gix`.
- Existing-branch case **stops cleanly** (defers to slice 03) — no clobber, no second orphan.
- `.odm/context.json` follows the resolved store root (carried item #1 disposed).
- `ROLLUP.md` placement decision recorded (carried item #2 disposed); no rollup-path regression.
- `cargo build` / `test` / `clippy --all-targets -- -D warnings` / `fmt` green; no `unsafe`.
- All SH-2 `ledger.md` rows reach a final status; bubble-up to `arc-plan.md` (SH-2; anything the
  arc-plan didn't anticipate; silent-drop diff).
