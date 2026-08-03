# CLAUDE.md

> **Resuming work? Read
> [`docs/design-v1.0.0/CDC-SESSION-BOOTSTRAP.md`](docs/design-v1.0.0/CDC-SESSION-BOOTSTRAP.md)
> first** — §0 is the dated resume (last: 2026-07-26, UAT phase). Then run
> `odm orient` (build: `cargo build --release -p oxur-odm` → `target/release/odm`;
> verified 2m54s from a clean container). The command-surface authority is
> `docs/design-v1.0.0/odm-command-inventory.md`. **Do not mint new nodes before
> the G-1 (ID scheme) decision** — see the workflow-gap review in
> `arc-release-hardening/`.
>
> **The corpus is no longer in this working tree.** Since the RH C-5 cutover
> (2026-07-26) odm's own nodes live on the orphan **`odm-store` branch** (renamed
> from `odm` on 2026-08-03), checked out at **`.worktrees/odm/`** — `odm.toml`
> here is only a locator pointing at it, and the operational config (gate-sets,
> display) is `.worktrees/odm/config.toml`. Run `odm` commands from the repo
> root as before; resolution follows the locator. To read a node file directly,
> look under `.worktrees/odm/nodes/`, not `nodes/`. **Commit corpus changes on
> the `odm-store` branch** (`git -C .worktrees/odm …`) — they are a separate
> history from the code, and `/.worktrees/` is gitignored here on purpose.


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
| **odm-index** | — | Persisted, derived stat-cache: the index snapshot record + format (Arc 04). |
| **odm-reconcile** | — | Desired-state facts + probes: the `Probe` trait, three-way `ProbeOutcome`, the shell probe (Arc 05). |
| **odm-cli** | — | clap command surface, `--json`, output (oxur-cli / tabled). |
| **oxur-odm** | `odm` | Umbrella: publishes the `odm` binary; re-exports the library API. |

> **v1.0.0 rebuild in progress.** These crates are currently stubs (the `odm`
> binary reports `--version` only); real behavior lands slice by slice under
> `docs/design-v1.0.0/`. The pre-rebuild crate is preserved at
> `legacy/oxur-odm` (package `oxur-odm-legacy`), excluded from the workspace and
> kept as the harvest source — do not delete it. `odm-index` (Arc 04) and
> `odm-reconcile` (Arc 05) now exist, created by the arcs that needed them;
> `odm-migrate` remains deferred to its arc (not yet created).

Shared dependency versions live in `[workspace.dependencies]`; crate manifests
reference them with `<dep>.workspace = true` (no version literals). Lints are
centralized in `[workspace.lints]` and inherited via `[lints] workspace = true`.

The only external Oxur dependency is **`oxur-term`** (git, tag `0.2.1`) — the
themed-table + terminal-helper crate extracted from `oxur-cli` for exactly this
purpose (RH C-1; ADR `arc-release-hardening/adr-c1-oxur-table-re-extraction.md`).
It brings only `tabled`/`colored`/`serde`/`toml`/`anyhow`.

**Do not add `oxur-cli`.** It is superseded here, and it does not compile with
`--no-default-features` (its crate-level `config` is ungated → `config::paths`
needs the binary-only `dirs`); its default `binary` feature pulls the entire
Oxur language/compiler stack.

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

## CDC binary access (cross-compiled Linux binary)

CDC (Cowork cloud-container sessions) generally cannot build `odm` from source
— no reliable Rust toolchain in the container. The operator's local build
cross-compiles a **fully static Linux binary** so CDC can run `odm` directly,
no build required:

- **Built automatically on every fresh build.** `make build` / `make
  build-release` detect a Darwin host and run `make build-linux` for you at
  the end — no separate step to remember. If the cross toolchain isn't
  installed yet, this **soft-skips with a warning** rather than failing the
  primary macOS build. Run `make build-linux` directly for a standalone
  cross-compile, or to see the setup requirements: `cargo-zigbuild`, the
  `zig` toolchain, and the `x86_64-unknown-linux-musl` rustup target
  (`cargo install cargo-zigbuild` + `brew install zig` + `rustup target add
  x86_64-unknown-linux-musl`). Runs `cargo zigbuild --release --target
  x86_64-unknown-linux-musl -p oxur-odm`; does **not** touch or delete the
  native macOS binary at `bin/odm`.
- **Where it lands:** `bin/odm-linux-x86_64-musl` — a build artifact
  (`bin/` is gitignored, not committed; rebuild it after pulling new commits,
  which happens for free the next time `make build`/`make build-release` runs
  on this machine). `file bin/odm-linux-x86_64-musl` should report "ELF
  64-bit LSB executable, x86-64, ..., statically linked".
- **Staging into a Cowork session:** transfer the binary from this worktree
  into the cloud container via the device bridge, then `chmod +x` and run
  directly (`--help`, `check`, `orient`, etc.) — no Rust toolchain needed
  there. **CDC has already reproduced this end-to-end** in the actual Cowork
  cloud container (slice05 ledger row F-2,
  `docs/design-v1.0.0/arc-store-lifecycle/slice05-cdc-binary-access/`):
  the full command surface (`--help`, `--version`, `check`, `orient`,
  `validate`, `node show`, `store --help`, `--json` variants) ran clean.
- **Architecture caveat:** the binary targets **x86_64**. If a given Cowork
  container (or the `device_bash` bridge VM) runs **aarch64 Linux** instead,
  this binary will fail with an exec-format error — check `uname -m` in that
  environment before assuming it will run; a `aarch64-unknown-linux-musl`
  build would be a one-line addition to `make build-linux` if needed.
- **Building from source remains the fallback**, not the primary path, for
  any session that *can* run cargo/rustc (e.g. this local worktree, or a CDC
  environment where the Rust toolchain does happen to be available).

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


## Related projects (peers)

**bitubardos** is a *peer repository project*, not an odm arc or feature branch.
It is governed by the full collaboration-framework workflow (its own project ->
arcs -> slices -> ledgers) and lives on its own branch/worktree
(`.worktrees/bitubardos`). Merge policy (operator, 2026-08-03): it is peer to the
1.0.x and (coming) 1.1.x release work; **nothing from bitubardos lands in `main`
until after the 1.0.0 release is cut**, after which its features
forward-merge/rebase into the 1.1.x line (`main` sees none of it until the first
1.1.0 release). Do not treat bitubardos as a sub-part of an odm release. See
`project-plan.md` Section 6 and project memory `bitubardos`.
