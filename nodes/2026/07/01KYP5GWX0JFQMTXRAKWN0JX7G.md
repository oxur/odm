---
id: 01KYP5GWX0JFQMTXRAKWN0JX7G
number: 504170700
type: artifact
schema: artifact/v1.1
name: 'Closing report — Slice 05 (Arc 05): `affects` edge + stale-doc-vs-decision check (C5)'
created: 2026-07-02
updated: 2026-07-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice05-affects-stale-doc/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTK54PMJWBANX7BN5V8
---
# Closing report — Slice 05 (Arc 05): `affects` edge + stale-doc-vs-decision check (C5)

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc05-slice05-affects-stale-doc`, branched off
> `arc05-slice04-drift-in-rollup-orient` (slices 01–04 unmerged; the prompt's
> named fallback). Rebase onto `main` once 01–04 merge.

## Per-row walk

**T-1 — stale-doc finding — done (attested).** `odm_core::check` gained a
`check_stale_docs` pass and a `Violation::StaleDoc { decision, decision_number,
decision_name, decision_updated, doc_updated }` variant. For each `A affects B`
with `A.updated > B.updated`, it emits a finding whose subject is **B** (the
potentially-stale doc) carrying **A** and both dates.
`check_flags_stale_doc_after_decision` → ok.

**T-2 — Warning + no false positive — done (attested).** A new
`violation_severity` helper in `commands.rs` maps `StaleDoc → Warning` (all other
structural violations stay `Error`); `odm check` surfaces it and exits `0`
without `--strict`, `1` with. `B.updated >= A.updated` yields no finding.
`check_stale_doc_is_warning` (odm-cli) + `check_fresh_doc_not_flagged` (odm-core)
→ ok. **Verify-amend (flagged):** the warning/`--strict` half is a `-p odm-cli`
test, not `-p odm-core` as the row's Verify wrote — severity and exit codes are
the CLI's concern; `odm-core::check` has no severity model (it emits `Violation`s,
the CLI assigns tiers).

**T-3 — structural, never semantic; day-granularity — done (attested).** The
predicate is `>` (not `>=`), so a same-day decision+doc edit is not flagged; the
`Violation::StaleDoc` doc comment states the "structural + temporal, never
semantic" boundary and the `NaiveDate` day-granularity limitation.
`check_stale_doc_same_day_not_flagged` → ok; the `stale|semantic|granular|NaiveDate`
grep matches the doc.

**T-4 — no new index field — done (attested).** `check` reads `affects` +
`updated` off the index (both synthesized since A4: `adapter.rs:95`
`EdgeKind::Affects`, `adapter.rs:51`/`record.rs:101` `updated`). `git diff --stat
-- crates/odm-index/` is empty — no adapter/fidelity/`FORMAT_VERSION` change; the
A4 invariant stays honored by non-triggering.

**T-5 — `--json` additive — done (attested).** The stale-doc finding joins the
existing `check --json` findings list (`code:"stale-doc"`, `severity:"warning"`);
`schema` is still `"check/v1"` (`CHECK_SCHEMA` unchanged). `check_json_includes_stale_doc`
→ ok.

**T-6 — gates — done (attested).** clippy `-D warnings` → 0; no `unsafe`;
coverage (line) `odm-core/check.rs` 99.23%, `odm-cli/commands.rs` 91.92% — both
≥ 90. Full workspace green (41 suites).

Rows: 6. Done: 6. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item shipped: the stale-doc finding in `odm_core::check`,
Warning severity + no false positive, structural/temporal-not-semantic with the
documented day-granularity, additive `--json`, and no index change. Every "out"
item stayed out: no deferred surfacing (slice06), no freshness/probes work
(07/08), no semantic contradiction detection (permanently out), no committed-state
gating beyond the `affects` edge.

## Deviations / decisions flagged

1. **T-2's `check_stale_doc_is_warning` lives in `odm-cli`, not `odm-core`** (the
   row's Verify wrote `-p odm-core`). Severity/`--strict`/exit are CLI concerns;
   `odm-core::check` deliberately has no severity model. The odm-core half of T-2
   (no-false-positive) is `check_fresh_doc_not_flagged`. A Verify-crate amendment,
   not a scope change.
2. **Per-violation severity in the aggregation.** The `aggregate` loop previously
   hardcoded `Severity::Error` for every `odm_core::check` finding; slice05 adds
   `violation_severity` so `StaleDoc` is a `Warning`. Necessary (T-2) and additive
   — no existing finding's severity changes.
3. **No amendment to the finding shape or the temporal predicate** was needed.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice05 deliver A-5 and the A-11 mechanism?** Yes to A-5 ("slice05
closed"). A-11 ("the `affects` edge powers a stale-doc-vs-committed-decision
finding in `check`") is a **compose** row *reproduced at arc scale*: slice05
lands its full mechanism (a doc governed by a decision that moved after it is
flagged, Warning-tier, `--strict`-gated, additive JSON). Recorded as
mechanism-complete, to be reproduced at arc-close (composition rows are never
inherited from a slice).

**2. What it reveals for slice06 (deferred surfacing + re-entry predicate).**
  - **The pure-`check` extension pattern is the template.** slice05 added a
    structural finding as a `check_*` helper that pushes onto the findings vector,
    with a CLI-side severity/label/detail/fix mapping and an *additive* JSON
    finding kind (no schema bump). slice06's deferred-node surfacing, if it flows
    through `check`/`orient`, should follow the same additive path.
  - **`violation_severity` is now the hook** for any further advisory structural
    finding — slice06 (or a later slice) that adds a non-error `check` finding
    plugs in there rather than re-hardcoding.
  - **`affects` is now semantically load-bearing, not just link-checked.** slice06
    (and any C5 follow-up) can rely on `affects` carrying the "committed decision
    governs this doc" meaning. A possible future tightening (gate on the decision
    node's *status*, not just the edge) was considered and left out of scope — the
    edge is the commitment for C5; flag if a later slice wants the stronger gate.

**3. Doc-keeping (unchanged from slices 03/04).** The stale `CLAUDE.md` line
naming `oxur_cli` output helpers is still queued for the arc-close doc fix;
slice05 rendered via `writeln!` per the actual convention.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-5 row evidence + a v1.9 version-history entry), not only here.
