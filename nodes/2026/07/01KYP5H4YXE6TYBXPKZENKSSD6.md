---
id: 01KYP5H4YXE6TYBXPKZENKSSD6
number: 23
type: design
schema: design/v1.1
name: Command-surface reorg — `node` / `store` groups + top-level workflow verbs
created: 2026-07-26
updated: 2026-07-26
tags:
- cli
- command-surface
- ergonomics
- subcommands
- node
- store
component: odm-cli
author: Duncan McGreggor
version: '1.1'
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design/04-accepted/0023-command-surface-node-store-groups.md
  class: odd
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
status:
  accepted:
    reached: 2026-07-26
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-26
  draft:
    reached: 2026-07-26
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-26
  revised:
    reached: 2026-07-26
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-26
  under-review:
    reached: 2026-07-26
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-26
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

`orient` · `rollup` · `next` · `blocked` · `chain` · **`validate`** · **`check`** · `reconcile` ·
`migrate` · `project` (was `context`) · **`use`** · `help`. These query or advance the whole store; an
LLM or human "operates the plan" through these.

### `odm node <cmd>` (node entity management)

`new` · `show` · `list` · `rename` · `retire` · `supersede` · the **edge** mutators (`link`/`unlink`,
`tear`, `depends-on`/`blocked-by`/…) · the **gate** mutators (`set-gate`) · and the LLM-arc additions
(`history`, `diff`, `info`, `search`). CRUD + relationships + gate advancement on a specific node.

### `odm store <cmd>` (store lifecycle)

`init` · `rename` · `sync` (arc-store-home) · possibly `status` (where/what is my store). Manages the
store *container* — distinct from the nodes inside it.

## 5. The debatable classifications — **decided** (operator, 2026-07-26)

All five are settled. Two reverse what v1.0 tentatively recommended; both reversals are recorded
below rather than quietly rewritten, because the reasoning is the useful part.

- **`use` — KEPT** (reverses v1.0's tentative *retire*). v1.0 called it redundant with `project`.
  **RH C-5 changed the ground:** `use arc X` now sets the CURRENT FOCUS that `orient` reads back, and
  C-5's dogfood fixed the use/orient root split precisely so that works. It has a real workflow —
  "set what I'm focused on" — and it acts on whole-graph session state, like a cursor. **Top-level.**
- **`context` → `project`** — RH C-4's rename; `project` stays **top-level** (a "where am I in the
  plan" query), current project by default, `--name` for others.
- **`validate` / `check` / `reconcile` — the pure command gets the honest name** (reverses v1.0's
  "`validate` is a trivial alias of `check`"):

  | Verb | Does | Side effects |
  |------|------|--------------|
  | **`validate`** | graph validation only — schema, links, cycles, recomposition, order, staleness. *This is what `check` does today.* | none |
  | **`reconcile`** | the probe pass only — re-run volatile probes, report drift, re-stamp `last_checked`. *Unchanged.* | yes (shell/file probes) |
  | **`check`** | **`validate`, then `reconcile`** — the composite "is my plan actually true?" | yes, via reconcile |

  v1.0 framed this as *"should `check` always reconcile?"* and answered *"no — make it opt-in via
  `check --reconcile`"*. The operator's resolution is better: the cheap, pure operation deserves its
  own name rather than being the default mode of a verb, and `check` should mean *go and look*, not
  *statically validate, unless you pass a flag*. `validate` connotes static checking; `check`
  connotes verification against reality. Nobody has to remember a flag to get the cheap path — they
  ask for the cheap thing by name.

  **`check` stops at `validate` errors.** If the graph is structurally broken — dangling edges,
  cycles, schema violations — probe results are noise layered on a known-bad state, and probes cost
  time and have side effects. Hard errors: report and exit 1 without probing. *Warnings* fall
  through to reconcile (they do not fail the run without `--strict`). Exit code is the worse of the
  two phases, in the existing vocabulary: `0` clean / `1` violations-or-drift / `2` error;
  `--strict` promotes warnings in both halves.

  **This is a behaviour change to an existing verb under the same name** — `odm check` was pure and
  is now side-effecting. That is signalled loudly in the machine contract rather than silently: see
  §6a.
- **`self-host`** — folded into `migrate` (RH C-5, landed); does **not** survive as its own verb.
- **`next` / `blocked` / `chain`** — graph *queries*; **top-level**. `path` → `chain` is C-4's rename.
- **`list` scope** (was §7) — **`odm node list`** for nodes; graph-wide views stay `orient`/`rollup`.

## 6. Migration & aliasing — **hard cut** (operator, 2026-07-26)

**Reverses v1.0's "deprecation window, not a hard cut".** The old top-level spellings of node verbs
(`odm show`, `odm list`, `odm new`, `link`, `set-gate`, …) simply **stop working**; there are no
aliases, hidden or visible.

The v1.0 argument for a window was protecting scripts, muscle-memory and LLM inventory consumers.
Weighed against the actual situation: odm is **pre-1.0 with a single operator**, its only consumer
is this project, and the inventory is a document that gets rewritten in the same pass. The cost the
window was insuring against is close to zero, and the cost it imposes is real and permanent-ish —
two spellings for every node verb, in `--help`, in the inventory, in every example, for a release,
plus the deprecation-notice machinery and its tests. **One true spelling, immediately.**

- **`--json` and help** must reflect the new structure (the machine path and the LLM's self-doc read
  from `--help`/inventory).
- Sequence: land after `arc-store-home` (so `store` has commands) and fold the **C-4 renames** into the
  same reorg pass so the surface churns **once**, not twice.

## 6a. The `--json` schema contract across the `check` rename

`check --json` today emits `schema: "check/v1"` for pure graph validation. After §5, anything named
`check` means something different, so the schema ids are **bumped loudly** rather than quietly
reused:

| Command | Schema | Note |
|---------|--------|------|
| `validate --json` | **`validate/v1`** | the payload that used to be `check/v1`, under its honest name |
| `check --json` | **`check/v2`** | composite: a `validate` section + a `reconcile` section |
| `reconcile --json` | `reconcile/v1` | unchanged |

The `v2` bump is deliberate signalling, not payload evolution. A consumer pinned to `check/v1` gets
a version it does not recognize — which is the correct outcome, because the command it is reading no
longer does what it did. Silently keeping `check/v1` on either the pure or the composite command
would let a machine consumer carry on believing it was reading the old thing.

## 6a. Interactions to keep aligned

- **RH C-4** (renames: `context`→`project`, `path`→`chain`, quiet `new`, `rollup` md/json) — do these
  *inside* this reorg, not before it, so there's one surface change.
- **LLM-command-surface arc** — its new node verbs (`history`/`diff`/`info`/`search`) land under
  `odm node`; its `--json` contract additions (F-21 dates) ride there.
- **`odm-command-inventory.md`** — rewritten to this three-tier shape as this ODD's companion.

## 7. Open questions — **resolved, except one**

- ~~**Group aliases / bare forms?**~~ **Resolved: none.** No permanent shortcuts and no deprecation
  window — §6's hard cut. `odm node show` is the only spelling.
- ~~**`list` scope**~~ **Resolved: `odm node list`**, with graph-wide views staying `orient`/`rollup`
  (recorded in §5).
- **`store status`** — still open. "Where is my store / is it synced" as a `store` verb, versus
  folding it into `orient`. **Not blocking C-4**: nothing in the reorg depends on the answer, and no
  such command exists to place. Left for whoever needs it.

**Group help ergonomics (decided):** bare `odm node` and bare `odm store` print their group's
subcommands rather than erroring — a group name is a reasonable question ("what can I do to a
node?"), and answering it is consistent with errors-as-affordances.

## 8. Boundaries

Naming/placement only — no change to the node model, gates, edges, or storage. Not the LLM-ergonomics
*content* (that's the LLM arc); this is the *shape* those commands hang on.

## 9. Version history

### v1.1 — 2026-07-26 — **Accepted**

Ratified by the operator at RH C-4 kickoff. Draft → **Accepted**; the §5 classifications and the §7
aliasing question are decided, and **three of them reverse v1.0's tentative recommendations**:

1. **`use` is kept**, not retired (§5). RH C-5 gave it a real workflow — it sets the CURRENT FOCUS
   `orient` reads back — which did not exist when v1.0 called it redundant.
2. **`validate` becomes the pure command and `check` becomes the composite** (§5), rather than
   `validate` being a trivial alias and reconcile being opt-in behind `check --reconcile`. The cheap
   operation gets its own name instead of being a verb's default mode; `check` means *go and look*.
   `check` stops at validate **errors** (probing a structurally broken graph is noise with side
   effects); warnings fall through. New §6a records the resulting `--json` bump —
   **`validate/v1`** and **`check/v2`** — because `check` changing meaning under the same name must
   be loud in the machine contract, not silent.
3. **Hard cut, no deprecation aliases** (§6, §7). v1.0's window insured against script/muscle-memory
   breakage that, for a pre-1.0 single-operator tool whose only consumer is this project, costs
   about nothing — while the window itself costs two spellings everywhere plus the notice machinery.

Also decided: `list` scope is `odm node list` (§5/§7); bare `odm node`/`odm store` print their
subcommands (§7). **Still open, non-blocking:** `store status` (§7).
Surfaced by: RH C-4 kickoff; the `use` reversal by RH C-5's dogfood cutover.

### v1.0 — 2026-07-26
Initial draft. Three-tier surface (top-level workflow verbs / `odm node` / `odm store`) with the
"whole-graph vs entity" organizing principle; the debatable classifications (`use`, `context`→`project`,
`check`+`reconcile`, `self-host`→`migrate`); a deprecation-alias migration folded into RH C-4 so the
surface churns once; sequenced after `arc-store-home`. Companion: the `odm-command-inventory.md` rewrite.
Surfaced by: Duncan's reorg proposal + the store commands needing a home.
