# Slice 06 (Store Lifecycle): `store set-remote`

> **Arc:** Store Lifecycle · **Branch:** `release/1.0.x` · **Status:** planned
>
> **Depends on:** s03 (`store sync`, done) · s05 (CDC binary access, done) ·
> ODD-0022 (the store discipline)
>
> **Unblocks:** The operator's first `store sync` against GitHub — which
> currently fails with "no upstream" because the orphan branch has never been
> pushed and `sync` has no first-push path. Also unblocks SL-4 (the composition
> row: "no raw git for the normal lifecycle") — without this slice, the operator
> must `git push origin odm-store` by hand for the initial push.

## Goal

Add `odm store set-remote` — a thin config command that tells the store which
git remote to sync to — and teach `sync` to handle the **first-push** case
(remote configured, branch not on the remote yet). Today `sync` hardcodes
`DEFAULT_REMOTE` ("origin") and treats a missing upstream ref as `NoUpstream`
("nothing to sync"), which is wrong when the remote exists but the branch
simply hasn't been pushed yet. After this slice: `store init → set-remote →
commit → sync` pushes the store to GitHub on the first run without dropping to
raw git.

## Why

The operator ran `odm store sync` live and got "nothing to sync — has no
upstream (no remote, or it does not carry the branch)." The remote (`origin`)
exists and is reachable; the branch just hasn't been pushed. `sync` can't
distinguish "no remote at all" from "remote is there but doesn't carry this
branch yet" — the second case is a first-push, not a "nothing to sync."

Two separable problems, one coherent fix:

1. **Remote configuration** — currently hardcoded to `DEFAULT_REMOTE`. The user
   needs a way to say "sync to this remote," stored persistently.
2. **First-push bootstrapping** — once a remote is configured and fetch
   succeeds, `sync` should recognize that a missing upstream ref means "push
   the branch" rather than "no upstream."

## Scope

### In

- `StoreCommand::SetRemote` — a new CLI subcommand (`odm store set-remote`).
- **Explicit form:** `odm store set-remote <remote>` — validates the named
  remote exists (`git remote get-url`), writes `remote = "<name>"` to `[store]`
  in `odm.toml`, reports.
- **Auto-detect form:** `odm store set-remote` (no argument) — lists the main
  repo's remotes; if exactly one, uses it; if zero or multiple, errors with a
  clear message. No interactive prompt.
- `--json`: structured output.
- **`StoreLocation.remote`** — a new `Option<String>` field in
  `StoreLocation` (`home.rs`), deserialized from `odm.toml`'s `[store]`
  section. Backward-compatible: existing stores without the field parse as
  `None`.
- **`sync` reads the configured remote** — falls back to `DEFAULT_REMOTE`
  ("origin") when `store.remote` is `None`, for backward compatibility.
- **`sync` first-push** — when fetch succeeds but `rev_parse` for the upstream
  ref returns `None`, treat as first-push: push the branch (unless
  `--dry-run`). This replaces the current `NoUpstream` arm for this case.
- **`store init` auto-sets remote** — when bootstrapping a new store, if the
  main repo has exactly one remote, write `remote = "<name>"` into the
  `[store]` section automatically. Multi-remote repos require explicit
  `set-remote`.
- **`worktree::list_remotes()`** — a new helper in `odm-store` that runs
  `git remote` and returns the list.

### Out

- **Remote management** (`add`, `remove`, `rename`) — odm uses whatever
  remotes git already has; `set-remote` only chooses which one the store syncs
  to.
- **Multi-remote sync** — one remote at a time; configurable but not
  fan-out.
- **Changes to `status`** — `status` stays read-only (s02 D-1 stands).

## Design decisions

### D-1: Auto-detect, no prompt

The no-argument form does not prompt `[Y/n]`. If exactly one remote exists, it
uses it and reports what it did. If zero remotes exist, it errors ("no remotes
configured for this repository"). If multiple exist, it errors and lists them
("multiple remotes configured — specify one: origin, upstream, …"). The action
is low-stakes (setting a config value, not pushing data) and easily reversed by
re-running with a different argument.

### D-2: Config location

`remote = "<name>"` under `[store]` in `odm.toml`, as an `Option<String>` on
`StoreLocation`. Backward-compatible: existing stores parse with `remote: None`.
`sync` falls back to `DEFAULT_REMOTE` when the field is absent, so no existing
workflow breaks.

### D-3: `store init` auto-sets remote

When bootstrapping a new store (the bootstrap path, not the attach path), if
the main repo has exactly one remote, `write_locator` includes `remote =
"<name>"` in the `[store]` section. This makes `sync` work out of the box for
the common case. Multi-remote repos require explicit `set-remote`. Existing
stores (already initialized) are unaffected — they use `set-remote` to opt in.

### D-4: First-push in `sync`

After a successful fetch, if `rev_parse` for `origin/<branch>` returns `None`,
the branch isn't on the remote. This is a first-push, not a "no upstream."
`sync` pushes the branch (or reports "would push" under `--dry-run`). The
`NoUpstream` arm is reserved for when fetch itself fails (no remote, no
network). The JSON `action` is `"pushed"` (same as `LocalAhead`) — from the
caller's perspective, it's a push.

### D-5: TOML update for existing stores

`set-remote` must add or update the `remote` field in an existing `[store]`
section. `write_locator` (init.rs) only appends a new section; `set-remote`
needs a different path. The recommended approach: read the file, use the `toml`
crate to parse, update the value, serialize back with `toml::to_string_pretty`
or a targeted string replacement that preserves comments and formatting. CC
should flag the approach chosen.

## Verification

Per-row in `ledger.md` (F-1…F-8). The real test: `store init → store
set-remote → store commit → store sync` pushes the store to a bare-repo remote
on the first run, with no raw git.

## Rollback

CLI and `odm-store` config. If `set-remote` lands wrong: revert on
`release/1.0.x`, rebuild. The only persistent effect is a `remote` field in
`odm.toml`, which is harmless if orphaned.

## Exit

SL-4 becomes closeable — the last raw-git step (first push) is now an odm
verb. The lifecycle reads `init → set-remote → mutate → status → commit →
sync` — or, for single-remote repos: `init → mutate → status → commit → sync`
(init auto-sets the remote).
