# odm as a PM tool — arc-6 baseline analysis

> **Purpose.** Baseline + evaluation of `odm`'s projected end-product (close of
> arc 6) viewed as a project-management tool: what it shares with mainstream PM
> tools, what is distinctive, and what it deliberately or incidentally lacks.
> Written 2026-06-24 (CDC + Duncan, the post-arc6 brainstorming thread) to anchor
> the design conversation for arcs 7+.
>
> **Sources.** `odm` side reproduced from ODD-0012 (project def), ODD-0013
> (architecture v1.7), ODD-0015 (arc/slice breakdown), ODD-0017 (interop). PM
> *evidence/practices* layer grounded in ODD-0016 (SWE PM failures & practices).
> The "what mainstream PM tools commonly have" comparison is synthesis from priors
> (Jira, Linear, Asana, GitHub Projects, Monday, Azure DevOps) — evidence level
> `asserted`, not web-verified; verify a specific competitor before any arc-7
> decision leans on it.
>
> **Scope note.** The two interop arcs in ODD-0017 (export-out,
> reference-and-reconcile-in) land *after* A6, so at the close of arc 6 `odm` has
> **no** cross-tool interop yet. This analysis is the A6 endpoint.

---

## 1. Features common to PM tools that odm *will* have

The overlap is real but mostly *structural metadata* — the nouns a PM tool tracks.

| PM-tool feature | odm's form at A6 |
|---|---|
| Work hierarchy (initiative→epic→story→task) | `project → arc → slice` via the `part_of` tree |
| Customizable status workflows | multi-gate vectors, gate-sets configured per node type in `odm.toml` |
| Dependencies (blocks / blocked-by) | typed `depends_on` / `blocked_by` / `consumes` edges |
| Labels & components | `tags` + `component` frontmatter |
| Milestones / releases | arcs (and the gate sequence on them) |
| Acceptance criteria / DoD | the per-slice **ledger** (verifiable rows) |
| Assignment | CC / CDC **roles** (not people — see §3) |
| Search, filters, saved views | `list` with type/gate/ready/blocked/drift filters, `odm-index`-accelerated |
| Dashboards / status reports | generated `ROLLUP.md` + `orient`/`brief` |
| Wiki / docs / decision records | `odd` / `adr` / `note` nodes on the same substrate |
| Activity history / versioning | git + `supersede` lineage |
| Importers | `migrate` |
| Required-check automation | `check` as a CI / pre-commit gate |
| Critical-path / dependency view | `path X [Y]`, topo listing |

## 2. Features odm will have that are *not* common in PM tools

Where it stops being "Jira for agents."

- **Order is derived, not assigned.** PM tools give a drag-rankable backlog; odm
  computes `next`/`blocked`/`path` from the dependency DAG by topological sort.
  Sequence is a *function of edges*, not a human's manual rank. (ODD-0016 §5 makes
  this a deliberate, evidence-backed choice: estimates are systematically
  optimistic, so order by dependency, not by estimate.)
- **Status carries epistemic confidence.** The evidence ladder
  `asserted < attested < reproduced < reconciled`, plus **evidence-leveled
  satisfaction that min-propagates along dependency chains** (ODD-0013 §4.4). No
  mainstream PM tool models *how well we know* a thing is done, let alone
  propagates that down the chain so a relayed "it's done" can't silently unblock
  critical work. Genuinely novel.
- **Reconciliation — drift detection for plans.** `desired_facts` + probes diff
  *declared* state against *observed reality* and report drift (Terraform's lesson
  lifted to project state). PM tools trust the status field; odm can check the
  world and flag when "done" isn't true.
- **Mechanical plan-integrity checking.** `check` fails CI on cycles-without-tears,
  out-of-order work, broken WBS recomposition (the drift-guarded
  `decomposed: complete` assertion), stale-doc-vs-decision (`affects`), and
  dangling refs. PM tools don't structurally *validate* the plan.
- **Explicit `tears`** — DSM-style deliberately-assumed dependencies with required
  rationale. A systems-engineering primitive, alien to PM tools.
- **`orient`-first cheap global state** — a fresh agent reconstitutes full
  situational awareness from one command. Dashboards inform humans; `orient` is
  built for the LLM context-reset tax.
- **Files-as-source, git-native, no server/DB/SaaS.** The plan lives in version
  control beside the code and diffs in PRs. A structural inversion of the
  hosted-tracker model.
- **`origin` (planned/discovered/amendment) + original-vs-emergent view** —
  requirements volatility made first-class and visible.
- **One substrate is self-documenting *and* self-tracking** — docs and work are the
  same node machinery, not a wiki bolted onto a tracker.
- **LLM-native ergonomics** — `--json` everywhere with stable schemas,
  question-named commands, errors-as-affordances, idempotent describe-or-create,
  and the CC/CDC implementer-vs-independent-verifier split baked into the workflow.

## 3. Features most PM tools have that odm will be *missing*

Three very different categories — and the distinction matters more than the list.

### (a) Deliberately rejected on evidence — adding these would betray the thesis
Estimation (story points, t-shirt sizes, effort), velocity, WSJF / cost-of-delay
priority, CCPM buffers, sprint-as-timebox capacity planning. ODD-0016 rates all of
these lore or actively harmful (Goodhart-gameable, no controlled evidence,
estimation doesn't improve with effort). The *evidence-backed substitutes* —
throughput / Monte-Carlo forecasting, reference-class forecasting — odm also
doesn't build yet, and **those are the interesting candidates, not the originals.**

> **Decision (2026-06-24):** Duncan confirmed (a) is out of scope — not wanted,
> not needed.

### (b) Missing because odm's "user" is an LLM + one operator in git, not a human team in a browser
Real people / assignees, permissions & multi-tenancy, notifications & activity
feeds, in-tool threaded comments, real-time collaboration, mobile/web/SaaS access.
ODD-0017's stance ("federate, don't convert — `odm export` is the evangelism
engine") implies odm's answer is *project out to where the humans already are*,
not rebuild their coordination surface.

> **Decision (2026-06-24):** Duncan confirmed (b) is out of scope — better served
> by exporting outward (ODD-0017) than rebuilding inward.

### (c) Genuinely absent but plausibly in-grain for arcs 7+
Roadmap / Gantt / timeline visualization, time-series analytics (burndown, cycle
time, lead time, throughput, CFD), a structured custom-field system (beyond
extensible frontmatter), boards/Kanban rendering, attachments, goal/OKR objects,
recurring/templated nodes.

> **Decision (2026-06-24):** Duncan wants to pull on (c). This is the arc-7+ menu.

## 4. The reframe

Mainstream PM is **issue-oriented and human-actor-centric**: the atom is a ticket a
person owns and drags across a board; the tool's job is coordination, visibility,
and reporting *for humans*. odm is **slice-oriented and evidence/dependency-centric**:
the atom is a verification contract (plan + cc-prompt + ledger), order is derived,
status is an evidence vector reconciled against reality, and the tool's job is to
let an agent regain context and to *refuse to let unverified work look done*. It's
less "Jira for agents" and more **a topological scheduler + a drift reconciler + a
build system, pointed at the plan itself.**

The practical filter that falls out: **odm is strongest exactly where PM tools are
weakest, and vice versa.** PM tools treat dependencies as an afterthought; odm makes
them the ordering substrate. PM tools treat status as a label; odm makes it an
evidence vector reconciled against reality. Conversely, PM tools own human
coordination (notifications, assignment, boards, comments); odm punts all of that to
git + the operator + export.

## 5. The grain filter (for arcs 7+)

The missing capabilities that *fit odm's grain* are the ones that stay **derived,
verifiable, file-native, and evidence-aware**:

- A **roadmap/timeline** is just another *projection of the rollup* — same shape as
  `export` in ODD-0017 (a renderer over generated state, not a new source of truth).
- **Forecasting**, if added, must be **throughput-based and evidence-tagged**, not
  estimate-based (ODD-0016 §5: count small uniform items + empirical throughput /
  Monte-Carlo; never summed story points).
- **Priority**, if added, is **advisory *on top of* derived order**, never the order
  itself (ODD-0012 non-goals; ODD-0011 guardrail).

Features that *don't* fit the grain (boards, notifications, human assignment) are
arguably better served by exporting outward than by rebuilding them inward.

### Candidate arc-7+ threads (the menu)
1. **Forecasting / analytics** — throughput, cycle/lead time, Monte-Carlo
   completion forecast, all derived from gate-reached timestamps + evidence levels.
2. **Visualization / roadmap-as-projection** — render the graph + rollup as a
   timeline / dependency map / decomposition tree; a renderer over existing state.
3. **Advisory priority layer** — an optional ranking *on top of* the derived order,
   explicitly non-authoritative.

(Open: which thread first, and whether any of the three is itself a self-hosted
dogfooding milestone once A6 lands.)
