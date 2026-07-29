---
id: 01KYP5FTSEM55QMKTYTAQZNFF3
number: 582916700
type: artifact
schema: artifact/v1.1
name: 'Slice 02 (Migration Fidelity): The fidelity model (ODD-0025)'
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice02-fidelity-model/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6S2CXBMG4KMCW702M1
---
# Slice 02 (Migration Fidelity): The fidelity model (ODD-0025)

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`). **Design/model slice** — the deliverable is a
> document; "reproduced" = an independent party (here the **operator**, since CDC authored)
> confirmed the decision is recorded unambiguously. No code / `cargo` rows. Mints no nodes.
>
> **CLOSED 2026-07-27.** Deliverable: **ODD-0025** (Accepted, `docs/design/04-accepted/`).
> Authored CDC-seat; **independent gate = operator** (Duncan confirmed every §7 decision
> 2026-07-27 — the closer≠verifier separation is satisfied by the operator, not a self-check).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | ODD-0025 exists, registered (number 25, `state:` set, format matches a sibling) | frontmatter parses; `number: 25`; sibling format | serious | slice-doc | **done** | ODD-0025 @ `docs/design/04-accepted/0025-migration-fidelity-model.md`, `number: 25`, `state: Accepted`, frontmatter matches 0024. reproduced (operator) | Moved 01-draft→04-accepted on acceptance. |
| F-2 | Stored migration record specified: paths (list), class, normalization, tool+version — computed at migration, **no stored hashes** | grep ODD for the schema block + no-hash statement | serious | design-notes F5 | **done** | ODD-0025 **§2.2** (`source` sub-map) + §2.0. reproduced (operator) | **Renamed `provenance`→`source`** to respect 0013's `provenance`=derived reservation (§2.0) — a tracked change, not silent. |
| F-3 | No-transform body rule recorded | grep ODD | serious | design-notes F3 | **done** | ODD-0025 **§2.1**. reproduced (operator) | |
| F-4 | Body-hash gate recorded (hard-fail, migration-time) | grep ODD | serious | design-notes F2/F5 | **done** | ODD-0025 **§2.1**. reproduced (operator) | |
| F-5 | Normalization boundary decided + recorded (F4) | grep ODD | serious | design-notes F4 | **done** | ODD-0025 **§2.1** — `trim+lf`. **operator-confirmed 2026-07-27** | |
| F-6 | Synthesis model recorded (`supersedes`→Vec, bidirectional, synthesis-type) | grep ODD | serious | design-notes F1/F2 | **done** | ODD-0025 **§2.3**. reproduced (operator) | |
| F-7 | `artifact` node class defined (F9), nearest-modeled-scale, no chunk scale | grep ODD | serious | design-notes F9 | **done** | ODD-0025 **§2.5**. **operator-confirmed** (new type, American spelling) | |
| F-8 | Report self-coverage disposition recorded (F10) | grep ODD | serious | design-notes F10 | **done** | ODD-0025 **§2.6** — **mint-all** (no exemption). **operator-confirmed** | Resolved stronger than the draft lean (mint-all, not exempt). |
| F-9 | Containment-optional for doc nodes recorded (F7) | grep ODD | correctness | design-notes F7 | **done** | ODD-0025 **§2.7**. reproduced (operator) | |
| F-10 | Update-in-place fix vector recorded (F8) | grep ODD | serious | design-notes F8 | **done** | ODD-0025 **§2.8**. reproduced (operator) | |
| F-11 | Frontmatter-fidelity mapping table; orphan cells (`author`, `version`) resolved | ODD has the table; every legacy field has a target-or-rationale | serious | arc-plan (property 3) | **done** | ODD-0025 **§2.4** — `author`/`version` **promoted to typed document-node fields** (§2.2), not dropped. **operator-confirmed** | Expansion beyond the plan — see closing-report bubble-up. |
| F-12 | 0013/0020 amendment spec section, by section | grep ODD for "Amendments required" citing 0013 §x / 0020 §y | serious | slice-doc | **done** | ODD-0025 **§4** (0013 §2.2/§2.3/§3/§9; 0020 §2/§4). reproduced (operator) | Specification only; s03 applies. |
| F-13 | Naming disambiguation line (`docs/dev/research/` vs `docs/dev/` vs `research` type) | grep ODD | polish | design-notes note | **done** | ODD-0025 **§3**. reproduced (operator) | |
| F-14 | No decided-fork drift: design-notes §3 (F1-F3,F5-F8) reflected without silent change | cross-read design-notes §3 vs ODD-0025 | correctness | LEDGER-DISCIPLINE | **done** | Cross-read: all decided forks reflected. The two intentional changes (`provenance`→`source`; author/version typed) are **tracked** in ODD-0025 v1.1 + this ledger, not silent. reproduced (operator) | |

## What Worked

- **Reading 0013 §2.2/§2.3/§3 before writing the amendments** caught two things a summary would
  have missed: the `provenance`=derived-lineage reservation (forcing the `source` rename) and the
  existing `note` type (forcing the artifact-vs-note decision to be explicit, not assumed).
- **Surfacing the open forks to the operator with reasoned proposals** (rather than pre-baking or
  blocking) turned three "confirm" rounds into a single agreement, and the operator's push on
  `author`/`version` corrected a real drop before it shipped.

## Closure

Closed 2026-07-27. Deliverable **ODD-0025** Accepted. Verified by: operator gate (Duncan,
all §7 decisions confirmed) — CDC authored, so not self-verified. Rows: 14. Done: 14. Deferred: 0.
No-op: 0. Bubbled up to `../arc-plan.md` (v1.3): s03's scope expanded (type `source`/`author`/
`version`); capability property 2, exit criteria, MF-3, MF-6 reworded `provenance`→`source` +
F10 mint-all.
