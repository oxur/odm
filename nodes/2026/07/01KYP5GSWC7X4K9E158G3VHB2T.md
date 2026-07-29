---
id: 01KYP5GSWC7X4K9E158G3VHB2T
number: 588223200
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 01 (Arc 05): `desired_facts` schema + `Probe` trait + shell probe'
created: 2026-06-30
updated: 2026-06-30
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice01-desired-facts-probe/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKM4PA275KRNSWQTWB
---
# CC Prompt — Slice 01 (Arc 05): `desired_facts` schema + `Probe` trait + shell probe

Arc 05's opener. Land the substrate the rest of A5 composes on: a node can **declare**
desired state (`desired_facts`), and code can **probe** one declared fact against reality
and get a typed **outcome** (holds / drifted / error). First probe impl: **shell**. No
runner, no `reconcile` command, no rollup wiring this slice — those are slices 02–04.

> **Start condition:** A4 is closed + CI-green (the index/cache arc). You are on a fresh
> branch off `main`: **`arc05-slice01-desired-facts-probe`** (not `main`).

## Read first
1. `slice01-desired-facts-probe/ledger.md` (7 rows — the contract) and `slice-doc.md`
   (same dir): scope in/out, the proposed YAML shape, the resolved trust model.
2. `../arc-plan.md` — the A5 capability, the Arc Ledger (this slice closes **A-1**), the
   resolved open questions (v1.3), and **the carried-in adapter-fidelity invariant**
   (you do *not* touch the index this slice — but read it so you know why slice02/03 will).
3. ODD-0013 §5.2 (desired-state facts + probes) for the design intent.
4. `crates/odm-core/src/frontmatter.rs` (how fields are declared + round-tripped; note the
   A4 slice02 serde-evolution lesson: always-serialized fields, no `skip_serializing_if`
   traps) and `crates/odm-index/Cargo.toml` (as the template for a new workspace crate).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design + error handling (the
  `Position`-carrying error convention), then trait design.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence at `attested`; per-row walk
  at close; write the **Bubble-up to the arc** section).

## Task
1. **`desired_facts` frontmatter field** (`odm-core`): optional list of `{ id, describe,
   probe }`; absent/empty default; stable round-trip; works on any node type incl. the
   project node. (F-1)
2. **Malformed → positioned error** (F-2): unknown probe `kind`, missing required field, or
   empty `id` is a build/parse error carrying `Position` — never a panic or silent drop.
3. **`Probe` trait + `ProbeOutcome`** (`odm-reconcile`, new crate): the three-way outcome
   `Holds | Drifted { expected, observed } | Error { reason }`. (F-3)
4. **Shell probe** (F-4): run the declared command; map exit (and optional stdout match) to
   the outcome — Holds on match, Drifted on divergence, Error when unrunnable. Test all
   three against real local commands.
5. **Document the trust model** (F-5): author-declared, local, user-privilege, no MVP
   sandbox — in the probe's doc comment.
6. **Scaffold `odm-reconcile`** (F-6): workspace member; `[workspace.dependencies]` +
   `[workspace.lints]`; no version literals; add to `[workspace] members` and the publish-
   order note in the root manifest / CLAUDE.md if present.
7. **Gates** (F-7): clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If a row is wrong/impossible, or the proposed YAML shape
  doesn't fit Rust/serde cleanly, raise an amendment — don't quietly diverge.
- **Stay in scope.** No probe-runner, no `file` probe, no `reconcile` command, no rollup/
  orient wiring, no index/`IndexRecord` changes. Adding any of those is scope creep — the
  slice boundaries are deliberate (one context, iteration headroom).
- **The three-way outcome is non-negotiable** — "couldn't check" must be distinct from
  "checked, drifted." No `bool`, no collapsing Error into Drifted.
- **Serde evolution:** a future probe `kind` (e.g. `file` next slice) must extend the spec
  without breaking the wire shape — explicit, always-serialized fields.

## Deliverables
The field + trait + shell probe + crate, with `ledger.md` evidence per row (at `attested`);
a `closing-report.md` — per-row walk **plus the v2.0 Bubble-up to the arc** (did slice01
deliver A-1's piece; what did it reveal the arc-plan didn't anticipate — especially
anything that sharpens the slice02/03 index-integration decision; the silent-drop diff).
Feature branch `arc05-slice01-desired-facts-probe`; not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-1) per LEDGER-DISCIPLINE v2.0 §A.
