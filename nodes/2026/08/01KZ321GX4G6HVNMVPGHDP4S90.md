---
id: 01KZ321GX4G6HVNMVPGHDP4S90
number: 567811600
type: artifact
schema: artifact/v1.1
name: Slice 04 — CDC verification
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-lifecycle/slice04-commit-index-consistency/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-03
edges:
  part_of: 01KZ321DTAZRXCZBCA7XNKZ35K
---
# Slice 04 — CDC verification

**Method:** LEDGER-DISCIPLINE v2.0 §A. **Verdict: PASS.** Structural rows reproduced
by direct code read; the F-1 acceptance anchor is backed by a **real-git integration
test** (not a unit stub); `make check`/cargo attested → CI. Code committed
(`e8e69de`, atop `e921138`). Verified 2026-08-02.

## Verdict

**PASS** — the cleanest fix of the session. Minimal, correct by construction, and it
ships the exact regression test SL-1 lacked. F-1's real leg (a clean `git status`
after `store commit` on the live store) rides the operator's next commit with the
rebuilt binary, but it is already exercised against real git in the test suite.

## Ledger walk (F-1…F-6)

- **F-1 — reproduced (code + real-git test).** `commit_all` now passes the **same
  `tree` object** to both `commit_as(...)` and `sync_index_to_tree(tree)`; the latter
  is `repo.index_from_tree(&tree)` + `index.write(...)`, so the on-disk index is
  written from exactly the committed tree → index == HEAD by construction → clean
  `git status`. The test `commit_leaves_a_raw_git_status_clean` runs a real
  `git status --porcelain` subprocess and asserts it is empty right after
  `store commit` — a true reproduction of the bug scenario, the assertion SL-1 never
  had.
- **F-2 — reproduced.** SL-1's tests are unmodified and present (the json-shape,
  delta-by-type, `-m` override, no-op, dry-run tests all remain); commit content is
  unchanged (`write_tree`/`commit_as` untouched; the sync is post-commit and
  read-only w.r.t. the tree). Cargo run of the suite attested → CI.
- **F-3 — reproduced (by construction + dedicated test).** Because the index is
  written from the *same* tree the commit used, the ODD-0022 `.odm/` exclusion (baked
  into `write_tree`) holds for the index automatically — no second filter to drift.
  Test `a_gitignored_odm_cache_never_lands_in_the_index` seeds a `.odm/` cache and
  asserts it is absent from the index.
- **F-4 — reproduced.** The no-op-on-clean path is covered (pre-existing +
  new-clean-index coverage); a second `store commit` makes no commit and leaves a
  clean index.
- **F-5 — reproduced.** `delta.rs` is provably untouched — `git show e8e69de --
  delta.rs` returns empty (verified, not just reviewed). The odm-aware
  status/summary path is unchanged; the index sync is additive.
- **F-6 — attested → CI.** `make format`/`lint`/`test` green per CC; not reproducible
  in the CDC sandbox (no toolchain).

**Rows: 6. Reproduced (structural / real-git test): 5. Attested → CI: F-6 (+ the
cargo legs of F-1/F-2).** No silent drops. Code committed at `e8e69de`.

## Note

The fix reuses the committed tree object for the index write — the "by construction,
not a second filter that could drift" choice CC flagged. That is the right call: a
re-derived index (a second walk/filter) could diverge from the committed tree over
time; reusing the one tree makes index-vs-HEAD equality a structural invariant, not a
maintained coincidence. Worth carrying as a pattern.

## Arc-plan disposition

Flip **SL-6 → done (CDC-verified)**; slice 04 → CDC-verified PASS. F-1's real leg
(clean `git status` on the live store) closes on the operator's next `store commit`
with the rebuilt binary — and it retires the manual `git reset` step for good. The
recurring "phantom staged-deletion" scare of this session is now root-caused (stale
index) and fixed (index synced to HEAD post-commit) with a real-git regression test.
