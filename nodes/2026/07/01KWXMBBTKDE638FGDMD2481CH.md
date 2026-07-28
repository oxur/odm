---
id: 01KWXMBBTKDE638FGDMD2481CH
number: 1201
type: slice
schema: slice/v1.1
name: Graph construction + reverse edges
created: 2026-06-22
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc02-graph-gates-derived-order/slice01-graph-construction/slice-doc.md
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
# Slice 01 (Arc 02) — Graph construction + reverse edges (plan-of-record)

> Refs: ODD-0013 §3 (edges), §4 (graph engine). `depends_on:` arc01 slice03
> (frontmatter edge schema) + slice06 (link-integrity — refs resolve).

## Goal

Build the in-memory graph the rest of Arc 02 queries: construct the petgraph DAG
from the edge data already parsed on each node, derive reverse edges (backlinks),
and select the **ordering DAG** (`depends_on ∪ consumes`) as distinct from the
`part_of` containment tree. **Done when a node set builds a correct graph with
working forward/reverse lookups and the ordering-DAG / containment-tree split.**

## Scope

**In:** `odm-graph` over abstract `(NodeId, EdgeKind)` (zero domain knowledge); an
`odm-core` translation layer that feeds node edge-data in; forward + **derived
reverse** adjacency (backlinks for `show`); the ordering-DAG view
(`depends_on ∪ consumes`); `part_of` exposed as a separate single-parent tree;
accessors (children/parents by edge kind).

**Out:** cycle detection (slice02), gates/status (slice03), `next`/`blocked`/`path`
(slice04), recomposition checks (slice05). No new persistence — edges come from the
arc01 schema.

## Verification

`cargo test -p odm-graph -p odm-core` green; `odm-graph` carries no domain types
(grep); clippy `-D warnings`; coverage ≥ 90%. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). Then slice02
(cycles + tears) opens.
