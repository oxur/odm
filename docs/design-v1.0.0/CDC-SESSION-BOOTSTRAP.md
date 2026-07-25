# CDC Session Bootstrap — pick up where we left off

> **The canonical bootstrap for the CDC/CC collaboration on `odm` — living resume + genesis.**
> Read this first in a new session to reach full situational awareness without re-reading the
> whole history. Updated at session close. **Resume last updated: 2026-07-07** (end of the
> self-host + UAT-pivot session). **2026-07-25:** the genesis doc (formerly
> `workbench/odm-session-bootstrap.md`, 2026-06-19 — why the project exists) was **merged in
> as §8** and this file made the single canonical bootstrap; §§1–7 (the "what's true now / do
> this next" resume) are unchanged from 2026-07-07.

---

## 1. The frame (how we work)

- **odm** = a markdown/git-native, dependency-ordered planning + documentation substrate,
  rebuilt at v1.0.0 as a multi-crate Rust workspace. It mechanically actualizes the
  collaboration-framework. Repo: `github.com/oxur/odm` (pkg `oxur-odm`, binary `odm`).
- **Roles.** I am **CDC** (the independent planner/verifier): I draw each slice's open doc-set
  (slice-doc + ledger + cc-prompt), CDC-verify **CC**'s implementation reports (writing
  `cdc-verification.md`), keep arc-plans via bubble-up, run arc-closes (composition check +
  independent subagent gate + project-plan bubble-up), and keep the project-status dashboard
  current. CC implements on a real toolchain; I verify structurally + rule on flags. **Peer
  frame, amend-don't-work-around, flag every deviation.**
- **Ledgers.** LEDGER-DISCIPLINE v2.0 (scale-free: slice/arc/project). Evidence strength
  `asserted < attested < reproduced < reconciled`; a `done` row reaches ≥ `reproduced` at its
  scale. **Stable IDs over renumbering** (append, never disturb rows with real evidence).
  PROJECT-MANAGEMENT.md governs plan-late-plan-deep + bubble-up/close + plan-change discipline
  (dated version-history entries naming which child surfaced each change).
- **Environment.** The CDC sandbox has **no 1.85+ cargo** (edition 2024). So cargo/executable
  rows are **attested-by-CC → reproduced-on-CI**; I reproduce *structural* rows by code +
  on-disk inspection. CC builds/tests on local 1.95.0.
- **⚠ Tooling note:** the `AskUserQuestion` popup has been failing this session
  ("stream closed"). **Ask decisions in prose**, not via the tool, until confirmed fixed.
- **Git:** `release/0.3.x` = pre-rebuild import; `main` reset onto it; **all rebuild work is on
  `release/1.0.x`** (Duncan set this up / freshened it this session). Branches cut from
  `release/1.0.x`.

## 2. Where the project stands

**MVP (A1–A3) + core (A4–A5) are in; A6 is nearly done; a UAT arc runs before A6 finishes.**

| Arc | State |
|-----|-------|
| A1 Substrate & node CRUD | ✅ complete (merged, CI-green) |
| A2 Graph, gates & derived order | ✅ complete |
| A3 Rollup & orient (MVP capstone) | ✅ complete |
| A4 Index & cache | ✅ closed (CI-green on release/1.0.x) |
| A5 Reconciliation (ODD-0019 freshness) | ✅ closed (CI-green on release/1.0.x) |
| **A6 Migrate, self-host & PM-skill** | **⏸ PAUSED after slice04** (slices 01–04 CDC-verified; **odm SELF-HOSTS**). Resumes at slice05 (PM-skill) after the Release Hardening arc. |
| **Release Hardening (UAT)** | **▶ ACTIVE** — named arc, canonical number deferred. Batch-1 punch list triaged; no code yet. |
| A7 telemetry / A8 forecasting | post-MVP horizon, **owned by another CDC** — ignore. |

**The big milestone this session: odm self-hosts (A6 slice04).** `odm self-host <plan>`
imported the `design-v1.0.0` plan-set → **45 work nodes** (1 project + 6 arcs + 38 slices)
under `nodes/`, each schema-stamped `<type>/v1.0`, in a containment tree (arc `part_of`
project, slice `part_of` arc); `odm check` green on **58 nodes** (13 `odd` + 45 work);
`rollup`/`orient` reproduce the real state. The hand-maintained truth (project-plan, the
dashboard) is now **derivable from odm querying itself**. project-plan **P-12**
reproducible-at-arc-close. Numbering scheme used: project=1000, arc N=1000+100·N,
slice=arc+position. `[gates.*]` added to `odm.toml` (ODD-0013 §5.1). No `odm-index` change.

Also this session: **slice03 (schema versioning, ODD-0020)** CDC-verified — per-type
`schema: <type>/vN.N` markers, absent⇒v0.1, per-type field-validity a `check` Error,
forward-compat; 13 nodes stamped `odd/v1.0`.

## 3. The pivot: A6 paused → Release Hardening arc

After slice04, Duncan moved into **hands-on UAT** of the self-hosted tool. The feedback
(CLI/naming/type/output) was too large + too model-level for a slice, so — per Duncan's call
and the mid-arc-pause precedent — it was **extracted to its own arc**:

**`docs/design-v1.0.0/arc-release-hardening/arc-plan.md`** — *Release Hardening: CLI, types,
naming & output (UAT-driven).* Decisions settled with Duncan:
- **Placement:** *named arc, canonical A-number deferred* (don't renumber the other CDC's
  A7/A8; numbering is itself under review here). A6 was briefly given a UAT slice05 (arc06
  v1.8) then reverted (v1.9) — A6 is back to slice05=PM-skill, slice06=retire-prose, marked
  `paused` on the dashboard.
- **`odm path` → `odm chain`** (decided).

**The UAT flow (different from other arcs):** feedback → CDC triages each into **surface**
(cc-prompt) / **model** (ODD-0013/0020 amendment or ADR first) / **question** → chunked
cc-prompts → CC implements → CDC verifies → **re-run `odm self-host`** to validate. Findings
accrete as **F-rows**; chunks as **C-rows**.

### Batch-1 punch list (Duncan's, first-pass, non-authoritative)

Verbatim source: **`arc-release-hardening/uat-punch-list.md`**. Triaged into 14 F-rows +
5 chunks in the arc-plan. Chunk breakdown, dependency-ordered:

| Chunk | Scope | Kind | F-rows |
|-------|-------|------|--------|
| **C-1** Adopt Oxur table styling/theming (colours, warm-orange theme) | **model/arch (ADR); ROUTE OPEN — see §4** | model | F-1 |
| **C-2** Type taxonomy: `odd`→`design`, add `research`; re-stamp the 13 nodes; reclassify research docs | **model (ODD-0013 + ODD-0020 amendment)** | model | F-2, F-3 |
| **C-3** `odm list` overhaul: drop number col; date-first + `--date=updated`; status col after type; branch-and-leaf tree (drop name-prefixing); max-width config+flag + ` ...` elision; de-numbered names | surface | F-4…F-9 |
| **C-4** Command cleanup: `context`→`project` (+`--name`); `path`→`chain`; `new` warns-not-displays on re-run; `rollup` help + md/json + `--out` (defaults md/ROLLUP) | surface | F-10…F-13 |
| **C-5** Fold `self-host` into `migrate` | surface/medium | F-14 |

**Order:** C-1 + C-2 are foundational (the renderer + the type names everything else uses) →
C-3 → C-4/C-5 anytime. **More batches expected (~3–4 total).**

**Model amendments queued** (draft the amendment *before* the cc-prompt): ODD-0013 (type
taxonomy), ODD-0020 (schema markers `design/v1.0`, `research/v1.0` + re-stamp path), + a new
**ADR** for the styling-crate route (§4). **Reflexive loop:** after C-2/C-3, re-run
`odm self-host` so the corpus reflects `design`/`research` + de-numbered names, re-verify green
(arc-plan row RH-7).

## 4. ⭐ THE OPEN QUESTION (start here next session): the oxur-cli / oxur-table route

C-1 (the styling work, Duncan's top-priority-for-v1 item) has a **route decision that is NOT
made** — it needs **mutual investigation, then discussion, then an ADR**. Do **not** just draft
it.

**The record (found this session — Duncan's memory was right, we'd discussed/built this before):**
- `oxur-table` was **originally its own crate** — ODD-0001 (Oxur Letter of Intent) lists it
  "✅ IMPLEMENTED"; there's a **Final** oxur design doc **0015 "oxur-table API (re)Design"**
  (2025-12-31).
- `oxur-cli/src/table/README.md` says it plainly: *"In late 2025 this module was in its own
  crate but as oxur-cli started to take shape, oxur-table was moved to oxur-cli/src/table."* It
  was **used by `oxd`** (odm's direct ancestor) for `list` — the warm-orange theme.
- **odm's own intent already mandates oxur-cli styling:** ODD-0012 + ODD-0013 §11 spec
  `odm-cli` output as "oxur-cli/tabled"; odm's `CLAUDE.md` says depend on `oxur-cli`
  `default-features = false` for `common::output` + `table`, don't enable `binary`. The
  compiler-stack fear is *already avoided* by `default-features = false` (lang/comp/repl/clap
  are behind the `binary` feature; lib-only deps are just tabled/serde/toml/colored/dirs).
- **The code drifted:** odm-cli currently uses raw `tabled` + `writeln!`, no oxur-cli, no theme
  → the plain colourless output Duncan sees. So C-1 **realigns code to intent** — I had earlier
  mislabeled the CLAUDE.md line as "doc-drift to fix"; it's the opposite (the intent is right,
  the code drifted). *(This is corrected in the arc-plan.)*

**The two routes (decide together):**
- **(A) Depend on `oxur-cli` lib-only** (`default-features = false`). Fast; matches CLAUDE.md
  verbatim. Cost: couples odm to oxur-cli's release cadence + drags its whole lib surface.
- **(B) Re-extract a standalone `oxur-table` crate** (reverse the late-2025 fold; matches
  ODD-0001 + the 0015 redesign + Duncan's "split term out of cli" instinct). Cleaner boundary
  (odm depends on one small crate). Cost: **upstream work in the oxur repo** (publish, version).

**Scoping question for the ADR:** table **only**, or table **+ terminal output helpers**
(`common::output`: success/error/info/warning)? Duncan said "table/**terminal**" → likely both.

**CDC lean:** B (it was always its own crate; there's a Final API design to lean on; cleaner
deps) — but it's more work and it's Duncan's repo, so it's his call. **API to lean on:**
`oxur_cli::table::OxurTable::new(data).render()`, generic over `Tabled`, `ColoredString` cells,
TOML theme (ANSI + hex), warm-orange default.

**Next-session action:** (1) jointly investigate A vs B — read oxur design doc 0015, weigh
coupling vs upstream-publish cost; (2) decide; (3) write the ADR; (4) settle the **base-branch**
question below; (5) then draft the C-1 cc-prompt.

## 5. Other open items / carry-ins

- **Base branch for the RH cc-prompts** (decide at C-1): the chunks build on the
  migrate+self-host+schema code, which lives on the **A6 slice04 tip**
  (`arc06-slice04-self-host-cutover`, off `release/1.0.x`, **unmerged**). Branch off that tip,
  or **merge green-A6-so-far to `release/1.0.x` first** and branch from there? **CDC lean:
  merge A6-so-far first** (keeps `release/1.0.x` the clean base, avoids a long dependent chain)
  — but confirm CI is green first.
- **A6 remaining after this arc:** slice05 (PM-skill, from ODD-0001, in
  `billosys/ai-engineering`) → slice06 (retire redundant framework prose; folds the CLAUDE.md
  oxur-cli fix — now the C-1 styling change, not a doc-only fix) → **A6 arc-close** → v1.0.0
  MVP-plus self-hosting.
- **Named-follow (out of scope, noted):** `odm rollup --json` regenerating
  `project-status.html` — the dashboard becomes derivable from odm.
- **Parked (post-MVP backlog):** the two-reads-per-bare-command optimization (odm-reconcile
  perf); the schema type↔`type:` mismatch check (slice03 CDC note); the per-class-vs-per-type
  validity tightening (slice03 CDC note).
- **CI:** A6 slices 01–04 cargo rows are attested-pending-CI; they flip to reproduced on CI
  green.

## 6. Key files

- **Dashboard:** `docs/design-v1.0.0/project-status.html` (keep updated each milestone — A6
  `paused`, Release Hardening `active`).
- **Project plan / ledger:** `docs/design-v1.0.0/project-plan.md` (P-12 = self-host DoD).
- **Release Hardening arc:** `docs/design-v1.0.0/arc-release-hardening/arc-plan.md` +
  `uat-punch-list.md` (Duncan's verbatim batch 1).
- **A6 arc-plan (paused):** `docs/design-v1.0.0/arc06-migrate-self-host/arc-plan.md` (v1.9).
- **Genesis (why odm exists):** §8 of this file (merged 2026-07-25); the verbatim
  original is git-tracked as `docs/dev/0025-odm-discussion-bootstrap-rebuild-planning.md`.
- **oxur-table records:** `oxur` repo — `crates/oxur-cli/src/table/` (+ README),
  `crates/design/docs/06-final/0015-phase-2-oxur-table-api-design.md`, ODD-0001.
- **Memory:** `odm-redesign-decisions`, `odm-release-hardening-arc`, `odm-status-dashboard`
  (in the space's memory dir).

## 7. Do-this-next (the crisp version)

1. Read this doc + `arc-release-hardening/arc-plan.md` + `uat-punch-list.md`.
2. **Investigate the oxur-cli/oxur-table route (§4)** — read oxur-0015, weigh A vs B — then
   **discuss with Duncan and decide.** (Ask in prose; the popup tool is flaky.)
3. Settle the base-branch question (§5); confirm A6 CI is green.
4. Write the **ADR** for the styling route, then the **C-1 cc-prompt**.
5. Expect more UAT batches from Duncan — triage into F-rows/chunks as they land.

---

## 8. Genesis — why odm exists

> Merged 2026-07-25 from `workbench/odm-session-bootstrap.md` (authored 2026-06-19 by
> Claude, CDC/team-support thread, with Duncan, at the close of the pos-loyalty-svc
> session where the failure this tool fixes surfaced in the worst way). The
> load-bearing sections are carried near-verbatim below. **Elided as superseded**
> (disclosed, not dropped — and correcting an earlier claim: `workbench/` is
> gitignored, so there is *no* git history at the old path; the genesis full text
> is git-tracked verbatim as
> `docs/dev/0025-odm-discussion-bootstrap-rebuild-planning.md`): the requirements
> catalogue (§4 — now governed by ODD-0012/ODD-0013), the suggested first steps
> (§7 — the SDLC that was then run, producing ODD-0011…0015), and the carry-over
> artifact list (§8 — the research report is now **ODD-0011**).

### 8.1 Mission (as chartered)

`odm` ("our document manager") pre-existed as a crate inside the Oxur
language project; the charter was to split it into its own repo and grow it into
the planning/tracking substrate for the collaboration-framework. The end-state: a
markdown/git-native, dependency-ordered planning system that is
**self-documenting and self-tracking** — so the *mechanical* rules of the
framework (numbering, ordering, deferral-tracking, status discipline,
drift-watching) stop living as prose rules a human/LLM must remember and become
**tool-encoded checks**. When `odm` is done, framework docs/rules that are purely
mechanical get deleted and replaced by `odm` commands. Success test: *a fresh
session (human or LLM) reaches full situational awareness from `odm orient`
alone.*

### 8.2 The failure this fixes (the founding case)

Over a long pos-loyalty-svc session, work was driven slice-by-slice with
disciplined per-slice rigor — yet the *program-level* vision was lost and
deployed state drifted invisibly. The worked example: **production had a Cloud
SQL DB provisioned and migrated, but the service was never wired to it**
(`DB_HOST` missing from the prod overlay). The authenticated API was fail-closed
at 503 by design, and nobody saw it, because the fact "prod service wired to its
DB" existed only as the intersection of five scattered documents. No single
artifact tracked it; no check could fire.

**Root causes (the diagnosis the tool answers):**

1. **Identity conflated with order** — "Phase 9 / 8.5 / 10" use the number as
   both name and claimed sequence; deferrals make the numbers lie.
2. **No single source of truth for state** — state reconstructed by archaeology
   (git log + grep + scattered ledgers); drift invisible until tripped on.
3. **Plan is desired-state with no reconciliation loop against actual-state** —
   verification point-in-time, against the repo, never against live reality.
4. **Binary status hides integration-level truth** — `done/open/deferred` too
   coarse; "done at its layer" masked "not working when integrated."
5. **Dependencies are prose, not data** — nothing can mechanically warn "you are
   working out of order; dependency X is still open."
6. **The information architecture makes vision-loss the path of least
   resistance** — rebuilding the global picture each turn is expensive, so the
   cheap, recent, local artifact wins attention; context resets make an LLM pay
   the reconstruction tax repeatedly. *Fix: make global state cheap to load.*
7. **Vocabulary drift** — "phase" predates "project/arc/slice"; renumbering
   created stale links.

### 8.3 The convergent architecture (research finding)

Five independent literatures — WBS/CPM project management, Design Structure
Matrix engineering, build systems, infrastructure reconciliation, and
docs-as-code — all converge on one architecture:

> **Stable-identity nodes + an explicit dependency DAG + order *derived* by
> topological sort + per-edge staleness/reconciliation checks + a single
> *complete* graph as the source of truth.**

`odm` is therefore two things fused: **a build system for the plan** (ordering +
readiness + staleness) and **a reconciler for the plan's state** (desired vs.
actual drift). The formal backbone: *Build Systems à la Carte* proves
**correctness requires a complete dependency set — an incomplete graph silently
permits running a step before its inputs are satisfied.** That is the DB
failure, stated as a theorem. Paired with the IaC/Kubernetes lesson: **you can
only detect drift on what the source of truth claims to manage.**

*Evidence calibration:* trust the formal (WBS-is-scope-not-sequence,
identity-≠-order, topo-sort + cycle detection, the à-la-Carte theorem,
closed-loop control, Little's Law); treat as advisory lore (CCPM buffers, WSJF
scoring, most agile/SAFe ceremony). Dependency order gives a *correct* order,
not the *fastest* — priority is a separate, softer layer. Full citations:
**ODD-0011** (`docs/design/06-final/0011-research-a-markdowngit-native-dependency-ordered-planning-system.md`).

### 8.4 Principles & guardrails (standing)

- **Markdown/git-native; no ticketing system.** Files are the source; `odm`
  commands are "the build."
- **Identity ≠ order.** Stable IDs; order derived from the graph.
- **Complete graph or no detection.** Every real dependency must be an edge;
  bias toward over-declaring deps.
- **Track integration-level facts**, not just per-layer completion.
- **Supersede, don't delete.** History is preserved (git + supersession links).
- **Trust formal over lore**; keep advisory layers optional.
- **Self-documenting, self-tracking** is the success test (`odm orient` alone).

### 8.5 How this actualizes the collaboration-framework

- "number by dependency / don't work out of order" → the DAG + staleness guard.
- "disclosed deferral with named re-entry" → first-class deferred status with a
  checkable re-entry condition.
- "spec-keeping / no silent drops" → diff scope-as-delivered vs scope-as-declared
  in the rollup.
- "verify, don't assert" → the reconciler (desired-vs-actual probes).
- "cheap global state so vision isn't lost" → `orient` + the generated rollup.

Once tool-encoded, the corresponding prose rules retire (replaced by "run
`odm check`"). The framework keeps its *character/posture* layer; `odm` absorbs
the mechanical layer. (This is A6 slice06's charter.)
