# Slice 01 closing report — Store resolution + two-config split

> **Arc:** Store Home & `init` (`arc-store-home`) · **Slice:** 01 · **Feeds:** SH-1
> **Realizes:** ODD-0022 §4.2 · **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (L-1…L-9)
> **Implemented by:** CC · **Date:** 2026-07-26 · **Branch:** `sh-slice01-store-resolution`
> (off `release/1.0.x` @ `604f816`) · **Evidence class:** attested-by-CC (local 1.85+); cargo
> rows reproduce on CI.

## What shipped

The store root is now **resolvable**, and configuration is **split in two** — without creating
anything and without changing any existing repo's behaviour.

```
odm.toml            the locator   — [store]: where the store is        (code branch)
└─ .worktrees/odm/  the store
   ├─ config.toml   operational   — gate-sets, display, author         (with the data)
   └─ nodes/        the corpus
```

New module **`odm-store/src/home.rs`**:

- **`StoreLocation`** — the `[store]` section: `worktree_base` (default `.worktrees`),
  `worktree_name` (default `odm`), `branch_name` (default `odm`). Every field optional, so a
  bare `[store]` is a complete locator.
- **`StoreHome::resolve(start)`** — the two-stage load: find `odm.toml` by the existing layered
  search (cwd → repo root → user config), read `[store]`, resolve the store root, then take the
  operational config from that store's `config.toml` — falling back to the locator when the
  store has none.
- **`StoreConfig::load`** now parses whichever file that resolution selects, and gained
  `from_home` for a caller that has already resolved.

Wired through in `odm-cli`: `dispatch` resolves once and opens the store at `home.store_root`;
`load_gate_config` and `display_max_width` read `home.operational_text()`.

## Two decisions worth recording

**1. Resolution never fails.** A malformed or unreadable locator resolves to the
*un-redirected* default rather than erroring. A broken `[store]` must not make the corpus
unreachable — that would turn a typo into "odm can't find your data". Strict parsing, and its
error, still live in `StoreConfig::load`, which reads the same file; the two are deliberately
different jobs. Tested (`test_a_malformed_locator_still_resolves_rather_than_wedging`).

**2. `root` keeps meaning the *invocation* root.** Only the node tree moves to the resolved
store. `root` is still what the config search starts from and what a relative path argument is
resolved against — neither of which follows the store. So `migrate <legacy-path>` still
resolves that path against the cwd, and `ROLLUP.md` is still written where it is today. See
"For the arc to decide" below.

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **54 binaries ok, 0 failed** (+10 `home` unit tests, +7 `store_home` integration tests) |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added |

**L-6 — commands operate on the resolved home.** With a hand-placed `.worktrees/odm/` store and
`[store]` in `odm.toml`, `odm new` writes into the worktree and **nothing appears at the repo
root** (the test asserts both halves, since only writing to the right place is half the claim);
`odm list` reads them back; `odm check` is green there.

**L-3/L-9 — back-compat, reproduced on the real corpus.** odm's own repo has no `[store]`:
`odm check` green at **60 nodes**, `nodes/` still at the repo root, no `.worktrees/` invented.
Every pre-existing test passes untouched, including the three `StoreConfig` tests that pin the
layered search.

**L-4/L-5 — precedence and fallback.** Proven behaviourally rather than by inspection: the
store's `config.toml` defines a gate name (`polished`) the locator does not, and `set-gate`
accepting it is what shows which file was read. The fallback case — a redirected store with no
`config.toml` yet, which is the state every repo sits in between this slice and the C-5 cutover
— is tested the same way.

**L-7 — the scope boundary held.** No `git` subprocess and no worktree operations: the only
`Command::new` anywhere in `crates/` is `odm-reconcile`'s pre-existing shell *probe*, and
`git.rs` is untouched. `worktree` appears in this diff only as config field names, test
fixtures, and prose.

## For the arc to decide (not anticipated by the arc-plan)

1. **`.odm/` is split across the two roots.** The index already follows `store.root()`, so it
   moves with the store — but `.odm/context.json` (the `use project`/`use arc` selection) is
   written relative to the invocation root, so with a redirected store the two halves of `.odm/`
   land in different places. Left as-is here because moving it is a behaviour change this slice
   was not scoped for, and the context is documented as *CLI* state rather than store state.
   Slice 02/03 should settle it: it names node ids, which argues for following the store.
2. **`ROLLUP.md` stays at the invocation root.** It is a projection *out* of the store, so it is
   arguably a code-branch artifact — but that is a decision, not an accident, and ODD-0022 does
   not say. Flagged rather than chosen.
3. **`branch_name` is parsed but unused.** It has no consumer until slice 02. It lives in
   `StoreLocation` because the locator is where a rename must update it atomically (§4.5), and
   splitting it elsewhere would invite the two to disagree.

## Silent-drop diff

None. Everything the cc-prompt scoped in shipped: `[store]` parsing, store-root resolution,
the two-stage load with fallback, and the `nodes/` layout routed through the resolved root.
Everything it scoped out — worktree/branch creation, gitignore, `init`/attach/sync/rename, any
`git` subprocess, migrating odm's corpus — is absent, and L-7 checks that mechanically.

## Ledger

- **`SH-1`** — ready to close: **attested** on this report; **reproduced** when CI runs the
  cargo rows green.
- **L-1…L-9** all `done`; see `ledger.md` for the per-row evidence.
- Bubble-up to `arc-store-home/arc-plan.md`: SH-1 attested; the three items above raised for
  slices 02–04; no silent drops.
