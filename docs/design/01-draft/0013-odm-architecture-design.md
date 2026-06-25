---
number: 13
title: "odm — Architecture & Design (v-major rebuild)"
author: "topological sort"
component: All
tags: [architecture, design, node-graph, dag, reconciliation]
created: 2026-06-20
updated: 2026-06-25
state: Draft
supersedes: null
superseded-by: null
version: 1.8
---

# odm — Architecture & Design (v-major rebuild)

> SDLC step 3 output. The *how* for the rebuild whose *what/why* is fixed in
> ODD-0012 (Project Definition) and grounded in ODD-0011 (Research). Where this
> doc and 0012 disagree, 0012 wins on scope and 0011 wins on evidence; raise the
> conflict rather than silently reconciling. Authored 2026-06-20 by Claude +
> Duncan.

## 1. Thesis

`odm` is **one graph of typed nodes** persisted as markdown-with-frontmatter in
git, plus two engines that operate on it:

1. a **build system for the plan** — derive order from dependencies, answer
   *what's ready / blocked / next*, and warn on out-of-order work; and
2. a **reconciler for the state** — diff each node's *declared desired facts*
   against *observed reality* and report drift.

There is no separate "doc system" and "plan system": a design document (ODD), a
decision record (ADR), and a unit of work (slice) are all **nodes** that differ
only by `type` and by which **gate-set** governs their status. This is what makes
the tool simultaneously self-documenting and self-tracking — both fall out of one
mechanism.

The single source of truth is the **complete set of node files**. Everything else
— the rollup, any caches, the directory layout — is *derived* and regenerable.
The cardinal rule (from ODD-0011's à-la-Carte result): **a fact that is not an
edge cannot be checked.** Bias toward over-declaring edges.

## 2. The node

Every node is one markdown file: YAML frontmatter (managed, queryable metadata) +
a markdown body (the human content / way-finding text).

### 2.1 Identity

- `id` — a **ULID** (via the `ulid` crate), assigned once at creation, **never
  reused, never renumbered**. This is the only identity. All edges reference ids.
- `number` — a human-friendly integer, **metadata, not identity**. Stable, never
  reused, but carries *no ordering claim* (the lesson of "Phase 8.5"). Used for
  display and as a CLI handle.
- `name` / `title` — human label; freely editable; never affects identity or
  file location.

Commands accept a `number`, a unique `name` prefix, or a full `id` and resolve to
the `id`. The id is what appears in frontmatter and git diffs.

### 2.2 Node types

Two families, one substrate:

- **Work nodes:** `project`, `arc`, `slice`. Scope decomposition only (WBS
  100%-rule: a parent's children sum to exactly the parent — no more, no less).
  Containment is the `part_of` tree, **not** sequence. There is deliberately **no
  `step` node**: a step is a single operation, always too small to deserve a
  node/document; the urge to drill into steps is funnelled into *breadth* (more
  slices/arcs), not depth. (Supersedes the parenthetical leaf-`step` note in
  0025-§4a.)
- **Document nodes:** `odd` (design doc), `adr`/`rfc` (decision record), `note`.
  Long-form content; same id/edge/gate machinery; supersede-don't-delete.

`type` is fixed at creation. New types are config + a gate-set; the engine is
type-agnostic. (Open Q-1: is `type` ever mutable? Current answer: no — model a
type change as supersede-by-a-new-node.)

### 2.3 Frontmatter schema (normative)

```yaml
id: 01J9Z3K7Q2V8M4N0XF7B5C3A1D     # ULID, immutable
number: 7                           # human handle, metadata
type: slice
name: "Store layer"
created: 2026-06-20                  # also encoded in the ULID; this is the human copy
updated: 2026-06-20
tags: [store, persistence]          # optional; free-form filter labels (carried from legacy)
component: odm-store                # optional; subsystem filter (carried from legacy)
origin: planned                     # how this node AROSE: planned | discovered | amendment
reserved: false                     # tentative future-work placeholder (not yet real work)
edges:
  part_of: 01J9...ARC               # single parent (containment tree)
  depends_on:
    - 01J9...AAA                     # bare id → satisfied at target's terminal gate
    - { node: 01J9...BBB, satisfied_at: tested }
  blocked_by: []
  verifies: [01J9...DOC]
  consumes: [01J9...OUT]
  affects: []                        # decision/doc → the docs it touches (0001-C5)
  supersedes: null                   # or { node: 01J9...OLD, kind: updates } — kind ∈ {obsoletes (replace), updates (amend)}; reverse (superseded_by) derived
  tears:                             # explicitly-broken dependency edges (see §4.3); omitted when empty
    - edge: 01J9...TORN              #   the assumed `depends_on` (bare id or { node, satisfied_at })
      because: "B is assumed to ship first"   # required rationale (audited; never dropped)
status:                              # multi-gate vector; absent gate = not reached
  built:  { reached: 2026-06-12, by: "duncan", evidence: reproduced }
  tested: { reached: 2026-06-13, evidence: reconciled }
  # evidence ∈ asserted | attested | reproduced | reconciled  (0001-D3)
decomposed:                          # a parent's guarded "complete" assertion (§4.5); typed, drift-guarded
  on: 2026-06-20                     #   when the child set was affirmed complete
  children: [01J9...AAA, 01J9...BBB] #   the affirmed child ids (a later add/remove is drift)
desired_facts:                       # for the reconciler (§5)
  - id: db-wired
    describe: "prod service has DB_HOST and connects"
    probe: { kind: shell, run: "scripts/check-db.sh", expect_exit: 0 }
```

Frontmatter is **emitted in a canonical field order** (round-trip stable;
`parse ∘ emit = identity` is a proptest invariant) — including the edge sub-keys in
the order shown above (`part_of, depends_on, blocked_by, verifies, consumes,
affects, supersedes, tears`), which §3's table mirrors. Unknown keys are preserved,
not dropped (forward-compat).

A node may also carry an optional **`retired: { reason, on }`** marker, set by
`odm retire` — the node is withdrawn but its file is *kept* (supersede-don't-delete;
git preserves history), absent until retired. (Added in build slice A1.5; Arc 02
may fold retirement into the gate model — see Q-10.)

## 3. Edges

Edges are first-class data, stored on the **source** node's frontmatter; reverse
edges (`superseded_by`, "depended-on-by", "part-of's children") are **derived**,
never stored, so there is exactly one place to edit.

| Edge | Meaning | Forms which graph |
|---|---|---|
| `part_of` | containment; single parent | the **hierarchy tree** |
| `depends_on` | needs target satisfied before this is ready | the **ordering DAG** |
| `blocked_by` | hard external block, not a scope dependency; **withholds from `next`** | ordering gate |
| `verifies` | this node (often a doc/test) verifies target | traceability |
| `consumes` | uses a concrete output/artifact of target | ordering DAG |
| `affects` | a decision/doc affects target docs; powers the stale-doc-vs-decision check (0001-C5) | traceability |
| `supersedes` | lineage; carries `kind: obsoletes` (replace) or `updates` (amend); old node stays | lineage chain |
| `tears` | a `depends_on` we have *deliberately* assumed (cycle break) | annotation |

The **ordering DAG** = `depends_on` ∪ `consumes` (∪ `blocked_by` as a soft gate).
`part_of` is a separate tree (containment ≠ sequence). `supersedes`/`verifies`
are tracked but don't drive ordering.

## 4. The graph engine (`odm-graph`)

Pure algorithms over abstract `(NodeId, EdgeKind)` — **zero domain knowledge**, so
it is independently testable. `odm-core` translates the domain model into it.

### 4.1 Derived-order queries
- `next` — nodes whose every `depends_on`/`consumes` edge is **satisfied** (§4.4),
  which have no active `blocked_by`, and which are not themselves complete: the
  ready frontier. (A `blocked_by` withholds a node from `next`; its reason surfaces
  under `blocked` — so `next` never overstates what is actionable. Q-3.)
- `blocked X` — the unsatisfied edges holding `X`, named.
- `path X [Y]` — dependency chain to `X` (or between `X` and `Y`); critical path.
- topological listing (Kahn).

### 4.2 Cycle detection
Kahn's algorithm yields cycle detection for free. Any cycle is surfaced **loudly**
and must be resolved by an explicit **tear**, never silently tolerated.

### 4.3 Tears
A tear marks one `depends_on` edge as *deliberately assumed* (DSM "tearing"). It
is recorded in `tears:` on the source node as a typed entry `{ edge, because }`
— the assumed edge **and** its required `because` rationale (the rationale is
persisted, not merely validated). `check` fails on a cycle that has no tear;
passes once a tear is declared; and lists all active tears **with their
rationale** so assumed dependencies stay visible.

### 4.4 Satisfaction & the staleness guard
An edge `A depends_on B` is **satisfied** when `B` has reached the gate named by
the edge's `satisfied_at` (default: `B`'s type's **terminal** gate). `A` is
**ready** when all its incoming dependency edges are satisfied.

**Evidence-leveled satisfaction.** Satisfaction carries not only *whether* the
satisfying gate was reached but *how well that is known* — the gate's evidence
level (§5.1 / 0001-D3): `asserted < attested < reproduced < reconciled`. The graph
**min-propagates** evidence along dependency chains: a node's effective confidence
is the *minimum* evidence level across its transitive dependency path, so a chain
is only as verified as its weakest link. A configurable **threshold** (default
`reproduced`) defines trustworthy satisfaction; a dependency satisfied only *below*
threshold is **soft-satisfied**:

- `next` still lists the node but flags it (`⚠ dep X satisfied at evidence=attested`);
- `blocked X` names the low-evidence dependency and how to raise it;
- `check` warns on below-threshold satisfaction (and fails it in strict/CI mode).

This never *blocks* proceeding on low evidence — it refuses to let the low
confidence be *invisible*. It is the direct antidote to 0001 F2/G3 (building
load-bearing work on a *relayed belief* — the prod-DB 503) and the internal
counterpart of the cross-team guard in ODD-0017 §3.3.

The **staleness guard**: advancing a node's gates (or editing it as "work") while
any `depends_on` is unsatisfied emits a loud out-of-order warning (build-staleness
applied to the plan). Non-fatal by default; `check` can make it fatal in CI.

### 4.5 Decomposition & recomposition integrity

A leading cause of the failures behind this rebuild was the inability to *see* how
a parent decomposed into children and to *recompose* the whole from the parts.
With the single-parent `part_of` tree (Q-4) the engine has the data to make this
structural and checkable — so this is **built, not deferred** (Q-7):

- **Recomposition is total and unambiguous:** reverse-`part_of` enumerates a
  parent's complete child set; every non-root node resolves to exactly one parent;
  no orphans, no dangling parents. `show <parent>` renders the full decomposition.
- **No undeveloped stubs:** a parent-capable node (`project`, `arc`) driven into a
  working/complete gate while it has zero children is flagged.
- **Guarded completeness assertion:** a parent may affirm a typed
  `decomposed: Decomposition { on, children }` ("these children fully account for
  my scope — no missing, no extra"). The affirmed child set is recorded (an
  enrichment of the bare `decomposed: complete` scalar, realized in arc02
  slice05) so a later add/remove is detectable as drift. `check` then guards it:
  children added/removed afterward, or a parent advanced toward done without the
  assertion, flags for re-affirmation.

What the tool deliberately does **not** attempt: *automatically* detecting
semantically missing or excess scope ("did you forget a slice?"). That is a human
judgement; faking it would be confabulation. The design makes the decomposition
cheap to review and turns "100% coverage" into an explicit, drift-guarded
assertion — the cheap global review the failures needed.

## 5. Status, gates, and reconciliation

### 5.1 Multi-gate status (fully configurable)
Status is a **vector over named, ordered gates**, not a scalar. Gate-sets are
defined **per node type in `odm.toml`** (decision: configurable from day one):

```toml
[gates.slice]   # the leaf work node carries the integration gates (0011-R5)
sequence = ["planned","built","tested","deployed","verified-live","operator-confirmed"]

[gates.arc]
sequence = ["planned","in-progress","complete","verified"]

[gates.odd]
sequence = ["draft","under-review","revised","accepted","active","final"]
```

A node records which gates it has reached (with date + actor); gates are ordered
so "terminal gate" and "advance/regress" are well-defined. The old single
`DocState` becomes simply the `odd` gate-set — *one* configuration, not a
privileged concept. Binary done/open is gone: "done at its layer" vs "verified
live" are now distinct gates, which is precisely the distinction that hid the
prod-DB failure.

### 5.2 Desired-state facts + probes (`odm-reconcile`)
A node may declare `desired_facts`. A `Probe` is a trait; the first impl is
`shell` (run a command, compare exit/stdout) — the freeze-harness pattern lifted
to the program level. The legacy checksum/mtime detector becomes a `file` probe.

`reconcile` diffs declared desired facts against probe results and reports
**drift**, on demand and on a schedule. It is honest **only about tracked facts**
(Terraform's lesson: you can't detect drift on what the SoT doesn't claim to
manage) — so the tool nudges enumerating integration-level facts. Drift is folded
into the rollup.

## 6. Storage & layout (`odm-store`)

- **Files are the source; `odm` is the build.** Markdown + frontmatter, git-native.
- **Path = pure function of the id:** `nodes/YYYY/MM/<ULID>.md`, where `YYYY/MM`
  is the node's **creation** month read from the ULID timestamp. Files therefore
  **never move** on retitle, reparent, or gate change → minimal git churn, O(1)
  locate from id alone (no lookup index needed). Side benefit: `find ./nodes`
  gives an at-a-glance creation/activity history across machines.
- Filenames **are** the id; humans never navigate the tree by hand (`odm list` is
  our `ls`). Hierarchy and state live in frontmatter, never in the path — we do
  **not** repeat the legacy mistake of encoding truth in directory structure.
- **Atomic writes** (write-temp-rename, carried over from the legacy state code).
- **Config:** per-repo `odm.toml` via layered search (confyg), carried over.
- **Generated rollup:** a regenerable `ROLLUP.md` (+ `--json`) — way-finding tree,
  per-node status vectors, ready/blocked sets, active tears, drift, and **deferred
  nodes with their checkable re-entry predicate** (0001-E5). Never hand-edited;
  `odm orient` reads/produces it. Any on-disk cache is derived and rebuildable from
  the node files alone.

### 6.1 Index & cache (`odm-index`) — the read-acceleration mini-infra

Distinct from storage (which reads/writes individual node files) **and** from the
rollup (a human-facing *view*): `odm-index` is the machine-facing **mini-infra**
that makes "which files define projects / arcs / slices?" and metadata filtering
fast at scale — built on the OS filesystem and nothing else (no database, no FTS
library, no daemon). This is the "as little infra as possible — i.e. none" line.

- **First run** blocks on a full scan: walk `nodes/`, parse frontmatter, build the
  index, persist it under `.odm/` (gitignored — it is derived, never truth).
- **Subsequent runs** are incremental: load the index, then a stat-walk detects
  only *changed / new / deleted* files (mtime+size fingerprint, content-hash as
  tiebreak); re-parse only those; update and persist. Cost scales with the
  *delta*, not the corpus.
- **Self-healing:** a missing or corrupt index is rebuilt from the node files — it
  carries no authority.
- **Consumers** (`list`, search/filter, `orient`, the graph build) read the index;
  they never re-walk the tree themselves.

Deliberately **research-gated** (Q-9): change-detection correctness (git's
stat-cache and the "racy-timestamp" problem), the DB-free persistent format, and
dependency-aware invalidation of derived views each deserve a proper research pass
before code. Its own crate keeps this infra isolated and swappable.

## 7. Command surface (`odm-cli`)

Commands named after the **question**, not the mechanism. `--json` on every query
(stable, documented schemas). `--dry-run` and `--yes` on every mutator. Errors
name the exact fix (errors-as-affordances). Bare `odm` **orients** (never bare-errors).

- **Orient/read:** `orient`/`brief`, `list` (filters: type, gate, ready, blocked,
  drift), `show X` (node + edges + status vector + way-finding, one call),
  `next`, `blocked X`, `path X [Y]`.
- **Context:** `use [project|arc] X` sets the current project/arc (so
  `--project`/`--arc` need not be repeated on every call); `context` shows the
  current selection. (closes 0025-§4a gap)
- **Mutate:** `new <type> <name>` (**idempotent** describe-or-create — re-running
  describes rather than duplicating), `rename`, `set-gate X <gate>`,
  `link X <edge> Y` / `unlink`, `tear X depends_on Y --because …`,
  `supersede X --with Y`, `retire X --because …` (mark withdrawn/removed; git
  preserves history — never a destructive delete of a tracked file).
- **Integrity:** `check` (schema, link-integrity, cycles-without-tears,
  out-of-order, decomposition/recomposition integrity (§4.5), stale docs vs
  committed decisions (0001-C5, via `affects`), unreconciled drift; CI/pre-commit
  gate, exit codes), `reconcile [--schedule]`, `rollup` (regenerate).
- **Migration:** `migrate <legacy-path>` (§9).

`check` is the lynchpin: it is the framework's mechanical disciplines made
executable, so the prose rules they replace can be retired.

## 8. Crate architecture

Dependency / publish order, top to bottom (mirrors the oxur umbrella pattern):

| Crate | Responsibility | Key deps |
|---|---|---|
| `odm-graph` | Pure DAG/tree engine over abstract ids: edges, topo-sort, Kahn cycles, tears, ready/blocked/path, staleness. | petgraph |
| `odm-core` | Domain model: node types, ULID identity, frontmatter schema (serde), edge & gate semantics, satisfaction, link-integrity, rollup model. | odm-graph, ulid, serde |
| `odm-store` | Persistence: layout, atomic writes, git, `odm.toml`, scan/load. | odm-core, gix, confyg |
| `odm-index` | Incremental index + cache mini-infra: stat-based change detection, DB-free persisted index, fast type/metadata lookup & filter acceleration. No FTS deps. (Research-gated — Q-9 / ODD-0014.) | odm-store, odm-core |
| `odm-reconcile` | Probe trait + shell/file probes, drift diff, schedule. | odm-core, odm-index |
| `odm-migrate` | Legacy importer → new model. | odm-store, odm-core |
| `odm-cli` | clap surface, `--json`, errors-as-affordances, output (oxur-cli/tabled). | all above |
| `oxur-odm` | Umbrella: publishes the `odm` binary; re-exports the library API. | odm-cli |

Errors carry source position where parsing frontmatter (`thiserror` in libs,
`anyhow` in the binary). 95%+ coverage target; `proptest` for invariants
(round-trip, id-uniqueness, topo-validity).

## 9. Migration (`odm-migrate`)

Map the legacy model onto the new one:

| Legacy | New |
|---|---|
| `number` (identity, reusable) | fresh **ULID** id; legacy number preserved as `number` metadata |
| `DocState` scalar | `odd` gate-set position |
| state directory (`05-active/…`) | dropped (was redundant truth); state → gate |
| `supersedes`/`superseded_by` | `supersedes` edge (reverse derived) |
| dustbin / Removed / Overwritten | supersede-don't-delete + git history |
| flat doc | `odd`/`adr` document node |

The importer is **idempotent** and `--dry-run`-able; it never deletes legacy
files (git preserves history). Once it can import odm's own `docs/`, `odm`
self-hosts and these design docs move under `nodes/` (the loop closes).

## 10. Decisions & open questions

**Decided** (rationale above): unified node graph · **no `step` node** (funnel to
breadth, not depth) · ULID via `ulid` crate · `nodes/YYYY/MM/<ULID>.md`
creation-time sharding · filenames are ids · configurable per-type gate-sets ·
edges on source + derived reverse · ordering DAG = depends_on ∪ consumes ·
explicit tears for cycles · `supersedes` carries `obsoletes`/`updates` kind
(0011-R6) · `tags`/`component` retained · `provenance` split into `origin` +
`reserved` · **git via a pure-Rust library, not shelling out** (Q-2) ·
**`odm-index` is its own crate** — incremental index/cache mini-infra (Q-9) ·
command surface adds current-context (`use`/`context`), `retire`, and idempotent
describe-or-create (closes 0025-§4a/§4c gaps) · git = `gix` (Q-2) · `blocked_by`
withholds from `next` (Q-3) · decomposition/recomposition integrity built (§4.5,
Q-7) · the framework's mechanical PM layer is extracted into a standalone skill
that defers to `odm` (§11) · evidence-level on gate transitions, an `affects` edge
+ stale-doc-vs-decision check, and a deferred re-entry predicate (from post-mortem
0001 — D3/C5/E5) · evidence-leveled satisfaction with a threshold + min-propagation
(§4.4) — the internal counterpart of ODD-0017 §3.3.

> **Terminology:** *provenance* is reserved for the **derived lineage** — git
> history + the `supersedes` chain + gate-reached timestamps — never a stored
> scalar. A node's current frontmatter records its `origin`, not its provenance.

**Open / resolved:**
- **Q-1** `type` immutability — **agreed: immutable** (model changes via supersede).
- **Q-2** Git via a Rust library, no shelling out — **decided: `gix`** (pure-Rust
  gitoxide; no C/libgit2 dependency — fits the minimal-infra goal).
- **Q-3** **Decided:** a `blocked_by` edge **hides** a node from `next` (so `next`
  never overstates actionability); the reason surfaces under `blocked`.
- **Q-4** Multi-parent containment — **agreed: not needed**; `part_of` is a tree.
- **Q-5** Rollup granularity/perf at scale — **TBD by observed need + data.**
- **Q-6** Month-shard key — **agreed: creation-time**; update-time is metadata
  (`updated:`).
- **Q-7** WBS 100%-rule as a `check` — **reframed & adopted (not deferred):** the
  part that bit us — *seeing* the parent→children decomposition and *recomposing*
  the whole — is structural, and we have the data, so it's built (§4.5: total
  recomposition, no-orphan/no-stub checks, drift-guarded `decomposed: complete`
  assertion). Only *automatic semantic* missing/excess-scope detection stays out (a
  human judgement; faking it = confabulation).
- **Q-8** Explicit entry-gate (Definition-of-Ready) distinct from
  dependency-readiness — **agreed: deferred.**
- **Q-9** Index/cache mini-infra (`odm-index`) — **research-first.** Scope:
  stat-based incremental change detection (git stat-cache / "racy-git" lessons), a
  DB-free persistent index format, dependency-aware invalidation of derived views,
  and filter/sort acceleration with **no FTS dependency**. Forthcoming research ODD
  (proposed **0014**) before implementation.
- **Q-10** Retirement representation — currently an optional top-level `retired: {
  reason, on }` field (build slice A1.5, since gates don't exist yet). Arc 02 may
  fold it into the gate model (a `withdrawn`/`retired` gate). Decide when gates land.

## 11. Scope beyond the engine, and next SDLC step

Two workstreams ride alongside the engine:

- **`odm-index` research → ODD-0014** (Q-9), in parallel — the index/cache
  mini-infra's correctness (git stat-cache / "racy-timestamp", DB-free persistence,
  dependency-aware invalidation) deserves a cited research pass before code.
- **Project-management skill overhaul** (in `billosys/ai-engineering`): the
  collaboration-framework's *mechanical* PM convention is extracted into its own
  skill that the framework references and defers to. On odm's release, most of that
  prose becomes "when you need to X, run `odm <cmd>`" entries paired with BAD /
  DON'T-DO counter-examples, seeded by a compiled list of the prior project's
  missteps (the failures that triggered this effort). This is the concrete form of
  bootstrap §6 ("retire mechanical prose → `odm check`"); it depends on a stable
  command surface (A1–A3) and lands as/after the migration+retirement arc.

**Next SDLC step:** the arc/slice breakdown (its own ODD). MVP = Arcs A1–A3
(substrate + DAG/gates + rollup/orient). Build each slice with a ledger; self-host
once A1–A3 land.
