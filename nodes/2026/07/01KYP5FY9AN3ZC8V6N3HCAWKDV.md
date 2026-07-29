---
id: 01KYP5FY9AN3ZC8V6N3HCAWKDV
number: 562390100
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 10 (Migration Fidelity): Coverage live run'
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice10-coverage-live-run/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYP5FRXJFZ3FSF123XS3FD9G
---
# CC Prompt — Slice 10 (Migration Fidelity): Coverage live run

Fire **s09's enforcement capability on the live `.worktrees/odm` corpus**: mint an `artifact` node for
every supporting doc (mint-all), backfill `source` onto the 14 design/research nodes, and **turn the
doc-coverage check on** — all as **one revertible commit** behind the s07 snapshot → dry-run →
adjudicate → fire → verify protocol. This makes "no file left behind" an enforced check on odm's own
store.

> **Start condition:** on `release/1.0.x` with **s09 merged + green** (the capability must exist first).
> **This slice mutates the live `odm` store** — a large mint + a gate flip — but only as one revertible
> commit after a clean, **adjudicated** dry-run. **Snapshot SHA + dry-run-first are HARD gates.** Unlike
> s08, **creates are expected** (you are minting ~211 nodes): the gate is **adjudication**, not
> "0 creates" — the created set must match s01's supporting-doc inventory, the modified set must be
> exactly the 14 design/research nodes, and **nothing else may change**. Any other deviation → **stop,
> flag CDC, do not fire.** Mint + gate-flip land **together** so `check` never has a red window.

## Read first

1. `slice10-coverage-live-run/ledger.md` (10 rows) — the spec of "done."
2. `slice-doc.md` (esp. **Why** + the dry-run adjudication gate); **s07** `slice07-live-run/closing-report.md`
   + `cdc-verification.md` (the live-mutation protocol you are reusing) and **s08**
   `slice08-source-path-portability/closing-report.md` (the adjudicate-past-a-gate discipline + the
   known-good tip `7226797`).
3. **s01** `slice01-coverage-discovery/coverage-report.md` — the exact supporting-doc inventory your
   mint must match (the adjudication ground-truth).
4. **ODD-0025** §2.5 (artifact containment), §2.6 (mint-all incl. reports), §2.7 (optional containment),
   §2.1 (the body-hash gate).
5. **The live store + config:** nodes under `.worktrees/odm/nodes/`; the operational gate-set is
   `.worktrees/odm/config.toml` (per the C-5 cutover — **not** the root `odm.toml` locator). The
   doc-coverage gate flips **on** there, committed with the data.

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first (mostly a run this slice; touch code only for the gate wiring).
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **Pre-flight** (F-1). Confirm `release/1.0.x` green (s09 in) + `.worktrees/odm` clean. **Capture the
   known-good SHA (`7226797`) + a before-manifest** (sha256 composite over all node files). This is your
   revert anchor.
2. **Dry-run + adjudicate** (F-2). `odm migrate --dry-run`. Diff the **created** set against s01's
   supporting-doc inventory (count + identity) and the **modified** set against the 14 design/research
   nodes. Confirm **0 unexpected change** to the 62 existing source-bearing nodes / project (`#1000`) /
   retired (`#1605`), and the store fingerprint is unchanged after the dry-run. **Adjudicate before
   firing** — investigate any deviation; clear it or revert. Do not fire on a mismatch.
3. **Fire as one revertible commit** (F-3…F-6, F-9). Mint the artifact corpus (mint-all incl. every
   report + `coverage-report.md`, §2.6), each 1:1 body-hash-gated (§2.1) and `part_of` its nearest
   modeled scale (§2.5); backfill `source` on the 14 design/research nodes (§2.7 optional containment).
   One commit on the `odm` branch atop `7226797`; document `git reset --hard 7226797`.
4. **Flip the coverage gate on** (F-7). Enable doc-coverage in `.worktrees/odm/config.toml` **in the same
   commit**, so `odm check` enforces coverage and is **green because everything is now covered** — no red
   window between mint and flip.
5. **Verify on the committed store** (F-4…F-8). Body-hash faithfulness on the mints; correct `part_of`;
   the 14 nodes source-bearing + not orphaned; **0 uncovered**; `check` exit 0 (enforcing);
   `orient`/`rollup` byte-stable ×2; re-run idempotent (0/0) + cross-root stable; project/retired
   untouched (last-touch commit unchanged); existing-node ids/bodies/schema intact.

## Constraints (flag, don't silently change)

- **Snapshot SHA + dry-run-first are non-negotiable.** Creates are expected — adjudicate the sets; any
  *unexpected* create/modify, any body-hash failure, any re-mint of an existing node, `check` not
  reaching green, or a non-idempotent re-run → **revert to `7226797` + finding**, not forced.
- **No capability code here.** The `artifact` type, discovery reach, check rule, and coverage.rs fixes
  are **s09**. If s10 exposes a capability gap, that is an **s09** fix (flag CDC) — don't grow s10.
- **Mint + gate-flip atomic** (one commit) so `check` has no red window.
- Don't pull **s11** (synthesis + L-8b) or **s12** (reconcile run) forward. The living-doc-drift
  reconcile is **s12** — s10 only records that the mint creates its mirror at scale (every report node is
  a snapshot the instant it's written).
- No `unsafe`; typed errors; any code touched stays clippy-clean; amend ODD-not-work-around if a model
  line is genuinely needed.

## Deliverables

The committed rewritten store (`odm` branch, one commit atop `7226797`); any gate-wiring code on
`release/1.0.x`; `ledger.md` evidence per row (`attested`/`reproduced` — cite counts, the adjudication
diff, command exits, the undo SHA); `closing-report.md` — per-row walk, the before/after (mint count vs
s01 inventory, 14 backfilled, gate on, 0 uncovered), the dry-run adjudication record, any findings,
**plus the v2.0 Bubble-up** (did s10 enforce coverage live; what the mint revealed; the silent-drop diff
vs In/Out; confirm s11 next + the s12 reconcile carry). Branches: `odm` (store) + `release/1.0.x` (gate
wiring, if any).

## Working agreement

Amend don't work around; flag every deviation; five-iteration cap (+ split-escape if the mint and the
verify won't both land in one context — land the mint + gate-flip first, split the extended cross-root
verification, flag CDC). Your `done` is proposed-done — CDC reproduces the store-state rows by direct
read (mint count + fidelity, containment, exclusions, 0-uncovered) + CI. On close, bubble up to
`../arc-plan.md` (MF-1/MF-3/MF-6 done; s11 next; reconcile → s12).
