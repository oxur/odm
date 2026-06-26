# Arc 03 — Close: recomposition / silent-drop check

> The arc-level close step (AI-ENGINEERING-METHODOLOGY: "at the close of every slice —
> and again when an arc closes — diff the original scope against the delivered work").
> CDC, planning thread, 2026-06-26. Run after all four slices were CDC-verified on
> structure (cargo rows pending CI). Spec sources: ODD-0015 A3 row + §5, ODD-0013 §6 /
> §4.1 / §7, and this arc's `arc-plan.md` exit criteria.

## Slice roll-up (all CDC-verified on structure; cargo rows pending CI)

| Slice | Capability | Verdict | Impl commit |
|-------|-----------|---------|-------------|
| 01 | Arc 02 cleanup (tear rationale, `assert_cmd` suite, severity recalibration, 0013 reconciliations) | verified, 1 iteration | `1baa3b8` |
| 02 | Rollup model + `odm rollup` (Markdown) | verified, 1 iteration | `4a50ac2` |
| 03 | `orient`/`brief` + bare-`odm` | verified, 1 iteration | `f3d9f21` |
| 04 | `--json` + polish (schema pinning) | verified, 1 iteration | `60f4d57` |

## A3-as-specified vs A3-as-delivered (silent-drop check)

Each A3 capability (0015 A3 row + 0013 §6) classified **delivered / disclosed-deferred /
silent-drop**:

| A3-specified capability | Status | Where |
|---|---|---|
| Generated `ROLLUP.md` (+ `--json`) | **delivered** | slice02 (Markdown, full-scan regenerate, committed at root) + slice04 (`--json`) |
| `orient`/`brief` leads vision → focus → ready/blocked → drift | **delivered (expanded)** | slice03 — *integrity* section inserted (vision → focus → ready/blocked → **integrity** → drift); a disclosed expansion (slice02 ruling 3), not an overwrite |
| Vision source (D-1: project body via `use`/`context`) | **delivered** | slice03 (extraction rule D-1a, Q-A3-3 resolved) |
| Ready/blocked with soft-sat surfacing | **delivered** | slice02 model + slice03 orient (soft-sat ⚠ on the ready frontier; bug fixed in slice02) |
| Original-vs-emergent (provenance) view | **delivered** | slice02 (rollup `origin` grouping, covers every node) |
| Active tears with rationale | **delivered** | slice01 (`TornEdge`) → slice02 render |
| Status vectors in gate-sequence order | **delivered** | slice02 (D-4) |
| `check` integrity unmissable from `orient` | **delivered** | slice03 (inline check Errors) — closes the slice02 orphan-visibility gap |
| errors-as-affordances; bare-`odm` orients; never bare-errors | **delivered** | slice03 (bare-`odm`, 3 no-project fallbacks) + slice04 (affordance sweep) |
| `--json` "on every query", stable documented schemas | **delivered** | slice04 (rollup/orient `--json`; `check`/`rollup`/`orient` schemas pinned in 0013 §7.1, shape-locked) |
| **Deferred nodes surfaced with re-entry predicate** | **disclosed-deferred → A5** | Q-A3-1 (Duncan + CDC): no `deferred` representation exists yet; A3 leaves a defined-but-empty slot; surfacing + predicate land with A5 |

**Silent drops: none.** The single A3-row item not delivered (deferred-node surfacing)
is a **disclosed, ratified deferral** to A5, not a silent drop — and the spec docs have
been reconciled to say so (below). One **disclosed expansion** (the `orient` integrity
section) improves the MVP DoD beyond the literal 0015 wording.

## Spec reconciliations applied (spec-keeping)

To keep the spec honest against delivery, CDC applied:

- **0015 A3 row** — deferred surfacing marked **(deferred to A5 — Q-A3-1)**; the orient
  order updated to include **integrity**.
- **0015 §5.3** — "Lands in A3 (surfacing) + A5 (predicate evaluation)" → **A5
  (surfacing + predicate evaluation), moved from A3 per Q-A3-1**.
- **0013 §6** — the rollup's deferred line now notes *deferred surfacing lands in A5
  (Q-A3-1); A3 leaves a defined-but-empty slot*.
- (Earlier, in-slice: **0013 → v1.8** tears/decomposed typing + Q-7 appendix (slice01);
  **0013 → v1.9** §7.1 JSON schemas (slice04).)

## Trending (recurring patterns across the arc)

- **CC/CDC rhythm held:** every slice closed in **1 iteration**; no slice reached the
  five-iteration cap. CC flagged every deviation rather than burying it.
- **D-3 (model-as-single-source) paid off compounding:** slice02's `Rollup` model made
  slice03 (`orient`) a thin reader and slice04 (`--json`) a 1:1 projection — no
  re-derivation at any layer.
- **CDC catches that improved delivery:** the `Tear<N>` naming collision (slice01,
  pre-empted an iteration), the soft-sat double-listing bug (slice02, CC-found), and the
  orphan-visibility gap (slice02 ruling 3 → slice03 integrity) all landed as fixes.
- **Recurring doc-honesty finding:** spec passages lagging realized code (tears/decomposed
  typing, the deferred deferral). Each was reconciled; the pattern argues for self-hosting
  (A6), where the plan lives *as* checkable nodes and can't silently drift from the docs.

## Outstanding before Arc 03 is *closed* (not blocking the verdict)

1. **CI green** — flips all four slices' cargo rows `attested → reproduced`. The only
   gate between verified-on-structure and closed.
2. **Schema-marker ratification** (J-5): CDC ratified `check/v1` (uniform `<command>/v1`,
   versioning the contract from introduction forward); **Duncan to confirm or switch to
   `check/v2`.**
3. **Merge order to `main`:** slices 01 → 02 → 03 → 04 (dependency order).

## Verdict

**Arc 03 is feature-complete and CDC-verified on structure, with no silent drops.** On
CI green + the schema-marker confirmation + merge, **Arc 03 closes and the MVP (A1–A3)
is complete** — the self-hosting trigger: the plan migrates *into* `odm` as nodes (A6),
after which this very doc set becomes queryable through `odm orient`.

CDC: planning thread, 2026-06-26.
