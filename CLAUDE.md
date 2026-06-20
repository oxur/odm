# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repository.

## Project Overview

`odm` is a command-line **ODD Document Manager**: it manages design documents
with YAML frontmatter, a state lifecycle, git integration, and automatic
indexing. It was extracted from the [Oxur](https://github.com/oxur/oxur)
monorepo so it can be used independently by any project.

- Package: `oxur-odm` (crates.io) — binary `odm`, library `odm` (`use odm::…`).
- The library was named `design` in the Oxur era; it is now `odm`.

## Workspace Structure

Cargo workspace (resolver v2, edition 2024, MSRV 1.85, `max_width = 100`).
Member crates, in dependency / publish order (see ODD-0013 §8):

| Crate | Binary | Purpose |
|-------|--------|---------|
| **odm-graph** | — | Pure DAG/tree engine over abstract ids: edges, topo-sort, cycles, readiness. |
| **odm-core** | — | Domain model: node types, ULID identity, frontmatter schema, edge & gate semantics. |
| **odm-store** | — | Persistence: `nodes/YYYY/MM/<ULID>.md`, atomic writes, git (`gix`), `odm.toml`. |
| **odm-cli** | — | clap command surface, `--json`, output (oxur-cli / tabled). |
| **oxur-odm** | `odm` | Umbrella: publishes the `odm` binary; re-exports the library API. |

> **v1.0.0 rebuild in progress.** These crates are currently stubs (the `odm`
> binary reports `--version` only); real behavior lands slice by slice under
> `docs/design-v1.0.0/`. The pre-rebuild crate is preserved at
> `legacy/oxur-odm` (package `oxur-odm-legacy`), excluded from the workspace and
> kept as the harvest source — do not delete it. `odm-index` / `odm-reconcile`
> / `odm-migrate` are deferred to the arcs that need them (not yet created).

Shared dependency versions live in `[workspace.dependencies]`; crate manifests
reference them with `<dep>.workspace = true` (no version literals). Lints are
centralized in `[workspace.lints]` and inherited via `[lints] workspace = true`.

The only external Oxur dependency is **`oxur-cli`** (crates.io, ≥0.2.1), used
with `default-features = false` for its library UI helpers only
(`common::output`, `table`). Do not enable its `binary` feature — that pulls in
the entire Oxur language/compiler stack.

## Build & Development Commands

```bash
make build          # build the binary into ./bin/
make build-release  # optimized release build
cargo check         # type-check

make test           # all tests (cargo test --all-features --workspace)
cargo test --package oxur-odm

make coverage       # coverage summary (target: 95%+)
make coverage-html

make lint           # clippy -D warnings + rustfmt --check
make format         # apply rustfmt

make check          # build + lint + test
```

## Conventions

- **Errors** carry source position; parse/build errors include `Position`.
- **CLI output** uses `oxur_cli::common::output::{success, error, info, warning}`
  and `oxur_cli::table` for tables.
- **Testing:** 95%+ coverage target; `proptest` for invariants (per-crate
  `proptest-regressions/`); integration tests in each crate's `tests/`. Test
  naming: `test_<function>_<scenario>_<expectation>`. Tests that mutate process
  globals (env vars, cwd) must be `#[serial]` (`serial_test`).
- **Cargo.lock is committed** — `odm` is a binary application.

## Rust Skill Guidelines

Before writing or reviewing Rust, load the Rust skill if available
(`assets/ai/ai-rust/skills/claude/SKILL.md`), starting with the anti-patterns and
core-idioms guides. If it isn't present, ask before cloning.

## Git Conventions

- Imperative, descriptive commit messages; explain *why*.
- Before submitting: `make test` + `make lint` + `make format` (+ `make coverage`).
- `make push` pushes `main` and tags to `origin` (GitHub).
