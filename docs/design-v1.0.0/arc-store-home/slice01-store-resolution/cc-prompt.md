# cc-prompt — arc-store-home slice 01: Store resolution + two-config split

> **Arc:** Store Home & `init` (`arc-store-home`) · **Slice:** 01 · **Feeds:** SH-1 · **Realizes:**
> ODD-0022 §4.2. **Foundational** — slices 02–04 and RH C-5 depend on this. **No git ops** in this
> slice (the `git` shell-out is slice 02).

## Goal

Introduce the **two-config model** and make the store root **resolvable to a worktree**, without
creating anything: `odm.toml` becomes a *locator* (`[store]`), and the *operational* config is read
from a **`config.toml`** at the resolved store root. Fully back-compatible — a repo with no
`[store]` and no `config.toml` behaves exactly as today.

## Changes (`odm-store`, mostly `config.rs` + `layout`)

1. **`[store]` config** — add a `StoreLocation` (or `[store]`) struct: `worktree_base: Option<String>`
   (default `.worktrees`), `worktree_name: Option<String>` (default `odm`), `branch_name:
   Option<String>` (default `odm`). Parsed from `odm.toml` alongside the existing fields.
2. **Store-root resolution** — a function `resolve_store_root(repo_root, &store_cfg) -> PathBuf`:
   `[store]` present ⇒ `repo_root / worktree_base / worktree_name`; absent ⇒ `repo_root`. Route the
   `nodes/` layout (`odm-store/src/layout.rs` — `<root>/nodes/…`) through this resolved root
   everywhere a root is taken today.
3. **Two-stage load** — split `StoreConfig::load`:
   - *locator*: keep the cwd → repo-root → user search to find `odm.toml`; read `[store]`.
   - *operational*: load `config.toml` from the **resolved store root** (gate-sets, display,
     `docs_directory`, author). **Fallback:** if `config.toml` is absent, read the operational
     settings from `odm.toml` (today's behaviour) so existing repos keep working pre-migration.
4. **Do not** move odm's own operational config yet — that split happens at the C-5 cutover. This
   slice only makes the code *able* to read from `config.toml` when present.

## Scope boundary (out)

Worktree/orphan-branch creation, gitignore, `init`/attach/sync/`rename`, any `git` subprocess, and
migrating odm's corpus — **all later** (slices 02–04 / RH C-5). Keep the diff free of
`Command::new("git")` and worktree logic (ledger **L-7**).

## Acceptance / ledger (SH-1, `ledger.md` L-1…L-9)

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- Unit: resolution both ways (L-2/L-3), config precedence + fallback (L-4/L-5).
- Integration: a hand-placed `.worktrees/odm/` store (with `config.toml` + `nodes/`) + `[store]` in
  `odm.toml` → `odm list`/`check` operate there (L-6/L-9); remove `[store]` → operate on
  `<repo>/nodes/`, unchanged (L-3/L-9).
- `grep` confirms no git subprocess/worktree code (L-7).

## Method

One branch (`sh-slice01-store-resolution`, off `release/1.0.x` or the latest tip — settle at start);
one mergeable diff; five-iteration cap. CC implements on local 1.85+; CDC verifies
(`cdc-verification.md`), cargo rows attested → CI. On close, bubble up to
`arc-store-home/arc-plan.md` (SH-1; note anything the arc-plan didn't anticipate; silent-drop diff).
