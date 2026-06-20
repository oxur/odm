# Slice 01: Workspace scaffolding

> Per LEDGER_DISCIPLINE: every row reaches a final status (`done`/`deferred`/
> `no-op`) with evidence before the slice advances. `done` requires a commit SHA +
> the Verify output. CC fills Evidence at the commit where each row is met; CDC
> re-runs every Verify independently. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | Workspace builds clean | `cargo build --workspace` → exit 0 | serious | 0015 A1.1 | done | `ac0b670`: `cargo build --workspace` → exit 0 (all 5 crates compile). | |
| F-2 | Exactly the 5 new crates are members; legacy is NOT a member | `cargo metadata --no-deps --format-version 1 \| jq -r '.packages[].name' \| sort` == `odm-cli odm-core odm-graph odm-store oxur-odm` | correctness | 0013 §8 | done | `ac0b670`: output = `odm-cli odm-core odm-graph odm-store oxur-odm` (exact). Legacy excluded via `[workspace] exclude`. | |
| F-3 | Clippy clean with warnings denied | `cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0 | serious | CLAUDE.md | done | `ac0b670`: clippy → exit 0. `-D warnings` enforcement lives here + in CI/Makefile (not as blanket `warnings="deny"` in `[workspace.lints]` — AP-01). | |
| F-4 | Formatting clean | `cargo fmt --all -- --check` → exit 0 | correctness | CLAUDE.md (max_width=100) | done | `ac0b670`: `cargo fmt --all -- --check` → exit 0. `.rustfmt.toml` bumped to edition 2024. | |
| F-5 | Tests pass; harness non-empty (≥5 crate smoke tests) | `cargo test --workspace 2>&1 \| grep -c 'test result: ok'` ≥ 5 | serious | LEDGER (vacuous-test guard) | done | `ac0b670`: count = 9 (5 crate smoke tests + 4 doctest harness lines), all pass. | Smoke tests assert `CARGO_PKG_NAME`; odm-cli asserts clap `command().debug_assert()`. |
| F-6 | Workspace lints are centralized and inherited | `grep -q '^\[workspace.lints\]' Cargo.toml` AND `grep -rl '^workspace = true' crates/*/Cargo.toml \| wc -l` == 5 | correctness | rust-guidelines (cargo) | done | `ac0b670`: header present (match=yes); 5 crate manifests inherit (`[lints] workspace = true`). | Used compact dotted form (`rust.unsafe_code`, `clippy.all`) under literal `[workspace.lints]` to satisfy the exact grep. |
| F-7 | Edition 2024 + MSRV pinned | `grep -q 'edition = "2024"' Cargo.toml` AND `grep -q 'rust-version' Cargo.toml` | correctness | slice-doc decision | done | `ac0b670`: both present in `[workspace.package]` (edition 2024, rust-version 1.85). | |
| F-8 | Deps centralized (no version literals in crate manifests) | `grep -rE '"\^?[0-9]+\.' crates/*/Cargo.toml` → no dependency-version matches (deps use `.workspace = true`) | correctness | rust-guidelines (cargo) | done | `ac0b670`: grep → no matches. External deps use `.workspace = true`; umbrella's internal dep is a path dep (`{ path = "../odm-cli" }`). | |
| F-9 | Umbrella builds the `odm` binary and runs | `cargo build -p oxur-odm && test -x target/debug/odm && target/debug/odm --version` → prints version, exit 0 | serious | 0013 §8 | done | `ac0b670`: `target/debug/odm --version` → `odm 1.0.0`, exit 0. Routed through `odm_cli::run()` (clap `#[command(version)]`). | |
| F-10 | Makefile exposes the standard targets | `for t in build test lint format check coverage; do make -n $t >/dev/null; done` → all exit 0 | polish | CLAUDE.md | done | `ac0b670`: all 6 targets dry-run exit 0 (migrated Makefile already workspace-aware). | |
| F-11 | CI runs fmt + clippy + test | `grep -Eiq 'fmt' .github/workflows/*.y*ml && grep -Eiq 'clippy' .github/workflows/*.y*ml && grep -Eiq 'test' .github/workflows/*.y*ml` | correctness | CLAUDE.md | done | `ac0b670`: all three present. ci.yml rewritten into explicit `fmt` / `clippy` (`-D warnings`) / `test` jobs. | |
| F-12 | Coverage harness runs end-to-end | `make coverage` → exit 0, report produced | polish | CLAUDE.md (95% target) | done | `ac0b670`: `make coverage` → exit 0, per-crate `cargo llvm-cov` summary produced (TOTAL 57.14%). | 95% target is aspirational for real code; stubs leave `run()`/`main()` uncovered. |
| F-13 | Legacy crate preserved, not deleted; relocated + renamed + excluded | `test -f legacy/oxur-odm/Cargo.toml && grep -q 'name = "oxur-odm-legacy"' legacy/oxur-odm/Cargo.toml` AND `git log --oneline --follow legacy/oxur-odm/Cargo.toml \| tail -1` shows pre-move history | serious | 0001 (supersede-don't-delete) | done | `ac0b670`: file present + renamed to `oxur-odm-legacy`; `--follow` tail = `e45f959 Initial commit` (history survives). | Pure-rename commit `6fa30ec` precedes content changes so rename detection is 100%. |
| F-14 | README documents the workspace layout | `for c in oxur-odm odm-cli odm-core odm-store odm-graph; do grep -q "$c" README.md; done` → all present | polish | methodology (legible-from-fs) | done | `ac0b670`: all 5 crate names present (new "Workspace layout" table). | |
| F-15 | No YAML frontmatter lib pinned yet (deferred to slice03) | `! grep -Eq 'serde_yaml|serde_yml|serde_norway' Cargo.toml` | polish | slice-doc decision | done | `ac0b670`: no YAML lib in root Cargo.toml (dropped `serde_yaml`; choice deferred to slice03). | Legacy crate keeps `serde_yaml` in its excluded manifest (not the workspace root; grep targets root only). |

## What Worked

- **Two-commit relocation.** Doing a pure `git mv` (commit `6fa30ec`, identical
  content) *before* rewriting the legacy manifest (commit `ac0b670`) kept rename
  detection at 100%, so `--follow` (F-13) traverses the move cleanly. Rewriting
  the manifest in the same commit as the move would have risked breaking it.
- **Reading the Verify command literally.** F-6's grep is `^\[workspace.lints\]`;
  the idiomatic `[workspace.lints.rust]` / `[workspace.lints.clippy]` table form
  does *not* match it. Switching to the compact dotted form (`rust.unsafe_code`,
  `clippy.all`) under a literal `[workspace.lints]` header satisfied the exact
  grep with identical semantics — a reminder that grep-verifiable criteria are
  matched as written, not by intent.
- **Declaring shared deps without referencing them.** `[workspace.dependencies]`
  entries that no member uses are not resolved or compiled, so the rebuild's
  dep set (petgraph/ulid/gix/confyg) could be pinned now without slowing the
  build or forcing version-resolution risk before the consuming slices exist.

## Closure

Closed at commit `ac0b670` on 2026-06-20. CDC verification: _pending_ (CC
proposes done; CDC re-runs every Verify independently before the slice advances).
Total rows: 15. Done: 15. Deferred: 0. No-op: 0.
