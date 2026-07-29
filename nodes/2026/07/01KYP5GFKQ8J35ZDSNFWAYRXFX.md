---
id: 01KYP5GFKQ8J35ZDSNFWAYRXFX
number: 513590000
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 07 (Arc 02): CLI graph-mutators'
created: 2026-06-24
updated: 2026-06-24
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc02-graph-gates-derived-order/slice07-cli-graph-mutators/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKJ6S8A3VPPMGZ59YA
---
# CC Prompt — Slice 07 (Arc 02): CLI graph-mutators

Wire the existing odm-core mutators to the CLI so a graph can be **built and
advanced through `odm` alone** — no hand-editing frontmatter. Closes Arc 02 and is
the self-hosting prerequisite.

> **Start condition:** arc02 slices 02 (Tear), 03 (Status/`set_gate`), 04 (typed
> edges/status in Frontmatter) CDC-closed. (Independent of slice06; sequenced after
> it only because you're one agent.) If those aren't in, hold.

## Read first
1. `slice07-cli-graph-mutators/ledger.md` (13 rows).
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §7 (commands), §3 (edges),
   §4.3 (tears), §5.1 (gates).

## Load skills
- **rust-guidelines**: `11-anti-patterns.md`, `14-cli-tools/02-argument-parsing.md`,
  `14-cli-tools/03-error-handling.md`, `02-api-design.md`.
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task (all via the in-process `dispatch` pattern from slice05; persist via odm-store)
- `link X <edge> Y` — `depends_on` (+ `--satisfied-at <gate>`), `blocked_by`,
  `consumes`, `verifies`, `affects`, `part_of`. Edge on the **source** X; reverse is
  derived, never written. `part_of` enforces single-parent (replace, not append).
- `unlink X <edge> Y` — remove; absent edge → clear no-op.
- `set-gate X <gate> [--by] [--evidence]` — via `Status::set_gate` (existing); reject
  out-of-set gate (`UnknownGate`); default evidence `asserted`; records the slice05.1
  `evidence_dates` first-reach.
- `tear X depends_on Y --because <r>` — via `Tear::new` (existing); empty rationale
  rejected (`MissingRationale`).
- `decomposed X --children <ref…>` — affirm X's decomposition complete (wraps
  `affirm_decomposed`); gives `check`'s decomposition affordances a real command.
- Confirm/extend `new --parent <ref>` sets `part_of`.
- Resolve endpoints by **id | number | unique name-prefix** (reuse slice05's
  resolver). `--dry-run`/`--yes` on every mutator.

## Constraints (flag, don't silently change)
- Reuse the odm-core ops (`edges_mut`, `Status::set_gate`, `Tear::new`) — don't
  reimplement model logic in the CLI.
- Reverse edges stay derived (never written). Mutations persist atomically (odm-store).
- Errors-as-affordances: every failure names the exact fix. No `unsafe`; coverage ≥ 90% (line).
- M-11 is the headline: a graph built **purely via the CLI** must answer
  `next`/`blocked` correctly (the self-host smoke test).

## Deliverables
Green `cargo test -p odm-cli` + clippy + coverage; `ledger.md` evidence per row;
`closing-report.md` (per-row walk for all 13, What Worked, uncertainties). Feature
branch (`arc02-slice07-cli-graph-mutators`); not `main`.

## Working agreement
Amend don't work around; five-iteration cap; proposed-done → CDC via CI/local 1.85+.
On close, **Arc 02 is complete** and odm is self-host-usable.
