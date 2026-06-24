# Slice 06 (Arc 01): `check` v1 + link-integrity

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| L-1 | `check` flags a node missing a required field for its type | `cargo test -p odm-cli check_missing_field` (in-process; see note) → ok | serious | 0013 §2.3 | done | `5b39c54`: `check_missing_field` → 1 passed. A node with a whitespace-only `name` → finding `missing-field`, exit code 1. | Tests drive `odm_cli::dispatch` in-process (not `assert_cmd`), per the slice05 user-approved pattern; see decision (3). v1 required field = non-empty `name` — decision (1). |
| L-2 | Link-integrity: a dangling `part_of` reference is flagged | `cargo test -p odm-cli check_dangling_part_of` → ok | serious | 0011 R6 / 0001 A3 | done | `5b39c54`: `check_dangling_part_of` → 1 passed. `part_of` → absent id → `dangling-part_of`, exit 1. | |
| L-3 | Link-integrity: a dangling `supersedes`/edge reference is flagged | `cargo test -p odm-cli check_dangling_edge` → ok | serious | 0011 R6 | done | `5b39c54`: `check_dangling_edge` → 1 passed. `supersedes` → absent id → `dangling-edge`, exit 1. All 8 edge kinds link-checked (odm-core `all_edge_kinds_are_link_checked`). | |
| L-4 | Supersession-chain integrity: a self-supersede or a cyclic chain is flagged | `cargo test -p odm-cli check_supersession_chain` → ok | serious | 0013 §3 | done | `5b39c54`: `check_supersession_chain` → 1 passed (self-supersede). Cycle detection covered by odm-core `supersession_cycle_is_flagged_once`; terminating chains pass. | |
| L-5 | A clean corpus passes with exit `0` | `cargo test -p odm-cli check_clean_passes` → ok | serious | 0013 §7 | done | `5b39c54`: `check_clean_passes` → 1 passed. Parent + child (part_of resolves) → `check: ok`, exit code 0. | |
| L-6 | Exit codes: `0` clean, `1` violations, `2` usage error | `cargo test -p odm-cli check_exit_codes_v1` → ok | serious | 0013 §7 | done | `5b39c54`: `check_exit_codes_v1` → 1 passed. clean → 0; violations → 1; an unknown flag is rejected by clap (→ exit 2). | `dispatch` returns the `u8` code; `run` maps `Err`→2. See decision (2). |
| L-7 | Every finding names the exact fix command (errors-as-affordances) | `cargo test -p odm-cli check_errors_name_fix_v1` → ok | serious | 0013 §7 / 0001 | done | `5b39c54`: `check_errors_name_fix_v1` → 1 passed. Every finding prints a `fix:` line; the empty-name finding names `odm rename`. | See decision (4): some fixes are precise file edits (the `link`/`unlink` mutators are Arc 02). |
| L-8 | `--json` report with a stable, documented schema | `cargo test -p odm-cli check_json_v1` (snapshot) → ok | correctness | 0013 §7 | done | `5b39c54`: `check_json_v1` → 1 passed. `{ok, findings:[{node,number,name,violation,detail,fix}]}`; clean → `ok:true`, empty findings. | Schema = `CheckReport`/`FindingJson` (documented in `commands.rs`). |
| L-9 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% | `cargo clippy -p odm-cli -p odm-core --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-cli -p odm-core --summary-only --ignore-filename-regex '(odm-store|odm-graph)/'` → **line** ≥ 90% (target 95%) | serious | CLAUDE.md | done | `5b39c54`: clippy → exit 0; no `unsafe`; coverage **line 95.84%**, region 92.49% (TOTAL, store/graph excluded; `check.rs` 99.07% line). | |

## What Worked

- **Validator in `odm-core`, presentation in `odm-cli`.** Per §8, the structural
  rules are a pure function — `odm_core::check::check(&[Frontmatter]) ->
  Vec<Finding>` — with no I/O and no CLI knowledge. odm-cli loads the corpus,
  maps each `Violation` to a fix affordance (CLI vocabulary), renders, and
  chooses the exit code. This split made the rules trivially unit-testable
  (odm-core, 99% line) and keeps the affordance/exit-code policy where it
  belongs. It also matches L-9's two-crate coverage scope exactly.
- **`#[non_exhaustive] Violation` + a `check_*` helper per family** mean `check`
  v2 (Arc 02: cycles-without-tears, staleness, recomposition) *extends* this —
  add a validator that pushes findings, add an enum variant — without rewriting
  v1. The CLI's `violation_label`/`_detail`/`_fix` already have a catch-all arm.
- **`dispatch` returns the exit code (`u8`), `run` returns `ExitCode`.** Making
  the code a return value (not a `process::exit`) kept the whole surface
  testable in-process: the check tests assert `0` vs `1` directly, and clap owns
  the `2` (usage) path. Clean three-way split with no global state.
- **Deterministic findings (id order)** made the `--json` snapshot and the
  multi-finding assertions stable despite random ULIDs.

## Closure

Closed at `5b39c54` on 2026-06-24. CDC: _pending_ (cargo rows via CI / local
1.85+; sandbox has no toolchain). Total rows: 9. Done: 9. Deferred: 0.
No-op: 0. **Arc 01 complete on close** (identity, schema, store, CRUD,
structural check).
