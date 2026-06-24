# Closing Report — Slice 06 (Arc 02): `check` v2

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain). **On close, Arc 02 is complete.**

- **Implementation commit:** `8022bc5`.
- **Branch:** `arc02-slice06-check-v2` (based on
  `arc02-slice05.1-evidence-dates`; not pushed; not merged to `main`).
- **Scope delivered:** `odm check` becomes the single mechanical gate — one
  command aggregating schema + link-integrity (v1), cycles-without-tears
  (slice02), recomposition (slice05), out-of-order/staleness and below-threshold
  satisfaction (slice04) — with severities, exit codes `0`/`1`/`2`, a `--strict`
  CI mode, errors-as-affordances, and a stable `--json` schema.
- **Result:** 10 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test --workspace` → all pass (11 check tests +
  CRUD/derived-order unregressed); clippy `-D warnings` (cli+core) → exit 0; no
  `unsafe`; coverage TOTAL line 93.69% (commands.rs 90.03%, recompose.rs 100%).

## Per-row walk

| ID | Status | Evidence (re-runnable at `8022bc5`) |
|----|--------|-------------------------------------|
| H-1 | done | `cargo test -p odm-cli check_schema_and_links` → 1 passed; `missing-field` + `dangling-edge` both surface; exit 1. |
| H-2 | done | `cargo test -p odm-cli check_cycle_requires_tear` → 1 passed; cycle → exit 1; declaring the edge in `edges.tears` → exit 0. |
| H-3 | done | `cargo test -p odm-cli check_staleness` → 1 passed; advanced-while-unsatisfied → `staleness` warning. |
| H-4 | done | `cargo test -p odm-cli check_recomposition` → 1 (orphan) + `check_recomposition_variants` → 1 (stub / advanced-without / drift). |
| H-5 | done | `cargo test -p odm-cli check_soft_satisfied` → 1 passed; below-threshold dep → `soft-satisfied` warning. |
| H-6 | done | `cargo test -p odm-cli check_exit_codes` → 1 passed; 0 clean / 1 violations / clap-rejects-`--bogus` (exit-2 path). |
| H-7 | done | `cargo test -p odm-cli check_strict_mode` → 1 passed; warning-only corpus: exit 0 normal, exit 1 `--strict`. |
| H-8 | done | `cargo test -p odm-cli check_errors_name_fix` → 1 passed; every finding carries a non-empty `fix`; human output prints `fix:`. |
| H-9 | done | `cargo test -p odm-cli check_json_schema` → 1 passed; stable envelope + per-finding keys; error+warning families labeled. |
| H-10 | done | clippy (cli+core) → exit 0; no `unsafe`; `cargo llvm-cov -p odm-cli -p odm-core …` → TOTAL line 93.69%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **`assert_cmd` vs. in-process `dispatch`.** The cc-prompt asks for `assert_cmd`
   exit-code tests, but all ten ledger rows are `cargo test -p odm-cli`, and
   odm-cli is **library-only** (the `odm` binary is built by the `oxur-odm`
   umbrella). Spawning a binary from odm-cli's own tests is impossible. I verify
   exit codes in-process: `dispatch` returns the `u8` that `run` maps to
   `ExitCode`, so `r.code == Some(0|1)` asserts the real code, and the `2` path
   is clap rejecting bad args before dispatch. This matches the L-6 precedent and
   the deliberate library-only architecture chosen in slice05. A small
   `assert_cmd` suite in `oxur-odm/tests/` against the real binary is a clean
   follow-up if CDC wants end-to-end coverage.

2. **Severity classification.** Errors (always fail): schema/link-integrity,
   cycle-without-tear, and **all** recomposition findings (orphan, undeveloped-
   stub, decomposition-drift, advanced-without-decomposition). Warnings (fail only
   under `--strict`): staleness and soft-satisfaction. This follows the ledger
   (H-4 "recomposition violations"; H-7 names only staleness + soft as warnings).
   Drift and advanced-without are arguably "re-affirmation prompts" and could be
   warnings — flagged for CDC to confirm the error classification.

3. **Tear rationale is synthesized.** `edges.tears` carries no rationale in the
   current schema (the gap deferred since slice04). The engine's `Tear::new`
   requires a non-empty rationale, so `check` injects a placeholder string purely
   so the cycle detector excludes the declared-torn edge. No rationale is enforced
   or shown. When the schema gains a tear rationale, `check` should enforce it
   (and stop synthesizing).

4. **v1 check tests updated for the stricter v2.** Two tests in this crate's
   suite changed: `check_clean_passes` now seeds a full project→arc→slice tree
   (under v2 a top-level arc with no project parent is an orphan), and
   `check_json_v1` asserts the v2 JSON envelope (which gained `severity`, `code`,
   and `errors`/`warnings` counts; `violation` → `code`). These are disclosed
   changes to `check`'s own tests as it is upgraded in this slice.

5. **JSON schema shape.** `{ ok, errors, warnings, findings: [{ severity, code,
   node, number, name, detail, fix }] }`. `ok` mirrors the exit code (pass/fail
   for the active mode), so a warning-only corpus is `ok: true` in normal mode and
   `ok: false` under `--strict`. `node`/`number`/`name` are nullable (a
   graph-spanning cycle attaches to its first member; no finding is currently
   node-less in practice).

## Uncertainties named

- **`run()` is uncovered.** The `lib.rs::run` wrapper reads the real cwd and
  writes real stdout/stderr; it cannot be exercised in-process, so its ~13 lines
  are the bulk of the coverage gap. Everything it delegates to (`dispatch`,
  `commands::*`) is covered. TOTAL line 93.69% clears the ≥90% bar; the 95%
  target would need binary-level tests (see decision 1).
- **Staleness trigger = advanced past planning.** "Advanced" means the node
  reached a gate beyond index-0 in its type's sequence; a type with no gate-set
  is exempt (advancement unjudgeable). A node merely at `planned` with an
  unsatisfied dep is *not* stale (planning ahead is legitimate).
- **Externally-blocked is not a finding.** `blocked_by` to an incomplete node is
  a legitimate state, not a violation, so `check` does not report it (only
  `Unsatisfied` → staleness and `SoftSatisfied` → soft-satisfied are surfaced).
- **`reconcile`/desired-fact drift out of scope** (Arc A5 extends `check`), and
  the `odm tear` / `decomposed`-affirm commands named in fixes are not yet
  implemented — named as affordances (the v1 pattern of naming commands that land
  later).
- **Sandbox/CI parity.** All cargo evidence was produced on a local toolchain;
  CDC should reproduce on CI / a local 1.85+ run, from a clean
  `target/llvm-cov-target`.

## Arc 02 status

With this slice closed (pending CDC), **Arc 02 is complete**: the graph engine
(slice01), cycles + tears (slice02), gates/status/evidence (slice03),
derived order + satisfaction (slice04), evidence-transition dates (slice05.1),
recomposition integrity (slice05), and now the aggregating `check` (slice06) are
all delivered. `odm` is "the build system for the plan"; the MVP needs only A3's
rollup/`orient` on top.
