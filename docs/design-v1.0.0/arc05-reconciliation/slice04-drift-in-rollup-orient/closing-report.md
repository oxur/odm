# Closing report — Slice 04 (Arc 05): drift in `rollup` / `orient`

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc05-slice04-drift-in-rollup-orient`, branched off
> `arc05-slice03-reconcile-command` (slices 01–03 unmerged; the prompt's named
> fallback). Rebase onto `main` once 01–03 merge.

## Per-row walk

**S-1 — `Drift` model fleshed (odm-core, no reconcile dep) — done (attested).**
`Drift { holds, drifted: Vec<DriftedFact>, errored: Vec<ErroredFact> }` plus
`DriftedFact`/`ErroredFact` (identity + expected/observed | reason), with
`new`/`is_clean`/`is_empty`. Kept `#[non_exhaustive]` + a `Drift::new`
constructor. `drift_projection_shape` → ok; the `odm_reconcile` grep is clean
(doc prose reworded to "the `reconcile` crate" so the layering guard stays a real
tripwire).

**S-2 — report carries render-identity; double-load resolved — done
(attested).** `NodeReport` gained `number`/`name`, `FactResult` gained
`describe`, joined once in `run_node`. `reconcile.rs` was simplified to render
from the enriched report — its slice03 `NodeView`/`HashMap`/second `load_all` are
gone (`load_all` grep clean). `report_carries_identity` → ok.

**S-3 — rollup renders real drift; placeholder gone — done (attested).**
`Rollup::assemble(...).with_drift(compute_drift(store)?)` injects the on-demand
projection; `render_drift` renders per-fact drift (identity + expected/observed)
and couldn't-check (reason); clean → "No drift". `rollup_drift_reported` +
`rollup_clean_no_drift` → ok; the "not yet tracked (A5)" grep is clean.

**S-4 — orient renders real drift (same helper) — done (attested).** orient's
section #5 renders the shared `Drift`; clean → "no drift"; placeholder gone.
`orient_drift_reported` + `orient_clean_no_drift` → ok.

**S-5 — drift on-demand store-read, never index-cached — done (attested).**
`grep desired_facts crates/odm-index/src` → none; the drift path calls
`Runner::run_corpus` (store). No `IndexRecord`/adapter/`FORMAT_VERSION` change —
the A4 invariant stays honored by non-triggering. *(Verify-amend flag: the
`Drift|drift` alternation in the row's Verify matches pre-A5 unrelated prose in
odm-index — "config drift", "recomposition drift"; the load-bearing check is
`desired_facts` + the `run_corpus` call.)*

**S-6 — one shared drift projector — done (attested).** `reconcile::compute_drift`
is the single compute/project, called by both `rollup` and `orient`; the
per-command functions only render. `grep "fn .*drift"` shows the one projector.

**S-7 — additive `--json` + gates — done (attested).** `rollup/v1` + `orient/v1`
`drift` slot populated `{tracked, counts, drifted, errored}` — `tracked` retained
(additive, no version bump; shape-lock test updated). clippy `-D warnings` → 0; no
`unsafe`; coverage (line, touched): reconcile.rs 96.30%, cli/rollup.rs 98.65%,
orient.rs 95.57%, json.rs 94.96%, core/desired.rs 100%, core/rollup.rs 98.73%,
runner.rs 97.14% — all ≥ 90. Full workspace green (41 suites).

Rows: 7. Done: 7. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item shipped: the `Drift` model (odm-core, no reconcile dep),
report identity enrichment, `rollup` + `orient` real drift via a shared helper,
additive `--json`, drift store-read/on-demand. Every "out" item stayed out: no
`affects`/deferred/scheduled, no index changes, no drift caching in the index.

Disclosed (non-silent) consequential edits, all in the diff:
1. The A3 placeholder tests (`rollup_drift_placeholder_no_fake_data`,
   `orient_drift_placeholder`) were **repurposed** into `rollup_clean_no_drift` /
   `orient_clean_no_drift` (they seeded factless corpora → now assert "no drift"
   + placeholder-gone).
2. The `rollup_json_shape_locked` drift key-set assertion was updated from
   `["tracked"]` to the additive `["counts","drifted","errored","tracked"]`.
3. Doc prose in `odm-core` (`desired.rs`, `rollup.rs`) reworded off the literal
   `odm-reconcile` token, and `reconcile.rs`'s off the literal `load_all`, so the
   S-1/S-2 guard greps stay meaningful (same technique as slice02 G-5 / slice03).

## Deviations / decisions flagged

- **`Rollup::with_drift` builder** (construct-complete) rather than a param on
  `assemble` or post-mutation of a public field — additive, keeps `assemble` pure
  and its callers untouched. (slice-doc left the mechanism to CC; this is it.)
- **S-5 Verify wording** — the `Drift|drift` alternation matches unrelated
  odm-index prose; flagged as a Verify-amend (the `desired_facts` half is the real
  check). Not a code deviation.
- **No amendment to the model shape or `assemble` seam** was needed.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice04 deliver A-4 and satisfy A-10?** Yes to A-4 ("slice04 closed").
A-10 ("drift surfaces in rollup/orient — the A3 placeholder is gone, replaced by
real drift, and 'no drift' when clean") is a **compose** row *reproduced at arc
scale*: slice04 lands its full mechanism (placeholder gone in both views; real
drift + honest "no drift"; additive JSON), so the arc-close demo has everything it
needs. Recorded as mechanism-complete, to be reproduced at arc-close — not marked
done here (composition rows are never inherited from a slice).

**2. Performance / UX finding — `orient` (bare `odm`) now runs every probe.**
`orient` is the default command, and it now runs an on-demand `run_corpus` —
spawning every declared shell probe (and, later, network probes) on **every bare
`odm`**. For a corpus with real probes (e.g. `pg_isready -h prod`), bare `odm`
could hang or time out — a regression against the "orient is the one cheap call"
ethos (ODD-0015 §2). The plan-of-record specifies "orient folds in an on-demand
reconcile," so slice04 implements it, but this tension is real. **Recommendation
for slice07 (scheduled reconcile) / the arc:** cache the last reconcile result
(a store-side `.odm/` drift snapshot with a timestamp) and have `orient` read the
*cached* drift by default, refreshing only on `reconcile` (on demand or on the
schedule). That keeps `orient` cheap and makes drift-in-orient a *read* of
reconcile's output rather than a *run* — consistent with "files are the source,
`odm` is the build." Flagged as the sharpest thing slice04 revealed.

**3. Persisted-`ROLLUP.md` drift can go stale under early-cutoff.** `rollup`'s
slice07 early-cutoff skips rewriting the file when the corpus **meta-fingerprint**
is unchanged — but drift depends on external reality, which the fingerprint does
not cover. So a committed `ROLLUP.md` can show stale drift until the corpus meta
changes or it is force-regenerated. (Also: `compute_drift` runs the probes even
when the cutoff then skips the write — wasted work.) Tied to finding #2: if drift
is read from a cached snapshot rather than recomputed inline, both the staleness
and the wasted-probe issues dissolve. Worth settling in slice07.

**4. For slice05 (`affects` / stale-doc) and slice06 (deferred).** The shared
`compute_drift` → `odm_core::rollup::Drift` → render pattern is the template those
slices should follow: a plain-data projection in `odm-core`, a projector in
`odm-cli`, one shared helper feeding both views. slice06's `Deferred` slot is
still empty and unwired (correctly untouched here) — it should get the same
treatment (a projection + a shared computizer) when it lands.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-4 row evidence + a v1.7 version-history entry), not only here.
