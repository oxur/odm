---
id: 01KYP5G6GW9CAMK4WCC3S72D7W
number: 521488700
type: artifact
schema: artifact/v1.1
name: Slice 04 — `odm store rename` (ledger)
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/slice04-store-rename/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SDKC6SHKGNN5VHEXK
---
# Slice 04 — `odm store rename` (ledger)

> Per LEDGER-DISCIPLINE v2.0 §A (slice tier). Rows are the grep-verifiable **steps**; each reaches a
> final status (`done` / `deferred` / `no-op`) before the slice closes. Evidence strength
> `asserted < attested < reproduced < reconciled`; cargo/executable rows are attested-by-CC →
> reproduced-on-CI. Closer ≠ verifier (CDC writes `cdc-verification.md`). Feeds arc row **SH-4**.
>
> **Note:** the rename git ops (`worktree move`, `branch -m`) are **not** `--orphan`-version-gated, but
> the integration rows still need a **real git** (and a bare remote for the published-branch row). The
> slice-02 CI matrix already runs the store suite on both git arms.

| ID | Criterion | Verify | Significance | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|----------|-------|
| L-1 | `odm store rename` exists under the `store` group: bare positional (both) + `--worktree` / `--branch` / `--dry-run` / `--yes` / `--json` | integration; `--help` | serious | **done** | integration `store_rename.rs` ×14; `odm store rename --help` | distinct from `odm node rename` (ODD-0023); bare positional + `--worktree`/`--branch`/`--dry-run`/`--yes`/`--json`, under the `store` group |
| L-2 | **Worktree-dir rename** via `git worktree move <old> <new>`: `.worktrees/<new>` exists, `.worktrees/<old>` gone, git worktree metadata consistent | integration: `git worktree list` shows the new path | serious | **done** | integration `renaming_the_worktree_moves_it_and_updates_the_locator` — new path exists, old gone, and **`git worktree list` agrees** | not `--orphan`-gated; git's own view asserted, not just the filesystem's |
| L-3 | **Local branch rename** via `git branch -m`: the worktree is on `<new>`, upstream config preserved | integration: `branch --show-current` = new; `@{upstream}` intact | serious | **done** | integration `renaming_the_branch_updates_the_worktree_and_the_locator`; `branch -m` verified to work on an **unborn** branch (the freshly-bootstrapped case) |  |
| L-4 | **Locator updated**: `[store]` `worktree_name`/`branch_name` rewritten; `StoreHome::resolve` finds the store at the new location | integration: read back `odm.toml`; `odm list`/`check` green at the new path | serious | **done** | integration: every rename test ends in `resolves_with_the_corpus` — `list` still shows the node and `check` is green at the new path | closes the loop with slice 01; the locator and git are asserted together, never separately |
| L-5 | **Safety invariant — the store is always resolvable.** git ops first, locator written **last** to mirror the **observed** git state; a git failure before the write changes nothing (old locator valid); a partial failure leaves the locator matching reality + a clear report | integration: inject a failing second op → resolution still works; unit on the mirror mapping | **serious** | **done** | integration `a_partial_failure_still_leaves_the_store_resolvable`; unit `test_observed_location_takes_the_name_from_the_real_path`, `test_locator_rewrite_*` | the whole point — a rename must never orphan the corpus; **a real bug was found and fixed here** — see the closing report: the first cut skipped the locator write when the branch rename failed, leaving a green `check` over an invisible corpus |
| L-6 | **Published-branch guard**: renaming a branch with an upstream / on a remote **warns** it is local-only (remote keeps the old name); does **not** attempt a remote rename | integration: bootstrap→push→`--branch` rename → warning; `origin/<old>` still present | serious | **done** | integration `renaming_a_published_branch_warns_that_it_is_local_only` — warns, renames locally, `origin/<old>` still present and no `origin/<new>` invented | shared-branch rename is a coordination event, out of scope (§6-style deferral) |
| L-7 | **Collision**: target worktree dir or branch already exists → **stop, don't clobber**, name the collision | integration | serious | **done** | integration `a_collision_with_an_existing_directory_stops_and_changes_nothing`, `a_collision_with_an_existing_branch_stops`; unit ×3 | never overwrite an existing worktree/branch; **odm must pre-check**: `git worktree move` does *not* refuse an occupied destination — it moves *into* it, `mv`-style, and reports success |
| L-8 | **No-op**: renaming to the current name → clean no-op, clear message, nothing touched | integration | correctness | **done** | integration `renaming_to_the_current_name_is_a_clean_no_op`; unit `test_renaming_to_the_current_names_is_a_no_op` | idempotent; the locator is not even rewritten |
| L-9 | **Dirty store**: uncommitted changes → `git worktree move` preserves the working tree, or git's refusal is surfaced verbatim (not forced) | integration | correctness | **done** | integration `uncommitted_work_survives_the_move` — an uncommitted file arrives byte-for-byte | don't lose the operator's in-flight work; `git worktree move` preserves the working tree; verified rather than assumed |
| L-10 | `--dry-run` reports old→new (worktree / branch / locator) and touches nothing; `--json` reports old/new worktree, branch, store_root | integration + dry-run assertions | serious | **done** | integration `dry_run_reports_and_touches_nothing`, `json_reports_old_and_new`, `json_reports_a_collision_rather_than_claiming_success` | the machine contract; `--json` reports the **observed** new path, and a collision reports `outcome: collision` rather than claiming success |
| L-11 | **Steady-state stays on gix**: the rename git ops live only in `worktree.rs` | `grep -rE 'Command::new\("git"\)' crates` → only that module | correctness | **done** | `grep -rn 'Command::new' crates/*/src/` → `worktree.rs` only (+ `odm-reconcile`'s pre-existing shell probe) | §5 boundary (rename is named in the exception); §5 already names rename; the boundary is unchanged at setup-time-only |
| L-12 | `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe` | CC attest (local 1.85+) → CI | serious | **done** | `cargo build` / `test --all-features --workspace` (**57 binaries, 0 failed**) / `clippy --all-targets -- -D warnings` / `fmt --check` — green; no `unsafe` | attested-by-CC → reproduced-on-CI; attested-by-CC (local 1.85+, git 2.39.5) → reproduced-on-CI |

**Close:** per-row walk in `closing-report.md` (CC) + `cdc-verification.md` (CDC), with the **bubble-up
to `arc-plan.md`** (did slice 04 deliver SH-4; anything unanticipated — e.g. `worktree move` behaviour
on the running git; silent-drop diff). With SH-1…SH-4 closed and SH-5 (three-mode `init`) reproducible,
this unlocks **`arc-store-home/closing-report.md`** — the arc closes code-complete + compose-verified,
**SH-6 (dogfood cutover) explicitly pending RH C-5** (recorded, not dropped).
