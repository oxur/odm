# CDC Verification — Arc 03 / Slice 01: Arc 02 cleanup

> Independent verification of CC's closed ledger (impl `1baa3b8`; closed `f5e3236`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran in
> this session; **attested** = CC's evidence (incl. CC's local 1.95.0 run), not
> independently reproduced here.

## Environment constraint (disclosed)

The Cowork sandbox has no 1.85+ Rust toolchain (apt cargo is 1.75). Cargo
build/test/clippy/coverage *executions* route to CI / a local 1.85+ toolchain; CDC
reproduces structural rows (type shapes, control flow, file existence/content, doc
text, grep invariants) by reading the branch at `1baa3b8`. CC reports reproducing
all cargo rows locally on rustc 1.95.0 — a strong signal, but CC's own machine is
not the independent gate, so those rows are **attested pending CI** (the operating
rhythm's `attested → reproduced` flip happens on Duncan's push to green CI).

## Row dispositions

**Row count:** 11 opened, 11 addressed in the closing report. No silent drops. ✔

**Reproduced by CDC (structural, at `1baa3b8`):**

- **C-1** — `Edges::tears` is `Vec<TornEdge>`; `pub struct TornEdge { edge:
  Dependency, because: String }` (`frontmatter.rs:458,504`), named distinctly from
  odm-graph's pure `Tear<N>` (`cycle.rs`). The collision I flagged in drafting is
  avoided. ✔
- **C-4** — the field keeps `skip_serializing_if = "Vec::is_empty"`, so empty `tears`
  is omitted on emit. ✔
- **C-5** — `check.rs` link-integrity adapted to the new shape (`.map(|t|
  dependency_target(&t.edge))`); the CLI surfaces an `active tears (N):` listing with
  rationale + an additive `--json` `tears` array. ✔
- **C-6 / C-7** — `crates/oxur-odm/tests/cli.rs` exists and drives
  `Command::cargo_bin("odm")` with `current_dir` on a `TempDir`, asserting `code(0)`
  (clean), `code(1)` (orphan), `code(2)` (usage), the `--json` envelope, and tear
  rationale surfacing through the real binary — the first tests to reach `run()` /
  the real `ExitCode`. dev-deps added to `oxur-odm/Cargo.toml`. ✔ (existence +
  content; *execution* pending CI)
- **C-8 / C-9** — `recompose_severity(&Issue)`: `UndevelopedStub |
  AdvancedWithoutDecomposition => Warning`, all else (orphan, decomposition-drift)
  `=> Error` (`commands.rs`), with a rationale comment. Exactly the recalibration the
  slice06 CDC recommended; matches the staleness / soft-satisfaction model. ✔
- **C-10** — 0013 bumped `1.7 → 1.8` (`updated: 2026-06-25`); `because` present in the
  §2.3 schema example (l.105) and §4.3 prose (l.171–172); typed `Decomposition { on,
  children }` in §4.5 (l.217). ✔ *(CDC follow-up applied — see below.)*
- **C-11 (no `unsafe`)** — `grep -RnE '\bunsafe\b' crates/*/src` → no matches. ✔

**Attested by CC (local rustc 1.95.0), pending CI for the independent gate:**
the cargo executions — `cargo test --workspace --all-features` → 187 passed; clippy
`--all-targets -- -D warnings` → exit 0; `fmt --check` clean; line coverage
**odm-core 98.68% / odm-cli 91.94% / oxur-odm 100%** (TOTAL 94.65%). All three crates
clear the 90% floor; oxur-odm 100% reflects the new `assert_cmd` suite closing the
`run()` gap. → **PENDING CI.**

## Rulings on CC's flagged items

1. **C-7 split into named tests** (`check_exit_ok_on_clean_graph` +
   `check_exit_violations_on_dirty_graph` + `usage_error_exits_two`) rather than one
   `check_exit_codes`. **Accepted** — strictly stronger and clearer; the criterion
   (exit codes verified end-to-end through the real process) is fully met, and the
   exit-2 usage path is a bonus.
2. **`--json` envelope gained an additive `tears` array.** **Accepted** — existing
   `ok`/`errors`/`warnings`/`findings` keys unchanged; v2 consumers still parse.
   *Forward note:* this is the **second** additive evolution of `check --json`
   (slice06 added severity/code/counts; this adds `tears`). **Slice04** (the `--json`
   schema doc) must pin the current `check` envelope as the canonical v2 schema, and
   ODD-0017 export consumers should target it.
3. **Bare-form-tear migration is a no-op** (no `nodes/` dir; no on-disk `tears:`; four
   test fixtures updated). **Accepted** — reproduced by the same grep. Making
   `TornEdge`'s rationale always-required (no back-compat empty-`because`
   deserializer) is sound *because* the evidence shows no persisted bare data —
   correct evidence-gated judgment, not a shortcut.
4. **Toolchain parity.** **Noted, not a deduction.** CC's local 1.95.0 run is a strong
   positive signal; I still hold the cargo rows as attested-pending-CI to preserve the
   CC/CDC independence boundary. Re-run on CI from a clean `target/llvm-cov-target`.

## CDC follow-up applied (doc honesty)

CC disclosed leaving the **Q-7 decisions appendix** (0013 l.406) reading the stale
`decomposed: complete` scalar, because it was outside the cc-prompt's named scope
(§2.3/§4.3/§4.5) — correct restraint (don't silently widen scope). But it is the same
doc-honesty class C-10 targets, and a one-token fix, so as CDC/doc-author I corrected
it to the typed `decomposed: Decomposition { on, children }` form. 0013 is now honest
in all three occurrences (l.110 schema, l.219 reconciliation note, l.406 appendix).

## Verdict

**Arc 03 / Slice 01 CDC-verified on structure; all flags accepted; cargo rows pending
CI.** The three Arc 02 follow-ups are closed and 0013 is honest. On CI green
(`attested → reproduced`), the slice closes. **Unblocks slice02** — tear rationale is
persisted and renderable, and `check` severities are calibrated before `orient`
surfaces them.

CDC: planning thread, 2026-06-25. Iterations used: 1.
