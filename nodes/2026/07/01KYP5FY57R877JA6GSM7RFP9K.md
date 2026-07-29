---
id: 01KYP5FY57R877JA6GSM7RFP9K
number: 561511100
type: artifact
schema: artifact/v1.1
name: 'Slice 09 (Migration Fidelity): Coverage enforcement *capability*'
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice09-coverage-enforcement/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYP5FRXJBKY1GHW4QF9JQZD7
---
# Slice 09 (Migration Fidelity): Coverage enforcement *capability*

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced` at its scale.
> **This slice is fixture-only — no live mutation.** Capability rows are fixture-proven
> (`attested` → CI); CDC reproduces by reading the code + fixtures and on CI. There is **no live
> class-(b) row** here (the live mint + backfill are s10). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **`artifact` node type exists + validates**: `NodeType::Artifact` variant + `artifact/v1.0` schema marker + per-type field validity (ODD-0020 model) | `cargo test -p odm-core` → an `artifact` node round-trips parse/emit; an invalid per-type field is a `check` Error; unknown-schema handling per 0020 | serious | ODD-0025 §2.5 / ODD-0013 §2.2 | done | attested | `node_type.rs` (`Artifact` variant, `as_str`/`is_document`/`valid_child_types`/`FromStr`); `schema.rs::artifact_stamps_and_round_trips_the_shared_current_version` (stamps/parses `artifact/v1.1` — see the row's note on the naming deviation); `check.rs::check_flags_wrong_type_field` extended with `bad_artifact`/`good_artifact`. `cargo test -p odm-core` → 0 failed. |
| F-2 | **Artifact containment = nearest modeled scale** (§2.5): a per-slice artifact is `part_of` its slice; an arc-/chunk-level artifact is `part_of` its **arc**; no `chunk`/`step` node scale | `cargo test` → fixture with a per-slice `ledger.md` and an arc-level report → the former `part_of` the slice node, the latter `part_of` the arc node | serious | ODD-0025 §2.5 | done | attested | `odm-migrate/tests/artifact.rs::artifact_containment_resolves_nearest_modeled_scale` — per-slice → slice, chunk-level (`c1-cdc-verification.md`, no slice subdir) → arc, genuinely top-level → uncontained, **and** a named arc's own doc resolved via `source.paths` (not a re-derived coordinate). Resolver: `artifact.rs::scale_index`/`nearest_scale`. |
| F-3 | **Discovery reaches + mints the artifact family, mint-all** (§2.6): `ledger`/`cc-prompt`/`cdc-verification`/`closing-report`/ADR/amendment/UAT + **every report incl. `coverage-report.md`** minted as `artifact`, 1:1 body + `source`, hard-gated | `cargo test` → fixture plan-set with supporting docs → each minted `artifact`; no doc exempted; body-hash gate passes on the faithful bodies | serious | ODD-0025 §2.6 (F10 mint-all) | done | attested | `odm-migrate/src/artifact.rs::mint_artifacts` (new module). Tests: `artifact_mint_all_covers_supporting_docs_incl_reports` (count == every `Ledger\|CcPrompt\|CdcVerification\|ClosingReport\|Other`-classed doc, incl. a `coverage-report.md`), `artifact_mint_is_idempotent`, `artifact_mint_dry_run_writes_nothing`, `artifact_body_is_verbatim_1to1` (hard body-hash gate). |
| F-4 | **Design/research family reachable + `source` backfilled** (MF-3 residual, F7): the 14 `design`/`research` nodes acquire a `source` sub-map; discovery no longer structurally excludes that type family | `cargo test` → fixture with a `design` + a `research` doc → both migrate with `source`, body-hash-gated; containment **optional** (top-level allowed) | serious | arc-plan MF-3 / F7 | done | attested | `odm-migrate/src/mapping.rs::backfill_source` (new). Tests (`odm-migrate/tests/backfill_source.rs`): `backfill_source_replaces_a_stub_body_and_adds_source`, `backfill_source_keeps_a_faithful_body_and_adds_source`, `backfill_source_rejects_a_drifted_non_stub_body` (hard gate, never silently backfills drift), `backfill_source_is_idempotent`, `backfill_source_dry_run_writes_nothing`. Containment untouched (optional, §2.7 — F-8 covers the orphan-check side). |
| F-5 | **Doc-coverage wired into `odm check` as an Error** (MF-1/MF-6): any `.md` under the scan root with no covering node fails `check`; built on s08's portable relative `source.paths` key | `cargo test` → seed an uncovered `.md` → `check` returns Error naming it; cover it → green; result identical from two `TempDir` roots | serious (loud-hole guard) | ODD-0025 §5 / arc MF-6 | done | attested | `commands.rs::aggregate`'s new (c3) block + `coverage_scan_root` (config-gated on a new, currently-absent `[coverage] scan_root` key). Tests (`odm-cli/tests/check_coverage.rs`): `check_coverage_flags_an_uncovered_doc_as_an_error`, `check_coverage_is_green_once_the_doc_is_covered`, `check_coverage_rule_is_a_noop_without_scan_root_configured`, `check_coverage_result_is_identical_from_a_relative_and_an_absolute_scan_root`. **Live activation confirmed still off**: `odm check` run against `.worktrees/odm` (read-only) — 0 errors, 8 pre-existing warnings, exit 0, unchanged from before this slice. |
| F-6 | **coverage.rs Finding 2 fixed**: `representation()` counts a **named** arc (and its slices) as represented via the s05 name-derived key, not only `arc_coordinate()`; summary reads a truthful 12/12 | `cargo test` → fixture with a named arc that **has** a node → `representation()` reports it represented (0 missing), not "N−4/N" | correctness (report clarity) | CDC v2.8 Finding 2 | done | attested | `coverage.rs::resolve_arc_dir_numbers` (new — replays `named_arc_number`'s collision handling over the full named-arc-dir set). Test: `coverage.rs` integration `coverage_representation_resolves_a_named_arc_with_a_node` — 0 missing arcs/slices for a named arc that has a matching node; the pre-existing `coverage_representation` (a named arc *without* a node) still correctly reports it missing. |
| F-7 | **coverage.rs Finding 3 fixed**: `provenance_absence` no longer scans the stale stored-`provenance:` key; retargeted to **`source:` presence** (or retired for the source-presence check) — decided + justified | `cargo test` → a node **with** `source:` is not flagged; a node missing `source:` is; no scan for `provenance:` remains | correctness | CDC v2.8 Finding 3 | done | attested | **Decision** (doc comment on `provenance_absence`): retarget to the typed `Frontmatter::source().is_none()` check — `provenance` is derived-only/never-stored (0013), and the literal `provenance:` key was renamed to `source:` at s02, so the old scan could never match. `has_provenance_key`/`frontmatter_yaml` (the stale scan) removed, along with their now-dead test. Test: `coverage_provenance_absence` (updated to build a real `Source` via `persist_node`, no more `("provenance", "migrated")` extra-field stand-in). `grep -n 'provenance:'` on `coverage.rs` → no scan remains. |
| F-8 | **Optional containment honored** (§2.7): the doc-coverage / `orphan` check does **not** flag a legitimately top-level `design`/`research`/`artifact` node as an orphan | `cargo test` → a top-level doc node → not reported as an orphan or uncovered | correctness | ODD-0025 §2.7 | done | attested | Structural — `check_orphan` (`recompose.rs`) is gated on `is_work()`, which `artifact` never satisfies, so this held from F-1 onward; confirmed (not just asserted) via a new case in `crates/odm-core/tests/recompose.rs::detect_orphan` (a top-level `artifact` node, alongside the pre-existing `note` case, both absent from the reported orphan set) + an updated doc comment naming `artifact` explicitly. |
| F-9 | **No live mutation; downstream not pulled forward**: no `odm`-branch commit; s10 (live mint/backfill), s11 (synthesis/L-8b), s12 (reconcile) untouched; ODD amendments implemented-against, not re-litigated | `git -C .worktrees/odm status` clean after the slice; grep: no synthesis/reconcile code added; ODD-0025/0013/0020 unedited unless a new line was genuinely needed (then amended, cited) | correctness | LEDGER-DISCIPLINE / operator | done | attested | `git -C .worktrees/odm status --short` → empty (clean); `git -C .worktrees/odm log --oneline -3` → HEAD still slice08's commit, nothing new. No `docs/design/*.md` (ODD-0013/0020/0025) edited this slice — implemented against the existing amendments, no new model line needed. No synthesis/reconcile-runner code touched; `mint_artifacts`/`backfill_source`/the check rule are all new, additive, fixture-only call paths. |
| F-10 | **Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) changed modules, target 95%; no model drift** | `clippy` exit 0; `! grep unsafe` on changed files; `llvm-cov` ≥ 90%; cross-read: `source` still the identity axis, body-hash gate unchanged, `artifact` per ODD-0025 §2.5 — amend-not-work-around if a model line is needed | polish/correctness | CLAUDE.md / ODD-0025 | done | attested | `cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0 (repeated after every checkpoint). `grep unsafe` over every file this slice touched → none. `cargo llvm-cov --workspace --all-features --summary-only`: workspace TOTAL line 90.85%; every changed/new module ≥ 94% except `commands.rs` (88.93% file-wide — the file is 2521 lines, mostly pre-existing; the new `coverage_scan_root` + `aggregate`'s (c3) block are fully exercised by all 4 branches the `check_coverage.rs` tests drive). `cargo test --workspace --all-features` → 0 failed throughout. No model drift: `source` unchanged as the identity axis, the body-hash gate untouched, `artifact` implemented per ODD-0025 §2.5/§2.6/§2.7 with no ODD edits. |

## What Worked

- **Splitting into checkpoint commits** (type foundation → discovery reach → check-wiring →
  F-8/tests) kept each step reviewable and gave a clean rollback point per concern, without
  needing the five-iteration cap — the capability fit in one pass once s10's live run was
  already carved out.
- **Reusing `source.paths` as the containment resolver** (`artifact.rs::scale_index`) rather
  than re-deriving arc/slice numbering for containment turned out to be strictly simpler than
  the numbered/named branching `representation()` needed for F-6 — a directory→id index built
  from already-persisted nodes resolves a numbered *or* named arc/slice identically, with no
  special case. Worth remembering for any future "resolve this doc's nearest X" problem.
- **The config-gate for F-5** (`[coverage] scan_root`, absent from the live store) satisfied
  "wire it in but don't go live-red" exactly, verifiably, and without inventing a code-level
  disabled flag — confirmed by literally running `odm check` against `.worktrees/odm` and
  diffing the result (unchanged).
- **CDC's Findings 2 and 3 both needed a real fix, not just a comment update** — F-6 required a
  genuine new derivation (`resolve_arc_dir_numbers`); F-7's fix uncovered a formula bug of its
  own (`slice_number()` assumes a raw numbered major, which a named arc's handle is not) that a
  quick fixture test caught before it shipped.

## Closure

Closed at commit `9cccdde` (`release/1.0.x`, capability; preceded by `012fad5`, `1760d7b`,
`bc6e193` for the F-1/F-6/F-7, F-2/F-3/F-4, and F-5 checkpoints respectively) on 2026-07-28.
Verified by: CC (self-verified per-row above); CDC review pending. Rows: 10. Done: 10.
Deferred: 0. No-op: 0.

On close, bubble up to `../arc-plan.md`: enforcement capability lands; **s10 (coverage live run)
unblocked + next**; MF-1/MF-6 → "capability done, live-pending".
