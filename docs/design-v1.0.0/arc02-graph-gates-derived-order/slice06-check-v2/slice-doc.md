# Slice 06 (Arc 02) — `check` v2 (plan-of-record)

> Refs: ODD-0013 §7 (command surface), §4 (graph), §4.4/§4.5. ODD-0001 B3/F3.
> `depends_on:` arc02 slices 01–05 (it aggregates their predicates) + arc01 slice06
> (`check` v1 = schema + link-integrity).

## Goal

Make `check` the single mechanical gate the framework's disciplines collapse into.
**Done when `odm check` aggregates every graph-level invariant, returns meaningful
exit codes, supports a strict/CI mode, names the fix in every error, and offers
`--json`.** This is the command that lets the framework retire prose rules in favor
of "run `odm check`."

## Scope

**In:** aggregate, over the whole graph — (a) schema + link-integrity (from v1),
(b) cycles-without-tears (slice02), (c) out-of-order / staleness (slice04),
(d) recomposition integrity (slice05), (e) **below-threshold satisfaction**
(slice04, soft-satisfied) — plus exit codes (`0` ok, `1` violations, `2` usage);
a `--strict` / CI mode that promotes warnings (staleness, soft-satisfaction) to
failures; **errors-as-affordances** (each finding names the exact command to fix
it); `--json` report.

**Out:** `reconcile` / desired-fact drift (Arc A5 adds that check later); the
derived-order *queries* themselves (slice04 — `check` consumes their predicates).

## Verification

`cargo test -p odm-cli -p odm-core` green; `assert_cmd` exit-code tests; clippy
`-D warnings`; coverage ≥ 90%. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). **Arc 02
complete** — `odm` is now "the build system for the plan" (MVP needs only A3's
rollup/orient on top).
