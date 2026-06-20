# Slice 01: Workspace scaffolding

> Per LEDGER_DISCIPLINE: every row reaches a final status (`done`/`deferred`/
> `no-op`) with evidence before the slice advances. `done` requires a commit SHA +
> the Verify output. CC fills Evidence at the commit where each row is met; CDC
> re-runs every Verify independently. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | Workspace builds clean | `cargo build --workspace` → exit 0 | serious | 0015 A1.1 | open | | |
| F-2 | Exactly the 5 new crates are members; legacy is NOT a member | `cargo metadata --no-deps --format-version 1 \| jq -r '.packages[].name' \| sort` == `odm-cli odm-core odm-graph odm-store oxur-odm` | correctness | 0013 §8 | open | | |
| F-3 | Clippy clean with warnings denied | `cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0 | serious | CLAUDE.md | open | | |
| F-4 | Formatting clean | `cargo fmt --all -- --check` → exit 0 | correctness | CLAUDE.md (max_width=100) | open | | |
| F-5 | Tests pass; harness non-empty (≥5 crate smoke tests) | `cargo test --workspace 2>&1 \| grep -c 'test result: ok'` ≥ 5 | serious | LEDGER (vacuous-test guard) | open | | |
| F-6 | Workspace lints are centralized and inherited | `grep -q '^\[workspace.lints\]' Cargo.toml` AND `grep -rl '^workspace = true' crates/*/Cargo.toml \| wc -l` == 5 | correctness | rust-guidelines (cargo) | open | | |
| F-7 | Edition 2024 + MSRV pinned | `grep -q 'edition = "2024"' Cargo.toml` AND `grep -q 'rust-version' Cargo.toml` | correctness | slice-doc decision | open | | |
| F-8 | Deps centralized (no version literals in crate manifests) | `grep -rE '"\^?[0-9]+\.' crates/*/Cargo.toml` → no dependency-version matches (deps use `.workspace = true`) | correctness | rust-guidelines (cargo) | open | | |
| F-9 | Umbrella builds the `odm` binary and runs | `cargo build -p oxur-odm && test -x target/debug/odm && target/debug/odm --version` → prints version, exit 0 | serious | 0013 §8 | open | | |
| F-10 | Makefile exposes the standard targets | `for t in build test lint format check coverage; do make -n $t >/dev/null; done` → all exit 0 | polish | CLAUDE.md | open | | |
| F-11 | CI runs fmt + clippy + test | `grep -Eiq 'fmt' .github/workflows/*.y*ml && grep -Eiq 'clippy' .github/workflows/*.y*ml && grep -Eiq 'test' .github/workflows/*.y*ml` | correctness | CLAUDE.md | open | | |
| F-12 | Coverage harness runs end-to-end | `make coverage` → exit 0, report produced | polish | CLAUDE.md (95% target) | open | | |
| F-13 | Legacy crate preserved, not deleted; relocated + renamed + excluded | `test -f legacy/oxur-odm/Cargo.toml && grep -q 'name = "oxur-odm-legacy"' legacy/oxur-odm/Cargo.toml` AND `git log --oneline --follow legacy/oxur-odm/Cargo.toml \| tail -1` shows pre-move history | serious | 0001 (supersede-don't-delete) | open | | |
| F-14 | README documents the workspace layout | `for c in oxur-odm odm-cli odm-core odm-store odm-graph; do grep -q "$c" README.md; done` → all present | polish | methodology (legible-from-fs) | open | | |
| F-15 | No YAML frontmatter lib pinned yet (deferred to slice03) | `! grep -Eq 'serde_yaml|serde_yml|serde_norway' Cargo.toml` | polish | slice-doc decision | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at commit `<SHA>` on `<date>`. CDC verification: `<name/session>`.
Total rows: 15. Done: _. Deferred: _. No-op: _.
