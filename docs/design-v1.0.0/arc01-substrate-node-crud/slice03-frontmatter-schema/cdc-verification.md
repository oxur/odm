# CDC Verification — Slice 03: Frontmatter schema + round-trip

> Independent verification of CC's closed ledger (impl `06662af`; closed `7de0ac3`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here.

## Environment constraint (disclosed)

Same as prior slices: the Cowork sandbox has no 1.85+ Rust toolchain, so cargo-gated
rows are routed to CI / a local run.

## Row dispositions

**Row count:** 10 opened, 10 addressed. No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
I-8 (`serde_norway` appears only in `crates/odm-core/src/frontmatter.rs` — grep
clean; pinned in workspace deps with a rationale comment), I-9 (no `unsafe` in
`odm-core/src`). → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
I-1 (parse/typed errors), I-2 (core fields), I-3 (edges block), I-4 (**round-trip
proptest**, 128 cases over arbitrary fields+tags+edges+multi-line body), I-5
(unknown-key preservation — literal + proptest), I-6 (canonical-order snapshot),
I-7 (`supersedes` kind), I-10 (clippy + 98.79% region / 99.61% line coverage). → **PENDING CI.**

I called out two watch-items at handoff; both are satisfied by the evidence: I-4 is
a genuine *arbitrary-node* proptest (not hand-picked) including unknown keys, and
I-6's evidence pins the actual emitted field list, not just "test passed."

## Rulings on CC's two flagged items

1. **YAML lib → `serde_norway` (the designated fallback).** **Accepted.** CC ran
   the maintenance check, found `serde_yaml_ng` frozen at 2024-05 (>12mo), and used
   the slice-doc's named fallback. The real caveat — the *whole* serde_yaml fork
   ecosystem has been quiet since late 2024 — is fair, but `serde_norway` is the
   freshest option and is **fully isolated behind `frontmatter.rs`** (one-file
   swap). The 12-month bar was a heuristic, not a hard gate; a stable, isolated,
   quiet serializer is acceptable. *Logged as a future option (not blocking): if
   the fork ecosystem stays frozen, consider a lower-level maintained parser
   (`yaml-rust2`) or hand-rolled emit — an amendment, cheap because of the
   isolation.*
2. **ODD-0013 §2.3 vs §3 edge-ordering inconsistency + §2.3 omits `affects`.**
   **Valid finding — fixed.** This is a real internal-doc inconsistency I
   introduced: adding the `affects` edge in 0013 v1.4 (§3) *affected* the §2.3
   example, and I didn't update it. Reconciled in **0013 v1.6**: `affects` added to
   the §2.3 example, §3 table reordered to match, canonical emission order noted.
   (Notably, this is exactly the C5 / stale-doc class our `affects` edge + check is
   designed to catch — caught here by CC because the tool that would catch it isn't
   built yet. Dogfooding the failure mode.)

## Verdict

Slice 03 **CDC-verified on structure; both deviations accepted (one → a doc fix);
cargo rows pending CI.** On CI green, slice 03 is fully closed and slice 04 (store
layer) opens.

CDC: planning thread, 2026-06-22. Iterations used: 1.
