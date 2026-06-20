# Closing Report — Slice 01: Workspace scaffolding

> Per LEDGER_DISCIPLINE: this is a per-row walk, not a prose summary. Every row
> gets a disposition + evidence. CC's `done` is **proposed done**; CDC re-runs
> every Verify independently before the slice advances.

- **Implementation commit:** `ac0b670` (scaffolding + docs).
- **Pure-rename commit:** `6fa30ec` (legacy relocation, precedes `ac0b670`).
- **Branch:** `slice01-workspace-scaffolding`.
- **Gates:** `make check` → exit 0; `make coverage` → exit 0.
- **Result:** 15 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.

## Per-row walk

| ID | Status | Evidence (re-runnable) |
|----|--------|------------------------|
| F-1 | done | `cargo build --workspace` → exit 0; 5 crates compile. |
| F-2 | done | `cargo metadata --no-deps … \| jq … \| sort` = `odm-cli odm-core odm-graph odm-store oxur-odm` (exact). Legacy excluded. |
| F-3 | done | `cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0. |
| F-4 | done | `cargo fmt --all -- --check` → exit 0. |
| F-5 | done | `cargo test --workspace 2>&1 \| grep -c 'test result: ok'` = 9 (≥5), all pass. |
| F-6 | done | `grep -q '^\[workspace.lints\]' Cargo.toml` matches; `grep -rl '^workspace = true' crates/*/Cargo.toml \| wc -l` = 5. |
| F-7 | done | `edition = "2024"` and `rust-version` both present in `[workspace.package]`. |
| F-8 | done | `grep -rE '"\^?[0-9]+\.' crates/*/Cargo.toml` → no matches. |
| F-9 | done | `cargo build -p oxur-odm && test -x target/debug/odm && target/debug/odm --version` → `odm 1.0.0`, exit 0. |
| F-10 | done | `make -n {build,test,lint,format,check,coverage}` → all exit 0. |
| F-11 | done | ci.yml contains `fmt`, `clippy`, `test` (three explicit jobs). |
| F-12 | done | `make coverage` → exit 0; `cargo llvm-cov` summary produced. |
| F-13 | done | `legacy/oxur-odm/Cargo.toml` exists + `name = "oxur-odm-legacy"`; `git log --follow … \| tail -1` = `e45f959 Initial commit`. |
| F-14 | done | All 5 crate names present in README ("Workspace layout" table). |
| F-15 | done | No `serde_yaml`/`serde_yml`/`serde_norway` in root `Cargo.toml`. |

## What Worked

- **Pure-rename-first relocation** kept `git --follow` intact (F-13): identical
  content move in `6fa30ec`, manifest rewrite in `ac0b670`.
- **Matching Verify commands literally** caught that the idiomatic
  `[workspace.lints.rust]` table form fails F-6's `^\[workspace.lints\]` grep;
  the dotted-key form under a literal header satisfies it with identical meaning.
- **Declare-don't-reference** for the rebuild dep set (petgraph/ulid/gix/confyg):
  unused `[workspace.dependencies]` are neither resolved nor compiled, so they
  could be pinned now with zero build cost or resolution risk.

## Decisions worth a CDC look (flagged, not silently changed)

1. **`[workspace.lints]` does not set blanket `warnings = "deny"`.** The slice-doc
   phrasing is "[workspace.lints] (rust + clippy, warnings denied)." A literal
   `rust.warnings = "deny"` in source conflicts with rust-guidelines **AP-01**
   (blanket deny-warnings in the manifest breaks future/downstream builds when a
   new lint appears). I set `rust.unsafe_code = "deny"` + `clippy.all = "deny"`
   and keep `-D warnings` enforcement in CI and `make lint` (the guideline-blessed
   location) — which is exactly what F-3 verifies. Net effect ("warnings denied
   on every build path that gates the slice") is preserved; only the *location*
   differs. Flagging per the working agreement; happy to add the blanket deny if
   CDC prefers the literal reading.

2. **Internal dependency edges are minimal.** Only `oxur-odm → odm-cli` is wired
   (needed for the `odm --version` binary, F-9). The §8 edges
   `odm-core→odm-graph`, `odm-store→odm-core`, `odm-cli→{core,store,graph}` are
   **not** wired yet: the stubs expose no API to depend on, and adding unused
   path deps would be dead wiring. These edges land in slices 02–06 as each crate
   gains real API. Consistent with "stubs only"; noted so CDC doesn't read the
   missing edges as an omission.

3. **Legacy binary renamed `odm` → `odm-legacy`.** To avoid any ambiguity with the
   new umbrella's `odm` binary. The legacy crate is excluded and not built, so
   this is cosmetic; the legacy **lib** name stays `odm`. Easily reverted.

## Uncertainties named

- **CI not executed on GitHub.** F-11 verifies the workflow *mentions* fmt/clippy/
  test; I have not pushed the branch, so I have not observed a green Actions run.
  The same commands pass locally (`make check`). Confidence high, not observed.
- **Coverage is low (57%) by construction.** F-12 only requires the harness to run
  end-to-end, which it does. The 95% target (CLAUDE.md) applies to real code;
  `run()` and `main()` are not exercised because there is nothing to exercise yet.
  Not a defect for this slice; called out so the number isn't misread.
- **`gix = "0.66"` (and other rebuild deps) are declared but unresolved.** Because
  no crate references them yet, Cargo has not resolved or compiled them, so their
  exact version compatibility is unverified until the consuming slice (04 for
  `gix`) wires them in. Pinned conservatively; may need a bump there.
