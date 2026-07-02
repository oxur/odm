# CC Prompt — Slice 05 (Arc 05): `affects` edge + stale-doc-vs-decision check (C5)

Fifth slice of A5. Cashes ODD-0001 **C5**: `odm check` flags a **potentially stale doc** —
when a committed decision `A affects` a doc `B` and `A` was updated *after* `B`, surface B as
possibly not reflecting A. The `affects` edge already exists (A2); this slice adds the
**check semantics**. Independent of the freshness rework (07/08).

> **Start condition:** slices 01–04 CDC-verified. Branch off `main`:
> **`arc05-slice05-affects-stale-doc`** (not `main`). If 01–04 are unmerged, branch off
> slice04's branch and flag for rebase (as prior slices did).

## Read first
1. `slice05-affects-stale-doc/ledger.md` (6 rows) + `slice-doc.md` (same dir) — especially
   **"The honest detection"**: structural/temporal, **never semantic**. The `affects` edge
   *is* the committed-decision assertion; staleness is `A.updated > B.updated`.
2. `../arc-plan.md` — A5 capability, Arc Ledger (this slice closes **A-5** and is the
   mechanism for compose row **A-11**), the C5 origin.
3. `crates/odm-core/src/check.rs` — how findings + severities work today (the dangling-ref
   pass already iterates `edges.affects`); add the stale-doc finding alongside.
4. `crates/odm-core/src/frontmatter.rs` (`affects: Vec<Id>`, `updated: NaiveDate`) and the
   A2 staleness guard (out-of-order `updated`) — the same temporal idea, across `affects`.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design, error handling.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc** at close; **propagate the bubble-up into `arc-plan.md` (A-5 row +
  version entry) yourself**, as in slices 02–04).

## Task
1. **Stale-doc finding** (T-1): in `odm-core::check`, for each `A affects B` with
   `A.updated > B.updated`, emit a finding naming A, B, and both dates.
2. **Warning + no false positive** (T-2): Warning severity (advisory; `--strict` fails);
   `B.updated >= A.updated` is **not** flagged.
3. **Structural, not semantic** (T-3): flag "potentially stale" only — never assert
   contradiction. Use `>` (not `>=`) so a same-day decision+doc edit isn't nagged; document
   the `NaiveDate` day-granularity limitation in the finding's doc comment.
4. **No new index field** (T-4): read `affects` + `updated` off the index (both already
   present since A4). Make **no** change under `crates/odm-index/` — no adapter, no
   `FORMAT_VERSION`. (The invariant stays honored by non-triggering; a row guards it.)
5. **`--json` additive** (T-5): the stale-doc finding joins the existing `check` findings
   list; **no `check` schema version bump**.
6. **Gates** (T-6): clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If the finding shape or the temporal predicate needs to
  differ, raise an amendment.
- **Never semantic.** Do not attempt content-based contradiction detection — structural +
  temporal only (the odm ethos; the recomposition-integrity boundary).
- **No index changes.** `affects`/`updated` are already indexed; touching `crates/odm-index/`
  would trip the A4 invariant and is unnecessary.
- **Stay in scope:** no deferred surfacing (slice06), no freshness/probes work (07/08).
- **Render convention:** `writeln!` + `tabled` (no `oxur-cli` dep — the slice03 finding).
- **Additive JSON:** don't bump the `check` schema version for an added finding kind.

## Deliverables
The stale-doc check + render + `--json`, with `ledger.md` evidence per row (`attested`); a
`closing-report.md` — per-row walk **plus the Bubble-up to the arc** (did slice05 deliver
A-5 + the A-11 mechanism; what it reveals for slice06; the silent-drop diff) — **and
propagate that into `arc-plan.md` (A-5 + a version entry)**. Feature branch
`arc05-slice05-affects-stale-doc`; not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-5) per LEDGER-DISCIPLINE v2.0 §A.
