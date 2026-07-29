---
id: 01KWXMBBTKW1C0RPYRW5GV37GK
number: 1205
type: slice
schema: slice/v1.1
name: Decomposition/recomposition integrity
created: 2026-06-22
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc02-graph-gates-derived-order/slice05-recomposition-integrity/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KWXMBBTKTMGY8EJV5TAEY7S0
status:
  built:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  planned:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  tested:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
---
# Slice 05 (Arc 02) — Decomposition/recomposition integrity (plan-of-record)

> Refs: ODD-0013 §4.5 (the spec), ODD-0001 E4 (the failure), ODD-0011 R1 (WBS).
> `depends_on:` arc02 slice01 (the `part_of` tree).

## Goal

Make "can we see the decomposition, and does it recompose?" structural and
checkable. **Done when reverse-`part_of` enumerates a parent's complete child set,
orphans/undeveloped-stubs are detected, and a `decomposed: complete` assertion is
guarded against drift.** (The *automatic* detection of semantically missing scope
is explicitly NOT attempted — that's a human judgement; faking it is confabulation.)

## Scope

**In:** total recomposition (every non-root node resolves to exactly one parent via
`part_of`; reverse enumerates the full child set); no-orphan check; no-stub check
(a `project`/`arc` advanced into a working/complete gate with zero children is
flagged); the `decomposed: complete` assertion + its guard (children added/removed
after assertion, or advanced-without-assertion → flag for re-affirmation).

**Out:** the `check` command aggregation (slice06 — this slice provides the
predicates); semantic missing-scope detection (non-goal).

## Verification

`cargo test -p odm-core` green; clippy `-D warnings`; coverage ≥ 90%. Rows in
`ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). Then slice06
(`check` v2) aggregates these predicates.
