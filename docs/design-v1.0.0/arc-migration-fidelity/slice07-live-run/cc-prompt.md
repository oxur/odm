# CC Prompt — Slice 07 (Migration Fidelity): Live repair run

Fire the s06-built, CDC-verified `odm migrate` flow on the **real** `.worktrees/odm` store, and
**prove** the outcome: zero stubs, every arc/slice node source-bearing (project + retired excluded),
all plan arcs/slices represented, every body-hash passing, `check` green — committed as **one
revertible commit**. This is the **arc's first live mutation**; everything before it was fixture-only.
The capability is verified — this slice is about running it **safely** and **evidencing** it.

> **Start condition:** on `release/1.0.x` (green). **This slice DOES mutate the live store** — but only
> the `.worktrees/odm` **orphan store branch**, and only as one revertible commit, after a clean
> dry-run. **Snapshot/revert + dry-run-first are HARD gates** (below). If the dry-run shows anything
> unexpected, **stop before firing** and flag CDC. **Why this slice:** s06 made the flow invocable,
> one-policy, and `--dry-run`-safe (`slice06-live-run-capability/cdc-verification.md`, PASS); s07 runs
> it for real so odm's own corpus stops being 44 stubs / 0 provenance / 6-of-12-arcs — the P-12
> self-host DoD path.

## Read first
1. `slice07-live-run/ledger.md` (11 rows) — the spec of "done."
2. `slice-doc.md` (esp. **Baseline**, **Rollback & findings discipline**); `slice06-live-run-capability/cdc-verification.md`
   **§ Finding** (the doc-comment you correct first) + the s06 closing report.
3. **ODD-0025** §2.1 (hard gate), §2.2 (`source` on every migrated node), §2.3 (project/synthesis
   exclusion), §2.8 (update-in-place).
4. **`../arc-llm-command-surface/odm-command-inventory.md`** — `migrate` (with `--dry-run`,
   `--coverage`), `check`, `orient`, `rollup`. Confirm the exact invocation + store-resolution against
   the binary (`odm migrate -h`) before firing.

## Load skills (via `/<name>`)
- `/rust-guidelines` — for the F-1 doc-comment edit (and any incidental code touch).
- `/collaboration-framework` → LEDGER-DISCIPLINE (evidence strengths; this is a live class-(b) slice).

## Task — in order (the order is the safety protocol)

1. **(F-1) Correct the s06 doc comment, then pre-flight.** In `odm-cli/src/migrate.rs`, fix
   `self_host_inner`'s doc comment: it calls the repair→import order load-bearing for *correctness* —
   s06 proved both orderings converge (correctness rests on `reconcile_source`, exercised both ways;
   the order is for composability + report attribution). State that truthfully; no behavior change.
   Then confirm `1.0.x` green, the **store worktree clean** (`git -C .worktrees/odm status --porcelain`
   empty), and **capture the before-manifest + the store HEAD SHA** (node count, id set, per-node
   body-hash, `source` count, schema spread, `context.json`). That SHA is the known-good state
   everything reverts to. **Baseline to expect** (re-measure — the store may have moved): 60 nodes / 0
   source / all `v1.0` / **44 stubs** (6 arc + 38 slice) / 6 arcs / `context.json → 01KWXM…`.

2. **(F-2) Dry-run first — HARD gate.** Build `odm` from `1.0.x`; run `odm migrate <plan-root>
   --dry-run` against the live store. **Inspect the preview** — reconciled / created / skipped, **zero
   drift errors** — and **adjudicate it against the before-manifest before firing** (expect ≈47 plan
   nodes reconciled: 44 stubs repaired + faithful backfilled, project + retired excluded; the 6 missing
   arcs + their slices created). Confirm the dry-run left the store **byte-identical** (`git status`
   still clean). **A surprise here stops the slice** — flag CDC, don't fire.

3. **(F-3) Fire — one revertible commit.** Run `odm migrate <plan-root>` for real. Commit the
   `.worktrees/odm` worktree as a **single** commit atop the captured SHA; message names the slice +
   the before/after deltas. Record the undo: `git -C .worktrees/odm reset --hard <SHA>`.

4. **(F-4…F-10) Verify the live outcome** on the committed store:
   - **0 stub bodies** among plan nodes; the 44 stubs now carry verbatim source bodies (F-4).
   - **Every arc/slice node carries `source.paths`**; the **project** node carries **no** 1:1 `source`;
     the **retired/tombstone** node excluded (F-5).
   - **Full representation** — `odm migrate --coverage` shows **0 uncovered arcs/slices**; all **12**
     plan arcs + their slices present (F-6).
   - **Every body-hash passes** — a second `reconcile`/`migrate` → **0** `BodyHashMismatch`; migrated
     nodes stamped `schema: */v1.1` (F-7).
   - **`odm check` exit 0**; `orient` + `rollup` regenerate **byte-stable** on re-run (F-8).
   - **`context.json` re-pointed** correctly — state the intended post-run value + why (F-9).
   - **Re-run idempotent** — a second `odm migrate` → 0 reconciled / 0 created; ids stable (F-10).

5. **(F-11) Rollback & findings discipline.** If **any** gate fails — a real `BodyHashMismatch` on a
   "faithful" node, an unexpected create, `check` red, a non-idempotent re-run — **revert to the
   captured SHA, file the failure as a finding, and stop.** A live gate failure is a *discovery about
   the corpus* (a body edited away from its source, or a moved source path): surface and adjudicate it
   (fix the source mapping, or accept + document the drift) — **never suppress the gate to make the run
   pass.** Hidden failure is the only unacceptable outcome.

## Constraints (flag, don't silently change)
- **Only the `.worktrees/odm` orphan store branch is mutated**, and only via the single committed run.
  Do **not** touch the `1.0.x` plan tree beyond the F-1 doc comment. Do **not** hand-edit node files —
  every node change comes from `odm migrate`, so the run is reproducible.
- **Dry-run-first and the snapshot SHA are non-negotiable.** No live `odm migrate` (real) until the
  dry-run is clean and adjudicated.
- **Scope is plan nodes** (project/arc/slice). The 14 design/research nodes' `source` and the ~211
  loose-doc coverage are **s08** — `discover()` can't reach them and `validate` doesn't require
  `source`, so leaving them source-less is correct and `check`-safe here. Disclose the mixed
  `v1.1`/`v1.0` interim; don't pull s08 forward to "clean it up."
- Don't pull **s09** (the project node's synthesis re-cast — this slice only *excludes* it) or **s10**
  (arc-close reconcile demo) forward.
- If a real drift finding forces a source-mapping fix in code, that's a legitimate in-scope repair —
  flag it, keep it minimal, and re-run the whole protocol (snapshot → dry-run → fire) from a clean SHA.

## Deliverables
The committed live store (`odm` branch) + the F-1 doc fix (`release/1.0.x`); `ledger.md` evidence per
row (`attested`/`reproduced` — this is live, so cite the actual store state, counts, and command
exits, not just "green"); `closing-report.md` — per-row walk, the **before/after manifest deltas**
(node counts, source count 0→N, stubs 44→0, arcs 6→12, schema, `context.json`), any live findings
surfaced, **plus the v2.0 Bubble-up to the arc** (did s07 make the corpus faithful + fully represented;
what did the live run reveal the fixtures didn't; the silent-drop diff vs `slice-doc.md` In/Out —
especially the disclosed s08 deferral). Branch: `release/1.0.x` (impl/doc) + `odm` (store).

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap. **Your `done` is proposed-done** —
because this is live, CDC reproduces the store-state invariants by direct read of your committed store
(recomputing the body-hash gate independently) + re-running `check`/`orient`. On close, bubble up to
`../arc-plan.md` (MF-2/MF-3/MF-5 → done; residual coverage → s08).
