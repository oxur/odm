---
id: 01KZ1YCP63BRF5V61BRXXPKEQ8
number: 555293800
type: artifact
schema: artifact/v1.1
name: Slice 06 cc-prompt — auto-extend affirmed decomposition on authored additions
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice06-auto-extend-decomposition/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK41NQNY11R2RF7831FT
---
# Slice 06 cc-prompt — auto-extend affirmed decomposition on authored additions

**You are CC.** Real toolchain, with tests. This **extends slice 05's**
`odm_migrate::decompose` (do not reopen slice 05; this is a follow-up slice). It
lands **before** the operator re-migrates + commits the store, so the real MF case
(F-2) is the acceptance anchor. Read `slice-doc.md` + `ledger.md` (F-1…F-7) first.

## Why

Slice 05's `auto_recompose` correctly refuses to *invent* a completeness judgment,
so a genuinely-new slice (e.g. MF's slice 16) is left as drift needing a manual
`odm node decomposed`. But in odm's model **the plan tree declares scope** —
authoring a slice under an arc *is* the membership decision — so that manual step is
redundant bookkeeping. This slice makes `migrate --all` maintain an **already-affirmed**
parent's decomposition automatically as authored children appear.

## What to build

Add an `AutoExtended` outcome to `odm_migrate::decompose::Outcome` and the logic in
`auto_recompose`. The pass runs at the end of `migrate --all`, which has just
reconciled the store to the plan tree, so current work-children are the plan-declared
set. For each parent that **already has** an affirmed `decomposed`:

1. Compute **affirmed work-children** = `decomposition_children`-filter over the
   affirmation's `children` (drop stale non-work entries), and **current
   work-children** = `recompose::decomposition_children(recomp, types, parent)`.
   Reuse slice 05's single definition for both — do not re-derive "work child."
2. **current ⊇ affirmed, no work-child removed (additions only)** → **`AutoExtended`**:
   rewrite `decomposed.children` to the current work-child set (write today's date).
   This also drops any stale non-work ids a pre-fix affirmation carried — which is
   what auto-heals the **MF transitional case** (old 17-with-2-artifacts → clean 16
   work slices, 0 drift).
3. **identity re-mint** (slice 05's `id_remap` maps removed→added) → `ReAffirmed`,
   unchanged.
4. **a work-child was removed** (an affirmed work-child is absent now) → `LeftAsDrift`.
5. **no affirmed decomposition at all** → skip entirely (untouched; it stays an
   `undecomposed-parent` — the first affirmation is a human act).

Under `--dry-run`, compute and report but write nothing. Update the `RECOMPOSE`
report + CLI/`--json` output to show `auto-extended` distinctly from `left as drift`.

## Signal choice (recommended)

Use the **additions-only** test (current ⊇ affirmed work-children) as the "plan-tree
declared" proxy — it is valid because the pass runs *inside* `migrate --all`, and it
is **model-independent** (do not gate on `source.paths`: planning nodes become
self-sourced under ODD-0026, so a source-based signal would break at slice 02). If
you find a case where an added child is genuinely *not* plan-declared and additions-only
would wrongly bless it, leave it as drift and note it in the closing report as a
bubble-up for ODD-0026 — do not expand scope here.

## Do not

- Auto-affirm a **never-decomposed** parent (F-4).
- Auto-heal a work-child **removal** (F-3) — additions are trusted, removals surfaced.
- Change the store-as-source model (slice 01 / ODD-0026).
- Remove slice 05's seam wholesale — this **refines** it: an addition the plan tree
  did not declare still leaves as drift.

## Definition of done

F-1…F-7 reach final status; `make check` green; and a real `reset → migrate --all`
leaves MF (and other affirmed parents) at **0 decomposition drift with no manual
`node decomposed`.** Close with the per-row ledger walk + a bubble-up to the arc
(did this deliver hands-off migrate; anything ODD-0026 must record about
"authoring declares scope"; the silent-drop diff).
