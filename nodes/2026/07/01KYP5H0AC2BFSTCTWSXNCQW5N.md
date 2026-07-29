---
id: 01KYP5H0AC2BFSTCTWSXNCQW5N
number: 580834100
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 02 (Arc 06): migrate odm''s own docs'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice02-migrate-odm-docs/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKDP779VQWKQ8JBG77
---
# CC Prompt — Slice 02 (Arc 06): migrate odm's own docs

Second slice of A6 — where the importer **meets reality**. Run `odm migrate` on odm's **own**
`docs/design` corpus, settle the three decisions slice01 flagged against the real docs, and
hold **`odm check` green** on the imported graph. The plan-set self-host is slice03.

> **Start condition:** slice01 CDC-verified. Branch off **`release/1.0.x`**:
> **`arc06-slice02-migrate-odm-docs`** (not `main`). If slice01 unmerged, branch off it +
> flag for rebase.

## Read first
1. `slice02-migrate-odm-docs/ledger.md` (6 rows) + `slice-doc.md` (same dir) — the three
   slice01 flags to settle (numbering space, real `supersedes` shape, multi-supersession) +
   the `deferred` mapping decision.
2. `../arc-plan.md` — A6 capability, Arc Ledger (this slice closes **A-2**, and is the
   mechanism for compose row **A-7** "runs cleanly on odm's own docs; `check` passes"), the
   open questions.
3. slice01: `crates/odm-migrate/` (the importer you now point at real docs) + its
   `slice01-…/cdc-verification.md` (the four flagged decisions + the slice02 bubble-up).
4. The real corpus: `docs/design/` (ODDs `0002`/`0009`–`0019` across `01-draft`…`10-superseded`).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then error handling.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc**; **propagate it into `arc-plan.md` (A-2 + version entry)**).

## Task
1. **Run migrate on odm's real `docs/design`** (N-1): odd nodes created under `nodes/`; every
   ODD created or reported-skipped (none silently dropped); legacy files intact. Test over a
   **snapshot copy** of `docs/design` for determinism.
2. **`odm check` green** on the imported graph (N-2) — exit 0, or every finding is a reported
   importer skip/warning (never a silent drop or a fabricated edge to force green).
3. **Settle the three flags** (N-3): document `odd` as a distinct numbering space (no
   collision on the real corpus); handle the **real `supersedes` value shape** (may be
   `"ODD-00NN"` strings / `null` / refs, not bare `u32`); **check multi-supersession** — if a
   real ODD supersedes >1, either keep warn+single-edge or **raise an `odm-core` amendment**
   (don't silently drop the extra).
4. **Settle `deferred`** (N-4): if a `07-deferred` ODD exists, decide its mapping (A5
   `deferred` marker vs retire vs distinct gate) + record; if none, record deferred→retire as
   the interim.
5. **Idempotent + `--dry-run` on the real corpus** (N-5): re-run creates 0; dry-run writes
   nothing.
6. **Gates + no regression** (N-6): clippy `-D warnings`; no `unsafe`; ≥ 90% line on new
   paths; `cargo test --workspace` green.

## Constraints (flag, don't silently change)
- **Never mutate a legacy file.** Fix mapping gaps **in `odm-migrate`** (+ a fixture), never
  by editing a legacy ODD to fit the importer.
- **Never force `check` green** by fabricating edges — a real graph issue is *reported*
  (warning/skip), a real importer bug is *fixed + tested*.
- **Amend, don't work around.** Multi-supersession or a real `deferred`-needs-the-marker are
  the two cases that can exceed "run on real docs" into a model change — raise an amendment.
- **Stay in scope:** **not** the `design-v1.0.0` plan-set (slice03); **not** oxur's
  `crates/design/docs` (out of scope, odm's own docs first); not PM-skill (04/05).
- **Render** `writeln!`+`tabled`; **branch off `release/1.0.x`**.

## Deliverables
The real-corpus migration + `check`-green + the settled decisions, with `ledger.md` evidence
per row (`attested`); a `closing-report.md` — per-row walk **plus the Bubble-up to the arc**
(did slice02 deliver A-2 + the A-7 mechanism; what the real corpus revealed for slice03's
plan-set cutover — esp. the reflexive-import ordering; the silent-drop diff) — **and propagate
it into `arc-plan.md` (A-2 + a version entry)**. Feature branch
`arc06-slice02-migrate-odm-docs`; not `main`/`release/1.0.x` directly.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-2) per LEDGER-DISCIPLINE v2.0 §A.
