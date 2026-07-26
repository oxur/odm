---
number: 23
title: "Command-surface reorg — `node` / `store` groups + top-level workflow verbs"
author: "Duncan McGreggor"
component: "odm-cli"
tags: [cli, command-surface, ergonomics, subcommands, node, store]
created: 2026-07-26
updated: 2026-07-26
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# Command-surface reorg — `node` / `store` groups + top-level workflow verbs

> **Drafting note.** Drafted by CDC 2026-07-26 from Duncan's proposal. The **`odm-command-inventory.md`**
> (`arc-llm-command-surface/`) is the surface authority and will be **rewritten to this structure** as
> the companion to this ODD. Sequenced **after `arc-store-home`** (so `store`'s commands exist to
> group) and **coordinated with RH C-4** (renames) + the LLM-command-surface arc (new node verbs).
> **ODD numbering:** G-1's ID-scheme ODD is also pending — this took **0023**; G-1 should take the
> next free number (confirm, to avoid an ODD-id collision).

## 1. Decision

Reorganize the odm CLI into **three tiers**:

- **Top-level = workflow / query verbs** that operate over *the store as a whole*.
- **`odm node <cmd>`** = entity management on **nodes**.
- **`odm store <cmd>`** = lifecycle of **the store itself**.

## 2. Problem

The surface is flat and growing. `arc-store-home` adds store commands (`init`, `rename`, `sync`) with
no home; the LLM-command-surface arc adds node verbs (`history`, `diff`, `info`, `search`); RH C-4
renames several. A flat top-level command list does not scale as **two entity families** (nodes,
store) each accrue CRUD-ish verbs — the top level becomes an undifferentiated pile, and there's no
principled place to put a new command.

## 3. The organizing principle (what makes the classification decidable)

**Top-level is for verbs that take the *graph as a whole* as their object** — orient yourself, roll it
up, ask what's next, check it, reconcile it, migrate into it. **The groups are for verbs that manage a
particular *entity*** — a node, or the store. The test for a new command: *does it act on the whole
graph (top-level), a node (`node`), or the store container (`store`)?* This is the same shape as
`git status`/`git log` (top-level) vs `git remote`/`git stash` (groups), and `kubectl` verbs vs
resources.

## 4. The three tiers

### Top-level (workflow / graph verbs)

`orient` · `rollup` · `next` · `check` (+ **`validate`** alias) · `reconcile` · `migrate` · `project`
(was `context`) · `help`. These query or advance the whole store; an LLM or human "operates the plan"
through these.

### `odm node <cmd>` (node entity management)

`new` · `show` · `list` · `rename` · `retire` · `supersede` · the **edge** mutators (`link`/`unlink`,
`tear`, `depends-on`/`blocked-by`/…) · the **gate** mutators (`set-gate`) · and the LLM-arc additions
(`history`, `diff`, `info`, `search`). CRUD + relationships + gate advancement on a specific node.

### `odm store <cmd>` (store lifecycle)

`init` · `rename` · `sync` (arc-store-home) · possibly `status` (where/what is my store). Manages the
store *container* — distinct from the nodes inside it.

## 5. The debatable classifications (decide here)

- **`use`** — set the active node/context. With `project` and the store model this may be **redundant**;
  tentatively **retire** unless a workflow needs it. Decide.
- **`context` → `project`** — this is RH **C-4's** rename; the two must agree. `project` stays top-level
  (a "where am I in the plan" query).
- **`check` calls `reconcile`** — the **`validate` alias** is trivial. But *check-always-reconciles* is a
  behaviour change: `reconcile` runs shell/file **probes** (side effects + latency) that `check` does
  not. **Recommend `check --reconcile` (opt-in)** or a *cheap* reconcile only — not every `check`
  paying probe cost. Decide the semantics.
- **`self-host`** — folds into `migrate` (RH **C-5**), so it does **not** survive as its own verb.
- **`next` / `blocked` / `chain`** — graph *queries*; top-level (they answer "what should I do"). `path`
  → `chain` is C-4's rename.

## 6. Migration & aliasing

- **Deprecation window, not a hard cut.** Keep the old top-level spellings as **hidden deprecated
  aliases** for one release (e.g. `odm show` → `odm node show`), emitting a one-line "moved to `odm node
  show`" notice, then remove. This protects any scripts/muscle-memory and the LLM inventory consumers.
- **`--json` and help** must reflect the new structure (the machine path and the LLM's self-doc read
  from `--help`/inventory).
- Sequence: land after `arc-store-home` (so `store` has commands) and fold the **C-4 renames** into the
  same reorg pass so the surface churns **once**, not twice.

## 6a. Interactions to keep aligned

- **RH C-4** (renames: `context`→`project`, `path`→`chain`, quiet `new`, `rollup` md/json) — do these
  *inside* this reorg, not before it, so there's one surface change.
- **LLM-command-surface arc** — its new node verbs (`history`/`diff`/`info`/`search`) land under
  `odm node`; its `--json` contract additions (F-21 dates) ride there.
- **`odm-command-inventory.md`** — rewritten to this three-tier shape as this ODD's companion.

## 7. Open questions

- **Group aliases / bare forms?** Do very common node verbs keep a top-level shortcut (`odm show` ≡
  `odm node show`) permanently, or only through the deprecation window? (Ergonomics vs. a clean surface.)
- **`list` scope** — `odm node list` vs a top-level `list` (it's arguably a graph query). Recommend
  `odm node list` for nodes, with the graph-wide views staying `orient`/`rollup`.
- **`store status`** — is "where is my store / is it synced" a `store` verb or folded into `orient`?

## 8. Boundaries

Naming/placement only — no change to the node model, gates, edges, or storage. Not the LLM-ergonomics
*content* (that's the LLM arc); this is the *shape* those commands hang on.

## 9. Version history

### v1.0 — 2026-07-26
Initial draft. Three-tier surface (top-level workflow verbs / `odm node` / `odm store`) with the
"whole-graph vs entity" organizing principle; the debatable classifications (`use`, `context`→`project`,
`check`+`reconcile`, `self-host`→`migrate`); a deprecation-alias migration folded into RH C-4 so the
surface churns once; sequenced after `arc-store-home`. Companion: the `odm-command-inventory.md` rewrite.
Surfaced by: Duncan's reorg proposal + the store commands needing a home.
