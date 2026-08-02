# Closing Report — Slice 06 (Store-as-Source): auto-extend affirmed decomposition

> Verified by: CC (this session). F-1/F-3/F-4/F-5/F-6/F-7 attested (module-level fixtures + real
> end-to-end `odm-cli` integration tests). F-2's fixture half attested; its real-store half deferred to
> CDC (no access to `.worktrees/odm` from this worktree). Closed 2026-08-02 on `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative.

**F-1/F-2 (the core mechanism).** Added `Outcome::AutoExtended { added, dropped_stale }` alongside slice
05's `ReAffirmed`/`LeftAsDrift`. The algorithm reuses slice 05's own `removed`/`added` diff, with one new
step ahead of it: filter the *affirmed* side to work-children before comparing (`affirmed_work`), using
the same `is_work()` test `decomposition_children` already applies to the *current* side. If nothing
affirmed is missing from the current set (`removed.is_empty()`), the churn is additions-only — auto-extend,
rewriting `decomposed.children` to the full current work-child set. This is also exactly what heals the MF
transitional shape: a pre-slice-05 affirmation that still names an artifact id has that id fall out of
`affirmed_work` (present in the corpus, not work-typed), so the rewrite drops it in the same step that
folds in the genuinely new slice — one automatic pass, not two. Proven by
`authored_addition_auto_extends_an_affirmed_parent` (clean addition) and
`mf_transitional_shape_drops_stale_non_work_and_extends` (stale-drop + addition together), both at the
`odm_migrate::decompose` unit level, and by
`migrate_all_auto_extends_an_affirmed_parent_for_an_authored_addition` through the real CLI pipeline
(`migrate --all` on a temp store, RECOMPOSE table asserted, `check` asserted clean after).

**The one place the naive version of this would have been wrong.** My first pass at "filter the affirmed
set to work-children" used the same one-line test `decomposition_children` uses for filtering *current*
children: `types.get(id).is_some_and(|ty| ty.is_work())`. That is correct for a child that still exists in
the corpus, but for an id that no longer exists at all — a genuinely deleted/retired work-child — the
lookup also returns "not work" (`None` fails `is_some_and`), which would have silently classified a real
removal as a stale non-work id to drop. `a_removal_with_no_remap_entry_is_left_as_drift` (a slice-05
fixture, re-run against the new code before trusting it) caught this immediately: the fixture removes a
work-child's file entirely from the store, and the naive filter would have folded that removal into
`AutoExtended`'s "drop the stale ids" path instead of leaving it as drift. The fix is
`types.get(id).is_none_or(|ty| ty.is_work())` — keep an id in `affirmed_work` unless it is *known* (still
present in the corpus) to be non-work; an absent id is kept, so its absence still shows up as a genuine
`removed` entry. Documented in the module doc comment and the inline comment at the filter site so a future
reader doesn't reintroduce the same one-line-looks-right trap.

**F-3.** `a_removal_with_no_remap_entry_is_left_as_drift` (module level, per above) plus a new real-pipeline
test, `migrate_all_leaves_a_removed_work_child_as_drift_not_auto_healed`. Constructing a genuine removal at
the CLI level needed one extra step beyond what I expected: `migrate` never deletes nodes
(`selfhost.rs`'s documented never-delete invariant, same fact slice 05's closing report already surfaced),
so deleting a slice's plan-tree markdown doc does **not** remove its minted node from the store — the node
just stops being touched. The test instead uses `odm node unlink <slice> part_of <arc>` to detach the
containment edge, which is the actual mechanism a real retire/reparent operation would use to take a
work-child out of a parent's current set. `migrate --all` then reports `Total: 0 re-affirmed, 0
auto-extended, 1 left as drift`, and `check` still shows `decomposition-drift`.

**F-4.** `a_never_affirmed_parent_is_untouched` — a parent-capable node with a real work-child but no
`affirm_decomposed` call is skipped by `auto_recompose`'s `let Some(decomp) = fm.decomposed() else {
continue }` guard, unchanged from slice 05. No new logic needed; the fixture makes the invariant explicit
for slice 06's own ledger rather than relying on slice 05's coverage by inference.

**F-5.** `a_second_run_after_auto_extend_reports_nothing_new` — `auto_recompose` called twice in a row on
the same store; after the first call rewrites the affirmation to the current work-child set, the early
`affirmed_raw == current` check (unchanged from slice 05) short-circuits the second call to `changed.is_empty()`.

**F-6.** `render_recompose` in `odm-cli/src/migrate.rs` now matches `Outcome::AutoExtended` to
`"auto-extended"` (`"would auto-extend"` under `--dry-run`), and the summary line reports all three counts
(`re-affirmed`/`auto-extended`/`left as drift`). One caveat disclosed in the ledger: `migrate --all` has no
`--json` output path at all today (checked directly — no `Serialize`/json handling anywhere in
`migrate.rs`), so the criterion's `--json` half doesn't apply to the current surface. Not a gap this slice
needs to close: the `Outcome` enum already carries the distinguishing data, so wiring `--json` later (if
`migrate --all` ever gets one) is mechanical.

**F-7.** All of slice 05's own tests (`identity_remint_auto_reaffirms_with_the_new_ids`,
`dry_run_reports_the_reaffirmation_but_writes_nothing`, `an_up_to_date_parent_is_not_reported`,
`a_non_work_child_added_alongside_never_triggers_a_report`) pass unmodified against the new code. The one
slice-05 integration test that needed a **behavioral** update, not just a re-run, is discussed below.
Full workspace `make format` + `make lint` + `make test` all green (every crate, every doctest).

## The pre-existing test slice 06 correctly flips, not breaks

`migrate_all_leaves_a_genuinely_new_slice_as_drift_not_auto_affirmed` (added in slice 05) exercises exactly
the scenario slice 06 exists to fix: an already-affirmed arc gains a genuinely new, plan-tree-authored
slice. Under slice 05's contract that was correctly left as drift (no id_remap could explain a brand-new
child); under slice 06's contract, an authored addition picked up by the same `migrate --all` run is by
definition plan-tree-declared, so it should now auto-extend. Running the real CLI pipeline confirmed this
concretely — the old test's assertions failed with the new RECOMPOSE output showing `auto-extended`, not
`drift`, which is the intended behavior change, not a regression. Renamed to
`migrate_all_auto_extends_an_affirmed_parent_for_an_authored_addition` and its assertions flipped to match
the new contract, rather than leaving a stale assertion of pre-s06 behavior sitting in the suite (which
would silently reduce it to "the old bug now passes as a false negative"). This is the update slice 05's
own closing report anticipated is possible when a later slice legitimately changes a preceding slice's
contract — the fix is to update the test's assertions to the new intended behavior with a clear rename, not
to touch slice 05's implementation.

## Scope discipline

Diff: `odm-migrate/src/decompose.rs` (the `AutoExtended` outcome + the affirmed-work-children filter +
seven new/updated unit tests), `odm-cli/src/migrate.rs` (`render_recompose` + the `all()` comment, no
control-flow change), `odm-cli/tests/migrate.rs` (one renamed+flipped integration test, one new integration
test). No touch to `odm_core::recompose` (slice 05's shared `decomposition_children` definition is reused,
not modified — per the cc-prompt's "do not reopen slice 05"). No store-as-source model change (ODD-0026 is
slice 01). No renumbering. `next` and every non-decomposition/migrate surface untouched.

## Iterations

One pass for the `AutoExtended` outcome and algorithm; one self-caught fix (the vanished-vs-non-work
filter trap, caught by re-running slice 05's own removal fixture before trusting the change, not by CDC or
an external report); one pass updating the pre-existing slice-05 integration test to the new intended
contract. Well inside the five-iteration cap.

## v1.4 bubble-up → `../arc-plan.md`

- **Slice 06 done**, delivering the row **SS-9** the arc-plan's v1.4 entry opened for it: `migrate --all`
  is now hands-off for the ordinary author→migrate flow (an already-affirmed parent auto-extends over a
  plan-tree-declared addition), while a genuine work-child removal and a never-affirmed parent still
  behave conservatively.
- **What implementing it revealed that the arc-plan didn't fully anticipate**: the arc-plan's slice 06 row
  states the additions-only signal is valid "because the pass runs inside `migrate --all`" — true, but the
  implementation detail worth recording for **ODD-0026** is *which* half of the comparison needed the new
  filter. It is not enough to compare "current work-children" (already filtered by
  `decomposition_children`) against "affirmed children" (the raw, un-filtered list `decomposed.children`
  stores) — the *affirmed* side needs the identical work-child filter applied before the sets are
  comparable at all, and that filter has a sharp edge: an id that has vanished from the corpus entirely
  must **not** be filtered out the same way a still-present-but-non-work id is, or a genuine removal
  silently reads as "stale cruft, safe to drop." If ODD-0026 formalizes "the plan tree declares scope" as
  a general principle, this specific asymmetry (present-and-non-work vs. absent-entirely) is worth stating
  explicitly, since it is exactly the kind of one-line-looks-right trap a future reimplementation (e.g. if
  slice 02 changes how the affirmed set is stored) could reintroduce.
- **Silent-drop check:** all 7 ledger rows reached a final status (6 done, 1 deferred); the deferred row
  (F-2's real-store leg) has a named reason and a concrete re-entry condition, not a silent gap — see
  `ledger.md`. No rows were dropped or left `open`.
- **For the operator's imminent `migrate --all` cycle**: this slice is what makes that cycle hands-off for
  MF's slice-16 addition specifically (and any other affirmed parent with an authored addition since its
  last affirm) — the real-store confirmation is F-2's deferred leg, and per the arc-plan's own sequencing
  (`05 → 06 → 01 → …`) this is the last piece before that cycle should be run.
