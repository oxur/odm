---
number: 17
title: "Interop — projection out, reference-and-reconcile in"
author: "topological sort"
component: All
tags: [interop, federation, export, reconcile, cross-team, evangelism]
created: 2026-06-20
updated: 2026-06-20
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# Interop — projection out, reference-and-reconcile in

> Design for how `odm` interoperates with teams that have *not* adopted the
> tooling. Refs: ODD-0013 (architecture), ODD-0015 (breakdown), ODD-0011/0016
> (the SoT-only-manages-what-it-claims and relayed-summary lessons), ODD-0001
> (the project-x failures this must not re-create across a team boundary).

## 1. Thesis: federate, don't convert

Two observations drive this:

1. **Legibility drove adoption.** Other teams picked up the methodology because
   the artifacts were legible *from the outside* — the PRs, the
   plan/cc-prompt/ledger/reconciliation diffs, and the `project/<arc>/<slice>/`
   tree. The maintenance burden and the evangelism value were the same property
   seen from two sides.
2. **Cross-team dependencies are the highest-risk interop surface.** "Their epic is
   done, so our dependent slice is unblocked" is project-x's marquee failure
   (ODD-0001 F2/G3 — a wrong *relayed summary* produced the prod 503) replayed
   across a team boundary.

So we do **not** translate between methodologies (lossy, fork-prone). We do two
separate, narrower things:

- **Export = projection *out*** — publish a legible, one-way, honestly-lossy view
  of our graph in another team's terminology.
- **Reference-and-reconcile = *in*** — represent other teams' work as *external
  nodes* (foreign keys) whose status we **reconcile, never author**, and wire
  cross-team dependencies as ordinary edges to them.

Export is a **renderer**; "import" is a **reconciler**. Each team stays
authoritative over its own source of truth.

## 2. Part A — Export (projection out)

`odm export` renders the graph and its legible artifacts through a
**vocabulary-mapping config** that maps our model to a target's terms
(project/arc/slice → epic/story/task; our gate vectors → their statuses).

Properties (all load-bearing):

- **One-way and generated.** Output is stamped "generated — do not edit." We never
  read it back (no round-trip → no fork). If they edit it, that is their fork, not
  our drift.
- **Honestly lossy.** Where our richer model collapses to theirs — a multi-gate
  vector (`built✓ tested✓ verified-live✗`) to a single status — the export states
  the collapse and the rule used. No silent fidelity loss.
- **Bounded targets.** Emit to a small set of *neutral* forms — a JSON interchange
  and a Markdown/templated view — **not** a bespoke emitter per team's directory
  layout. A target is a (vocabulary map + template), not custom code.
- **Reuses the rollup.** Export is a projection of the same generated rollup
  (ODD-0013 §6); it adds a mapping + a renderer, nothing more.

This is also the **evangelism engine**: `odm export` can publish exactly the
artifacts that made others adopt us (the tree, the rollup, the ledger/reconcile
views) as a read-only showcase. It needs only the graph + rollup, so it can land
right after Arc A3.

## 3. Part B — Reference-and-reconcile (in)

Replaces the dangerous "import their files" idea. Three pieces:

### 3.1 The `external` node type
A foreign key, not a copy: `{ system, external_id, label, status_probe }`. We
control its shape, so their format never leaks into our SoT. It participates in the
graph like any node.

### 3.2 Cross-team edges
Our nodes wire ordinary `depends_on` / `blocked_by` edges **to** the external node.
Cross-team work then shows up in `next`, `blocked`, `path`, and the staleness guard
**for free** — cross-boundary dependencies become first-class, queryable data
(closing ODD-0001 B1: "dependencies were prose").

### 3.3 Reconciled status with evidence levels (the safety mechanism)
An external node's status is **never authored — only reconciled**, by a thin
per-source **adapter/probe** that reuses the `odm-reconcile` machinery (Arc A5):
read their tracker API, one known field in their `epics/*.md`, a URL, or — worst
case — a manual update. Adapters are **status-only, minimal-extraction, opt-in**.

Crucially, the reconciled status carries an **evidence level** (ODD-0001 D3:
`asserted | attested | reproduced | reconciled`). And: **a `depends_on` satisfied
only by a low-evidence external status surfaces that in `next`/`blocked`** — so a
relayed "it's done" can *never silently* unblock critical work. This is the
explicit guard against re-creating the 503 across a team boundary, and it
generalizes a latent improvement to the internal model (see §6, Q-2).

## 4. Why this is safer than convert/import

- **SoT integrity** (ODD-0011/0016, the Terraform lesson): we only claim what we
  can manage. We manage the *external node* (our object) and a *probe*; we do not
  ingest their documents and inherit their ambiguity/drift.
- **No fork trap:** export is one-way and labeled; we never reconcile against our
  own exported copy.
- **No cross-team 503:** evidence-leveled, reconciled status replaces relayed
  belief; low-evidence satisfaction is visible, not silent.
- **Bounded cost:** per-source adapters are bespoke, but small, status-only, and
  opt-in — versus an open-ended general importer of whole foreign documents.

## 5. Crate placement & scheduling (proposed)

- **Export:** a renderer over the rollup — `odm-cli` + a small projection module
  (promote to `odm-export` only if it earns it). Depends on A3. → its own arc,
  **after A3**.
- **Reference-and-reconcile:** the `external` node type in `odm-core`; adapters as
  `Probe` impls in `odm-reconcile`. Depends on A5 (the reconciler). → its own arc,
  **after A5**.

Neither is in the MVP (A1–A3). They slot into ODD-0015 as two post-MVP arcs
(provisionally after A5/A6).

## 6. Non-goals & open questions

**Non-goals:** methodology conversion or two-way content sync; whole-document
import; round-trip of exported artifacts; supporting arbitrary nested target
directory structures (targets are vocab-map + template, not custom emitters).

**Open:**
- **Q-1** Export target set for v1 — JSON interchange + one Markdown template?
  Which terminologies first (Jira epic/story, GitHub Projects)? Decide at the
  export arc.
- **Q-2** *(resolved — adopted)* Evidence-leveled satisfaction applies internally
  too: an internal `depends_on` satisfied only at low evidence is soft-satisfied
  and surfaced (not silently green). Folded into ODD-0013 §4.4 (v1.5): the evidence
  ordering, a default `reproduced` threshold, and min-propagation along dependency
  chains. Built in Arc A2 (the derived-order & satisfaction slice).
- **Q-3** Adapter interface: is the external `status_probe` exactly the A5 `Probe`
  trait, or a thin specialization? Lean: the same trait, so cross-team reconcile
  and desired-fact reconcile share one mechanism.
- **Q-4** Identity of external nodes: do they get a normal ULID `id` (yes) plus the
  `{system, external_id}` as the reconcilable key (yes)? Confirm at the arc.
