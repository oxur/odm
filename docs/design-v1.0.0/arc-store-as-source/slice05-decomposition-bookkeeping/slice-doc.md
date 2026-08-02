# Slice 05 — decomposition bookkeeping: consistency fix + deterministic auto-recompose

<!-- Name/title carries no document-role metadata, per ODD-0013 §2.1. -->

**Arc:** Store-as-source-of-truth & native authoring · **Kind:** code · **Runs
FIRST** (operator priority 2026-08-02 — independent of the ODD-0026 model, and to
be exercised in the imminent `migrate --all` cycle) · **Opened:** 2026-08-02

## Goal

Make decomposition bookkeeping deterministic and self-maintaining, so the tool
stops nagging the user with drift it caused and can prove away. Two coupled fixes:

- **(a) Consistency.** `node decomposed` and `check` must use **one** definition of
  "a parent's decomposition children," so an affirmed parent whose child set hasn't
  meaningfully changed reports **zero** drift.
- **(b) Auto-recompose.** When `migrate` deterministically churns a parent's
  children and can *prove the resulting set is the same as the affirmed set*
  (identity re-mint, no membership change), it re-affirms automatically instead of
  leaving a manual `odm node decomposed` chore.

## The bug, proven (fix a)

Diagnosed by direct read of the 2026-08-02 store + the code:

- `crates/odm-core/src/recompose.rs::check_decomposition` builds the current child
  set and **filters `is_work()`**:
  ```rust
  let kids: Vec<Id> = recomp.children(fm.id()).iter().copied()
      .filter(|id| types.get(id).is_some_and(|ty| ty.is_work()))
      .collect();
  ```
  → for MF `#58837400`, current = **16 slices**.
- `crates/odm-cli/src/commands.rs::decomposed`, with no `--children`, affirms **all**
  reverse-`part_of` children with **no `is_work` filter**:
  ```rust
  store.load_all()?.iter()
      .filter(|d| d.frontmatter().edges().part_of == Some(id))
      .map(|d| d.frontmatter().id()).collect()
  ```
  → it affirmed **18** (16 slices + the 2 non-slice children `536513400`
  provenance/synthesis and `560811200` closing-report).

So `check`'s drift = `affirmed(18) − current(16) = 2 artifacts = "removed 2"`,
permanently, surviving any re-affirm. This matches the observed run exactly
(before affirm: added 1/removed 2; after affirming 18: added 0/**removed 2**).

## Scope — in

- **(a)** Extract **one shared helper** in `odm-core` — e.g. `decomposition_children(recomp, types, parent) -> Vec<Id>` returning the **work-typed** reverse-`part_of` children — and call it from **both** `check_decomposition` and `node decomposed`. The work-typed definition is canonical (decomposition is about scope-bearing children; artifacts/notes are outputs). A unit test must assert the two paths return the *identical* set for a parent with mixed child types.
- **(b)** In `migrate` (the `odm-migrate` crate / `migrate` command path): after the mint/reconcile pass, for each parent-capable node that **has an affirmed decomposition**, recompute its work-children and:
  - **equal to affirmed** → nothing to do (no drift);
  - **differs only by identity re-mint** — every "removed" affirmed child maps, via the old→new id mapping *migrate itself performed*, to an "added" current child (same logical node, new id) → **auto-re-affirm** with the new ids;
  - **genuine membership change** (a removed child with no re-mint mapping, or an added child migrate did not re-mint from a prior one) → **leave it as a drift finding** for a human affirm.

## Scope — out

- **Do NOT auto-assert completeness on a genuinely-new child.** A new work child
  under an affirmed parent legitimately drifts and needs a human `decomposed` — that
  is spec-keeping (odm's whole reason to exist), not noise. The mechanical child-set
  half auto-heals; the completeness-judgment half does not. Preserve this seam.
- No store-as-source model change (ODD-0026 is slice 01). No renumbering. Keep the
  diff to the decomposition/migrate paths + their tests.

## Verification approach

Fixture + real-store. The MF case is the acceptance anchor: after the fix, running
`odm node decomposed 58837400` (or the auto-recompose) leaves `odm check` with **no
DecompositionDrift** for MF. Unit tests cover the shared-helper equality, the
artifact-child case, the re-mint auto-heal, and — critically — the genuine-new-child
case still surfacing drift (the seam). `make check` green.

## Exit criteria

One shared work-child definition used by both paths; MF drift clears; migrate
auto-recomposes provably-identity churn with no manual step; the new-child seam is
preserved and tested; CI green. Then re-run `migrate --all` (the imminent cycle) and
confirm the MF error is gone and no false auto-affirmation occurred.
