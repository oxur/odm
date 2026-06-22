# CDC Verification — Slice 01: Workspace scaffolding

> Independent verification of CC's closed ledger (commit `ac0b670`; ledger closed
> `2c7aede`), per LEDGER_DISCIPLINE CDC protocol. CDC ≠ implementer. Evidence
> levels (ODD-0001 D3): **reproduced** = CDC re-ran the Verify and observed the
> result; **attested** = CC's evidence, not independently reproduced here.

## Environment constraint (disclosed)

CDC ran in the Cowork sandbox, which has **no Rust toolchain** (`cargo`/`rustc`/
`rustup` absent; apt offers only cargo 1.75, *below* the 1.85 floor edition 2024
requires) and **cannot execute CC's binary** (built for a different platform —
Exec-format error). Therefore the **compile/test-gated rows cannot be reproduced
here.** They are routed to CI (row F-11's workflow) or a local 1.85+ `make check`
as the reproduction of record. This is a CDC-environment limitation, not a defect
in CC's work — and a standing process note: *CDC of compile-gated Rust rows needs a
toolchain environment.*

## Row dispositions

**Row count:** 15 opened, 15 addressed in the closing report. No silent drops. ✔

**Reproduced by CDC (structural — re-run in-session):**
F-2 (members are exactly the 5 crates; legacy excluded — read from `Cargo.toml`),
F-6 (`[workspace.lints]` header present; 5 crates inherit; lints non-vacuous:
`rust.unsafe_code="deny"`, `clippy.all="deny"`), F-7, F-8, F-9 *(code structure:
umbrella `main.rs` delegates to `odm_cli::run()`; clap `version` wired — the
binary's run path is correct; the **execution** is attested, see below)*, F-10
(all 6 make targets dry-run clean), F-11 (CI has fmt/clippy/test), F-13 (legacy
renamed `oxur-odm-legacy`; `--follow` history → `e45f959`), F-14, F-15. → **PASS.**

**Attested by CC, pending reproduction in CI / 1.85+ local:**
F-1 (build), F-3 (clippy `-D warnings`), F-4 (fmt `--check`), F-5 (tests),
F-9 (binary *runs* `--version`), F-12 (coverage). CC's evidence is internally
consistent and candid (e.g. F-12 coverage 57% flagged as stub-driven). Not
independently reproduced here for the environment reason above. → **PENDING CI.**

## Rulings on CC's three flagged deviations

1. **Lints location (warnings deny in CI, not blanket in-manifest).** **Accepted.**
   F-3's criterion — `clippy … -D warnings` → exit 0 — is satisfied by the
   CI/Makefile invocation regardless of where the flag lives; denying meaningful
   lints (`unsafe_code`, `clippy.all`) in-manifest while gating `-D warnings` in CI
   is sound idiom that avoids breaking local dev on every warning. Ruled on
   engineering merits (not on the exact "AP-01" label). No change requested.
2. **Minimal internal crate edges (only `oxur-odm → odm-cli`).** **Accepted.** The
   ODD-0013 §8 edges between core/store/graph materialize when there is real API to
   depend on (slices 02–06). Correct for a skeleton; not an omission.
3. **Legacy binary renamed `odm → odm-legacy`.** **Accepted.** Cosmetic; legacy is
   excluded and not built; avoids a binary-name collision with the new umbrella.

## Verdict

Slice 01 is **CDC-verified on structure, with all three deviations accepted.** The
6 compile/test rows are **attested, pending CI reproduction.** Recommended path to
full closure: push `slice01-workspace-scaffolding` → CI reproduces F-1/3/4/5/9/12 →
if green, the slice is fully CDC-closed and slice02 (identity core) may open.
Nothing is on `main`; no risk while this completes.

CDC: planning thread, 2026-06-20. Iterations used: 1.

## Closure update (2026-06-22)

**CI is green.** The previously-attested compile/test rows (F-1 build, F-3 clippy,
F-4 fmt, F-5 test, F-9 binary-run, F-12 coverage) are now **reproduced** by an
independent CI run. **Slice 01 is fully CDC-closed.** (CI-green taken on operator
confirmation; CDC did not read the CI log directly — the reproduction is CI's, not
a CDC re-run. Evidence level: `reproduced` via CI.)
