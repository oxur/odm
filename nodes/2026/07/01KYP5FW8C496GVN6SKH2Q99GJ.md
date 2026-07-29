---
id: 01KYP5FW8C496GVN6SKH2Q99GJ
number: 522630800
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 06 (Migration Fidelity): Live-run capability'
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice06-live-run-capability/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6S6MH11W8QX4XWGQWB
---
# CC Prompt — Slice 06 (Migration Fidelity): Live-run capability

Make the full migration-fidelity flow **invocable and correct** so the live run (s07) has a
CDC-verified entry point behind it. Two pieces: **(a)** unify the two `source`-backfill paths so
`source` is never stamped ungated or onto the synthesis project node by *any* path (CDC v2.1 finding);
**(b)** wire the repair/reconcile flow into **`odm migrate`'s existing self-host path** — today it runs
`self_host()` (import) only, never `repair()` (stub-repair + gated backfill), so the live run has no
entry point (v2.2 discovery). Prove it **end-to-end on fixtures** — no live mutation (the live run is
**s07**).

> **Start condition:** on `release/1.0.x`. **Fixture-only — do NOT touch `.worktrees/odm`** (F-8
> hard). If `1.0.x` isn't green, hold. **Why this slice:** s05 made `source` the identity key and
> generalized `repair()`, but left two gaps between here and a safe live run — an *ungated* second
> backfill site (`self_host`'s `to_populate` transition) that can stamp `source` onto the synthesis
> project node, and the fact that `repair()` **has no caller outside tests**, so nothing invokes the
> reconcile-before-import flow the live run needs. This slice closes both, fixture-proven, so s07 is a
> snapshot-dry-run-inspect-fire on a capability that's already verified.

## Read first
1. `slice06-live-run-capability/ledger.md` (10 rows) — the spec of "done."
2. `slice-doc.md`; `slice05-source-identity/cdc-verification.md` **§ Finding** (the two-path
   inconsistency — the exact shape of F-1); `../arc-plan.md` **v2.1/v2.2** history entries.
3. **ODD-0025** §2.1/§2.8 (the hard body-hash gate + update-in-place reconcile), **§2.3** (synthesis —
   why the project node is excluded from 1:1 `source`), §5 (coverage — arc scope, s08's job not this
   one's).
4. **`../arc-llm-command-surface/odm-command-inventory.md`** — the command surface. Note especially:
   `self-host` was **removed** as a spelling (C-4) and **folded into `migrate`** (C-5) — "one verb,"
   already idempotent and `--dry-run`-able. **This means: extend `odm migrate`, do not add a new verb.**
5. **The code you change:**
   - `crates/odm-cli/src/migrate.rs` — `self_host_inner` (~line 110). Today it calls
     `selfhost::self_host()` only. This is where `repair()` gets wired in, *before* the import.
   - `crates/odm-migrate/src/selfhost.rs` — the `to_populate` coordinate→source transition (~line 288,
     inside `self_host()`): the **ungated, non-project-excluding** second backfill site F-1 targets;
     `repair()` (~line 462): the gated, project-excluding reference path; `SelfHostReport`/`RepairReport`.
   - `crates/odm-migrate/src/fidelity.rs` — reuse `verify_body_hash` / `build_source` (do not re-implement).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `03-error-handling.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task
1. **Unify the two `source`-backfill paths** (F-1, F-5). `self_host()`'s `to_populate` transition
   currently clones the fm and stamps `source` **ungated** and **without the project exclusion** that
   `repair()` enforces (CDC v2.1 finding). Make there be **one** `source`-population policy:
   - **Preferred:** route the transition through the *same* gated, project-excluding reconcile logic as
     `repair()` — a body is verified against its source body via `fidelity::verify_body_hash` before
     `source` is added (drift → `BodyHashMismatch`, **not** stamped over), and the **synthesis project
     node is excluded** (body ≠ source, ODD-0025 §2.3). *Or* remove the transition's own backfill
     entirely and let the unified `repair()` own **all** `source` population. Either way, no path can
     add `source` ungated or give the project node a 1:1 `source`.
2. **Wire the flow into `odm migrate`'s self-host path** (F-2, F-3) — **not a new command**. In
   `self_host_inner`, run the **full flow in the correct order**: `repair()` **first** (reconcile
   existing — repair stubs + gated faithful-backfill, so every existing node carries `source`), **then**
   `self_host()` (import the missing arcs/slices), so the import's `by_source` key finds the
   already-reconciled nodes. Fold the repair counts into the rendered report. **Both arms already honor
   `Mode`/`--dry-run`** — keep that: a `--dry-run` previews repair/backfill/import counts and writes
   nothing. Confirm the flag surface (default-on vs an explicit `--repair`/`--reconcile` opt-in) against
   the inventory + `odm migrate -h` and **flag your choice** — don't silently pick.
3. **End-to-end fixture test** (F-4, F-6, F-7) in `odm-cli` tests: a fixture corpus carrying stubs +
   faithful non-stub nodes + a **synthesis-shaped project node** + the 6-missing-arc shape → run
   `odm migrate` on it → **every** stub repaired (verbatim body + `source`), **every** faithful node
   backfilled (gated), **every** missing arc/slice imported, `v1.1` stamped, `source` on every node
   **except** the excluded project node, `context.json` re-pointed, **no duplicate on re-run**
   (source-keyed, s05), and **`--dry-run` mutates nothing** (store byte-identical before/after).
4. **Project-node-by-any-path test** (F-5) in `odm-migrate` tests: after the full flow on a fixture
   whose project node is synthesis-shaped, that node carries **no** 1:1 `source` — reached via the
   transition path, not just via `repair()`. This is the exact case the v2.1 finding flagged.

## Constraints (flag, don't silently change)
- **Fixture-only. Do NOT mutate `.worktrees/odm`** (F-8 hard). The live run is **s07**.
- **One `source` policy** — after this slice no path stamps `source` ungated or gives the project node
  a 1:1 `source`. If unifying reveals a case where the gate legitimately shouldn't apply, that's a
  **finding to surface**, not a second ungated path to keep.
- **`odm migrate` is the entry point** — do not add a new verb (`self-host` was deliberately removed).
  Reuse `Mode`/`--dry-run`; reuse `fidelity`. Extend, don't fork.
- **Order is load-bearing:** `repair()`/reconcile **before** `self_host()`/import — flag it in a test
  comment so a future reader can't reorder it blind.
- No `unsafe`; typed errors (`thiserror`); coverage ≥ 90% (line, changed modules), target 95%.
- Don't pull **s07** (the live run — any real-store mutation), **s08** (`artifact` supporting docs +
  doc-coverage wired into `check`), **s09** (synthesis — the project node's re-cast; this slice only
  *excludes* it), or **s10** (the arc-close reconcile demo) forward.

## Deliverables
Green `cargo test -p odm-migrate` + `-p odm-cli` + clippy (`-D warnings`) + coverage; `ledger.md`
evidence per row (`attested`; cargo rows reproduce on CI/CDC); `closing-report.md` — per-row walk
**plus the v2.0 Bubble-up to the arc** (did s06 make the flow invocable and one-policy; what did wiring
it reveal the plan didn't anticipate — e.g. a report-shape or ordering wrinkle, a fixture the
transition path needed; the silent-drop diff vs `slice-doc.md`'s In/Out). Branch: `release/1.0.x` (the
arc's fast-forward pattern — no per-slice branch).

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap (+ split-escape if the CLI wiring and
the two-path fix won't both land in one comfortable context — land the **two-path fix + `repair()`
wired into `migrate`** first, split the exhaustive end-to-end fixture out, flag CDC). Your `done` is
*proposed-done* (`attested`) → CDC reproduces on the live machine + CI. On close, bubble up to
`../arc-plan.md` (v2.1 + v2.2 findings resolved; s07 fires the flow live).
