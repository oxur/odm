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
