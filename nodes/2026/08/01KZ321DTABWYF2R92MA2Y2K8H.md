---
id: 01KZ321DTABWYF2R92MA2Y2K8H
number: 61401502
type: slice
schema: slice/v1.1
name: Slice 02 -- self-sourced planning nodes
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice02-self-sourced-nodes/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-03
edges:
  part_of: 01KZ1YCK41SK7J1H134AK7WB7T
status:
  built:
    reached: 2026-08-03
    evidence: asserted
    evidence_dates:
      asserted: 2026-08-03
  planned:
    reached: 2026-08-03
    evidence: asserted
    evidence_dates:
      asserted: 2026-08-03
  tested:
    reached: 2026-08-03
    evidence: asserted
    evidence_dates:
      asserted: 2026-08-03
---
# Slice 02 -- self-sourced planning nodes

<!-- Name/title carries no document-role metadata, per ODD-0013 sec. 2.1. -->

**Arc:** Store-as-source-of-truth & native authoring -- **Kind:** code (+ two ODD amendments)
**Opened:** 2026-08-03 -- **Design basis:** ODD-0026 (Accepted)

## Goal

Make a planning node that has **no external source** a first-class, valid citizen of the
store, so `migrate` / `reconcile` / `check` stop treating "no `./docs` source" as an error.
This is the **core enabler** of the arc: after this slice the store *is* authoritative -- the
`./docs` planning tree is no longer load-bearing for validation, only for provenance and (until
slice 04) as the place bodies are still edited.

The single sentence this slice makes true: *`odm check` is green on a planning node whose body
lives only in the store (`origin: authored`, no external `source.paths`), while every other
validation -- schema, edges, decomposition, derived order -- still applies to it.*

## Scope -- in

1. **Schema (odm-core).** Add `origin: authored` to the origin value set, and define the authored
   `source` shape per ODD-0026 sec. 2.1: `source: { class: authored }`, no `paths`, no
   `migrated_by`/`migrated_on`. Encode the **author-vs-odm field boundary** (odm owns
   `id`/`number`/placement/`source`; author owns name/type/`edges.part_of`/status-intent/tags) so
   later slices (03) and `check` can enforce it.
2. **`check` (odm-core / recompose).** A node with `origin: authored` + no external source passes
   with **no `undeveloped-stub`, no `missing-source`, no body-hash error**. Schema/edge/cycle/
   decomposition/derived-order validation is **unchanged** and still fires on authored nodes.
3. **`migrate` (odm-migrate).** Authored nodes are **not required to have** an external source and
   are **not churned**: `migrate --all` neither rewrites their bodies nor flags them missing-source;
   their bodies are the source. Genuinely-migrated nodes (external `source.paths`) behave exactly as
   today.
4. **`reconcile`.** Treats authored nodes as self-sourced: no attempt to re-establish fidelity
   against a (nonexistent) external source; no spurious drift for authored nodes.
5. **Convert the existing planning corpus to self-sourced** per ODD-0026 (see the sub-decision
   below), so `migrate --all` + `check` is green **with the `./docs` plan tree still present**
   (SS-3). This is the dogfood step that proves the model on the real 400+-node store.
6. **Two ODD amendments** (ODD-0026 sec. 3 assigned them here):
   - **ODD-0013 amendment** -- node schema: `origin: authored`; the authored `source` shape; the
     "`source` is enduring provenance, `authored` is a *value* not the absence of the field"
     language; the author-vs-odm field boundary.
   - **ODD-0025 amendment** -- authored nodes bypass the migration body-hash gate *by construction*
     (no external source to gate); migration fidelity for genuinely-migrated content is unchanged.

## Scope -- out

- **No authoring commands.** `node new --from-file`/`--content`/`--metadata`, `node edit`,
  `node set`/`set-body` are **slice 03**. Slice 02 makes authored nodes *valid*; slice 03 makes
  them *creatable via the CLI*. (Until 03, an authored node is produced by hand-editing a store file
  + `check`, or by the conversion in scope item 5.)
- **No deletion of `./docs`.** That is slice 04 (cutover). `./docs` stays present through this slice.
- **No dual-format metadata I/O** (JSON/TOML `--metadata`): that is the slice-03 authoring surface.

## The one open sub-decision (resolve early in the slice, confirm with operator)

**What becomes of the *existing* migrated planning nodes' provenance when converted to
self-sourced?** ODD-0026 fork A keeps `source` as enduring provenance; fork B makes the body-hash
gate migration-time-only. Two readings of "convert to self-sourced":

- **(i) Re-classify as authored, preserving migration provenance.** Flip `origin` -> `authored`,
  set `source.class: authored`, but **retain a historical marker** of where they were migrated from
  (e.g. `source.migrated_from: docs/...`) so provenance is not lost. *Recommended* -- it is the most
  faithful to fork A ("provenance is enduring"): the node is store-owned going forward *and* still
  records its origin.
- **(ii) Leave them as migrated (`origin: planned`, external `source.paths`) and only make `check`
  tolerant** of a source file that will vanish at cutover (safe, since the gate is migration-time-
  only). Simpler, but after cutover their `source.paths` dangle with no marker that they are now
  store-owned.

**Lean: (i).** Decide in the slice's first step and record it in the ODD-0013 amendment; it changes
what the conversion writes and what `check` expects. (Flagged, not silently chosen.)

## Verification approach

Fixtures for each `check` behavior (authored node clean; authored node with a bad edge still fails;
migrated node with a corrupted body still fails -- SS-5). The real leg is the **live corpus**: a
full `migrate --all` + `check` green with `./docs` present (SS-3), reproduced on the store. Cargo/
exec rows are attested -> CI (no macOS toolchain in the CDC sandbox); structural rows reproduced by
code read + fixture.

## Exit criteria

SS-2 (authored node passes `check`), SS-3 (existing corpus converted; `migrate --all` + `check`
green with `./docs` present), SS-5 (migrated-content gate not regressed) all met; the ODD-0013 and
ODD-0025 amendments written and accepted; schema/edge/decomposition/order validation demonstrably
still applies to authored nodes. Slice 03 (authoring commands) can then build on a store where
authored nodes are already valid.
