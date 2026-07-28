---
id: 01KWXMBBTKRBQQCVA978YB6YMM
number: 1105
type: slice
schema: slice/v1.1
name: Node CRUD commands
created: 2026-06-22
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc01-substrate-node-crud/slice05-node-crud/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KWXMBBTKKTRXE1DF6WYSG5JB
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
# Slice 05 (Arc 01) — Node CRUD commands (plan-of-record)

> Refs: ODD-0013 §7 (command surface), ODD-0015 §3 (A1.5). `depends_on:` slice04
> (store) + slice02 (types). Closes the gap noted in slice01 (idempotent
> describe-or-create, `use`/`context`, `retire`).

## Goal

Give `odm` its node CRUD surface over the store: create, list, show, rename,
retire, supersede — plus current-project/arc context. **Done when each command
round-trips through the store with `--dry-run`/`--yes`/`--json`, and identity
survives rename (file never moves).**

## Scope

**In:** `new <type> <name>` (mints a ULID, persists; **idempotent
describe-or-create** — re-running describes rather than duplicating);
`list` (full scan; filters: type, tag, component); `show X` (node + edges +
way-finding in one call); `rename` (changes `name`, not `id`/path);
`retire X --because` (mark withdrawn/removed — git-preserved, never a destructive
delete); `supersede X --with Y --kind obsoletes|updates`; `use [project|arc] X`
and `context` (current selection, so `--project`/`--arc` need not repeat).
`--dry-run` + `--yes` on mutators; `--json` on queries.

**Out:** graph queries `next`/`blocked`/`path` (Arc 02); `check` (slice06); the
generated rollup/`orient` (Arc A3); status `set-gate` (Arc 02 slice03).

## Verification

`cargo test -p odm-cli` + `assert_cmd` integration tests green; rename leaves
`id`/path unchanged; `new` is idempotent; `--dry-run` writes nothing; `--json`
schema snapshot-tested; clippy `-D warnings`; coverage ≥ 90%. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). Then slice06
(`check` v1) validates the corpus these commands produce.
