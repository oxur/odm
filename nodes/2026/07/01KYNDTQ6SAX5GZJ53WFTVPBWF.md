---
id: 01KYNDTQ6SAX5GZJ53WFTVPBWF
number: 7773603
type: slice
schema: slice/v1.1
name: Slice 03 — `init` attach + ff-sync (slice-doc / plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc-store-home/slice03-init-attach-sync/slice-doc.md
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
# Slice 03 — `init` attach + ff-sync (slice-doc / plan-of-record)

> **Arc:** Store Home & `init` (`arc-store-home`) · **Ledger:** SH-3 · **Realizes:** ODD-0022 §4.3
> (attach + sync) + §6 (sharing, idempotence/ff-only). **Builds on slice 02** (`init.rs::detect`
> already returns `ExistsLocally` / `ExistsOnRemote`, and stops on both with a "slice 03" message;
> `worktree.rs` is the git shell-out). **Completes the three-way `init`** and unblocks the C-5 cutover.

## Goal

Make `odm store init` do the right thing when a store **already exists** — the two arms slice 02
detected but deferred. **Attach** stands up a local worktree over an existing `odm` branch (a fresh
clone / a teammate joining) *without* re-scaffolding or re-orphaning. **ff-sync** freshens an existing
local store from its upstream by fast-forward only, and **warns + stops** on anything that isn't a
clean fast-forward. After this slice, `init` is idempotent-and-useful in all three states: bootstrap
(slice 02), attach, sync.

## The three-way detection (now fully wired)

`detect` (slice 02) already classifies; slice 03 gives the two non-bootstrap arms behaviour:

- **`Bootstrap`** → slice 02 (create the orphan + worktree + scaffold). *Unchanged.*
- **`ExistsOnRemote(remote)`** → **attach**: check the existing branch out into the worktree.
- **`ExistsLocally`** → **ff-sync**: fast-forward the local store from upstream, or warn + stop.

## Scope

**In:**

- **Attach** (ODD-0022 §4.3, the teammate/fresh-clone case): `git worktree add <store_root> <branch>`
  — checkout of the **existing** branch, **not** `--orphan`. When the branch is remote-only
  (`origin/<branch>`), fetch and let the worktree checkout create the local tracking branch
  (`worktree add` DWIMs `origin/<branch>` → a tracking `<branch>`; do it explicitly if that's clearer).
  **No re-scaffold, no re-orphan** — `config.toml` + `nodes/` ride *with* the branch, so attach must
  not write them. Ensure the code-branch `.gitignore` carries `/.worktrees/` and the `odm.toml`
  locator is present (both usually already committed on a clone — idempotent top-up only).
- **ff-sync** (ODD-0022 §6, idempotence): on an existing **local** store, `fetch` then compare local
  `<branch>` vs its upstream and act by ancestry:
  - upstream is **ahead** (local is an ancestor) → `merge --ff-only` in the worktree; report freshened.
  - **up to date** → no-op, report fresh.
  - local is **ahead** (unpushed commits, upstream not advanced) → report "N to push", **no error**.
  - **diverged** (both advanced) → **warn + stop**, leave resolution to the operator (never rebase or
    merge a shared branch).
  - **no upstream / no remote** → **warn + stop** ("nothing to sync from"), not an error.
- **Wiring**: replace slice 02's "defer to slice 03" stops in the `ExistsOnRemote` / `ExistsLocally`
  arms with these behaviours; `--dry-run` reports the chosen arm + plan and touches nothing; `--json`
  reports `mode` (`attach` / `sync-fast-forwarded` / `sync-up-to-date` / `sync-local-ahead` /
  `sync-diverged` / `sync-no-upstream`) with the relevant refs/counts.

**Out (later / deferred):**

- **Repair of partial/broken states** (branch present but worktree gone; a half-finished `init`) —
  **deferred per ODD-0022 §6 (YAGNI)**. Detect it and **warn clearly** pointing at the future
  `--force`; do not silently "fix" it.
- **`--force` / `--overwrite`** (non-destructive re-init / destructive reset) — deferred (§6).
- **A standalone `odm store sync` command** — ODD-0022 §6 **defers** the odm-wrapped sync (team sync is
  plain `git push`/`pull`); ff-sync here is `init`'s ExistsLocally behaviour, **not** a new verb. *(See
  the surface note below — this is a small inventory reconciliation for the operator.)*
- **`rename`** (slice 04); **corpus migration** (RH C-5 cutover).

## Decision to settle at slice start — where attach/sync git calls live

Attach's `git worktree add` is a worktree-registration op **gix 0.66 cannot do** (same wall as
bootstrap), so it **must** shell out — squarely inside the ratified ODD-0022 §5 exception (`init`
setup). ff-sync's `fetch` + `merge --ff-only` *could* go through gix, but slice 02 already put the
`init` git in `worktree.rs`. **Recommendation:** keep **all `init`-time git** (attach's `worktree add`,
ff-sync's `fetch`/ancestry/`merge --ff-only`) in that one shell-out module, and hold the invariant that
**matters** — *no git subprocess during steady-state node reads/writes* (the L-14 boundary) — unchanged.
This is a minor widening of §5 from "create" to "create + attach + ff-sync", all still `init`-only; flag
it for the operator rather than widening silently. *(Alternative: gix for fetch/ff, shell only for
`worktree add` — more moving parts for no steady-state benefit. Recommend against for now.)*

## Surface note (inventory reconciliation)

The command inventory lists `odm store sync` as a `[store-home]` command. ODD-0022 §6 **defers** the
standalone wrapper (YAGNI); slice 03 delivers ff-sync as `init` behaviour only. **Operator call:** mark
`store sync` in the inventory as *deferred / folded into `init`*, or add it later as a thin alias for
"`init` on an existing local store." Not implemented here either way.

## Verification approach

- **Unit:** ancestry classification (up-to-date / upstream-ahead / local-ahead / diverged /
  no-upstream) as a pure function over two commit ids + an upstream option, so the decision table is
  tested without a network.
- **Integration** (`assert_cmd`, scratch repos with a **local bare remote** as `origin`): 
  - *attach* — clone a repo whose `origin/odm` exists (bootstrap in repo A, push `odm` to a bare
    remote, clone as repo B); `odm store init` in B checks out the worktree, `nodes/`+`config.toml`
    ride with it (not re-scaffolded), `odm check` green, the branch is **not** re-orphaned (shares
    history with the pushed `odm`).
  - *ff-sync* — advance `origin/odm` (a new node pushed from A), `init` in B fast-forwards and the node
    appears; **diverged** (commit locally in B *and* advance origin) → warns + stops, worktree
    untouched; **no upstream** → warns + stops; **up-to-date** → clean no-op.
  - `--dry-run` touches nothing on every arm; `--json` reports the right `mode`.
- **Scope grep:** the new git calls stay in the `init`/worktree module; steady-state stays gix (L-14).

## Exit criteria

- All three `init` arms work end-to-end; attach never re-scaffolds/re-orphans; ff-sync fast-forwards
  or warns-and-stops and **never** rebases/merges a shared branch.
- Diverged / no-upstream / local-ahead / up-to-date each handled with the correct message and no data
  loss; deferred repair states warn clearly rather than acting.
- `--dry-run`/`--json` correct on every arm; steady-state stays on gix.
- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- All SH-3 `ledger.md` rows reach a final status; bubble-up to `arc-plan.md` (SH-3; anything the
  arc-plan didn't anticipate; silent-drop diff). This closes the three-way `init` → SH-5 (the
  bootstrap/attach/ff-sync compose demo) becomes reproducible.
