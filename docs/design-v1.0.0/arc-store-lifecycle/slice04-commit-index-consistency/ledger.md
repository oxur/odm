# Slice 04 ledger — `store commit` leaves the git index consistent

Per `LEDGER-DISCIPLINE.md` §A. Code slice (a remediation of SL-1). CC fills Evidence
at the commit each is met (`attested`); CDC reproduces. F-1 is the acceptance anchor.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | After `odm store commit`, a raw **`git status --porcelain`** on the store worktree is **empty** (index synced to the new HEAD) | fixture: `migrate`/dirty → `store commit` → assert porcelain empty | serious | 2026-08-02 finding | open | | the gap SL-1's tests missed |
| F-2 | The commit **content** is unchanged — the orphan branch gains the right commit with the auto-summary; all SL-1 tests pass | run SL-1's suite; orphan branch log correct | serious | SL-1 | open | | no behavior change to what's committed |
| F-3 | `.odm/` derived caches remain **excluded** from both the commit **and** the index (no ODD-0022 / `write_tree` gix-exclude regression) | fixture: a `.odm/index` present → absent from the commit tree and not staged after commit | correctness | ODD-0022 | open | | protect the s01 fix |
| F-4 | **Idempotent** — a second `store commit` on a clean store is a no-op and leaves a clean index | run twice; second is no-op, `git status` still clean | correctness | SL-1 | open | | |
| F-5 | The odm-aware delta/status (`delta.rs` tree-comparison) is **unchanged** — the index sync is additive, not a new status dependency | cross-read: `delta.rs` untouched; `store commit`'s summary still derives from the tree delta | correctness | SL-5 | open | | preserves the race-free design |
| F-6 | No regression: `make check` green | `make check` exit 0 | correctness | standing | open | | |

## What Worked

_(At slice close.)_

## Closure

Closed at commit <SHA> on <date>. Verified by: <name/session>.
Rows: 6. Done: _. Deferred: _. No-op: _.
