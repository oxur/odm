# Slice 01 (Arc 03): Arc 02 cleanup

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+ (sandbox has no
> 1.85 toolchain). CDC-authored acceptance rows; CC fills Status/Evidence/Notes per
> commit. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| C-1 | `tears` entries are a typed struct carrying the torn `edge` (a `Dependency`) **and** the `because` rationale (was a bare `Vec<Dependency>`) | `cargo test -p odm-core tear_carries_rationale` → ok | serious | slice07 CDC #1 / 0013 §4.3 | open | | Name the persistence type distinctly from odm-graph's pure `Tear<N>` (cycle.rs). |
| C-2 | `odm tear X depends_on Y --because <r>` **persists** the rationale on source X (not just validates it) | `cargo test -p odm-cli tear_persists_rationale` (in-process dispatch) → ok | serious | slice07 CDC #1 | open | | |
| C-3 | A populated `tears` round-trips: `parse ∘ emit = identity` | `cargo test -p odm-core tears_roundtrip_identity` (proptest) → ok | serious | 0013 §2.3 | open | | |
| C-4 | Empty `tears` is omitted on emit; arc01/02 nodes (no tears) round-trip byte-identically | `cargo test -p odm-core empty_tears_roundtrip` → ok | correctness | back-compat | open | | CC: confirm no on-disk node carries a bare-form tear; if so, note migration is a no-op (pre-release). |
| C-5 | `check`'s active-tears listing surfaces each tear's rationale | `cargo test -p odm-cli check_lists_tear_rationale` → ok | serious | 0013 §4.3 | open | | |
| C-6 | A binary-level `assert_cmd` suite exists in `oxur-odm/tests/` and exercises the real `odm` process (`run()`) end-to-end | `cargo test -p oxur-odm --test cli` → ok | serious | slice06/07 CDC | open | | First tests to reach `run()` / real `ExitCode`. |
| C-7 | Real `odm check` exits `EXIT_OK` (0) on a clean graph and `EXIT_VIOLATIONS` (1) on violations | `cargo test -p oxur-odm --test cli check_exit_codes` → ok | serious | slice06 CDC #1 | open | | Verifies the `u8`→`ExitCode` path end-to-end, not just the unit mapping. |
| C-8 | Recomposition severities recalibrated: orphan + decomposition-drift = `Error`; undeveloped-stub + advanced-without-decomposition = `Warning` | `cargo test -p odm-cli check_recomposition_severities` → ok | serious | slice06 CDC #2 | open | | |
| C-9 | An undeveloped-stub / advanced-without-decomposition finding does **not** fail default `check` (exit 0) but **does** fail `--strict` (exit 1) | `cargo test -p odm-cli stub_warns_default_fails_strict` → ok | serious | slice06 CDC #2 | open | | Matches staleness / soft-satisfaction treatment. |
| C-10 | 0013 reconciled: §2.3/§4.3 show `tears` carrying a `because` rationale; §2.3/§4.5 show `decomposed` as typed `Decomposition { on, children }` (not the scalar) | `grep -n 'because' docs/design/01-draft/0013-odm-architecture-design.md` (tears example/§4.3) AND `grep -n 'Decomposition { on, children }' docs/design/01-draft/0013-odm-architecture-design.md` → both match | polish | doc-honesty | open | | Doc row — grep matches the reconciled schema. Code already flags the mismatch (frontmatter.rs:401). |
| C-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-core, odm-cli, and oxur-odm | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/*/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | open | | The `assert_cmd` suite (C-6/C-7) should finally cover `run()` in oxur-odm. |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Total rows: 11.)_
