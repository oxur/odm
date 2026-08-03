# Slice 01 -- CDC verification (ODD-0026)

**Method:** LEDGER-DISCIPLINE v2.0 sec. A, design-slice variant -- "verify" means the decision is
**recorded and coherent** and **reconciled** against the corpus, not that code runs. **Verdict:
PASS.** Verified 2026-08-03.

## Independence note (disclosed)

This is a design slice CDC authored jointly with the operator, so CDC is not fully independent of
the *authoring*. The independence that gates it comes from two places kept honest: (1) the
**operator ratified each fork** in the decision session and **accepted** the written document --
the decision is not self-approved; and (2) the reconciliation claims below were checked against the
**accepted ODD-0025 on disk**, not from memory. Where I cannot be independent (I wrote the prose),
I say so rather than dressing it up as an external review.

## Checks

- **D1-1 -- reproduced.** `docs/design/04-accepted/0026-store-as-source-model.md` exists,
  frontmatter `state: Accepted`, `version: 1.1`; the file was moved from `01-draft/` and the intro
  state line + v1.1 history entry reflect acceptance. Direct file read.
- **Forks recorded (D1-2..D1-6) -- reproduced.** sec. 2.1-2.5 each carry a decision + rationale;
  sec. 6 restates all five as ratified. Fork A explicitly records the overturn of the A1 lean (not
  a silent change). Read and confirmed.
- **Reconciliation (D1-7) -- reproduced against the source.** The two ODD-0025 statements fork B
  depends on -- "migration-time gate only" and "no hash is stored" -- were grep-confirmed verbatim
  in `04-accepted/0025-migration-fidelity-model.md` (sec. 2.1). The `origin` value set
  (`planned`/`discovered`/`amendment`) fork A extends was confirmed in ODD-0025 sec. 2.0. So fork A
  (add `origin: authored`) and fork B (N/A by construction) sit inside 0025's model rather than
  contradicting it -- the reconciliation is real, not asserted.
- **Amendments scoped, not applied (D1-8) -- reproduced.** sec. 3 specifies ODD-0013 + ODD-0025
  changes as *specification only*, applied by slice 02 / dedicated amendments. No amendment was
  silently applied here (checked: 0013 and 0025 on disk are unchanged by this slice).
- **No-regression (D1-9) -- reproduced.** sec. 2.2 carries the clause; the gate keying off external
  `source.paths` means migrated-node fidelity is untouched. Coherent.
- **Silent-drop check -- clean.** 9 rows open, 9 rows closed; all forks present; scope-out honored.

## Verdict

**PASS.** ODD-0026 is Accepted, internally coherent, and reconciled with the corpus it touches.
The one substantive movement (fork A) is recorded as a correction across all four docs. Slice 01
closes; arc **SS-1 -> done**. Real-world validation of the *model* comes downstream: slice 02
proves a self-sourced node passes `check` (SS-2), and the arc DoD (SS-6) is closed by a fresh
context -- neither is a slice-01 claim.
