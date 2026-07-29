---
id: 01KYP5GABABHRWRJS6CG4CPRN3
number: 584217500
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 01 (Arc 02): Graph construction + reverse edges'
created: 2026-06-22
updated: 2026-06-22
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc02-graph-gates-derived-order/slice01-graph-construction/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKDE638FGDMD2481CH
---
# CC Prompt — Slice 01 (Arc 02): Graph construction + reverse edges

Build the in-memory graph Arc 02 queries. **No cycle detection, no gates, no
`next`/`blocked` yet** — just construction, reverse derivation, and the
ordering-DAG / containment-tree split.

> **Start condition:** arc01 must be CDC-closed (the edge schema + link-integrity
> exist). If not, hold.

## Read first
1. `slice01-graph-construction/ledger.md` (8 rows) — read before coding.
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §3 (edges) + §4 (engine).

## Load skills
- **rust-guidelines**: `11-anti-patterns.md`, `06-traits.md`, `05-type-design.md`.
  Keep `odm-graph` a pure engine over abstract `(NodeId, EdgeKind)` — domain
  translation lives in `odm-core`.
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
- `odm-graph`: build a petgraph-backed DAG from `(NodeId, EdgeKind, NodeId)` edges;
  forward + **derived reverse** adjacency; accessors by edge kind. No domain types.
- `odm-core`: translate a node set's edge-data into the graph; expose the
  **ordering DAG** (`depends_on ∪ consumes`) and the **`part_of` tree** (separate,
  single-parent) as distinct views.

## Constraints
- Reverse edges are **derived**, never stored (one place to edit — the source).
- `odm-graph` stays domain-agnostic (H-6 greps for leaked names).
- No `unsafe`; typed errors; coverage ≥ 90%.

## Deliverables
- Green: `cargo test -p odm-graph -p odm-core`, `clippy -D warnings`, `llvm-cov ≥ 90%`.
- `ledger.md` with evidence per row; `closing-report.md` (per-row walk, What
  Worked, uncertainties). Feature branch; not `main`.

## Working agreement
Amend don't work around; five-iteration cap; `done` is proposed-done (CDC re-runs
via CI/local 1.85+) before slice02 opens.
