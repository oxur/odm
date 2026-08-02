---
id: 01KYZTRFG3CVBWRAQ6Q8E100NR
number: 539638400
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 15 (Migration Fidelity): Reconcile completeness + collapse the project-vision pair'
created: 2026-08-01
updated: 2026-08-01
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice15-reconcile-completeness/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-01
edges:
  part_of: 01KYZTRDEQ5KD76YCNNB7DHEG3
---
# CC Prompt — Slice 15 (Migration Fidelity): Reconcile completeness + collapse the project-vision pair

Two coupled changes so the arc-close freeze produces a genuinely byte-faithful, fully idempotent corpus:
**(1)** collapse the over-engineered project-vision pair to one faithful 1:1 project node, and **(2)** make
*every* ingested family reconcile on drift. **All in code, fixture-proven, `.worktrees/odm` untouched** —
the collapse + reconcile fire live at the arc-close freeze, so **s15 blocks that freeze.**

> **Start condition:** on `release/1.0.x`, green, s14 merged. **Fixture only — no `odm`-branch commit, fire
> nothing on the live store.**

## Background (operator decisions, 2026-08-01)

- **Collapse the `#1000`/`#1001` pair.** Today `#1000` is an editorial-merge **synthesis** (curated
  `# Vision` body) that **supersedes** `#1001`, the byte-1:1 of the full `project-plan.md`. The operator
  judged the split documentation over-engineering (it's also the source of the reconcile gap + the fragile
  "already a synthesis" idempotency). Collapse to **one faithful 1:1 project node**. `#1000` **must
  survive** — it carries all **12 arcs** as `part_of` children; `#1001` has **0** children.
- **"Everything we ingest should reconcile."** Not just the project — artifacts and notes too.

## The gap this closes (arc-close dry-run, config-fixed, 2026-07-31)

The freeze reconciled 2 of 4 drifted nodes and **could not** close `#1001` (1:1 project base, orphaned by
the blanket `node_type == Project` exclusion, selfhost.rs:578/757) or `#509907700` (slice10 ledger artifact,
skipped by mint-only `mint_artifacts`).

## Read first

1. `slice15-reconcile-completeness/ledger.md` (10 rows) + `slice-doc.md` (esp. the **Out** list: the
   `orient` concise-view is the **LLM arc's**, not here — interim verbosity is disclosed & accepted).
2. **ODD-0025** §2.1/§2.3 (the project-vision-as-synthesis you are **reversing**)/§2.6/§2.9; **ODD-0020 v1.4**
   (`source.synthesis` key).
3. **Code:** `selfhost.rs` (`reconcile_source` :578 exclusion; the `matches!(Arc|Slice)` loop :757; the
   import/backfill :332; how the project node is minted/mapped); `synthesis.rs` (`apply_project_vision` :259
   — un-wire from the migrate path, keep the module); `artifact.rs` (`mint_artifacts` :170 — reconcile
   branch + `is_excluded` guard); `notes.rs` (`mint_notes` — reconcile branch); `legacy.rs::is_excluded`.

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `03-error-handling`, `05-type-design`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **Collapse the pair** (F-1/F-2). `#1000` → plain 1:1 project node: re-snapshot its body from
   `project-plan.md`, drop `source.synthesis` + the `supersedes`→`#1001` edge, preserve
   `id`/`number`/`part_of` children; retire `#1001` (supersede-don't-delete, reason recorded). Remove
   `apply_project_vision` from the migrate/self-host path; **keep the `synthesis` module** for future genuine
   syntheses. Make it **idempotent** (a re-run over an already-collapsed store is a no-op). Amend **ODD-0025
   §2.3**.
2. **Key the reconcile-exclusion on `source.synthesis`** (F-3), not `node_type == Project`, at
   selfhost.rs:578/757/332. Post-collapse the project reconciles like any plan node; a genuine
   `source.synthesis`-bearing node stays excluded.
3. **Artifacts + notes mint-or-reconcile** (F-4). A drifted already-covered `artifact`/`note` re-snapshots
   in place (`id`/`number`/`part_of` preserved) instead of being skipped.
4. **`is_excluded` guard in `mint_artifacts`** (F-5).
5. **Fixtures** (F-1…F-7): collapse (1:1 + children intact + `#1001` retired + re-run no-op); project
   reconciles on `project-plan.md` edit; artifact + note reconcile; `is_excluded` guard; a genuine synthesis
   node still excluded; no-collateral / idempotent / dry-run-writes-nothing.

## Constraints (flag, don't silently change)

- **No live store mutation. No `odm`-branch commit.** The collapse + reconcile fire at the arc-close freeze.
- **`#1000` survives; its 12 `part_of` children stay intact.** Never orphan the arcs.
- **Keep the `synthesis` module** — un-wire it from the project vision, don't delete it.
- **Gate stays migration-time-only.** Reconcile only sets a body from its current source, identity preserved.
- **Flag sub-decisions:** retire-vs-remove `#1001`; whether a minimal `orient` vision-view rides along (default
  **no** — it's the LLM arc's, interim verbosity accepted).
- No `unsafe`; typed errors; coverage ≥ 90% on touched code; clippy `-D warnings` clean.

## Deliverables

The code change on `release/1.0.x` (collapse + `source.synthesis` exclusion key + artifact/note reconcile +
`is_excluded` guard) and the ODD-0025 §2.3 amendment; `ledger.md` evidence per row (`attested` → CI — cite
tests, exits); `closing-report.md` — per-row walk, the collapse mechanics + `#1001` disposition, the
sub-decisions, the ODD amendment, **plus the v2.0 Bubble-up** (did s15 collapse the pair + make everything
reconcile; silent-drop diff vs In/Out; confirm the freeze collapses + closes all 4 drifts next). Branch:
`release/1.0.x` only.

## Working agreement

Amend don't work around; flag every deviation; five-iteration cap. If the collapse proves larger than the
reconcile work, land reconcile-completeness + the `is_excluded` guard first and split the collapse, flagging
CDC. Your `done` is proposed-done — CDC reproduces the code + fixtures + CI (esp. `#1000` 1:1 + children
intact, `#1001` retired, synthesis module still green). On close, bubble up to `../arc-plan.md`.
