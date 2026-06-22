# CC Prompt — Slice 02 (Arc 02): Cycle detection + tears

Add Kahn cycle detection and the explicit `tears` mechanism to the graph.

> **Start condition:** arc02 slice01 (graph construction) CDC-closed. Else hold.

## Read first
1. `slice02-cycles-and-tears/ledger.md` (8 rows).
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §4.2–§4.3.

## Load skills
- **rust-guidelines** (`11-anti-patterns.md`, `06-traits.md`).
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
- Kahn-based cycle detection over the ordering DAG; report cycle members.
- `tears` marker (source-node `depends_on` deliberately assumed, **rationale
  required**) removes that edge from ordering, breaking the cycle.
- A cycle without a tear is a hard, typed error (slice06's `check` consumes it).
- Enumerate active tears so assumed dependencies stay visible.

## Constraints
- A tear is **explicit** — never silently tolerate a cycle (0013 §4.3).
- Stays in `odm-graph` (domain-agnostic); no `unsafe`; typed errors; coverage ≥ 90%.

## Deliverables
Green test/clippy/coverage; `ledger.md` evidence per row; `closing-report.md`.
Feature branch; not `main`.

## Working agreement
Amend don't work around; five-iteration cap; proposed-done → CDC via CI/local 1.85+.
