# CDC Verification — Arc 02 / Slice 06: `check` v2

> Independent verification of CC's closed ledger (impl `8022bc5`; closed `1164ea0`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 10 opened, 10 addressed. No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
v1 structural checks remain in `odm-core::check` (link-integrity over all 8 edge
kinds, supersession); v2 **aggregation + `Severity` (Error always fails, Warning
fails only under `--strict`) + exit codes (`EXIT_OK`/`EXIT_VIOLATIONS`) + `--json` +
errors-as-affordances** live in `odm-cli::commands`; the graph predicates
(cycles/staleness/recomposition/satisfaction) are **folded in, not reimplemented**
(odm-core's check doc explicitly defers them to the graph/recompose/satisfaction
modules); no `unsafe`. → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
the 11 check tests + full workspace; clippy; coverage line 93.69% (recompose.rs
100%, commands.rs 90.03%). → **PENDING CI.**

## Rulings on CC's flagged items

1. **`assert_cmd` deviation — exit codes verified in-process via `dispatch`'s `u8`.**
   **Accepted** (the L-6 precedent). *Optional hardening, tracked:* a binary-level
   `assert_cmd` suite in `oxur-odm/tests/` would verify the real process's `ExitCode`
   end-to-end (today only the `u8`→`ExitCode` mapping is unit-tested). Nice-to-have.
2. **Recomposition findings classified as errors.** **Accepted as a conservative
   default — but I recommend a recalibration.** `Severity` already treats staleness
   and soft-satisfaction as *Warnings* (surface, don't block; fail only under
   `--strict`) — the evidence-leveled philosophy. By that logic: **orphan** and
   **decomposition-drift** are genuine Errors (a structural break / a now-false
   assertion), but **undeveloped-stub** and **advanced-without-decomposition** are
   advisory and should be **Warnings** (promoted in `--strict`). Otherwise everyday
   `check` fails (exit 1) on any arc advanced before its decomposition is affirmed —
   too aggressive for normal work. Quick decision; small tweak if you agree.
3. **Tear rationale synthesized (no rationale enforced/shown yet).** **Accepted** —
   completed by slice 07's `odm tear` (rationale-required), which this slice's
   affordance already points to.
4. **v1 check tests + JSON envelope updated for v2 (orphan now catches a parentless
   arc; envelope gained severity/code/counts).** **Accepted — disclosed, expected**
   evolution (v2 extends v1). *Note:* `check --json` is now the **v2 schema**; any
   future consumer (export/interop in ODD-0017) should target it.
5. **Affordances name not-yet-built commands.** `odm tear` → slice 07 (✓ in scope).
   **But the decomposition-affirmation command is NOT yet in slice 07's scope** —
   `affirm_decomposed` exists in odm-core with no CLI surface, yet `check` tells the
   user to affirm. **Folding an `odm decomposed`/affirm command into slice 07 this
   turn** so the affordance has a real command.

## Layering note (minor)

The v2 aggregation + severity *policy* lives in `odm-cli`, not `odm-core`. Fine for
the MVP (check is a command). If a library consumer (reconcile A5, export 0017)
later needs a programmatic aggregated check, lift `aggregate()` into odm-core then.

## Verdict

Arc 02 / Slice 06 **CDC-verified on structure; all flags accepted; cargo rows
pending CI.** `odm check` is now the single mechanical gate. **Arc 02's *engine* is
complete** — but the arc closes (and odm becomes self-host-usable) only with **slice
07 (CLI graph-mutators)**, which I added after CC's slice06 and just extended with
the decomposition-affirm command. On CI green, slice06 closes; slice07 is queued.

CDC: planning thread, 2026-06-22. Iterations used: 1.
