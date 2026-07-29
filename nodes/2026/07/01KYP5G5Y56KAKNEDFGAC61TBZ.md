---
id: 01KYP5G5Y56KAKNEDFGAC61TBZ
number: 596623400
type: artifact
schema: artifact/v1.1
name: Slice 03 — `init` attach + ff-sync (ledger)
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/slice03-init-attach-sync/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SAX5GZJ53WFTVPBWF
---
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
| L-1 | **`ExistsOnRemote` → attach** invoked (slice 02's stop is replaced) | integration: remote-only `odm` → `init` attaches | serious | **done** | integration `attach_checks_out_the_existing_branch_without_re_orphaning` | completes detection arm 2 |
| L-2 | **Attach checks out the existing branch** into the worktree via `git worktree add <store_root> <branch>` — **not** `--orphan` | integration: worktree on `<branch>`, **shares history** with the pushed `odm` (has a merge-base) | serious | **done** | same test — `branch --show-current` = `odm`, and `git merge-base odm origin/odm` **succeeds** (shared history) | the anti-re-orphan guarantee; the mirror of slice 02's orphan assertion: there, merge-base must *fail*; here it must *succeed* |
| L-3 | **Remote-only branch** → fetch + local tracking branch created, then worktree added | integration: fresh clone whose `odm` is only `origin/odm` | serious | **done** | same test — B is a fresh clone whose `odm` exists only as `origin/odm`; attach fetches, then adds the worktree | the teammate/fresh-clone case |
| L-4 | **Attach does not re-scaffold:** `config.toml` + `nodes/` ride with the branch; attach writes neither | integration: pre-existing store content survives byte-for-byte | serious | **done** | integration `attach_does_not_overwrite_the_stores_own_config` — A's custom `config.toml` (`max_width = 99`) survives, and the scaffold default is **absent** | store rides *with* the branch (ODD-0022 §4.3); asserts both halves: what survived *and* what was not written |
| L-5 | **Attach top-ups are idempotent:** `.gitignore` `/.worktrees/` + `odm.toml` locator ensured-present, never duplicated | integration: re-run adds nothing | correctness | **done** | integration `attach_top_ups_are_idempotent` — exactly one `/.worktrees/` entry | usually already committed on a clone |
| L-6 | **`ExistsLocally` → ff-sync** invoked (slice 02's stop is replaced) | integration | serious | **done** | integration `sync_fast_forwards_when_upstream_is_ahead`, `sync_is_a_clean_no_op_when_up_to_date` | completes detection arm 3 |
| L-7 | **Ancestry classification** is a pure function: up-to-date / upstream-ahead / local-ahead / diverged / no-upstream | unit test over two commit ids + upstream option | serious | **done** | unit `test_sync_action_*` ×6 — all five cases plus the `mode()` JSON contract, no repo/remote/network | the decision table, tested without a network; `sync_action(Option<Ancestry>)` is pure; git measures ancestry, the table decides |
| L-8 | **Fast-forward** when upstream is ahead: `merge --ff-only` in the worktree; the freshened nodes appear | integration: advance `origin/odm`, `init` ff's | serious | **done** | integration `sync_fast_forwards_when_upstream_is_ahead` — A's node appears in B | the useful re-init; reproduced by hand end-to-end (bootstrap → attach → ff-sync) |
| L-9 | **Diverged → warn + stop**, worktree untouched; **no** rebase/merge of the shared branch | integration: local commit + advanced upstream | **serious** | **done** | integration `sync_stops_on_divergence_and_touches_nothing` — B's HEAD unmoved, B's work intact, **A's work not merged in** | the safety invariant — never rewrite/merge a published branch; the invariant asserted three ways, not just by the message |
| L-10 | **No upstream / no remote → warn + stop** ("nothing to sync from"), exit success, no data touched | integration | serious | **done** | integration `sync_without_an_upstream_warns_and_succeeds` — exit 0, `check` still green | not an error, just nothing to do |
| L-11 | **Local-ahead** (unpushed commits, upstream not advanced) → report "N to push", **no error**, no change | integration | correctness | **done** | integration `sync_reports_local_ahead_without_erroring` — `mode: sync-local-ahead`, `local_ahead: 1` | you have work to push; don't nag as failure; success, not a nag |
| L-12 | **Up-to-date → clean no-op**, reports fresh | integration | correctness | **done** | integration `sync_is_a_clean_no_op_when_up_to_date` — HEAD identical before/after | idempotent re-init |
| L-13 | **Deferred repair** (branch present, worktree gone; half-init) → **warn clearly**, point at future `--force`; do **not** silently act | integration: delete the worktree dir, keep the branch | correctness | **deferred** | integration `a_missing_worktree_is_reported_not_silently_repaired`; unit `test_needs_repair_*` ×2 | ODD-0022 §6 (YAGNI) — detected, not fixed; **deferred by design** (ODD-0022 §6): detected and reported, pointing at `--force`; never silently fixed |
| L-14 | `--dry-run` touches nothing on **every** arm; `--json` reports `mode` (`attach`/`sync-fast-forwarded`/`sync-up-to-date`/`sync-local-ahead`/`sync-diverged`/`sync-no-upstream`) + refs/counts | integration + dry-run assertions | serious | **done** | integration `dry_run_touches_nothing_on_the_attach_arm`, `dry_run_touches_nothing_on_the_sync_arm`; `--json` modes asserted per arm | the machine contract for all arms; **deviation recorded:** a dry-run sync *does* fetch — see the closing report; without it the preview reports the wrong arm |
| L-15 | **Steady-state stays on gix:** the new `init`-time git (attach `worktree add`, ff-sync `fetch`/ancestry/`merge --ff-only`) lives only in the `init`/worktree shell-out module | `grep -rE 'Command::new\("git"\)' crates` → only that module | correctness | **done** | `grep -rn 'Command::new' crates/*/src/` → `worktree.rs` only (+ `odm-reconcile`'s pre-existing shell probe) | the §5 boundary holds at "init-only"; steady-state untouched; §5 widened from *create* to *create + attach + ff-sync*, all `init`-time; steady state still all `gix` |
| L-16 | `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe` | CC attest (local 1.85+) → CI | serious | **done** | `cargo build` / `test --all-features --workspace` (**56 binaries, 0 failed**) / `clippy --all-targets -- -D warnings` / `fmt --check` — green; no `unsafe` | attested-by-CC → reproduced-on-CI; attested-by-CC (local 1.85+, git 2.39.5) → reproduced-on-CI, which runs both git arms |

**Close:** per-row walk in `closing-report.md` (CC) + `cdc-verification.md` (CDC), with the
**bubble-up to `arc-plan.md`** (did slice 03 deliver SH-3; the §5-widening decision as recorded;
anything the arc-plan didn't anticipate; the silent-drop diff). With SH-3 closed, **SH-5** (the
bootstrap→attach→ff-sync compose demo) becomes reproducible at arc scale.
