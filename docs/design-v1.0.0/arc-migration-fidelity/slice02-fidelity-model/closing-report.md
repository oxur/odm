# Slice 02 closing report — The fidelity model (ODD-0025)

> **Arc:** Migration Fidelity · **Slice:** 02 (model) · **Feeds:** MF-2, MF-3, MF-4, MF-6, MF-7
> **Seat:** authored **CDC**; independent gate = **operator** (Duncan). **Ledger:** `ledger.md`
> (F-1…F-14) · **Deliverable:** ODD-0025 (Accepted) · **Date:** 2026-07-27

## What shipped

**ODD-0025 — Migration Fidelity** (`docs/design/04-accepted/0025-migration-fidelity-model.md`,
Accepted), the single authoritative model s03–s06 build against. It formalizes the seven decided
forks, records the operator-confirmed rulings on the open three, gives the frontmatter-fidelity
schema mapping, names the ODD-0013/0020 amendments by section, and introduces the
**origin / source / provenance** three-axis split (§2.0) that resolves a terminology collision
found during authoring. No code, no minting.

## Ledger — per-row walk

14/14 **done**, 0 deferred, 0 no-op (full evidence in `ledger.md`). Every decision the slice-doc
required is recorded in an ODD-0025 section, and the operator confirmed each. Two rows recorded
outcomes **stronger or different than the draft proposal**, tracked (not silent): F-2 (the stored
record renamed `provenance`→`source`) and F-8/F-11 (F10 resolved *mint-all* not exempt;
`author`/`version` *promoted to typed fields* not dropped).

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s02 deliver its assigned piece?** Yes — the arc-plan's s02 row asked for "the ODD(s):
provenance sub-map; supersedes→Vec + bidirectional; supporting-doc artifact-vs-work node class;
frontmatter-fidelity schema mapping; F4/F7/F10." ODD-0025 delivers all of it and closes F4/F9/F10.

**What authoring revealed that the arc-plan did not anticipate** (the slice → arc feedback):

1. **`provenance` was the wrong name.** 0013 explicitly reserves `provenance` for *derived*
   lineage ("never a stored scalar"). The stored migration record is a distinct axis — renamed
   **`source`**. The arc-plan's Capability property 2, the Provenance exit criterion, and MF-3 all
   say "provenance sub-map" and **need rewording to `source`** (done in v1.3).
2. **`author` and `version` become new typed node fields** — an expansion the plan did not carry.
   The operator corrected a proposed drop: `author` must be preserved (git-derivation returns the
   migrator, not the source author) and `version` is SoT quick-access. This **expands s03's scope**:
   s03 now types three new frontmatter fields (`source`, `author`, `version`) + amends 0013 §2.3 +
   bumps schema per 0020, on top of the import-fidelity work. Reflected in the v1.3 s03 row (and a
   sizing flag — s03 may split).
3. **F10 resolved *mint-all*** (every report, incl. `coverage-report.md`, gets an `artifact` node;
   no exemption) — stronger than the v1.2 "a node class / exemption / ignore rule" framing. MF-6's
   note updated.
4. **The `note`-vs-`artifact` sub-fork** surfaced (0013 already has a `note` type) and was resolved
   to a new `artifact` type. Recorded so s05 (which mints the artifacts) inherits a settled model.

**Silent-drop diff (slice scale):** scope-as-specified (slice-doc "In") vs delivered — no drops.
The "Out" items hold: no code, no minting, no store-schema change, no 0013/0020 file edits (those
are s03), no `check` wiring (s05). The one-context sizing held (the model fit; no split needed at
s02).

**Recommended arc-ledger update:** MF-2/MF-3/MF-4/MF-6/MF-7 stay **planned** — s02 produced the
*model*, not the implementation. Recorded as pointers to ODD-0025 as their design baseline (v1.3).
