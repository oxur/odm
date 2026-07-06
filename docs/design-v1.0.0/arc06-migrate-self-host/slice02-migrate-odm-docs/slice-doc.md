# Slice 02 (Arc 06): migrate odm's own docs

> Plan-of-record for A6 slice02 — where the importer **meets reality**. slice01 built the
> core on fixtures; this slice runs `odm migrate` on odm's **own** `docs/design` corpus,
> settles the three real-corpus decisions slice01 flagged, and holds `odm check` green on the
> imported graph. The *plan-set* self-host (project-plan/arc-plans/slice-docs) is slice03.

## Goal

`odm migrate docs/design` imports odm's real ODD corpus (~13 ODDs, `0002`/`0009`–`0019`,
across the `01-draft`…`10-superseded` state-dirs) into `odd` nodes under `nodes/`, with the
legacy files untouched (supersede-not-delete) and **`odm check` green** on the result. Real
inputs shake out the edge cases fixtures can't; this slice resolves them.

## Scope — in

1. **Run `odm migrate docs/design`** on odm's real ODD corpus → `odd` nodes created under
   `nodes/YYYY/MM/<ULID>.md`; every legacy ODD is **created or reported-skipped, none silently
   dropped**; legacy `docs/design` files **intact** (never-delete carries from slice01).
2. **`odm check` green on the imported graph** — no orphans / dangling refs / integrity
   errors on the imported `odd` nodes. This is the slice's real acceptance bar: the imported
   graph is structurally sound.
3. **Settle the three slice01-flagged decisions against reality** (arc-plan v1.2 / slice01
   bubble-up):
   - **`odd` numbering space** — document `odd` as a **distinct** numbering space (the code
     already keys idempotence on `(type=odd, number)`); confirm no collision with work-node
     numbers on the real corpus.
   - **real `supersedes` value shape** — the legacy `supersedes`/`superseded-by` may not be
     bare `u32`s (could be `"ODD-0011"` strings, `null`, or refs); handle whatever
     `docs/design` actually uses (or report a clean skip).
   - **multi-supersession** — check whether any real ODD supersedes **>1**; if so, decide
     (warn + single edge as-is, or raise an `odm-core` model amendment); if none occur,
     record "not present in the corpus" and keep the single-edge model.
4. **`deferred` mapping settled against reality** — if any `07-deferred` ODD exists, decide
   its mapping (A5's first-class `deferred` marker vs retire vs a distinct gate); if none,
   record deferred→retire as the interim (revisitable) — no real instance forces it now.
5. **Idempotent + `--dry-run` hold on the real corpus** — re-running creates 0; `--dry-run`
   previews the real import without writing.

## Scope — out (named, not dropped)

- **The `design-v1.0.0` plan-set self-host** (project-plan, arc-plans, slice-docs → nodes;
  `orient`/`rollup` on the self-hosted corpus; the reflexive cutover) — **slice03**.
- **Migrating oxur's `crates/design/docs`** — **out of scope** (odm's own docs first; oxur's
  corpus is a separate later concern, not an A6 slice). Decided here per the arc-plan open Q.
- **PM-skill / retiring framework prose** — **slice04/05**.

## Design notes / decisions to flag

- **Real import, not a demo.** This slice creates the `odd` nodes for real (committed on the
  branch); legacy `docs/design` stays (supersede-not-delete). Whether/when the legacy files
  are eventually retired is a slice03/post-cutover call — not this slice.
- **Edge-case fixes land in `odm-migrate`.** If the real corpus surfaces a mapping gap (a
  `supersedes` shape, a state the fixtures lacked), fix it in `odm-migrate` + add a fixture —
  don't hand-edit a legacy ODD to fit the importer (never-mutate legacy).
- **If `check` isn't green**, the honest fixes are: (a) a real importer bug → fix + test;
  (b) a genuine graph issue in the ODDs (e.g., a dangling supersedes) → the importer reports
  it (warning/skip), not silently repaired. Don't force green by fabricating edges.
- **Amendment trigger:** multi-supersession or a real `deferred` ODD needing the A5 marker are
  the two cases that could exceed "run on real docs" into a model change — **raise an
  amendment** if so (may spawn a small follow), don't quietly extend scope.

## Verification approach

Integration test(s) driving `odm migrate` over a **snapshot copy of the real `docs/design`**
(so the test is deterministic + doesn't depend on the live tree), plus the real run:

- migrate the real corpus → node count = ODD count (created + reported-skipped); legacy files
  byte-identical after (never-delete).
- `odm check` on the imported graph → exit 0 (green), or every finding is a reported
  importer skip/warning, not a silent drop.
- the real `supersedes` shape parses (or skips cleanly); numbering space has no collision;
  multi-supersede / deferred observations recorded with their decision.
- re-run → 0 created (idempotent); `--dry-run` → 0 written.
- clippy `-D warnings`; no `unsafe`; coverage ≥ 90% on new `odm-migrate` paths; no regression.

Cargo rows are CC-`attested`, reproduced via CI / local 1.85+.

## Exit criteria

Ledger N-1…N-6 reach a final status: `odm migrate` runs on odm's real `docs/design` (odd
nodes created, legacy intact, none dropped); `odm check` is green on the imported graph; the
three slice01 flags (numbering space, supersedes shape, multi-supersede) + the deferred
mapping are settled against the real corpus (with any model gap raised as an amendment);
idempotent + dry-run hold; gates pass. **On close, slice03 brings the plan-set in and the
self-host loop closes.**

> **Render/convention:** `writeln!` + `tabled` (no `oxur-cli`). **Git:** branch off
> `release/1.0.x`. Reuse the slice01 importer; fix edge cases *in the importer*, never in a
> legacy file.
