# CC Prompt — Slice 15 (Migration Fidelity): Reconcile completeness (1:1 project node + artifacts)

Close the two reconcile gaps the arc-close dry-run exposed so the final freeze can produce a genuinely
byte-faithful corpus. **All in code, fixture-proven, `.worktrees/odm` untouched** — the completed reconcile
is fired live by the arc-close freeze, so **s15 blocks that freeze.**

> **Start condition:** on `release/1.0.x`, green, s14 merged. **Fixture only — no `odm`-branch commit,
> fire nothing on the live store.** Capability fix to the reconcile paths; the operator runs the corrected
> `migrate --all` live at the arc-close.

## The gap (arc-close dry-run, 2026-07-31, correct config)

The freeze reconciled 2 of 4 drifted nodes and **structurally could not** close the other 2:

- **`#1001`, the 1:1 `project-plan` node** — `reconcile_source` returns `None` for *any* `node_type ==
  Project` (`selfhost.rs:578`) and the live-reconcile loop only visits `Arc | Slice` (`selfhost.rs:757`).
  That blanket exclusion is meant to protect the **synthesis `#1000`** (editorial body, never 1:1;
  ODD-0025 §2.3) — but it also orphans `#1001`, whose whole job is fidelity. `apply_project_vision` no-ops
  once the synthesis exists, so nothing re-snapshots the base.
- **`#509907700`, the slice10 `ledger.md` artifact** — `mint_artifacts` only mints *uncovered* docs; a
  drifted already-covered artifact is skipped, never re-snapshotted.

## Read first

1. `slice15-reconcile-completeness/ledger.md` (10 rows) — the spec of "done". `slice-doc.md` — esp.
   **D-1** (artifacts reconcile) and **D-2** (reconcile locus). **ODD-0025** §2.1/§2.3/§2.6/§2.9;
   **ODD-0020 v1.4** (the `source.synthesis` carve-out — reuse the same key).
2. **The code you change:**
   - `crates/odm-migrate/src/selfhost.rs` — `reconcile_source` (line ~578: the `node_type == Project`
     exclusion → key on `source.synthesis`), the live-reconcile loop filter (line ~757:
     `matches!(Arc|Slice)`), the import/backfill exclusion (line ~332), and how `#1001` maps into
     `plan_by_key` so it's reachable against `project-plan.md`.
   - `crates/odm-migrate/src/synthesis.rs` — `apply_project_vision` (line ~259): candidate home for the
     1:1-base reconcile (D-2).
   - `crates/odm-migrate/src/artifact.rs` — `mint_artifacts` (line ~170): add the reconcile-drifted branch
     (D-1) and the `legacy::is_excluded` guard.
   - `crates/odm-migrate/src/legacy.rs` — `is_excluded` (reuse; already `pub(crate)`).

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `03-error-handling`, `05-type-design`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **Narrow the project reconcile-exclusion** (F-1/F-2). Change the reconcile-exclusion from `node_type ==
   Project` to **`source.synthesis` present**: a synthesis-bearing node (`#1000`) stays excluded; a project
   node *without* `source.synthesis` (`#1001`) reconciles to current `project-plan.md`, `id`/`edges`/`status`
   preserved, the §2.1 gate re-passing. Apply at all three sites (578/757/332) consistently. **Keep the
   synthesis protected** — verify `#1000` is byte-identical after a run.
2. **Decide the reconcile locus** (F-3, **D-2**). Put the 1:1-base reconcile where the `#1000`/`#1001` pair
   invariants stay in one place — self-host reconcile (broaden the exclusion + map `#1001` into
   `plan_by_key`) or `apply_project_vision` (re-snapshot `#1001` on drift; refresh `#1000`'s vision body
   only if the DoD section it distills changed). Recommend + flag. If mapping `#1001` is more than wiring,
   split it out and flag.
3. **Artifact mint-or-reconcile** (F-4, **D-1**). A drifted already-covered artifact re-snapshots its body
   in place (`id`/`number`/`part_of` preserved) instead of being skipped. Recommend this; if you'd argue
   for scoping artifacts out of the gate instead (an ODD-0025 §2.6 line), flag it for the operator rather
   than choosing silently.
4. **`is_excluded` guard in `mint_artifacts`** (F-5). Apply `legacy::is_excluded` so `index.md`/`templates/*`
   are never minted regardless of root.
5. **Model line** (F-8). Amend ODD-0025 (§2.3/§2.6/§2.9) if the 1:1-base or artifact reconcile needs a
   normative sentence. Amend, don't work around.
6. **Fixtures** (F-1…F-7): 1:1 project re-snapshot (synthesis untouched); artifact re-snapshot (identity
   preserved); `is_excluded` guard; idempotence + dry-run-writes-nothing; no-collateral.

## Constraints (flag, don't silently change)

- **No live store mutation. No `odm`-branch commit.** The live re-run is the arc-close freeze.
- **The synthesis `#1000` stays excluded and untouched** — s15 narrows the exclusion to *admit the 1:1
  base*, it does not touch the synthesis model.
- **The gate stays migration-time-only** — reconcile only ever sets a body from its current source, identity
  preserved.
- **Flag D-1 (artifact reconcile vs snapshot-scope-out) and D-2 (reconcile locus)** with your resolution.
- No `unsafe`; typed errors; coverage ≥ 90% on touched code; clippy `-D warnings` clean.

## Deliverables

The code change on `release/1.0.x` (exclusion narrowing + 1:1-base reconcile + artifact mint-or-reconcile +
the `is_excluded` guard) and any ODD-0025 amendment; `ledger.md` evidence per row (`attested` → CI — cite
test names, exits); `closing-report.md` — per-row walk, the D-1/D-2 decisions + rationale, any ODD
amendment, **plus the v2.0 Bubble-up** (did s15 make the 1:1 project node + artifacts reconcile; the
silent-drop diff vs In/Out; confirm the arc-close freeze now closes all 4 drifts next). Branch:
`release/1.0.x` only.

## Working agreement

Amend don't work around; flag every deviation; five-iteration cap. The live run is carved to the arc-close,
so this should fit one context — if the 1:1-base mapping proves large, land the artifact + `is_excluded`
work first and split the project-node reconcile, flagging CDC. Your `done` is proposed-done — CDC reproduces
the code + fixtures + CI. On close, bubble up to `../arc-plan.md` (reconcile complete; the arc-close freeze
closes all 4 drifts next).
