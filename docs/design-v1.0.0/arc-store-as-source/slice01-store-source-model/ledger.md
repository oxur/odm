# Slice 01 ledger -- ODD-0026: the store-as-source model

Per `LEDGER-DISCIPLINE.md` sec. A. This is a **design slice**: rows verify that a
decision is *recorded and coherent*, not that code runs. Evidence strength tops out
at `attested`/`reconciled` (against the other ODDs) rather than `reproduced`.

> **Status 2026-08-03: slice 01 CLOSED.** All five forks ratified; **ODD-0026 Accepted**
> at `docs/design/04-accepted/0026-store-as-source-model.md` (operator, 2026-08-03). Every row
> reached a final status: D1-1 `reconciled`, D1-2..D1-9 `attested` (recorded in the accepted ODD).
> **Fork A was CORRECTED** from the original A1 lean (drop external source) to *keep provenance* -- see D1-2.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| D1-1 | ODD-0026 exists and is **Accepted** | `04-accepted/0026-*.md` present, status Accepted | serious | arc-plan SS-1 | reconciled | **Accepted** at `docs/design/04-accepted/0026-store-as-source-model.md` (operator 2026-08-03) | slice 01 closed |
| D1-2 | **Fork A** (`source.paths` for planning nodes) decided with rationale | ODD-0026 sec. 2.1 states the choice + why | serious | arc-plan | attested | ODD-0026 sec. 2.1 | **DECIDED: KEEP provenance** -- authored node = `origin: authored` + `source {class: authored}`, no external `paths`. ~~lean: drop external source (A1)~~ **overturned by operator 2026-08-03**; `source` is enduring provenance, not migration scaffolding |
| D1-3 | **Fork B** (body-hash gate) decided: migrated-with-source only, N/A for authored | ODD-0026 sec. 2.2 states the rule | serious | arc-plan | attested | ODD-0026 sec. 2.2 | B1 confirmed; N/A by construction (authored nodes never migrate); migration gate unchanged |
| D1-4 | **Fork C recorded as settled**: ODDs 0011-0025 already-migrated planning artifacts; `docs/design/` deletes in cutover | ODD-0026 sec. 2.3 states it as settled | polish | arc-plan | attested | ODD-0026 sec. 2.3 | recorded fact, not a decision -- sets slice 04 scope |
| D1-5 | **Fork D** (end-user `./docs` outside the planning store) decided | ODD-0026 sec. 2.4 states the boundary | correctness | arc-plan | attested | ODD-0026 sec. 2.4 | D1 confirmed |
| D1-6 | **Fork E** (authoring/update contract) sketched enough to drive slice 03 | ODD-0026 sec. 2.5 names the create + update affordances | serious | arc-plan | attested | ODD-0026 sec. 2.5 | odm owns the seam; canonical metadata partial + dual JSON/TOML I/O; author-vs-odm field boundary; validate-before-write; create (dual-source + skeleton) + channel-addressed edit surface |
| D1-7 | Reconciled against **ODD-0025 / ODD-0013 / ODD-0017** -- no unresolved contradiction | ODD-0026 sec. 2.0/3/8 walk each | serious | arc-plan | attested | ODD-0026 sec. 2.0, 3, 8 | slots into ODD-0025 sec. 2.0 axes (verified verbatim); sec. 3 specifies amendments to 0013 (schema) + 0025 (gate); 0017 projection-out unaffected |
| D1-8 | Slices **02-04 acceptance criteria** derived and recorded (SS-2..SS-7 made concrete) | ODD-0026 sec. 7 + arc-plan SS-rows cite ODD-0026 | serious | arc-plan | attested | ODD-0026 sec. 7 | SS-2/SS-4/SS-5/SS-6 + the metadata-contract test made concrete |
| D1-9 | **No-regression contract** stated: the fidelity gate still guards genuinely-migrated content | ODD-0026 sec. 2.2 carries the migrated-node clause | correctness | arc-plan SS-5 | attested | ODD-0026 sec. 2.2 | the gate keys off external `source.paths`; migrated-node fidelity unchanged |

## What Worked

The forks came in with recorded leans, so the decision session was a ratification pass,
not open research -- which is why one slice held the whole model. The one substantive
change (fork A) was caught by the operator, not the author, and is recorded as a
spec-keeping correction across all four docs rather than a silent edit.

## Closure

ODD-0026 **Accepted** (`docs/design/04-accepted/0026-store-as-source-model.md`); the slice's
deliverable is met and all forks are recorded and reconciled. See `closing-report.md`
(per-row walk + bubble-up to the arc) and `cdc-verification.md` (independent coherence check).

Closed 2026-08-03 (uncommitted in the plan tree; migrates to the store on the next `migrate --all`).
Verified by: CDC + operator acceptance.
Rows: 9. Final: 9 (D1-1 reconciled; D1-2..D1-9 attested). Deferred: 0. No-op: 0.
