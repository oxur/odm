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
> **Status:** shaped 2026-08-01. **Scoped run (operator 2026-08-02): do s01 (`store commit`) only, then ⏸ PAUSE this arc** — s02 (`store status`) / s03 (`store sync`) deferred. The arc resumes after Migration Fidelity closes. This detour exists so the operator stops hand-committing the store with raw git. **Update 2026-08-02: s04 (SL-1 index-consistency remediation) inserted and running now** — a live defect where `store commit` leaves the git index stale (raw `git status` misreports every commit); s02/s03 stay paused. **Update 2026-08-03: arc RESUMED — s02/s03 un-paused** (operator call). Migration Fidelity is closed (P-16, the gate); the odm store branch has never been pushed to GH because `store sync` doesn't exist yet, blocking CDC's ability to clone the store. **s03 (`store sync`) is the immediate priority** — unblocks pushing the store to origin; s02 (`store status`) follows. **Update 2026-08-03: s02 (`store status`) closed** — SL-2 done. **s03 (`store sync`) is now the sole remaining slice.**

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
- **s05 — CDC binary access (cargo-zigbuild cross-compilation).** Add a `cargo-zigbuild`
  cross-compilation step to the operator's local build that produces a **static
  `x86_64-unknown-linux-musl` binary** — a fully self-contained Linux executable CDC can
  stage directly from the worktree. No Rust toolchain needed in the cloud container; no
  duplicate build. CDC verifies: stage, `chmod +x`, smoke-test (`--help`, `check`, `orient`).
  The binary location is documented in `CLAUDE.md`. **Runs ahead of s02/s03** — enables CDC
  to participate in verifying those slices. *(Inserted v1.2, 2026-08-03.)*
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
| SL-2 | `store status` reports the pending node delta + ahead/behind, odm-aware, `--json` | dirty a node, run status → the delta shows; clean → "nothing to commit"; upstream ahead/behind correct | serious | **done — CC-attested PASS 2026-08-03** (`slice02-store-status/closing-report.md`; 12 fixtures incl. a real bare-repo remote for ahead/behind, no mock) |
| SL-3 | `store sync` push/pull, ff-only, divergence stops (never rewrites history) | round-trip against a remote; a diverged branch stops with an affordance, not a merge/rebase | serious (ODD-0022 discipline) | planned |
| SL-4 | **No raw git needed** for the normal lifecycle (`init → mutate → status → commit → sync`); each command idempotent + `--dry-run`/`--json`; the freeze flow is end-to-end odm | reproduce the arc-migration-fidelity freeze-commit step with `store commit` instead of raw git | serious (the composition) | planned |
| SL-5 | No model drift: no node-schema change; store discipline (orphan/history/divergence) unchanged; ODD-0022 amended only if a line is needed | cross-read: CLI + odm-store git plumbing only; ODD cited if touched | correctness | planned |
| SL-7 | CDC can run odm **without building from source**: a static `x86_64-unknown-linux-musl` binary, cross-compiled by the operator via cargo-zigbuild, is staged from the worktree and runs in the cloud container (`--help`, `check`, `orient` exit 0); the workflow is documented | CDC stages + smoke-tests the binary in a Cowork session; a fresh session finds the binary location from `CLAUDE.md` | serious (enables CDC verification of s02/s03) | **done — CDC-reproduced (F-1/F-2, `cdc-verification.md`) + CC (F-3/F-4/F-5, `closing-report.md`) PASS 2026-08-03** |
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

### 2026-08-03 — s02 (`store status`) closed; bubble-up

`slice02-store-status` closed — CC-attested (`closing-report.md`): `odm store status` reports the
pending node delta (**identical** computation to `commit`, proven by a fixture that diffs
`status`'s and `commit --dry-run`'s `delta` output on the same dirty worktree, not just asserted
individually) and ahead/behind vs. the configured upstream, reusing `init.rs`'s
`Ancestry`/`SyncAction`/`sync_action()` plumbing with **no fetch** (D-1, flagged rather than
built). The ahead/behind fixtures use a **real bare-repo remote** (`git init --bare` + push +
fetch, no network, no mock). Read-only invariant (F-6) fixtured with a before/after snapshot of
`HEAD`, `git status --porcelain`, and the node-path listing, plus a second `status` run diffed
byte-for-byte against the first to rule out a hidden fetch. Zero `odm-store` changes — every
function needed was already public. SL-2 done. **s03 (`store sync`) is now the arc's only
remaining slice.**

### 2026-08-03 — s05 (CDC binary access) closed; bubble-up

`slice05-cdc-binary-access` closed — F-1/F-2 CDC-reproduced in the actual Cowork cloud container
(`cdc-verification.md`: static ELF x86_64 binary, full command surface — `--help`/`--version`/`check`/
`orient`/`validate`/`node show`/`store --help` — all exit 0, `orient --json`'s `drift.tracked:true`
proving live store connection), F-3/F-4/F-5 closed by CC (`closing-report.md`): added `make build-linux`
(reproducible cross-compile target, doesn't disturb the native `bin/odm`), documented the binary's
location + staging + an explicit x86_64-only architecture caveat in `CLAUDE.md`, and confirmed `make
test`/`make lint` stay green with zero Rust source changes. SL-7 done. **This unblocks CDC's
participation in verifying s02/s03**, the arc's next (and now immediate-priority) slices.

### v1.2 — 2026-08-03 — s05 (CDC binary access) inserted; arc RESUMED

**s05 inserted:** cargo-zigbuild cross-compilation producing a static
`x86_64-unknown-linux-musl` binary for CDC's cloud container. Operator-driven
(the build runs on the operator's Mac); CDC verifies (stage + smoke-test in
the container). **Runs ahead of s02/s03** — enables CDC to participate in
verifying the remaining store lifecycle commands. Ledger row SL-7 added.
Slice docs: `slice05-cdc-binary-access/`. Surfaced by: the 2026-08-03
dev-workflow/tooling discussion (operator + CDC).

### 2026-08-03 — arc RESUMED; s02/s03 un-paused (operator call)

Migration Fidelity is closed (P-16); the gate holding the arc pause is clear.
**s03 (`store sync`) is the immediate priority** — the odm store branch has never
been pushed to GH because `store sync` doesn't exist yet; this blocks CDC from
cloning the store in cloud sessions. s02 (`store status`) follows. s01 + s04
remain done. Surfaced by: the 2026-08-03 dev-workflow/tooling discussion
(operator + CDC).

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
