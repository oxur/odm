# Slice 04 (arc-store-home) — CDC verification + arc-close endorsement

> **Verifies:** SH-4 (+ SH-5 compose, + the arc close) · **Slice:** `odm store rename` · **Branch:**
> `sh-slice04-store-rename` (`6d6b839`) · **Date:** 2026-07-26 · **Verifier:** CDC, independent of CC —
> clean container clone on **git 2.43.0**, cargo 1.95. The rename git ops (`worktree move`, `branch -m`)
> are not `--orphan`-gated; a single modern-git run reproduces them fully.

## Verdict

**Slice 04 delivered; SH-4 reproduced. The arc is code-complete and compose-verified; SH-6 correctly
open.** No defects. The bug the slice found — a green `check` over an *invisible corpus* — is the exact
failure this arc most needed to catch, and it's caught, guarded, and reproduced. I independently ran the
full store suite (33 integration + 41 unit) and walked the SH-5 composition end-to-end on git 2.43.
This is the strongest of the four slices: the safety invariant I asked for in the prompt ("locator
mirrors observed state **even on failure**") is exactly what surfaced the bug and exactly what fixes it.

## Reproduced by CDC (git 2.43)

- **Rename suite green:** `store_rename` **14/14**, `store_init` **8/8**, `store_attach_sync` **11/11**
  (33 integration), `odm-store --lib` **41/41** (incl. the rename decision-table units).
- **The invisible-corpus fix — verified two ways.** (a) The regression test
  `a_partial_failure_still_leaves_the_store_resolvable` pins it: a branch-rename failure after a
  worktree move must still leave the corpus resolvable, with the error naming what succeeded — green.
  (b) In code, `rename()` **holds** `branch_result`, writes the locator from **observed** git state
  (`path_of_branch` / `current_branch`), and only *then* returns the error — so the locator can never
  name a store that isn't there. The comment at the branch-result site names the failure precisely.
- **SH-5 composition — independently walked** (bootstrap in A → push to a bare remote → attach in B →
  add a node in A + push → ff-sync in B → `store rename planning` in B → `check`):

  ```
  bootstrap ✓ /A/.worktrees/odm on orphan "odm"        attach   ✓ B attached to existing "odm"
  ff-sync   ✓ fast-forwarded "odm" to origin/odm         rename   ✓ …/odm → …/planning, "odm" → "planning"
  check     ✓ ok (2 node(s), no problems)   ← 2, not 0
  ```

  The decisive detail: after the rename, `check` is green over **2 nodes at the renamed home** — the
  real corpus — not the green-over-zero the invisible-corpus bug would have produced. The
  published-branch warning also fired (B's `odm` tracks `origin/odm`), local-only, as designed.
- **§5 boundary holds.** Every `Command::new("git")` is still in `worktree.rs`; the six rename helpers
  (`move_worktree`, `rename_branch`, `path_of_branch`, `branch_exists`, `current_branch`, `is_published`)
  all live there. Steady-state stays on gix. The widening (create → +attach/ff-sync → +rename) is
  documented at the module head and stayed one module wide, exactly as §5 named `init`/rename.

## CC's git-probing findings — endorsed

1. **`git worktree move` behaves like `mv`** (moves *into* an existing dir, reports success) → odm
   pre-checks collisions and writes the locator from where the tree *landed*, not where it was asked to
   go. Correct: git won't refuse the collision, so odm must, and the observed-path write is what makes
   even a surprising landing safe. Verified by `a_collision_with_an_existing_directory_stops…`.
2. **`branch -m` works on an unborn branch** → a freshly bootstrapped store needs no special case.
   Sound, and it's why bare-init-then-rename works.

Both were found by *probing real git before designing the flow* — the same "exercise the path in the
environment that will run it" discipline whose absence caused the slice-02 argv defect. The right lesson,
applied.

## The arc close — endorsed (code-complete), with the gate deferred to SH-6

The `closing-report.md` is exemplary: the four-slice ledger, the SH-5 reproduction (matching mine), a
genuinely useful retrospective (**the three bugs share one shape — a path unexercised in its authoring
environment, or success and failure looking identical; the fix each time is *assert both halves***), every
decision recorded, and the carried items named. **SH-6 is honestly open, not dropped:** odm still lives in
`nodes/` on the working branch via the back-compat path, and the cutover rides **RH C-5** deliberately —
C-5 already rewrites `migrate`/`self-host` and must fix F-20 (creation-date loss) in the same pass, so the
corpus is rewritten once, not twice. That sequencing is correct and I endorse it.

**One scope note on the close, not an objection.** A4/A5 each closed with an *independent fresh-context
arc-gate review* (PASS-WITH-NOTES). arc-store-home is **code-complete, not fully closed** — SH-6 is open —
so the formal arc-gate belongs at the **true** close, when the C-5 cutover lands and SH-6 reproduces. My
independent-environment reproduction of SH-1…SH-5 is the verification for *this* milestone; the
fresh-context arc-gate runs with the SH-6/C-5 join. Flagging so the deferred gate isn't forgotten, not
asking for one prematurely.

## Not a defect — recorded

`cargo test -p odm-store` shows the same 2 `edge_cases` failures as prior slices
(`atomic_write_temp_failure_in_readonly_dir`, `load_all_surfaces_unreadable_dir`) — pre-existing,
root-only (euid 0 bypasses the `PermissionsExt` readonly denial), untouched by slice 04. CC's
`--all-features --workspace` on a non-root machine reports 57 binaries, 0 failed.

## Carried items (from the closing report) — all legitimate, non-blocking

`--yes` accepted but unused on every `store` subcommand; `sync` assumes the upstream remote is `origin`;
`--json`'s `worktree` duplicates `store_root`; **`worktree_base` is not renameable** (new this slice —
rename moves `worktree_name`/`branch_name` but not the `.worktrees` parent, a fine scope limit). Record
for whoever picks up the `store` group; none blocks the close.

## Ledger

- **SH-4 → reproduced (CDC, git 2.43).** SH-5 (compose) reproduced independently. SH-1…SH-4 closed;
  **SH-6 open, pending RH C-5** (the dogfood cutover) — correctly deferred, on the arc ledger and P-14.
- **Silent-drop diff:** none. The carried items and SH-6 are all recorded.
- **Durable `reproduced` on CI** flips when the branches push and both git-arm jobs run green (push
  still blocked — origin is SSH, unreachable from the cloud session).
- **The arc is code-complete + compose-verified.** Full arc-gate + true close ride the SH-6/C-5 cutover.
