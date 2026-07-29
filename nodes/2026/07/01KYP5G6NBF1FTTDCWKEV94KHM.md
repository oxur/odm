---
id: 01KYP5G6NBF1FTTDCWKEV94KHM
number: 511982700
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 01: Workspace scaffolding'
created: 2026-06-20
updated: 2026-06-20
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc01-substrate-node-crud/slice01-workspace-scaffolding/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKSH6T4XPS87BQFDD5
---
# CC Prompt — Slice 01: Workspace scaffolding

You are implementing **Slice 01** of the `odm` v1.0.0 rebuild in the `odm` repo.
This slice restructures the single legacy crate into the new multi-crate workspace
skeleton. **No domain logic — stubs only.** The slice is done when `make check` is
green and the ledger is fully closed.

## Read first (in this order)

1. `docs/design-v1.0.0/arc01-substrate-node-crud/slice01-workspace-scaffolding/ledger.md`
   — the acceptance criteria. **Read before writing any code.** It is the spec of
   "done," not an end-of-slice checklist.
2. `slice-doc.md` (same dir) — the plan, scope, and the baked-in decisions.
3. `arc-plan.md` (parent dir) — the arc this slice belongs to.
4. `docs/design/01-draft/0013-odm-architecture-design.md` §8 — the crate map.
5. `CLAUDE.md` — repo conventions (you will also update it: edition + crate map).

## Load these skills

- **rust-guidelines** — start with `11-anti-patterns.md`, then
  `12-project-structure.md` and the `15-cargo/` guides (workspace, lints,
  manifest-and-workspace-advanced). This slice is pure Cargo/workspace work.
- **collaboration-framework → LEDGER_DISCIPLINE** — you work *against* the ledger:
  fill Evidence at the commit each row is met; in the closing report walk every row
  with its disposition (no prose summary); name any uncertainty.

## Task

Build the workspace skeleton per `slice-doc.md`:

- Relocate the legacy crate: `git mv crates/oxur-odm legacy/oxur-odm`, rename its
  package to `oxur-odm-legacy`, and exclude it from the workspace. **Do not delete
  it** — it is the harvest source and git history must survive (`--follow`).
- Root `Cargo.toml`: `[workspace]` members = the 5 new crates; `[workspace.package]`
  (version `1.0.0`, edition `2024`, rust-version `1.85`, authors/license/repository);
  `[workspace.dependencies]` (the shared set in `slice-doc.md`); `[workspace.lints]`
  (rust + clippy, warnings denied).
- Create 5 crates — `oxur-odm` (umbrella, `[[bin]] name = "odm"`, prints
  `--version`), `odm-cli`, `odm-core`, `odm-store`, `odm-graph` — each inheriting
  workspace package + lints, with a minimal `lib.rs`/`main.rs` and **one
  `#[test] fn smoke()`** so the test/coverage harness is non-empty.
- Adapt the migrated `Makefile` and `.github` CI to the workspace.
- Update `README.md` (workspace layout) and `CLAUDE.md` (edition 2024; new crate map).

## Constraints & decisions (from `slice-doc.md` — honor exactly, flag don't silently change)

- Edition 2024, resolver 2, `max_width = 100`, clippy `-D warnings`.
- **Add** `petgraph`, `ulid`, `gix`, `confyg` to shared deps; **drop** `uuid`.
- **Do NOT pin a YAML frontmatter library** — that choice is deferred to slice03
  (`serde_yaml` is archived; slice03 decides deliberately). This slice needs no YAML.
- Only the 5 crates above; do not create `odm-index`/`-reconcile`/`-migrate` yet.
- Dependencies live in `[workspace.dependencies]`; crate manifests reference them
  with `.workspace = true` (no version literals in crate manifests).

## Deliverables

- The workspace compiling green: `make check` and `make coverage` exit 0.
- `ledger.md` updated with Evidence (commit SHA + Verify output) per row.
- `closing-report.md` in this dir: a per-row walk (status + evidence for all 15
  rows), a "What Worked" note, and any uncertainties named.

## Working agreement

- If a ledger row is wrong/impossible/supersedable, **raise it as an amendment** —
  do not silently work around it.
- Five-iteration cap. If you hit it without convergence, stop and report rather than
  grinding a sixth pass.
- CDC (Duncan / the planning thread) verifies every `done` row independently before
  the slice advances; treat your `done` as "proposed done."
