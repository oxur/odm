---
id: 01KYNDTQ6SM22NCEXWSSF2D5E2
number: 58837404
type: slice
schema: slice/v1.1
name: Slice 04 (Migration Fidelity) — Scope + repair capability (plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice04-scope-repair-capability/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
status:
  built:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  planned:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  tested:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
---
# Slice 04 (Migration Fidelity) — Scope + repair capability (plan-of-record)

> Refs: **ODD-0025 §2.8** (update-in-place repair — the authority) + §2.1/§2.2 (the fidelity gate +
> `source`); `../arc-plan.md` v1.6 (s04 row + the v1.6 decisions); `../design-notes.md` §3 (F8,
> F11, F12). `depends_on:` s03 (the fidelity primitives). **Executes** the ODD-0020 schema-minor
> bump s03 deferred.
>
> **This is a *capability* slice — fixture-verified, NO live mutation.** The live run (repair the
> real 44 stubs, import the 6 missing arcs) is **s05**. Building the destructive op and firing it are
> deliberately separate slices (operator call, v1.6).

## Goal

Build — and prove on fixtures — the capability to (a) **import every arc/slice dir** (the
`MAX_MVP_ARC` cap removed; named arcs get a `number` handle) and (b) **repair a stub node
update-in-place** (real verbatim body + `source`, preserving id/edges/status, through the s03 hard
body-hash gate), plus (c) **execute the ODD-0020 schema-minor bump** (`v1.0 → v1.1`). **Done when**
all three are implemented in `odm-migrate`/`odm-core`, the removed cap is reflected in the shared
`coverage.rs` usage, and fixture tests prove import-all + update-in-place repair + the schema bump —
with the live `.worktrees/odm` corpus untouched.

## Scope

**In:**

- **Remove the `MAX_MVP_ARC` / `arc_in_scope` cap** (`selfhost.rs:35/79`). `self_host` imports
  **every** arc dir it finds — numbered (A1–A6, `arc07`/`arc08`) and named (`arc-store-home`,
  `arc-release-hardening`, `arc-llm-command-surface`, `arc-migration-fidelity`). `arc_in_scope` is
  **shared with `coverage.rs:512`** — update that call site so the representation detector treats
  all real arc dirs as in-scope (removing, not just widening, the cap). Prefer deleting the
  predicate over hardcoding a higher constant (v1.6 decision — no new lock-in).
- **Named-arc `number` handle assignment.** A named arc dir has no `arcNN` coordinate, but `number`
  is a required `u32` and a non-structural handle (not identity/order — v1.6 F12). Assign named
  arcs (and their slices) a **deterministic, collision-free** handle — e.g. a band above the
  numbered arcs (A1–A8 occupy 1100–1800; named arcs take 1900, 2000, … in dir-sort order; slices
  offset within). Numbered arcs keep their derived numbers. **State the rule in code + report it.**
- **Update-in-place repair op** (ODD-0025 §2.8): given an existing node identified as a **stub**
  (body is a lone H1 — reuse s01's stub predicate), match it to its source doc by structural
  coordinate, read the **verbatim** source body + build the `source` record (reuse
  `fidelity::build_source`), and **write both into the *same* node** — preserving `id` / `edges` /
  `status` / `number` — via `Store::persist` (which overwrites by id; **no `delete` needed**),
  through the hard body-hash gate (`fidelity::verify_body_hash`). Populate `author`/`version` for
  document nodes.
- **Execute the ODD-0020 schema-minor bump** (`SchemaVersion::CURRENT` `v1.0 → v1.1`, `schema.rs`).
  New and repaired nodes stamp `<type>/v1.1`; **existing `v1.0` nodes remain valid** (the added
  fields are optional/additive — forward-compatible); update `is_newer_than_current` and any `v1.0`
  assertions so the workspace contract holds. Apply the matching ODD-0020 §4 edit (record v1.1 as
  current — closes the s03 deferral).
- **Tests (fixtures/temp stores only):** import-all (a fixture plan-set carrying named + `arc07` +
  A1–A6 arcs → every arc/slice imported, named arcs handled, no collision); update-in-place repair
  (a fixture stub node → repaired body verbatim, `source` populated, `id`/`edges`/`status`/`number`
  **preserved**, `v1.1` stamped); the gate on repair (a mismatched body errors); schema bump
  (a `v1.0` node still validates; a new node stamps `v1.1`; `is_newer_than_current` treats `v1.1` as
  current).

**Out:** the **live run** — any mutation of `.worktrees/odm` (s05); minting the supporting-doc
`artifact` nodes + wiring doc-coverage into `check` (s06); synthesis (s07); re-pointing the live
`context.json` (s05 — the capability may include the helper, but firing it is s05); **re-keying
idempotence on `source`** (a noted future refinement — s04 keeps coordinate-matching, which
ODD-0025 §2.8 requires anyway since stubs have no `source` yet).

## Verification

`cargo test -p odm-migrate` (import-all, update-in-place repair, gate-on-repair) and
`cargo test -p odm-core` (schema bump, forward-compat) green; `cargo clippy --workspace
--all-targets -- -D warnings`; no `unsafe`; coverage ≥ 90% (line) on changed modules; the
import-all test demonstrates a named arc + `arc07` imported with handles; the repair test
demonstrates id/edges/status **preserved** across a stub→real-body rewrite. **Live store untouched**
(0 nodes gain `source:`; `.worktrees/odm` byte-identical). Cargo rows `attested`→`reproduced`-on-CI.

## Exit

`ledger.md` closed; CDC-verified (`cdc-verification.md`). The scope-removal + update-in-place repair
+ schema bump exist and are fixture-green, so **s05** can fire them on the live corpus. On close,
bubble up to `../arc-plan.md` (MF-2/MF-3/MF-5 plan against this capability; the schema bump closes
the s03 carry-forward).
