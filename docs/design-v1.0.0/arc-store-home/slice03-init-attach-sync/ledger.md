# Slice 03 — `init` attach + ff-sync (ledger)

> Per LEDGER-DISCIPLINE v2.0 §A (slice tier). Rows are the grep-verifiable **steps**; each reaches a
> final status (`done` / `deferred` / `no-op`) before the slice closes. Evidence strength
> `asserted < attested < reproduced < reconciled`; cargo/executable rows are attested-by-CC →
> reproduced-on-CI. Closer ≠ verifier (CDC writes `cdc-verification.md`). Feeds arc row **SH-3**.
>
> **Note (from slice 02):** the git-path-sensitive rows must be exercised where the git actually runs.
> attach's `worktree add <dir> <branch>` (checkout of an existing branch) is **not** version-gated the
> way bootstrap's `--orphan` was, but the integration rows still need a **real git + a real remote**
> (a local bare repo as `origin`), not a mock.

| ID | Criterion | Verify | Significance | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|----------|-------|
| L-1 | **`ExistsOnRemote` → attach** invoked (slice 02's stop is replaced) | integration: remote-only `odm` → `init` attaches | serious | open | | completes detection arm 2 |
| L-2 | **Attach checks out the existing branch** into the worktree via `git worktree add <store_root> <branch>` — **not** `--orphan` | integration: worktree on `<branch>`, **shares history** with the pushed `odm` (has a merge-base) | serious | open | | the anti-re-orphan guarantee |
| L-3 | **Remote-only branch** → fetch + local tracking branch created, then worktree added | integration: fresh clone whose `odm` is only `origin/odm` | serious | open | | the teammate/fresh-clone case |
| L-4 | **Attach does not re-scaffold:** `config.toml` + `nodes/` ride with the branch; attach writes neither | integration: pre-existing store content survives byte-for-byte | serious | open | | store rides *with* the branch (ODD-0022 §4.3) |
| L-5 | **Attach top-ups are idempotent:** `.gitignore` `/.worktrees/` + `odm.toml` locator ensured-present, never duplicated | integration: re-run adds nothing | correctness | open | | usually already committed on a clone |
| L-6 | **`ExistsLocally` → ff-sync** invoked (slice 02's stop is replaced) | integration | serious | open | | completes detection arm 3 |
| L-7 | **Ancestry classification** is a pure function: up-to-date / upstream-ahead / local-ahead / diverged / no-upstream | unit test over two commit ids + upstream option | serious | open | | the decision table, tested without a network |
| L-8 | **Fast-forward** when upstream is ahead: `merge --ff-only` in the worktree; the freshened nodes appear | integration: advance `origin/odm`, `init` ff's | serious | open | | the useful re-init |
| L-9 | **Diverged → warn + stop**, worktree untouched; **no** rebase/merge of the shared branch | integration: local commit + advanced upstream | **serious** | open | | the safety invariant — never rewrite/merge a published branch |
| L-10 | **No upstream / no remote → warn + stop** ("nothing to sync from"), exit success, no data touched | integration | serious | open | | not an error, just nothing to do |
| L-11 | **Local-ahead** (unpushed commits, upstream not advanced) → report "N to push", **no error**, no change | integration | correctness | open | | you have work to push; don't nag as failure |
| L-12 | **Up-to-date → clean no-op**, reports fresh | integration | correctness | open | | idempotent re-init |
| L-13 | **Deferred repair** (branch present, worktree gone; half-init) → **warn clearly**, point at future `--force`; do **not** silently act | integration: delete the worktree dir, keep the branch | correctness | **deferred** | | ODD-0022 §6 (YAGNI) — detected, not fixed |
| L-14 | `--dry-run` touches nothing on **every** arm; `--json` reports `mode` (`attach`/`sync-fast-forwarded`/`sync-up-to-date`/`sync-local-ahead`/`sync-diverged`/`sync-no-upstream`) + refs/counts | integration + dry-run assertions | serious | open | | the machine contract for all arms |
| L-15 | **Steady-state stays on gix:** the new `init`-time git (attach `worktree add`, ff-sync `fetch`/ancestry/`merge --ff-only`) lives only in the `init`/worktree shell-out module | `grep -rE 'Command::new\("git"\)' crates` → only that module | correctness | open | | the §5 boundary holds at "init-only"; steady-state untouched |
| L-16 | `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe` | CC attest (local 1.85+) → CI | serious | open | | attested-by-CC → reproduced-on-CI |

**Close:** per-row walk in `closing-report.md` (CC) + `cdc-verification.md` (CDC), with the
**bubble-up to `arc-plan.md`** (did slice 03 deliver SH-3; the §5-widening decision as recorded;
anything the arc-plan didn't anticipate; the silent-drop diff). With SH-3 closed, **SH-5** (the
bootstrap→attach→ff-sync compose demo) becomes reproducible at arc scale.
