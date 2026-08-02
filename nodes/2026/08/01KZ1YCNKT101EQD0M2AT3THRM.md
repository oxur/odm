---
id: 01KZ1YCNKT101EQD0M2AT3THRM
number: 556932700
type: artifact
schema: artifact/v1.1
name: 'Slice 05 cc-prompt — decomposition bookkeeping: consistency + auto-recompose'
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice05-decomposition-bookkeeping/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK41S3RNWSHHCWMVMTRH
---
# Slice 05 cc-prompt — decomposition bookkeeping: consistency + auto-recompose

**You are CC.** Implement this against the real toolchain, with tests. This slice
**runs first** in the store-as-source arc and will be exercised in an imminent
`odm migrate --all` cycle, so correctness on the MF case is the acceptance anchor.
Read `slice-doc.md` and `ledger.md` (F-1…F-6) first; work the ledger, not around it.

## The problem (proven, don't re-derive — but do confirm with a fixture)

`node decomposed` and `check` disagree on what a parent's "decomposition children"
are:

- `crates/odm-core/src/recompose.rs::check_decomposition` filters current children to
  **work-typed**:
  ```rust
  let kids: Vec<Id> = recomp.children(fm.id()).iter().copied()
      .filter(|id| types.get(id).is_some_and(|ty| ty.is_work()))
      .collect();
  ```
- `crates/odm-cli/src/commands.rs::decomposed` (no `--children` branch) affirms
  **all** reverse-`part_of` children, unfiltered.

On the live store this makes MF `#58837400` report a permanent `DecompositionDrift {
added: 0, removed: 2 }` — the 2 removed are its non-work children (`536513400`
provenance/synthesis, `560811200` closing-report): affirmed 18, check's current 16.

## What to build

**(a) One shared child-set definition.** Extract a single helper in `odm-core`
(e.g. `decomposition_children(recomp, types, parent) -> Vec<Id>`, or an equivalent
method) returning the **work-typed** reverse-`part_of` children. Call it from **both**
`check_decomposition` and the `node decomposed` command (build the `Recomposition` +
type map from `store.load_all()` in the CLI path). The work-typed set is canonical:
decomposition affirms that scope-bearing children account for scope; artifacts/notes
are outputs and neither belong in the affirmation nor count as drift. Add a unit test
asserting both paths return the identical set for a parent with mixed child types.

**(b) Deterministic auto-recompose in `migrate`.** After the mint/reconcile pass,
for each parent-capable node with an affirmed decomposition, recompute its
work-children and:
- **equal to affirmed** → no-op;
- **differs only by identity re-mint** — each "removed" affirmed child maps, via the
  old→new id mapping migrate itself performed this run, to an "added" current child
  (same logical node, new id) → **auto-re-affirm** with the new ids (deterministic;
  migrate proved the set is the same);
- **genuine membership change** — a removed child with no re-mint mapping, or an
  added child migrate did not re-mint from a prior node → **leave it as a drift
  finding.** Do not touch the affirmation.

If migrate cannot cheaply prove "same logical set" for a churn, the safe default is
to **leave it as drift** (never auto-affirm on uncertainty).

## The seam — do not cross it

`decomposed: complete` is a human completeness judgment. Migrate may auto-maintain
the *mechanical* child-set (identity re-mints) but must **never** auto-assert
completeness on a genuinely-new child. A new work child under an affirmed parent must
still raise `DecompositionDrift` (F-5). This is odm's core value — silencing it would
reintroduce the scope-drift blindness odm exists to kill.

## Constraints

- No store-as-source model change (that's slice 01 / ODD-0026). No renumbering.
- Keep the diff to the decomposition/migrate paths + tests.
- Deterministic output; no new per-actor metrics.

## Definition of done

F-1…F-6 all reach a final status; MF reports 0 decomposition-drift after affirm;
`make check` green. Close with the per-row ledger walk and a **bubble-up to the arc**
(did this slice deliver the bookkeeping fix; what did implementing it reveal that the
arc-plan didn't anticipate — e.g. whether migrate actually re-mints in practice, which
bears on how often (b) fires; the silent-drop diff).
