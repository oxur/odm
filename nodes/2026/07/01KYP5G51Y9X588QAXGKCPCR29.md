---
id: 01KYP5G51Y9X588QAXGKCPCR29
number: 527674100
type: artifact
schema: artifact/v1.1
name: Slice 02 (arc-store-home) — CDC verification
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/slice02-init-bootstrap/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SCCW0SCM1E55JWH7W
---
# Slice 02 (arc-store-home) — CDC verification

> **Verifies:** SH-2 · **Slice:** `git`-worktree plumbing + `odm store init` bootstrap · **Branch:**
> `sh-slice02-init-bootstrap` (`6703394`, off slice-01 tip) · **Date:** 2026-07-26 · **Verifier:**
> CDC, independent of CC — **in a different environment**: a clean container clone (via git bundle)
> running **git 2.43.0** and cargo 1.95, i.e. the git ≥ 2.42 that CC's local 2.39.5 could not exercise.

## Verdict

**Slice 02 is one iteration short — a single confirmed, high-severity defect blocks it.** The modern
`git worktree add --orphan` argv is **wrong**, so `odm store init` **fails on git ≥ 2.42** — most
current systems and CI. I reproduced it (6 of 8 integration tests fail under git 2.43), root-caused
it, and **verified the one-line fix** (all 8 pass). Everything *else* in the slice is sound and often
sharp — including the two bugs CC caught in-slice. **SH-2 cannot attest as delivered** until the argv
fix lands; with it, the slice is complete. This is a "one more commit," not a redo.

**The finding vindicates CC's own flag exactly.** CC wrote: *"the modern `--orphan` arm is verified
only at the argv level … the CI git version is load-bearing evidence, not an environment detail."*
Precisely so — and the argv it was verified against is the one that doesn't run. An independent
environment on git 2.43 is what surfaced it; this is the whole reason "closer ≠ verifier" and
"reproduce in a different environment" are in the discipline.

## The defect (CONFIRMED, reproduced)

`worktree.rs::plan()` emits, for git ≥ 2.42:

```
git worktree add --orphan <branch> <dir>      # e.g. …--orphan odm /repo/.worktrees/odm
```

git parses the two positionals as `<path>=odm` and `<commit-ish>=/repo/.worktrees/odm` and rejects it:

```
fatal: '--orphan' and '<commit-ish>' cannot be used together
```

The 2.42+ synopsis is `git worktree add … [--orphan] [(-b|-B) <branch>] <path> [<commit-ish>]` — the
branch name is **not** a bare positional; it needs `-b`. Verified directly on git 2.43: `--orphan -b
odm .worktrees/odm` creates the orphan branch `odm`, empty of code files, history disjoint from
`main`. The fallback arm (`add --detach` → `checkout --orphan` → `rm -rf`) is correct and unaffected —
which is exactly why CC's suite was green: **local git 2.39.5 only ever ran the fallback.**

### Reproduction (git 2.43, this container)

`cargo test -p oxur-odm --test store_init`, branch as-shipped:

```
test result: FAILED. 2 passed; 6 failed
  failed: init_bootstraps_the_store_home, nodes_land_in_the_new_home_and_check_is_green,
          json_reports_the_created_home, context_is_written_under_the_store_root,
          custom_worktree_and_branch_names_are_honoured, a_second_init_refuses_rather_than_clobbering
  passed: dry_run_touches_nothing (never calls git),
          an_existing_branch_stops_without_touching_anything (returns before worktree creation)
```

Every failure is the same `--orphan`/`<commit-ish>` error. The two survivors are the paths that never
reach worktree creation — which is why old-git CI *and* those two tests hid it.

### The fix (VERIFIED — all 8 pass)

```diff
 // worktree.rs::plan(), the version.has_orphan_flag() branch
-add: vec!["worktree".into(), "add".into(), "--orphan".into(), branch.into(), dir],
+add: vec!["worktree".into(), "add".into(), "--orphan".into(), "-b".into(), branch.into(), dir],
```

…and the matching one-line update to `test_modern_git_uses_the_single_orphan_command` (insert `"-b"`
before `"odm"`). With this, under git 2.43:

```
cargo test -p oxur-odm --test store_init   → 8 passed; 0 failed
```

`-b` is the *only* change needed; the DetachThenOrphan fallback, detection, scaffolding, and locator
logic are untouched and correct.

## Checks (reproduced by CDC)

| Row | Result |
|-----|--------|
| L-1 (modern `--orphan` path) | **FAIL as shipped** — argv wrong; `odm store init` broken on git ≥ 2.42. **PASS after `-b`** (reproduced on git 2.43). |
| L-2 (version→argv mapping) / L-3 (git missing error) | **PASS** — `GitVersion::parse` tolerant of vendor suffixes; boundary at 2.42; `version()` returns an actionable `StoreError` naming the fix. (The unit test asserted the *wrong* modern argv, so it passed against a broken command — the gap was test-vs-git, not logic.) |
| L-4 (bootstrap detection) / L-5 (existing branch stops) | **PASS** — `detect` is a pure gix read; existing-branch case stops with the "slice 03" message and leaves the branch un-re-orphaned (verified). |
| **Bug 1 — unborn-branch detection** | **PASS — good catch.** `detect` checks `store_root.exists()` *before* the ref, because `checkout --orphan` leaves `refs/heads/<branch>` absent until the first commit; dir-then-ref covers both blind spots. Correct and well-reasoned. |
| **Bug 2 — fallback carried the checkout** | **PASS — good catch.** `checkout --orphan` keeps the working tree; the `rm -rf --ignore-unmatch --quiet .` clear step makes the fallback match the modern empty-store. The integration test now asserts the *negative* (README.md absent, nothing staged) — the both-halves discipline restored. |
| L-6…L-9 (worktree/orphan, locator, scaffold, gitignore) | **PASS after `-b`** — worktree on the orphan branch, disjoint `merge-base`, `[store]` locator appended (existing keys preserved), `config.toml` + empty `nodes/` in the store, `/.worktrees/` gitignored idempotently. Reproduced on git 2.43 post-fix. |
| L-10 (`--dry-run`/`--json`) | **PASS** — dry-run touches nothing (verified: no worktree/locator/gitignore/branch); `--json` reports `mode:"bootstrap"`, branch, store_root, git_version. |
| L-11 (home is live) | **PASS after `-b`** — `odm new` writes under the store, nothing at the repo root, `odm check` exit 0. |
| L-12 (`.odm/context.json` follows the store — carried #1) | **PASS after `-b`** — context.json under the store root, not the invocation root; `context` reads it back. Correct disposition. |
| L-13 (ROLLUP.md at repo root — carried #2) | **PASS** — decision recorded; no rollup-path change. Correct. |
| L-14 (steady-state stays on gix) | **PASS (reproduced)** — the `git` subprocess lives only in `worktree.rs`; `detect` and all reads are gix. Boundary held. |
| L-15 (build/test/clippy/fmt) | **attested-by-CC on git 2.39.5** (fallback arm) → the modern arm was **never executed**. Post-`-b`, reproduced green on git 2.43. **CI must run git ≥ 2.42** to keep it that way. |

## Not a defect — recorded so it isn't re-chased

`cargo test -p odm-store` shows 2 failures here — `atomic_write_temp_failure_in_readonly_dir`,
`load_all_surfaces_unreadable_dir` (in `tests/edge_cases.rs`). Both are **pre-existing** (0 hits in the
slice-02 diff) and fail only because **this container runs as root** (euid 0), which bypasses the
`PermissionsExt` readonly denial they assert. Environment artifact, not slice-02 code.

## Decisions CC recorded — endorsed

1. **Detection is dir-then-ref, both checked.** The unborn-branch window (bug 1) means neither signal
   alone suffices; checking the store directory first and the ref second is the right call, and the
   safety framing ("re-orphaning a branch with someone's data would destroy it") is sound.
2. **Bootstrap orders worktree → locator → scaffold → gitignore**, so a failed `init` never leaves an
   `odm.toml` pointing at nothing. Plus a belt-and-suspenders `store_root.exists()` refusal. Good.
3. **CC left the CDC `uat-coverage-audit.md` routing unstaged** rather than folding another author's
   work into the slice commit. Correct ledger hygiene — that change is mine to commit.

## Ledger

- **SH-2 → stays `open`** (cannot attest as delivered): the slice's headline capability fails on the
  majority git. → after the `-b` fix + re-run on git ≥ 2.42, **SH-2 `attested`**; **`reproduced` on
  CI** — and **CI must pin git ≥ 2.42**, now doubly load-bearing (it is the only arm that catches this
  class, and the two arms must produce identical stores — bug 2's lesson).
- **Silent-drop diff:** none. Both carried items (context-follows-store, ROLLUP placement) disposed as
  planned.
- **Next:** CC applies the one-line `-b` fix (+ the unit-test assertion), re-runs `store_init` on git
  ≥ 2.42 to reproduce all 8 green, closes SH-2. Then slice 03 (attach + ff-sync).

---

## Amendment sign-off — `e6d24bb` (2026-07-26, CDC)

**Verified. SH-2 clears to `attested` (amended); `reproduced` flips when CI runs both git jobs green.**

Re-pulled the branch at `e6d24bb` into the container clone and re-ran on **git 2.43**:

- **The `-b` fix matches the verified patch** — `plan()` now emits `worktree add --orphan -b <branch>
  <dir>`, with an inline comment explaining why the branch is never a bare positional. The unit test
  asserts the corrected argv (it previously pinned the broken one — a test that reads as coverage
  while locking in the bug).
- **`cargo test -p oxur-odm --test store_init` → 8 passed, 0 failed** on CC's committed tree. The
  modern arm is now reproduced on real git ≥ 2.42, not just argv-checked.
- **The CI guard is sound, and it is the part that matters.** Two jobs, each asserting its own git
  version: `test` fails fast below 2.42 (so the modern `--orphan` arm is always the one under test),
  and `test-old-git` pins `debian:bullseye` (git 2.30) and *confirms* it is pre-2.42 before running
  `store_init` on the fallback arm. The version parse (`major`/`minor` off `git --version`) is correct
  across 2.39/2.42/2.43/3.0; the backtick command-substitution bug CC caught in its own guard is gone.
  This makes both arms load-bearing — which is the right fix, since bug 2 proved they can diverge.

**Endorsed, without reservation.** CC corroborated the `-b` requirement independently from the git
synopsis rather than only taking the reproduction; kept the original "⚠ git version" caveat standing
with a note that its predicted gap is where the defect lived (prediction + outcome together beat a
tidied account); and was explicit that local green still only covers the fallback, so the modern arm's
evidence is CDC's 2.43 run plus the CI job. That is exactly the honesty the ledger wants.

**The one thing neither of us can close from here:** the CI jobs have not *run* — the branch is
unpushed (origin is SSH; the device bridge can't reach github:22). Both arms are independently green
(CC's local 2.39.5 fallback + CDC's 2.43 modern), so the evidence exists; CI is what will keep it
true. `reproduced` is earned when the branch pushes and both jobs pass.
