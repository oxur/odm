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
- **A4 — Index & cache: CLOSED (composed; CI-pending).** All 8 slices CDC-verified; the
  arc-close composition check passed with an independent arc-gate review
  (`arc04-index-cache/closing-report.md`, PASS-WITH-NOTES). The index/cache capability
  lands: `list`/`orient`/graph-build/`check`/`rollup` are index-backed and match baseline,
  change detection is racy-git-correct, the index self-heals, and the 100k benchmark
  promoted ODD-0014's `[P]` perf claims to `[E]`. Executable/number rows route to CI
  (sandbox has no 1.85+ toolchain); P-4 flips `done` on CI-green + merge. Forward-carried:
  the **adapter-fidelity invariant** is now a hard gate in `arc05`'s arc-plan (v1.2).
- **A5 / A6 — PLANNED, not started.** Arc-plans drafted this session
  (`arcNN-<slug>/arc-plan.md`); per *plan late, plan deep*, no per-slice doc sets exist
  yet — those are written when each arc becomes the active work.

**Next-arc sequencing (open — operator's call):** dependencies leave A4/A5/A6 free to
order. The recurring signal across the build is that **A3 triggers self-hosting**
(A6's migrate + self-host), which would make odm dogfood its own plan immediately;
A4 (index/perf) is optimization the corpus does not yet need; A5 (reconciliation)
closes the marquee state-drift class. A reasonable case exists for **A6 next** (realize
the self-hosting payoff) over the 0015 numeric order. Decide when picking the next arc.

## 4. Post-MVP extension roadmap (v1.0.0+)

> **Changed in v1.2 (2026-06-26, post-arc6 thread w/ Duncan).** This section was
> *"Beyond v1.0.0 (horizon — not yet scoped as arcs)"*, which listed A7/A8 and
> interop as horizon-only and deferred their scoping until v1.0.0 closes. The
> post-arc6 design thread worked the telemetry→forecasting line down to arc/slice
> altitude and research-gated A8 (ODD-0018), so **A7 and A8 are promoted to scoped
> extension arcs with `arc-plan.md`s**, ahead of the A6-close sequencing, at
> operator request. The remaining items stay **tentative (roadmap-only)** per *plan
> late, plan deep* — no arc-plans are manufactured for work not yet worked out.
> (The old single-paragraph horizon note is expanded here, not deleted — its A7/A8
> detail moved into the table below + the two new arc-plans.)

These arcs **extend the v1.0.0 line** — they do not change the core MVP/DoD (A1–A6,
§1/§5). Numbers are labels, not a fixed order (the model orders by dependency);
export, for instance, needs only A3 and could land before A7 if prioritized.
**Open call:** whether these warrant a design-version bump to a `design-v1.1.0/`
tree, or stay in this one, is unsettled — flagged in Version History.

| Arc | Capability | `depends_on` | Status |
|-----|-----------|--------------|--------|
| **A7 — Two-clock telemetry** | Event log derived from gates+git; two-clock decomposition (active-work vs inter-slice latency); the **language-agnostic** covariate collection layer + **structured CDC-verification emission**; status×gate evidence-matrix view. *Collect + describe — no forecasting; independently valuable.* | A2, A3 (matures w/ A5) | **scoped** — `arc07-two-clock-telemetry/arc-plan.md`; design-ahead (builds post-A6 / self-host) |
| **A8 — Forecasting** | Monte-Carlo over the real DAG with measured, censoring-corrected node-time distributions; Bayesian hierarchical reference classes; the **total-cycle control + two-clock experiment**; honest self-widening ranges, never dates. | A7, A2 | **scoped** — `arc08-forecasting/arc-plan.md`; **research-gated by ODD-0018** |
| Interop — export (out) | One-way, honestly-lossy projection of the rollup into another team's vocabulary; also the evangelism engine (ODD-0017 Part A). | A3 | tentative (designed in 0017) |
| Interop — reference-and-reconcile (in) | `external` node type + cross-team edges; status **reconciled, never authored**, evidence-leveled (ODD-0017 Part B). | A5 | tentative (designed in 0017) |
| Visualization | On-demand DAG render (`ascii-dag`) + roadmap-as-projection; a renderer over the rollup + the A7 matrix (same projection layer as export). | A3, A7 | tentative |
| Advisory priority | An optional, non-authoritative ranking *on top of* the derived order — never the order itself (ODD-0012 non-goal boundary). | A2 | tentative |
| (Saga tier) | Multi-version vision spanning several projects (PROJECT-MANAGEMENT Part I); named only to mark the slot. | — | exploratory; no operational weight |

**Carried constraints (A7/A8):** language-agnostic covariates only (no code-AST);
**metadata, never targets; team/process, never per-actor** (Goodhart); the two-clock
split is a *hypothesis* tested against the total-cycle control (ODD-0018), not assumed.
Proposed new crates: `odm-telemetry` (A7), then `odm-forecast` (A8).

## 5. Project Ledger

> Per LEDGER-DISCIPLINE v2.0 §C (**provisional** — the project tier is validated by
> analogy and established practice, not yet by a closed project; revised against
> experience when the MVP/v1.0.0 closes). Option A: the ledger opens here and closes in
> a project-level `closing-report.md`. **Definition of done:** the full v1.0.0 (A1–A6,
> §1); the **MVP boundary is A1–A3**. Class-(b) rows are reproduced at *project scale* —
> an end-to-end acceptance demonstration — and the DoD is partly an operator *judgment*,
> recorded as such, never inherited from the arcs.
>
> **Scope note (extension arcs, v1.3).** This ledger verifies the **v1.0.0 DoD
> (A1–A6) only.** The post-MVP extension arcs in §4 — **A7/A8** (scoped) and the
> tentative interop/visualization/priority/Saga — are **intentionally outside** it:
> each scoped extension arc carries its **own** arc-ledger (`arc07`/`arc08`), and they
> enter a *project* ledger only if/when the §4 version-tree call folds them into a
> versioned DoD (e.g. a `design-v1.1.0/` project-plan with its own ledger). They are
> **out-of-DoD by design, not silent drops.** No A7/A8 rows are added below pending
> that call.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| P-1 | A1 (substrate & node CRUD) closed + composed | ptr: arc01 close | correctness | project-plan | done | merged to `main`, CI-green | **Disclosed gap:** A1 predates the v2.0 arc-close discipline — no formal arc `closing-report.md`. Composed-in-fact (the MVP runs on it). |
| P-2 | A2 (graph, gates & derived order) closed + composed | ptr: arc02 close | correctness | project-plan | done | merged to `main`, CI-green | Same disclosed gap as P-1 (no formal arc closing-report). |
| P-3 | A3 (rollup & orient) closed + composed | ptr: `arc03-rollup-and-orient/arc-close.md` | correctness | project-plan | done | arc-close composition check passed, no silent drops; merged | A3's bubble-up forced no project-plan change (recorded). (File is `arc-close.md`; canonical name is `closing-report.md`.) |
| P-4 | A4 (index & cache) closed + composed | ptr: `arc04-index-cache/closing-report.md` | correctness | project-plan | open | attested: arc-close done (`arc04-index-cache/closing-report.md`) — 8/8 slices CDC-verified, class-(b) composition rows A-9…A-13 reproduced at arc scale **on structure** (end-to-end tests span the slices), independent arc-gate review **PASS-WITH-NOTES**; ODD-0014 `[P]`→`[E]`. Executable/number rows pending CI. | → `done` on CI-green + merge. Composed-in-fact now; the green checkmark is CI's. Delivered set is *broader* than the roadmap line (also `check`+`rollup`) — over-delivery, not drift. |
| P-5 | A5 (reconciliation) closed + composed | ptr: arc05 closing-report | correctness | project-plan | open | | attested-on-close. |
| P-6 | A6 (migrate, self-host & PM-skill) closed + composed | ptr: arc06 closing-report | correctness | project-plan | open | | attested-on-close. |
| P-7 | **Compose (DoD):** a fresh session reaches full situational awareness from `odm orient` alone | project-scale demo: fresh session, `odm orient` only | serious | project-plan / 0015 §2 | open | | MVP-demonstrable now; reproduce at project scale on close. The headline DoD. |
| P-8 | **Compose (DoD):** every dependency is a queryable edge; `next`/`blocked`/`path` answer correctly | project-scale demo over a real corpus | serious | project-plan / 0015 §2 | open | | MVP-demonstrable now. |
| P-9 | **Compose (DoD):** `check` catches cycles-without-tears, dangling refs, out-of-order work, broken recomposition | project-scale demo: seed each violation, observe the finding | serious | project-plan / 0015 §2 | open | | MVP-demonstrable now. |
| P-10 | **Compose (DoD):** status is multi-gate with evidence levels | project-scale demo | serious | project-plan / 0015 §2 | open | | MVP-demonstrable now. |
| P-11 | **Compose (DoD):** desired-vs-actual drift is detected and reported (the prod-DB-503 class) | project-scale demo: declare a fact, diverge reality, observe drift | serious | project-plan / 0001-C2 | open | | Lands with A5 (reconciliation). |
| P-12 | **Compose (DoD):** odm self-hosts — manages its own plan as nodes; these design docs are queryable via `odm orient` | project-scale demo: migrate + orient on odm's own corpus | serious | project-plan / 0013 §9 | open | | Lands with A6 (the self-hosting trigger). |
| P-13 | Arc bubble-up findings dispositioned | ptr: project-plan change-log (Version History) | correctness | bubble-up | open | | Accrues per arc close. A3: no project-plan change (recorded). A4: no roadmap re-scope; one finding forward-carried (the adapter-fidelity invariant → arc05 v1.2) — recorded, v1.4. |

Closes in a project-level `closing-report.md` with the per-row walk and the **project
gate** (go / adjust / kill against the DoD, reviewed by the operator + an independent
context). A failed DoD row spawns a **remediation arc** or a roadmap re-scope, not an
unbounded grind.

## Version History

### v1.4 — 2026-06-30
**A4 (Index & cache) closed + composed (CI-pending).** Arc 04 reached its arc-close: all 8
slices CDC-verified, the class-(b) composition rows (A-9…A-13) reproduced at arc scale on
structure (end-to-end tests that span the slices — racy-correctness, self-heal, delta-only,
adapter-fidelity-backed baseline match, 100k benchmark), and an **independent arc-gate
review** (fresh-context subagent) returned PASS-WITH-NOTES — confirming, in particular, that
the A-12 idempotence tests' tautology was *named and avoided* (equivalence rests on the
adapter-fidelity tests), not walked into. ODD-0014's index-engine `[P]` perf claims were
promoted to `[E]`. Updated §3 status (A4 → CLOSED, composed, CI-pending) and **P-4** (open →
attested-on-close; flips `done` on CI-green + merge). **Bubble-up disposition (P-13): no
roadmap re-scope forced** — A4 delivered its capability as the roadmap defined it (the
delivered consumer set is *broader* than the line named: also `check`+`rollup`). One finding
forward-carried, not dropped: the **adapter-fidelity invariant** (A4 removed the `load_all`
A/B net, so any future index reader must extend the adapter + its fidelity test in the same
slice — the CLI idempotence tests pass tautologically and won't catch a regression) is now a
hard gate in **`arc05`'s arc-plan (v1.2)**. Surfaced by: A4 arc-close + its arc-gate review
(`arc04-index-cache/closing-report.md`).

### v1.3 — 2026-06-26
Added a **scope note** to §5 (Project Ledger): the ledger verifies the v1.0.0 DoD
(A1–A6) only; the §4 extension arcs (A7/A8 + tentative) are intentionally out-of-DoD,
each scoped one carrying its own arc-ledger (`arc07`/`arc08`, added this session). **No
A7/A8 DoD rows added** — that awaits the §4 version-tree call (fold into a versioned
DoD vs. a separate `design-v1.1.0/`). Surfaced by: the v1.2 roadmap extension raising
the question of whether extension arcs belong in the v1.0.0 project ledger (they do
not, by design). Pairs with the new `## Arc Ledger` sections in arc07/arc08.

### v1.2 — 2026-06-26
Expanded §4 from a horizon note into the **post-MVP extension roadmap**: promoted
**A7 (two-clock telemetry)** and **A8 (forecasting)** from horizon-only to *scoped*
extension arcs, each now with an `arc-plan.md` (`arc07-two-clock-telemetry/`,
`arc08-forecasting/`), and added interop (export / reference-and-reconcile, ODD-0017),
visualization, advisory-priority, and the Saga tier as *tentative* roadmap entries.
A8 is research-gated by **ODD-0018**. Core v1.0.0 MVP/DoD (§1/§5, A1–A6) unchanged —
these arcs extend the line. **Surfaced by:** the post-arc6 design thread (with Duncan),
*ahead of* the A6-close sequencing that §3/arc06 previously assumed, at operator
request (a deliberate, recorded override of "scope A7+ only when v1.0.0 closes"). Open:
whether A7+ warrant a `design-v1.1.0/` tree vs. staying here.

### v1.1 — 2026-06-26
Added the **`## Project Ledger`** section (class-(a) arcs-closed, class-(b)
DoD-composition, class-(c) bubble-up rows) per LEDGER-DISCIPLINE v2.0 §C, when that
discipline went scale-free. Pure addition — §1–§4 unchanged. A1–A3 marked `done`
(A1/A2 with a disclosed no-formal-arc-close-report gap, as they predate the discipline);
A4–A6 and the DoD-composition rows open. Surfaced by: the ledger-discipline upgrade
(v1→v2.0), not an arc bubble-up.

### v1.0 — 2026-06-26
Initial `project-plan.md`, created mid-project (after A1–A3 shipped) by synthesizing
ODD-0012 (definition) + ODD-0015 (arc/slice breakdown). Reflects current reality:
A1–A3 complete and merged; A4–A6 planned with arc-plans drafted this session. Carries
forward the A3-close reconciliation already applied (deferred-node surfacing moved from
A3 to A5, per Q-A3-1). Subsequent entries grow one per arc-close bubble-up, naming the
arc that surfaced the change (per `docs/PROJECT-MANAGEMENT.md` Part V).
