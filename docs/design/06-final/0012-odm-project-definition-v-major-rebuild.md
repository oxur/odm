---
number: 12
title: "odm — Project Definition (v-major rebuild)"
author: "topological sort"
component: All
tags: [change-me]
created: 2026-06-20
updated: 2026-06-20
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# odm — Project Definition (v-major rebuild)

> SDLC step 1 output. The agreed spec for the drawing-board rebuild of `odm`.
> Diff delivery against this; spec-softening is the failure mode to catch.
> Authored 2026-06-19 by Claude + Duncan. Seed context: `odm-session-bootstrap.md`,
> `planning-system-research.md`.

## Mission

Make `odm` the markdown/git-native, dependency-ordered substrate that
**mechanically actualizes** the collaboration-framework: stable-identity nodes +
an explicit dependency DAG + order *derived* by topological sort + per-edge
staleness/reconciliation + a single *complete* graph as the source of truth. The
framework's mechanical disciplines (numbering, ordering, deferral-tracking,
drift-watching) stop being prose rules a human/LLM must remember and become
tool-encoded checks. Success test: a fresh session reaches full situational
awareness from `odm orient` alone.

## Confirmed decisions (2026-06-19)

1. **One unified node graph.** Work nodes (project → arc → slice → step) and
   document nodes (ODD/ADR/RFC) share ONE substrate — same stable-ID, typed-edge,
   and multi-gate-status machinery — differing only by `type` and gate-set.
   "Self-documenting AND self-tracking" then falls out of one mechanism.
2. **Multi-crate workspace** with `oxur-odm` as the umbrella publishing the `odm`
   binary (see proposed decomposition below).
3. **Migrate importer.** Legacy number-/directory-based ODDs (incl. oxur's
   `crates/design/docs`) are imported into the new model via a `migrate` command.

## In scope

- Hierarchy CRUD (project/arc/slice/step) + document-node CRUD, one node model.
- **Stable opaque IDs**, never reused/renumbered; human number/name is metadata.
- **Typed edges** as first-class data: `depends_on`, `verifies`, `supersedes`,
  `consumes`, `part_of`, `blocked_by` → petgraph DAG.
- **DAG validation + cycle detection** (Kahn); cycles require an explicit *tear*.
- **Derived-order queries:** `next` (ready), `blocked` (+ by what), `path`, topo list.
- **Out-of-order / staleness guard.**
- **Multi-gate status vectors** per node (e.g. planned/built/tested/deployed/
  verified-live/operator-confirmed), gate-set configurable per node type.
- **State reconciliation** (desired-fact vs. actual via pluggable probes).
- **Generated rollup** (orient/brief + ready/blocked/drift), never hand-maintained.
- **Decision records as a node type**; supersede-don't-delete; link-integrity.
- **LLM-ergonomics:** `orient`, `--json` everywhere, question-named commands,
  errors-name-the-fix, `check` as pre-commit/CI gate, idempotent describe-or-create,
  `--dry-run`/`--yes`, bare `odm` orients.
- **Migrate importer** from the legacy on-disk format.

## Non-goals

- No ticketing system / server / database — files are the source, `odm` is the build.
- Not a scheduler/optimizer: dependency order gives a *correct* order, not the
  *fastest*. Priority (WSJF/CCPM/cost-of-delay) is an optional advisory layer
  on top, never the ordering, and not in the MVP.
- No attempt at *complete* traceability — trace dependency + verification edges
  rigorously, not everything (RE literature: full traceability is infeasible).
- No preservation of the legacy model's on-disk truth-encoding (number-as-identity,
  state-in-directory, dustbin). Migrated *into* the new model, not carried forward.

## Proposed crate decomposition (confirm / adjust)

Dependency / publish order top-to-bottom:

| Crate | Responsibility |
|---|---|
| `odm-graph` | Pure DAG engine over abstract node-ids: typed edges, topo-sort, Kahn cycle detection + tears, ready/blocked/path/staleness queries. petgraph lives here. Minimal domain knowledge. |
| `odm-core` | Domain model: node types, stable-ID allocation, front-matter schema (serde), edge semantics, multi-gate status vectors, desired-state-fact format, link-integrity. Depends on `odm-graph`. The heart. |
| `odm-store` | Persistence: markdown/frontmatter parse+emit, file/dir layout, git integration, atomic writes, `odm.toml` config. "Files are the source." Reuses legacy git/config/filename/normalize/extract. |
| `odm-reconcile` | Desired-vs-actual: probe trait, shell/freeze-harness probes, drift reporting, scheduled diff. Reuses the legacy checksum/mtime detector as one probe. |
| `odm-migrate` | Legacy importer (number/dir ODDs → new model). Depends on `odm-store` + `odm-core`. |
| `odm-cli` | clap surface: question-named commands, `--json`, errors-as-affordances, output (oxur-cli/tabled). Depends on all above. |
| `oxur-odm` | Umbrella crate; publishes the `odm` binary; re-exports the library API. |

Rollup/orient generation starts inside `odm-core` (+ rendering in `odm-cli`); it
can split into `odm-report` later if it earns its own crate.

## Legacy code disposition

Summarized in memory (`odm-existing-crate-survives-supersedes-deletes`):
survives = config/git/markdown-utils/atomic-write/CLI scaffolding; superseded =
DocState scalar, flat DocMetadata, flat index, supersedes-pair, mechanism-commands;
deleted = number-as-identity + reuse, directory-as-truth, dustbin machinery.

## Next SDLC steps

3. Design doc — front-matter schema (node/edge/gate-set/desired-fact), file/dir
   layout, command surface, check/reconcile model.
4. Arc/slice breakdown — MVP arc: hierarchy + stable IDs + DAG + next/blocked/check
   + orient/rollup. Reconciler + gate-probes + migrate as following arcs.
5. Build with a ledger; self-host in `odm` once the MVP can.
