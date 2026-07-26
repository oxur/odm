# cc-prompt — arc-store-home slice 03: `init` attach + ff-sync

> **Arc:** Store Home & `init` (`arc-store-home`) · **Slice:** 03 · **Feeds:** SH-3 · **Realizes:**
> ODD-0022 §4.3 (attach + sync) + §6 (sharing, ff-only idempotence). **Builds on slice 02**:
> `init.rs::detect` already returns `Mode::ExistsOnRemote` / `Mode::ExistsLocally` and currently
> **stops** on both with a "slice 03" message; `worktree.rs` is the git shell-out (now carrying the
> `-b` fix + the CI guard). This slice replaces those two stops with real behaviour.

## Goal

Complete the three-way `odm store init`. **Attach**: stand up a worktree over an **existing** `odm`
branch (fresh clone / teammate) — checkout, never re-orphan, never re-scaffold. **ff-sync**: freshen
an existing local store from upstream by **fast-forward only**, and **warn + stop** on anything that
isn't a clean fast-forward. Bootstrap (slice 02) is unchanged.

## Changes

### 1. Attach — `Mode::ExistsOnRemote(remote)`

- `git worktree add <store_root> <branch>` — a checkout of the **existing** branch, **not**
  `--orphan`. This is *not* version-gated (worktree add is old), so no `GitVersion` branching — but it
  still shells out (gix can't register a worktree), inside the ODD-0022 §5 `init` exception.
- If the branch is **remote-only** (`origin/<branch>` but no local `refs/heads/<branch>`): `fetch`
  first, then the `worktree add` (git DWIMs `origin/<branch>` into a local tracking `<branch>`; make it
  explicit if clearer).
- **Do not scaffold** `config.toml` / `nodes/` — they arrive **with** the branch. **Do not** write the
  locator or re-orphan. **Idempotent top-up only:** ensure the code-branch `.gitignore` has
  `/.worktrees/` and the `odm.toml` locator exists (both usually already committed on a clone).

### 2. ff-sync — `Mode::ExistsLocally`

- `fetch`, then classify local `<branch>` vs its upstream **by ancestry** (extract this as a pure
  function `sync_action(local, upstream: Option<Oid>, ...) -> SyncAction` so it's unit-testable):
  - upstream **ahead** (local is ancestor of upstream) → `merge --ff-only` **in the worktree**.
  - **up to date** → no-op, report fresh.
  - local **ahead** (upstream is ancestor of local) → report "N commit(s) to push", **success, no
    change**.
  - **diverged** (neither is an ancestor) → **warn + stop**. **Never** `rebase`/`merge` (non-ff) a
    shared branch — rewriting/merging published history corrupts teammates' clones.
  - **no upstream / no remote** → **warn + stop** ("nothing to sync from"), **success**, nothing
    touched.
- Ancestry via `git merge-base --is-ancestor` (or gix), whichever is cleaner; keep it in the same
  shell-out module (see §5 boundary).

### 3. Wiring + surface

- Replace the two slice-02 "defer to slice 03" stops with the above. Keep the existing-branch **safety**
  (attach/sync must never re-orphan or clobber store data).
- `--dry-run`: report the chosen arm + the plan, touch nothing (all arms).
- `--json`: `mode` ∈ `attach` | `sync-fast-forwarded` | `sync-up-to-date` | `sync-local-ahead` |
  `sync-diverged` | `sync-no-upstream`, plus the relevant refs/counts.
- **No standalone `odm store sync` command** — ODD-0022 §6 defers it; ff-sync is `init` behaviour only.

## Scope boundary (out)

- **Repair** (branch present, worktree gone; half-finished init): **detect and warn clearly** pointing
  at a future `--force`; do **not** silently fix (ODD-0022 §6, deferred). `--force`/`--overwrite`
  deferred.
- `rename` (slice 04); corpus migration (RH C-5).
- **Steady-state stays on gix** — the new git is `init`-time only (L-15).

## §5 boundary — confirm at start

attach's `worktree add` must shell out (gix can't). Keep **all** `init`-time git — attach's
`worktree add` and ff-sync's `fetch`/ancestry/`merge --ff-only` — in the one shell-out module, and hold
the invariant that steady-state node reads/writes touch **no** git subprocess (L-15). This minorly
widens §5 from "create" to "create + attach + ff-sync", all `init`-only. Record the decision in the
closing report (don't widen silently).

## Acceptance / ledger (SH-3, `ledger.md` L-1…L-16)

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- Unit: `sync_action` ancestry table (all five cases) without a network.
- Integration (`assert_cmd`, scratch repos with a **local bare remote** as `origin`):
  - *attach* — bootstrap in repo A, push `odm` to a bare remote, clone as B; `init` in B checks out the
    worktree, `config.toml`+`nodes/` ride with it (not re-scaffolded), `check` green, branch **shares
    history** with the pushed `odm` (not re-orphaned).
  - *ff-sync* — advance `origin/odm` from A → `init` in B fast-forwards, the node appears; **diverged**
    (commit in B + advance origin) → warns + stops, worktree untouched; **no upstream** → warns + stops;
    **up-to-date** → clean no-op; **local-ahead** → reports, no error.
  - `--dry-run` touches nothing on every arm; `--json` mode correct.
- `grep` confirms the git subprocess stays in the `init`/worktree module (L-15).

## Method

One branch (`sh-slice03-init-attach-sync`, off the slice-02 tip — which now includes the amendment; or
`release/1.0.x` if slice 02 has merged — settle at start); one mergeable diff; five-iteration cap. CC
implements on local 1.85+ **with a real git** (attach/sync shell out; a local bare remote gives you
`origin` without a network). CDC verifies (`cdc-verification.md`); cargo rows attested → CI (the slice-02
git-version matrix already exercises this suite on both git arms — attach isn't `--orphan`-gated, but
keep it running under both). On close, bubble up to `arc-store-home/arc-plan.md` (SH-3; the §5-widening
decision; anything unanticipated; silent-drop diff). Closing SH-3 makes **SH-5** (the three-mode compose
demo) reproducible.
