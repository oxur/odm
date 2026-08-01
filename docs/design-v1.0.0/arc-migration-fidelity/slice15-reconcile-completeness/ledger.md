# Slice 15 (Migration Fidelity): Reconcile completeness + collapse the project-vision pair

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. **Capability
> slice — fixture-only; `.worktrees/odm` untouched.** Code/fixtures class-(a): CDC reproduces by direct
> read; runtime attested→CI. The collapse + full reconcile fire live at the **arc-close freeze**. Five-
> iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **Project-vision pair collapsed**: `#1000` → plain 1:1 project node (body == `project-plan.md`, gate passes, no `source.synthesis`, no `supersedes`), `id`/`number`/`part_of`-children preserved; `#1001` retired (supersede-don't-delete, reason recorded) | fixture: run the collapse → `#1000` 1:1 + 12 children intact, `#1001` retired; ODD-0025 §2.3 amended | serious (the model change) | operator 2026-08-01 | open | | `#1000` must survive (12 arcs `part_of` it); `#1001` has 0 children. |
| F-2 | **`apply_project_vision` off the migrate path; synthesis library kept**: the project migrates as a normal plan node; the `synthesis` module (s11) stays for future genuine syntheses | read the migrate/self-host flow: no vision-apply call; `synthesis` module + its tests intact | serious | s11 capability | open | | Un-wire, don't delete — the capability is still valid, just not used for the project vision. |
| F-3 | **Reconcile-exclusion keys on `source.synthesis`**, not `node_type == Project`: post-collapse the project reconciles like any plan node; a `source.synthesis`-bearing node stays excluded | fixture: edit `project-plan.md` → `#1000` re-snapshots; a seeded synthesis node is *not* 1:1-reconciled | serious (MF-9 fidelity) | arc-close dry-run finding | open | | Sites: selfhost.rs:578/757/332. ODD-0020 v1.4 key. |
| F-4 | **Artifacts + notes mint-or-reconcile**: a drifted already-covered `artifact`/`note` re-snapshots in place (`id`/`number`/`part_of` preserved), not skipped | fixture: drift a minted artifact + a minted note → both re-snapshot, identity preserved | serious (MF-9 fidelity; "everything ingested reconciles") | operator D-1 | open | | Closes `#509907700` (slice10 ledger) + the same gap in the note family. |
| F-5 | **`mint_artifacts` `is_excluded` guard**: `index.md` (basename) + any `templates/`-component path never minted, regardless of root | fixture: `index.md` + `templates/x.md` under the artifacts root → 0 minted | correctness (latent-bug fix) | this session's finding | open | | The guard the coverage report + notes pass already have. |
| F-6 | **Idempotent + dry-run-safe**: a second run over an already-collapsed/reconciled store is 0-change on every path (collapse, reconcile, mint); `--dry-run` writes nothing | run twice: second pass 0/0/0; dry-run leaves store byte-identical | serious (operator req: idempotency through the migration period) | s13 protocol | open | | "already a plain project node" is the collapse's idempotent no-op. |
| F-7 | **No collateral**: only body (+ `updated`) changes on reconciled nodes; `#1000`'s children + ids/schema/edges intact; retired + already-faithful nodes untouched; genuine synthesis nodes untouched | direct read of fixtures: diffs show body/updated only where expected; `#1000` children preserved | serious | slice-doc | open | | |
| F-8 | **Gate stays migration-time-only; model amended not worked around**: reconcile only sets a body from its current source; ODD-0025 §2.3 reversed (cited), §2.6/§2.9 lines if needed | cross-read: gate not continuous; §2.3 amendment present + coherent | correctness | ODD-0025 | open | | The 1:1 rule + §2.1 gate unchanged; s15 removes the project's synthesis special-case + extends *which* nodes reconcile. |
| F-9 | **No live mutation; no downstream pulled forward**: no `odm`-branch commit; no freeze/P-12; no L-8; `orient` view left to the LLM arc (interim verbosity disclosed) | `git -C .worktrees/odm status` clean; HEAD `e06fffe`; no `orient` render change | serious | LEDGER-DISCIPLINE | open | | The collapse fires live at the freeze. |
| F-10 | **Clippy clean; no `unsafe`; coverage ≥ 90% on touched code** | clippy `-D warnings` exit 0; `! grep unsafe`; `llvm-cov` ≥ 90% on changed | polish/correctness | CLAUDE.md | open | | The collapse, the exclusion-key change, the artifact/note reconcile branches, the guard. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Fixture-only — no store commit. Verified by: `<CC then CDC>`. Rows: 10. Done: `<n>`.
Deferred: `<n>`. On close, bubble up to `../arc-plan.md`: s15 done (project collapsed to one 1:1 node;
everything ingested reconciles); **the arc-close resumes** — the freeze collapses + closes all 4 drifts,
then the P-12 demo, then Migration Fidelity closes.
