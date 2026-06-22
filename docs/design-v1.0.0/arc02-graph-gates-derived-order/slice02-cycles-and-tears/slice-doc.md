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
