# Slice 01 — Store resolution + two-config split (ledger)

> Per LEDGER-DISCIPLINE v2.0 §A (slice tier). Rows are the grep-verifiable **steps**; each reaches a
> final status (`done` / `deferred` / `no-op`) before the slice closes. Evidence strength
> `asserted < attested < reproduced < reconciled`; cargo/executable rows are attested-by-CC →
> reproduced-on-CI. Closer ≠ verifier (CDC writes `cdc-verification.md`). Feeds arc row **SH-1**.

| ID | Criterion | Verify | Significance | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|----------|-------|
| L-1 | `[store]` section parses from `odm.toml` — `worktree_base` / `worktree_name` / `branch_name`, all **optional** with defaults `.worktrees` / `odm` / `odm` | unit test; `grep` the config struct | serious | open | | the locator contract (ODD-0022 §4.2) |
| L-2 | With `[store]` present, store root resolves to `<repo>/<worktree_base>/<worktree_name>` | unit test | serious | open | | |
| L-3 | **Back-compat:** with **no** `[store]`, store root = repo root (unchanged) | unit test | serious | open | | zero behaviour change for existing repos |
| L-4 | Two-stage load: operational config (`gate-sets`, `display`, `docs_directory`, author) loads from **`config.toml`** at the resolved store root | unit + integration | serious | open | | the operational half of the split |
| L-5 | **Back-compat:** `config.toml` absent → operational config falls back to `odm.toml` | unit test | serious | open | | existing repos keep working before migration |
| L-6 | Node reads/writes use the **resolved** store root (not the repo root) | integration: `odm list`/`check` on a hand-placed `.worktrees/odm/` store | serious | open | | the payoff — commands operate on the configured home |
| L-7 | **No git-worktree ops / no `git` subprocess** introduced in this slice | `grep -rE 'Command::new\("git"\)\|worktree' crates` over the diff → none | correctness | open | | the scope boundary (git shell-out is slice 02) |
| L-8 | `cargo build` / `test` / `clippy --all-targets -- -D warnings` / `fmt` green; no `unsafe` | CC attest (local 1.85+) → CI | serious | open | | attested-by-CC → reproduced-on-CI |
| L-9 | `odm check` green in **both** modes (with and without `[store]`) | integration | serious | open | | |

**Close:** per-row walk in `closing-report.md` (CC) + `cdc-verification.md` (CDC), with the
**bubble-up to `arc-plan.md`** (did slice 01 deliver SH-1; anything the arc-plan didn't anticipate;
the silent-drop diff).
