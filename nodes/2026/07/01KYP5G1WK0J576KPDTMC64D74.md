---
id: 01KYP5G1WK0J576KPDTMC64D74
number: 519035200
type: artifact
schema: artifact/v1.1
name: 'cc-prompt — RH C-4: command-surface cleanup + the ODD-0023 reorg (one pass)'
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/cc-prompt-c4-command-surface.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# cc-prompt — RH C-4: command-surface cleanup + the ODD-0023 reorg (one pass)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-4 · **Covers:** `F-10`/`F-11`/`F-12`/`F-13`
> (renames) **folded with ODD-0023** (the `node`/`store` three-tier reorg + deprecation aliases) so
> the surface churns **once**. **Companion authority:** `arc-llm-command-surface/odm-command-inventory.md`
> (already rewritten to the three-tier shape) — implement it, and keep it in sync. **Sequenced after**
> arc-store-home (the `store` group already exists) and C-5 (`migrate` already absorbed `self-host`).

## Why one pass

ODD-0023 §6 is explicit: fold the C-4 renames into the reorg so the surface, `--help`, `--json`, and the
LLM inventory change **once**, not twice. The `store` group is already born (arc-store-home); `self-host`
is already gone (C-5). So C-4's job is: the four renames, the **`odm node` group**, top-level workflow
verbs, and one release of deprecation aliases.

## Phase 0 — ratify ODD-0023 first (model/decision before code)

ODD-0023 is **Draft** with `§5` debatable classifications still open. Land those decisions (operator
ratifies; move ODD-0023 → **Accepted**) **before** the code, same model-first rule C-2 followed. The
opens, with CDC recommendations:

- **`use` — KEEP it (revised recommendation).** ODD-0023 §5 tentatively *retired* `use` as redundant.
  **C-5 changed the ground:** `use arc X` now sets the CURRENT FOCUS that `orient` reads back (and the
  C-5 dogfood *fixed* the use/orient root split precisely so this works). `use` has a real workflow now
  — "set what I'm focused on." **Recommend keep**, as a top-level verb (it acts on the whole-graph
  session state, like a cursor). Do **not** retire it. *(This supersedes ODD-0023 §5's tentative call —
  record the reversal in the ODD.)*
- **`context` → `project`** (F-11): rename; `project` stays **top-level** (a "where am I in the plan"
  query), current project by default, `--name` for others.
- **`check` + `reconcile`:** keep `validate` as a trivial **alias** of `check`. Do **not** make `check`
  always reconcile — `reconcile` runs side-effecting probes; **recommend opt-in `check --reconcile`**
  (or a cheap subset), not every `check` paying probe cost. Decide the semantics.
- **`list` scope:** `odm node list` for nodes; graph-wide views stay `orient`/`rollup`. (Bare `odm list`
  survives as a deprecated alias for the window — below.)

## The renames (F-10…F-13)

- **F-10 — quiet-idempotent `new`:** on re-run against an existing node, **warn** (not print full
  details): *"<type> exists; for details run `odm project --name=<name>`"* (depends on F-11's `project`).
- **F-11 — `context` → `project`** (+ `--name`, current default). *(Also the top-level placement above.)*
- **F-12 — `path` → `chain`** ("the critical chain / X→Y path"; avoids "filepath").
- **F-13 — `rollup`:** help must not hardcode `ROLLUP.md`; support **md and json** output + `--out
  <name>` / `--format={md|json}` (defaults `md` / `ROLLUP`).

## The reorg (ODD-0023 three tiers)

- **Top-level (workflow / whole-graph verbs):** `orient` (alias `brief`), `rollup`, `next`, `blocked`,
  `chain` (F-12), `check` (+`validate` alias), `reconcile`, `migrate`, `project` (F-11), `use`, `help`.
- **`odm node <cmd>` (node entity management):** `new` (F-10), `show`, `list`, `rename`, `retire`,
  `supersede`, `link`/`unlink`, `set-gate`, `tear`, `decomposed`. *(The LLM-arc additions —
  `history`/`diff`/`info`/`search` — are NOT this chunk; they land under `node` in that arc.)*
- **`odm store <cmd>` (store lifecycle):** `init`, `rename`, `sync` — **already shipped**
  (arc-store-home). C-4 touches it only if the reorg needs consistency (e.g. help grouping); don't
  re-implement it.

## Deprecation aliases (ODD-0023 §6 — not a hard cut)

Keep every old top-level spelling as a **hidden deprecated alias** for **one release** (e.g. `odm show`
→ `odm node show`, `odm list` → `odm node list`), emitting a one-line *"moved to `odm node show`"*
notice to **stderr** (data stays clean on stdout), then remove next release. This protects scripts,
muscle-memory, and the LLM inventory consumers through the transition.

## `--json` and `--help` reflect the new structure

Both the machine path (`--json`) and the LLM's self-doc (`--help` / the inventory) must show the tiers.
The inventory is already rewritten — reconcile the built `--help` against it (they must agree). **Not**
in this chunk: F-21 (`--json` dates → LLM arc).

## Acceptance / ledger

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- **Renames:** `odm project`/`odm chain` work; `odm context`/`odm path` are deprecated aliases emitting
  the move notice; `new` on an existing node warns + points at `project` (no full dump); `rollup`
  supports md/json + `--out`, help doesn't hardcode `ROLLUP.md`.
- **Groups:** `odm node <cmd>` dispatches all the node verbs; top-level keeps the workflow verbs; old
  top-level node spellings work as hidden deprecated aliases with a stderr notice; `--help` shows three
  tiers.
- **`use` kept**, `orient` still reads the focus it sets (regression against the C-5 fix — verify on the
  self-hosted store, not just a scratch repo, since that's where the root split hid).
- **Inventory ↔ `--help` agree**; `odm check` green; the corpus is unaffected (a surface change moves no
  nodes).
- ODD-0023 **Accepted** (with the `use`-keep reversal recorded) **before** the code. F-10…F-13 +
  the ODD-0023 opens dispositioned in the bubble-up (RH-4).

## Decisions to confirm at kickoff

1. **`use` — keep** (CDC-recommended, given C-5) vs. ODD-0023 §5's tentative retire. **Confirm keep.**
2. **`check`/`reconcile`** semantics: `validate` alias (yes) + `check --reconcile` opt-in (recommended)
   vs. check-always-reconciles.
3. **Alias window:** one release of hidden deprecated aliases, then remove (recommended) vs. permanent
   bare-form shortcuts for the very common verbs (`odm show`).
4. **Group help ergonomics:** does bare `odm node` / `odm store` print the group's subcommands (yes,
   recommended)?

## Method / housekeeping

- One branch (e.g. `rh-c4-command-surface`, off the C-5 tip on `release/1.0.x`); one mergeable change;
  five-iteration cap. CC on local 1.85+; cargo rows → CI. Reflexive check (RH-6/RH-7): run
  `odm list`/`orient`/`project`/`chain` on the **self-hosted store** and confirm the surface is coherent
  + themed + the inventory matches `--help`.
- Bubble up to `arc-plan.md` (RH-4; ODD-0023-Accepted note; the `use`-keep reversal; alias-window
  decision; silent-drop diff). CDC verifies (`cdc-verification.md`) — inventory-vs-`--help` parity + the
  `use`/`orient` regression on the real store.
- **Not in this chunk:** C-6 (check-hardening: G-2 + G-3 + L-3b), C-8 (F-19 normalized status), the LLM
  arc's `node` verbs + F-21. Those keep their homes.
