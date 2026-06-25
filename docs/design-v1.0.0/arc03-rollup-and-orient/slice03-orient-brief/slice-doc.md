# Slice 03 (Arc 03) — orient / brief + bare-`odm` (plan-of-record)

> Refs: ODD-0013 §4.1 (orient + derived-order queries), §7 (command surface — bare
> `odm` orients, never bare-errors), §6 (the rollup model orient consumes); arc-plan
> D-1/D-1a/D-1b (vision source + rendering + deferred escalation), Q-A3-2 (drift
> placeholder); slice02 `cdc-verification.md` rulings 2 (soft-sat ⚠ on the ready
> frontier) + 3 (surface `check` integrity inline); ODD-0015 §2 (MVP DoD: full
> situational awareness from `odm orient` alone). `depends_on:` slice02 (the `Rollup`
> model), arc01 slice05 (`use`/`context`), slice01 (the `oxur-odm` `assert_cmd` suite,
> reused for the bare-`odm` test).
>
> **Why this slice exists:** the rollup (slice02) is the full structural view; `orient`
> is the *cheap, actionable read* a fresh session leads with. This slice composes
> `orient`/`brief` over the rollup model and makes bare `odm` orient — the single most
> important LLM affordance (0015 §2). It is the **last capability slice of the MVP**
> (slice04 adds `--json` + polish).

## Goal

Compose `odm orient` (and its alias `brief`) over the slice02 `Rollup` model so one
cheap call leads with **vision → current focus → ready/blocked → drift**, and make
bare `odm` run it. **Done when** a fresh session reaches full situational awareness
from `odm orient` alone (0015 §2): it sees the program vision, the current arc, what's
ready (with soft-evidence flags) and blocked (with reasons), any structural integrity
break, and a drift placeholder — and bare `odm` never bare-errors.

## Scope

**In:**

- **`orient` composition** (`odm-cli`, e.g. `orient.rs`). Reuse `Rollup::assemble`
  (D-3) — **do not re-derive**; render a *terse* orient view (distinct from the full
  `ROLLUP.md`) in order:
  1. **Vision** — lead with the current project's `name` + the vision section of its
     body (D-1; extraction rule below). Load the project's `Document` (frontmatter +
     `.body()`).
  2. **Current focus** — the current arc (from `.odm/context.json` via `Context`),
     with its status vector.
  3. **Ready / blocked** — from the model: the ready frontier with the **soft-sat ⚠**
     surfaced (`ReadyNode.soft`, slice02 ruling 2); blocked nodes with named reasons.
  4. **Integrity** — surface `check` findings inline (reuse the existing aggregation),
     at least every **Error** (orphan, cycle-without-tear) so a structural break is
     *unmissable* (slice02 ruling 3 / MVP DoD).
  5. **Drift** — "not yet tracked (A5)" (Q-A3-2).
- **Vision extraction rule (D-1a — settled here).** From the project body: if a
  heading matching `# Vision` (case-insensitive, ATX) exists, take that section (to the
  next same-or-higher heading); else take the **lead section** (body text before the
  first ATX heading). Truncate to a line budget (~15 non-empty lines) with a
  `… (full vision: odm show <project>)` continuation marker when cut. Always lead with
  the project `name`.
- **Bare `odm` orients** — no subcommand dispatches to `orient`; **never bare-errors**.
- **No-current-project fallback** (never bare-errors): no project in the corpus →
  affordance to `odm new project <name>`; exactly one project → orient on it; multiple
  with no `context` selection → list them + prompt `odm use project <ref>`.
- **`brief`** — an alias of `orient` (identical output for now).

**Out:** `--json` for orient/rollup (slice04 — pins the schema); the `.odm/` cache
(A4); `reconcile`/drift computation (A5); deferred-node surfacing (A5, Q-A3-1); a
distinct terser `brief` mode (alias only for now); the vision *document node*
escalation (D-1b, deferred).

## Verification

`cargo test -p odm-core -p odm-cli` + the `oxur-odm` `assert_cmd` suite green; a corpus
with a project + arc + ready/blocked/soft-sat/orphan yields an `orient` that leads with
vision and surfaces each section; bare `odm` orients; the no-project fallbacks all exit
0; clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line) for odm-core + odm-cli. Rows
in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+). `odm orient` gives
full situational awareness from one call and bare `odm` orients — **the MVP capability
is feature-complete; slice04 (`--json` + polish) closes the arc.**
