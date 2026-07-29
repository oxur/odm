---
id: 01KWXMBBTKPJ5YSKRRPDZBX90X
number: 1601
type: slice
schema: slice/v1.1
name: '`migrate` importer core'
created: 2026-07-06
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice01-migrate-importer-core/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KWXMBBTKNA3A0QC3SWPHBNAX
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
# Slice 01 (Arc 06): `migrate` importer core

> Plan-of-record for the first slice of A6 (Migrate, self-host & PM-skill) — the arc that
> **closes the bootstrap loop**. This slice builds the importer *core*: the legacy → new
> mapping + the `odm migrate` command, **idempotent**, **`--dry-run`-able**, **never-delete**,
> tested on **fixtures** (a synthetic legacy corpus). Running it on odm's *own* docs is
> slice02; the reflexive self-host cutover is slice03.

## Goal

`odm migrate <legacy-path>` reads a legacy number-/state-directory ODD corpus and creates
equivalent nodes in the new model — losslessly (legacy provenance preserved), safely (no
legacy file touched), and repeatably (re-running is a no-op). After this slice the mechanism
exists and is proven on fixtures; slice02 points it at the real `docs/design`.

## The legacy → new mapping (the crux)

The legacy model (odm's own `docs/design`, oxur-era): numbered files in state-directories
(`01-draft`…`10-superseded`) with frontmatter `{ number, title, author, component, tags,
created, updated, state, supersedes, superseded-by, version }`. The new model:
ULID-identified `nodes/YYYY/MM/<ULID>.md` with gates.

| Legacy | → | New |
|--------|---|-----|
| `number` (e.g. 14) | → | fresh **ULID** identity; legacy number **preserved as the node's `number`** |
| `state` scalar (`Draft`…`Superseded`) | → | the **`odd` gate-set position** (per the gate config); the state-*directory* is **dropped** (redundant truth) |
| `supersedes` / `superseded-by` pair | → | a **`supersedes` edge** (with kind) on the superseding node |
| dustbin (`08-rejected`/`09-withdrawn`/`10-superseded`) | → | **supersede/retire** node state — **git preserves history**, the file is never deleted |
| `title`/`author`/`created`/`updated`/`tags`/`component` | → | carried onto the new node's frontmatter |
| node type | → | **`NodeType::Odd`** |

**Idempotence key (resolved, arc-plan v1.2):** identity is a fresh ULID (can't be re-minted),
so idempotence is a **describe-or-create keyed on the preserved legacy `number`** — a legacy
doc whose `number` already exists as an `odd` node is **skipped**, not duplicated.

## Scope — in

1. **`odm-migrate` crate** + the **`odm migrate <legacy-path> [--dry-run]`** command
   (workspace member; `[workspace.dependencies]`/`[workspace.lints]`; no version literals).
   Depends on `odm-core` (node model, gate-sets, `supersedes` edge) + `odm-store` (node
   create/write). Mirrors how A4 slice01 created `odm-index`.
2. **The mapping** (table above) — each legacy field mapped; `NodeType::Odd`; legacy `number`
   preserved; `state` → `odd` gate position (pin the exact per-state mapping against the `odd`
   gate-set).
3. **Idempotent describe-or-create** — re-running `migrate` creates nothing new (skips docs
   whose `number` already exists as an `odd`).
4. **`--dry-run`** — report the plan (per-doc create/skip + counts) and write **nothing**.
5. **Never-delete / supersede-not-delete** — the importer only reads legacy + writes new
   nodes; **no legacy file is removed or mutated**; dustbin docs import as superseded/retired
   (git preserves history), legacy files intact.
6. **Malformed/edge input handled** — a legacy doc with a missing `number`, an unknown
   `state`, or a dangling `supersedes` target → a **clear reported error/skip, never a panic
   or silent drop** (the project error convention).

## Scope — out (named, not dropped)

- **Running on odm's own `docs/design`** (real-corpus edge cases; `check` green) — **slice02**.
- **Self-host cutover** (the `design-v1.0.0` plan set into `nodes/`; the reflexive migrate;
  git-checkpoint safety) — **slice03**.
- **PM-skill population / retiring framework prose** — **slice04/05** (incl. the carried
  `CLAUDE.md` oxur-cli doc-drift fix, arc-plan v1.2).
- **Migrating oxur's `crates/design/docs`** — a scope call deferred to slice02.

## Verification approach

`odm-migrate` tests over a **fixture legacy corpus** (synthetic ODDs under
`test-data/legacy/`, spanning several states + a supersedes pair + a dustbin doc + an
edge-case doc):

- each legacy field maps correctly (number→number+ULID; state→gate; supersedes-pair→edge;
  dustbin→supersede/retire; type=odd).
- migrate twice → the second run creates 0 (idempotent, keyed on `number`).
- `--dry-run` writes nothing (no `nodes/` created) and prints the plan.
- after migrate, **all legacy fixture files still exist** (never-delete).
- a malformed/edge doc → a positioned error / reported skip, no panic.
- clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

Cargo rows are CC-`attested`, reproduced via CI / local 1.85+.

## Exit criteria

Ledger M-1…M-7 reach a final status: the `odm-migrate` crate + `odm migrate` command exist;
the legacy→new mapping is faithful (number preserved, state→gate, supersedes→edge,
dustbin→supersede, type=odd); re-running is a no-op (idempotent on `number`); `--dry-run`
writes nothing; no legacy file is ever deleted/mutated; malformed input is a clean
error/skip; gates pass. **On close, slice02 runs it on odm's real docs.**

> **Render/convention:** `writeln!` + `tabled` (no `oxur-cli` dep — the A5/slice03 finding).
> **Git:** branch off `release/1.0.x`. Reuse odm-store's node create/write + odm-core's gate
> config; do not reimplement node persistence.
