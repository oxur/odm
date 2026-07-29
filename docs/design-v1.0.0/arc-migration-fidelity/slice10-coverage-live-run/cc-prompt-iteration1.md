# CC Prompt — Slice 10 (Migration Fidelity), **iteration 1**: source-path portability regression + enforced invariant

**Why this iteration exists.** CDC verification of s10 reproduced the live store and found a **silent
portability regression**: **4 nodes carry absolute `source.paths`** again — the exact CDC v2.8 Finding 1
bug s08 fixed and drove to 0. s10's ODD-import path minted the four `04-accepted` ODDs **without**
routing their `source.paths` through s08's `relativize` seam. It passed `check` green only because the
coverage index relativizes stored paths *before* matching, so an absolute-stored path still matches — the
s08 transition-tolerance **masks** the regression, and nothing enforces "a stored `source.paths` is
relative." This is iteration 1 of s10's five-iteration budget: fix the data + the seam, and add the
invariant so it can't silently recur.

> **The 4 nodes** (on the `odm` branch, `355404a`), each stored as
> `/Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design/04-accepted/00XX-*.md`:
> **#22** (store-home), **#23** (command-surface), **#24** (id-scheme/ULID), **#25** (migration-fidelity).
> All are `type: design` (restamped from `odd`). Their **bodies, ids, schema, and git-derived dates are
> correct** — only the `source.paths` *form* is wrong. Confirmed comprehensive: a full scan of all 369
> live nodes finds these 4 and no others.

> **Start condition:** on `release/1.0.x` (green, s10 merged). The live store is at `odm` tip **`355404a`**
> — the revert target + base for the live rewrite. This iteration **mutates the live store** (rewrites 4
> nodes) as one revertible commit after a clean dry-run. **Snapshot SHA + dry-run-first are HARD gates.**

## Read first

1. **s08** `slice08-source-path-portability/` — `slice-doc.md` (the portability invariant + why absolute
   breaks it) and `cdc-verification.md`; the `fidelity.rs` `git_toplevel`/`anchor_for`/`relativize`/
   `resolve_from_anchor` seam is the one every storable path must pass through.
2. **s10** `slice10-coverage-live-run/cdc-verification.md` (this finding, with the masking mechanism).
3. **The seam to fix:** the design/ODD import path that mints new `design` nodes from `docs/design/**`
   ODD files (contrast `artifact.rs::mint_artifacts` / `notes.rs`, which relativize correctly, and
   `mapping.rs::backfill_source`, which relativizes existing design/research). The date-stamping on this
   path already works (git-derived) — **only the `relativize` step is missing**. Find where this path
   writes `source.paths` and route it through `fidelity::relativize` on the same anchor everything else
   uses (the plan-tree git toplevel).

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `03-error-handling.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **Fix the seam** (so future ODD/design imports are relative). Route the design/ODD import's
   `source.paths` write through `fidelity::relativize` (anchor = plan-tree git toplevel), identical to
   the artifact/note/backfill paths. No new anchor logic — reuse s08's.
2. **Add the enforced invariant** (the durable fix). Add an **unconditional** `check` rule: any
   `source.paths` entry that is **not** repo-content-root-relative (absolute, or `.worktrees/`-prefixed)
   is an **Error** (e.g. code `absolute-source-path`). Unconditional — unlike the coverage rule there is
   no legitimate absolute case, so it needs no config gate. This makes portability an *enforced
   invariant*, not a one-time data fix — had it existed, this regression would have gone red at s10, not
   slipped through green.
3. **Fixtures** (`TempDir`, on `release/1.0.x`, green before touching the live store):
   - the design/ODD import stores a **relative** `source.paths` (regression test for the seam);
   - the new check rule: a seeded **absolute** `source.paths` → **Error** naming the node; a relative one
     → green;
   - **reproduce-the-bug:** with the guard in place but *before* the live rewrite, the guard flags
     **exactly these 4** nodes (proves it catches the real regression), and **0** after.
4. **Live rewrite** (behind the s07/s08 protocol). Snapshot `355404a` + before-manifest → **`--dry-run`**
   → adjudicate (**exactly 4 `source.paths` rewrites, 0 creates, 0 body/id/schema/date changes**; store
   byte-identical after the dry-run) → **fire as one revertible commit atop `355404a`** → verify.
5. **Verify on the committed store:** **0 absolute `source.paths`** remain (was 4); the 4 nodes'
   bodies/ids/schema/dates **byte-identical** (only the path string moved); `odm check` **exit 0 with the
   new guard enforcing**; coverage still **371/371** (the 4 now match via the *primary* source.paths, no
   longer via the fallback); `orient`/`rollup` byte-stable; re-run idempotent (0/0). **Land the rewrite +
   the guard together so `check` has no red window** (the live store is clean of absolutes at the moment
   the guard goes enforcing).

## Constraints (flag, don't silently change)

- **Only the 4 nodes' `source.paths` *form* changes** — not bodies, ids, schema, or the (correct)
  git-derived dates. Mirror s08's "path string only" rewrite.
- **Snapshot SHA + dry-run-first are non-negotiable.** Any deviation (an unexpected create/modify, a
  body/date change, `check` not green after) → **revert to `355404a` + finding**, not forced.
- **The guard is unconditional and an Error** (not a warning, not config-gated).
- Don't pull **s11** (synthesis) or **s12** (reconcile — the 2 *drifted* design nodes ODD-0013/0020,
  which have *no* source and are a different case, stay s12; this iteration does **not** touch them).
- No `unsafe`; typed errors; changed code clippy-clean; coverage ≥ 90% (target 95%) on changed modules.

## Optional companion (s09 follow-up, fold in only if trivial — else skip and flag)

ODD-0025 **§4** still literally reads "add an `artifact/v1.0` schema marker," but the shipped code stamps
the shared global `SchemaVersion::CURRENT` = `artifact/v1.1` (correct — the minor is one global
generation counter, not a per-type axis). A **one-line §4 amendment** reconciling the spec to delivery
(with that rationale) closes the s09 CDC LOW finding. This is a **doc-only** change to an Accepted ODD —
if you touch it, keep it to that one line + a dated ODD version-history entry; if it's not clean, **skip
and leave it flagged** for the operator.

## Deliverables

Code on `release/1.0.x` (the seam fix + the guard + fixtures); the live rewrite on `odm` (one revertible
commit atop `355404a`); **s10 `ledger.md`** — re-disposition **F-10** (the "no model drift" row this
regression falsified) and add rows for the seam fix + the guard, marked iteration-1; update
`closing-report.md` with an **Iteration 1** section (what regressed, the fix, the guard, before/after
counts); **arc-plan bubble-up** noting the regression found + fixed + now enforced. Cite the undo SHA
(`355404a`), the 4 rewritten paths (before/after), and the guard's reproduce-the-bug evidence (flags 4 →
0).

## Working agreement

Iteration 1 of s10's five-iteration budget. Amend don't work around; flag every deviation. Your `done` is
proposed-done — CDC reproduces the store-state rows by direct read (0 absolute remain, the 4 unchanged
but for the path, the guard flags 4→0) + CI. On close, hand back for CDC re-verification; s10 flips to a
clean CDC-verified PASS only once 0 absolute `source.paths` remain **and** the invariant is enforced.
