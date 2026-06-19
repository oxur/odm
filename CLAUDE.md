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

Cargo workspace (resolver v2, edition 2021, `max_width = 100`):

| Crate | Binary | Purpose |
|-------|--------|---------|
| **oxur-odm** | `odm` | The document manager: model, index, state machine, config, CLI |

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
- **Testing:** 95%+ coverage target; `proptest` for invariants (regressions in
  `crates/oxur-odm/proptest-regressions/`); integration tests in
  `crates/oxur-odm/tests/`. Test naming: `test_<function>_<scenario>_<expectation>`.
- **Cargo.lock is committed** — `odm` is a binary application.

## Rust Skill Guidelines

Before writing or reviewing Rust, load the Rust skill if available
(`assets/ai/ai-rust/skills/claude/SKILL.md`), starting with the anti-patterns and
core-idioms guides. If it isn't present, ask before cloning.

## Git Conventions

- Imperative, descriptive commit messages; explain *why*.
- Before submitting: `make test` + `make lint` + `make format` (+ `make coverage`).
- `make push` pushes `main` and tags to `origin` (GitHub).
