---
id: 01KZ1YCPAK0P27G53YS64SDFW9
number: 577184700
type: artifact
schema: artifact/v1.1
name: Slice 06 — CDC verification
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice06-auto-extend-decomposition/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK41NQNY11R2RF7831FT
---
# Slice 06 — CDC verification

**Method:** LEDGER-DISCIPLINE v2.0 §A. **Verdict: PASS.** Structural rows reproduced
by direct code read; F-2's real-MF leg (the acceptance anchor) is
deferred-with-re-entry to the operator's next real `migrate --all` (no macOS binary
in the CDC sandbox). Code committed (`7cd5e01`, atop `209b1a4`). Verified 2026-08-02.

## Verdict

**PASS.** Auto-extend is correct; the seam is **refined, not removed** (authored
additions auto-extend, removals and never-affirmed parents stay conservative); and
the sharp edge CC flagged is genuinely handled. One row (F-2 real leg) awaits the
operator's next migrate — it is the arc's acceptance anchor and must not be skipped.

## Ledger walk (F-1…F-7)

- **F-1 — reproduced (code).** `Outcome::AutoExtended { added, dropped_stale }`;
  `auto_recompose`: `affirmed_raw == current` → skip; else filter the affirmation to
  work-children, and additions-only (`removed` empty) → `AutoExtended`, rewriting the
  affirmation to `current` and dropping stale non-work ids. Unit
  `authored_addition_auto_extends_an_affirmed_parent` (decompose.rs:317) + integration
  `migrate_all_auto_extends_an_affirmed_parent_for_an_authored_addition`.
- **F-2 — structural reproduced; REAL LEG DEFERRED.** For MF at `6225d1f`:
  `affirmed_work` = 15 slices (the 2 artifacts drop as stale non-work), `current` = 16
  slices, `removed` empty → `AutoExtended { added: [slice16], dropped_stale: [2
  artifacts] }` → affirmation rewritten to the 16 work slices → **0 drift, no manual
  step.** Structurally certain. **Real confirmation = the operator's next `reset →
  migrate --all`.** Re-entry: MF shows **0 decomposition-drift with no manual `node
  decomposed`**. Acceptance anchor — do not skip.
- **F-3 — reproduced (the sharp edge).** `affirmed_raw.filter(|id|
  types.get(id).is_none_or(|ty| ty.is_work()))` **keeps** an affirmed id *absent from
  the corpus* (a vanished work-child) so it falls into `removed` → not additions-only
  → `provably_same_set` (returns `None` for an unremapped removal) → `LeftAsDrift`;
  and **drops** a *present non-work* id as stale. A genuine removal is therefore never
  swallowed. Integration `migrate_all_leaves_a_removed_work_child_as_drift_not_auto_healed`
  detaches a slice via `node unlink … part_of …` and asserts `drift` + that `check`
  still reports `decomposition-drift`. Strong, non-weakened.
- **F-4 — reproduced.** `let Some(decomp) = fm.decomposed() else { continue }` — a
  never-affirmed parent is skipped entirely. Unit `a_never_affirmed_parent_is_untouched`
  (decompose.rs:434).
- **F-5 — reproduced.** After an `AutoExtended` rewrite to `current`, a second run hits
  `affirmed_raw == current → continue`. Units `a_second_run_after_auto_extend_reports_nothing_new`
  (456) + `migrate_all_is_idempotent`.
- **F-6 — reproduced.** migrate.rs renders `re-affirmed` / `auto-extended` / `left as
  drift` distinctly (plus `would …` dry-run variants) and a `Total: N re-affirmed, M
  auto-extended, K left as drift` summary.
- **F-7 — reproduced.** Slice 05's seam test was **renamed and re-asserted** to the
  auto-extend behavior — a legitimate intentional flip, documented in-test as a
  genuinely authored (not re-minted) addition — and a removal companion (F-3) added.
  The seam is refined, not deleted. Bonus edge covered:
  `a_non_work_child_added_alongside_never_triggers_a_report` (501).

**Rows: 7. Reproduced (structural): 6. Deferred-with-re-entry: 1 (F-2 real leg).** No
silent drops. `make check` green (CC-attested) + code committed → CI reproduces on push.

## Finding

### SS6-1 — additions-only blesses *any* addition in the migrate context (accepted property; ODD-0026 note)

The auto-extend fires on any additions-only change, not a per-child
plan-declaration check — correct and model-independent as specified, and valid
because the pass runs inside `migrate --all`, where any present child came through
the plan tree. Residual edge: a work child manually `node new`'d + linked (no plan
doc) would be auto-blessed on the next migrate. This is the documented tradeoff of
the additions-only signal (a `source.paths`-based per-child check would break under
store-as-source). Fold into ODD-0026's "authoring declares scope"; no action here.
**Severity: design-note / non-blocking.**

## Arc-plan disposition

Flip slice 06 → **CDC-verified PASS** (structural), with **F-2's real leg carried to
the operator's next `migrate --all`**. SS-9 is satisfied structurally and closes fully
when the real re-migrate shows MF at 0 drift, hands-off.
