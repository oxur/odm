---
id: 01KWXMBBTK54PMJWBANX7BN5V8
number: 1505
type: slice
schema: slice/v1.1
name: '`affects` edge + stale-doc-vs-decision check (C5)'
created: 2026-07-02
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc05-reconciliation/slice05-affects-stale-doc/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KWXMBBTKQ3QE7FGTM80MYNDH
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
# Slice 05 (Arc 05): `affects` edge + stale-doc-vs-decision check (C5)

> Plan-of-record for A5 slice05. Independent of the freshness rework (07/08). Cashes
> ODD-0001 **C5**: a doc that no longer reflects a decision that governs it should be
> **flagged**. The `affects` edge already exists (A2); this slice adds the **check
> semantics** that use it.

## Goal

`odm check` flags a **potentially stale doc**: when a committed decision `A affects` a doc
`B`, and `A` was updated *after* `B`, surface B as possibly not reflecting A. This is the
C5 failure — a decision moves, the doc it governs quietly falls behind.

## The honest detection (structural/temporal, never semantic)

odm does **not** detect contradiction by understanding content — that would be confabulation
(the same boundary recomposition-integrity drew: no automatic *semantic* scope detection).
The signal is **structural + temporal**:

- **The `affects` edge is the author's assertion that A is a committed decision governing B.**
  Declaring `A affects B` *is* the "this decision governs this doc" claim — so the edge itself
  carries the "committed decision" meaning (ODD-0001 C5's word). No separate "committed"
  gating is needed; the edge is the commitment.
- **Staleness = `A.updated > B.updated`** — the governing decision moved after the doc it
  governs, so the doc *may* not reflect it. The check flags B as **potentially stale relative
  to A**, with both identities + dates. It never claims B *contradicts* A — the human judges.

This mirrors A2's staleness guard (out-of-order `updated`), applied across the `affects` edge.

## Scope — in

1. **`check` gains a stale-doc finding** (`odm-core::check`): for each `A affects B` with
   `A.updated > B.updated`, emit a `stale-doc` finding naming A, B, and both dates.
2. **Severity = Warning** (potential staleness; human judges), consistent with `check`'s
   Error/Warning tiers — fails only under `--strict`, exactly like the other advisory
   findings. Not an Error (we assert *possible* staleness, not a proven defect).
3. **No false positive**: a doc updated at-or-after its governing decision (`B.updated >=
   A.updated`) is **not** flagged.
4. **Render + `--json`**: the finding appears in `check`'s human output and in the `check`
   `--json` findings list — **additively** (a new finding kind in the existing list; **no
   check schema version bump**, same as slice04's drift-additive).

## Index / invariant note (no new field — invariant honored by non-triggering)

`check` is already index-backed (A4/slice06), and both signals this slice needs —
`affects` (edges) and `updated` — are **already** in the `IndexRecord` + adapter
(`odm-index/src/adapter.rs:95`, `record.rs:101`) and already read by `check` (the
dangling-ref pass). So slice05 adds a **new use of already-indexed fields, not a new
field**: **no adapter / fidelity-test / `FORMAT_VERSION` change**, and the A4
adapter-fidelity invariant stays honored by non-triggering. (A light ledger row guards this.)

## Scope — out (named, not dropped)

- **Deferred surfacing + re-entry predicate** — **slice06**.
- **The freshness rework** (probes-as-rules, drift snapshot, every-command freshness) —
  **slices 07/08** (ODD-0019). Independent of this slice.
- **Semantic contradiction detection** — **out, permanently** (a non-goal: human judgment,
  not a fake automated check).
- **Committed-state gating beyond the edge** — the `affects` edge *is* the commitment (see
  above); a future tightening could additionally gate on the decision node's status/gate, but
  that's not needed for C5 and is not in scope.

## Verification approach

`odm-core` unit tests + `odm-cli` integration tests:

- `A affects B`, `A.updated > B.updated` → a `stale-doc` Warning naming A, B, both dates.
- `B.updated >= A.updated` → **not** flagged (no false positive).
- the finding renders in `check` human output + `--json` (additive, no schema bump).
- `--strict` promotes it to a failing exit; without `--strict` it's advisory (check-consistent).
- the check reads `affects`/`updated` off the index; no adapter/`FORMAT_VERSION` change.
- clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

Cargo rows are CC-`attested`, reproduced via CI / local 1.85+.

## Exit criteria

Ledger T-1…T-6 reach a final status: `check` flags a potentially-stale doc across the
`affects` edge on the temporal signal (structural, not semantic; the edge = the committed
decision); Warning severity, no false positives, `--strict`-gated; `--json` additive; no new
index field (invariant non-triggered); gates pass.

> **Render convention (slice03 finding):** `check` renders with `writeln!` + `tabled` (odm-cli
> has **no** `oxur-cli` dep). Use that; do not reach for `oxur_cli`.
