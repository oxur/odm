# CC Prompt — Slice 05 (Arc 01): Node CRUD commands

Give `odm` its node CRUD surface over the store, plus current-project/arc context.

> **Start condition:** slice04 (store) CDC-closed. Else hold.

## Read first
1. `slice05-node-crud/ledger.md` (11 rows).
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §7 (command surface).

## Load skills
- **rust-guidelines**: `11-anti-patterns.md`, `14-cli-tools/02-argument-parsing.md`,
  `14-cli-tools/04-output-and-ux.md`, `02-api-design.md`.
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
`new` (mint ULID + next human number + persist; **idempotent describe-or-create**),
`list` (full scan; filter type/tag/component), `show X` (node + edges + way-finding),
`rename` (name only — id/path unchanged), `retire X --because` (withdraw; git-
preserved, never delete), `supersede X --with Y --kind obsoletes|updates`,
`use [project|arc] X` + `context`. `--dry-run`/`--yes` on mutators; `--json` on
queries.

## Constraints (flag, don't silently change)
- `rename` must not change `id` or the on-disk path (K-5) — identity is stable.
- `retire` ≠ delete — git preserves history (K-6).
- `new` re-run is describe, not duplicate (K-2).
- Data → stdout, diagnostics → stderr; detect TTY before colour; no `unsafe`;
  coverage ≥ 90%.

## Deliverables
Green `cargo test -p odm-cli` + `assert_cmd`; clippy/coverage; `ledger.md` evidence
per row; `closing-report.md`. Feature branch (`slice05-node-crud`); not `main`.

## Working agreement
Amend don't work around; five-iteration cap; proposed-done → CDC via CI/local 1.85+
before slice06 opens.
