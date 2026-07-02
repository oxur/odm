# Slice 05 (Arc 05): `affects` edge + stale-doc-vs-decision check (C5)

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Independent of the
> freshness rework (07/08); cashes ODD-0001 C5.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| T-1 | `odm check` gains a **stale-doc** finding: for each `A affects B` where `A.updated > B.updated`, emit a finding naming **A**, **B**, and both dates — the governing decision moved after the doc it governs | `cargo test -p odm-core check_flags_stale_doc_after_decision` → ok | serious | ODD-0001 C5 / arc-plan A-11 | done | `cargo test -p odm-core check_flags_stale_doc_after_decision` → ok (1 passed): `A affects B`, `A.updated 2026-06-25 > B.updated 2026-06-22` → a `Violation::StaleDoc` finding whose subject is **B** and which carries `decision=A` (id/number/name) + both dates. Added `check_stale_docs` pass to `odm_core::check`. **attested** | The `affects` edge **is** the "committed decision governs this doc" assertion — no separate committed-gating (slice-doc). |
| T-2 | The finding is a **Warning** (potential staleness — human judges), consistent with `check`'s tiers: advisory without `--strict`, failing under `--strict`; a doc updated **at-or-after** its governing decision (`B.updated >= A.updated`) is **not** flagged (no false positive) | `cargo test -p odm-core check_stale_doc_is_warning` + `check_fresh_doc_not_flagged` → ok | serious | check severity model / arc-plan | done | `cargo test -p odm-cli check_stale_doc_is_warning` → ok (1 passed): `odm check` surfaces the `stale-doc` at `warning` severity and exits **0** without `--strict`; `odm check --strict` → exit **1**. `cargo test -p odm-core check_fresh_doc_not_flagged` → ok (1 passed): `B.updated >= A.updated` → no finding (no false positive). New `violation_severity` helper maps `StaleDoc → Warning` (all other structural violations stay `Error`). **attested** | Not an Error: *possible* staleness, human judges. **Verify-amend (flagged):** `check_stale_doc_is_warning` is a `-p odm-cli` test, not `-p odm-core` — severity/`--strict`/exit are the CLI's concern; `odm-core::check` has no severity model (it emits `Violation`s). The odm-core half (no-false-positive) is `check_fresh_doc_not_flagged`. |
| T-3 | The check is **structural/temporal, never semantic** — it flags "potentially stale" from `affects` + `updated`, never asserts B *contradicts* A; the **day-granularity** limitation (`updated` is a `NaiveDate`, so same-day edits are not distinguished) is documented in the finding's doc comment | `cargo test -p odm-core check_stale_doc_same_day_not_flagged` → ok AND `grep -nE "stale|semantic|granular|NaiveDate" crates/odm-core/src/check.rs` | serious | odm ethos (no fake semantic detection) | done | `cargo test -p odm-core check_stale_doc_same_day_not_flagged` → ok (1 passed): same-day `A.updated == B.updated` → not flagged (`>` not `>=`); the test also confirms a dangling `affects` target is skipped (link-integrity's concern). `grep -nE "stale\|semantic\|granular\|NaiveDate" crates/odm-core/src/check.rs` → matches in the module doc + the `Violation::StaleDoc` doc comment (documents "structural + temporal, never semantic" and the `NaiveDate` day-granularity `>`-not-`>=` limitation). **attested** | The recomposition-integrity boundary, applied to docs: structural signal, human judgment. |
| T-4 | Index-backed with **no new index field**: `check` reads `affects` + `updated` off the index (both already in `IndexRecord`/adapter since A4); **no** adapter / fidelity-test / `FORMAT_VERSION` change — the A4 adapter-fidelity invariant stays honored **by non-triggering** | `grep -nE "EdgeKind::Affects|updated" crates/odm-index/src/adapter.rs crates/odm-index/src/record.rs` (fields already present) AND `git diff --stat` shows **no** change under `crates/odm-index/` | serious | A4 invariant (`arc04 closing-report` #2) / arc-plan v1.3 | done | `git diff --stat -- crates/odm-index/` → **empty** (no change under `crates/odm-index/`). `grep -nE "EdgeKind::Affects\|updated" crates/odm-index/src/adapter.rs crates/odm-index/src/record.rs` → `adapter.rs:95 EdgeKind::Affects => edges.affects.push(...)`, `adapter.rs:51 record.updated`, `record.rs:101 pub updated: NaiveDate` — both fields already synthesized. `check` reads them via `index_frontmatters`. **attested** | New *use* of already-indexed fields, not a new field. No adapter/fidelity/`FORMAT_VERSION` change; the A4 invariant stays honored by non-triggering. |
| T-5 | `check --json` carries the stale-doc finding **additively** — a new finding kind in the existing findings list; **no `check` schema version bump** (the schema already enumerates finding kinds; adding one is backward-compatible) | `cargo test -p odm-cli check_json_includes_stale_doc` → ok AND `grep -rnE "check/v" crates/odm-cli/src` shows the marker **unchanged** | serious | A3 schema-marker convention / slice04 additive precedent | done | `cargo test -p odm-cli check_json_includes_stale_doc` → ok (1 passed): `check --json` findings list carries `{code:"stale-doc", severity:"warning", number:2, detail:"…Decision…"}`; `schema` still `"check/v1"`, `ok:true`, `warnings ≥ 1` (advisory). `grep -rnE "check/v" crates/odm-cli/src` → `CHECK_SCHEMA = "check/v1"` **unchanged**. **attested** | Same additive discipline as slice04's drift slot: a new finding kind in the existing list, no schema bump. |
| T-6 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the touched `odm-core`/`odm-cli` paths | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only -p odm-core -p odm-cli` → **line** ≥ 90% | serious | CLAUDE.md | done | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0; `grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src` → none; `cargo llvm-cov --summary-only -p odm-core -p odm-cli` → line: `odm-core/check.rs` **99.23%**, `odm-cli/commands.rs` **91.92%** (both ≥ 90). Full workspace green (41 suites). **attested** | |

## What Worked

- **The check was a one-pass addition to a pure function.** `odm_core::check` is
  built to grow (a `check_*` helper that pushes onto the findings vector);
  `check_stale_docs` slotted in with zero disturbance to the existing passes, and
  the finding order stayed deterministic (appended after supersession).
- **`affects` + `updated` were already there** — both indexed since A4, already
  read by the dangling-ref pass. The slice was a *new use of existing signals*,
  not a new field: the A4 invariant never came near triggering (`git diff` under
  `crates/odm-index/` is empty).
- **Structural/temporal, never semantic** kept it honest and tiny: `A.updated >
  B.updated` is the whole predicate; the `affects` edge carries the "committed
  decision" meaning, so no content analysis (confabulation) was needed.
- **Per-violation severity** was the one CLI change of substance: the aggregation
  hardcoded `Error` for all `odm_core::check` findings; a small `violation_severity`
  helper makes stale-doc a `Warning` without disturbing the others — additive,
  and it's where the next advisory structural finding will hook in.

## Closure

Closed at commit `ab5420a` on 2026-07-02. Verified by: CC (self-attested); CDC to
reproduce. Rows: 6. Done: 6. Deferred: 0. No-op: 0.

> Per LEDGER-DISCIPLINE v2.0 §A, all evidence is **attested** (CC built and ran
> every cargo/grep row on a local 1.95 toolchain). Independent **reproduction**
> (CI / local 1.85+) elevates to `reproduced` in `cdc-verification.md`.
> **Branching note:** slices 01–04 had not merged to `main`, so this slice
> branched off `arc05-slice04-drift-in-rollup-orient` (per the prompt's fallback)
> — rebase onto `main` once 01–04 merge.
