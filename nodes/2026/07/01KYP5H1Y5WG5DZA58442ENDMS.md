---
id: 01KYP5H1Y5WG5DZA58442ENDMS
number: 592599000
type: artifact
schema: artifact/v1.1
name: 'Closing report — Slice 04 (Arc 06): self-host cutover'
created: 2026-07-07
updated: 2026-07-07
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice04-self-host-cutover/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTK8VQ0JMYBMR5R6SY2
---
# Closing report — Slice 04 (Arc 06): self-host cutover

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc06-slice04-self-host-cutover`, branched off the slice03 tip
> (slices 01–03 unmerged; the prompt's fallback). Rebase onto `release/1.0.x`
> once 01–03 merge.
>
> **The loop-closer.** odm's own `design-v1.0.0` plan set (project + A1–A6 arcs +
> their slices) now lives under `nodes/` as work nodes; `orient`/`rollup`/`check`
> run over the self-hosted corpus. **odm manages its own plan** (project-plan P-12;
> ODD-0013 §9).

## Per-row walk

**S-1 — plan-set → work nodes — done (attested).** New `odm_migrate::selfhost`
module (`self_host`) reusing the slice01 rails (report / idempotence / `--dry-run`
/ never-delete / two-pass id resolution). `selfhost_creates_work_nodes` → ok. The
**real cutover is committed**: `odm self-host docs/design-v1.0.0` created **45 work
nodes** (1 project + 6 arcs + 38 slices) under `nodes/`, each schema-stamped
`<type>/v1.0` (the first mint of `project/v1.0`·`arc/v1.0`·`slice/v1.0`); the
plan-set Markdown is byte-for-byte intact (never-delete, byte-snapshot proven).

**S-2 — containment tree — done (attested).** The directory hierarchy *is* the
tree: `arc part_of project`, `slice part_of arc`. `selfhost_tree_matches_hierarchy`
+ `check_no_orphan_work_nodes` → ok — the project is the root (no `part_of`), every
arc/slice is parented, and `odm check` finds no orphan on the real corpus.
Containment only (work↔odd reference edges are out of scope).

**S-3 — gate status reflects reality (Asserted) — done (attested).**
`selfhost_status_from_plan` → ok: a closed arc reaches its terminal (`verified`),
an active arc is partial (`in-progress`, not `complete`), a planned slice reaches
nothing — all at `Evidence::Asserted`. The arc-close signal is a **P-row `done` OR
an arc-level close file** (A1/A2 predate the close file — the disclosed gap — so
the Project-Ledger P-row is authoritative); slice-complete is `closing-report.md`
presence. The real `odm rollup` shows A1–A5 verified, A6 planned+in-progress.

**S-4 — mixed-corpus `check` green — done (attested).**
`check_green_on_self_hosted_corpus` → ok, and the real corpus reports
"ok (58 node(s), no problems)" (13 odd + 45 work), exit 0 — **no findings at all**.
Green by construction: work nodes carry only `{part_of, gates, decomposed}` (never
the document-only `{supersedes, affects}` → no wrong-type-field); document nodes
are orphan-exempt; closed arcs affirm `decomposed` (no advanced-without-decomposition
warning); schema markers are `<type>/v1.0` (valid). slice03's per-type validity met
work nodes for the first time and the bucket rule held.

**S-5 — `orient`/`rollup` reproduce the real state — done (attested).**
`rollup_reflects_self_hosted_state` (A5 `verified` reached; A6 `in-progress`
reached, `complete` not) + `orient_runs_on_self_hosted_corpus` (resolves the
self-hosted project, not the empty fallback) → ok. The real `odm rollup` reproduces
A1–A5 done / A6 active; `odm orient` runs clean over the self-hosted corpus. **The
loop closes** — the hand-maintained truth is now derivable from odm querying its
own nodes. (Requires the `[gates.*]` config added to `odm.toml` this slice.)

**S-6 — reflexive-import safety — done (attested).**
`selfhost_dry_run_writes_nothing` + `selfhost_idempotent` → ok. `self_host`
persists **children-up** — slices, then arcs, then the project root **last**
(`persist_rank`) — so a partial failure never leaves a dangling root. The real
cutover ran on a committed checkpoint; a re-run reports "0 created, 45 skipped"
(idempotent, keyed on `(type, number)`); the plan-set MD is intact. The reflexive
slice04 node (this slice describing its own cutover) is just another slice, handled
by idempotence + last-ordering.

**S-7 — gates + no regression + no index change — done (attested).** clippy
`--all-targets --all-features -D warnings` exits 0; no `unsafe` in
`crates/odm-migrate/src`; line coverage selfhost.rs 96.25% / lib.rs 99.21% /
mapping.rs 99.06% / legacy.rs 93.86% (all ≥ 90); `cargo test --workspace` green (52
suites); **`git diff crates/odm-index/` is empty** — self-host rides the store +
command surface (A1/A3), not the index.

Rows: 7. Done: 7. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item shipped: the plan-set adapter, the 45-node self-hosted work
tree committed under `nodes/`, the containment tree, asserted gate status
reflecting reality, mixed-corpus `check` green, `rollup`/`orient` reproducing the
state, and reflexive-import safety (dry-run / children-up / idempotent /
never-delete). Every "out" item stayed out: **no** A7/A8 import (post-MVP horizon),
**no** work↔odd reference edges (containment only), **no** dashboard regeneration
(the named follow), **no** prose retirement (slice06), **no** PM-skill (slice05),
**no** `odm-index` change.

Two additions are disclosed, not scope-creep: the `[gates.*]` config in `odm.toml`
(a genuine self-host prerequisite — Deviation 4) and the `decomposed` affirmation
on closed arcs (fidelity + warning suppression — Deviation 6).

## Deviations / decisions flagged

See the ledger's **Deviations / decisions flagged**. In brief: mechanism =
directory-structure adapter (no manifest needed); arc-close status from the P-row
or arc-level close file (A1/A2 gap → P-row); scope = A1–A6 (arc07/arc08 excluded,
flagged); `[gates.*]` added to `odm.toml`; `created`/`updated` = cutover date;
closed arcs affirm `decomposed`; **no model amendment** (the work-node model hosted
the plan faithfully as-is).

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice04 deliver A-4 + the A-8 mechanism?** Yes. The plan-set is
self-hosted (45 work nodes + 13 odd nodes, `check` green, `rollup`/`orient`
reproducing reality). A-4 (slice04 closed) is `attested`; A-8 (compose: odm
self-hosts — its plan lives under `nodes/` and `orient`/`rollup`/`check` run on the
real corpus) is **mechanism-complete**, to be reproduced at arc scale at arc-close.
**Project-plan P-12 is now reproducible-at-arc-close** — the self-hosting trigger
has fired.

**2. What slice04 reveals for slice05/06.**
  - **The redundant *mechanical* prose is now concretely identifiable (feeds
    slice06).** The plan-set's structure, status, and dependency information now
    live queryably in nodes: the project→arc→slice tree (`odm rollup`), arc/slice
    gate status (`odm rollup`/`orient`), and integrity (`odm check`). The
    hand-maintained artifacts that duplicate this — the `project-status.html`
    dashboard (derivable from `odm rollup --json`), the per-arc "slice status"
    prose, and the numbering/ordering bookkeeping — are what slice06 retires with a
    pointer to `odm`. slice06's scope is now grounded in what actually became
    redundant, not a guess.
  - **The dashboard-from-`rollup --json` follow is real and close.** S-5 makes
    `project-status.html` derivable; regenerating it from `odm rollup --json` is
    the natural next step (named out-of-bar here) and a concrete slice06/post-MVP
    deliverable — the "beautiful end-state" where the status page is a projection,
    not a hand-edit.
  - **Both representations coexist (the interim answer).** The MD plan-set stays
    the human-authored source; the nodes are the queryable representation. Retiring
    the redundant prose is slice06 — slice04 deliberately did not force it. The
    "two representations coexist" question (raised by the slice02/03 bubble-ups) is
    now live for the plan layer too, with the same interim answer.
  - **A7/A8 import is a deferred coordination call.** The post-MVP arcs are not
    self-hosted (0 slices, owned separately). When they become real, whoever closes
    them imports them — or a future slice imports them as `planned` stubs if the
    full roadmap should be represented in `nodes/`. Flagged for the project owner.

**3. Silent-drop diff at the arc altitude.** Nothing the arc-plan scoped for A-4 /
A-8 is missing. The `[gates.*]`/`odm.toml` addition, the A7/A8 scope call, and the
`decomposed`-affirm decision are disclosed here so the arc-plan records them.

**4. Reusable finding.** *When the filesystem already encodes the structure, don't
parse prose for it.* The `design-vX.Y.Z/` tree was the project→arc→slice tree
verbatim; the adapter read structure from directories and only *status* from
content. Any future importer over a well-structured tree should lean on the
structure and reserve parsing for the genuinely unstructured signal.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-4 row evidence + A-8 mechanism-complete + a v1.7
version-history entry), not only here.
