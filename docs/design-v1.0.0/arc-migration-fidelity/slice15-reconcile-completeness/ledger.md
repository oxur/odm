# Slice 15 (Migration Fidelity): Reconcile completeness — the 1:1 project node + artifacts

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. **Capability
> slice — fixture-only; `.worktrees/odm` untouched.** Code/fixtures class-(a): CDC reproduces by direct
> read; runtime execution attested→CI. The completed reconcile is fired live by the **arc-close freeze**.
> Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **1:1 project-plan node reconciles**: the reconcile-exclusion keys on `source.synthesis` (present → excluded), **not** `node_type == Project`; a synthesis-less project node (`#1001`) re-snapshots to current `project-plan.md`, gate re-passes | fixture: edit `project-plan.md` post-vision-mint → `#1001` body matches new source, `id`/`edges`/`status` preserved | serious (MF-9 fidelity) | arc-close dry-run finding | open | | Sites: `reconcile_source` (selfhost.rs:578), live-reconcile loop `matches!(Arc\|Slice)` (selfhost.rs:757), import/backfill (selfhost.rs:332). Key on `source.synthesis` (ODD-0020 v1.4 consistency). |
| F-2 | **Synthesis `#1000` protected**: a `source.synthesis`-bearing project node is still excluded — body, `supersedes`, attestation untouched by reconcile | fixture: same run, `#1000` byte-identical before/after; lineage intact | serious (don't clobber the vision) | ODD-0025 §2.3 | open | | The narrowing must *keep* the synthesis excluded — that was the original, valid reason for the exclusion. |
| F-3 | **1:1-base reconcile locus decided** (D-2): the 1:1 project-node reconcile lives in self-host **or** `apply_project_vision`, whichever keeps the `#1000`/`#1001` pair's invariants in one place; `#1001` maps to `project-plan.md` so it's reachable | read the chosen path; confirm `#1001` is found + reconciled (not orphaned as today) | serious | slice-doc D-2 | open | | **D-2 flag.** Recommend vision-apply (owns the pair). If bigger than wiring, split + flag. |
| F-4 | **Drifted artifacts reconcile** (D-1): `mint_artifacts` becomes mint-or-reconcile — an already-covered artifact whose body drifted re-snapshots in place (`id`/`number`/`part_of` preserved), not skipped | fixture: drift a minted artifact's source → node re-snapshots, identity preserved; slice10 ledger case (#509907700) covered by the shape | serious (MF-9 fidelity) | arc-close dry-run finding | open | | **D-1 flag.** Recommend reconcile; alt = scope artifacts out of the gate (ODD-0025 §2.6 line). Ratify with operator. |
| F-5 | **`mint_artifacts` `is_excluded` guard**: `index.md` (basename) + any `templates/`-component path never minted, regardless of root | fixture: `index.md` + `templates/x.md` under the artifacts root → 0 minted | correctness (latent-bug fix) | this session's finding | open | | The guard the coverage report + notes pass already have; closes the root-fragile exclusion. |
| F-6 | **Idempotent + dry-run-safe**: second identical run = 0-change on every path (1:1 reconcile, artifact reconcile, mint); `--dry-run` writes nothing | run twice: 0 reconciled / 0 minted second pass; dry-run leaves store byte-identical | serious | s13 protocol | open | | |
| F-7 | **No collateral**: only body (+ `updated`) changes on reconciled nodes; synthesis, retired, already-faithful nodes untouched; ids/schema/edges/status intact | direct read of fixtures: diffs show body/updated only on the reconciled nodes | serious | slice-doc | open | | |
| F-8 | **Gate stays migration-time-only; no model drift**: reconcile only sets a body from its current source; any normative line amended in ODD-0025 (§2.3/§2.6/§2.9), not worked around | cross-read: gate not made continuous; ODD amended if a line is needed | correctness | ODD-0025 | open | | The 1:1 rule + §2.1 gate unchanged; s15 extends *which nodes* reconcile, not the gate. |
| F-9 | **No live mutation; no downstream pulled forward**: no `odm`-branch commit; no arc-close/P-12/freeze run; no L-8 | `git -C .worktrees/odm status` clean; HEAD unchanged at `e06fffe` | serious | LEDGER-DISCIPLINE | open | | The live re-run is the arc-close freeze. |
| F-10 | **Clippy clean; no `unsafe`; coverage ≥ 90% on touched code** | clippy `-D warnings` exit 0; `! grep unsafe`; `llvm-cov` ≥ 90% on changed | polish/correctness | CLAUDE.md | open | | The exclusion-narrowing, the artifact reconcile branch, and the `is_excluded` guard are the new logic to cover. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Fixture-only — no store commit. Verified by: `<CC then CDC>`. Rows: 10. Done: `<n>`.
Deferred: `<n>`. On close, bubble up to `../arc-plan.md`: s15 done (reconcile complete — 1:1 project node +
artifacts); **the arc-close resumes** — the final reconcile-and-freeze closes all 4 living drifts, then the
P-12 demo, then Migration Fidelity closes.
