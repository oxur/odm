---
id: 01KYP5GZP36PBEDMP6JT4Y8MHB
number: 551849000
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 01 (Arc 06): `migrate` importer core'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice01-migrate-importer-core/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKPJ5YSKRRPDZBX90X
---
# CC Prompt — Slice 01 (Arc 06): `migrate` importer core

First slice of A6 — the arc that **closes the bootstrap loop** (migrate → self-host →
retire the prose odm replaces). Build the importer *core*: the legacy → new mapping + the
`odm migrate` command, **idempotent**, **`--dry-run`-able**, **never-delete**, tested on
**fixtures**. Running it on odm's own docs is slice02; the reflexive self-host cutover is
slice03.

> **Start condition:** A5 CDC-verified + CI-green. Branch off **`release/1.0.x`** (not `main`
> — `main` is the pre-rebuild import). Branch: **`arc06-slice01-migrate-importer-core`**.

## Read first
1. `slice01-migrate-importer-core/ledger.md` (7 rows) + `slice-doc.md` (same dir) — the
   mapping table + the resolved **idempotence key** (preserved legacy `number`, skip-if-exists).
2. `../arc-plan.md` — A6 capability, Arc Ledger (this slice closes **A-1**), the open
   questions (idempotence key + DocState→gate mapping, resolved v1.2), the Carry-ins section.
3. **ODD-0013 §9** (migration) for the intended mapping semantics.
4. The legacy model: `docs/design/` (state-dirs `01-draft`…`10-superseded`; frontmatter
   `{number,title,author,component,tags,created,updated,state,supersedes,superseded-by}`).
   The new model: `crates/odm-core` (node model, `NodeType::Odd`, gate-sets, `supersedes`
   edge) + `crates/odm-store` (node create/write — **reuse it**, don't reimplement
   persistence). `crates/odm-index/Cargo.toml` as the new-crate template.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design + error handling (positioned
  errors), CLI (`clap`).
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc** at close; **propagate it into `arc-plan.md` (A-1 + version entry)
  yourself**, as A5's slices did from slice02 on).

## Task
1. **`odm-migrate` crate + `odm migrate <legacy-path> [--dry-run]`** (M-1): workspace member;
   depends on `odm-core` + `odm-store`; builds; command wired in `odm-cli`.
2. **The mapping** (M-2): legacy `number` → preserved node `number` + fresh **ULID**; `state`
   → `odd` gate-set position (pin per-state against the `odd` gate config); `supersedes`/
   `superseded-by` pair → a `supersedes` edge; carry title/author/created/updated/tags/
   component; `NodeType::Odd`; drop the state-directory.
3. **Idempotent** (M-3): describe-or-create keyed on the preserved `number` — re-run creates
   0, skips existing.
4. **`--dry-run`** (M-4): report the plan, write nothing.
5. **Never-delete** (M-5): read legacy + write new only; **no legacy file removed/mutated**;
   dustbin (`rejected`/`withdrawn`/`superseded`) → superseded/retired node (git = history).
6. **Malformed/edge → clean error/skip** (M-6): missing `number`, unknown `state`, dangling
   `supersedes` → a positioned error / reported skip, **never a panic or silent drop**.
7. **Gates** (M-7): clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.
8. **Fixtures:** add a synthetic legacy corpus under `test-data/legacy/` (several states + a
   supersedes pair + a dustbin doc + an edge-case doc) — this slice tests on fixtures, **not**
   odm's real `docs/design` (that's slice02).

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If the mapping (esp. `state` → `odd` gate) or the idempotence
  key doesn't fit the real model, raise an amendment against the arc-plan/ODD-0013 §9.
- **Never delete or mutate a legacy file** — non-negotiable (M-5); the importer is read-legacy
  / write-new. Dustbin is a node *state*, not a file deletion.
- **Idempotence keys on the preserved `number`**, not the (fresh) ULID — don't try to
  re-mint ids.
- **Fixtures only this slice** — do not run against `docs/design` or touch the
  `design-v1.0.0` plan set (slice02/03).
- **Reuse** odm-store persistence + odm-core gate config; **render** with `writeln!`+`tabled`
  (no `oxur-cli`).
- **Loud on malformed** — a silent drop here corrupts the self-host corpus in slice02/03.

## Deliverables
The `odm-migrate` crate + `odm migrate` command + the mapping + idempotent/dry-run/never-delete
semantics + fixtures, with `ledger.md` evidence per row (`attested`); a `closing-report.md` —
per-row walk **plus the Bubble-up to the arc** (did slice01 deliver A-1; what it reveals for
slice02's real-corpus run — mapping edge cases, the odd-numbering-space question; the
silent-drop diff) — **and propagate that into `arc-plan.md` (A-1 + a version entry)**. Feature
branch `arc06-slice01-migrate-importer-core`; not `main`/`release/1.0.x` directly.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-1) per LEDGER-DISCIPLINE v2.0 §A.
