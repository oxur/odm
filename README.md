# odm

[![][build-badge]][build]
[![][crate-badge]][crate]
[![][tag-badge]][tag]
[![][docs-badge]][docs]

[![][logo]][logo-large]

<sup><em>Odim, the god of odd</em></sup>

*The Odd Document Manager*

`odm` is a command-line tool for managing design documents (ODDs — "Oxur Design
Documents", or any design-doc workflow) with YAML frontmatter, a state
lifecycle, git integration, and automatic indexing.

It was originally developed inside the [Oxur](https://github.com/oxur/oxur)
language project and now lives on its own so any project can adopt the same
design-doc workflow independently.

- **Binary:** `odm` (published by the `oxur-odm` umbrella crate).
- **Library:** the document model, store, and graph engine are split across the
  workspace crates below.

> **Status — v1.0.0 rebuild in progress.** `odm` is being rebuilt from a single
> crate into the multi-crate workspace described below. The workspace skeleton is
> in place (the `odm` binary currently reports `--version` only); the command
> examples further down describe the target / legacy behavior and are being
> reimplemented slice by slice (see `docs/design-v1.0.0/`). The pre-rebuild
> implementation is preserved under [`legacy/oxur-odm`](legacy/oxur-odm) as the
> harvest source.

## Workspace layout

`odm` is a Cargo workspace (resolver 2, edition 2024, `max_width = 100`). Crates,
in dependency / publish order:

| Crate | Binary | Purpose |
|-------|--------|---------|
| **odm-graph** | — | Pure DAG/tree engine over abstract ids (edges, topo-sort, cycles, readiness). |
| **odm-core** | — | Domain model: node types, ULID identity, frontmatter schema, edge & gate semantics. |
| **odm-store** | — | Persistence: `nodes/YYYY/MM/<ULID>.md` layout, atomic writes, git (`gix`), `odm.toml`. |
| **odm-cli** | — | The clap command surface (`--json`, output via oxur-cli/tabled). |
| **oxur-odm** | `odm` | Umbrella: publishes the `odm` binary and re-exports the library API. |

The pre-rebuild crate lives at `legacy/oxur-odm` (package `oxur-odm-legacy`); it
is excluded from the workspace and kept only for harvesting.

## Installation

From crates.io:

```bash
cargo install oxur-odm      # installs the `odm` binary
```

From source:

```bash
git clone https://github.com/oxur/odm
cd odm
make build                  # binary at ./bin/odm
# or: cargo build --release # binary at ./target/release/odm
```

## Quick Start

```bash
# List all documents
odm list

# Create a new document
odm new "My Feature Design"

# Add an existing document (numbering, headers, git staging)
odm add path/to/document.md

# Transition a document to a new state
odm transition docs/01-draft/0001-my-feature.md "under review"

# Validate all documents
odm validate

# Update the index
odm update-index
```

A full command reference (all subcommands, flags, and aliases), the document
state lifecycle, frontmatter format, and workflow examples live in the
legacy package README: [`legacy/oxur-odm/README.md`](legacy/oxur-odm/README.md)
(being migrated into the new crates).

## Configuration

`odm` reads an `odm.toml` from the current directory, the git repository root,
or `~/.config/odm/`. Example (this repo dogfoods `odm` on its own docs):

```toml
docs_directory = "./docs"
dev_directory = "./docs/dev"
preserve_dustbin_structure = true
auto_stage_git = true
```

You can also override the docs directory per-invocation with `odm --docs-dir <path> …`.

## Library usage

```rust
use odm::config::Config;
use odm::index::DocumentIndex;
use odm::state::StateManager;
```

## Development

```bash
make build        # build the binary into ./bin/
make test         # run the full test suite
make lint         # clippy (-D warnings) + rustfmt --check
make format       # apply rustfmt
make coverage     # coverage summary (cargo llvm-cov)
make check        # build + lint + test
```

The workspace targets the stable toolchain (edition 2024, MSRV 1.85,
`max_width = 100`).

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

[//]: ---Named-Links---

[logo]: assets/images/odim-x250.png
[logo-large]: assets/images/odim-x1254.png
[build]: https://github.com/oxur/odm/actions/workflows/ci.yml
[build-badge]: https://github.com/oxur/odm/actions/workflows/ci.yml/badge.svg
[crate]: https://crates.io/crates/oxur-odm
[crate-badge]: https://img.shields.io/crates/v/oxur-odm.svg
[docs]: https://docs.rs/oxur-odm/
[docs-badge]: https://img.shields.io/badge/rust-documentation-blue.svg
[tag-badge]: https://img.shields.io/github/tag/oxur/odm.svg
[tag]: https://github.com/oxur/odm/tags
