# Slice 02 cc-prompt -- self-sourced planning nodes

**You are CC.** Real toolchain, with tests. Read, in order: **ODD-0026** (Accepted,
`docs/design/04-accepted/0026-store-as-source-model.md`) -- especially sec. 2.1 (fork A, kept
provenance), sec. 2.2 (fork B, gate is migration-time-only), and sec. 3 (the amendments); then this
slice's `slice-doc.md` and `ledger.md` (F-1..F-11). The acceptance anchors are **F-2** (authored
node passes `check`), **F-6** (live corpus green), and **F-7** (no regression).

## The model (from ODD-0026)

A planning node's body can live only in the store. Such an **authored** node has
`origin: authored` and `source: { class: authored }` with **no external `paths`**. It is not
migrated, so the body-hash migration gate never applies to it (fork B: the gate is a migration-time
check keyed off an external `source.paths`; no external source -> nothing to gate). Everything else
-- schema, typed edges, cycle detection, decomposition, derived order -- **still applies**.
`source`/provenance is **kept** as an enduring field (fork A): do not drop it; an authored node's
provenance *value* is "authored", not "absent".

## What to build

1. **Schema (odm-core).** Add the `authored` origin value and the authored `source` shape; validate
   it. Encode the author-vs-odm field boundary (odm owns `id`/`number`/placement/`source`) so it can
   be enforced -- slice 03 will lean on it.
2. **`check`.** Suppress `undeveloped-stub` / `missing-source` / body-hash errors for authored
   nodes **only**; keep every other rule firing on them. Migrated nodes are unaffected.
3. **`migrate --all`.** Do not require/rewrite an external source for authored nodes; do not churn
   their bodies. Migrated-node behavior is unchanged.
4. **`reconcile`.** Authored nodes are self-sourced -- no re-fidelity, no spurious drift.
5. **Convert the existing planning corpus.** FIRST resolve the sub-decision in `slice-doc.md`
   ("open sub-decision") -- the lean is **(i) re-classify as authored, preserving a
   `source.migrated_from` marker** -- confirm with the operator, record it in the ODD-0013
   amendment (F-10), then convert. `migrate --all` + `check` must be green with `./docs` present.
6. **Amendments (F-8, F-9).** Write the ODD-0013 and ODD-0025 amendment docs in this slice
   directory (follow the RH amendment-stub format, e.g. `arc-release-hardening/C-2-amendment-ODD-0013.md`).

## Constraints -- do not regress

- **F-7 is load-bearing:** a genuinely-migrated node whose body diverges from its external source
  MUST still fail `check`. Fork B narrows the gate to migrated-with-source nodes; it does not remove
  it. Add the regression fixture.
- **Keep `delta.rs` / the store git plumbing behavior** intact (this slice is model/validation, not
  store-commit).
- **Names/titles carry no role metadata** (ODD-0013 sec. 2.1) -- do not reintroduce labels on any
  node you touch during conversion.

## Definition of done

F-1..F-11 reach a final status; an authored node passes `check` while a bad-edge authored node and a
corrupted migrated node both fail; `migrate --all` + `check` green on the live store with `./docs`
present; the two amendments written; `make check` green. Close with the per-row ledger walk + a
bubble-up to the arc (did this deliver the core enabler; what should the arc-plan or slice 03/04
note; the silent-drop diff). Flag anything ODD-0026 under-specified.
