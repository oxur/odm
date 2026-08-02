---
id: 01KZ1YCNAXEMCNYA7MSJ1M4K4R
number: 568560200
type: artifact
schema: artifact/v1.1
name: 'Slice 01 design brief — ODD-0026: the store-as-source model'
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice01-store-source-model/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK416DAFH90W8V1K0WDG
---
# Slice 01 design brief — ODD-0026: the store-as-source model

> **This slice is unlike a normal CC implementation slice.** Its deliverable is an
> accepted design decision (ODD-0026), not code. The primary author is **CDC with
> the operator** — a decision, not an implementation. This document stands in for
> the usual `cc-prompt.md`; there is no code assignment for CC here. Slices 02–04
> carry the CC implementation prompts, written against the model this slice settles.

## The assignment

Author **ODD-0026 — store-as-source-of-truth for planning nodes**, resolving the
four open forks below (A, B, D, E; C is already settled), and take it to Accepted. The model must make one sentence true:
*a planning node's body is authored and owned in the store; odm no longer depends on
an external `./docs` source to hold or verify it.*

Work against `slice-doc.md` (the forks + recommendations) and `ledger.md` (the nine
rows that define done). Do not write code, delete anything, or design the slice-03
command surface beyond the fork-E contract sketch.

## Grounding facts (verified 2026-08-02, direct read)

- `node new` (odm-cli `lib.rs`) takes only `node_type`, `name`, `parent`,
  `dry_run`, `yes` — **no body arg**. There is no `node edit`/`set-body`. Node
  bodies enter the store only via `migrate` from an external file; the CLI's error
  affordances say *"edit {file}"*.
- Live planning nodes carry `source.paths: [docs/design-v1.0.0/…]` + a `class`
  (e.g. `arc-plan`) and are body-hash-gated against that file (ODD-0025).
- ODD-0025 **already** exempts the project synthesis node and retired nodes from
  the 1:1-external-source rule — so "a node with no external source" is an existing
  modeled category, not a new one. This ODD generalizes it.

## What ODD-0026 must contain

1. **The model**: planning nodes are self-sourced; the store node file is the source
   of truth; `./docs` is not required to hold or verify a planning node.
2. **Fork A — source**: the exact marker for an authored (self-sourced) node and
   what happens to `source.paths`/`class`.
3. **Fork B — fidelity gate**: the rule for when the body-hash gate applies
   (migrated-with-source only) and the explicit no-regression clause (D1-9): a
   genuinely-migrated node with a corrupted body still fails `check`.
4. **Fork C — design corpus (settled, record only)**: the ODDs 0011–0025 are
   already-migrated planning artifacts → store-as-source; `docs/design/` deletes in
   the cutover. State it as settled and carry the consequence into slice 04's scope.
5. **Fork D — end-user `./docs`**: the boundary (odm tracks planning, not product
   docs).
6. **Fork E — authoring contract**: the create/update affordances slice 03 builds
   (`node new --from-file` + `node edit <ref>`; hand-edit-plus-`check` fallback).
7. **Reconciliation**: a row-by-row pass against ODD-0025, ODD-0013, ODD-0017 with
   any tension named and resolved.
8. **Downstream**: the acceptance criteria for slices 02–04 (mapping to arc-ledger
   SS-2…SS-7), so implementation has concrete targets.

## Definition of done

All nine `ledger.md` rows reach a final status; ODD-0026 is Accepted; and the
arc-plan's SS-2…SS-7 rows are concrete enough to test. Close with the per-row walk
and a bubble-up to the arc (did this slice settle the model the arc needs; what did
deciding it reveal that the arc-plan didn't anticipate).
