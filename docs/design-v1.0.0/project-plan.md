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

## Vision

`odm` is a markdown/git-native, dependency-ordered planning + documentation substrate
that **mechanically actualizes** the collaboration framework: stable-identity nodes +
an explicit dependency DAG + order *derived* by topological sort + per-edge
staleness/reconciliation + one *complete* graph as the source of truth. Success test:
a fresh session reaches full situational awareness from `odm orient` alone.

The architecture is fixed in **ODD-0013** (this file is the plan, not the design).

## 1. Definition of done & boundaries

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

### 2a. Named, number-deferred arcs (v1.0.x insertions)

> **Added v1.8 (2026-07-25).** Two arcs that did **not** exist in the original A1–A6
> roadmap were inserted **within the A6 window** after hands-on UAT of the self-hosted
> tool (A6 slice04). They are **named but deliberately un-numbered**: A7/A8 (§4) are
> another CDC's active post-MVP arcs, and the A-numbering scheme is *itself* under review
> inside the Release Hardening arc, so locking an A-number now would be premature. Both
> are v1.0.0 **release-blocking** and are pure **expansion** of the roadmap — nothing in
> A1–A6 is replaced.
>
> **Added v1.9 (2026-07-26):** a **third** named arc — **Store Home & `init`** — joins them, from
> **ODD-0022** (Accepted), *not* from UAT. Same treatment (named, number-deferred, v1.0.x, pure
> expansion). It sequences **before RH C-5**, which re-self-hosts *into* the new store home.
>
> **Added v1.13 (2026-07-27):** a **fourth** named arc — **Migration Fidelity** — from the corpus-reconciliation audit (`reconciliation-audit-2026-07-27.md`), *not* UAT. Same treatment (named, number-deferred, v1.0.x, pure expansion). **Unblocked by ODD-0024 (G-1 closed).** It **subsumes the standing L-8b gate** and sits on the **P-12** self-host DoD path.

| Arc | Capability | `depends_on` | Status |
|-----|-----------|--------------|--------|
| **Release Hardening (RH)** | UAT-driven v1.0.0 hardening of the self-hosted CLI: themed/coloured output via a shared styling crate (extracted from `oxur-cli`); type taxonomy `odd`→`design` + new `research`; tree-structured de-numbered `list`; command renames (`context`→`project`, `path`→`chain`, quiet `new`, format-agnostic `rollup`); fold `self-host` into `migrate`. Feedback triaged **surface** (cc-prompt) vs. **model** (ODD-0013/0020 amendment or ADR). | A6·s04 (the self-hosted corpus UAT runs on) | **CLOSED 2026-07-27** — `arc-release-hardening/closing-report.md`. All chunks done (C-1…C-6, C-8; **C-7 retired into C-5**): theming, type taxonomy, `list` overhaul, the ODD-0023 three-tier command surface + `validate`/`check` split, the fold + real dates + names + vision **+ the SH-6 store cutover**, normalized status, and `validate` hardening. **RH-6 compose reproduced**, **RH-7 reflexive reproduced** (`validate` exit 0 at 60 nodes), RH-8 dispositioned. |
| **LLM command surface** | Make the CLI sufficient for an LLM to regain situational awareness, decide what's next, and verify what it did **without reading planning prose** — the stated DoD (§1). Slices: read-back status vector, `info`, explain-readiness + ordered `next`, `search` + flat rollup, `history`/`diff`, `export`, `viz`. | RH, A1–A3 | **next** (shaped, not started) — `arc-llm-command-surface/arc-plan.md`; reconciled against `odm-command-inventory.md` |
| **Store Home & `init`** | Dedicated store home — orphan `odm` branch in a `.worktrees/odm` git worktree — + the `init` command (bootstrap / attach / ff-sync), the two-config split (`odm.toml` locator + in-store `config.toml`), and the store-resolution rework. Decouples the planning DB from the code branches (ODD-0022). | A1–A3; ODD-0022 | **CLOSED 2026-07-27** — `arc-store-home/closing-report.md`; **SH-1…SH-5 done + SH-6 (dogfood cutover) delivered by RH C-5** — odm now dogfoods its own store (60 nodes on the orphan `odm` branch); an independent fresh-context arc-gate returned PASS-WITH-NOTES |
| **Migration Fidelity** | Faithful, verifiable, repeatable migration: 1:1 verbatim bodies (**hard body-hash gate**), a `provenance` sub-map on every node, frontmatter-fidelity via schema-mapping, and a **doc-coverage** check (no file left behind). Repairs odm's own self-hosted corpus from skeleton (44 stub bodies, 6/11 arcs, ~211 uncovered docs, 0 provenance) to **100%**; general per project. Synthesis split out as a separate supersede step. | A6·s04; **ODD-0024** (G-1 closed → minting unfrozen) | **✅ CLOSED 2026-08-02 — MF-9 fidelity ACHIEVED.** All slices s04–s16 CDC-verified PASS (s16 2026-08-02); the **freeze fired** (project-vision pair collapsed, **0 drift**, `check` exit 0) and the **P-12 `orient` demo runs clean** on the live corpus (store committed `41ace1f`). Composition CDC-confirmed (`arc-migration-fidelity/closing-report.md`); **MF-4 → L-8** (CDC-ARC-1, design-corpus migrate + standing frontmatter-fidelity check). Store-side transition landed 2026-08-02 (arc node `58837400` → `complete` gate + decomposition affirmed, 17 children; `odm@6225d1f`, CDC-reproduced). **Closed on both doc and store; CI green on `release/1.0.x` (`e4e0a95`, 2026-08-02).** |
| **Store Lifecycle** | Complete the store's git lifecycle as native, odm-aware commands so the operator never drops to raw git: **`store commit`** (persist the worktree's node changes on the orphan branch; auto-summary + `-m`; idempotent; `--dry-run`/`--json`), **`store status`** (pending node delta + ahead/behind), **`store sync`** (ff-only push/pull, divergence stops). Completes the ODD-0022 store-home promise. | arc-store-home; ODD-0022 | **shaped 2026-08-01; scoped run 2026-08-02** — `arc-store-lifecycle/arc-plan.md`. **Do s01 (`store commit`) only, then ⏸ pause** (s02 status / s03 sync deferred); then resume + close Migration Fidelity. Surfaced by the MF freeze (no verb to commit the store). |

**Sequencing (updated 2026-07-27):** Release Hardening **closed** → LLM command surface
(**next**, and materially lighter — RH C-8 delivered its blocking slice 01, L-1 status
read-back; see that arc's v1.2) → **A6 resumes at slice05** (PM-skill) + slice06 (retire prose),
so the skill and prose-retirement target the *settled* command surface. Both inserted
arcs consume A6 slice04's self-hosted corpus; A6 is **paused after slice04**, not
abandoned (§3).

## 3. Current status (2026-07-27)

- **A1 — COMPLETE.** Merged to `main`, CI-green.
- **A2 — COMPLETE.** Merged to `main`, CI-green.
- **A3 — COMPLETE.** All four slices (cleanup, rollup, orient, `--json`) CDC-verified
  and merged to `main`; pushed for CI. The arc-close recomposition check passed with no
  silent drops (`arc03-rollup-and-orient/arc-close.md`). **MVP (A1–A3) is done.**
- **A4 — Index & cache: ✅ CLOSED (composed; CI-green 2026-06-30).** All 8 slices
  CDC-verified; the arc-close composition check passed with an independent arc-gate review
  (`arc04-index-cache/closing-report.md`, PASS-WITH-NOTES); **CI green across slices 01–08
  flipped every row to `reproduced`** (P-4 `done`). The index/cache capability lands:
  `list`/`orient`/graph-build/`check`/`rollup` are index-backed and match baseline, change
  detection is racy-git-correct, the index self-heals, and the 100k benchmark promoted
  ODD-0014's `[P]` perf claims to `[E]`. Forward-carried: the **adapter-fidelity invariant**
  is now a hard gate in `arc05`'s arc-plan (v1.2).
- **A5 — Reconciliation: ✅ CLOSED (composed; CI-green 2026-07-06, `release/1.0.x`).** All 8
  slices CDC-verified; arc-close composition check passed with an independent arc-gate review
  (`arc05-reconciliation/closing-report.md`, PASS-WITH-NOTES). The marquee state-drift killer
  lands: `desired_facts` + shell/file probes; drift in `reconcile`/`rollup`/`orient`; `affects`
  → stale-doc check; deferred + re-entry. Per the **v1.8 redirection (ODD-0019)** freshness is
  **incremental on every command** (rides the A4 stat-cache; bare `odm` runs zero volatile
  probes; honest "last checked Xm ago"). A5's zero-index-change streak held across all 8
  slices. Carried to A6: two-reads-per-command optimization; the `CLAUDE.md` oxur-cli
  doc-drift fix. **Only A6 remains for the v1.0.0 DoD.**
- **A6 — Migrate, self-host & PM-skill: ▸ IN PROGRESS, ⏸ PAUSED after slice04.** Slices
  01–04 CDC-verified. **★ The loop closed: odm SELF-HOSTS** (slice04) — `odm self-host`
  imported the `design-v1.0.0` plan-set into **45 work nodes** (1 project + 6 arcs + 38
  slices) under `nodes/`, `odm check` is green on 58 nodes (13 `odd` + 45 work), and
  `rollup`/`orient` reproduce the hand-maintained truth (P-12 reproducible-at-arc-close;
  cargo/executable rows pending CI). slice03 also landed schema-versioning (ODD-0020,
  per-type `schema:<type>/vN.N`). **Paused after slice04** because hands-on UAT of the
  self-hosted tool surfaced CLI/naming/type/output feedback too large and too model-level
  for a slice — extracted into the **Release Hardening** arc (§2a). A6 **resumes at
  slice05** (PM-skill) + slice06 (retire prose) once Release Hardening and the
  LLM-command-surface arc wrap, so those target the settled surface. *(Was, at v1.0–1.6:
  "PLANNED, not started.")*
- **Release Hardening (UAT) — ✅ CLOSED 2026-07-27.** Named, number-deferred (§2a).
  **The surface is settled.** 21 findings worked down to 18 done + 3 carried with reasons,
  across seven chunks: C-1 adopted the re-extracted **`oxur-term`** styling crate (Route B,
  ADR) — `oxur-cli` shed; C-2 landed the type taxonomy (`odd`→`design` + `research`); C-3
  overhauled `list` (date/type/status/tree, de-numbered, retired excluded by default); C-4
  landed **ODD-0023's three-tier surface** (`odm node` / `odm store` / workflow verbs) with a
  hard cut and the **`validate`/`check` split**; C-5 folded `self-host` into `migrate`,
  recovered the plan's **real creation dates**, normalized names, wrote the project's vision
  — **and moved odm onto its own store branch** (SH-6); C-8 normalized STATUS so types
  compare, **and delivered the LLM arc's blocking L-1** on the way; C-6 hardened `validate`
  (G-3 undecomposed-parent, L-3b no-vision). C-7 retired into C-5.
  **Compose:** RH-6 reproduced (whole surface in one pass on the self-hosted store), RH-7
  reproduced (`validate` exit 0 at 60 nodes, every ULID preserved through the cutover).
  **Carried:** `store status`, shared-`context.json` two-level model, **L-8b (reconcile 4
  ODDs — a pre-ship gate)**, F-16/F-17/F-21, and **G-1 still gates minting any new node**.
  Companion surface work landed alongside arc-store-home: **ODD-0023**
  (command reorg → top-level verbs / `odm node` / `odm store`) drafted and the
  `odm-command-inventory.md` rewritten to it; the **UAT coverage audit** routed the four
  previously un-routed items (L-3/L-6/L-8/G-7). `arc-release-hardening/arc-plan.md`.
- **LLM command surface — ◆ NEXT (shaped, not started).** Named, number-deferred (§2a).
  Shaped 2026-07-25 from the pass-2 LLM UAT; 7 slices closing the LLM situational-awareness
  gaps, reconciled against `odm-command-inventory.md` (the command-surface authority). Runs
  after Release Hardening, before A6 resumes.
- **Store Home & `init` — ✅ CLOSED 2026-07-27.** Named, number-deferred (§2a; **ODD-0022**,
  Accepted). Four slices done (SH-1…SH-4), SH-5 composed, and **SH-6 (the dogfood cutover)
  delivered by RH C-5** — `odm store init`/`rename` stand up the orphan-branch store home
  (bootstrap / attach / ff-sync), and odm now lives in its own store (60 nodes on the `odm`
  branch; `odm.toml` a locator). CDC-reproduced on git 2.43 throughout; an independent
  fresh-context arc-gate returned **PASS-WITH-NOTES**. The arc found four defects across its
  slices + the cutover, all fixed, sharing one through-line (a path unexercised in its authoring
  environment, or success and failure looking identical). `arc-store-home/closing-report.md`.
  *(Durable CI-`reproduced` rides the push.)*

**Next-arc sequencing — RESOLVED** *(was "open — operator's call" at v1.0–1.6, when A4/A5/A6
were free to order and a case was floated for taking A6 next to realize self-hosting
early).* The order taken was **A4 → A5 → A6**: A4 and A5 are closed, A6 is in progress and
self-hosts. The remaining v1.0.0 path is **Release Hardening (✅ closed 2026-07-27) → arc-store-home (✅ closed;
SH-6 delivered by RH C-5) → LLM command surface (next — materially lighter, since RH C-8 already
delivered its blocking slice L-1) → resume A6 slice05–06 → A6 arc-close → v1.0.0 DoD** (§5). ~~Two
standing pre-ship gates: **G-1** (the id-scheme ODD — gates minting any new node) and **L-8b**~~ **G-1 is CLOSED (2026-07-27 — ODD-0024: ULID retained, register-style rejected, minting freeze lifted).** One standing pre-ship gate remains: **L-8b**
(reconcile the four state-drifted ODDs).

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
>
> **Scope note (UAT hardening arcs, v1.8).** Two **release-blocking** arcs — **Release
> Hardening** and **LLM command surface** (§2a) — were inserted within the A6 window after
> UAT of the self-hosted tool. Unlike A7/A8 these *are* v1.0.0 work. Whether they earn
> their **own** project-ledger P-rows, or instead fold into the existing DoD-compose rows
> — especially **P-7** ("full situational awareness from `odm orient` alone", which the
> LLM-command-surface arc exists to actually satisfy) and **P-12** (self-host) — is an
> **open operator call**, flagged here rather than silently dropped, to settle before the
> v1.0.0 project close. **No new P-rows added pending that call.**

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| P-1 | A1 (substrate & node CRUD) closed + composed | ptr: arc01 close | correctness | project-plan | done | merged to `main`, CI-green | **Disclosed gap:** A1 predates the v2.0 arc-close discipline — no formal arc `closing-report.md`. Composed-in-fact (the MVP runs on it). |
| P-2 | A2 (graph, gates & derived order) closed + composed | ptr: arc02 close | correctness | project-plan | done | merged to `main`, CI-green | Same disclosed gap as P-1 (no formal arc closing-report). |
| P-3 | A3 (rollup & orient) closed + composed | ptr: `arc03-rollup-and-orient/arc-close.md` | correctness | project-plan | done | arc-close composition check passed, no silent drops; merged | A3's bubble-up forced no project-plan change (recorded). (File is `arc-close.md`; canonical name is `closing-report.md`.) |
| P-4 | A4 (index & cache) closed + composed | ptr: `arc04-index-cache/closing-report.md` | correctness | project-plan | done | **reproduced/reconciled:** arc-close done (`arc04-index-cache/closing-report.md`) — 8/8 slices CDC-verified, class-(b) composition rows A-9…A-13 reproduced at arc scale (structural CDC + executable **CI green 2026-06-30**), independent arc-gate review **PASS-WITH-NOTES**; ODD-0014 `[P]`→`[E]`. | Delivered set is *broader* than the roadmap line (also `check`+`rollup`) — over-delivery, not drift. |
| P-5 | A5 (reconciliation) closed + composed | ptr: `arc05-reconciliation/closing-report.md` | correctness | project-plan | done | **reproduced/reconciled:** arc-close done — 8/8 slices CDC-verified, class-(b) compose rows reproduced at arc scale, independent arc-gate review **PASS-WITH-NOTES**; CI green on `release/1.0.x`. Freshness model (ODD-0019) realized; zero index change across the arc. | Delivered incremental-freshness beyond the roadmap line (ODD-0019) — over-delivery, not drift. |
| P-6 | A6 (migrate, self-host & PM-skill) closed + composed | ptr: arc06 closing-report | correctness | project-plan | open | | attested-on-close. |
| P-7 | **Compose (DoD):** a fresh session reaches full situational awareness from `odm orient` alone | project-scale demo: fresh session, `odm orient` only | serious | project-plan / 0015 §2 | open | | MVP-demonstrable now; reproduce at project scale on close. The headline DoD. **Surface settled 2026-07-27** by the Release Hardening close, so this demo now has a stable target: `orient` renders a vision and a resolved focus on odm's own store (both were blank before RH C-5), and the command surface will not churn again before ship. |
| P-8 | **Compose (DoD):** every dependency is a queryable edge; `next`/`blocked`/`path` answer correctly | project-scale demo over a real corpus | serious | project-plan / 0015 §2 | open | | MVP-demonstrable now. |
| P-9 | **Compose (DoD):** `check` catches cycles-without-tears, dangling refs, out-of-order work, broken recomposition | project-scale demo: seed each violation, observe the finding | serious | project-plan / 0015 §2 | open | | MVP-demonstrable now. |
| P-10 | **Compose (DoD):** status is multi-gate with evidence levels | project-scale demo | serious | project-plan / 0015 §2 | open | | MVP-demonstrable now. |
| P-11 | **Compose (DoD):** desired-vs-actual drift is detected and reported (the prod-DB-503 class) | project-scale demo: declare a fact, diverge reality, observe drift | serious | project-plan / 0001-C2 | done | **reproduced (A5, CI green):** `odm reconcile`/`rollup`/`orient` detect + report drift end-to-end (arc05 A-8/A-10 reproduced at arc scale); fresh on every command (ODD-0019). The C2 marquee case is cashed. | Landed with A5 (reconciliation). |
| P-12 | **Compose (DoD):** odm self-hosts — manages its own plan as nodes; these design docs are queryable via `odm orient` | project-scale demo: migrate + orient on odm's own corpus | serious | project-plan / 0013 §9 | open | | **Self-host landed (attested) at A6 slice04** — `odm self-host` → 45 work nodes, `check` green on 58, `rollup`/`orient` reproduce the state; cargo/exec rows pending CI. Reproduce at A6 arc-close for `done`. **Migration Fidelity arc-close (2026-07-31):** the corpus P-12 demonstrates against is now *faithful* (composition CDC-confirmed) rather than a skeleton; the live `odm orient` fresh-session demo + the final reconcile-and-freeze are the arc's §7 runtime acts — **P-12 → `done` when they run green.** **2026-08-02 — the MF §7 acts ran green:** the reconcile-and-freeze fired (0 drift, `check` exit 0) and the fresh-session `odm orient` demo auto-orients to the faithful `#1000` vision, reproduced on `.worktrees/odm` (s16 retired-filter CDC-verified PASS; store committed `41ace1f`). **The orient/self-host demo side of P-12 is demonstrated against a faithful corpus.** **2026-08-02 — CI green on `release/1.0.x` (`e4e0a95`, operator-attested):** the cargo/exec rows for everything on the branch are now green, so the orient/self-host demo side of P-12 is CI-reproduced; P-12's remaining gate for `done` is the **A6 arc-close** formality (A6 paused, resumes at slice05). |
| P-13 | Arc bubble-up findings dispositioned | ptr: project-plan change-log (Version History) | correctness | bubble-up | open | | Accrues per arc close. A3: no project-plan change (recorded). A4: no roadmap re-scope; adapter-fidelity invariant → arc05 v1.2 (v1.4). A5: no roadmap re-scope; three findings forward-carried to A6 (two-reads optimization; `CLAUDE.md` oxur-cli doc-drift; volatile-re-entry behavior change) — recorded, v1.6. A6 (in progress): UAT of the self-hosted tool surfaced release-blocking CLI/type/naming/output work → **two arcs inserted** (Release Hardening, LLM command surface; §2a), A6 paused after slice04 — expansion (A1–A6 DoD unchanged), recorded v1.7/v1.8. **v1.9:** arc-store-home added from ODD-0022 (expands the v1.0.0 DoD) — recorded. **Release Hardening closed 2026-07-27** (`arc-release-hardening/closing-report.md`): 21 findings → 18 done, 3 carried (F-16 upstream `oxur-term`, F-17 a decision not a defect, F-21 routed to the LLM arc); routed L/G rows closed out with **L-8b (reconcile 4 ODDs) still a pre-ship gate** and **G-1 still gating any new node**. The arc also folded in **arc-store-home's SH-6 dogfood cutover** (C-5), so odm now runs on its own store branch. Ten defects were surfaced and fixed by the arc's own work — the reusable lesson recorded there is **check the brief's facts before implementing them**: four of the ten came from reading the corpus or running the binary before writing code. **Migration Fidelity closed 2026-07-31** (`arc-migration-fidelity/closing-report.md`): 13 slices (s04–s13 CDC-verified PASS); composition CDC-confirmed, 8/9 arc rows done-reproduced. Two findings forward-carried — **CDC-ARC-1** (14 RH-era design/research nodes dropped source `version`, traced to `odm@b45b122`; → **L-8** design-corpus migrate + a standing frontmatter-fidelity check) and **CDC-F1** (2 living-plan-tail body drifts → the final reconcile-and-freeze, §7). **L-8b subsumed + cleared; L-8 remains.** Recorded v1.14. |
| P-14 | **arc-store-home (store home & `init`) closed + composed** — odm's store lives on the orphan `odm` branch; `init` bootstraps/attaches/ff-syncs it | ptr: `arc-store-home/closing-report.md` | serious | project-plan / ODD-0022 | **done** — CI-green (`e8e69de`) | `arc-store-home/closing-report.md` (2026-07-26): slices 01–04 closed — SH-1/SH-2/SH-4 attested, **SH-3 CDC-reproduced** (git 2.43), **SH-5 composition reproduced** (bootstrap → attach → ff-sync → rename, `check` green). 57 test binaries, 0 failed (58 after C-5); clippy `-D warnings`, fmt clean; CI runs the store suite on **both git arms**. | **SH-6 closed 2026-07-26 by the RH C-5 cutover** — odm's 60 nodes now live on the orphan `odm` branch at `.worktrees/odm`, `check` green at 60 **in the home**, the working branch no longer carries `nodes/`, and `odm.toml` is locator-only with the operational half in the store's `config.toml`. Every ULID preserved: a relocation and in-place re-stamp, not a re-derivation (which would have minted ids, broken every edge and tripped the G-1 freeze). See `arc-release-hardening/c5-closing-report.md`. **The dogfood immediately found three defects four green slices had not** — `orient` resolving the CLI context against the invocation root while `use` wrote the store root (indistinguishable until odm's own store moved), `init` scaffolding no store `.gitignore`, and a first fix that would have withheld the current focus from every clone. All fixed in-chunk. The arc's own lesson, recorded there: **an opt-in feature nobody has opted into is not verified.** |
| P-15 | **Release Hardening (RH) closed + composed** — UAT-driven CLI/type/naming/output hardening; the ODD-0023 three-tier surface | ptr: `arc-release-hardening/closing-report.md` | serious | project-plan (§2a) | **done** — CI-green (`e8e69de`) | CLOSED 2026-07-27; C-1…C-6, C-8 (C-7 folded into C-5): theming, type taxonomy, `list` overhaul, the three-tier surface + `validate`/`check` split, cutover, normalized status, check-hardening. RH-6 compose + RH-7 reflexive reproduced; 21 findings → 18 done, 3 carried (F-16 upstream `oxur-term`, F-17 a decision, F-21 → LLM arc). **Resolves the v1.8 open call: RH earns its own P-row.** |
| P-16 | **Migration Fidelity (MF) closed + composed** — faithful/verifiable/repeatable migration; odm's own corpus repaired to 100% | ptr: `arc-migration-fidelity/closing-report.md` | serious | project-plan (§2a) | **done** — CI-green (`e8e69de`) | ✅ CLOSED 2026-08-02; s04–s16 CDC-verified PASS; the freeze fired (project-vision pair collapsed, **0 drift**, `check` exit 0); the P-12 `orient` self-host demo runs clean on the live corpus; store-side close landed (arc `58837400` → `complete` + decomposition affirmed). MF-4 → L-8 (CDC-ARC-1) carried to the LLM arc. |

Closes in a project-level `closing-report.md` with the per-row walk and the **project
gate** (go / adjust / kill against the DoD, reviewed by the operator + an independent
context). A failed DoD row spawns a **remediation arc** or a roadmap re-scope, not an
unbounded grind.

## Version History

### v1.18 — 2026-08-02 — closed arcs recorded **done (100%)**; CI-green at `e8e69de`

The latest `release/1.0.x` push is **CI-green at `e8e69de`** (operator-attested), covering the new
**Store-as-Source** arc's slices **05** (decomposition-bookkeeping consistency) and **06** (hands-off
auto-extend of authored decompositions) and **Store-Lifecycle s04** (`store commit` now leaves the git
index clean) — all three **CDC-verified PASS**. Per the operator call, the **fully-closed arcs are now
recorded done + composed with their own ledger rows:** SH bumped `attested → done` (**P-14**), and
**P-15 (Release Hardening)** + **P-16 (Migration Fidelity)** added as `done` — resolving the v1.8 open
question in favour of own P-rows for the named closed arcs (alongside A1–A5 = P-1…P-5, already done). The
dashboard recolors `closed` → green (fully-done, CI-green) to match `complete`.

**Deliberately *not* marked done (accurate, not silent drops):** **A6** (paused — PM-skill s05 /
retire-prose s06 remain; its migrate/self-host work landed but the arc is not closed), **Store-Lifecycle**
(s01 + s04 done; s02 `status` / s03 `sync` deferred), **LLM command surface** (reconciled to v1.4, not
started), **Store-as-Source** (s05/s06 done; s01 ODD-0026 + s02/s03/s04 remain), **A7/A8** (scoped). The
**DoD compose rows P-7…P-10, P-12** also stay open — they reproduce at **project** close, not arc close;
A6's arc-close is still P-12's remaining gate. Surfaced by: operator (CI green; close the closed arcs).

### v1.17 — 2026-08-02 — CI green on `release/1.0.x` (`e4e0a95`)

The latest push to `release/1.0.x` is **CI-green at `e4e0a95`** (operator-attested), covering the Migration Fidelity s16 close, the `store commit` verb (SL-1), and the refreshed status dashboard. The `attested → CI` / cargo-exec evidence rows those carried are now **CI-reproduced**. No arc-status change — this records evidence strength (asserted/attested → CI-reproduced) across the recently-closed work. The older A6 `→ done when reproduces (CI green)` ledger rows stay as-is: they flip at A6's own arc-close, not here.

### v1.16 — 2026-08-02 — Migration Fidelity arc **fully closed** (freeze fired, P-12 demonstrated, s16 CDC-verified PASS)

**Migration Fidelity → CLOSED** in §2a — completes the partial close recorded at v1.14 ("composition
CDC-confirmed; freeze + P-12 pending"). The two runtime acts v1.14 named have both fired: the arc-close
**reconcile-and-freeze** ran (the project-vision pair collapsed to one faithful 1:1 node, **0 drift**,
`odm check` exit 0, 0 errors), and the **P-12 `orient` self-host demo** runs clean on the live corpus — a
fresh session auto-orients to the single active project `#1000` and renders its vision. A last leak surfaced
by that demo (`orient` listed the retired `#1001`) was closed by a 14th-hour slice, **s16** (`orient`
retired-filter), now **CDC-verified PASS 2026-08-02**. **All slices s04–s16 CDC-verified PASS.** The `odm`
store is committed at `41ace1f` (via the new `store commit` verb from the Store Lifecycle detour). Forward
carry unchanged: **MF-4 → L-8** (CDC-ARC-1: 14 RH-era design nodes' dropped source `version` → design-corpus
migrate + a standing frontmatter-fidelity check). One store-side projection remains for the operator — the
arc node `58837400` status→`complete` + `odm node decomposed 58837400` (17 children), then `store commit`;
the doc plan-of-record does not wait on it. **P-12 moves to demonstrated** (see the P-13 / DoD rows).

### v1.15 — 2026-08-01 — Store Lifecycle arc shaped (store commit/status/sync)

A named, number-deferred, **v1.0.x** arc — **Store Lifecycle** — added to §2a
(`arc-store-lifecycle/arc-plan.md`). **Which surfaced it:** the Migration Fidelity arc-close freeze
(`migrate --all`) mutates the store worktree but there is **no odm verb to persist it** — the operator must
drop to raw `git -C .worktrees/odm`, which breaks the ODD-0022 store-home promise (*odm owns the orphan
branch; you never touch it by hand*). Three slices complete the lifecycle: `store commit` (the near-term
need), `store status`, `store sync` — each idempotent, `--dry-run`/`--json`-capable, honoring the
orphan-branch / never-rewrite-history / divergence-stops discipline. Depends on arc-store-home; mostly CLI
wiring over `odm-store`'s existing git plumbing.

### v1.14 — 2026-07-31 — Migration Fidelity arc closed (composition CDC-confirmed; freeze + P-12 pending)

**Migration Fidelity → CLOSED** in §2a (was "shaped, not started"). Thirteen slices delivered a faithful,
verifiable, repeatable migration and repaired odm's own plan corpus from skeleton to faithful; s04–s13 are
CDC-verified PASS. The arc-close composition check (CDC, independent; `arc-migration-fidelity/closing-report.md`)
reproduced 8/9 arc-ledger rows done against the shipped store `odm@e06fffe`. **MF-4 (frontmatter fidelity):**
capability-complete and proven on this arc's own design-node migrations; **CDC-ARC-1** finds 14 RH-era
design/research nodes dropped their source `version` (git-traced to the pre-ODD-0025 `b45b122` store cutover,
bodies faithful, invisible to the coverage check) — **routed to L-8** (design-corpus migrate) plus a standing
frontmatter-fidelity `check` rule as hardening. **L-8b is subsumed and cleared** by this arc (the last
non-L-8 standing pre-ship gate); **L-8 (design-corpus migrate) remains**, now with CDC-ARC-1 attached.
**Two runtime acts finish the arc** (Mac binary; closing-report §7): the final reconcile-and-freeze (closes
CDC-F1's two living-plan-tail body drifts) and the **P-12 self-host acceptance demonstration** — after which
P-12 is *demonstrated against a faithful corpus*. `make check` green (operator-confirmed). Bubble-up: P-6/P-12
advanced (self-host faithful-corpus-ready), P-13 carries the two findings.

### v1.13 — 2026-07-27 — Migration Fidelity arc shaped (corpus reconciliation)

A **fourth** named, number-deferred, **release-blocking** arc — **Migration Fidelity** — is added to §2a (`arc-migration-fidelity/arc-plan.md`), shaped from the corpus-reconciliation audit (2026-07-27). It makes migration faithful and verifiable — 1:1 verbatim bodies + a hard body-hash gate, a `provenance` sub-map, frontmatter-fidelity schema-mapping, and a doc-coverage check — and repairs odm's own self-hosted corpus from skeleton (44 stub bodies, 6 of 11 arcs, ~211 uncovered docs, 0 provenance) to 100%. **Which surfaced it:** the reconciliation audit + the migration/provenance design thread. It **subsumes L-8b** and is on the **P-12** DoD path; unblocked by **ODD-0024** (G-1 closed).

### v1.12 — 2026-07-27 — G-1 closed (id scheme decided); minting unfrozen

**G-1 is closed** by **ODD-0024** (Accepted): odm retains **ULID** identity; the register-style
`D-YYMM-XXXX` proposal is **rejected**; `number` is unchanged. The standing "do not mint before
G-1" freeze is **lifted**. **Which child surfaced it:** the `arc-migration-fidelity` planning
thread — the freeze blocked the arc that repairs the self-hosted corpus, forcing the
long-deferred decision. One standing pre-ship gate remains: **L-8b** (reconcile the four drifted
ODDs).

### v1.11 — 2026-07-27 — Release Hardening closed; the surface is settled

**Release Hardening (UAT) is closed** (`arc-release-hardening/closing-report.md`). Seven
chunks — C-1…C-6 and C-8, with C-7 retired into C-5 — took the UAT punch-list from 21 open
findings to 18 done and 3 carried with stated reasons. **RH-6 (compose) reproduced** on the
self-hosted store; **RH-7 (reflexive) reproduced** with `validate` exit 0 at 60 nodes.

**What it unblocks.** The command surface will not churn again before ship, so the two arcs
behind it now target a settled target:

1. **LLM command surface — next, and materially lighter.** RH C-8 delivered its *blocking*
   slice 01 (L-1, status read-back) as a side effect of checking a premise: the chunk was
   briefed that `show`/`--json` already exposed the gate vector, and neither exposed gates at
   all. Fixing that is L-1. The arc's v1.2 records the remainder.
2. **A6 resumes at slice05** (PM-skill) + slice06 (retire prose), against the settled surface.

**Two things this arc absorbed beyond its brief.** It folded in **arc-store-home's SH-6
dogfood cutover** (C-5), because C-5 already rewrote the corpus and doing both once beat doing
it twice — so odm now manages its own plan on an orphan `odm` branch, with every ULID
preserved. And it took **ODD-0023 to Accepted** (v1.1) with three recorded reversals, one of
which — `validate` as the pure verb and `check` as the composite — was the operator's
correction to the draft's own recommendation.

**Still gating.** **G-1 (the ID scheme) is unchanged and still blocks minting any new node**;
C-5 was built to preserve every id precisely so it would not trip that freeze. **L-8b
(reconcile the four ODDs) remains a pre-ship gate.** Neither is release-blocked by this close.

**Honest on evidence.** Every cargo row across the arc is **attested-by-CC on a real
toolchain, not CI-reproduced** — `release/1.0.x` is 38 commits ahead of an SSH `origin` and
unpushed. The durable `reproduced` rides that push.

Surfaced by: the Release Hardening arc close (RH-6/RH-7 compose + RH-8 bubble-up).

### v1.10 — 2026-07-26
**arc-store-home slices 01–03 landed (SH-1/2/3); ODD-0023 + inventory rewrite; UAT tail routed.**
The Store Home arc moved from *planned* to **active**: `odm store init` now bootstraps, attaches, and
ff-syncs the dedicated orphan-`odm`-branch store home, each slice CDC-**reproduced** in an independent
git-2.43 container (§2a status + a new §3 bullet). **Slice 02 caught a real defect:** the modern
`git worktree add --orphan` argv was wrong (broken on git ≥ 2.42) and only the pre-2.42 fallback had
ever run on the implementer's machine — CDC reproduced the failure on git 2.43, the one-line `-b` fix
was verified, and a **CI two-arm git-version matrix** now keeps both `--orphan` and the fallback under
test (the load-bearing guard that had been missing — bug 2 in the same slice had shown the two arms can
diverge). Slice 03's fast-forward-only safety invariant (never rebase/merge the shared branch) is
verified on real state (diverged → HEAD unmoved, own work intact, other's absent). **Companion surface
work:** **ODD-0023** (command-surface reorg → top-level workflow verbs / `odm node` / `odm store`)
drafted, and `odm-command-inventory.md` rewritten to its three-tier shape; the RH **UAT coverage
audit** routed the four previously un-routed items (L-3 → C-5 data + C-6 check rule; L-6 → a
release-engineering follow-up; L-8 → pre-release housekeeping + a design-corpus migrate; **G-7** →
arc-store-home, *largely resolved by design* since the orphan-branch worktree isolates odm's commits).
**Bubble-up disposition (P-13): expansion, not re-scope** — the A1–A6 DoD is unchanged; **P-14**
(arc-store-home) stays open, closing at the arc's SH-5/SH-6 compose. **Next:** slice 04 (`rename`) →
RH C-5 re-self-hosts into the home. **Push note:** these commits sit on `sh-slice0x` branches locally —
origin is SSH, unreachable from the cloud session, so they await a push from the operator's terminal.
Surfaced by: arc-store-home slices 01–03 and their CDC verifications.

### v1.9 — 2026-07-26
**arc-store-home added to the roadmap (from ODD-0022).** A third named, number-deferred arc — **Store Home & `init`** — joins §2a, realizing **ODD-0022 (Accepted)**: a dedicated store home (orphan `odm` branch in a `.worktrees/odm` worktree), the two-config split (`odm.toml` locator + in-store `config.toml`), and the `init` command (bootstrap / attach / ff-sync). **v1.0.x — expands the v1.0.0 DoD** (operator call). Added the §2a row + §3 sequencing (runs **before RH C-5**, which re-self-hosts *into* the new home — the dogfood cutover, SH-6) + **P-14**. Not UAT-driven (unlike RH/LLM) — ODD-0022-driven. Surfaced by: ODD-0022 acceptance + the slice-breakdown session.

### v1.8 — 2026-07-25
**LLM command-surface arc shaped; command inventory landed; §2a/§3/§5 brought current.**
Added **§2a** (the two named, number-deferred UAT hardening arcs) and rewrote §3's stale
A6 line — *was "PLANNED, not started"* — to reflect A6 **in progress / self-hosting /
paused after slice04**, plus the **active** Release Hardening arc and the **next**
LLM-command-surface arc; the §3 "next-arc sequencing" open question is marked RESOLVED. The
**LLM command surface** arc (`arc-llm-command-surface/arc-plan.md`, 7 slices) was shaped
from the pass-2 LLM UAT and reconciled against the new **`odm-command-inventory.md`** — the
command-surface authority, reconstructed by the operator from the four kickoff transcripts
after the spec was found to live nowhere in-repo. §5 gains a **scope note**: whether the two
UAT arcs earn their own P-rows or fold into P-7/P-12 is an open operator call (no new P-rows
added). ⚠ **G-1**: the operator intends to switch the node ID scheme (register-style
`D-YYMM-XXXX` vs ULID) — recorded in `arc-release-hardening/workflow-gap-coverage-review.md`;
the ODD + decision must land **before any new node is minted** (identity cannot change after
ship). **Surfaced by:** the pass-2 LLM UAT + the operator's command-surface reconstruction.
Also re-synced the `project-status.html` dashboard (LLM-command-surface card added after RH).

### v1.7 — 2026-07-07
**A6 in progress — odm self-hosts (slice04); A6 paused; Release Hardening arc created.** A6
moved from planned to in-progress: slices 01–04 CDC-verified. **★ slice04 closed the loop —
`odm self-host`** imported the `design-v1.0.0` plan-set into 45 work nodes, `odm check` green
on 58 nodes, `rollup`/`orient` reproduce the hand-maintained truth (**P-12** now
reproducible-at-arc-close; cargo/exec rows pending CI); **slice03** landed schema-versioning
(ODD-0020). Hands-on UAT of the self-hosted tool then produced CLI/naming/type/output feedback
too large and too model-level for a slice, so — per the mid-arc-pause precedent — it was
**extracted into the Release Hardening arc** (batch 1: 14 findings → chunks C-1…C-5) and **A6
was paused after slice04**, to resume at slice05 once the surface settles. **Bubble-up
disposition (P-13):** expansion, not re-scope — arcs inserted within the A6 window, the A1–A6
DoD unchanged. **Surfaced by:** A6 slice04 self-host + Duncan's hands-on UAT.

### v1.6 — 2026-07-06
**A5 (Reconciliation) closed + composed — CI green on `release/1.0.x`.** Arc 05 reached its
arc-close: 8/8 slices CDC-verified, class-(b) compose rows reproduced at arc scale, an
**independent arc-gate review** (PASS-WITH-NOTES) — the reconciliation capability (the marquee
state-drift killer, ODD-0001 C2) lands, and the **v1.8 freshness redirection (ODD-0019)**
makes it incremental on every command. Flipped **P-5** (A5 closed+composed) and **P-11**
(the C2 drift-DoD compose row) → `done`. **Bubble-up disposition (P-13): no roadmap
re-scope** — A5 delivered its capability (plus incremental freshness beyond the roadmap
line). Three findings forward-carried to A6, not dropped: the two-reads-per-bare-command
optimization, the `CLAUDE.md` oxur-cli doc-drift, and the volatile-re-entry behavior change.
**MVP (A1–A3) + A4 + A5 are in; only A6 (migrate + self-host + PM-skill) remains for the
v1.0.0 DoD.** Git: work is on `release/1.0.x` (main reset onto `release/0.3.x`, the pre-rebuild
import); future branches cut from `release/1.0.x`. Surfaced by: the A5 arc-close.

### v1.5 — 2026-06-30
**CI green — A4 fully closed (P-4 `done`).** Duncan confirmed workspace CI is green across
Arc 04's slices 01–08, discharging the "CI-pending" holding state from v1.4: the arc's
attested cargo/number rows and the executable half of the composition rows are now
`reproduced` (arc-plan v1.15, closing-report CI banner). P-4 (A4 closed + composed) flips
open → `done`. No DoD re-scope. Surfaced by: operator CI-green confirmation.

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
