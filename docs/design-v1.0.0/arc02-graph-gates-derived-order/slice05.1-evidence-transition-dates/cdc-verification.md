# CDC Verification — Arc 02 / Slice 05.1: Evidence-transition dates

> Independent verification of CC's closed ledger (impl `5c238a8`; closed `26f6e50`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here. An *inserted* slice (not in the
> original arc02 plan) — see "Provenance & numbering" below.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 8 opened, 8 addressed. No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
`GateRecord.evidence_dates: BTreeMap<Evidence, NaiveDate>` (evidence-ordered),
`#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]`; `set_gate` records
via `.entry(evidence).or_insert(reached)` → **first-reach only** (a raise inserts
the new level, preserves earlier levels' dates); no `unsafe`. → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
the 5 status tests incl. the 128-case populated round-trip proptest and the
**byte-identical back-compat** round-trip against literal pre-field YAML; clippy;
odm-core line 97.88% (status.rs 94.23%). → **PENDING CI.** Back-compat held exactly
(a pre-field node parses to an empty map and re-emits byte-identically) — the right
invariant for adding a field to a live schema.

## Rulings on CC's flagged items

1. **Evidence regression recorded as first-reach, not flagged** — **Accepted**, and
   correctly routed to the telemetry arc (A7) which owns regression semantics.
2. **`reached` still overwrites on a raise; durable history now lives in
   `evidence_dates`** — **Accepted.** `reached` = "current reach," per-level history
   is the new map. Clean separation.
3. **Signal captured, not consumed** — **Accepted.** Latency math / rollup surfacing
   is A7. This slice is deliberately *data capture only*.

## Provenance & numbering (a gentle, on-point observation)

This slice was **inserted** (`a9b61e6` "Added new slice") and numbered **`05.1`** —
a bisection between 05 and 06. That is, precisely, the **`Phase 8.5` anti-pattern**
from ODD-0001 (A2: ad-hoc bisection numbering; identity doubling as sequence) that
`odm` exists to make impossible. Worth naming with a smile: it happened *in odm's
own construction*, during the **markdown-bootstrap phase** where slices are
dir-named and order is encoded in the name. It's the cleanest possible argument for
the tool — once we self-host (post-A3), slices get ULIDs + derived order and a
"05.1" wedge is structurally unrepresentable. **No rename demanded** (churn);
logged as the motivating irony. The work itself is sound and well-placed.

**Why the insert is *good* placement:** `evidence_dates` is the verification-latency
signal the **forecasting/telemetry arc (A7)** needs (two-clock telemetry +
Monte-Carlo-PERT, per the project's telemetry direction). Capturing it now — while
touching `status.rs` — is cheaper than retrofitting later, and back-compat means no
disruption. Spec-keeping: arc02 has grown this slice beyond ODD-0015's A2 sketch;
the arc-plan slice list is updated to reflect it.

## Verdict

Arc 02 / Slice 05.1 **CDC-verified on structure; all flags accepted; cargo rows
pending CI.** First concrete groundwork for the A7 telemetry/forecasting layer. On
CI green it closes and slice 06 (`check` v2) opens — which now aggregates three
predicate sets (satisfaction/staleness, recomposition, structural v1).

CDC: planning thread, 2026-06-22. Iterations used: 1.
