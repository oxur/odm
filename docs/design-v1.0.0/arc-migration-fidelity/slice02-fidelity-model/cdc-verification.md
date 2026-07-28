# Slice 02 verification — The fidelity model (ODD-0025)

> **Arc:** Migration Fidelity · **Slice:** 02 (model) · **Deliverable:** ODD-0025 (Accepted)
> **Date:** 2026-07-27 · **Verdict: PASS — closed.**

## Note on the seat and the gate

This slice was **authored in the CDC seat** (operator decision, per the "agreed about ODD-0025"
call). LEDGER-DISCIPLINE requires the closer to be structurally separate from the verifier; here
that separation is provided by the **operator (Duncan)**, not a self-check. The operator reviewed
the model and **confirmed every open decision on 2026-07-27**:

- **F4** normalization = `trim + CRLF→LF` — confirmed.
- **F9** `artifact` — new document node type (American spelling), nearest-modeled-scale
  containment, no `chunk` scale — confirmed.
- **F10** report coverage — **mint-all** (every report incl. `coverage-report.md` gets an
  `artifact` node; no exemption) — confirmed.
- **`author` / `version`** — preserved as new typed document-node fields (not dropped, not
  git-derived) — operator-corrected and confirmed.
- **Naming** — the stored migration record is **`source`**, not `provenance` (0013 reserves
  `provenance` for derived lineage) — confirmed.
- **Timing** — `source`/`author`/`version` typed at creation in s03 — confirmed.

## Verification

The ledger's 14 rows are each satisfied by a section of ODD-0025 (see `ledger.md` Evidence
column); a read of the ODD against the ledger confirms every decision is recorded unambiguously,
the frontmatter-fidelity mapping is complete (every legacy field has a target or a
disclosed-with-rationale disposition), and the ODD-0013/0020 amendment section cites concrete
sections. The two decisions that changed from the draft proposal (`provenance`→`source`;
`author`/`version` promoted to typed fields) are **tracked** in ODD-0025's v1.1 Version History and
the ledger — not silent drift.

**Disposition:** ODD-0025 Accepted; s02 ledger 14/14 done; bubble-up applied to `arc-plan.md`
(v1.3). The model is adequate for s03 to implement against. **Closed.**
