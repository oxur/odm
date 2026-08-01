# Slice 15 (Migration Fidelity) — Reconcile completeness + collapse the project-vision pair

> Refs: `../arc-plan.md` (s15, before the arc-close freeze) · **s11–s13** (the synthesis capability +
> vision mint this *un-wires from the project*, keeping the capability) · **ODD-0025** §2.1 (gate), §2.3
> (project vision — **being reversed** here), §2.6 (artifacts), §2.9 (living-plan reconcile) · **ODD-0020
> v1.4** (`source.synthesis` as the exclusion key) · `crates/odm-migrate/src/{selfhost.rs,synthesis.rs,artifact.rs,notes.rs}`.
> `depends_on:` s13.
>
> **Capability slice — fixture-proven; no live `odm`-branch mutation.** The collapse + completed reconcile
> fire live at the **arc-close final reconcile-and-freeze**. **s15 blocks that freeze.**

## Goal

Two coupled changes so the freeze produces a genuinely byte-faithful, fully idempotent corpus:

1. **Collapse the project-vision pair** (operator decision, 2026-08-01 — the split was documentation
   over-engineering). Today the project is *two* nodes: `#1000`, an editorial-merge **synthesis** whose
   body is a curated `# Vision` statement, **supersedes** `#1001`, the byte-1:1 of the full
   `project-plan.md`. Collapse to **one faithful 1:1 project node**: `#1000` (which carries all 12 arcs as
   `part_of` children — it must survive) becomes the plain 1:1 of `project-plan.md`, dropping
   `source.synthesis` and the `supersedes` edge; `#1001` (0 children) is **retired** (supersede-don't-delete).
   The vision becomes a **rendered view** of the one node, not a second node.
2. **Reconcile everything we ingest** (operator: "everything we ingest should reconcile"). Every
   source-bearing family re-snapshots its body when its source drifts — plan nodes (incl. the now-1:1
   project), design/research, **artifacts**, and **notes**. The **sole** reconcile exclusion becomes a node
   carrying `source.synthesis` (a genuine synthesis body is not a 1:1 copy) — which, post-collapse, is
   *nothing* for the project, and stays available for future genuine syntheses.

**Done when** `#1000` is the faithful 1:1 project node (gate passes, 12 `part_of` children intact),
`#1001` is retired, `apply_project_vision` is off the migrate path (the synthesis *library* stays), the
reconcile-exclusion keys on `source.synthesis` (not `node_type == Project`), artifacts and notes are
mint-or-reconcile, the `mint_artifacts` `is_excluded` guard is in, and it's all idempotent + `--dry-run`-safe
— fixture-proven, `.worktrees/odm` untouched.

## Why

The arc-close final-reconcile dry-run (config-fixed, 2026-07-31) reconciled 2 of 4 drifted nodes and
**structurally could not** close `#1001` (the 1:1 project base — orphaned by the blanket `node_type ==
Project` reconcile-exclusion, selfhost.rs:578/757) or `#509907700` (the slice10 ledger artifact — skipped
because `mint_artifacts` only mints *uncovered* docs). Digging into `#1001` surfaced the deeper issue: the
`#1000`/`#1001` synthesis-plus-1:1-base pair (ODD-0025 §2.3) is where both the reconcile gap *and* the
fragile "already a synthesis" idempotency come from — two project-typed nodes from one source, one a
"synthesis" whose body is a near-verbatim derivation of the other. The operator's requirements — **preserve
fidelity + provenance, and full idempotency through the migration period** — are best served by removing the
over-engineering: one faithful 1:1 project node (fidelity + provenance via the `source` record), the vision
as a view, and a single uniform rule "everything ingested reconciles." That collapses the special case
instead of adding machinery to it.

## Scope

**In (capability; `release/1.0.x` code + fixtures; `.worktrees/odm` untouched):**

- **Collapse (F-1/F-2).** `#1000` → plain 1:1 project node: body re-snapshot from `project-plan.md`,
  `source.synthesis` + `supersedes`→`#1001` dropped, `id`/`number`/`part_of`-children preserved; `#1001`
  retired (supersede-don't-delete, reason recorded). Remove `apply_project_vision` from the migrate/self-host
  path (the project migrates as a normal plan node); **keep the `synthesis` module** (s11) for future genuine
  syntheses. **Idempotent:** a re-run over an already-collapsed store is a 0-change no-op ("already a plain
  project node"). Amend **ODD-0025 §2.3** (project vision = a 1:1 node + a rendered view, not a synthesis).
- **Reconcile-exclusion keys on `source.synthesis`** (F-3). Replace `node_type == Project` at the three
  sites (selfhost.rs:578 `reconcile_source`, :757 the `matches!(Arc|Slice)` loop filter, :332 the
  import/backfill) with "excluded iff the node carries `source.synthesis`". Post-collapse the project has
  none → it reconciles like any plan node; genuine future syntheses stay excluded. (ODD-0020 v1.4 key.)
- **Artifacts + notes mint-or-reconcile** (F-4). A drifted already-covered `artifact` (`mint_artifacts`)
  or `note` (`mint_notes`) re-snapshots its body in place (`id`/`number`/`part_of` preserved) instead of
  being skipped — closing `#509907700` and the same latent gap in the note family.
- **`mint_artifacts` `is_excluded` guard** (F-5). `index.md` (basename) + any `templates/`-component path
  never minted, regardless of root — the guard the coverage report + notes pass already have.

**Out:**

- **The live re-run** — the arc-close **freeze** fires the collapse + full reconcile on `.worktrees/odm`
  (closing all 4 drifts, collapsing the project, retiring `#1001`). Not here.
- **`orient`'s concise vision view.** Post-collapse `orient` shows the project node's body (now the full
  plan). Rendering a **concise vision view** (extract §1 / a 2-line summary) is command-surface work already
  punch-listed to the **LLM-command-surface arc** (`command-surface-uat-checklist.md`, the `orient` row).
  **Disclosed interim consequence:** until that lands, `orient` is more verbose than the old distilled-vision
  body. Acceptable (already tracked); flag if you'd rather fold a minimal view in.
- **Retiring the numeric display handles** — a command-surface item (LLM arc, punch B2-2), not this arc.
- **L-8 / CDC-ARC-1**, the P-12 demo, the rest of the arc-close.

## Verification

Fixture, class-(a) — code + `TempDir` fixtures; runtime execution attested→CI. After the change:

- **Collapse:** a fixture with a minted vision pair runs the migrate/collapse and asserts `#1000` is a
  plain 1:1 project node (body == `project-plan.md`, gate passes, no `source.synthesis`, no `supersedes`),
  its `part_of` children intact, and `#1001` retired; a second run is a no-op.
- **Project reconciles:** edit `project-plan.md`, re-run → `#1000` re-snapshots (it's no longer excluded).
- **Artifact + note reconcile:** drift a minted artifact and a minted note → both re-snapshot, identity
  preserved; un-drifted + re-run are no-ops.
- **`is_excluded` guard:** `index.md`/`templates/x.md` under the artifacts root → 0 minted.
- **Synthesis capability intact:** the `synthesis` module's own tests still pass (a genuine synthesis node
  is still excluded from 1:1 reconcile by the `source.synthesis` key).
- **No collateral / idempotent / dry-run-safe** as usual.

Runtime rows attested-by-CC → CI; CDC reproduces code + fixtures by direct read.

## Rollback & findings discipline

Fixture-only — no snapshot/revert gate. **Amend, don't work around:** the collapse is an ODD-0025 §2.3
reversal (a cited amendment), not silent behavior; the `source.synthesis` exclusion key and any artifact/
note policy get their model lines. Flag any sub-decision (e.g. retire-vs-remove `#1001`; whether a minimal
`orient` view rides along). The gate stays migration-time-only. Five-iteration cap; if the collapse proves
larger than the reconcile work, land reconcile-completeness + the guard first and split the collapse,
flagging CDC.

## Exit

`ledger.md` closed; CDC-verified against code + fixtures. One faithful 1:1 project node; `#1001` retired;
every ingested family reconciles; index/template never minted; the synthesis capability preserved but
un-wired from the project. On close, bubble up to `../arc-plan.md`: s15 done; **the arc-close resumes** —
the freeze collapses the project + closes all 4 drifts live → P-12 demo → Migration Fidelity closes,
genuinely byte-faithful.
