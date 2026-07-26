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
| L-1 | `odm store rename` exists under the `store` group: bare positional (both) + `--worktree` / `--branch` / `--dry-run` / `--yes` / `--json` | integration; `--help` | serious | open | | distinct from `odm node rename` (ODD-0023) |
| L-2 | **Worktree-dir rename** via `git worktree move <old> <new>`: `.worktrees/<new>` exists, `.worktrees/<old>` gone, git worktree metadata consistent | integration: `git worktree list` shows the new path | serious | open | | not `--orphan`-gated |
| L-3 | **Local branch rename** via `git branch -m`: the worktree is on `<new>`, upstream config preserved | integration: `branch --show-current` = new; `@{upstream}` intact | serious | open | | |
| L-4 | **Locator updated**: `[store]` `worktree_name`/`branch_name` rewritten; `StoreHome::resolve` finds the store at the new location | integration: read back `odm.toml`; `odm list`/`check` green at the new path | serious | open | | closes the loop with slice 01 |
| L-5 | **Safety invariant — the store is always resolvable.** git ops first, locator written **last** to mirror the **observed** git state; a git failure before the write changes nothing (old locator valid); a partial failure leaves the locator matching reality + a clear report | integration: inject a failing second op → resolution still works; unit on the mirror mapping | **serious** | open | | the whole point — a rename must never orphan the corpus |
| L-6 | **Published-branch guard**: renaming a branch with an upstream / on a remote **warns** it is local-only (remote keeps the old name); does **not** attempt a remote rename | integration: bootstrap→push→`--branch` rename → warning; `origin/<old>` still present | serious | open | | shared-branch rename is a coordination event, out of scope (§6-style deferral) |
| L-7 | **Collision**: target worktree dir or branch already exists → **stop, don't clobber**, name the collision | integration | serious | open | | never overwrite an existing worktree/branch |
| L-8 | **No-op**: renaming to the current name → clean no-op, clear message, nothing touched | integration | correctness | open | | idempotent |
| L-9 | **Dirty store**: uncommitted changes → `git worktree move` preserves the working tree, or git's refusal is surfaced verbatim (not forced) | integration | correctness | open | | don't lose the operator's in-flight work |
| L-10 | `--dry-run` reports old→new (worktree / branch / locator) and touches nothing; `--json` reports old/new worktree, branch, store_root | integration + dry-run assertions | serious | open | | the machine contract |
| L-11 | **Steady-state stays on gix**: the rename git ops live only in `worktree.rs` | `grep -rE 'Command::new\("git"\)' crates` → only that module | correctness | open | | §5 boundary (rename is named in the exception) |
| L-12 | `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe` | CC attest (local 1.85+) → CI | serious | open | | attested-by-CC → reproduced-on-CI |

**Close:** per-row walk in `closing-report.md` (CC) + `cdc-verification.md` (CDC), with the **bubble-up
to `arc-plan.md`** (did slice 04 deliver SH-4; anything unanticipated — e.g. `worktree move` behaviour
on the running git; silent-drop diff). With SH-1…SH-4 closed and SH-5 (three-mode `init`) reproducible,
this unlocks **`arc-store-home/closing-report.md`** — the arc closes code-complete + compose-verified,
**SH-6 (dogfood cutover) explicitly pending RH C-5** (recorded, not dropped).
