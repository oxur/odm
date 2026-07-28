---
id: 01KYNDTQ6SABSXWZ58M8TQK3G3
number: 58837401
type: slice
schema: slice/v1.1
name: Slice 01 (Migration Fidelity) — Coverage discovery (plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc-migration-fidelity/slice01-coverage-discovery/slice-doc.md
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
# Slice 01 (Migration Fidelity) — Coverage discovery (plan-of-record)

> Refs: `reconciliation-audit-2026-07-27.md` (the discovery); `../arc-plan.md` (the arc
> ledger MF-1/MF-5 + the capability); `../design-notes.md` (the dimensions + root causes).
> Reuses `odm-migrate/src/selfhost.rs` (the arc/slice → coordinate/number logic) and
> `mapping.rs` (ODD number/title classification). `depends_on:` A6·s04 (the self-hosted
> corpus this reports on).
>
> **Why this slice exists:** every downstream slice — the model (s02), fidelity core (s03),
> scope+repair (s04) — is scoped against *how many* holes and *which* ones. Today we have the
> audit's estimates (≈44 stubs, ≈211 uncovered docs, 5 missing arcs) but **not an exact,
> re-runnable count**. This slice builds the detector and produces the authoritative
> inventory. It is **read-only** — it mints no node, changes no schema, fixes no hole — so it
> is the safe first step, and its output is the work-list the arc plans against.

## Goal

Build a **read-only coverage/gap detector** in `odm-migrate` (a `coverage` module + a
read-only `odm migrate --coverage` reporting mode), run it over odm's own `1.0.x/docs`
corpus, and produce **`coverage-report.md`** — the exact, re-runnable inventory across four
dimensions. **Done when** the detector enumerates every `.md` under the docs roots, matches
each to a node (or reports it uncovered), emits per-class counts + lists for the four
dimensions, and the report is committed as the arc's work-list.

## Scope

**In:**

- **The `coverage` module** in `odm-migrate`: enumerate + classify every source `.md` under
  the docs roots (`docs/design-v1.0.0/` planning corpus, `docs/design/` ODDs, `docs/dev/`,
  and the research docs), and match each to a store node — **heuristically**, by structural
  coordinates for the planning corpus (reuse `selfhost.rs`) and by number/title for the
  frontmatter docs (reuse `mapping.rs`), since provenance does not exist yet (s03 lands it).
- **Four detectors**, each emitting counts + the offending list:
  1. **doc-coverage** (inverse `orphan`) — every source `.md` has a node; list the uncovered by class.
  2. **representation** — arc/slice **directories** vs. arc/slice **nodes**; name the gap (the 5 arcs `self_host`'s `arc_in_scope` A1–A6 excludes, and any missing slices).
  3. **stub bodies** — nodes whose body is ≤ 1 non-blank line (a lone synthesized H1); list them.
  4. **provenance-absence** — nodes carrying no `provenance` field; count them.
- **A read-only `--coverage` mode** on `migrate` (sibling to `--dry-run`): runs the detectors and writes/prints the report. Mints nothing; touches no node.
- **Tests** for classification, each detector, and the read-only guarantee.
- **The produced `coverage-report.md`**, committed in the slice dir.

**Out:** any minting / migration / re-migration / schema change (s02+); provenance
persistence (s03); the **body-hash equality** check (needs re-deriving the source body — s03,
not here; this slice flags *stubs*, the obvious body-fidelity case, only); wiring coverage
into `check` / `validate` as an enforced gate (s05); **fixing** any hole (the rest of the arc).

**Heuristic caveat (state it in the report):** until provenance lands (s03), source↔node
matching is structural/heuristic — a genuinely unmatched doc *may* be a matching-logic miss
rather than a true hole. The report marks its matching basis; s03 makes it exact.

## Verification

`cargo test -p odm-migrate coverage` green (classification, four detectors, read-only
guarantee); `odm migrate --coverage` runs read-only and emits the report; the report's
headline counts **reconcile with the audit's ballpark** (≈44 stubs, ≈211 uncovered, 5 missing
arcs) — any divergence is investigated and explained, not silently accepted; `cargo clippy
-p odm-migrate -- -D warnings`; no `unsafe`; coverage ≥ 90% (line) for the new module. Rows in
`ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows `attested` → `reproduced` via CI / local 1.85+
per LEDGER-DISCIPLINE v2.0). `coverage-report.md` committed as the arc's authoritative
work-list, so s02 (model) and s04 (scope+repair) plan against exact numbers rather than the
audit's estimates.
