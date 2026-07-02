# CC Prompt — Slice 06 (Arc 05): deferred surfacing + re-entry predicate (Q-A3-1)

Sixth slice of A5. Cashes the Q-A3-1 deferral: a node can declare itself **deferred** with a
**checkable re-entry condition**, and `rollup`/`orient` surface it — *why* it's parked and
*whether it's ready to resume*. Fills the defined-but-empty A3 `Deferred` slot.

> **Start condition:** slices 01–05 CDC-verified. Branch off `main`:
> **`arc05-slice06-deferred-surfacing`** (not `main`). If 01–05 unmerged, branch off
> slice05's branch and flag for rebase.

## Read first
1. `slice06-deferred-surfacing/ledger.md` (7 rows) + `slice-doc.md` (same dir) — especially
   **"Where deferred is read from"** (the reconcile store-overlay, **not** the index — the
   A4 invariant must stay non-triggered) and the **`next`-withholding limitation** (out of
   scope, documented).
2. `../arc-plan.md` — A5 capability, Arc Ledger (this slice closes **A-6** and is the
   mechanism for compose row **A-12**), Q-A3-1 origin, and ODD-0019 (the freshness model —
   the re-entry predicate is a probe that 07/08 will make incremental).
3. `crates/odm-core/src/rollup.rs` — the **empty A3 `Deferred` slot** (`struct Deferred`,
   ~238; `rollup.deferred`) you will fill; and how slice04 filled `Drift` via
   `Rollup::with_drift` (the same shape for deferred).
4. slice01 `desired_facts` (`crates/odm-core/src/desired.rs`) — `reenter_when` references a
   fact by id. slice04 `compute_drift` (`crates/odm-cli/src/reconcile.rs`) — the store-read
   projector you extend/parallel for deferred. slice05 `check_*` + `violation_severity`
   (`check.rs`, `commands.rs`) — the pattern for the dangling-`reenter_when` finding.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design, error handling, layering.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc** at close; **propagate the bubble-up into `arc-plan.md` (A-6 row +
  version entry) yourself**, as in slices 02–05).

## Task
1. **`deferred` marker** (D-1): `deferred: { because, reenter_when: <fact_id> }` in
   frontmatter (`reenter_when` references one of the node's own `desired_facts`); round-trip;
   absent ⇒ not deferred; `skip_serializing_if` for the absent default (YAML-additive).
2. **Re-entry predicate eval** (D-2): resolve `reenter_when` to the fact's outcome via the
   reconcile store-read — `Holds` → ready-to-re-enter; `Drifted`/`Error` → still-deferred
   ("waiting on `<describe>`").
3. **Fill the `Deferred` slot** (D-3) + **surface in `rollup` and `orient`** (D-3/D-4) via
   one shared projector (off the reconcile overlay, like `compute_drift`); clean → no section,
   no fabricated data.
4. **Store-overlay, no index change** (D-5): read the marker + predicate from
   `compute_drift`/`run_corpus`; make **no** change under `crates/odm-index/`. (Invariant
   honored by non-triggering — a row guards it with an empty `git diff`.)
5. **`--json` additive** (D-6): the `deferred` slot populated; **no `rollup/v1`/`orient/v1`
   version bump**. A **dangling `reenter_when`** → a `check` finding (Warning, reusing
   slice05's `violation_severity`), never a panic.
6. **Gates** (D-7): clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If the marker shape or `reenter_when`-by-id coupling is
  awkward, raise an amendment (the inline-ProbeSpec alternative is noted in the slice-doc).
- **No index change.** Deferred is reconcile-bound; read it from the store-overlay. Touching
  `crates/odm-index/` would fire the A4 invariant and is the rejected design.
- **Do NOT add `next`/graph withholding** of deferred nodes — explicitly out of scope
  (it would force indexing the marker). Surfacing only. (Documented limitation.)
- **Reuse, don't duplicate:** `reenter_when` references a `desired_fact` (not a second inline
  probe); the deferred projector parallels/extends `compute_drift`; the dangling finding
  reuses slice05's `check_*` pattern.
- **No fabricated data:** none deferred → no section. **Render:** `writeln!` + `tabled`.
- **Additive JSON:** no schema version bump for the populated `deferred` slot.

## Deliverables
The `deferred` marker + re-entry eval + filled `Deferred` slot + rollup/orient surfacing +
`--json` + the dangling-`reenter_when` check, with `ledger.md` evidence per row (`attested`);
a `closing-report.md` — per-row walk **plus the Bubble-up to the arc** (did slice06 deliver
A-6 + the A-12 mechanism; what it reveals for 07/08 — especially how the re-entry predicate
should fold into the incremental freshness model; the silent-drop diff) — **and propagate
that into `arc-plan.md` (A-6 + a version entry)**. Feature branch
`arc05-slice06-deferred-surfacing`; not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-6) per LEDGER-DISCIPLINE v2.0 §A.
