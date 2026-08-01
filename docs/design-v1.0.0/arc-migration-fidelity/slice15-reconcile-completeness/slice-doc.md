# Slice 15 (Migration Fidelity) — Reconcile completeness: the 1:1 project node + artifacts (plan-of-record)

> Refs: `../arc-plan.md` (s15 inserted before the arc-close freeze) · **s12/s13** (the reconcile capability
> + vision mint this completes) · **ODD-0025** §2.1 (migration-time gate), §2.3 (project synthesis + its
> 1:1 base), §2.6 (artifacts), §2.9 (living-plan reconcile) · **ODD-0020 v1.4** (the `source.synthesis`
> carve-out this reuses as the exclusion key) · `crates/odm-migrate/src/{selfhost.rs,synthesis.rs,artifact.rs}`.
> `depends_on:` s13.
>
> **Capability slice — fixture-proven; no live `odm`-branch mutation.** The completed reconcile is then
> fired live by the **arc-close final reconcile-and-freeze**. **s15 blocks that freeze** — the freeze must
> close *all* living drift, which today it cannot.

## Goal

Close the two reconcile gaps the arc-close dry-run exposed, so the final freeze produces a genuinely
byte-faithful corpus. **Done when** (1) the **1:1 `project-plan` node** re-snapshots to its current source
when `project-plan.md` drifts — the blanket `node_type == Project` reconcile-exclusion is narrowed to key
on `source.synthesis` (protect the vision synthesis `#1000`, whose body is editorial; reconcile the 1:1
base `#1001`, whose body must stay faithful); and (2) a drifted **artifact** node re-snapshots its body
instead of being silently skipped (D-1) — plus the `mint_artifacts` `is_excluded` guard, so index/template
files can never be minted regardless of root. All fixture-proven, `.worktrees/odm` untouched.

## Why

The arc-close final-reconcile dry-run (2026-07-31, correct config) reconciled 2 of 4 drifted nodes and
**structurally could not** close the other 2:

- **`#1001`, the 1:1 `project-plan` node** — drifted when `project-plan.md` was edited (the arc-close
  bubble-up). `selfhost::reconcile_source` returns `None` for *any* `node_type == Project` (selfhost.rs:578),
  and the live-reconcile loop only visits `Arc | Slice` (selfhost.rs:757). That blanket exclusion exists to
  protect the **synthesis** `#1000` (its body is the editorial vision, never a 1:1 re-snapshot; ODD-0025
  §2.3) — but it also orphans `#1001`, whose *entire purpose is fidelity*. `apply_project_vision` is an
  idempotent no-op once the synthesis exists, so nothing re-snapshots the 1:1 base. **The faithful 1:1 node
  cannot stay faithful** — the exact incoherence a fidelity arc must not ship.
- **`#509907700`, the slice10 `ledger.md` node** (an `artifact`) — drifted when the ledger gained its
  *Closure* prose after mint. `mint_artifacts` only mints *uncovered* docs; an already-covered artifact
  whose source drifted is skipped, never re-snapshotted (artifact.rs). So artifact drift never closes.

Both are living-doc drift (§2.9) with **no reconcile owner**. Same root cause as s12/s13's living-plan
work, one layer out: the reconcile capability was built for design/research + `Arc`/`Slice` plan nodes and
never extended to the 1:1 project base or the artifact family. A third, adjacent defect surfaced the same
session: `mint_artifacts` never applies `legacy::is_excluded`, so it mints `index.md`/`templates/*` when
the root makes them classify as `Other` (masked today only by the correct parent root) — folded in here
since it's the same file and the same "make the artifact pass correct" theme.

## Scope

**In (capability; `release/1.0.x` code + fixtures; `.worktrees/odm` untouched):**

- **Reconcile the 1:1 `project-plan` node** (F-1). Narrow the reconcile-exclusion from `node_type ==
  Project` to **`source.synthesis` present** — a synthesis-bearing node (`#1000`) stays excluded (its body
  is editorial, ODD-0025 §2.3); a project node *without* `source.synthesis` (`#1001`, the 1:1 base) is
  reconciled to current `project-plan.md`, `id`/`edges`/`status` preserved, the §2.1 body gate re-passing.
  Keyed on `source.synthesis` for consistency with **ODD-0020 v1.4**. Touches the three exclusion sites:
  `reconcile_source` (selfhost.rs:578), the live-reconcile loop's `matches!(Arc|Slice)` filter
  (selfhost.rs:757), and — if it gates the same node — the import/backfill exclusion (selfhost.rs:332).
  **Decision D-2 (flag):** the natural home for the 1:1-base reconcile is either self-host's reconcile
  (broaden the exclusion + ensure `#1001` maps to `project-plan.md` in `plan_by_key`) **or**
  `apply_project_vision` (which already owns the `#1000`/`#1001` pair — make it re-snapshot `#1001` on
  drift, and refresh `#1000`'s vision body only if the DoD section it distills changed). Recommend the
  locus that keeps the pair's invariants in one place (likely vision-apply); flag the choice + why.
- **Reconcile drifted artifacts** (F-2, **D-1**). `mint_artifacts` becomes **mint-or-reconcile**: an
  already-covered artifact whose body no longer matches its source is re-snapshotted in place
  (`id`/`number`/`edges`/`part_of` preserved), not skipped. Recommended resolution (a fidelity arc should
  not ship stale artifact bodies); the alternative — declare artifacts point-in-time snapshots outside the
  fidelity gate (an ODD-0025 §2.6 line) — is a legitimate model call, so **flag D-1** with the
  recommendation and let the operator ratify.
- **`mint_artifacts` `is_excluded` guard** (F-3). Apply `legacy::is_excluded` in `mint_artifacts` (skip any
  `index.md` by basename, any `templates/`-component path) so those infra files are never minted regardless
  of the root — the belt-and-suspenders the coverage report and notes pass already have.
- **The model line.** Amend **ODD-0025** (cited) if the 1:1-base reconcile or artifact reconcile needs a
  normative sentence (§2.3 for the base, §2.6 for artifacts, §2.9 for the living-doc policy). Amend, don't
  work around.

**Out:**

- **The live re-run** — the arc-close **final reconcile-and-freeze** fires the completed reconcile on
  `.worktrees/odm` (closing all 4 drifts: `#22`, `#58837400`, `#1001`, `#509907700`). Not here.
- **The P-12 demo** and the rest of the arc-close.
- **L-8 / CDC-ARC-1** (the RH-era design-node `version` back-fill) — post-close.
- Any change to the synthesis model itself — `#1000` stays the editorial-merge synthesis, untouched; s15
  only stops *excluding* the 1:1 base from reconcile.

## Verification

Fixture, class-(a) — code + `TempDir` fixtures; runtime execution attested→CI. After the change:

- **1:1 project reconcile:** a fixture edits `project-plan.md` after the vision is minted, runs the
  reconcile, and asserts `#1001`'s body re-snapshots to the new source (gate re-passes), `id`/`edges`/
  `status` intact, while `#1000` (synthesis) is **untouched** (body, `supersedes`, attestation all stable).
- **Artifact reconcile (D-1):** a fixture drifts an already-minted artifact's source and asserts the node
  re-snapshots in place (same `id`/`number`), and that an *un*-drifted artifact + a re-run are no-ops.
- **`is_excluded` guard:** a fixture with `index.md` / `templates/x.md` directly under the artifacts root
  asserts neither is minted.
- **Idempotent / dry-run:** a second identical run is 0-change on every path; `--dry-run` writes nothing.
- **No collateral:** only body (+ `updated`) changes on reconciled nodes; the synthesis, retired, and
  already-faithful nodes are untouched.

Runtime rows (`cargo`/`clippy`/`llvm-cov`; any live exec) attested-by-CC → CI; CDC reproduces code +
fixtures by direct read.

## Rollback & findings discipline

Fixture-only — no snapshot/revert gate. Amend-don't-work-around: the exclusion narrowing and any artifact
policy are ODD-0025 lines, not silent behavior. Flag **D-1** (artifacts reconcile vs snapshot-scope-out)
and **D-2** (reconcile locus: self-host vs vision-apply) with resolutions. The gate stays migration-time-only
— reconcile re-establishes fidelity, it does not become a continuous check. Five-iteration cap; if the
1:1-base mapping turns out to be more than wiring, split it from the artifact work and flag.

## Exit

`ledger.md` closed; CDC-verified against code + fixtures. The 1:1 project node and drifted artifacts both
reconcile; index/template can't be minted. On close, bubble up to `../arc-plan.md`: s15 done (reconcile
complete); **the arc-close resumes** — the final reconcile-and-freeze now closes all 4 living drifts →
P-12 demo → Migration Fidelity closes, genuinely byte-faithful.
