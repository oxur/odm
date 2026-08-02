# Slice 06 ledger — auto-extend affirmed decomposition on authored additions

Per `LEDGER-DISCIPLINE.md` §A. Code slice. CC fills Evidence at the commit each is
met (`attested`); CDC reproduces. The real-MF row (F-2) is the acceptance anchor.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | An already-affirmed parent that gains a plan-tree-declared work-child has its `decomposed` **auto-extended** by `migrate --all` — includes the new child, `check` clean, **no manual `node decomposed`** | fixture: affirm N slices, author slice N+1, `migrate --all`, assert affirmation ⊇ {N+1} and 0 drift | serious | SS5-1 | done | `odm-migrate/src/decompose.rs::tests::authored_addition_auto_extends_an_affirmed_parent` (module-level fixture) + `odm-cli/tests/migrate.rs::migrate_all_auto_extends_an_affirmed_parent_for_an_authored_addition` (real CLI pipeline: `migrate --all` → RECOMPOSE shows `auto-extended`, `Total: 0 re-affirmed, 1 auto-extended, 0 left as drift`, `check` clean). `cargo test -p odm-migrate --lib decompose` and `cargo test -p odm-cli --test migrate` both green. | the core ask |
| F-2 | **The MF transitional shape auto-heals** (affirmation carrying stale non-work children + a genuine new slice → rewritten to the current work-child set, 0 drift, no manual step) | fixture reproducing MF's shape; and the real store: reset → `migrate --all` → MF 0 drift, no affirm | serious | SS5-1 | deferred | Fixture half attested: `odm-migrate/src/decompose.rs::tests::mf_transitional_shape_drops_stale_non_work_and_extends` — affirmation `[x, artifact]` + new slice `z` → `AutoExtended{added:[z], dropped_stale:[artifact]}`, reloaded affirmation `{x, z}`. Real-store half not attempted: the compound criterion ("fixture … **and** the real store …") is only half satisfied, so this row stays `deferred` rather than `done` rather than softpedal the AND. | **reason**: the odm corpus lives on the orphan `odm` branch at `.worktrees/odm` (ODD-0022 §4.2 locator), not checked out in this worktree (`.worktrees/` is gitignored here per `CLAUDE.md`) — CC has no access to it from this session. **re-entry**: CDC (or the operator, fresh context) runs `cd .worktrees/odm && odm migrate --all` (the reset→migrate cycle already planned for this arc's imminent cycle) and confirms MF `#58837400` at 0 decomposition-drift with no manual `node decomposed`; this is also the acceptance anchor, so it should not be skipped |
| F-3 | A parent that **lost** a work-child still raises `DecompositionDrift` (not auto-healed) | fixture: remove/retire a work-child, `migrate --all`, assert drift flagged | correctness | scope-out | done | `odm-migrate/src/decompose.rs::tests::a_removal_with_no_remap_entry_is_left_as_drift` (module-level: child's node file never persisted) + `odm-cli/tests/migrate.rs::migrate_all_leaves_a_removed_work_child_as_drift_not_auto_healed` (real pipeline: `odm node unlink <slice> part_of <arc>` — `migrate` is never-delete, so detaching containment is how a work-child genuinely leaves a parent's set — then `migrate --all` reports `Total: 0 re-affirmed, 0 auto-extended, 1 left as drift`, `check` still shows `decomposition-drift`). Also guarded against the id-lookup trap: a vanished (not just non-work-typed) affirmed id is kept in `affirmed_work`, not silently dropped as "stale" — see the module doc comment and the `types.get(id).is_none_or(...)` filter. | removals stay surfaced |
| F-4 | A **never-affirmed** parent is **not** auto-affirmed — it stays `undecomposed-parent` | fixture: undecomposed parent with children, `migrate --all`, assert still undecomposed | serious | scope-out | done | `odm-migrate/src/decompose.rs::tests::a_never_affirmed_parent_is_untouched` — parent with a work-child but no `affirm_decomposed` call; `auto_recompose` skips it entirely (`report.changed.is_empty()`), reloaded node still has `decomposed().is_none()`. | first affirm is a human act |
| F-5 | **Idempotent** — a second `migrate --all` after an auto-extend extends nothing | fixture: run twice, assert 0 auto-extends on the second | correctness | standing | done | `odm-migrate/src/decompose.rs::tests::a_second_run_after_auto_extend_reports_nothing_new` — first `auto_recompose` call auto-extends 1, second call on the same store reports `changed.is_empty()`. | |
| F-6 | The `RECOMPOSE` report/output distinguishes **auto-extended** from **left as drift** (new outcome surfaced) | CLI output shows the auto-extend line; `--json` carries the outcome | polish | scope-in | done | `crates/odm-cli/src/migrate.rs::render_recompose` now matches `Outcome::AutoExtended` to `"auto-extended"` / `"would auto-extend"` (dry-run), distinct from `"re-affirmed"` and `"drift (needs \`node decomposed\`)"`; summary line reports all three counts. Exercised by both new `odm-cli/tests/migrate.rs` integration tests (assert on `"auto-extended"` and the `Total: …` line). **Caveat**: `migrate --all` has no `--json` output path at all today (checked — no `Serialize`/`json` handling in `migrate.rs`), so the `--json` half of Verify doesn't apply to the current surface; the `Outcome` enum already carries the distinguishing data (`added`/`dropped_stale`), so wiring `--json` later is mechanical, not a redesign. | no `--json` surface exists yet for RECOMPOSE to extend |
| F-7 | No regression: slice 05's `ReAffirmed` (identity churn) and the undeclared-addition seam still hold; `make check` green | `make check` exit 0; slice 05 tests pass | correctness | standing | done | `cargo test -p odm-migrate --lib decompose`: all 9 tests pass, incl. slice 05's `identity_remint_auto_reaffirms_with_the_new_ids`, `dry_run_reports_the_reaffirmation_but_writes_nothing`, `an_up_to_date_parent_is_not_reported`, `a_non_work_child_added_alongside_never_triggers_a_report`. `cargo test -p odm-cli --test migrate`: all 33 tests pass. `make lint`: clippy + rustfmt clean. `make format`: applied, no functional diff. `make test` (full workspace): see Closure. | refines, doesn't remove, the seam |

## What Worked

- **The additions-only signal fell out of slice 05's own filter, cleanly.** `decomposition_children`
  already gave the "current work-children" half for free; the only new piece was filtering the
  *affirmed* side the same way before comparing, so the `removed`/`added` diff slice 05 already computed
  just needed a superset check ahead of the identity-remint check. No new data model, no new pass over
  the corpus.
- **The vanished-vs-non-work distinction would have been a silent bug without the F-3 fixture.**
  Filtering the affirmed set with `types.get(id).is_some_and(is_work)` (mirroring `decomposition_children`
  exactly) looks right at a glance, but treats "id absent from the corpus" and "id present but
  non-work-typed" identically — both read as `false`/`None`. Writing `a_removal_with_no_remap_entry_is_left_as_drift`
  against the new code *before* trusting the change surfaced that a genuinely deleted work-child would
  have been silently dropped from the affirmation as if it were stale artifact cruft. The fix
  (`is_none_or` — keep an id unless it's *known* to be non-work) is one line different from the wrong
  version and the test is the only thing that tells them apart.
- **The real CLI pipeline test caught what the unit fixture couldn't**: `migrate` never deletes nodes
  (`selfhost.rs`'s "never-delete" invariant), so a plan-tree-doc deletion doesn't produce a removed
  work-child at all — the corresponding integration test needed `odm node unlink <slice> part_of <arc>`
  to construct a genuine removal, the same mechanism a real retire/reparent would use.
- **The pre-existing slice05 integration test was the exact scenario slice06 flips.**
  `migrate_all_leaves_a_genuinely_new_slice_as_drift_not_auto_affirmed` asserted the old (pre-s06)
  behavior for precisely the case s06 exists to fix; updating rather than deleting it (renamed to
  `migrate_all_auto_extends_an_affirmed_parent_for_an_authored_addition`) kept the regression coverage
  while flipping its assertion to match the new contract — a clean signal that the change landed where
  intended.

## Closure

Closed 2026-08-02. Verified by: CC (this session) — attested for F-1/F-3/F-4/F-5/F-6/F-7 (module-level
fixtures + real end-to-end `odm-cli` integration tests, not just the pure `auto_recompose` function).
F-2's fixture half is attested the same way; its real-store half is **deferred** to CDC (see F-2's row for
reason + re-entry) — the acceptance anchor is not fully closed until that leg runs. `make format` +
`make lint` + `make test` (full workspace, all crates + doctests) all green. CDC reproduction of every
`done` row, plus F-2's deferred real-store leg, is the open item. Rows: 7. Done: 6. Deferred: 1. No-op: 0.
