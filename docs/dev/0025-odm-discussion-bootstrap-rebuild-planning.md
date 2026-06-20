# odm — New Session Bootstrap: build the planning / doc-management system

> **Purpose of this document.** Start a fresh Claude Desktop Cowork session,
> dedicated to building `odm` into a standalone tool that makes our
> collaboration-framework self-documenting and self-tracking. This file is
> self-contained: it carries the context, the root-cause analysis, the research
> finding, and the full requirements, so the new session does not start cold.
> **Travel companion:** copy `planning-system-research.md` (the cited research
> report) into the new repo alongside this file.
>
> _Authored 2026-06-19 by Claude (CDC/team-support thread) with Duncan, at the
> close of a long project-x session where the failure this tool fixes
> surfaced in the worst way._

---

## 1. Mission

`odm` ("our document manager") already exists — a Rust tool we wrote last year
to manage prompts, project plans, and a densely-connected dependency graph for a
large language-implementation project (a Lisp on Rust). It currently lives as a
crate inside that project. **This session: split `odm` into its own repo and
grow it into the planning/tracking substrate for the collaboration-framework.**

The end-state: a markdown/git-native, dependency-ordered planning system that is
**self-documenting and self-tracking** — so that the *mechanical* rules of the
collaboration-framework (numbering, ordering, deferral-tracking, status
discipline, drift-watching) stop living as prose rules a human/LLM must
remember, and instead become **tool-encoded checks**. Two threads of multi-year
work unify here: the doc-management tool (improve) and the collaboration-
framework (fix its mechanical gaps). When `odm` is done, several framework
docs/rules that are purely mechanical can be deleted and replaced by `odm`
commands.

Rust libraries already in hand: **petgraph** for the dependency DAG.

## 2. Why now — the failure this fixes

This was born from a concrete, expensive failure. Over a long project-x
session, work was driven slice-by-slice with disciplined per-slice rigor — yet
the *program-level* vision was lost and deployed state drifted invisibly. The
worked example: **production had a Cloud SQL DB provisioned and migrated, but
the service was never wired to it** (`DB_HOST` missing from the prod overlay).
The authenticated API was fail-closed at 503 by design, and nobody saw it,
because the fact "prod service wired to its DB" existed only as the intersection
of five scattered documents. No single artifact tracked it; no check could fire.

### Root causes (the diagnosis the tool must answer)

1. **Identity conflated with order** — "Phase 9 / 8.5 / 10" use the number as
   both name and claimed sequence; deferrals and out-of-order completion make
   the numbers lie.
2. **No single source of truth for state** — state is reconstructed by
   archaeology (git log + grep + scattered ledgers); drift is invisible until
   someone trips on it.
3. **Plan is desired-state with no reconciliation loop against actual-state** —
   verification is point-in-time, against the repo, never continuously against
   live reality. (The freeze-harnesses were a brilliant *local* instance of the
   right pattern; it was never lifted to the program level.)
4. **Binary status hides integration-level truth** — `done/open/deferred` is too
   coarse; "done at its layer" masked "not working when integrated."
5. **Dependencies are prose, not data** — so nothing can mechanically warn "you
   are working out of order; dependency X is still open."
6. **The information architecture makes vision-loss the path of least
   resistance** — rebuilding the global picture each turn is expensive, so the
   cheap, recent, local artifact wins attention. This bites an LLM especially
   hard (context resets re-pay the reconstruction tax). *The fix is to make the
   global state cheap to load and mechanically present, so it can compete with
   the local task.*
7. **Vocabulary drift** — "phase" predates "project/arc/slice"; artifacts were
   never retrofitted, so cross-references became ambiguous and renumbering
   created stale links.

## 3. The convergent architecture (research finding)

Five independent literatures — WBS/CPM project management, Design Structure
Matrix engineering, build systems, infrastructure reconciliation, and
docs-as-code — **all converge on one architecture, and it is the one our
substrate already runs on:**

> **Stable-identity nodes + an explicit dependency DAG + order *derived* by
> topological sort + per-edge staleness/reconciliation checks + a single
> *complete* graph as the source of truth.**

`odm` is therefore two things fused: **a build system for the plan** (ordering +
readiness + staleness) and **a reconciler for the plan's state** (desired vs.
actual drift). The formal backbone (peer-reviewed): *Build Systems à la Carte*
proves **correctness requires a complete dependency set — an incomplete graph
silently permits running a step before its inputs are satisfied.** That is the
DB failure, stated as a theorem. Pair with the IaC/Kubernetes lesson: **you can
only detect drift on what the source of truth claims to manage** — so the plan
must explicitly track integration-level facts, or they stay invisible.

**Evidence calibration (build on the formal, treat the lore as advisory):**
- *Trust:* WBS-is-scope-not-sequence (ISO 21511 / PMI / MIL-STD-881), identity-≠-order (protobuf spec; genomics evidence), topo-sort + cycle detection, the à-la-Carte correctness theorem, closed-loop control, Little's Law.
- *Advisory lore only:* CCPM buffers, WSJF/Cost-of-Delay scoring, most agile/SAFe ceremony — weak/absent controlled evidence. Use on top of dependency order, never as the ordering.
- *Real limit:* dependency order gives a *correct* order, not the *fastest* — priority is a separate, softer layer.

Full citations and per-claim empirical/lore tags: see `planning-system-research.md`.

## 4. Requirements

### 4a. Hierarchy & lifecycle (Duncan's baseline)

- Per-repo **config file**; one option sets the location of project docs/plans/prompts.
- Create a project (dir + a **summary/way-finding file** with all project
  metadata in YAML front-matter); list / rename / delete projects (git preserves
  history; **deleting non-git-tracked files is prohibited**).
- List / add / rename / delete **arcs**; each arc has a summary/way-finding file
  with managed, queryable front-matter metadata.
- List / add / rename / delete **slices**; each slice has a summary/way-finding
  file with managed, queryable front-matter metadata.
- (Add **steps** as the leaf level, same pattern.)
- **Current project** and **current arc** context (so `--project`/`--arc` aren't
  needed on every call).
- Canonical vocabulary = **project → arc → slice → step**; the tool owns it; a
  **migrate** command retrofits legacy ("phase") artifacts.

### 4b. Differentiating capabilities (research-derived)

- **Stable, opaque IDs** per node, never reused/renumbered; human name/number is
  metadata, not identity. Link-integrity checking (no dangling refs;
  supersession chains preserved).
- **Dependency edges as first-class data** (`depends_on`, `verifies`,
  `supersedes`, `consumes`, `part_of`, `blocked_by`) → petgraph DAG.
- **DAG validation + cycle detection** (Kahn); cycles surface and require an
  explicit **tear** (assume-this-dependency) marker — never silent.
- **Derived-order queries:** `next` (ready: all deps satisfied), `blocked` (+ by
  what), `path` (dependency chain / critical path), topo-order listing.
- **Out-of-order / staleness guard** — touching a node with unsatisfied deps
  warns loudly (build-staleness applied to the plan).
- **Multi-gate status vectors** — per node, status across named gates
  (`planned / built / tested / deployed / verified-live / operator-confirmed`),
  each set independently; gate sets configurable per node type.
- **State reconciliation (`plan`-style diff)** — nodes declare *desired-state
  facts*; the tool diffs them against *actual* via pluggable probes (shell
  checks / the freeze-harness pattern), on demand and scheduled. Honest only
  about tracked facts → nudges enumerating integration-level facts.
- **Generated global rollup/index** — one regenerable view (way-finding tree +
  status vectors + ready/blocked + drift). Never hand-maintained.
- **Provenance + future-work reservation** — mark origin (planned vs
  amendment/discovered); allow tentative `future` placeholders so cross-stream
  work is visible and reserved, and emergent scope stays distinguishable from
  original intent.
- **Decision records as a node type** (ADR/RFC-style): numbered, immutable,
  supersede-don't-delete, linkable — folded into the same graph.

### 4c. LLM-ergonomics (the tool, QA'd for an LLM agent's reach)

The point: **encode the framework's behavioral disciplines into the tool so they
don't depend on the agent's in-context vigilance.**

- **One-shot orient** (`odm brief`/`odm orient`): vision → current project/arc →
  ready work → blockers → drift, in a single cheap call. Run first thing every
  session — "global state cheap to load" made literal; defeats the
  context-reconstruction tax.
- **`--json` on every query**, stable documented schemas (human-readable by
  default, machine-readable on request).
- **Commands named after the question, not the mechanism:** `next`, `blocked X`,
  `show X` (node + edges + status vector + way-finding in one call), `check`
  (plan consistent?), `reconcile` (did reality drift?).
- **Errors that name the fix** — problem + the exact command to resolve it.
  Errors as affordances.
- **`check` as a git pre-commit / CI gate** — fails on cycles, dangling refs,
  out-of-order work, unreconciled drift. Discipline enforced mechanically.
- **Idempotent describe-or-create + `--dry-run`** on mutators; **non-interactive
  mode** (`--yes`) for agent runs.
- **`odm` with no args orients** (capability map + current context), never bare-errors.

## 5. Principles & guardrails

- **Markdown/git-native; no ticketing system.** Files are the source; `odm`
  commands are "the build."
- **Identity ≠ order.** Stable IDs; order derived from the graph.
- **Complete graph or no detection.** Every real dependency must be an edge, or
  out-of-order/drift cannot be caught. Bias toward over-declaring deps.
- **Track integration-level facts**, not just per-layer completion — drift is
  only visible on what the SoT manages.
- **Supersede, don't delete.** History is preserved (git + supersession links).
- **Trust formal over lore** (see §3 calibration); keep advisory layers optional.
- **Self-documenting, self-tracking** is the success test: a fresh session (human
  or LLM) should reach full situational awareness from `odm orient` alone.

## 6. How this actualizes the collaboration-framework

The framework's *mechanical* disciplines become `odm` checks:
- "number by dependency / don't work out of order" → the DAG + staleness guard.
- "disclosed deferral with named re-entry" → a first-class deferred status with a
  checkable re-entry condition.
- "spec-keeping / no silent drops" → diff scope-as-delivered vs scope-as-declared
  in the rollup.
- "verify, don't assert" → the reconciler (desired-vs-actual probes).
- "cheap global state so vision isn't lost" → `orient` + the generated rollup.

Once these are tool-encoded, the corresponding prose rules in the framework can
be retired (replaced by "run `odm check`"). The framework keeps its
*character/posture* layer; `odm` absorbs the mechanical layer.

## 7. Suggested first steps for the new session

Run the framework's own SDLC on `odm` itself (dogfood from line one):

1. **Project definition** — confirm mission/scope/non-goals (this doc is the seed).
2. **Survey the existing `odm` crate** — what's already built; what splits cleanly
   into the new repo; what petgraph usage exists.
3. **Design doc** — the front-matter schema (node types, edge types, status-gate
   sets, desired-state-fact format), the file/dir layout, the command surface
   (§4c), and the check/reconcile model.
4. **Arc/slice breakdown** — MVP first: hierarchy CRUD + stable IDs + the DAG +
   `next`/`blocked`/`check` + `orient`/rollup. Reconciler and gate-probes as a
   following arc.
5. **Build with a ledger** — and, fittingly, track `odm`'s own construction in
   `odm` as soon as the MVP can self-host.

## 8. Carry-over artifacts

- `planning-system-research.md` — the cited research report (copy into the new repo).
- The collaboration-framework docs (Constitution Supplement + Engineering
  Methodology + the four working-practice docs) — `odm` is their mechanical
  actualization.
- This bootstrap.

_After this is handed over, the original session returns to the project-x
plan to find the best make-do solution under current (tool-less) limitations —
fully aware of the blind spots `odm` will eventually close: the prod-DB 503, the
public-IP exposure, the staging/Phase-8.5/Phase-10 sequencing, and the env-parity
vision._
