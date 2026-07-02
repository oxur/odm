# CDC Verification — Arc 05 / Slice 05: `affects` edge + stale-doc-vs-decision check (C5)

> Independent verification of CC's closed ledger (impl + close on
> `arc05-slice05-affects-stale-doc`, commits `ab5420a` + `164887d`), per
> LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo rows
> route to CI / a local 1.85+ run.

## Environment constraint (disclosed)

CDC sandbox has no 1.85+ toolchain; CC built + ran on local 1.95.0 (attested → reproduced on
CI). Branch cut off `arc05-slice04-…` (01–04 unmerged) — rebase onto `main` when they merge.

## Row dispositions

**Row count:** 6 opened, 6 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **T-1** — `Violation::StaleDoc` (`check.rs:92`) + `check_stale_docs` pass (`:267`): for
  `A affects B` with `A.updated > B.updated`, a finding whose subject is B, carrying the
  decision A + both dates. `check_flags_stale_doc_after_decision` present. ✔
- **T-2** — `violation_severity` helper (`commands.rs`) maps `StaleDoc → Warning` (others
  stay Error); advisory without `--strict`, fails with. `check_fresh_doc_not_flagged`
  (odm-core, no false positive) + `check_stale_doc_is_warning` (odm-cli, severity/exit).
  ✔ (see ruling 1 on the crate split)
- **T-3** — `A.updated > B.updated` (`>`, not `>=`); the `Violation::StaleDoc` doc comment
  (`check.rs:78–88`) documents "structural + temporal, never semantic," the `affects`-edge-is-
  the-commitment framing, and the `NaiveDate` **day-granularity** limitation.
  `check_stale_doc_same_day_not_flagged` present (also confirms a dangling `affects` target is
  link-integrity's concern, not this pass). ✔
- **T-4 (the invariant guard)** — `git diff --stat -- crates/odm-index/` → **empty**. Zero
  index change; `affects` (`adapter.rs:95`) + `updated` (`record.rs:101`) already synthesized;
  `check` reads them via `index_frontmatters`. **A4 adapter-fidelity invariant honored by
  non-triggering** — a new *use* of indexed fields, not a new field. ✔
- **T-5** — `CHECK_SCHEMA = "check/v1"` **unchanged** (`commands.rs:915`); the `stale-doc`
  finding joins the existing findings list additively. `check_json_includes_stale_doc`
  present. ✔
- **T-6 (no `unsafe`)** — grep empty in `check.rs` + `commands.rs`. ✔

**Attested by CC (local 1.95.0), pending CI:** clippy `-D warnings` → 0; line coverage
**check.rs 99.2% / commands.rs 91.9%**; 41 suites green. → **PENDING CI.**

## Rulings on CC's flagged decisions

1. **T-2 verify-crate amendment — CC was right, and it corrects my imprecision.** My ledger
   put `check_stale_doc_is_warning` in `-p odm-core`, but **`odm-core::check` has no severity
   model** — it emits structural `Violation`s; severity/`--strict`/exit are `odm-cli`'s
   concern (the new `violation_severity` helper). CC split it correctly: odm-core half =
   `check_fresh_doc_not_flagged` (no false positive), odm-cli half = `check_stale_doc_is_warning`
   (severity/exit). Accepted — the layer boundary is real and I mis-stated the crate; flagged
   in the ledger, closing-report, and arc-plan. (Second time CC has caught a layer/convention
   imprecision in my Verify — a healthy pattern; my rows should name the crate by where the
   behavior *lives*, not where it's conceptually "about.")
2. **`violation_severity` helper (per-finding severity).** Accepted — a clean, additive
   change: the aggregation previously hardcoded `Error` for every structural finding;
   `StaleDoc → Warning` with all others unchanged. It is exactly the hook slice06's advisory
   deferred-surfacing findings will reuse. Good forward-setup, not scope creep.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-5: the stale-doc check; A-11 **mechanism-complete**
  (correctly marked "reproduce at arc-scale," not inherited).
- **Silent-drop diff honest?** ✔ — 6/6; the verify-crate amendment + the `violation_severity`
  change are disclosed.
- **Findings + arc-plan?** ✔ — CC propagated the bubble-up itself (A-5 + A-11 note + v1.9),
  including the "pure-check extension pattern" (a `check_*` helper + CLI
  severity/label/detail/fix mapping + additive JSON) as the **template for slice06's deferred
  surfacing**, and `violation_severity` as its advisory hook.

## Verdict

**Arc 05 / Slice 05 CDC-verified on structure; both flags ruled; cargo rows pending CI.**
C5 is cashed honestly: a doc whose governing decision moved after it is flagged as
*potentially* stale (structural/temporal, never semantic — the `affects` edge carries the
"committed decision" meaning), at Warning severity, `--strict`-gated, with zero index change
(the invariant guarded by an empty `git diff`). A-5 attested-on-close; flips `done` on CI
green. **A5 at 5/8.**

**Next (in-order, per the settled 05→06→07→08): slice06 — deferred surfacing + re-entry
predicate.** Note for its planning: the *surfacing* half reuses slice05's check-extension
pattern (ready now), but the *re-entry predicate* is a **probe**, so it has affinity with the
freshness model (07/08). Pre-freshness it will evaluate via `run_corpus` (like drift does
today); 07/08 will fold it into the incremental machinery. A clean, disclosed interim — to be
made explicit in the slice06 doc.

CDC: planning thread, 2026-07-02. Iterations used: 1.
