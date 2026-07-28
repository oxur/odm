---
id: 01KWXMBBTK8VQ0JMYBMR5R6SY2
number: 1604
type: slice
schema: slice/v1.1
name: self-host cutover
created: 2026-07-07
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc06-migrate-self-host/slice04-self-host-cutover/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KWXMBBTKNA3A0QC3SWPHBNAX
status:
  built:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  planned:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  tested:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
---
# Slice 04 (Arc 06): self-host cutover

> Plan-of-record for A6 slice04 — **the loop-closer.** Bring the `design-v1.0.0`
> plan set (project-plan, arc-plans, slice docs) into the node model as **work
> nodes**; the plan lives under `nodes/`; `orient`/`rollup`/`check` run on the
> self-hosted corpus. After this slice, **odm manages its own plan** — the design
> documents become queryable through `odm orient`. This is the self-hosting trigger
> the whole MVP has pointed at (project-plan P-12; ODD-0013 §9).

## Goal

odm's own plan — the project, its arcs, and their slices — exists as
project/arc/slice **work nodes** under `nodes/`, connected by containment edges,
schema-stamped `<type>/v1.0` (slice03's payoff), with gate status reflecting
reality. `odm rollup`/`orient`/`check` run over the mixed corpus (the 13 `odd`
nodes from slice02/03 **plus** the new work tree) and reproduce the real project
state. The hand-maintained truth (project-plan.md, the dashboard) becomes
**derivable from odm querying its own nodes**.

## Why now / why slice03 first

slice03 stamped every `nodes/` file `odd/v1.0` and taught `check` per-type
field-validity. slice04 is the **first time work nodes are created via import** —
so the per-type schema markers (`project/v1.0`, `arc/v1.0`, `slice/v1.0`) get
minted for real, and the per-type validity rule (`{supersedes, affects}` are
document-only; work nodes carry `{desired_facts, deferred}`) meets work nodes for
the first time. Sequencing slice03 first is what lets the plan-set land **already
at v1.0** and stay `check`-green.

## The crux: reflexive-import ordering

The plan-set we are importing **contains the plan that governs this very import**
(arc06's arc-plan, and slice04's own docs). Importing it is reflexive. The safety
discipline (arc-plan open Q "self-host cutover safety"):

- **Cutover at a git checkpoint** — a committed, consistent plan-set state, so a
  bad import is recoverable by `git reset`.
- **`--dry-run` first** — preview the full cutover, write nothing, eyeball the plan.
- **The project (governing root) node created last** — a partial failure never
  leaves a dangling root; the tree is built children-up, root committed only when
  the arcs/slices are in place.
- **Idempotent re-run** — the node representing "slice04 self-host cutover" is
  created *by* slice04; that's fine because the import is idempotent (keyed on the
  work node's stable number, per slice01's describe-or-create) and last-ordered.

## Open design questions — resolved / to settle here

- **What migrates (scope boundary). → Recommend: project + A1–A6 + their slices
  (the v1.0.0 MVP).** The `design-v1.0.0/` tree also holds `arc07-*`/`arc08-*` —
  the **post-MVP horizon** (project-plan §4: "scoped"), owned by another CDC and
  actively being drawn (0 slices today). Importing them now would create
  undeveloped-stub arcs and collide with in-flight work. So the cutover imports the
  **deep-planned MVP (A1–A6)**; A7/A8 enter when they are real, by whoever closes
  them. *(Flag: if we want the full roadmap represented, A7/A8 import as `planned`
  arc stubs — but that is a coordination call, not this slice's default.)*

- **How the plan-set becomes work nodes (the mechanism). → Recommend: a
  directory-structure + ledger migrate adapter** (reusing `odm-migrate`'s report /
  idempotence / `--dry-run` / never-delete / two-pass id-resolution rails). The
  `design-v1.0.0/` **directory hierarchy is already the project→arc→slice tree** —
  no prose parsing for *structure* (it's the filesystem). Number/name come from the
  dir names + the doc H1. **Status** derives from the most-structured available
  signal — the ledgers: an arc with a `closing-report.md` **or** a `done` Project-
  Ledger `P-row` ⇒ closed; a slice whose `ledger.md` rows are all `done` ⇒ its gate
  reached; else planned — at `Evidence::Asserted`. *(Flag: if ledger-table parsing
  proves fragile, a small explicit **cutover manifest** authored from the plan-set
  is the fallback — a one-time bootstrap seed, reviewable in the PR; after cutover
  status lives in node gates, never the manifest. Amend-don't-workaround: pick one,
  flag it.)*

- **Status source for the A1–A3 gap.** A1/A2 predate the closing-report convention
  (project-plan P-1/P-2 "disclosed gap"); A3 uses `arc-close.md`. So status can't
  come from `closing-report.md` presence alone — fall back to the **Project Ledger
  P-row** status (P-1…P-5 `done`, P-6 `open`) as the authoritative arc-close signal.

- **Which is source of truth after cutover? → Both coexist (interim), like the
  ODDs.** The MD plan-set stays the human-authored source; the nodes become the
  queryable representation. Retirement of the redundant prose is **slice06 / post-
  MVP** — do **not** force it here. (Flag: this is the "two representations coexist"
  question the slice02/03 bubble-ups raised, now live for the plan layer.)

## Scope — in

1. **Plan-set → work nodes** — project + A1–A6 arcs + their slices become
   project/arc/slice nodes under `nodes/`, schema-stamped `<type>/v1.0`;
   number/name/created/updated carried; legacy plan-set MD **intact** (never-delete).
2. **Containment tree** — `arc part_of project`, `slice part_of arc`, matching the
   plan-set hierarchy; the work-decomposition children rules (project→arc→slice)
   satisfied; no orphan work nodes (project is root; every arc/slice parented).
3. **Gate status reflects reality** — each arc/slice's gate position derived from
   the plan (closed ⇒ gates reached; active ⇒ partial; planned ⇒ none) at
   `Evidence::Asserted` (claimed from the record — the honest level for a migration;
   independent reproduction already happened in the CDC verifications).
4. **Mixed-corpus `check` green** — the 13 `odd` nodes + the work tree coexist;
   `odm check` exits 0 (or every finding is a Warning — e.g. an undeveloped stub —
   never an Error): work nodes parented, docs orphan-exempt, no dangling edges, no
   wrong-type-field, schema markers valid.
5. **`orient`/`rollup` on the self-hosted corpus reproduce the real state** — the
   loop-closing demonstration: `odm rollup` shows A1–A5 done, A6 active, the rest
   planned; `odm orient` gives a coherent brief (vision → focus → ready/blocked →
   integrity).
6. **Reflexive-import safety** — `--dry-run` previews the whole cutover writing
   nothing; project node last; a git checkpoint documented; idempotent re-run.

## Scope — out (named, not dropped)

- **A7/A8 import** — the post-MVP horizon, owned separately (see scope boundary).
- **work↔odd reference edges** (a slice `verifies`/`consumes` an ODD; an arc cites
  ODDs) — a rich **post-cutover enrichment**; slice04 does **containment only**.
  The plan docs reference ODD-0013/0019/0020 heavily; wiring those edges is real
  fidelity but not the MVP bar.
- **Regenerating project-status.html from `odm rollup --json`** — the natural
  *follow* S-5 enables (the dashboard becomes derivable), but out of this slice's
  bar. Named because it's the beautiful end-state.
- **Retiring the redundant plan/framework prose** — **slice06**.
- **PM-skill population** — **slice05**.

## Fidelity bar (calibration)

The bar is **structure + plausible asserted status + the read commands run over
it** — not perfect gate-by-gate reproduction. At `Asserted`, the migrated status is
claimed-from-record; `rollup` should reflect the real arc-close reality (A1–A5
done, A6 active), but exact per-gate evidence positions are asserted, reproduced at
arc-close (A-8). What must be exact: the **tree** (every arc/slice present and
correctly parented) and **`check` green**.

## Verification approach

`odm-migrate` + `odm-cli` tests:

- the plan-set adapter creates project/arc/slice nodes with the right types +
  `<type>/v1.0` stamps; a byte-snapshot of the plan-set is unchanged after.
- the imported containment tree matches the directory hierarchy (arc `part_of`
  project, slice `part_of` arc); no orphan work nodes.
- a closed arc (A5) imports with its gates reached; an active arc (A6) partial; a
  planned slice none — at `Asserted`.
- `odm check` on the full self-hosted corpus (odd + work) → exit 0 (findings, if
  any, are Warnings, never Errors); no wrong-type-field on the work nodes.
- `odm rollup` / `odm orient` over the self-hosted corpus reflect the real project
  state (A1–A5 done, A6 active, remainder planned).
- `--dry-run` writes nothing; a re-run is idempotent (creates 0); never-delete
  proven by byte-snapshot at plan-set scale.
- clippy `-D warnings`; no `unsafe`; ≥ 90% new-path line cov; **full workspace
  green**; **no `odm-index` change** (self-host does not require the index).

Cargo rows are CC-`attested`, reproduced via CI / local 1.85+.

## Exit criteria

Ledger S-1…S-7 reach a final status: the plan-set (project + A1–A6 + slices) lives
under `nodes/` as schema-stamped work nodes in a correct containment tree; gate
status reflects reality at `Asserted`; the mixed corpus is `check`-green;
`rollup`/`orient` reproduce the real project state; the cutover is `--dry-run`-safe,
idempotent, and never-delete; gates + no regression + no index change. **On close,
odm self-hosts** — P-12 becomes reproducible-at-arc-close, and only PM-skill
(slice05) + prose-retirement (slice06) remain before the arc closes and the v1.0.0
MVP-plus is self-hosting.

> **Render/convention:** `writeln!` + `tabled` (no `oxur-cli`). **Git:** branch off
> `release/1.0.x` (if 01–03 unmerged, off the slice03 tip + flag rebase). Reuse the
> `odm-migrate` rails (report / idempotence / `--dry-run` / never-delete / two-pass
> id resolution); the schema stamping rides slice03's `stamp_schema` on every create
> path.
