---
id: 01KZ1YCP1D6F91XH3X2YY82JND
number: 531565100
type: artifact
schema: artifact/v1.1
name: 'Slice 05 ledger — decomposition bookkeeping: consistency + auto-recompose'
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice05-decomposition-bookkeeping/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK41S3RNWSHHCWMVMTRH
---
# Slice 05 ledger — decomposition bookkeeping: consistency + auto-recompose

Per `LEDGER-DISCIPLINE.md` §A. Code slice: rows are grep/test-verifiable. CC fills
Evidence at the commit each is met (strength `attested`); CDC reproduces.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **One** shared "decomposition children" definition (work-typed reverse-`part_of`) exists in `odm-core` and is called by **both** `check_decomposition` and `node decomposed` | grep: both call the shared helper; unit test asserts identical sets for a mixed-child-type parent | serious | slice-doc (a) | done | attested — `odm_core::recompose::decomposition_children` (new, `crates/odm-core/src/recompose.rs`), called by `check_decomposition` (unchanged behavior, refactored) and by `commands::decomposed`'s no-`--children` branch (`crates/odm-cli/src/commands.rs`, previously an unfiltered `store.load_all()` scan). Unit test: `crates/odm-core/tests/recompose.rs::decomposition_children_is_the_single_work_typed_definition` (mixed slice+artifact+note parent → only the slices). `cargo test -p odm-core --test recompose`: 17/17 pass. | the divergence root — closed by construction, not by keeping two filters in sync |
| F-2 | MF `#58837400` reports **0** decomposition-drift after `odm node decomposed 58837400` | real-store (or fixture) `check`: no `DecompositionDrift` for MF | serious | 2026-08-02 bug | done | reproduced — real store, not just fixture. Before (built binary, F-1 fix in place, no affirm yet): `odm check` on `.worktrees/odm` shows `[error] #58837400 … [decomposition-drift] children changed since decomposed was affirmed (added 0, removed 2)` — reproduces the bug exactly as diagnosed. After `odm node decomposed 58837400` (16 children affirmed — the work-typed current count): `odm check` shows **no** `decomposition-drift` finding anywhere, for MF or any other node. | **Live-store flag:** this ran the real `node decomposed 58837400` against `.worktrees/odm`, left **uncommitted** in that worktree (not committed to the `odm` branch) — the operator's call whether to commit it via `odm store commit` (arc-store-lifecycle s01) or leave it for the next `migrate --all`, which would re-derive the same 16-child set. |
| F-3 | A parent with a non-work (artifact/note) child affirms cleanly — the artifact is neither required in the affirmation nor flagged as drift | fixture unit test | correctness | slice-doc (a) | done | attested — `crates/odm-cli/tests/cli.rs::decomposed_with_no_children_affirms_only_work_typed_children` (a slice + artifact + note under one arc; `node decomposed` with no `--children` affirms only the slice; `check` clean). Reinforced at the `odm-migrate` layer: `decompose::tests::a_non_work_child_added_alongside_never_triggers_a_report`. Pre-existing odm-core coverage (`decomposed_drift_guard_ignores_document_family_children`) continues to pass, now exercising the shared helper. | |
| F-4 | `migrate` auto-recomposes an affirmed parent whose children were **re-minted to new ids but are the same logical set** — no manual re-affirm, no residual drift | re-mint fixture; post-`migrate` `check` clean; no manual step | serious | slice-doc (b) | done | attested — `odm_migrate::decompose::auto_recompose` (new module, `crates/odm-migrate/src/decompose.rs`), wired into `migrate --all` (`crates/odm-cli/src/migrate.rs::all`, after every mint/reconcile step). Fixtures: `decompose::tests::identity_remint_auto_reaffirms_with_the_new_ids` (synthetic `id_remap`; re-affirms with the new id, no manual step) + `::dry_run_reports_the_reaffirmation_but_writes_nothing`. | **Flag (see closing-report):** investigated whether `self_host`/`reconcile_source` (the current migrate pipeline) ever produce a real old→new id correlation for a work node — they don't (`self_host` matches-by-source-or-coordinate and updates in place; `mapping::Reconciled`'s own doc comment: "the node's identity (unchanged — a reconcile never re-mints)"). So `all()` wires `auto_recompose` with an **empty** `id_remap` today — the mechanism is correct and fixture-proven, but this branch is presently a no-op safety net on real data, not something today's `migrate --all` triggers. |
| F-5 | **The seam holds:** a genuinely-new work child under an affirmed parent **still** produces a `DecompositionDrift` finding (migrate does NOT auto-bless completeness) | add-child fixture; drift finding present after `migrate` | serious | slice-doc (out) | done | attested + reproduced end-to-end — unit: `decompose::tests::genuinely_new_child_is_left_as_drift_not_auto_affirmed` + `::a_removal_with_no_remap_entry_is_left_as_drift`. **Real CLI pipeline**, not just the unit function: `crates/odm-cli/tests/migrate.rs::migrate_all_leaves_a_genuinely_new_slice_as_drift_not_auto_affirmed` — seeds a plan set, self-hosts + affirms an arc, adds a genuinely new slice directory, runs `migrate --all` for real, and asserts the RECOMPOSE table reports it as drift (`Total: 0 re-affirmed, 1 left as drift`) and `check` still shows `decomposition-drift` — proving the seam holds through the actual wiring, not only the pure function. | protects spec-keeping |
| F-6 | No regression: `make check` / the test suite is green; existing decomposition/recompose tests pass | `make check` exit 0 | correctness | standing | done | attested — `make check` (build + `clippy --workspace --all-targets -- -D warnings` + full test suite) exit 0; `cargo fmt --check` clean; `grep -rn unsafe` on every new/changed file empty. | |

## What Worked

- **The doc comments already had the answer.** `mapping.rs`'s `Reconciled` struct comment ("the node's identity (unchanged — a reconcile never re-mints)") and `selfhost.rs`'s module doc (never-delete, source-keyed match-or-create) settled, in about five minutes of reading, a question that could otherwise have taken a much longer investigation: does today's `migrate` ever actually re-mint a work node? No — which meant F-4's real wiring is honestly an empty-map no-op today, and F-4's fixture had to prove the *mechanism*, not exercise a real code path. Named clearly rather than silently building dead-looking code with no context.
- **A real end-to-end CLI test caught a wrong assertion, not a code bug.** `migrate_all_leaves_a_genuinely_new_slice_as_drift_not_auto_affirmed`'s first version asserted `!out.contains("re-affirmed")`, which failed against the correct output `"Total: 0 re-affirmed, 1 left as drift"` — a substring false-positive in the test itself, not the feature. Running the real CLI pipeline end-to-end (not just the unit-tested `auto_recompose` function) is what surfaced it immediately, the same payoff the previous two slices' real-store/real-binary checks had.
- **`decomposition_children` closing F-1 "by construction"** — both call sites now literally invoke the same function, so a future third caller can't reopen the same divergence by accident the way `commands::decomposed` did.

## Closure

Closed 2026-08-02. Verified by: CC (this session) — attested for F-1/F-3/F-4/F-6, attested + reproduced for
F-2 (real store, before/after) and F-5 (real CLI pipeline, not just the unit function). CDC reproduction is
the open item. Rows: 6. Done: 6. Deferred: 0. No-op: 0.
