# Slice 05 (Arc 05): `affects` edge + stale-doc-vs-decision check (C5)

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Independent of the
> freshness rework (07/08); cashes ODD-0001 C5.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| T-1 | `odm check` gains a **stale-doc** finding: for each `A affects B` where `A.updated > B.updated`, emit a finding naming **A**, **B**, and both dates — the governing decision moved after the doc it governs | `cargo test -p odm-core check_flags_stale_doc_after_decision` → ok | serious | ODD-0001 C5 / arc-plan A-11 | open | | The `affects` edge **is** the "committed decision governs this doc" assertion — no separate committed-gating (slice-doc). |
| T-2 | The finding is a **Warning** (potential staleness — human judges), consistent with `check`'s tiers: advisory without `--strict`, failing under `--strict`; a doc updated **at-or-after** its governing decision (`B.updated >= A.updated`) is **not** flagged (no false positive) | `cargo test -p odm-core check_stale_doc_is_warning` + `check_fresh_doc_not_flagged` → ok | serious | check severity model / arc-plan | open | | Not an Error: we assert *possible* staleness, never a proven defect. Mirrors `check`'s Warning tier + `--strict`. |
| T-3 | The check is **structural/temporal, never semantic** — it flags "potentially stale" from `affects` + `updated`, never asserts B *contradicts* A; the **day-granularity** limitation (`updated` is a `NaiveDate`, so same-day edits are not distinguished) is documented in the finding's doc comment | `cargo test -p odm-core check_stale_doc_same_day_not_flagged` → ok AND `grep -nE "stale|semantic|granular|NaiveDate" crates/odm-core/src/check.rs` | serious | odm ethos (no fake semantic detection) | open | | The recomposition-integrity boundary, applied to docs: structural signal, human judgment. Same-day (`>` not `>=`) avoids nagging on a decision+doc edited the same day. |
| T-4 | Index-backed with **no new index field**: `check` reads `affects` + `updated` off the index (both already in `IndexRecord`/adapter since A4); **no** adapter / fidelity-test / `FORMAT_VERSION` change — the A4 adapter-fidelity invariant stays honored **by non-triggering** | `grep -nE "EdgeKind::Affects|updated" crates/odm-index/src/adapter.rs crates/odm-index/src/record.rs` (fields already present) AND `git diff --stat` shows **no** change under `crates/odm-index/` | serious | A4 invariant (`arc04 closing-report` #2) / arc-plan v1.3 | open | | New *use* of already-indexed fields, not a new field. First A5 slice where `check` reads across the index for a new purpose — confirmed no adapter work needed. |
| T-5 | `check --json` carries the stale-doc finding **additively** — a new finding kind in the existing findings list; **no `check` schema version bump** (the schema already enumerates finding kinds; adding one is backward-compatible) | `cargo test -p odm-cli check_json_includes_stale_doc` → ok AND `grep -rnE "check/v" crates/odm-cli/src` shows the marker **unchanged** | serious | A3 schema-marker convention / slice04 additive precedent | open | | Same additive discipline as slice04's drift slot. |
| T-6 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the touched `odm-core`/`odm-cli` paths | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only -p odm-core -p odm-cli` → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(At slice close. Patterns that made the slice close cleanly.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 6. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
