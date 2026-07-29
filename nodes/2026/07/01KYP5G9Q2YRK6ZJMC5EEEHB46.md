---
id: 01KYP5G9Q2YRK6ZJMC5EEEHB46
number: 514901800
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 06 (Arc 01): `check` v1 + link-integrity'
created: 2026-06-22
updated: 2026-06-22
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc01-substrate-node-crud/slice06-check-v1/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKGJPFBVZZ3QKZ226T
---
# CC Prompt — Slice 06 (Arc 01): `check` v1 + link-integrity

The first mechanical gate: validate the corpus's structure (completeness,
link-integrity, supersession chains) with CI-grade exit codes. Closes Arc 01.

> **Start condition:** slices 03 (schema), 04 (store), 05 (CLI) CDC-closed. Else hold.

## Read first
1. `slice06-check-v1/ledger.md` (9 rows).
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §7 (+ §2.3, §3).

## Load skills
- **rust-guidelines**: `11-anti-patterns.md`, `14-cli-tools/03-error-handling.md`
  (exit codes), `03-error-handling.md`.
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
`odm check` over the full-scan-loaded corpus:
- Required-field completeness per node type.
- **Link-integrity**: every `part_of`/`supersedes`/edge ref resolves to a real id
  (no dangling refs).
- **Supersession-chain integrity**: acyclic, terminating; no self-supersede.
- Exit codes `0`/`1`/`2`; **errors-as-affordances** (name the exact fix command);
  `--json` report.

## Constraints (flag, don't silently change)
- This is `check` **v1** — structural only. Cycles/staleness/recomposition/
  soft-satisfaction are Arc 02 `check` v2; do not implement them here, but structure
  the code so v2 can *extend* (not rewrite) it.
- No `unsafe`; typed errors; coverage ≥ 90%; `assert_cmd` for exit-code tests.

## Deliverables
Green test/clippy/coverage (broken corpus flagged; clean corpus exit 0); `ledger.md`
evidence per row; `closing-report.md`. Feature branch (`slice06-check-v1`); not
`main`. **On close, Arc 01 is complete.**

## Working agreement
Amend don't work around; five-iteration cap; proposed-done → CDC via CI/local 1.85+.
