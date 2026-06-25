# Arc 02 — Graph, gates & derived order (plan-of-record)

> The arc that turns `odm` into *the build system for the plan*. Refs: ODD-0013
> §3–§5 (+ §4.4 evidence-leveled satisfaction, v1.5), ODD-0015 §1 (A2), ODD-0001.
> `depends_on:` Arc 01 (the node substrate must exist).

## Goal

Make order, readiness, and confidence *derived* and *checkable*: typed edges as
data, the petgraph DAG with cycle detection + explicit tears, multi-gate status
vectors with evidence levels, satisfaction (incl. **evidence-leveled** + threshold
+ min-propagation), the derived-order queries, the staleness guard,
decomposition/recomposition integrity, and `check` v2.

## Exit criteria (arc acceptance)

- Dependencies are queryable edges; `next`/`blocked`/`path`/topo answer correctly.
- Cycles are detected (Kahn) and require an explicit tear; no silent loops.
- Status is a multi-gate vector with per-type gate-sets and an evidence level per
  transition.
- A dependency satisfied only below the evidence threshold (default `reproduced`)
  is **soft-satisfied**: surfaced in `next`/`blocked`, warned by `check`.
- `check` v2 fails on cycles-without-tears, dangling refs, out-of-order work,
  broken recomposition, and (strict mode) below-threshold satisfaction.

## Slices (dependency-ordered)

1. **slice01 — graph construction + reverse edges** — build the petgraph DAG from
   the edge data parsed in arc01 (the edge *schema* + link-integrity already exist
   from arc01 slice03/06); derive reverse edges/backlinks; select the ordering DAG
   (`depends_on ∪ consumes`). — `odm-graph`/`odm-core`.
2. **slice02 — cycle detection + tears** (Kahn) — `odm-graph`.
3. **slice03 — multi-gate status ops + per-type gate-sets + evidence recording** —
   gate-set config (`odm.toml`), the status vector, `set-gate` with an evidence
   level per transition. — `odm-core` (+ `odm.toml`).
4. **slice04 — derived order & satisfaction** (`next`/`blocked`/`path`/topo,
   satisfaction, **evidence-leveled satisfaction + threshold + min-propagation**,
   staleness guard) — `odm-graph`/`odm-core`. ← carries the evidence-level work.
5. **slice05 — decomposition/recomposition integrity** (+ `decomposed: complete`,
   realized as a typed `Decomposition { on, children }` to enable the drift guard).
5b. **slice05.1 — evidence-transition dates** *(inserted)* — `GateRecord` gains a
   first-reach-per-level `evidence_dates` map: the verification-latency *signal*
   (captured, not yet consumed). Groundwork for the A7 telemetry/forecasting layer.
   Back-compat: pre-field nodes round-trip byte-identically. (Numbered `05.1` as a
   bootstrap-phase bisection — the very `Phase 8.5` pattern odm will make
   impossible once self-hosting; see its CDC note.)
6. **slice06 — `check` v2** (the lynchpin gate).
7. **slice07 — CLI graph-mutators** *(added)* — `link`/`unlink`, `set-gate`, `tear`,
   `new --parent`: wires the engine to the command surface so a graph can be built +
   advanced through `odm` alone (no hand-editing). **Self-hosting prerequisite**
   (surfaced in slice04 CDC). On its close, **odm is self-host-usable and Arc 02 is
   done.**

Evidence-leveled satisfaction spans **slice03** (recording the evidence level on
each gate transition) and **slice04** (consuming it: min-propagation, threshold,
soft-satisfied surfacing). ODD-0013 §4.4 is the spec.

## Method

Ledger per slice; CC implements, CDC verifies every row (compile/test rows via CI
or a local 1.85+ toolchain — the sandbox has none); five-iteration cap.
