# CC Prompt — Slice 15 iteration 1: wire the collapse into `migrate --all`

**Narrow, thin-wiring iteration.** The arc-close freeze dry-run (`migrate --all --dry-run`, 2026-08-01,
both step-0 fixes applied) is clean on everything **except the collapse — which is entirely absent.**
`collapse_project_vision` (s15) has **no CLI caller**: it is invoked only from its own unit tests. So
`migrate --all` reconciles everything but never collapses the project — `#1000` stays a synthesis, `#1001`
stays a drifted, unreconciled base. **s15's headline change cannot fire.** This iteration wires it in.

> **Start condition:** on `release/1.0.x`, s15 (`1c779ec`) merged. **Fixture only — no `odm`-branch commit.**
> The live run is the arc-close freeze.

## Task

1. **Call `collapse_project_vision` from `migrate --all`'s `all()` orchestration**
   (`crates/odm-cli/src/migrate.rs`). Order: run it as a step in the compose (natural place — **before** the
   self-host + reconcile passes, so the just-collapsed 1:1 project node is then reconciled like any plan
   node in the same run; confirm that ordering holds, and flag if a different order is needed). Pass the
   plan root + `mode` (dry-run/commit) through exactly as the other steps do.
2. **Dry-run preview.** The collapse must render its own section under `--dry-run` (a `COLLAPSE (DRY RUN)`
   block or equivalent) showing the one re-cast project node + the retired base — so the adjudicator sees
   it, consistent with every other `--all` step. Reuse `CollapseReport`.
3. **Idempotent + safe.** `collapse_project_vision` already no-ops when there's no synthesis-bearing project
   (returns `collapsed: None`); confirm `--all` surfaces that as a 0-change step and a re-run writes
   nothing. `--dry-run` writes nothing.
4. **Fixture.** A `--all` fixture over a store containing a vision pair asserts the pass **collapses** it
   (project node re-cast to 1:1, base retired) **and** the collapsed project then reconciles on a subsequent
   `project-plan.md` edit — i.e. the two s15 behaviors compose in one `--all` run. Plus a re-run-no-op and a
   dry-run-writes-nothing assertion.

## Constraints

- **Thin wiring only.** `collapse_project_vision`'s logic is done + CDC-verified — do not change it; just
  invoke it. If wiring reveals a real defect in the collapse itself, flag CDC (that would be an s15 fix, not
  iteration scope).
- **No live store mutation. No `odm`-branch commit.** The freeze fires it.
- Keep the compose idempotent and dry-run-safe end to end.
- No `unsafe`; typed errors; clippy `-D warnings` clean; cover the new wiring.

## Deliverables

The `all()` wiring + the dry-run preview + the compose fixture on `release/1.0.x`; update
`slice15-reconcile-completeness/ledger.md` (a wiring row / note under F-1) + `closing-report.md` (the
iteration: the missing-caller finding, the fix, the re-run evidence). Confirm the arc-close freeze
(`migrate --all`) now collapses + reconciles in one pass. Branch: `release/1.0.x` only.

## Working agreement

Five-iteration cap (this is iteration 1). Your `done` is proposed-done — CDC re-verifies by reading the
`all()` call + the compose fixture, then re-runs `migrate --all --dry-run` and adjudicates that the collapse
now appears. On close, no arc-plan slice-status change (s15 stays CDC-verified PASS; this is a wiring
iteration noted in its closing-report).
