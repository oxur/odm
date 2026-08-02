# Closing Report — Slice 05 (Store-as-Source): decomposition bookkeeping

> Verified by: CC (this session). F-1/F-3/F-4/F-6 attested; F-2 attested + reproduced (real store,
> before/after); F-5 attested + reproduced (real CLI pipeline via `migrate --all`, not just the unit
> function). CDC reproduction is the open item. Closed 2026-08-02 on `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative.

**F-1 (the consistency fix).** The bug was pre-diagnosed in the cc-prompt and confirmed on read:
`recompose.rs::check_decomposition` already filtered a parent's current children to `is_work()`;
`commands.rs::decomposed` (no `--children`) did not. Extracted the filter into
`odm_core::recompose::decomposition_children(recomp, types, parent) -> Vec<Id>`, called from both —
`check_decomposition` now delegates to it (no behavior change, confirmed by the existing 16 recompose
tests passing unmodified), and `commands::decomposed`'s no-`--children` branch now builds a
`Recomposition` + type map from `store.load_all()` and calls the same function instead of an unfiltered
`part_of`-equality scan. One new unit test (`decomposition_children_is_the_single_work_typed_definition`)
proves the shared definition directly, with a parent carrying two slices, an artifact, and a note.

**F-2 (the acceptance anchor) reproduced on the real store, not just asserted.** Before touching the live
corpus, `odm check` on `.worktrees/odm` was run to confirm the bug reproduces exactly as diagnosed:
`#58837400` showed `[decomposition-drift] children changed since decomposed was affirmed (added 0, removed
2)`. Running `odm node decomposed 58837400` against the real store (16 children affirmed — the correct
work-typed count, matching the slice-doc's own arithmetic) and re-running `odm check` showed **zero**
`decomposition-drift` findings anywhere in the corpus. This is a genuine before/after reproduction of the
literal bug the freeze surfaced, not a fixture standing in for it.

**F-3.** Covered three ways: a CLI-level fixture
(`decomposed_with_no_children_affirms_only_work_typed_children`) mirroring the exact MF shape (a slice, an
artifact, and a note under one arc, affirmed with no `--children`); an `odm-migrate`-layer fixture
(`a_non_work_child_added_alongside_never_triggers_a_report`) proving the auto-recompose pass agrees; and the
pre-existing odm-core fixture (`decomposed_drift_guard_ignores_document_family_children`) continuing to pass
unmodified, now routed through the shared helper instead of its own inline filter.

**F-4/F-5 (auto-recompose) — built as specified, with one finding that reshapes what "wired in" means
today.** `odm_migrate::decompose::auto_recompose` is a new, pure-ish function (reads the store, writes only
the parents it re-affirms) taking an explicit `id_remap: &HashMap<Id, Id>`. For each affirmed parent whose
current work-children differ from its affirmation: if every removed id maps via `id_remap` to a distinct
added id (and every added id is accounted for that way), it re-affirms with the new set; otherwise it leaves
the affirmation untouched and reports the parent as drift. The "otherwise" branch is deliberately the safe
default — a partial mapping, a collision (two removals mapping to the same added id), or no mapping at all
all fall through to "leave it as drift," per the slice-doc's explicit instruction.

Wired into `migrate --all` (`crates/odm-cli/src/migrate.rs::all`), called once after every mint/reconcile
step, rendered as a new `RECOMPOSE` table.

**The finding:** before wiring it, I checked whether `migrate`'s current pipeline (`self_host`,
`reconcile_source`, `mapping::backfill_source`) ever actually produces an old→new id correlation for a
*work* node — the premise `id_remap` needs data for. It doesn't, by design: `selfhost.rs`'s module doc says
plainly ("never-delete... matched-or-created, updated in place"), and `mapping.rs`'s `Reconciled` struct
carries the comment "the node's identity (unchanged — a reconcile never re-mints)" on its `id` field. Under
the source-keyed idempotence model (arc-migration-fidelity s05), re-minting a duplicate for an
already-matched node would be a *bug* that self-hosting's own matching machinery exists to prevent, not a
normal transformation. So `all()` wires `auto_recompose` with an **empty** `HashMap::new()` — correct,
because there is genuinely nothing to map today, and the function's own logic (empty `removed` set never
needs mapping; a non-empty `removed` set with no entries never proves "same set") degrades to exactly the
right behavior: every real membership change is left as drift, F-5's seam intact.

This means F-4's "auto-re-affirm" branch is, on today's real corpus, dead code in the sense that nothing in
the current pipeline triggers it — but it is not speculative or unreachable: it is unit-tested directly with
a synthetic `id_remap` (`identity_remint_auto_reaffirms_with_the_new_ids`,
`dry_run_reports_the_reaffirmation_but_writes_nothing`) proving the mechanism is correct and ready for the
day migrate (or a future native-authoring rename/re-mint operation, once arc-store-as-source's slice 01–03
land) actually produces such a mapping. Flagged per the working agreement rather than silently building
inert-looking wiring with no explanation.

**F-5 reproduced through the real pipeline, not only the pure function.** Added
`migrate_all_leaves_a_genuinely_new_slice_as_drift_not_auto_affirmed` to `odm-cli`'s integration suite: seeds
a plan set, runs `migrate --all` (self-hosts the arc with zero slices), affirms the arc's decomposition, adds
a genuinely new slice directory, and runs `migrate --all` again for real. The RECOMPOSE table correctly
reports `Total: 0 re-affirmed, 1 left as drift`, and `check` still shows `decomposition-drift` for the arc —
proving the seam holds through the actual CLI wiring, not just the unit-tested `auto_recompose` call.

## What was deliberately *not* done on the live store

The cc-prompt's exit criteria say "re-run `migrate --all` (the imminent cycle) and confirm the MF error is
gone and no false auto-affirmation occurred." I did not run a full `migrate --all` against `.worktrees/odm`
in this session. Two reasons: first, F-2's specific acceptance criterion (0 decomposition-drift after the
targeted affirm) is already reproduced without it, and F-4/F-5's mechanism is proven by fixture and by a
real end-to-end CLI run in an isolated `TempDir`. Second, `odm check` on the live store currently reports
several `uncovered-doc` errors for this session's own new arc-store-as-source planning docs (committed
earlier this session, not yet migrated/minted) — a real `migrate --all` would mint artifact nodes for them
as a side effect, which is correct behavior but a broader live-store mutation than this slice's own scope
needs to demonstrate its acceptance criteria. Left for the operator's "imminent cycle" to do deliberately,
with full awareness of what it will mint, rather than folded in here as a side effect.

**Live-store state disclosed:** `odm node decomposed 58837400` was run for real against `.worktrees/odm`
(F-2's reproduction) and is **uncommitted** in that worktree — not committed to the `odm` branch. The
operator can commit it with `odm store commit` (arc-store-lifecycle s01) or let the next `migrate --all`
re-derive the identical 16-child affirmation; either is safe, since the value is deterministic from the
current corpus.

## Scope discipline

Diff: `odm-core/src/recompose.rs` (the shared helper + refactor), `odm-cli/src/commands.rs` (the
`decomposed` command's child-set resolution), `odm-migrate/src/decompose.rs` (new), `odm-migrate/src/lib.rs`
(module registration), `odm-cli/src/migrate.rs` (the `all()` wiring + `render_recompose` + a small `today()`
helper), plus tests in all four crates. No store-as-source model change (ODD-0026 untouched — that's slice
01). No renumbering. `next` and every non-decomposition/migrate surface untouched.

## Iterations

One pass for the core fix (F-1/F-2/F-3) and the auto-recompose mechanism (F-4/F-5); one self-caught test
assertion fix (a false-positive substring match in the new end-to-end test, not a code defect) during the
same pass. Well inside the five-iteration cap.

## v2.0 bubble-up → `../arc-plan.md`

- **Slice 05 done**, delivering arc ledger row **SS-8** in full: (a) one shared work-typed child-set
  definition used by both `check` and `node decomposed`; (b) `migrate --all` auto-recomposes a provably
  identical child-set churn, with the completeness-judgment seam (F-5) intact and proven through the real
  CLI pipeline.
- **What implementing it revealed that the arc-plan didn't anticipate** (the cc-prompt's own invited
  question): **migrate does not currently re-mint work nodes in practice.** The source-keyed idempotence
  model (self_host matches-or-creates, `reconcile_source` explicitly "never re-mints") means F-4's
  auto-re-affirm branch is real, tested, and correctly wired — but inert on today's corpus; every
  child-set change `migrate --all` can currently produce is either "up to date" or "genuine," never
  "identity re-mint." This bears directly on **slice 01 (ODD-0026)** and the arc's own framing ("migrate
  re-mints the child set" is named in the arc-plan's "why this arc exists" section as one of the two
  decomposition-drift symptoms) — worth checking against ODD-0026's design forks whether *native authoring*
  (slice 03) introduces a real rename/re-mint operation that would finally exercise this branch, or whether
  the auto-recompose mechanism's practical value turns out to be "ready for a future that may not arrive via
  this path."
- **Silent-drop check:** all 6 ledger rows closed; none deferred; the one scope decision (not running a live
  `migrate --all`) is disclosed above with its reasoning, not a silent omission.
