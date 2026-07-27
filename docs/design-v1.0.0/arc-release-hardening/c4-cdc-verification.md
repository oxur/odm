# RH C-4 (command surface + ODD-0023 reorg) — CDC verification

> **Verifies:** RH-4 · F-10/F-11/F-12/F-13 + ODD-0023 (Accepted v1.1) · **Landed:** `release/1.0.x`
> @ `50d72bc` (linear: ODD-0023-Accepted `d525618` → reorg `7786dc4` → close `50d72bc`) · **Date:**
> 2026-07-27 · **Verifier:** CDC, independent of CC — clean container clone, store worktree
> materialized, built debug binary, exercised on the **self-hosted store**.

## Verdict

**C-4 verified — clean.** The three-tier surface, the four renames, the hard cut, and the
validate/check inversion all shipped correctly and reproduce against the running binary. Phase 0 was
honoured (ODD-0023 → Accepted *before* the code). No defects in C-4's own work. **The two
operator-directed departures from my plan are both sound — and, in this context, better than the
defaults I'd recommended.** I say that as an assessment, not deference; the reasoning is below.

## Reproduced by CDC

| Check | Result |
|-------|--------|
| **Three tiers (parity 11/11/2)** | Top-level workflow verbs: `orient` `rollup` `next` `blocked` `chain` `project` `use` `validate` `check` `reconcile` `migrate` (+ `node`/`store`/`help`). `odm node`: `new list show rename retire supersede link unlink set-gate decomposed tear` (11). `odm store`: `init rename` (2). Matches the inventory. |
| **Renames (F-11/F-12)** | `odm project` shows current project/arc; `odm chain 1600` works; `odm context`/`odm path` are **gone**. |
| **Quiet `new` (F-10)** | help: "idempotent: re-running describes rather than duplicating" — the warn-not-dump behaviour. |
| **`rollup` (F-13)** | help: "the single cheap view of the whole plan" — no hardcoded `ROLLUP.md`. |
| **Hard cut is clean** | `odm context` / `odm path` / `odm show` / `odm list` → clap's `error: unrecognized subcommand '<x>'` — a clean, standard error, not a crash. No deprecation aliases (deliberate — see below). |
| **`use` kept** | top-level; `use arc 1600` → `orient` CURRENT FOCUS shows arc #1600 (the C-5 fix **survives the reorg** — verified on the self-hosted store, which is the only place the root split ever hid). |
| **validate = pure** | `odm validate --json` → `schema: validate/v1`, static only (schema/links/cycles/recomposition/order). |
| **check = validate + reconcile** | `odm check --json` → `schema: check/v2` with nested `validate` + `reconcile` blocks + `reconcile_skipped: false`. Runs validate then reconcile, stopping at validate errors. |
| **Defect fixes (parity-caught)** | (1) the phantom `store sync` command is gone — ff-sync is an arm of `init` (correct per ODD-0022 §6); (2) `store init --help` now describes all three arms (bootstrap/attach/ff-sync), not "bootstrap only"; (3) `odm self-host` → `unrecognized subcommand` (the dead CLI wrapper removed; the library derivation `migrate` calls is untouched). |
| **Tests** | `odm-cli` 25, `odm-migrate` 61, store suites (8/12/10/11/22/4/11/6) — **all 0 failed** in my clone. CC reports 58 binaries / 0 failed, clippy `-D warnings` clean, fmt clean, no `unsafe` on the full workspace. |

## The two operator-directed changes — assessed, both endorsed

**1. Hard cut, not a deprecation-alias window.** My plan recommended one release of hidden deprecated
aliases. **Duncan's hard cut is the right call here, and I'd revise my recommendation.** The
deprecation window earns its keep when you have *external consumers mid-stream* to protect. odm is
**pre-1.0.0, single-maintainer, unpublished** — the only consumers of the old spellings are odm's own
tests and muscle memory. A hard cut buys a **clean 1.0.0 surface with no alias cruft** to carry and
then remove; the ~290 test-invocation edits are a one-time bill paid now instead of a
deprecation-plus-removal chore spread across two releases. My recommendation was the general-case
conservative default; this was the context-appropriate call.

**2. validate/check inverted.** My plan had `check` pure with an opt-in `check --reconcile`. **Duncan's
model — `validate` = the pure static pass, `check` = "validate, then reconcile" — is cleaner**, and I
endorse it: two honest verbs instead of a flag, and `check` now *means* "check the plan against
reality" (the fuller thing) rather than being the cheap path wearing the authoritative name. The cheap
path is simply `validate`. The detail CC decided in passing is the best part: when a validate **error**
halts the run, the reconcile half is `null` with an explicit **`reconcile_skipped: true`**, so a
machine consumer **cannot read "no drift" where the truth is "we never looked."** That is exactly the
both-halves honesty the arc keeps rewarding — a skipped check that advertised success would be the
`check`-over-invisible-corpus failure in a new place. (Verified the flag is present and `false` on the
green corpus; the `null`+`true` path is the validate-error case.)

## Worth noting: the parity check caught *my* error

The mechanical inventory↔`--help` parity (11/11/2, MATCH) caught that **the command inventory I
rewrote documented a `store sync` command that was never built** — ff-sync is an arm of `store init`,
per ODD-0022 §6, which I even flagged at the time as the "store-sync inventory reconciliation." The
doc drifted from the code and a *reading* review (mine) missed it; a *mechanical* parity check caught
it. Good argument for the mechanical check, and a fair mark against my inventory pass — now corrected,
with the built help carried verbatim in the inventory Appendix.

## Forward (recorded, non-blocking)

- **`store status`** stays open in ODD-0023 §7 — explicitly non-blocking (nothing depends on it).
- **`node tear` rationale** is still validated-but-not-persisted — a **pre-existing** gap (this is
  **G-2**), now sitting visibly under the `node` group. Its fix is **C-6** (the check-hardening bundle:
  G-2 + G-3 + L-3b). Correctly not pulled into C-4.

## Ledger

- **RH-4 → attested** (CDC-reproduced on `release/1.0.x`; durable `reproduced` rides the push + CI).
  F-10/F-11/F-12/F-13 **done**; ODD-0023 **Accepted (v1.1)** with the three reversals recorded as
  reversals. **Silent-drop diff:** none — `store status` and `tear`-rationale are recorded forward.
- **RH remaining:** **C-6** (check-hardening) and **C-8** (normalized status). Then RH-6/RH-7 compose +
  arc close → A6 resumes at slice05.
