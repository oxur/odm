---
id: 01KYNDTQ6S2CXBMG4KMCW702M1
number: 58837402
type: slice
schema: slice/v1.1
name: Slice 02 (Migration Fidelity) — The fidelity model (ODD-0025) (plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice02-fidelity-model/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
status:
  built:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  planned:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  tested:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
---
# Slice 02 (Migration Fidelity) — The fidelity model (ODD-0025) (plan-of-record)

> Refs: `../design-notes.md` §2–§3 (the decisions + the open forks this slice records);
> `../arc-plan.md` (s02 row + the arc capability's four properties); `slice01-coverage-discovery/`
> (the exact inventory this model must be adequate to repair). `depends_on:` s01 (the work-list).
> Amends: **ODD-0013** (node model) and **ODD-0020** (schema markers). Registers: **ODD-0025**.
>
> **Seat note.** This is a **model / design slice** — its deliverable is a design document, not
> code, and it *mints no nodes*. The decisions it records are **CDC + operator** work (captured in
> `design-notes.md` §3 and ratified by the operator); the frontmatter-fidelity schema mapping in
> particular carries real design judgment. Recommended: author in the **CDC seat with operator
> review**. The `cc-prompt.md` is written so it *can* be handed to CC to author from a
> fully-specified spec, but whoever authors it makes **no new design calls** — amend, don't
> work around.

## Goal

Author **ODD-0025 — Migration Fidelity: provenance, synthesis, artifact nodes & frontmatter
fidelity** (Draft, under `docs/design/01-draft/`), the single authoritative model the arc's
implementation slices (s03 fidelity-core, s04 scope+repair, s05 enforcement, s06 synthesis)
build against. The ODD (a) **formalizes the decided forks** F1/F2/F3/F5/F6/F7/F8 from
`design-notes.md` §3, (b) **records rulings** on the still-open forks **F4, F9, F10**, (c)
specifies the **frontmatter-fidelity schema mapping** over originally-present fields, and (d)
names the **concrete amendments** it requires of ODD-0013 and ODD-0020, precisely enough that s03
applies them without re-deriving the model. **Done when** ODD-0025 exists, registered and Draft,
records every decision below unambiguously, contradicts no decided fork, and enumerates its
0013/0020 amendments by section.

## Scope

**In:**

- **ODD-0025**, authored to the existing ODD format (frontmatter matching a sibling, e.g.
  `0024`; `state: Draft`; next number = 25 confirmed against `docs/design/`). It records:
  1. **Provenance sub-map** (F5-decided): `source_paths` (list, 1+), `source_class`,
     `normalization`, `migrated_by` (tool+version), `migrated_on` — **computed at migration time,
     no stored hashes**. Lands additively via `Frontmatter.extra` (`#[serde(flatten)]`) before
     being formally typed.
  2. **Body definition + no-transform rule** (F3-decided): body = post-frontmatter (FM docs) /
     whole file (FM-less corpus); the importer synthesizes **no** `# {name}` H1 and injects no
     headers.
  3. **Body-hash gate** (F2/F5-decided): hard-fail when
     `sha256(normalize(source_body)) != sha256(normalize(node_body))`; migration-time only,
     nothing re-verified later.
  4. **Normalization boundary** (**F4 — CDC-proposed, confirm**): `normalize` = trim
     leading/trailing whitespace **+ CRLF→LF**, and nothing else — internal changes still fail
     the gate; line-endings are a platform artifact, not content. `normalization: trim+lf`
     recorded in provenance so the hash stays interpretable.
  5. **Synthesis model** (F1/F2-decided): `edges.supersedes` → **`Vec`**; reverse
     `superseded_by` **derived, tooling-guaranteed** on every synthesis (+ a check); synthesis
     type {`concatenation` hash-gated | `editorial-merge` attested | `other`}; two axes kept
     (synthesis-type = *how merged*; `SupersedesKind` Obsoletes/Updates = *what it does to the
     target*).
  6. **`artifact` (supporting-doc) node class** (**F9 — CDC-proposed, confirm**): one class for
     `ledger`/`cc-prompt`/`cdc-verification`/`closing-report`/ADR/amendment/UAT, distinct from
     work nodes, `part_of` its **nearest modeled scale** (per-slice → slice; arc-level &
     chunk-level → **arc**). **No "chunk" node scale is introduced** — the project/arc/slice
     vocabulary is constant by design.
  7. **Report self-coverage disposition** (**F10 — CDC-proposed, confirm**): authored reports
     (`closing-report`, `cdc-verification`) are ordinary `artifact` nodes (so "no file left
     behind" stays literally true; per-run body changes are fine under F5). The **regenerable**
     `coverage-report.md` gets an explicit **coverage-exemption** (an ignore rule) since it is
     derived tool output, not authored source. *(This is the one sub-cell most wanting operator
     confirmation.)*
  8. **Containment-optional for design/research nodes** (F7-decided): doc nodes may be top-level
     *or* `part_of`; the coverage/orphan check must not flag a legitimately top-level doc node.
  9. **Update-in-place fix vector** (F8-decided): match existing node → source by structural
     coordinate (provenance absent at repair time), rewrite body + provenance into the same node,
     hard-fail the gate, preserve id/edges/status; a stub is decidable by "body is a lone H1."
  10. **Frontmatter-fidelity schema mapping** — a versioned, table-form mapping checked over
      **only the fields the source actually had**. Must cover the legacy ODD fields
      (`number`→`number`, `title`→`name`, `created`/`updated`/`tags`/`component` direct,
      `state`→cumulative gate reach per the already-faithful `mapping.rs::reach_cumulative`,
      `supersedes`→`edges.supersedes`, `superseded-by`→derived) and **resolve the two orphan
      cells** — `author` and `version` have no node field today (CDC lean: `author`→a provenance
      or metadata field; `version`→dropped-with-rationale or a tag — the ODD decides, operator
      confirms).
  11. **Naming disambiguation** — one line distinguishing `docs/dev/research/`, `docs/dev/`, and
      the `research` node type (per the `design-notes.md` §3 note).
- **The 0013/0020 amendment specification** — a section of ODD-0025 naming, by section, what
  ODD-0013 (node model: add `provenance`; add the `artifact` node type/class) and ODD-0020
  (schema markers: `provenance` + `artifact` schema versions) must gain. **Specification only**
  — the physical edits to 0013/0020 land with s03's implementation, not here.

**Out:** any **code** (s03+); any **minting / migration / schema change to the store** (s03–s05);
applying the 0013/0020 edits to those files (s03); the L-8b state reconciliation of
0013/0017/0018 (s06); wiring coverage into `check` (s05); the synthesis *implementation* (s06 —
s02 only specifies the model). If drafting reveals the frontmatter-fidelity mapping is large or
contentious enough to threaten the one-context budget, **split** s02 into `provenance+synthesis
model` and `frontmatter-fidelity mapping` — decided at execution per the sizing rule, not
pre-committed.

## Verification

A design deliverable is "reproduced" by an independent reader confirming each decision is stated
unambiguously and consistently — not by running code. CDC (or a fresh context / the operator)
reads ODD-0025 against `ledger.md` (F-1…F-N), confirms every decided fork in `design-notes.md` §3
is reflected without contradiction (spec-keeping: no decided item silently changed), confirms the
three open-fork rulings match what the operator confirmed, and confirms the 0013/0020 amendment
section is concrete enough for s03. No `cargo`/code rows in this slice.

## Exit

`ledger.md` closed; CDC-verified (`cdc-verification.md`), independent read reproduced. ODD-0025
committed under `docs/design/01-draft/`, Draft, as the arc's authoritative fidelity model. On
close, bubble up to `../arc-plan.md` — confirm s02 delivered the model MF-2/MF-3/MF-4/MF-7 and the
enforcement (MF-6) plan against, and record any fork the operator re-ruled during authoring.
