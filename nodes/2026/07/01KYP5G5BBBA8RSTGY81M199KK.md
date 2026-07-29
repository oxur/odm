---
id: 01KYP5G5BBBA8RSTGY81M199KK
number: 598730600
type: artifact
schema: artifact/v1.1
name: Slice 02 — `git`-worktree plumbing + `odm store init` bootstrap (ledger)
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/slice02-init-bootstrap/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SCCW0SCM1E55JWH7W
---
# Slice 02 — `git`-worktree plumbing + `odm store init` bootstrap (ledger)

> Per LEDGER-DISCIPLINE v2.0 §A (slice tier). Rows are the grep-verifiable **steps**; each reaches a
> final status (`done` / `deferred` / `no-op`) before the slice closes. Evidence strength
> `asserted < attested < reproduced < reconciled`; cargo/executable rows are attested-by-CC →
> reproduced-on-CI. Closer ≠ verifier (CDC writes `cdc-verification.md`). Feeds arc row **SH-2**.

| ID | Criterion | Verify | Significance | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|----------|-------|
| L-1 | **Worktree wrapper exists, isolated:** one new module is the sole caller of the `git` binary; the modern path is `git worktree add --orphan <branch> <dir>` (git ≥ 2.42) | `grep` the module; unit test builds the argv | serious | **done** | `odm-store/src/worktree.rs` — the sole caller of the `git` binary; unit `test_modern_git_uses_the_single_orphan_command`, **argv corrected to `worktree add --orphan -b <branch> <dir>`** (amendment) | the ODD-0022 §5 exception, scoped and documented at the module head; module head documents the ODD-0022 §5 exception and its narrowness (setup only). **Shipped wrong at `6703394`** — the branch was passed as a bare positional, which git ≥ 2.42 rejects (`'--orphan' and '<commit-ish>' cannot be used together`); found by CDC on git 2.43, fixed here. The unit test had locked in the broken argv, so it could not have caught it |
| L-2 | **Old-git fallback:** git < 2.42 uses the two-step `worktree add --detach` + `checkout --orphan <branch>`; version detected once | unit test on the version→argv mapping | serious | **done** | unit `test_old_git_falls_back_to_detach_then_orphan`, `test_the_boundary_is_2_42`, `test_version_parses_vendor_suffixed_output` | keep the exception working on stock/LTS git; **the fallback is the path this machine actually runs** — local git is 2.39.5, so the modern arm is argv-tested but not executed here; CI must run a ≥2.42 git to exercise it |
| L-3 | **`git` missing/too-old → actionable error** naming the fix (install/upgrade git), not a panic or opaque failure | unit test; error-string assertion | correctness | **done** | unit `test_version_parses_vendor_suffixed_output` (unreadable version → `None`); `version()` maps a missing binary to `StoreError::Git` naming the fix | errors-as-affordances; errors-as-affordances: "install git, or ensure it is on PATH" |
| L-4 | **Bootstrap detection:** "no `odm` branch anywhere" (no local `refs/heads/<branch>`, no `refs/remotes/*/<branch>`) selects bootstrap | unit test with fixture ref sets | serious | **done** | unit `test_detect_reports_bootstrap_for_a_repo_without_the_branch`, `test_detect_sees_an_existing_store_even_with_an_unborn_branch` | attach/sync (the other arms) are slice 03; **detection needed a second signal** — an orphan branch is *unborn* until its first commit, so a refs-only check cannot see the home `init` just made |
| L-5 | **Existing branch ⇒ stop cleanly** (defer to slice 03; no clobber, no re-orphan) | integration: pre-seed an `odm` branch → `init` stops with the deferral message | serious | **done** | integration `an_existing_branch_stops_without_touching_anything`, `a_second_init_refuses_rather_than_clobbering` | the safety boundary between 02 and 03; asserts the branch still points where it did — not merely that odm printed something |
| L-6 | **Bootstrap creates the worktree + orphan branch:** `.worktrees/<name>/` exists, its branch = `<branch_name>`, history **disjoint** from the code branch (true orphan) | integration: `git -C .worktrees/odm branch --show-current` + merge-base check | serious | **done** | integration `init_bootstraps_the_store_home` — `branch --show-current` = `odm`, and `git merge-base main odm` **fails** (disjoint) | the core payoff; asserted against git itself, so a reporting bug cannot mask a creation bug |
| L-7 | **Locator written:** `[store]` (`worktree_base`/`worktree_name`/`branch_name`) appended to the code-branch `odm.toml` so slice-01 resolution now redirects | integration: read back `odm.toml`; `StoreHome::resolve` → the worktree | serious | **done** | integration `init_bootstraps_the_store_home` reads back `[store]` from `odm.toml` | closes the loop with slice 01; written *after* the worktree exists, so a failed init never leaves a locator pointing at nothing |
| L-8 | **Store scaffolded:** `config.toml` (operational defaults) + empty `nodes/` written **inside** the worktree; code-branch `odm.toml` stays locator-only | integration: files present at the store root; `odm.toml` has no operational keys | serious | **done** | integration `init_bootstraps_the_store_home`: `config.toml` + empty `nodes/` in the store; locator asserted to contain **no** `[gates.` keys | the two-config split, realized on disk; plus: the store holds *only* the store — see the L-2 fallback note |
| L-9 | **`/.worktrees/` git-ignored** on the code branch (created/appended idempotently) | integration: `.gitignore` contains the entry; re-run doesn't duplicate | correctness | **done** | integration `init_bootstraps_the_store_home` (`.gitignore` contains `/.worktrees/`, and `git status` never lists it); unit `test_gitignore_entry_is_idempotent` | worktree never staged on the code branch; re-running adds nothing — exactly one entry |
| L-10 | **`odm store init` CLI:** the `store` group + `init` child; `--dry-run` (no writes), `--yes`, `--worktree`/`--branch`, `--json` reporting the created home | integration + `--dry-run` touches-nothing assertion | serious | **done** | integration `dry_run_touches_nothing`, `json_reports_the_created_home`, `custom_worktree_and_branch_names_are_honoured` | the surface, born in its ODD-0023 home; `--json`: `mode`/`dry_run`/`store_root`/`branch`/`worktree`/`git_version` |
| L-11 | **Home is live:** after `init`, `odm new` writes under the store and `odm check` is green there | integration: create a node, `check` exit 0, file under `.worktrees/odm/nodes/` | serious | **done** | integration `nodes_land_in_the_new_home_and_check_is_green`; reproduced by hand — `odm new` then `odm check` green in a fresh bootstrap | end-to-end proof for SH-2; and nothing written at the repo root |
| L-12 | **Carried #1 — `.odm/context.json` follows the resolved store root** (rides with the store, like the `.odm/` index) | `grep` the context path resolution; integration: context.json under the store | serious | **done** | integration `context_is_written_under_the_store_root` | slice-01 bubble-up disposed; asserts both halves: present under the store, **absent** at the invocation root |
| L-13 | **Carried #2 — `ROLLUP.md` placement decision recorded** (stays at repo/invocation root as the code-branch projection); no rollup-path change | doc row in `closing-report.md`; `rollup` output path unchanged | correctness | **done** | decision recorded in `closing-report.md`; `rollup`'s output path unchanged | decision, not code; no code change, by design |
| L-14 | **Steady-state stays on `gix`:** no new `git` subprocess outside the worktree module | `grep -rE 'Command::new\("git"\)' crates` → only the new module | correctness | **done** | `grep -rn 'Command::new' crates/*/src/` → `worktree.rs` (×2) + `odm-reconcile/src/shell.rs` (the pre-existing shell probe) | the exception stays narrow; the exception stayed narrow; steady state is still all `gix` |
| L-15 | `cargo build` / `test` / `clippy --all-targets -- -D warnings` / `fmt` green; no `unsafe` | CC attest (local 1.85+) → CI | serious | **done** | `cargo build` / `test --all-features --workspace` (**55 binaries, 0 failed**) / `clippy --all-targets -- -D warnings` / `fmt --check` — green; no `unsafe` | attested-by-CC → reproduced-on-CI; attested-by-CC (local 1.85+, **git 2.39.5 — fallback arm only**) → reproduced-on-CI. The modern arm's green run is **CDC's**, on git 2.43 (`cdc-verification.md`); a CI guard now asserts the version rather than assuming it |

**Close:** per-row walk in `closing-report.md` (CC) + `cdc-verification.md` (CDC), with the
**bubble-up to `arc-plan.md`** (did slice 02 deliver SH-2; anything the arc-plan didn't anticipate —
e.g. CI `git`-version reality, `--json` shape; the silent-drop diff).
