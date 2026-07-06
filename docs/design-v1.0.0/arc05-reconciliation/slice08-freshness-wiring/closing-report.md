# Closing report — Slice 08 (Arc 05): freshness on every command + honest staleness (arc capstone)

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc05-slice08-freshness-wiring`, branched off
> `arc05-slice07-incremental-drift-snapshot` (slices 01–07 unmerged; the prompt's
> named fallback). Rebase onto `main` once 01–07 merge.
>
> **The last slice of A5.** slice07 built the freshness mechanism as a library;
> this slice wires it into every command, renders honest staleness, and settles
> the arc's two loose ends. On close, the **A5 arc-close** runs.

## Per-row walk

**L-1 — `reconcile_views` runs the incremental path; zero volatile probes — done
(attested).** `reconcile_views(store)` now runs
`reconcile_incremental(store, &default_drift_path(store.root()))` and projects the
resulting `DriftSnapshot` (joined with `store.load_all()` for render identity) into
`(Drift, Deferred)` — the single compute both `rollup` and `orient` call, so the
two views cannot diverge. The slice04 regression is *dissolved*, proven with a
counting `#!/bin/sh` probe: `orient_runs_zero_volatile_probes` asserts the counter
is **0** after (repeated) bare orient; `orient_reprobes_changed_input` asserts an
input-derived probe runs 1× → is carried unchanged → runs 2× only after an input
edit. Bare `odm` re-hashes only changed inputs and runs no volatile probes.

**L-2 — `odm reconcile` runs the full path — done (attested).** The explicit
command calls `reconcile_full(store, &default_drift_path(store.root()))` — the one
place volatile probes run — then projects + renders.
`reconcile_command_runs_full_refreshes_volatile` shows the counting probe going
1→2 across two reconciles, and `reconcile --json` carrying a numeric
`last_checked` on the volatile fact (the re-stamp).

**L-3 — honest-staleness rendering, additive JSON — done (attested).**
`staleness_suffix`/`humanize_age` render a volatile finding as " (last checked
Xs/Xm/Xh/Xd ago)" from the absolute `last_checked`; input-derived findings render
fresh (empty suffix). A never-checked volatile fact is modeled as
`UncheckedFact` (no outcome) and renders "not yet checked — run `odm reconcile`"
in both rollup and orient — `orient_never_checked_volatile` asserts the honest
marker **and** the absence of any "last checked" claim. JSON gains
`freshness:{kind:"fresh"|"last-checked", at?}` on drift/error entries and an
`unchecked[]` array; `rollup_json_staleness_additive` /
`orient_json_staleness_additive` assert the shapes with the `rollup/v1` ·
`orient/v1` markers **unchanged** (additive, no schema bump).

**L-4 — `.odm/drift` default path, gitignored — done (attested).**
`default_drift_path(root) = <root>/.odm/drift` (a sibling of the index's
`.odm/index`). `drift_snapshot_default_path` asserts the file exists after a
`reconcile`; `.gitignore` line 28 is `.odm/` (derived state, never truth, never
committed).

**L-5 — `ROLLUP.md` cutoff is drift-aware — done (attested).** The early-cutoff
fingerprint is now `content_fingerprint(model)` = sha256 over the whole
`RollupJson` projection (corpus tree + drift + deferred), replacing the
corpus-only meta-fingerprint. Because the JSON carries **absolute** `last_checked`
(never the relative "Xm ago"), the fingerprint is stable across wall-clock ticks.
`rollup_regenerates_on_drift_change` toggles a file-probe flag (a drift change with
**no** corpus change) and asserts `ROLLUP.md` regenerates;
`rollup_skips_when_drift_and_corpus_unchanged` asserts two rollups over an
unchanged corpus+drift are byte-identical (skipped). Closes slice04 finding #2.

**L-6 — `next` stays graph-pure — done (attested).** `next` answers graph
readiness (deps + gates) and is deliberately unaffected by a `deferred` marker;
deferral is surfaced only in the reconcile-aware views.
`next_unaffected_by_deferred_marker` seeds a deferred, dependency-free slice and
asserts it still appears in `odm next` (not withheld, exit 0). The layering
boundary is documented in a doc-comment on `commands::next` and in this slice's
docs. Withholding was rejected: it would require indexing the marker (a rejected
schema invariant / the `FORMAT_VERSION` path) or running a reconcile inside `next`
(which would re-introduce the slice04 hot-path regression). Graph-ready ≠ parked;
both surfaces are correct at their own layer.

**L-7 — gates + no index change — done (attested).** `git diff --stat --
crates/odm-index/` is empty; clippy `--all-targets --all-features -- -D warnings`
exits 0; no `unsafe` in `crates/odm-cli/src` or `crates/odm-reconcile/src`. Line
coverage on the touched paths: `json.rs` 100%, `rollup.rs` 96.96%, `orient.rs`
94.60%, cli `reconcile.rs` 93.48%, `incremental.rs` 98.21%, `snapshot.rs` 93.65%
— all ≥ 90. Full workspace green (44 suites).

Rows: 7. Done: 7. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item shipped: the incremental wiring of `reconcile_views`, the
full-path wiring of `odm reconcile`, honest-staleness rendering (human + additive
JSON), the `.odm/drift` default path + gitignore, the drift-aware `ROLLUP.md`
cutoff, and the settled graph-pure `next`. Every "out" item stayed out: **no**
`odm-index` change, **no** schema bump, **no** `next`-withholding, **no** new
probe class or amendment to ODD-0019.

The only scope *movement* is disclosed, not dropped: six slice06 deferred tests
now run `odm reconcile` before asserting a volatile re-entry fact's ready/waiting
split (see Deviation 2) — a consequence of L-1, propagated into the tests rather
than left to false-pass.

## Deviations / decisions flagged

1. **Two store reads per bare command.** `reconcile_incremental` loads the corpus
   to probe; `reconcile_views` then loads it again for render identity
   (number/name/describe), which the lean snapshot omits by design. Both are cheap
   reads, **not probes** — the L-1 zero-volatile-probes invariant is untouched.
   Chose the join over fattening the snapshot (slice07's recommendation). An
   accepted read-cost tradeoff.
2. **Pre-slice08 deferred tests now `reconcile`-first.** slice06's ready/waiting
   tests used a *volatile* re-entry fact and relied on bare `odm rollup`/`orient`
   probing it live. Under L-1 that no longer happens, so a never-checked re-entry
   fact honestly reads "waiting (not yet checked)". The tests (in
   `crates/odm-cli/tests/deferred.rs`) were updated to run `odm reconcile` first,
   restoring the genuine `true`→ready / `false`→waiting split. This is a real
   slice06→08 behavior change (bubbled up below), disclosed here, not a silent
   test massage. The four drift-render tests (rollup/orient) got the same
   treatment for the same reason.
3. **No amendment to ODD-0019.** The incremental wiring, the
   `Freshness`/`UncheckedFact` model additions to `odm_core::rollup`, and the
   drift-aware cutoff all fit the design of record as written.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice08 deliver A-8, and is the arc complete?** Yes. A-8 (freshness
wired into every command + honest staleness + the two settled loose ends) is a
tested capability: bare commands run zero volatile probes, `odm reconcile` is the
sanctioned refresh, staleness is rendered honestly (human + additive JSON), and
both loose ends (`next` graph-pure; `ROLLUP.md` cutoff drift-aware) are settled
with tests + docs. **All 8 slices of A5 are delivered** — the **arc-close is
next** (composition check A-1…A-13; the class-(b) compose rows reproduced at arc
scale). A-8 stays `attested` until CDC reproduces.

**2. What the wiring revealed for the arc.**
  - **The projector was the load-bearing seam, exactly as slice07 predicted.**
    Flipping `reconcile_views` from `run_corpus` to `reconcile_incremental` +
    snapshot join changed the compute of the whole drift/deferred surface without
    a single caller edit. The arc's decision (slice04) to route both rollup and
    orient through one shared projector is what made the capstone a one-seam change
    — worth recording as an arc-level win.
  - **Deferred re-entry inherited freshness semantics.** A deferred node's
    `reenter_when` fact is now resolved from the *snapshot*, so an **input-derived**
    re-entry predicate is always-fresh at near-zero cost, while a **volatile** one
    carries honest staleness ("waiting (not yet checked)" until an explicit
    reconcile). This is the correct end-state, but it **changes slice06's observable
    behavior**: `odm rollup`/`orient` no longer resolve a volatile re-entry fact
    live. The arc-plan's A5 narrative should note that deferred re-entry for a
    *volatile* predicate requires `odm reconcile` (by design — the same honesty rule
    as drift).
  - **The drift-aware cutoff completes slice04 finding #2.** A persisted
    `ROLLUP.md` can no longer hide stale drift: the cutoff now keys on the drift
    projection, and incremental drift is cheap enough (slice07) to compute on every
    rollup. The arc closes the "cutoff hides stale drift" gap it opened.

**3. Silent-drop diff at the arc altitude.** Nothing the arc-plan scoped for A5 is
missing. The one behavior change (volatile deferred re-entry needs an explicit
reconcile) is a *consequence* of the arc's own honest-staleness principle, not a
dropped requirement — surfaced here so the arc-plan records it rather than a reader
discovering it.

**4. Reusable finding.** The "absolute stamp in the snapshot, relative render in
the view, fingerprint over the absolute-stamped projection" split is the general
recipe for a *time-sensitive value behind a content-addressed cutoff*. Any future
derived-and-cached view with a freshness dimension should copy it.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-8 row evidence + a v2.2 version-history entry), not only here.
