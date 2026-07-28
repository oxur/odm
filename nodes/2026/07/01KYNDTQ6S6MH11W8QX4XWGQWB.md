---
id: 01KYNDTQ6S6MH11W8QX4XWGQWB
number: 58837406
type: slice
schema: slice/v1.1
name: Slice 06 (Migration Fidelity) — Live-run capability (plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc-migration-fidelity/slice06-live-run-capability/slice-doc.md
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
# Slice 06 (Migration Fidelity) — Live-run capability (plan-of-record)

> Refs: `../arc-plan.md` v2.1/v2.2 (the two findings this slice resolves); `slice05-source-identity/cdc-verification.md` (the two-path finding); ODD-0025 §2.1/§2.8 (gate + reconcile), §2.3 (synthesis/project exclusion); `../design-notes.md` §3 (F8/F11/F12). `depends_on:` s05 (the source-keyed identity + generalized `repair`).
>
> **This is a *capability* slice — fixture-verified, NO live mutation.** The live run (fire it on the real corpus) is **s07**. This slice makes the live run *possible* and *safe to invoke*.

## Goal

Make the full migration-fidelity flow **invocable and correct** so s07 can fire it on the real store:
(a) **fix the two-path `source`-backfill inconsistency** (CDC v2.1 finding) so `source` is never
stamped ungated or onto the synthesis project node by *any* path, and (b) **wire the repair/reconcile
flow into the existing `odm migrate` self-host path** — the flow currently has **no entry point**
(v2.2 discovery: `odm migrate`'s self-host dispatch calls `selfhost::self_host()` only; `repair()`'s
only callers are tests). **Done when** `odm migrate` on the self-hosted corpus runs the whole flow —
reconcile existing (repair stubs + gated faithful-backfill) **then** import missing arcs, `v1.1` +
`source`/`author`/`version` + `context.json` — `--dry-run`-able (the flag already exists on
`migrate`), end-to-end fixture-proven, with the live corpus untouched.

## Scope

**In:**

- **Unify the two `source`-backfill paths** (v2.1 finding). `self_host`'s coordinate→source transition
  (`to_populate`, ~`selfhost.rs:288`) currently clones the fm and stamps `source` **ungated** and
  **without the project exclusion** `repair()` enforces. Route it through the **same gated,
  project-excluding reconcile logic** as `repair()` (or remove the transition's own backfill and let
  the unified reconcile own *all* `source` population): a body is verified against its source through
  `fidelity::verify_body_hash` before `source` is added (drift → error, not stamped over), and the
  **synthesis project node is excluded** (body ≠ source — ODD-0025 §2.3, s09). After this there is
  **one** `source`-population policy, not two.
- **Wire the repair/reconcile flow into `odm migrate`'s self-host path** — do **not** add a new verb.
  The inventory (`arc-llm-command-surface/odm-command-inventory.md`) is explicit: `self-host` was
  *removed* as a spelling (C-4) and folded into `migrate` (C-5) — "one verb," already idempotent and
  `--dry-run`-able. Today `odm migrate` → `self_host_inner` (`odm-cli/src/migrate.rs`) calls
  `selfhost::self_host()` *only*; `selfhost::repair()` (the gated stub-repair + faithful-backfill) has
  **no** caller outside tests. Extend that path so `odm migrate` on the self-hosted tree runs the
  **full flow in the correct order** — `repair()` first (reconcile existing: repair stubs + gated
  faithful-backfill, so every existing node carries `source`), **then** `self_host()` (import the
  missing arcs/slices), stamping `v1.1` and populating `source`/`author`/`version`, and re-pointing
  `context.json`. Both arms already honor `Mode`/`--dry-run` (preview repair/backfill/import counts,
  write nothing). Confirm the exact flag surface (default-on vs an explicit `--repair`/`--reconcile`
  opt-in) against the inventory + `odm migrate -h` before committing — but the entry point is
  `odm migrate`, not a new command.
- **End-to-end fixture test** of the whole flow: a fixture corpus carrying stubs + faithful non-stub
  nodes + a synthesis-shaped project node + the 6-missing-arc shape → run the command → every stub
  repaired (verbatim body + `source`), every faithful node backfilled (gated), every missing arc/slice
  imported, `v1.1` stamped, `source` on every node **except** the excluded project node, **no
  duplicate on re-run** (source-keyed), and **`--dry-run` mutates nothing**.

**Out:** the **live run** — any `.worktrees/odm` mutation (s07); minting the `artifact` supporting docs
+ wiring doc-coverage into `check` (s08); the **synthesis** re-cast of the project node (s09 — this
slice only *excludes* it from 1:1 `source`, it does not synthesize it); the arc-close reconcile
demonstration (s10).

## Verification

`cargo test -p odm-migrate` + `-p odm-cli` (the two-path gate/exclusion, the CLI command, the
end-to-end fixture flow, `--dry-run` no-op, re-run idempotence) green; `cargo clippy --workspace
--all-targets -- -D warnings`; no `unsafe`; coverage ≥ 90% (line) on changed modules; the end-to-end
test demonstrates the full flow on a fixture corpus, and a test asserts the **project node receives no
1:1 `source` by any path**. **Live store untouched.** Cargo rows `attested`→`reproduced`-on-CI.

## Exit

`ledger.md` closed; CDC-verified. The flow is one gated policy, invocable, `--dry-run`-able, and
fixture-proven end-to-end — so **s07** can snapshot the `odm` branch, dry-run, inspect, and fire it on
the real corpus with a CDC-verified capability behind it. On close, bubble up to `../arc-plan.md`
(the v2.1 + v2.2 findings resolved; MF-2/MF-3/MF-5 now have an invocable, correct path to their live
reproduction in s07).
