---
id: 01KZ1YCK41NQNY11R2RF7831FT
number: 61401506
type: slice
schema: slice/v1.1
name: Slice 06 — auto-extend affirmed decomposition on authored additions
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice06-auto-extend-decomposition/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK41SK7J1H134AK7WB7T
status:
  built:
    reached: 2026-08-02
    evidence: asserted
    evidence_dates:
      asserted: 2026-08-02
  planned:
    reached: 2026-08-02
    evidence: asserted
    evidence_dates:
      asserted: 2026-08-02
  tested:
    reached: 2026-08-02
    evidence: asserted
    evidence_dates:
      asserted: 2026-08-02
---
# Slice 06 — auto-extend affirmed decomposition on authored additions

<!-- Name/title carries no document-role metadata, per ODD-0013 §2.1. -->

**Arc:** Store-as-source-of-truth & native authoring · **Kind:** code · **Runs
next** (before the store is re-migrated + committed) · **Opened:** 2026-08-02 ·
**Origin:** SS5-1 (slice 05 bubble-up) + operator decision

## Goal

Make `migrate --all` **hands-off for the normal author→migrate flow.** An
already-affirmed parent that gains a slice the plan tree declares has its
decomposition **auto-extended** — no manual `odm node decomposed`. This resolves
**SS5-1**: slice 05's seam correctly refused to *invent* a completeness judgment,
but the operator's real friction is exactly the genuine-addition case, and in
odm's model that addition is not unplanned — *the plan tree is the scope
declaration.* Authoring a slice under an arc **is** the membership decision; a
second `node decomposed` to re-confirm it is the micromanagement odm exists to
kill.

## The principle (to be formalized in ODD-0026)

The plan tree declares scope. An **affirmed** parent therefore *tracks its authored
children automatically*; the decomposition-drift signal is repurposed from "you
forgot to re-affirm" to "something genuinely anomalous happened" — a child
**removed**, or a child appearing that the plan tree did **not** declare. Slice 01
records this in ODD-0026; slice 06 implements it with a model-independent signal so
it can land first.

## Scope — in

Extend `odm_migrate::decompose::auto_recompose` (slice 05) with a new
`AutoExtended` outcome. Because the pass runs at the **end of `migrate --all`** —
which has just reconciled the store to the plan tree — the current work-children of
any parent *are* the plan-tree-declared set. So, for a parent that **already has**
an affirmed decomposition, compare its **affirmed work-children** to its **current
work-children** (both via `recompose::decomposition_children`, reusing slice 05's
one definition):

- **current ⊇ affirmed (additions only, no work-child removed)** → **auto-extend**:
  rewrite the affirmation to the current work-child set. This also drops any stale
  **non-work** entries a pre-fix affirmation still carries — so it **closes the MF
  transitional case** (the old 17-with-2-artifacts affirmation becomes the clean 16
  work slices in one automatic step, 0 drift, no manual affirm).
- **identity re-mint** (slice 05's `id_remap` case) → `ReAffirmed`, unchanged.
- **a work-child was removed** → `LeftAsDrift` — a real anomaly (retire / supersede
  / reparent / deleted doc) worth a human look.

Update the `RECOMPOSE` report + CLI output to distinguish **auto-extended** from
**left as drift**.

## Scope — out

- **Never auto-affirm a never-decomposed parent.** The *first* affirmation is a
  human act — migrate only **maintains an existing** affirmation. A parent with no
  `decomposed` stays an `undecomposed-parent` warning (its own honest state).
- **Never auto-heal a work-child removal.** Additions are trusted (the plan tree
  just declared them); removals are surfaced.
- No store-as-source model change (ODD-0026 is slice 01). Use the model-independent
  additions-only signal, not a `source.paths` check — planning nodes go
  self-sourced under ODD-0026, so a source-based signal would be fragile; revisit
  at slice 02 only if needed.
- This **refines, not removes,** slice 05's seam: a genuine addition the plan tree
  did *not* declare (a child with no plan provenance) still leaves as drift.

## Verification approach

Fixtures + the real MF case. Fixtures: (1) affirm a parent with N slices, author
slice N+1's doc, `migrate --all` → the parent's `decomposed` now includes N+1 and
`check` is clean, no manual step; (2) the MF transitional shape (affirmation with
stale non-work children + a new slice) auto-heals to the work-child set; (3) a
removed work-child still raises `DecompositionDrift`; (4) a never-affirmed parent is
**not** auto-affirmed; (5) idempotent — a second `migrate --all` extends nothing.
Real: after this lands, reset the store, `migrate --all`, and MF shows **0 drift
with no manual `node decomposed`.**

## Exit criteria

`migrate --all` needs no manual affirm for authored additions; removals and
never-affirmed parents still behave conservatively; `make check` green; and the real
re-migrate leaves MF (and the other affirmed parents) clean with no manual step — so
the store can be committed straight from a single `migrate --all`.
