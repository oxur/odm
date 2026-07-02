# Slice 06 (Arc 05): deferred surfacing + re-entry predicate (Q-A3-1)

> Plan-of-record for A5 slice06. Cashes the Q-A3-1 deferral A3 deliberately left open ("don't
> invent a `deferred` status just to render it — settle the schema when slice06 is planned").
> Fills the **defined-but-empty A3 `Deferred` slot** (`odm-core::rollup::Deferred`).

## Goal

A node can declare itself **deferred** with a **checkable re-entry condition**, and
`rollup`/`orient` surface it honestly: *why* it's parked and *whether it's ready to resume*.
The re-entry condition is a probe (a `desired_fact`), so "can this re-enter?" is answered by
reconcile, not by a human eyeballing it — the whole A5 ethos, applied to parked work.

## The design (settle Q-A3-1)

- **`deferred` frontmatter marker** (new field): `deferred: { because: <text>, reenter_when:
  <fact_id> }`. `reenter_when` references one of the node's own `desired_facts` by id — the
  fact whose *holding* means "the reason to defer is gone; this can resume." Absent ⇒ not
  deferred (default). This reuses slice01's `desired_facts` (DRY) rather than embedding a
  second probe spec.
  - *Alternative considered:* an inline `ProbeSpec` in the marker. Rejected — it duplicates
    the probe machinery; a `fact_id` reference reuses `desired_facts` and keeps one probe
    model. (Flag if the reference coupling proves awkward.)
- **Re-entry status = the referenced fact's outcome:** `Holds` → **ready to re-enter**;
  `Drifted`/`Error` → **still deferred** ("waiting on <describe>"). Evaluated by the
  reconciler — the same probe path drift uses.

## Where deferred is read from — and why the A4 invariant does NOT fire

Deferred is **reconcile-bound**: its whole meaning is its re-entry *predicate* (a probe). So
both the marker and its predicate status are read from the **reconcile store-overlay** — the
`compute_drift` / `run_corpus` store-read that `rollup`/`orient` already run for drift
(slice04) — **not** from the index.

- **No `IndexRecord`/adapter/`FORMAT_VERSION` change.** The A4 adapter-fidelity invariant
  stays **honored by non-triggering** (as with `desired_facts` and drift). A ledger row
  guards it (empty `git diff` under `crates/odm-index/`).
- *Alternative considered — index the `deferred` marker* (it is light structural metadata,
  like `origin`/`decomposed` which are indexed). **Rejected:** deferred is meaningless without
  its reconcile-evaluated predicate, so it belongs with reconcile, not in the fast structural
  model; indexing it would fire the invariant + force a `FORMAT_VERSION` bump for a field
  whose truth is reconcile-bound. Keep the marker with its predicate. (Revisit only if `next`
  must withhold deferred nodes — see the limitation below.)

## Scope — in

1. **`deferred` marker** (`odm-core`): the frontmatter field + round-trip; absent = default.
2. **Re-entry predicate evaluation**: resolve `reenter_when` to the node's `desired_fact`
   outcome via the reconcile store-read; classify ready-to-re-enter vs still-deferred.
3. **Fill the `Deferred` slot** (`odm-core::rollup::Deferred`): populate it (was empty in A3)
   with the deferred nodes + because + re-entry status.
4. **Surface in `rollup` + `orient`** via a shared `odm-cli` projector (off the reconcile
   overlay, like `compute_drift`); clean (none deferred) → no section / no fabricated data.
5. **`--json`** (`rollup/v1` + `orient/v1`): the `deferred` slot populated **additively** (it
   was empty/absent in A3; no version bump — the slice04 drift precedent).

## Known limitation (flagged, out of scope) — `next` does not yet withhold deferred nodes

Per the arc-plan, slice06 is **surfacing** (rollup/orient) + predicate eval — **not** graph
withholding. So a deferred node that is dependency-ready could still appear in `next` while
`rollup`/`orient` show it as deferred — a mild inconsistency. Withholding deferred from `next`
would require the **index-backed graph reader** to see the marker → indexing it → the A4
invariant + a `FORMAT_VERSION` bump (the rejected alternative above). **Deferred to a scoped
follow** (likely alongside the freshness work or a later slice); surfacing-first is the
deliberate A5-plan scope. Documented, not silently dropped.

## Scope — out (named, not dropped)

- **`next`/graph withholding of deferred nodes** — flagged above; a scoped follow.
- **The freshness rework** (07/08) — the re-entry predicate is a **probe**, so pre-freshness
  it evaluates via `run_corpus`/`compute_drift` (store-read, like drift today); **07/08 fold
  it into the incremental machinery**. A clean, disclosed interim.
- **A dangling `reenter_when`** (references a non-existent `fact_id`): surfaced as a **`check`
  finding** (link-integrity, reusing slice05's `check_*` + `violation_severity` pattern), not
  a panic. Small; in scope as a validation row.

## Verification approach

`odm-core` unit tests + `odm-cli` integration tests:

- the `deferred` marker round-trips; absent = not deferred.
- `reenter_when` → fact Holds → "ready to re-enter"; Drift/Error → "still deferred (waiting
  on <describe>)".
- rollup + orient surface deferred (shared projector); none deferred → no section.
- `--json` deferred slot populated additively (no version bump).
- read from the reconcile store-overlay — **no** change under `crates/odm-index/`.
- a dangling `reenter_when` → a `check` finding (not a panic).
- clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

Cargo rows are CC-`attested`, reproduced via CI / local 1.85+.

## Exit criteria

Ledger D-1…D-7 reach a final status: the `deferred` marker + re-entry predicate exist; the
predicate is reconcile-evaluated (ready vs waiting); rollup + orient surface deferred honestly
(the A3 slot filled; no fabricated data); `--json` additive; read from the store-overlay with
no index change (invariant non-triggered); dangling `reenter_when` is a check finding; gates
pass.

> **Render convention (slice03):** `writeln!` + `tabled` (no `oxur-cli` dep).
