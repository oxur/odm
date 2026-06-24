# Slice 06 (Arc 02): `check` v2

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | `check` aggregates schema + link-integrity (v1) on the whole graph | `cargo test -p odm-cli check_schema_and_links` → ok | serious | arc01 slice06 | done | `8022bc5`: `check_schema_and_links` → 1 passed. A blank-name slice (`missing-field`) and a dangling `depends_on` (`dangling-edge`) both surface; exit 1. Consumes `odm_core::check::check`. | |
| H-2 | `check` fails on a cycle-without-tear; passes once torn | `cargo test -p odm-cli check_cycle_requires_tear` → ok | serious | 0013 §4.3 | done | `8022bc5`: `check_cycle_requires_tear` → 1 passed. X↔Y cycle → `cycle` error, exit 1; declaring `edges.tears=[Y]` on X → exit 0. Consumes `NodeGraph::topological_order(tears)`. | Tear rationale synthesized (schema gap) — flagged. |
| H-3 | `check` reports out-of-order / staleness | `cargo test -p odm-cli check_staleness` → ok | serious | 0013 §4.4 / 0001 B3 | done | `8022bc5`: `check_staleness` → 1 passed. A slice advanced to `built` while its dep is unsatisfied → `staleness` warning (exit 0 normal). Consumes `staleness_on_advance`. | |
| H-4 | `check` reports recomposition violations (orphan/stub/decomposition drift) | `cargo test -p odm-cli check_recomposition` → ok | serious | 0013 §4.5 | done | `8022bc5`: `check_recomposition` → 1 (orphan) + `check_recomposition_variants` → 1 (undeveloped-stub, advanced-without-decomposition, decomposition-drift). Consumes `recompose::integrity`. | |
| H-5 | `check` reports below-threshold (soft-satisfied) dependencies | `cargo test -p odm-cli check_soft_satisfied` → ok | serious | 0013 §4.4 / 0001 F2 | done | `8022bc5`: `check_soft_satisfied` → 1 passed. Dep satisfied at `attested` < `reproduced` → `soft-satisfied` warning (exit 0 normal). Consumes `NodeGraph::blocked`. | |
| H-6 | Exit codes: `0` clean, `1` violations, `2` usage error | `cargo test -p odm-cli check_exit_codes` → ok | serious | 0013 §7 | done | `8022bc5`: `check_exit_codes` → 1 passed. Clean tree → 0; orphan → 1; `clap` rejects `--bogus` (the binary's exit-2 path). | Exit codes verified in-process via `dispatch`'s return (not `assert_cmd`) — flagged. |
| H-7 | `--strict`/CI mode promotes warnings (staleness, soft-satisfaction) to failures | `cargo test -p odm-cli check_strict_mode` → ok | correctness | 0013 §4.4 | done | `8022bc5`: `check_strict_mode` → 1 passed. A soft-satisfied-only corpus → exit 0 normally, exit 1 under `--strict`. | |
| H-8 | Every finding names the exact fix command (errors-as-affordances) | `cargo test -p odm-cli check_errors_name_fix` → ok | serious | 0013 §7 / 0001 | done | `8022bc5`: `check_errors_name_fix` → 1 passed. Every JSON finding has a non-empty `fix`; human output prints a `fix:` line per finding. | |
| H-9 | `--json` report with a stable, documented schema | `cargo test -p odm-cli check_json_schema` (snapshot) → ok | correctness | 0013 §7 | done | `8022bc5`: `check_json_schema` → 1 passed. Envelope `{ok, errors, warnings, findings}`; per-finding `{code, detail, fix, name, node, number, severity}`; both error+warning families present and labeled. | |
| H-10 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% | `cargo clippy -p odm-cli -p odm-core --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-cli -p odm-core --summary-only --ignore-filename-regex '(odm-store|odm-graph)/'` → **line** ≥ 90% (target 95%) | serious | CLAUDE.md | done | `8022bc5`: clippy (cli+core) → exit 0; unsafe grep → no match; fmt clean; coverage TOTAL **line 93.69%** (commands.rs 90.03%, recompose.rs 100%). | `run()` (real stdout/cwd) uncovered in-process — see report. |

## What Worked

- **One aggregator, five predicate sources.** `aggregate()` walks the corpus
  once and folds in `odm_core::check::check` (v1), `topological_order` (cycles),
  `recompose::integrity` (slice05), and per-node `blocked` + `staleness_on_advance`
  (slice04) into a single `Vec<CheckEntry>`. No predicate is reimplemented — the
  CLI only maps each to a `(severity, code, detail, fix)` and renders.
- **Severity is the whole exit-code story.** Errors always fail; warnings
  (staleness, soft-satisfaction) fail only under `--strict`. `failed = errors > 0
  || (strict && warnings > 0)` is the one rule; the JSON `ok` mirrors it.
- **Errors-as-affordances fell out of the per-family mapping.** Each family's
  render arm already knows the precise next action (repoint `part_of`, `odm tear`
  the edge, raise evidence, affirm `decomposed`), so naming the fix was local.
- **In-process exit-code assertions.** `dispatch` returns the `u8` the binary
  maps to `ExitCode`, so the `run()` harness asserts `r.code` directly — exit
  codes are tested without spawning a process, consistent with the slice05
  node-CRUD pivot.

## Closure

Closed at `8022bc5` on `2026-06-24`. CDC: pending (cargo rows reproduced by CDC
in CI or a local 1.85+ toolchain). All `done` states are *proposed done* pending
that independent verification. Total rows: 10. Done: 10. Deferred: 0. No-op: 0.
**Arc 02 complete on close.**

**Flagged for CDC.** (1) **`assert_cmd` deviation:** the cc-prompt asks for
`assert_cmd` exit-code tests, but every ledger row is `-p odm-cli`, and odm-cli
is library-only (the `odm` binary lives in `oxur-odm`). Exit codes are verified
in-process via `dispatch`'s returned `u8` — the exact value `run` maps to
`ExitCode` — matching the L-6 precedent. If CDC wants end-to-end binary tests, a
small `assert_cmd` suite in `oxur-odm/tests/` is a clean follow-up. (2)
**Recomposition classified as errors:** orphan/stub/drift/advanced-without are
errors (always fail), per the ledger's "recomposition violations" framing and
H-7 naming only staleness + soft-satisfaction as the promotable warnings. Drift
and advanced-without are borderline (re-affirmation prompts) and could move to
warnings if CDC prefers. (3) **Tear rationale synthesized:** `edges.tears` has no
rationale in the schema (the gap deferred since slice04), but the engine's `Tear`
requires one, so `check` injects a placeholder purely to exclude the torn edge
from cycle detection — no rationale is enforced or displayed. (4) **v1 tests
updated for the stricter v2:** `check_clean_passes` now seeds a total tree (a
top-level arc with no project is now an orphan) and `check_json_v1` asserts the
v2 envelope (gained `severity`/`code`/counts) — disclosed changes to this
crate's own check tests. (5) **`reconcile`/desired-fact drift not in scope**
(Arc A5 extends `check`), and the `odm tear` / `decomposed`-affirm commands named
in fix strings are not yet implemented (named as affordances, per the v1 pattern
of naming commands that land later).
