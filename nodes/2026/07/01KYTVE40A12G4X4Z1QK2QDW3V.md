---
id: 01KYTVE40A12G4X4Z1QK2QDW3V
number: 556344000
type: artifact
schema: artifact/v1.1
name: 'Slice 12 (Migration Fidelity) — CDC verification: Reconcile capability'
created: 2026-07-30
updated: 2026-07-30
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice12-reconcile-capability/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-31
edges:
  part_of: 01KYSX4R1GPDKH7M5ANCTDND04
---
# Slice 12 (Migration Fidelity) — CDC verification: Reconcile capability

> **Arc:** Migration Fidelity · **Slice:** 12 · **Verifier:** CDC (independent) · **Date:** 2026-07-29 ·
> **Method:** LEDGER-DISCIPLINE v2.0 §A + PM Part IV. **Fixture + doc only — no live class-(b) row.** Code
> + tests + the ODD amendment reproduced by direct read on `release/1.0.x`; runtime execution
> (`cargo`/`clippy`/`llvm-cov`) attested-by-CC → CI.
> **Under review:** `ledger.md` (F-1…F-10), commits `5e99889` (open), `3daf893` (F-1…F-7 code + ODD),
> `30ee7d2` (close) — all `release/1.0.x`.

## Verdict

**PASS — CDC-verified.** The reconcile capability reproduces in code with thorough, non-vacuous tests; the
living-plan-node policy is decided and recorded as a well-formed ODD-0025 §2.9 amendment; the §4 follow-up
is resolved; and **no live store mutation occurred** (`odm` HEAD unchanged — reproduced). A clean slice.
CC independently arrived at the same capability/live split I'd drawn (s12 capability / s13 live) — a good
sign the cut was right — and updated the arc-plan accordingly.

## 1. Reconcile capability (F-1…F-6) — reproduced

| Claim | Reproduced in code | Tests (read, non-vacuous) |
|-------|--------------------|---------------------------|
| **F-1** re-snapshot mode | `mapping.rs`: a drifted non-stub is **re-snapshotted in place** (`reconciled`), `id`/`edges`/`status` preserved, gate re-runs against current source (trivially holds); replaces the old skip/hard-reject | `reconcile_resnapshots_a_drifted_arc_plan_body` — `reconciled_count==1`, `id` preserved ("same node, not re-minted"), body == amended content |
| **F-2** moved-source re-discovery | re-found by the family's **identity key** (`number` / `(type,number)`), *not* a stored path — "moved legacy file still resolves by identity, for free"; `source.paths` rewritten | `reconcile_rediscovers_an_arc_whose_directory_moved` — `path_moved` detected, `id` preserved, stored path rewritten to the new location |
| **F-3** living-plan-node policy | reconcile-to-current, no exclusion; recorded in **ODD-0025 §2.9** | `reconcile_reconciles_cleanly_across_repeated_arc_plan_edits` (V2→V3 — the living case, directly) |
| **F-4** vision-apply | `synthesis::apply_project_vision` (`synthesis.rs:243`) — s11's inline mechanism promoted to reusable library code any s13 caller invokes | `vision_synthesis.rs` |
| **F-5** stub-repair vs non-stub re-snapshot | §2.9 keeps §2.8 stub-repair distinct from the new non-stub re-snapshot | (fixtures above) |
| **F-6** idempotent + dry-run-safe | | `reconcile_is_idempotent_and_dry_run_writes_nothing` |

Zero `unsafe` in `mapping.rs`. `clippy`/coverage → CI.

## 2. ODD-0025 §2.9 + §4 — reproduced and sound

- **§2.9 (living-plan-node policy)** is a well-formed amendment: it distinguishes §2.8 stub-repair from the
  new reconcile-of-a-legitimately-changed-source; keeps the §2.1 gate **migration-time-only** (reconcile
  re-runs the verbatim check against *current* source, so it holds by construction, not a continuous
  check); and decides the s08 open question — a living-plan node reconciles to current with **no special
  exclusion**, safe because (1) inter-reconcile drift is by-design invisible to `check` and (2) at
  arc-close the source stabilizes. The project **synthesis** node stays the one 1:1 exclusion (§2.3), its
  re-cast handled by vision-apply, not reconcile. This resolves the s08 finding cleanly.
- **§4 resolved:** `artifact/v1.0` → `artifact/v1.1` (the shared global generation minor) — the s09 CDC
  LOW follow-up, now closed. Correctly disclosed that editing ODD-0025 drifts its own node **#25**, routed
  to **s13** as a re-snapshot target ("resolved *here* because the reconcile mechanism now exists to absorb
  the drift on its own source node"). The self-referential loop is handled, not hand-waved.
- **F-9 (amend-not-work-around) honored:** the 1:1 rule and the migration-time-only gate are unchanged;
  §2.9 is the single, cited model addition.

## 3. No live mutation (F-8) — reproduced

`odm` branch HEAD unchanged at `2fc25f5` (s10 iteration 1) — **no s12 store commit**. The reconcile +
vision-apply are library code with fixture tests; no live reconcile, no vision mint. The arc-plan re-split
(s12 capability / **s13 live reconcile + vision mint** / arc-close after) is recorded (v2.20).

## 4. Ledger — CDC disposition

F-1…F-6 confirmed in code + non-vacuous tests (reproduced structurally / attested→CI on execution). F-7
(§4) **confirmed** — amended to `v1.1`, #25 → s13. F-8 **reproduced** — no live mutation. F-9 confirmed —
§2.9 the one cited model addition. F-10: 0 `unsafe` (reproduced); clippy/coverage → CI. **10 rows, no
silent drops** — every "Out" item (live reconcile + vision mint → s13; MF-9 composition + P-12 → arc-close;
L-8a post-1.0) confirmed untouched.

## 5. Observations

1. **The moved-source-by-identity design is a keeper** — resolving a relocated source by the corpus's
   identity key rather than a remembered path means a moved file "still resolves for free." Worth carrying
   into any future move/migration work (the L-8a design-corpus migration especially).
2. **CC's independent convergence on the capability/live split** — the same cut I'd drawn — is a small but
   real signal the arc's capability-then-live rhythm is the right shape for this work, not just my habit.

No findings.

## 6. Bubble-up check (PM Part IV)

- **Did s12 deliver?** Yes — reconcile is a real, verified capability (re-snapshot in place, moved-source
  re-discovery, the living-plan-node policy, the reusable vision-apply); the s08 living-node question and
  the s09 §4 follow-up are both resolved and recorded.
- **Silent-drop diff:** none. The one self-referential subtlety (editing ODD-0025 drifts node #25) is
  disclosed and routed to s13, not dropped.
- **Arc-plan change:** flip s12 → **CDC-verified PASS** (v2.21). **s13 (live reconcile + vision mint) is
  next** — fire the capability on the live corpus (re-snapshot the arc node, ODD-0013/0017/0018/0020, and
  #25; mint the vision synthesis) behind the snapshot → dry-run → fire protocol. After s13 CDC-closes, the
  **arc-close**: the MF-9 composition check + the **P-12 self-host acceptance demonstration** + the arc
  `closing-report.md` + the bubble-up to `project-plan.md`.

## Closure

s12 **CDC-verified PASS** on 2026-07-29. Reconcile capability reproduced with thorough tests; the
living-plan-node policy and §4 resolved and recorded (§2.9); no live mutation. s13 (the live close) is
next, then the arc-close reproduces P-12 — the end of Migration Fidelity.

_Verified by: CDC (independent), 2026-07-29 — against `release/1.0.x` (`5e99889`…`30ee7d2`); `odm@2fc25f5`
unchanged._
