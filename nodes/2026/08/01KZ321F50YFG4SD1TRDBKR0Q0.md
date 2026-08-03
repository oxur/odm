---
id: 01KZ321F50YFG4SD1TRDBKR0Q0
number: 568422300
type: artifact
schema: artifact/v1.1
name: 'Slice 01 -- closing report (ODD-0026: the store-as-source model)'
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice01-store-source-model/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-03
edges:
  part_of: 01KZ1YCK416DAFH90W8V1K0WDG
---
# Slice 01 -- closing report (ODD-0026: the store-as-source model)

**Slice:** 01 -- Store-as-source-of-truth & native authoring -- **Kind:** design/decision
**Closed:** 2026-08-03 -- **Deliverable:** ODD-0026, Accepted (`docs/design/04-accepted/0026-store-as-source-model.md`)

## Per-row walk (ledger D1-1..D1-9)

| Row | Final | Evidence |
|---|---|---|
| D1-1 | **reconciled** | ODD-0026 present in `04-accepted/`, `state: Accepted`, operator-accepted 2026-08-03 |
| D1-2 | **attested** | Fork A decided in sec. 2.1 -- **KEEP provenance** (`origin: authored`, no external `paths`); A1 "drop" lean overturned + recorded |
| D1-3 | **attested** | Fork B in sec. 2.2 -- body-hash N/A for authored by construction; migration gate unchanged |
| D1-4 | **attested** | Fork C in sec. 2.3 -- design corpus settled store-as-source; deletes in cutover |
| D1-5 | **attested** | Fork D in sec. 2.4 -- end-user `./docs` outside the planning store |
| D1-6 | **attested** | Fork E in sec. 2.5 -- odm owns the seam; canonical metadata partial + dual JSON/TOML I/O; field boundary; validate-before-write; command surface |
| D1-7 | **attested** | sec. 2.0/3/8 reconcile against ODD-0025/0013/0017; the two ODD-0025 claims fork B leans on were confirmed verbatim in the accepted 0025 |
| D1-8 | **attested** | sec. 7 records the SS-2/SS-4/SS-5/SS-6 + metadata-contract acceptance tests slices 02-04 inherit |
| D1-9 | **attested** | sec. 2.2 carries the no-regression clause (gate keys off external `source.paths`; migrated fidelity intact) |

Rows at open: 9. Rows at close: 9. **No silent drops.**

## Bubble-up to the arc

**1. Did this slice deliver the arc-plan's assigned piece?** Yes. The arc-plan assigned slice
01 "author + accept ODD-0026, from which slices 02-04 derive their acceptance criteria." ODD-0026
is Accepted and sec. 7 supplies those criteria; arc ledger **SS-1 -> done**.

**2. What did implementing this slice reveal that the arc-plan did not anticipate?**

- **Fork A reversed.** The arc-plan's "*Lean: drop it*" for fork A was **overturned** -- `source`
  is enduring provenance, kept universally; "authored" is a provenance *value*. The arc-plan's
  "The design forks" section and the slice-doc recommendation both carried the stale lean and were
  corrected (spec-keeping). This is the one real plan change from the slice.
- **Scope added to slice 02.** ODD-0026 requires a **schema amendment to ODD-0013** (add
  `origin: authored`, the authored `source` shape, the author-vs-odm field boundary) and an
  **ODD-0025 amendment** (authored nodes bypass the migration body-hash gate). These are now
  explicit deliverables for slice 02, not just "make check pass" -- recorded in slice 02's open set.
- **No re-sequencing.** `02 -> 03 -> 04` unchanged; the model landed as planned.

**3. Silent-drop diff (scope-as-specified vs delivered).** Slice-doc scope-in = resolve forks
A/B/D/E, record C, author+accept ODD-0026, reconcile against 0025/0013/0017, derive 02-04 criteria.
All delivered. Scope-out (no code, no deletion, no authoring-command detail beyond the fork-E
sketch) -- honored; none leaked in. Nothing dropped.

## Notes

Authored in the plan tree per the operator ("do this in the old place; hard switch after native
authorship lands"); ODD-0026 and these close docs migrate into the store on the next `migrate --all`.
Uncommitted at close.
