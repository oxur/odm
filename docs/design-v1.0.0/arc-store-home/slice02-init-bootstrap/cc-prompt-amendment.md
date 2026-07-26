# cc-prompt — slice 02 **amendment**: fix the modern-git `--orphan` argv (SH-2 blocker)

> **Arc:** Store Home & `init` (`arc-store-home`) · **Slice:** 02 · **Amends:** `6703394` · **Blocks:**
> SH-2 close. **Source:** CDC verification (`slice02-init-bootstrap/cdc-verification.md`), which
> reproduced this on **git 2.43** and **verified the fix**. One-line change + one test update + a CI
> guard. Not a redo — the rest of the slice is sound.

## The defect (confirmed, reproduced on git 2.43)

`odm store init` **fails on git ≥ 2.42** (most current systems and CI). `worktree.rs::plan()` emits,
for the modern arm:

```
git worktree add --orphan <branch> <dir>
```

git reads the branch as `<path>` and the dir as `<commit-ish>` and rejects it:

```
fatal: '--orphan' and '<commit-ish>' cannot be used together
```

The branch name is **not** a bare positional in the 2.42+ synopsis
(`worktree add … [--orphan] [(-b|-B) <branch>] <path>`) — it needs `-b`. Your suite was green because
local git 2.39.5 only ever runs the **fallback** arm; the modern arm had never executed. As-shipped,
`cargo test -p oxur-odm --test store_init` fails **6 of 8** under git 2.43 (the two survivors never
reach worktree creation). This is exactly the CI-git-version risk you flagged — the argv it was
"verified at" is the one that doesn't run.

## The fix (verified — all 8 pass on git 2.43)

**1. `crates/odm-store/src/worktree.rs`, `plan()` — the `has_orphan_flag()` branch:**

```diff
-            add: vec!["worktree".into(), "add".into(), "--orphan".into(), branch.into(), dir],
+            add: vec![
+                "worktree".into(),
+                "add".into(),
+                "--orphan".into(),
+                "-b".into(),
+                branch.into(),
+                dir,
+            ],
```

**2. Same file, `test_modern_git_uses_the_single_orphan_command` — insert `"-b"` before `"odm"`** so
the unit test asserts the *correct* argv (it currently locks in the broken one):

```diff
                     "add".into(),
                     "--orphan".into(),
+                    "-b".into(),
                     "odm".into(),
                     "/w/odm".into()
```

Nothing else changes — `DetachThenOrphan` (fallback), `detect`, scaffolding, and the locator are all
correct.

## Acceptance

- `cargo test -p oxur-odm --test store_init` → **8 passed, 0 failed**, run on **git ≥ 2.42**. This is
  the load-bearing rerun: on git < 2.42 it exercises only the fallback and cannot prove the fix.
  - You are on git 2.39.5 locally — either install git ≥ 2.42 for this rerun, or rely on the CI guard
    below plus CDC's independent reproduction (all 8 green on 2.43, recorded in `cdc-verification.md`).
- `cargo build`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.

## CI guard (do this here, not later — it's why the bug hid)

Make the git version an **explicit, asserted** part of CI, not an ambient assumption:

- Pin/assert **git ≥ 2.42** on the job that runs `store_init` (fail fast with a clear message if it
  isn't), so the modern arm is always the one under test.
- **Better: a two-version matrix** — one job on git ≥ 2.42 (modern arm) and one on a git < 2.42
  (fallback arm). Bug 2 already proved the two arms can diverge; a matrix is what keeps "both arms
  produce an identical empty store" honest going forward. If a stock old-git image is inconvenient,
  at minimum keep the unit-level `plan()` assertions for both arms (they now encode the right argv)
  and run the integration test on modern git.

## Ledger / close

- Flip **L-1** (modern `--orphan` path) to done with the corrected argv; **L-15** reproduces green on
  git ≥ 2.42.
- **SH-2 → `attested`** on close (was blocked); **`reproduced` on CI** once the guard runs it on git
  ≥ 2.42.
- Bubble up to `arc-plan.md` (SH-2): note the CI-git-version guard as the anticipated-but-unguarded
  risk this slice surfaced, and the two-arm matrix as the standing mitigation.

## Method

Same branch (`sh-slice02-init-bootstrap`), one small amend-style commit on top of `6703394`; the CDC
verification stands as the independent reproduction, so no re-verification round is needed for a
one-line argv fix — just confirm the acceptance rerun (or the CI guard) is green on modern git.
