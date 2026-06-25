# CC Prompt — Slice 02 (Arc 03): Rollup generation

Build the generated `ROLLUP.md` — the single cheap view of the whole plan. A
`rollup` **model** in `odm-core` (pure assembly over the corpus) plus an `odm rollup`
command in `odm-cli` that renders it to Markdown. This model is the single source
slice03 (`orient`) and slice04 (`--json`) will both consume — build it once, here.

> **Start condition:** slice01 (Arc 02 cleanup) CDC-verified / CI-green — the tear
> rationale (`TornEdge`) must be persisted so it can be rendered. If slice01 isn't in,
> hold.

## Read first
1. `slice02-rollup-generation/ledger.md` (11 rows).
2. `slice-doc.md` (same dir) and the **arc-plan** (`../arc-plan.md`) — decisions
   D-2/D-3/D-4 and resolutions Q-A3-1 (deferred OUT) / Q-A3-2 (drift placeholder).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §6 (rollup), §4.1
   (next/blocked/path), §4.5 (recomposition).
4. The ops you will reuse: `crates/odm-core/src/recompose.rs` (`Recomposition`),
   `graph.rs` (`next`/`blocked`/`active_tears`), `gates.rs` (`GateSet::sequence`),
   `status.rs` (`Status::reached`), `frontmatter.rs` (`origin()`); and how the
   existing `check`/`next` commands load the corpus + build the graph in
   `crates/odm-cli/src/commands.rs`.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `02-api-design.md`.
- `/collaboration-framework` → LEDGER_DISCIPLINE.

## Task

1. **`odm-core` rollup model** (`rollup.rs`): a **pure function** of a loaded corpus +
   `GateSets` (no I/O, no cache — D-2) assembling: the way-finding **tree**
   (`Recomposition` forest), per-node **status vector ordered by `GateSet::sequence()`**
   (absent gates = not-reached — D-4), **ready**/**blocked** (reuse
   `NodeGraph::next`/`blocked`, naming unsatisfied edges), **active tears** with their
   `because` rationale, and a **provenance/origin** grouping. Leave a **drift** slot
   (renders "not yet tracked (A5)") and a **deferred** slot defined but **empty** —
   do **not** invent a `deferred` status (Q-A3-1). Reuse the existing ops; don't
   reimplement graph/recompose logic in the model.
2. **`odm rollup` command** (`odm-cli`): full-scan load → render the model to Markdown
   → write `ROLLUP.md` at the repo root via odm-store (atomic write-temp-rename), with
   a "generated — do not edit (`odm rollup`)" header. Idempotent (same corpus → same
   bytes). `--dry-run` writes nothing. Section order: tree (status inline) →
   ready/blocked → active tears → provenance → drift.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Reuse, don't reimplement:** the rollup model assembles existing odm-core ops; the
  model is the single source for slice03 (`orient`) and slice04 (`--json`) — keep
  Markdown rendering in `odm-cli`, the assembled model in `odm-core` (D-3).
- **`--json` is OUT of this slice** (slice04 pins the schema). Render Markdown only.
- Full-scan regenerate; **no cache** (the `.odm/` index is A4).
- Errors-as-affordances; no `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test -p odm-core -p odm-cli` + clippy + coverage; `ledger.md` evidence
per row; `closing-report.md` (per-row walk for all 11, What Worked, uncertainties
named). Feature branch (`arc03-slice02-rollup-generation`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration
cap; your `done` is *proposed done* → CDC verifies (cargo rows via CI / local 1.85+).
