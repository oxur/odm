<!-- GENERATED — do not edit by hand. Regenerate with `odm rollup`. fingerprint=aeb13776c6efb5ebfe117dccd94645b39c52795ce6acbfd30824e3a459096f3b -->

# Rollup

## Way-finding tree

- odd #9 Oxur Design Documentation CLI - Extended Features Plan (Phases 6-8) — draft=asserted, under-review=asserted, revised=asserted, accepted=asserted, active=asserted, final=asserted
- odd #11 Research: A markdown/git-native, dependency-ordered planning system — draft=asserted, under-review=asserted, revised=asserted, accepted=asserted, active=asserted, final=asserted
- odd #10 Oxur Design Documentation CLI - Phases 9-10 Build Plan — draft=asserted, under-review=asserted, revised=asserted, accepted=asserted, active=asserted, final=asserted
- odd #17 Interop — projection out, reference-and-reconcile in — draft=asserted, under-review=–, revised=–, accepted=–, active=–, final=–
- odd #15 odm — Arc/Slice Breakdown (build plan) — draft=asserted, under-review=–, revised=–, accepted=–, active=–, final=–
- odd #16 Research — SWE project- & epic-level PM failures and best practices — draft=asserted, under-review=asserted, revised=asserted, accepted=asserted, active=asserted, final=asserted
- odd #19 Incremental drift — probes as input-tracked rules over the stat-cache, with honest staleness — draft=asserted, under-review=asserted, revised=asserted, accepted=asserted, active=–, final=–
- odd #18 Research — Forecasting under small, bursty, DAG-structured work — draft=asserted, under-review=–, revised=–, accepted=–, active=–, final=–
- odd #2 Oxur Design Documentation CLI - Build Plan — draft=asserted, under-review=asserted, revised=asserted, accepted=asserted, active=asserted, final=asserted
- odd #14 Research — odm-index: incremental indexing & caching (no DB, no FTS) — draft=asserted, under-review=asserted, revised=asserted, accepted=asserted, active=asserted, final=asserted
- odd #13 odm — Architecture & Design (v-major rebuild) — draft=asserted, under-review=–, revised=–, accepted=–, active=–, final=–
- odd #12 odm — Project Definition (v-major rebuild) — draft=asserted, under-review=asserted, revised=asserted, accepted=asserted, active=asserted, final=asserted
- odd #20 Versioned file-metadata schemas — per-type schema markers, v0.1 → v1.0 — draft=asserted, under-review=asserted, revised=asserted, accepted=asserted, active=–, final=–
- project #1000 odm v1.0.0 — Project Plan (arc roadmap) — planned=asserted, in-progress=asserted, complete=–, verified=–
  - arc #1400 Arc 04 — Index & cache (plan-of-record) — planned=asserted, in-progress=asserted, complete=asserted, verified=asserted
    - slice #1407 Slice 07 (Arc 04) — Early-cutoff invalidation (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1401 Slice 01 (Arc 04) — Index record + snapshot persistence (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1405 Slice 05 (Arc 04) — Index→graph adapter + wire graph readers & composed views (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1406 Slice 06 (Arc 04) — Enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient` (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1408 Slice 08 (Arc 04) — Benchmark harness (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1403 Slice 03 (Arc 04) — Warm-path change detection (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1402 Slice 02 (Arc 04) — Cold-path build (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1404 Slice 04 (Arc 04) — Enrich record + wire consumers (plan-of-record) — planned=asserted, built=asserted, tested=asserted
  - arc #1100 Arc 01 — Substrate & node CRUD (plan-of-record) — planned=asserted, in-progress=asserted, complete=asserted, verified=asserted
    - slice #1102 Slice 02 — Stable identity core (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1103 Slice 03 (Arc 01) — Frontmatter schema + round-trip (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1106 Slice 06 (Arc 01) — `check` v1 + link-integrity (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1105 Slice 05 (Arc 01) — Node CRUD commands (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1101 Slice 01 — Workspace scaffolding (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1104 Slice 04 (Arc 01) — Store layer (plan-of-record) — planned=asserted, built=asserted, tested=asserted
  - arc #1600 Arc 06 — Migrate, self-host & PM-skill (plan-of-record) — planned=asserted, in-progress=asserted, complete=–, verified=–
    - slice #1604 Slice 04 (Arc 06): self-host cutover — planned=asserted, built=asserted, tested=asserted
    - slice #1603 Slice 03 (Arc 06): schema versioning (ODD-0020) — planned=asserted, built=asserted, tested=asserted
    - slice #1602 Slice 02 (Arc 06): migrate odm's own docs — planned=asserted, built=asserted, tested=asserted
    - slice #1601 Slice 01 (Arc 06): `migrate` importer core — planned=asserted, built=asserted, tested=asserted
  - arc #1500 Arc 05 — Reconciliation (plan-of-record) — planned=asserted, in-progress=asserted, complete=asserted, verified=asserted
    - slice #1505 Slice 05 (Arc 05): `affects` edge + stale-doc-vs-decision check (C5) — planned=asserted, built=asserted, tested=asserted
    - slice #1502 Slice 02 (Arc 05): `file` probe + probe-runner — planned=asserted, built=asserted, tested=asserted
    - slice #1506 Slice 06 (Arc 05): deferred surfacing + re-entry predicate (Q-A3-1) — planned=asserted, built=asserted, tested=asserted
    - slice #1507 Slice 07 (Arc 05): incremental drift — probes-as-rules + the `.odm/` drift snapshot — planned=asserted, built=asserted, tested=asserted
    - slice #1503 Slice 03 (Arc 05): `odm reconcile` (on demand) — planned=asserted, built=asserted, tested=asserted
    - slice #1501 Slice 01 (Arc 05): `desired_facts` schema + `Probe` trait + shell probe — planned=asserted, built=asserted, tested=asserted
    - slice #1508 Slice 08 (Arc 05): freshness on every command + honest staleness (the arc capstone) — planned=asserted, built=asserted, tested=asserted
    - slice #1504 Slice 04 (Arc 05): drift in `rollup` / `orient` — planned=asserted, built=asserted, tested=asserted
  - arc #1300 Arc 03 — Rollup & orient (plan-of-record) — planned=asserted, in-progress=asserted, complete=asserted, verified=asserted
    - slice #1301 Slice 01 (Arc 03) — Arc 02 cleanup (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1302 Slice 02 (Arc 03) — Rollup generation (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1304 Slice 04 (Arc 03) — `--json` + polish (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1303 Slice 03 (Arc 03) — orient / brief + bare-`odm` (plan-of-record) — planned=asserted, built=asserted, tested=asserted
  - arc #1200 Arc 02 — Graph, gates & derived order (plan-of-record) — planned=asserted, in-progress=asserted, complete=asserted, verified=asserted
    - slice #1202 Slice 02 (Arc 02) — Cycle detection + tears (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1203 Slice 03 (Arc 02) — Gates, status & evidence recording (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1201 Slice 01 (Arc 02) — Graph construction + reverse edges (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1207 Slice 07 (Arc 02) — CLI graph-mutators (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1206 Slice 06 (Arc 02) — `check` v2 (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1251 Slice 05.1 (Arc 02) — Evidence-transition dates (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1205 Slice 05 (Arc 02) — Decomposition/recomposition integrity (plan-of-record) — planned=asserted, built=asserted, tested=asserted
    - slice #1204 Slice 04 (Arc 02) — Derived order & satisfaction (plan-of-record) — planned=asserted, built=asserted, tested=asserted

## Ready

- odd #17 Interop — projection out, reference-and-reconcile in
- odd #15 odm — Arc/Slice Breakdown (build plan)
- odd #19 Incremental drift — probes as input-tracked rules over the stat-cache, with honest staleness
- odd #18 Research — Forecasting under small, bursty, DAG-structured work
- odd #13 odm — Architecture & Design (v-major rebuild)
- odd #20 Versioned file-metadata schemas — per-type schema markers, v0.1 → v1.0
- project #1000 odm v1.0.0 — Project Plan (arc roadmap)
- arc #1600 Arc 06 — Migrate, self-host & PM-skill (plan-of-record)

## Blocked

_(nothing blocked)_

## Active tears

_(none)_

## Provenance

### Planned

- odd #2 Oxur Design Documentation CLI - Build Plan
- odd #9 Oxur Design Documentation CLI - Extended Features Plan (Phases 6-8)
- odd #10 Oxur Design Documentation CLI - Phases 9-10 Build Plan
- odd #11 Research: A markdown/git-native, dependency-ordered planning system
- odd #12 odm — Project Definition (v-major rebuild)
- odd #13 odm — Architecture & Design (v-major rebuild)
- odd #14 Research — odm-index: incremental indexing & caching (no DB, no FTS)
- odd #15 odm — Arc/Slice Breakdown (build plan)
- odd #16 Research — SWE project- & epic-level PM failures and best practices
- odd #17 Interop — projection out, reference-and-reconcile in
- odd #18 Research — Forecasting under small, bursty, DAG-structured work
- odd #19 Incremental drift — probes as input-tracked rules over the stat-cache, with honest staleness
- odd #20 Versioned file-metadata schemas — per-type schema markers, v0.1 → v1.0
- project #1000 odm v1.0.0 — Project Plan (arc roadmap)
- arc #1100 Arc 01 — Substrate & node CRUD (plan-of-record)
- slice #1101 Slice 01 — Workspace scaffolding (plan-of-record)
- slice #1102 Slice 02 — Stable identity core (plan-of-record)
- slice #1103 Slice 03 (Arc 01) — Frontmatter schema + round-trip (plan-of-record)
- slice #1104 Slice 04 (Arc 01) — Store layer (plan-of-record)
- slice #1105 Slice 05 (Arc 01) — Node CRUD commands (plan-of-record)
- slice #1106 Slice 06 (Arc 01) — `check` v1 + link-integrity (plan-of-record)
- arc #1200 Arc 02 — Graph, gates & derived order (plan-of-record)
- slice #1201 Slice 01 (Arc 02) — Graph construction + reverse edges (plan-of-record)
- slice #1202 Slice 02 (Arc 02) — Cycle detection + tears (plan-of-record)
- slice #1203 Slice 03 (Arc 02) — Gates, status & evidence recording (plan-of-record)
- slice #1204 Slice 04 (Arc 02) — Derived order & satisfaction (plan-of-record)
- slice #1205 Slice 05 (Arc 02) — Decomposition/recomposition integrity (plan-of-record)
- slice #1206 Slice 06 (Arc 02) — `check` v2 (plan-of-record)
- slice #1207 Slice 07 (Arc 02) — CLI graph-mutators (plan-of-record)
- slice #1251 Slice 05.1 (Arc 02) — Evidence-transition dates (plan-of-record)
- arc #1300 Arc 03 — Rollup & orient (plan-of-record)
- slice #1301 Slice 01 (Arc 03) — Arc 02 cleanup (plan-of-record)
- slice #1302 Slice 02 (Arc 03) — Rollup generation (plan-of-record)
- slice #1303 Slice 03 (Arc 03) — orient / brief + bare-`odm` (plan-of-record)
- slice #1304 Slice 04 (Arc 03) — `--json` + polish (plan-of-record)
- arc #1400 Arc 04 — Index & cache (plan-of-record)
- slice #1401 Slice 01 (Arc 04) — Index record + snapshot persistence (plan-of-record)
- slice #1402 Slice 02 (Arc 04) — Cold-path build (plan-of-record)
- slice #1403 Slice 03 (Arc 04) — Warm-path change detection (plan-of-record)
- slice #1404 Slice 04 (Arc 04) — Enrich record + wire consumers (plan-of-record)
- slice #1405 Slice 05 (Arc 04) — Index→graph adapter + wire graph readers & composed views (plan-of-record)
- slice #1406 Slice 06 (Arc 04) — Enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient` (plan-of-record)
- slice #1407 Slice 07 (Arc 04) — Early-cutoff invalidation (plan-of-record)
- slice #1408 Slice 08 (Arc 04) — Benchmark harness (plan-of-record)
- arc #1500 Arc 05 — Reconciliation (plan-of-record)
- slice #1501 Slice 01 (Arc 05): `desired_facts` schema + `Probe` trait + shell probe
- slice #1502 Slice 02 (Arc 05): `file` probe + probe-runner
- slice #1503 Slice 03 (Arc 05): `odm reconcile` (on demand)
- slice #1504 Slice 04 (Arc 05): drift in `rollup` / `orient`
- slice #1505 Slice 05 (Arc 05): `affects` edge + stale-doc-vs-decision check (C5)
- slice #1506 Slice 06 (Arc 05): deferred surfacing + re-entry predicate (Q-A3-1)
- slice #1507 Slice 07 (Arc 05): incremental drift — probes-as-rules + the `.odm/` drift snapshot
- slice #1508 Slice 08 (Arc 05): freshness on every command + honest staleness (the arc capstone)
- arc #1600 Arc 06 — Migrate, self-host & PM-skill (plan-of-record)
- slice #1601 Slice 01 (Arc 06): `migrate` importer core
- slice #1602 Slice 02 (Arc 06): migrate odm's own docs
- slice #1603 Slice 03 (Arc 06): schema versioning (ODD-0020)
- slice #1604 Slice 04 (Arc 06): self-host cutover

### Discovered

_(none)_

### Amendment

_(none)_

## Drift

_No drift._

