# Slice 01 — Workspace scaffolding (plan-of-record)

> Per-slice implementation plan (SDLC step 5). The *plan*; `ledger.md` is the
> grep-verifiable acceptance, `cc-prompt.md` is the assignment CC executes.
> Refs: ODD-0013 §8, ODD-0015 §3 (A1.1), CLAUDE.md, rust-guidelines (project-
> structure / cargo). `depends_on:` nothing.

## Goal

Convert the single legacy crate into the v1.0.0 multi-crate workspace skeleton with
shared metadata, workspace-wide lints, pinned toolchain, build/CI tooling, and
empty-but-compiling crates. **Done when `make check` is green on the new
workspace** and the legacy code is preserved (not deleted) for harvesting.

No domain logic in this slice — stubs only. Identity/model/store/CLI come in
slices 02–06.

## Decisions baked into this slice (flag any you'd veto)

- **Target version `1.0.0`** (the "v-major" rebuild). Design dir is
  `docs/design-v1.0.0/`. (Reversible: rename if you'd rather 0.4.0.)
- **Edition 2024** (toolchain 1.85+; current `rust-toolchain.toml` = stable, which
  satisfies it). Bumped from the legacy 2021 because this is a ground-up rebuild and
  the rust-guidelines target 2024. Update `CLAUDE.md` (which still says 2021).
- **Resolver 2** retained (consider resolver 3 — defer unless trivial).
- **Crate set this slice:** `oxur-odm` (umbrella, owns the `odm` binary),
  `odm-cli`, `odm-core`, `odm-store`, `odm-graph` (stub). `odm-index`/`-reconcile`/
  `-migrate` are NOT created yet (added when arcs 04/05/06 open).
- **Legacy crate is relocated, not deleted:** `git mv crates/oxur-odm
  legacy/oxur-odm`, rename its package to `oxur-odm-legacy`, exclude from the
  workspace. It stays as the harvest source (config/git/markdown utils) and git
  history is preserved (honors supersede-don't-delete).

## Scope

**In:** root `Cargo.toml` (`[workspace]` members, `[workspace.package]`,
`[workspace.dependencies]`, `[workspace.lints]`); the 5 crate dirs each with a
`Cargo.toml` (inheriting workspace metadata + lints) and a minimal `lib.rs`/`main.rs`
plus one smoke test; `oxur-odm` umbrella exposing the `odm` binary that prints
`--version`; `rust-toolchain.toml` (already present); `.rustfmt.toml` (`max_width =
100`, present); `Makefile` targets (reuse/adapt the migrated one); CI workflow
(reuse/adapt `.github`) running fmt + clippy `-D warnings` + test; `README.md`
documenting the workspace layout; legacy relocation.

**Out:** ULID/identity, node model, frontmatter, store, real CLI commands, graph —
all later slices. No new third-party deps beyond declaring the shared set.

## Shared dependencies to declare (in `[workspace.dependencies]`)

Carry forward the still-relevant legacy deps and add the rebuild's:
`serde`, `serde_json`, `toml`, `anyhow`, `thiserror`, `chrono`, `clap`, `walkdir`,
`sha2`, `tabled`, `colored`, `oxur-cli` (default-features = false), plus **`petgraph`**
(graph), **`ulid`** (ids), **`gix`** (git), `confyg` (config). Dev: `assert_cmd`,
`predicates`, `tempfile`, `proptest`, `serial_test`, `cargo-llvm-cov`.

- **Drop `uuid`** (ULID replaces it) and **`unicode-normalization`/`regex`/`glob`**
  unless a later slice needs them (add when needed).
- **Defer the YAML-parser choice to slice03:** `serde_yaml` is archived/unmaintained
  (cf. ODD-0014's note on archived crates). Do NOT pin a frontmatter YAML lib here;
  slice03 chooses it deliberately. This slice needs no YAML.

## Steps (each maps to a ledger row)

1. Relocate legacy crate (`git mv`), rename package to `oxur-odm-legacy`, exclude.
2. Write root `Cargo.toml`: members (5), `[workspace.package]` (version 1.0.0,
   edition 2024, rust-version 1.85, authors/license/repository),
   `[workspace.dependencies]` (above), `[workspace.lints]` (rust + clippy, warnings
   denied).
3. Create the 5 crates: `Cargo.toml` (inherit `package`/`lints`), minimal
   `lib.rs`/`main.rs`, one `#[test] fn smoke()` each so the test/coverage harness is
   non-empty. `oxur-odm` declares `[[bin]] name = "odm"` and prints `--version`.
4. Adapt `Makefile` (build/test/lint/format/check/coverage) and the `.github` CI to
   the workspace.
5. Update `README.md` (workspace layout) and `CLAUDE.md` (edition 2024; new crate map).
6. Green the gates: `make check` + `make coverage`.

## Verification

`make check` (build + clippy `-D warnings` + fmt --check + test) exits 0;
`make coverage` produces a report; `odm --version` runs; `cargo metadata` lists the
5 members and not the legacy crate. Full grep-verifiable criteria in `ledger.md`.

## Exit

`ledger.md` fully closed (every row `done`/`deferred`/`no-op` with evidence), CDC
verified, `make check` green. Then slice02 (identity core) opens.
