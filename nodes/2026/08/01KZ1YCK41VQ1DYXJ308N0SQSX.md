---
id: 01KZ1YCK41VQ1DYXJ308N0SQSX
number: 61576100
type: arc
schema: arc/v1.1
name: 'Arc — Store Lifecycle: `commit`, `status`, `sync`'
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-lifecycle/arc-plan.md
  class: arc-plan
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-03
edges:
  part_of: 01KYSX4RGCZT9H1N83TJFX8XFB
status:
  in-progress:
    reached: 2026-08-02
    evidence: asserted
    evidence_dates:
      asserted: 2026-08-02
  planned:
    reached: 2026-08-02
    evidence: asserted
    evidence_dates:
      asserted: 2026-08-02
---
# Arc — Store Lifecycle: `commit`, `status`, `sync`

> **Named arc, canonical number deferred** (operator call, 2026-08-01) — same §2a treatment as
> Release Hardening / arc-store-home / Migration Fidelity / LLM-command-surface. **v1.0.x**, near-term
> fast-follow. Directory: `arc-store-lifecycle/` (provisional; settles with the numbering).
>
> **Why now:** the arc-migration-fidelity freeze surfaced it — after `migrate --all` mutates the store
> worktree, there is **no odm verb to persist it**; the operator must drop to raw `git -C .worktrees/odm`.
> That breaks the store-home promise (ODD-0022: *odm owns the orphan branch; you never touch it by hand*).
>
> `depends_on:` **arc-store-home** (the store home + `store init` this completes) · **ODD-0022** (the store
> model — orphan branch, never rewrite history, divergence stops). **Refs:** `arc-store-home/arc-plan.md`;
> `crates/odm-store/src/{git,worktree,init}.rs`; `crates/odm-cli/src/store_cmd.rs`.
>
> **Status:** shaped 2026-08-01. **Scoped run (operator 2026-08-02): do s01 (`store commit`) only, then ⏸ PAUSE this arc** — s02 (`store status`) / s03 (`store sync`) deferred. The arc resumes after Migration Fidelity closes. This detour exists so the operator stops hand-committing the store with raw git. **Update 2026-08-02: s04 (SL-1 index-consistency remediation) inserted and running now** — a live defect where `store commit` leaves the git index stale (raw `git status` misreports every commit); s02/s03 stay paused.

## Capability

Complete the store's git lifecycle as **native, odm-aware** commands, so the operator never drops to raw
git to manage the orphan-branch store. Today the lifecycle is `init → mutate → ??? → ???`: `store init`
bootstraps the home (and makes the *first* commit), `migrate`/`node` ops mutate the worktree — but
**persisting** (commit), **inspecting** (status), and **remoting** (sync) have no verbs. This arc fills
them, each: **odm-aware** (reports node deltas, not raw `git status` porcelain); **idempotent** (no-op when
there's nothing to do); **`--dry-run`- and `--json`-capable** (the house LLM-ergonomics contract); and
faithful to the ODD-0022 store discipline (**orphan branch, never rewrite history other clones hold,
divergence stops rather than merges**). The end state: `migrate → store status → store commit → store sync`
is a fully odm-native flow — and the arc-migration-fidelity freeze's "commit the store" step stops being a
raw-git footnote.

## Slice breakdown (plan-late, plan-deep; one-line altitude)

- **s01 — `store commit`.** Stage the store worktree's pending node changes and commit them on the orphan
  branch. **Auto-summary message** from the node delta (N created / modified / removed, by type) with `-m`
  override; `--dry-run` (show what would be committed, write nothing); **no-op + clear message when clean**;
  `--json`. *(Design decision D-1: the auto-summary is git-delta-derived node counts in v1; a richer
  migrate-aware summary — "reconciled / minted / collapsed" — is a possible enhancement via a
  migrate-written pending-summary handoff; flag, don't over-build.)* **The near-term need.**
- **s02 — `store status`.** Read-only view of the store's git state: the pending node delta `commit` would
  write (dirty?) **and** ahead/behind vs. the configured upstream. odm-aware counts, not porcelain;
  `--json`. Reuses s01's delta computation. *(The read half of the same coin as commit.)*
- **s03 — `store sync`.** Push/pull the orphan branch to/from its remote — the **ff-only, divergence-stops**
  discipline `store init`'s attach/ff-sync arm already embodies (`store_cmd.rs`), lifted into a standalone
  verb; `--dry-run`/`--json`; never rewrites history, stops (with a clear affordance) on divergence.
- **s04 — `store commit` leaves the git index consistent (remediation of SL-1; runs now).** `commit_all`
  writes the commit tree directly via `gix` and never updates the on-disk index, so after every
  `store commit` a raw `git status` misreports the just-committed files as staged-deletions + untracked
  (the root cause of this session's recurring "phantom staged-deletion" scare — the commit was always
  correct; the index was stale). Sync the index to the new HEAD after committing; add the clean-`git status`
  test SL-1's suite lacked. Additive — the race-free `odm store status` tree-comparison (`delta.rs`) and the
  `.odm/`-exclude (ODD-0022) are untouched. **CDC-verified PASS 2026-08-02** (code `e8e69de`;
  `slice04-commit-index-consistency/`): the fix reuses the committed tree for the index write
  (index == HEAD by construction), with a real `git status --porcelain` regression test; `delta.rs`
  provably untouched.

## Arc Ledger (composition rows — opens here, closes in `closing-report.md`)

> LEDGER-DISCIPLINE v2.0 §B. Rows reproduced at arc scale. Status ladder
> `asserted < attested < reproduced < reconciled`; a `done` row reaches ≥ `reproduced`. All open **planned**.

| ID | Criterion | Verify | Significance | Status |
|----|-----------|--------|--------------|--------|
| SL-1 | `store commit` persists the worktree's node changes on the orphan branch; auto-summary + `-m`; idempotent no-op when clean; `--dry-run`/`--json` | run it after a `migrate`; the orphan branch gains one commit with the right message; a second run is a no-op | serious (the gap this arc opens for) | **done — CDC-verified PASS 2026-08-02** (`slice01-store-commit/cdc-verification.md`); + a real ODD-0022 fix (`.odm/` caches now excluded from the orphan branch, `write_tree` gix-exclude-aware) |
| SL-2 | `store status` reports the pending node delta + ahead/behind, odm-aware, `--json` | dirty a node, run status → the delta shows; clean → "nothing to commit"; upstream ahead/behind correct | serious | planned |
| SL-3 | `store sync` push/pull, ff-only, divergence stops (never rewrites history) | round-trip against a remote; a diverged branch stops with an affordance, not a merge/rebase | serious (ODD-0022 discipline) | planned |
| SL-4 | **No raw git needed** for the normal lifecycle (`init → mutate → status → commit → sync`); each command idempotent + `--dry-run`/`--json`; the freeze flow is end-to-end odm | reproduce the arc-migration-fidelity freeze-commit step with `store commit` instead of raw git | serious (the composition) | planned |
| SL-5 | No model drift: no node-schema change; store discipline (orphan/history/divergence) unchanged; ODD-0022 amended only if a line is needed | cross-read: CLI + odm-store git plumbing only; ODD cited if touched | correctness | planned |
| SL-6 | After `store commit`, the git **index** matches the new HEAD — a raw `git status` is **clean** (no stale-index staged-deletions), without changing the commit content, the `.odm/` exclusion, or the odm-aware delta | fixture: `store commit` → `git status --porcelain` empty; SL-1 tests still green | serious (SL-1 remediation; underpins SL-4) | **done — CDC-verified PASS 2026-08-02** (`slice04-commit-index-consistency/cdc-verification.md`, code `e8e69de`); real leg = operator's next `store commit` on the rebuilt binary |

## Exit criteria (arc acceptance)

The store's full git lifecycle is native odm — **no raw git for normal operation**; `commit`/`status`/`sync`
are idempotent, `--dry-run`/`--json`-capable, and honor ODD-0022 (orphan branch, never rewrite history,
divergence stops); and the migration/freeze flow reads `migrate → store status → store commit` with no
git footnote. Composition reproduced at arc close.

## Method

Per-slice CDC/CC loop (draw the open set → CC implements → CDC verifies → bubble up). s01 first (the need);
s02/s03 follow. Mostly CLI wiring over `odm-store`'s existing git plumbing (`git.rs`/`worktree.rs` already
do worktree/branch/commit ops for `init`) — the capability is largely *exposing* what init already uses.

## Version History

### 2026-08-02 — s04 inserted (SL-1 index-consistency remediation)

Added **s04** + ledger row **SL-6** after a live defect surfaced: `store commit`
commits by building a tree via `gix` and never updates the on-disk index, so raw
`git status` misreports every commit as staged-deletions + untracked. This is the
root cause of the "phantom staged-deletion" confusion that recurred across the
2026-08-02 session — twice mistaken for a corrupted store; it was always just the
stale index (the commit content was correct throughout). SL-1's tests verified the
commit *content*, not the resulting `git status`, so the gap slipped through its
PASS. s04 syncs the index to HEAD after committing and adds the clean-`git status`
test, without touching the commit content, the `.odm/` exclusion, or the race-free
`odm store status` tree-comparison. It runs **now** (operator call) as an SL-1
remediation; s02/s03 stay paused. Surfaced by: operator (hit it live twice) + CDC
root-cause read of `git.rs::commit_all`. **CDC-verified PASS 2026-08-02** (`e8e69de`):
`commit_all` now syncs the index from the committed tree (`sync_index_to_tree`), a
real `git status --porcelain` test asserts clean, `delta.rs` provably untouched. SL-6
done; the manual `git reset` step retires on the operator's next `store commit`.

### 2026-08-02 — s01 (`store commit`) closed; bubble-up

`slice01-store-commit` closed (CC attested; CDC reproduction pending) — see its `ledger.md` and
`closing-report.md`. SL-1's criteria (F-1…F-8) all `done`. The slice's real-orphan-branch fixture discipline
surfaced and fixed a genuine ODD-0022 gap beyond the command surface itself: `odm-store`'s `write_tree` (the
filesystem-walking tree builder `is_clean`/`commit_all` share) never consulted the store's own `.gitignore`,
so any command that populated `.odm/index`/`.odm/drift` before a commit would have baked those derived
caches permanently into the orphan branch. Fixed in `git.rs` (now `gix`-exclude-aware); regression-fixtured
in `odm-store`. No ODD-0022 amendment judged necessary — the fix brings behavior into line with what the
store's own scaffolded `.gitignore` already promised, rather than changing the model (flagged in the ledger
for CDC to confirm). Carried forward as a note for s02/s03: any future store-worktree verb going through the
same filesystem-walking git plumbing inherits the fix automatically; verify rather than assume when s02
(`store status`) is implemented. Per the scoped-run note above, the arc stays **paused** — s02/s03 not
started; this entry only closes s01.

### v1.1 — 2026-08-02 — s01 (`store commit`) CDC-verified PASS; arc ⏸ PAUSED per the scoped run

**SL-1 done — CI green on `release/1.0.x` (`e4e0a95`, 2026-08-02, operator-attested).** `odm store commit` lands: persists the store worktree's pending node changes on the orphan
branch, odm-aware auto-summary (`-m` overrides), idempotent no-op, `--dry-run`/`--json`; git ops in
`odm-store` (`tree_delta` + a reusable `delta` module for s02). **CC caught + fixed a real ODD-0022 defect**
— `commit_all` walked the filesystem without honoring the store's `.gitignore`, so the `.odm/index`/`.odm/drift`
derived caches would have baked permanently into the orphan branch; `write_tree` is now gix-exclude-aware,
with a negation-honoring regression test. Reproduced clean by CDC; `odm@e06fffe` untouched. **Per the
scoped-run note the arc now ⏸ PAUSES** (s02 `store status` / s03 `store sync` not started); **Migration
Fidelity/s16 resumes next.**

### v1.0 — 2026-08-01 — arc shaped

Shaped from the arc-migration-fidelity freeze, which had no odm verb to commit the store worktree. Three
slices (commit / status / sync) complete the ODD-0022 store lifecycle. Named, number-deferred, v1.0.x.
Depends on arc-store-home. s01 (`store commit`) is the near-term need.
