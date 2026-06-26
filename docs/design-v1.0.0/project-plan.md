# odm v1.0.0 — Project Plan (arc roadmap)

> The project's **plan-of-record**: the arc roadmap, in dependency order, with the
> capability each arc delivers and the current status of each. This is the document a
> fresh session reads to understand **all the arcs at once** before opening any single
> `arc-plan.md`. Per `docs/PROJECT-MANAGEMENT.md` Part III.
>
> **Synthesis note (2026-06-26):** created mid-project, after the MVP (A1–A3) had
> already shipped, by synthesizing the project definition (ODD-0012) and the
> arc/slice breakdown (ODD-0015) into the canonical `project-plan.md` shape. ODD-0012
> remains the SDLC step-2 definition and ODD-0015 the step-4 breakdown; this file is
> the *living* roadmap that arc-close bubble-ups maintain from here forward.

## 1. Definition of done & boundaries

`odm` is a markdown/git-native, dependency-ordered planning + documentation substrate
that **mechanically actualizes** the collaboration framework: stable-identity nodes +
an explicit dependency DAG + order *derived* by topological sort + per-edge
staleness/reconciliation + one *complete* graph as the source of truth. Success test:
a fresh session reaches full situational awareness from `odm orient` alone.

The architecture is fixed in **ODD-0013** (this file is the plan, not the design).

**In scope (v1.0.0):** one unified node graph (work nodes project/arc/slice +
document nodes odd/adr/note); stable ULID identity; typed edges → petgraph DAG; cycle
detection + explicit tears; derived-order queries (`next`/`blocked`/`path`); multi-gate
status vectors with evidence levels + evidence-leveled satisfaction; mechanical
integrity checking (`check`); generated rollup + `orient`; incremental index;
desired-vs-actual reconciliation; a legacy `migrate` importer + self-hosting; LLM
ergonomics (`--json` everywhere, question-named commands, errors-as-affordances,
idempotent describe-or-create, `--dry-run`/`--yes`, bare `odm` orients).

**Non-goals (ODD-0012):** no ticketing system / server / database (files are the
source, `odm` is the build); not a scheduler/optimizer (dependency order is *correct*,
not *fastest*; priority is an optional advisory layer, not in the MVP); no attempt at
*complete* traceability (dependency + verification edges, not everything); no
preservation of the legacy on-disk truth-encoding (number-as-identity,
state-in-directory, dustbin) — migrated *into* the new model, not carried forward.

## 2. The arc roadmap (dependency order)

Order is *derived* from dependencies (dogfooding the model). **MVP = A1–A3.** Each arc
is independently demoable.

| Arc | Capability | `depends_on` | Crates |
|-----|-----------|--------------|--------|
| **A1 — Substrate & node CRUD** | Workspace + crates; ULID identity; node types; frontmatter schema; `nodes/YYYY/MM/<ULID>.md` store; `gix`; `odm.toml`; CRUD + `use`/`context`; `check` v1. | — | odm-core, odm-store, odm-cli, oxur-odm, odm-graph (stub) |
| **A2 — Graph, gates & derived order** | Typed edges; petgraph DAG; cycles + tears; multi-gate status with evidence; evidence-leveled satisfaction (threshold + min-propagation); `next`/`blocked`/`path`/topo; staleness guard; decomposition/recomposition integrity; `check` v2. | A1 | odm-graph, odm-core |
| **A3 — Rollup & orient** | Generated `ROLLUP.md` (+`--json`); `orient`/`brief` (vision → focus → ready/blocked → integrity → drift); provenance view; errors-as-affordances; bare-`odm` orients. **← MVP COMPLETE** | A2 | odm-core, odm-cli |
| **A4 — Index & cache** | Incremental, DB-free/FTS-free stat-cache under `.odm/` (ODD-0014); replaces full-scan in `list`/`orient`/graph-build; racy-git-correct; self-healing; 100k-node benchmark. | A3 | odm-index |
| **A5 — Reconciliation** | `desired_facts` + probe trait (shell/file); `reconcile` on demand + scheduled; drift folded into rollup/orient (replaces the A3 placeholder); `affects` edge + stale-doc-vs-decision check; deferred surfacing + re-entry predicate. | A2, A3 | odm-reconcile |
| **A6 — Migrate, self-host & PM-skill** | `migrate` importer (idempotent, `--dry-run`, supersede-not-delete); run on odm's own docs; self-host the plan; populate the PM skill from ODD-0001; retire redundant framework prose → "run `odm check`". | A1, A3 | odm-migrate; `billosys/ai-engineering` |

**Dependency note:** A4, A5, and A6 all depend only on A1–A3 (now done), so none is
forced as "next" by the graph — their order is a sequencing *choice*, recorded in §3.

## 3. Current status (2026-06-26)

- **A1 — COMPLETE.** Merged to `main`, CI-green.
- **A2 — COMPLETE.** Merged to `main`, CI-green.
- **A3 — COMPLETE.** All four slices (cleanup, rollup, orient, `--json`) CDC-verified
  and merged to `main`; pushed for CI. The arc-close recomposition check passed with no
  silent drops (`arc03-rollup-and-orient/arc-close.md`). **MVP (A1–A3) is done.**
- **A4 / A5 / A6 — PLANNED, not started.** Arc-plans drafted this session
  (`arcNN-<slug>/arc-plan.md`); per *plan late, plan deep*, no per-slice doc sets exist
  yet — those are written when each arc becomes the active work.

**Next-arc sequencing (open — operator's call):** dependencies leave A4/A5/A6 free to
order. The recurring signal across the build is that **A3 triggers self-hosting**
(A6's migrate + self-host), which would make odm dogfood its own plan immediately;
A4 (index/perf) is optimization the corpus does not yet need; A5 (reconciliation)
closes the marquee state-drift class. A reasonable case exists for **A6 next** (realize
the self-hosting payoff) over the 0015 numeric order. Decide when picking the next arc.

## 4. Beyond v1.0.0 (horizon — not yet scoped as arcs)

A post-A6 thread is live but **not** part of the v1.0.0 roadmap: **A7 two-clock
telemetry** → **A8 forecasting** (research-gated in ODD-0018; Monte-Carlo-PERT over the
real DAG with measured node-time distributions). Interop (ODD-0017: export-projection
out + reference-and-reconcile in) rides alongside A5/A6. Listed here so the horizon
stays visible; these become roadmap arcs only when v1.0.0 closes and they are scoped.

## Version History

### v1.0 — 2026-06-26
Initial `project-plan.md`, created mid-project (after A1–A3 shipped) by synthesizing
ODD-0012 (definition) + ODD-0015 (arc/slice breakdown). Reflects current reality:
A1–A3 complete and merged; A4–A6 planned with arc-plans drafted this session. Carries
forward the A3-close reconciliation already applied (deferred-node surfacing moved from
A3 to A5, per Q-A3-1). Subsequent entries grow one per arc-close bubble-up, naming the
arc that surfaced the change (per `docs/PROJECT-MANAGEMENT.md` Part V).
