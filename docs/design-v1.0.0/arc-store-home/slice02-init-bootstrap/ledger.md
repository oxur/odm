# Slice 02 — `git`-worktree plumbing + `odm store init` bootstrap (ledger)

> Per LEDGER-DISCIPLINE v2.0 §A (slice tier). Rows are the grep-verifiable **steps**; each reaches a
> final status (`done` / `deferred` / `no-op`) before the slice closes. Evidence strength
> `asserted < attested < reproduced < reconciled`; cargo/executable rows are attested-by-CC →
> reproduced-on-CI. Closer ≠ verifier (CDC writes `cdc-verification.md`). Feeds arc row **SH-2**.

| ID | Criterion | Verify | Significance | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|----------|-------|
| L-1 | **Worktree wrapper exists, isolated:** one new module is the sole caller of the `git` binary; the modern path is `git worktree add --orphan <branch> <dir>` (git ≥ 2.42) | `grep` the module; unit test builds the argv | serious | open | | the ODD-0022 §5 exception, scoped and documented at the module head |
| L-2 | **Old-git fallback:** git < 2.42 uses the two-step `worktree add --detach` + `checkout --orphan <branch>`; version detected once | unit test on the version→argv mapping | serious | open | | keep the exception working on stock/LTS git |
| L-3 | **`git` missing/too-old → actionable error** naming the fix (install/upgrade git), not a panic or opaque failure | unit test; error-string assertion | correctness | open | | errors-as-affordances |
| L-4 | **Bootstrap detection:** "no `odm` branch anywhere" (no local `refs/heads/<branch>`, no `refs/remotes/*/<branch>`) selects bootstrap | unit test with fixture ref sets | serious | open | | attach/sync (the other arms) are slice 03 |
| L-5 | **Existing branch ⇒ stop cleanly** (defer to slice 03; no clobber, no re-orphan) | integration: pre-seed an `odm` branch → `init` stops with the deferral message | serious | open | | the safety boundary between 02 and 03 |
| L-6 | **Bootstrap creates the worktree + orphan branch:** `.worktrees/<name>/` exists, its branch = `<branch_name>`, history **disjoint** from the code branch (true orphan) | integration: `git -C .worktrees/odm branch --show-current` + merge-base check | serious | open | | the core payoff |
| L-7 | **Locator written:** `[store]` (`worktree_base`/`worktree_name`/`branch_name`) appended to the code-branch `odm.toml` so slice-01 resolution now redirects | integration: read back `odm.toml`; `StoreHome::resolve` → the worktree | serious | open | | closes the loop with slice 01 |
| L-8 | **Store scaffolded:** `config.toml` (operational defaults) + empty `nodes/` written **inside** the worktree; code-branch `odm.toml` stays locator-only | integration: files present at the store root; `odm.toml` has no operational keys | serious | open | | the two-config split, realized on disk |
| L-9 | **`/.worktrees/` git-ignored** on the code branch (created/appended idempotently) | integration: `.gitignore` contains the entry; re-run doesn't duplicate | correctness | open | | worktree never staged on the code branch |
| L-10 | **`odm store init` CLI:** the `store` group + `init` child; `--dry-run` (no writes), `--yes`, `--worktree`/`--branch`, `--json` reporting the created home | integration + `--dry-run` touches-nothing assertion | serious | open | | the surface, born in its ODD-0023 home |
| L-11 | **Home is live:** after `init`, `odm new` writes under the store and `odm check` is green there | integration: create a node, `check` exit 0, file under `.worktrees/odm/nodes/` | serious | open | | end-to-end proof for SH-2 |
| L-12 | **Carried #1 — `.odm/context.json` follows the resolved store root** (rides with the store, like the `.odm/` index) | `grep` the context path resolution; integration: context.json under the store | serious | open | | slice-01 bubble-up disposed |
| L-13 | **Carried #2 — `ROLLUP.md` placement decision recorded** (stays at repo/invocation root as the code-branch projection); no rollup-path change | doc row in `closing-report.md`; `rollup` output path unchanged | correctness | open | | decision, not code |
| L-14 | **Steady-state stays on `gix`:** no new `git` subprocess outside the worktree module | `grep -rE 'Command::new\("git"\)' crates` → only the new module | correctness | open | | the exception stays narrow |
| L-15 | `cargo build` / `test` / `clippy --all-targets -- -D warnings` / `fmt` green; no `unsafe` | CC attest (local 1.85+) → CI | serious | open | | attested-by-CC → reproduced-on-CI |

**Close:** per-row walk in `closing-report.md` (CC) + `cdc-verification.md` (CDC), with the
**bubble-up to `arc-plan.md`** (did slice 02 deliver SH-2; anything the arc-plan didn't anticipate —
e.g. CI `git`-version reality, `--json` shape; the silent-drop diff).
