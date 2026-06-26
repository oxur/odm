# odm

[![][build-badge]][build]
[![][crate-badge]][crate]
[![][tag-badge]][tag]
[![][docs-badge]][docs]

[![][logo]][logo-large]

<sup><em>Odim, the god of odd</em></sup>

*The Odd Document Manager*

`odm` keeps your **plan and your design docs in one git-native graph**. Projects,
arcs, and slices — together with design docs, decision records, and notes — are all
*nodes* with stable IDs, typed dependency edges, and multi-gate status. `odm` derives
execution order from the dependency graph (`next` / `blocked` / `path`), records how
well each step is actually *known* (an evidence ladder from *asserted* to
*reconciled*), validates the whole graph mechanically (`check`), and reconstitutes
full situational awareness from one cheap command (`odm orient`).

Files are the source of truth; git is the backend. No server, no database, no SaaS.

- **Binary:** `odm` (published by the `oxur-odm` umbrella crate).
- **Library:** the document model, store, and graph engine, split across the
  workspace crates below.

> **Status — v1.0.0, MVP shipped.** The rebuild's MVP (arcs A1–A3) is on `main` and
> CI-green: node CRUD, the typed dependency graph with gates and evidence-leveled
> satisfaction, `check`, the generated rollup, and `orient`. The next arcs —
> incremental index (A4), reconciliation/drift (A5), and the legacy importer +
> self-hosting (A6) — are on the [roadmap](#roadmap) and not yet built. The
> pre-rebuild implementation is preserved under [`legacy/oxur-odm`](legacy/oxur-odm)
> as the harvest source.

## What odm does today

Most project tools are *issue-oriented* — a person dragging a ticket across a board.
`odm` is *slice-oriented* and built around dependency order and evidence. What's
working now (A1–A3):

- **Order is derived, not assigned.** `next` / `blocked` / `path` are computed from
  the dependency DAG by topological sort — sequence is a function of edges, not a
  hand-ranked backlog.
- **Status carries epistemic confidence.** Every gate records an evidence level
  (`asserted < attested < reproduced < reconciled`), and satisfaction
  *min-propagates* along dependency chains — a chain is only as verified as its
  weakest link, so a relayed "it's done" can't silently unblock critical work.
- **Mechanical plan-integrity checking.** `check` reports (and, in `--strict`, fails
  CI on) cycles-without-tears, out-of-order work, broken WBS recomposition, and
  dangling references — the plan itself is validated, not just stored.
- **Explicit tears.** A deliberately-assumed, cycle-breaking dependency is recorded
  with a *required rationale* and stays visible — DSM "tearing" applied to plans.
- **`orient`-first global state.** A fresh session reconstitutes the whole picture —
  vision, current focus, ready/blocked, integrity, drift — from one command. Bare
  `odm` runs it; it never bare-errors.
- **One substrate, self-documenting and self-tracking.** Design docs, decision
  records, and units of work are all nodes with the same id / edge / gate machinery —
  not a wiki bolted onto a tracker.
- **Intent vs. emergence is first-class.** Each node records its `origin` (planned /
  discovered / amendment); the rollup shows original-vs-emergent scope.
- **Files are the source; git is the backend.** The plan lives in version control
  beside the code and diffs in pull requests.
- **LLM-native ergonomics.** `--json` on every query with stable, documented schemas;
  question-named commands; errors that name the fix; idempotent describe-or-create.

## The model in one breath

A **node** is one markdown file — YAML frontmatter (managed metadata) plus a markdown
body (way-finding text). Identity is an immutable **ULID**; the human `number` and
`name` are just metadata. Edges live on the **source** node (`part_of`, `depends_on`,
`blocked_by`, `consumes`, `verifies`, `affects`, `supersedes`, `tears`); reverse edges
are *derived*, never stored. **Containment** (`part_of`) is a tree; **ordering**
(`depends_on ∪ consumes`) is a DAG. Status is a **vector over named gates** defined
per node type in `odm.toml`. The single source of truth is the set of node files;
everything else — the rollup, any cache — is derived and regenerable.

## Quick start

```bash
# Orient — the cheap, whole-picture view (bare `odm` runs this; `brief` is an alias)
odm orient

# Create nodes (idempotent: re-running describes rather than duplicating)
odm new project "Payments"
odm new arc "Card capture" --parent 1
odm new slice "Tokenize PAN" --parent 2
odm new slice "Charge endpoint" --parent 2

# Set the working context so you don't repeat --project/--arc
odm use project "Payments"

# Wire dependencies (edge lives on the source; reverse is derived)
odm link 4 depends_on 3            # Charge endpoint depends_on Tokenize PAN

# Advance status with an evidence level
odm set-gate 3 built --evidence reproduced

# Ask the graph
odm next                           # the ready frontier
odm blocked 4                      # why #4 is held back
odm path 4                         # the dependency chain into #4

# Validate the whole graph (CI gate; --strict promotes warnings to failures)
odm check --strict

# Regenerate the shared way-finding view
odm rollup                         # writes ROLLUP.md (never hand-edited)

# Everything queryable also speaks JSON with a stable schema
odm orient --json
odm check --json
```

Run `odm <command> --help` for flags. Every mutator takes `--dry-run` and `--yes`;
every query takes `--json`.

## Command surface

| Area | Commands |
|------|----------|
| Orient / read | `orient` (alias `brief`), `list`, `show`, `next`, `blocked`, `path`, `rollup` |
| Context | `use`, `context` |
| Create / edit | `new`, `rename`, `retire`, `supersede` |
| Graph | `link`, `unlink`, `set-gate`, `tear`, `decomposed` |
| Integrity | `check` |

## Workspace layout

`odm` is a Cargo workspace (resolver 2, edition 2024, MSRV 1.85, `max_width = 100`).
Crates, in dependency / publish order:

| Crate | Binary | Purpose |
|-------|--------|---------|
| **odm-graph** | — | Pure DAG/tree engine over abstract ids (edges, topo-sort, cycles, tears, readiness). No domain knowledge. |
| **odm-core** | — | Domain model: node types, ULID identity, frontmatter schema, edge & gate semantics, satisfaction, the rollup model. |
| **odm-store** | — | Persistence: `nodes/YYYY/MM/<ULID>.md` layout, atomic writes, git (`gix`), `odm.toml`. |
| **odm-cli** | — | The clap command surface (`--json`, output via oxur-cli/tabled). Library-only, in-process dispatch. |
| **oxur-odm** | `odm` | Umbrella: publishes the `odm` binary and re-exports the library API. |

The pre-rebuild crate lives at `legacy/oxur-odm` (package `oxur-odm-legacy`); it is
excluded from the workspace and kept only for harvesting.

## Installation

From source (the reliable path during the rebuild):

```bash
git clone https://github.com/oxur/odm
cd odm
make build                  # binary at ./bin/odm
# or: cargo build --release # binary at ./target/release/odm
```

From crates.io:

```bash
cargo install oxur-odm      # installs the `odm` binary
```

## Configuration

`odm` reads an `odm.toml` from the current directory, the git repository root, or
`~/.config/odm/`. The core configuration is the **per-node-type gate-sets** — the
ordered status gates each node type moves through (the last gate is its *terminal*
gate):

```toml
[gates.project]
sequence = ["planned", "in-progress", "complete", "verified"]

[gates.arc]
sequence = ["planned", "in-progress", "complete", "verified"]

[gates.slice]
sequence = ["planned", "built", "tested", "deployed", "verified-live", "operator-confirmed"]

[gates.odd]
sequence = ["draft", "under-review", "revised", "accepted", "active", "final"]
```

## Roadmap

The MVP (A1–A3) is shipped. Beyond it, on the design board (`docs/design/`,
`docs/design-v1.0.0/`) and **not yet built**:

- **A4 — Index & cache.** An incremental, stat-based index under `.odm/` (DB-free, no
  FTS) replacing the full-scan; self-healing and derived.
- **A5 — Reconciliation.** Nodes declare `desired_facts`; probes diff *declared* state
  against *observed reality* and report **drift** (Terraform's lesson, lifted to plan
  state). This is also where deferred-node surfacing lands. Today `orient`/`rollup`
  show drift as "not yet tracked (A5)".
- **A6 — Migrate & self-host.** A `migrate` importer brings legacy number-based docs
  into the model; `odm` then manages its *own* plan as nodes.
- **A7+ — Telemetry & forecasting.** Two-clock telemetry and evidence-backed forecasts
  (research in ODD-0018).

## Development

```bash
make build        # build the binary into ./bin/
make test         # run the full test suite (cargo test --all-features --workspace)
make lint         # clippy (-D warnings) + rustfmt --check
make format       # apply rustfmt
make coverage     # coverage summary (target 95%; floor 90%)
make check        # build + lint + test
```

The workspace targets the stable toolchain (edition 2024, MSRV 1.85, `max_width = 100`).
Cargo.lock is committed — `odm` is a binary application.

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
