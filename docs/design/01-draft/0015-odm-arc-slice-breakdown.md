---
number: 15
title: "odm — Arc/Slice Breakdown (build plan)"
author: "topological sort"
component: All
tags: [planning, arcs, slices, breakdown, sdlc, build-plan]
created: 2026-06-20
updated: 2026-06-20
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# odm — Arc/Slice Breakdown (build plan)

> SDLC step 4. Dependency-ordered arcs for the rebuild, Arc 1 broken into slices,
> and a coverage matrix proving the plan closes the failures in the post-mortem
> (dev/skill ODD-0001). Builds on 0012 (project def), 0013 (architecture), 0014
> (index research). Once A1–A3 land, this plan migrates *into* `odm` and self-hosts.

## 1. The arcs (dependency DAG)

Order is *derived* from dependencies, not assigned (dogfooding the model). Each arc
is independently demoable.

| Arc | Deliverable | `depends_on` | Crates |
|---|---|---|---|
| **A1 — Substrate & node CRUD** | Workspace + crates; ULID identity; node types (project/arc/slice + odd/adr/note); frontmatter schema; `nodes/YYYY/MM/<ULID>.md` store; `gix`; `odm.toml`; CRUD (`new`/`list`/`show`/`rename`/`retire`/`supersede`); `use`/`context`; `check` v1 (schema + link-integrity). Full-scan list (no index yet). | — | odm-core, odm-store, odm-cli, oxur-odm, odm-graph (stub) |
| **A2 — Graph, gates & derived order** | Typed edges; petgraph DAG; cycles + explicit tears; multi-gate status vectors **with evidence-level**; **evidence-leveled satisfaction** (threshold + min-propagation, ODD-0013 §4.4); `next`/`blocked`/`path`/topo; staleness guard; decomposition/recomposition integrity (+ `decomposed: complete`); `check` v2. | A1 | odm-graph, odm-core |
| **A3 — Rollup & orient** | Generated `ROLLUP.md` (+`--json`); `orient`/`brief` leading with **vision → current focus → ready/blocked → drift**; deferred surfaced with re-entry predicate; original-vs-emergent (provenance) view; errors-as-affordances; bare-`odm` orients. **← MVP COMPLETE** | A2 | odm-core, odm-cli |
| **A4 — Index & cache** | Incremental stat-cache per 0014 (DB-free, no FTS); replaces full-scan in `list`/`orient`/graph-build; self-healing; **100k-node benchmark** promoting 0014's `[P]` perf claims to `[E]`. | A3 | odm-index |
| **A5 — Reconciliation** | `desired_facts` (incl. integration-level + program-level acceptance); probe trait + shell/freeze-harness probes; `reconcile` on demand + scheduled; drift in rollup; stale-doc-vs-decision check (`affects` edge). | A2, A3 | odm-reconcile |
| **A6 — Migrate, self-host & PM-skill** | `migrate` importer (idempotent, `--dry-run`, supersede-not-delete); run on odm's own docs; self-host the plan in odm; populate the PM skill from ODD-0001; retire redundant framework prose → "run `odm check`". | A1, A3 | odm-migrate; `billosys/ai-engineering` |

**MVP = A1–A3.** It already makes the identity/numbering chaos, the parked
dependency, the binary-status blindness, and the vision-loss *structurally
impossible*. A5 closes the marquee state-drift class (the prod-DB 503).

## 2. MVP "definition of done" (program-level acceptance — dogfooding E4)

A fresh session reaches full situational awareness from `odm orient` alone; every
dependency is a queryable edge; `odm next`/`blocked` answer correctly; `check`
catches cycles, dangling refs, out-of-order work, and broken recomposition; status
is multi-gate with evidence; and odm can describe its *own* plan. (These become
tracked acceptance facts once A5 exists.)

## 3. Arc 1 → slices (dependency-ordered)

1. **A1.1 — Workspace scaffolding.** Cargo workspace (resolver v2); the crate
   skeletons; `[workspace.package]`/`[workspace.dependencies]`/`[lints]`;
   MSRV/toolchain; Makefile + CI (fmt, `clippy -D warnings`, test, coverage 95%+).
   Reuse migrated Makefile/.github. **DoD:** `make check` green on empty crates.
2. **A1.2 — Stable identity core** (odm-core). ULID via the `ulid` crate; `Id`
   newtype; allocation; never-reuse; `Node` skeleton (id, number, type, name,
   created/updated, `origin`, `reserved`); `NodeType`. **Tests:** proptest id
   uniqueness/stability; number→id resolution.
3. **A1.3 — Frontmatter schema + round-trip** (odm-core + odm-store). serde schema
   (edges block, tags, component, `supersedes` w/ kind, status w/ evidence);
   canonical field order; unknown-key preservation. **Tests:** proptest
   `parse ∘ emit = identity`. Reuse legacy `extract`/`normalize`/`filename`.
4. **A1.4 — Store layer** (odm-store). `nodes/YYYY/MM/<ULID>.md` path-from-ULID;
   atomic write-temp-rename **+ fsync** (0014); `gix` (stage/commit/status);
   `odm.toml` via confyg; full-scan load. **DoD:** persist + reload a node set,
   git-tracked.
5. **A1.5 — Node CRUD commands** (odm-cli). `new` (idempotent describe-or-create),
   `list` (full scan), `show`, `rename`, `retire`, `supersede --with --kind`;
   `use`/`context`; `--dry-run`/`--yes`/`--json`. **Tests:** `assert_cmd`.
6. **A1.6 — `check` v1 + link-integrity** (odm-core/cli). Frontmatter completeness;
   no dangling `part_of`/`supersedes`/edges; supersession-chain integrity; exit
   codes; errors that name the fix. (Graph checks — cycles, out-of-order,
   recomposition — arrive in A2.)

Each slice is built with a ledger (collaboration-framework discipline): CC
implements, an independent reviewer verifies per-row before close.

## 4. Failure-mode coverage (post-mortem ODD-0001 → arc)

| Finding (0001) | Closed by |
|---|---|
| A1–A3 identity = name+sequence, renumber rot | **A1** stable ULID ids + link-integrity; order **A2** |
| A4 vocabulary drift | **A1** canonical vocab · **A6** `migrate` |
| A5 no reserved namespace / cross-stream blindness | **A1** `reserved`/`origin` · **A3** rollup shows all streams |
| B1–B4 deps as prose, no graph, no guard, chat-sequencing | **A2** edges-as-data, `next`/`blocked`/`path`, staleness guard |
| B5 recency bias | **A2** `next` (advisory priority later, optional) |
| C1 no SoT for state | **A3** generated rollup |
| C2 prod-DB drift (marquee) · C3/C4/F1/F4 reconcile gaps | **A5** desired-facts + reconciler (freeze-harness generalized) |
| C5 stale docs vs committed decision | **A5** `affects` edge + stale-doc check *(new — see §5)* |
| D1/D2 binary status, ad-hoc qualifiers | **A2** multi-gate vectors + configurable gate-sets |
| D3 evidence level untracked | **A2** evidence-level on gates *(new — see §5)* |
| E1/G1 vision lost, context tax | **A3** `orient` leads with vision · **A6** PM-skill rule |
| E2 intent vs emergent blurred | **A1** `origin` · **A3** original-vs-emergent view |
| E3/E4 stakeholder mismatch, no program DoD | **A5** program-level acceptance facts |
| E5 deferrals in prose | **A3** deferred + re-entry predicate *(new — see §5)* |
| F2/F3/G3/G4 unverified claims, no CI gate | **A2/A6** `check` in CI · PM-skill evidence rules |
| G2/G5/G6 altitude, question-calibration, continuity | **A6** PM skill · **A3** rollup as shared memory |

Every numbered finding maps to a shipped capability — and the MVP (A1–A3) alone
closes the majority. The one-paragraph prevention summary of 0001 ("a complete
dependency graph + a desired-vs-actual reconciler, lifted to the program level")
is exactly A2 + A5.

## 5. New requirements surfaced by the post-mortem → folded into 0013 (v1.4)

1. **Evidence-level on gate transitions** (0001-D3): each `status` entry records
   `evidence ∈ asserted | attested | reproduced | reconciled`. Lands in **A2**.
2. **`affects` edge + stale-doc-vs-decision check** (0001-C5): a decision node
   `affects` the docs it touches; `check`/`reconcile` flags a doc that contradicts
   a committed decision. Edge in **A2**, check in **A5**.
3. **Deferred re-entry predicate** (0001-E5): `deferred` is a first-class status
   carrying a *checkable* re-entry condition (a `desired_fact`/probe), surfaced in
   the rollup. Lands in **A3** (surfacing) + **A5** (predicate evaluation).

## 6. Method & sequencing

- **Ledger per slice;** independent verification before close; 95%+ coverage.
- **Self-host early:** once A1–A3 land, migrate 0011–0015 + this plan *into* `odm`
  and run the build from the tool itself.
- **Benchmark milestone** (A4): the 100k-node corpus from 0014, to promote its
  performance `[P]` claims to `[E]`.
- **Parallelism:** 0014 (index research) is done; A4 is unblocked but post-MVP.
  The PM-skill (A6) waits on a stable command surface (A1–A3) and is seeded by
  ODD-0001 (now in hand). A5 needs A2's gate model.

## 7. Next SDLC step

Open the **A1.1 ledger** and begin the workspace scaffolding — the first buildable
slice.
