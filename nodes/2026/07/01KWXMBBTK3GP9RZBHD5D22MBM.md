---
id: 01KWXMBBTK3GP9RZBHD5D22MBM
number: 1202
type: slice
schema: slice/v1.1
name: Cycle detection + tears
created: 2026-06-22
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc02-graph-gates-derived-order/slice02-cycles-and-tears/slice-doc.md
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
# Slice 02 (Arc 02) — Cycle detection + tears (plan-of-record)

> Refs: ODD-0013 §4.2–§4.3. `depends_on:` arc02 slice01 (the graph).

## Goal

Detect dependency cycles (Kahn) and make breaking one an **explicit, recorded**
decision. **Done when a cycle is detected and surfaced, a `tears:` marker breaks a
named `depends_on` for ordering purposes, and an un-torn cycle is a hard error.**

## Scope

**In:** Kahn-based cycle detection over the ordering DAG; cycle reporting (the
member nodes); the `tears` marker (on the source node: a `depends_on` deliberately
assumed, with a required rationale) removing that edge from ordering; listing all
active tears; the error a cycle-without-tear raises (consumed by `check` v2 in
slice06).

**Out:** `next`/`blocked`/`path` (slice04); the `check` command wiring (slice06 —
this slice provides the detection + error type).

## Verification

`cargo test -p odm-graph` green; clippy `-D warnings`; coverage ≥ 90%. Rows in
`ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). Then slice03
(gates/status/evidence) opens.
