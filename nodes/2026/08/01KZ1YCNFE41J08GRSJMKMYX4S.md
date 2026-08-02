---
id: 01KZ1YCNFE41J08GRSJMKMYX4S
number: 515494000
type: artifact
schema: artifact/v1.1
name: 'Slice 01 ledger — ODD-0026: the store-as-source model'
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice01-store-source-model/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK416DAFH90W8V1K0WDG
---
# Slice 01 ledger — ODD-0026: the store-as-source model

Per `LEDGER-DISCIPLINE.md` §A. This is a **design slice**: rows verify that a
decision is *recorded and coherent*, not that code runs. Evidence strength tops out
at `attested`/`reconciled` (against the other ODDs) rather than `reproduced`.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| D1-1 | ODD-0026 exists and is **Accepted** | `docs/design/04-accepted/0026-*.md` present, status Accepted | serious | arc-plan SS-1 | open | | |
| D1-2 | **Fork A** (`source.paths` for planning nodes) decided with rationale | ODD-0026 §"source" states the choice + why | serious | arc-plan | open | | lean: drop external source (A1) |
| D1-3 | **Fork B** (body-hash gate) decided: applies to migrated-with-source only, N/A for authored | ODD-0026 §"fidelity gate" states the rule | serious | arc-plan | open | | lean: B1 |
| D1-4 | **Fork C recorded as settled**: ODDs 0011–0025 are already-migrated planning artifacts → store-as-source; `docs/design/` deletes in cutover | ODD-0026 §"design corpus" states it as settled | polish | arc-plan | open | | not a decision — a recorded fact that sets slice 04 scope |
| D1-5 | **Fork D** (end-user `./docs` outside the planning store) decided | ODD-0026 §"end-user docs" states the boundary | correctness | arc-plan | open | | lean: D1 |
| D1-6 | **Fork E** (authoring/update contract) sketched enough to drive slice 03 | ODD-0026 §"authoring" names the create + update affordances | serious | arc-plan | open | | lean: `node new --from-file` + `node edit` |
| D1-7 | Reconciled against **ODD-0025 / ODD-0013 / ODD-0017** — no unresolved contradiction | ODD-0026 §"reconciliation" walks each; tensions named + resolved | serious | arc-plan | open | | ODD-0025 is the one most affected |
| D1-8 | Slices **02–04 acceptance criteria** derived and recorded (SS-2…SS-7 made concrete) | ODD-0026 §"downstream" or the arc-plan's SS-rows updated to cite ODD-0026 sections | serious | arc-plan | open | | closes the loop into the arc ledger |
| D1-9 | **No-regression contract** stated: the fidelity gate still guards genuinely-migrated content | ODD-0026 §"fidelity gate" carries the migrated-node clause | correctness | arc-plan SS-5 | open | | prevents B1 from weakening migration fidelity |

## What Worked

_(At slice close.)_

## Closure

Closed at commit <SHA> on <date>. Verified by: <name/session>.
Rows: 9. Done: _. Deferred: _. No-op: _.
