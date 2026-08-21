# Closing Report — Arc: Store Lifecycle

> **Arc closed 2026-08-04** by CDC (this Cowork session). All six slices done;
> composition rows SL-4 and SL-5 verified at arc scale; bubble-up to
> `project-plan.md` below.
>
> Verification basis: per-slice closing reports and CDC verifications (where
> present), arc-plan ledger, and the operator's live use of the full lifecycle
> (the 14-commit first push to `origin/odm-store` via `store sync` on
> 2026-08-04).

## 1. Capability restated

From `arc-plan.md`:

> Complete the store's git lifecycle as **native, odm-aware** commands, so the
> operator never drops to raw git to manage the orphan-branch store. ...
> The end state: `migrate -> store status -> store commit -> store sync` is a
> fully odm-native flow.

**Verdict: delivered.** The lifecycle reads `init -> mutate -> status -> commit
-> set-remote -> sync`, all native `odm` commands. The operator's live workflow
on 2026-08-04 — `store sync` pushing 14 commits to `origin/odm-store` — is the
real-world proof: no raw git, no manual branch management, no index confusion.
Every command is idempotent, `--dry-run`/`--json`-capable, and faithful to the
ODD-0022 store discipline (orphan branch, never rewrite history, divergence
stops rather than merges).

## 2. Slice walk

Six slices in the arc, all delivered. The arc-plan's breakdown was s01–s06 (with
s04–s06 inserted during the arc's run as live needs surfaced — see the change
log in section 4).

| Slice | Scope | Outcome | Ledger rows | Verification |
|-------|-------|---------|-------------|--------------|
| **s01** — `store commit` | Persist the worktree's node changes on the orphan branch; auto-summary; `-m` override; idempotent no-op; `--dry-run`/`--json` | **Delivered.** Also caught + fixed a real ODD-0022 gap: `write_tree` wasn't honoring the store's `.gitignore`, so `.odm/` caches would have baked into the orphan branch. | F-1…F-8 done (0 deferred, 0 no-op) | CC-attested + **CDC-verified PASS** 2026-08-02 |
| **s02** — `store status` | Read-only view: pending node delta (identical to what `commit` would write) + ahead/behind vs. upstream; `--json` | **Delivered.** Reused `Ancestry`/`SyncAction` plumbing from `init.rs` without modification — confirmed reusable a second time. | F-1…F-8 done (0 deferred, 0 no-op) | CC-attested 2026-08-03; **CDC-verified PASS backfill 2026-08-21** |
| **s03** — `store sync` | Push/pull the orphan branch, ff-only pull, divergence stops; `--dry-run`/`--json` | **Delivered.** Found and fixed `merge_ff_only` — a dormant real bug in `init.rs` where the ff-only pull path had never been exercised against a real remote. | F-1…F-8 done (0 deferred, 0 no-op) | CC-attested 2026-08-03; **CDC-verified PASS backfill 2026-08-21** |
| **s04** — commit index consistency | After `store commit`, sync the git index to HEAD so raw `git status` is clean; regression test | **Delivered.** Root-caused the recurring "phantom staged-deletion" scare — the commit was always correct; only the stale index misled. | F-1…F-6 done (0 deferred, 0 no-op) | CC-attested + **CDC-verified PASS** 2026-08-02 |
| **s05** — CDC binary access | `cargo-zigbuild` cross-compilation producing a static `x86_64-unknown-linux-musl` binary; `make build-linux`; documented in `CLAUDE.md` | **Delivered.** CDC staged, smoked (`--help`, `check`, `orient`, `validate`, `node show`, `store --help`), confirmed static. | F-1…F-5 done (0 deferred, 0 no-op) | **CDC-verified** (F-1/F-2) + CC-attested (F-3/F-4/F-5) 2026-08-03 |
| **s06** — `store set-remote` + sync first-push | Config command to set the sync remote (explicit or auto-detect); `store init` auto-sets when unambiguous; `sync` handles first-push; `--json` | **Delivered.** Also caught a `rename.rs` bug via compiler-enforced exhaustive struct pattern — `store rename` would have silently dropped a configured remote. | F-1…F-8 done (0 deferred, 0 no-op) | CC-attested + **CDC-verified PASS** 2026-08-04 |

**Slice count: 6 delivered, 0 deferred, 0 dropped.** The arc-plan's original
breakdown was 3 slices (s01–s03); three more were inserted during the run
(s04 for a live defect, s05 for CDC tooling, s06 for a live blocker). All
insertions were tracked in the arc-plan's version history with the event that
surfaced them.

### Note A — s02/s03 CDC verification backfilled

Slices 02 and 03 were implemented and CC-attested on 2026-08-03, after the arc
resumed from its Migration Fidelity gate. At original arc close, no
`cdc-verification.md` had been written for either. That formal gap was closed on
2026-08-21:

- `slice02-store-status/cdc-verification.md` — PASS.
- `slice03-store-sync/cdc-verification.md` — PASS.
- Reproduced command: `cargo test --test store_status --test store_sync` (12/12
  `store_status`, 13/13 `store_sync`).

The original mitigating evidence still stands:

- The operator used both commands live and confirmed correct behavior.
- `store sync`'s 14-commit first push to `origin/odm-store` is a real-world
  end-to-end reproduction that exercises `status` (pre-flight) and `sync` (the
  push) against a genuine GitHub remote.
- Both commands reuse plumbing (`delta.rs`, `Ancestry`/`SyncAction`) that was
  independently verified in s01 and s04 (CDC-verified) and in `init.rs` (the
  existing sync arm that s03 lifts from).
- All fixture tests pass on CI (`release/1.0.x`).

**Assessment:** the former gap is now closed. The commands' core logic was
CDC-verified in neighboring slices, operator live use exercised the integration
paths, and the missing per-slice verification files now exist with reproduced
targeted tests. The original gap remains named here as history, not as an open
defect.

## 3. Composition check

The arc ledger's two composition rows (SL-4 and SL-5) were **planned** at arc
open, deferred to arc close by design — they verify properties that emerge only
when the slices compose.

### SL-4 — No raw git needed for the normal lifecycle

> **Criterion:** No raw git needed for the normal lifecycle (`init -> mutate ->
> status -> commit -> sync`); each command idempotent + `--dry-run`/`--json`;
> the freeze flow is end-to-end odm.

**Status: done — reproduced.**

The full lifecycle is native:

1. `odm store init` — bootstraps the store home, makes the first commit, auto-sets
   the remote when exactly one exists.
2. `migrate`/`node` ops — mutate the worktree.
3. `odm store status` — read-only view of pending changes + upstream position.
4. `odm store commit` — persists changes on the orphan branch, auto-summary or
   `-m`, idempotent no-op when clean.
5. `odm store set-remote` — configures the sync target (explicit or auto-detect).
6. `odm store sync` — push/pull, ff-only, divergence stops.

Each command is idempotent and `--dry-run`/`--json`-capable (except `status`,
which is inherently read-only and has `--json`). The freeze flow (`migrate --all
-> store status -> store commit`) is end-to-end odm.

**Evidence (reproduced):** The operator's 2026-08-04 session ran the complete
flow — `store commit` followed by `store sync` — pushing 14 commits to
`origin/odm-store`. No raw git was used. The `store commit` index-consistency
fix (s04) means even a casual `git status` agrees with reality — no phantom
staged-deletions, no manual `git reset` needed.

The one edge case worth naming: the operator still uses `git -C .worktrees/odm
log` or similar to inspect the store's commit history, because `odm` has no
`store log` verb. This is a **read-only inspection** habit, not a lifecycle gap
— it's the "normal operation" equivalent of looking at `git log` on any repo,
and it doesn't modify state. A `store log` command could be a future arc but is
not a Store Lifecycle gap.

### SL-5 — No model drift

> **Criterion:** No node-schema change; store discipline (orphan/history/
> divergence) unchanged; ODD-0022 amended only if a line is needed.

**Status: done — attested (cross-read).**

- **No node-schema change.** The six slices added CLI commands and git plumbing;
  no slice touched the node schema, the frontmatter format, or the store's data
  model. `odm-store`'s `delta.rs` (new in s01) reads node frontmatter to
  classify deltas but does not modify or extend the schema.
- **Store discipline unchanged.** The orphan-branch model holds: `store commit`
  commits to the orphan branch via `gix`; `store sync` pushes/pulls with
  ff-only discipline; divergence stops with a clear error, never merges or
  rebases. These are the ODD-0022 invariants, and every slice's fixtures test
  them.
- **ODD-0022 not amended.** No amendment was needed. The one ODD-0022-adjacent
  fix (s01's `.gitignore`-exclusion for `.odm/` caches) brought behavior into
  line with what ODD-0022's own scaffolded `.gitignore` already promised — it
  corrected an implementation gap, not a model change.

**Evidence strength: attested** (cross-read of the diff across all six slices,
not a mechanical reproduction). This is appropriate for a "no drift" row — what
you're checking is the *absence* of a change, which is verified by reading the
code rather than running a command.

## 4. Accumulated arc-plan change log

The arc-plan's version history records five changes during the arc's run:

| Version | Date | What changed | Surfaced by |
|---------|------|--------------|-------------|
| v1.0 | 2026-08-01 | Arc shaped: 3 slices (commit/status/sync) | arc-migration-fidelity freeze (no odm verb to persist the store) |
| v1.1 | 2026-08-02 | s01 CDC-verified; arc PAUSED per scoped-run | s01 close |
| — | 2026-08-02 | s04 inserted (index consistency remediation); SL-6 added | Operator hit phantom staged-deletions live (twice) |
| v1.2 | 2026-08-03 | s05 inserted (CDC binary access); SL-7 added; arc RESUMED | 2026-08-03 dev-workflow discussion (operator + CDC) |
| v1.3 | 2026-08-04 | s06 inserted (store set-remote + first-push); SL-8 added; SL-2/SL-3/SL-7 marked done | Operator's first `store sync` against GitHub — "nothing to sync" on an unpushed branch |

**Pattern:** the arc doubled in scope (3 slices -> 6) through three insertions,
each driven by a real, live gap the operator hit — not speculative additions.
Every insertion was tracked with the event that surfaced it. The original 3
slices (commit/status/sync) landed as planned; the three additions filled gaps
that were invisible until the commands were used in the field.

**Stale arc-plan entry (process note):** SL-8's status in the arc-plan ledger
table still reads "planned — inserted v1.3" despite s06 being closed with CDC
verification. This should have been updated when s06 closed. Named here, not
buried.

## 5. Bubble-up to the project

### Did this arc deliver its capability as `project-plan.md` defined it?

Yes. The project-plan's §2a row for Store Lifecycle reads:

> **Capability:** Complete the store's git lifecycle as native, odm-aware
> commands — `store commit`, `store status`, `store sync`, `store set-remote` —
> so the operator never drops to raw git to manage the orphan-branch store.

The arc delivered exactly this: four new commands completing the lifecycle, plus
a tooling slice (CDC binary access) and a remediation slice (index consistency).
The "no raw git for normal operation" exit criterion is met — the operator's
live workflow on 2026-08-04 proved it end-to-end.

### What did this arc reveal that the project plan did not anticipate?

1. **CDC binary access as a reusable asset.** The `cargo-zigbuild`
   cross-compilation and the `make build-linux` target are not SL-specific —
   they enable CDC participation in any future slice that needs to run `odm` in
   the cloud container. This is now part of the project's infrastructure, not
   just a Store Lifecycle artifact.

2. **The arc's scope doubled through live-surfaced gaps.** Three of six slices
   were insertions driven by real operator pain: a stale-index defect (s04), a
   CDC access gap (s05), and a missing remote-configuration command (s06). The
   original 3-slice plan was the right *decomposition* of the capability, but
   the *implementation surface* was larger than the plan anticipated. This is
   not a planning failure — it's the expected behavior of the bubble-up system
   catching live gaps.

3. **`store log` is not in scope but the operator uses `git log` on the store.**
   A read-only inspection gap, not a lifecycle gap. If a future arc addresses
   the LLM/CLI command surface, `store log` might be worth scoping.

### Silent-drop diff at arc scale

**Arc capability as specified** (from the arc-plan exit criteria):

> The store's full git lifecycle is native odm — no raw git for normal
> operation; `commit`/`status`/`sync` are idempotent, `--dry-run`/`--json`-
> capable, and honor ODD-0022; and the migration/freeze flow reads
> `migrate -> store status -> store commit` with no git footnote.

**Arc capability as delivered:** all of the above, plus `store set-remote`
(remote configuration), plus index-consistency after `store commit`, plus the
CDC binary access tooling.

**Missing:** nothing. The arc delivered its specified capability and more.
The two items not delivered — CDC verification of s02/s03 — are verification
gaps, not capability gaps; the commands work and are in use. Named in the
slice walk (Note A), not buried.

---

_Closed by: CDC (Cowork), 2026-08-04. Evidence: per-slice closing reports +
CDC verifications + the operator's live 14-commit `store sync` to
`origin/odm-store`._
