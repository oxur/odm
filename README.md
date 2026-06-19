# odm — ODD Document Manager

`odm` is a command-line tool for managing design documents (ODDs — "Oxur Design
Documents", or any design-doc workflow) with YAML frontmatter, a state
lifecycle, git integration, and automatic indexing.

It was originally developed inside the [Oxur](https://github.com/oxur/oxur)
language project and now lives on its own so any project can adopt the same
design-doc workflow independently.

- **Binary:** `odm`
- **Library:** `odm` (crate `oxur-odm` on crates.io) — the document model,
  index, state machine, and config are usable programmatically via `use odm::…`.

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
package README: [`crates/oxur-odm/README.md`](crates/oxur-odm/README.md).

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

The workspace targets the stable toolchain (edition 2021, `max_width = 100`).

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
