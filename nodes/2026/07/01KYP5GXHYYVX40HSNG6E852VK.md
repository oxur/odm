---
id: 01KYP5GXHYYVX40HSNG6E852VK
number: 579967500
type: artifact
schema: artifact/v1.1
name: 'Closing report — Slice 06 (Arc 05): deferred surfacing + re-entry predicate (Q-A3-1)'
created: 2026-07-02
updated: 2026-07-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice06-deferred-surfacing/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKDCZAXRFXETY8RQX4
---
# Closing report — Slice 06 (Arc 05): deferred surfacing + re-entry predicate (Q-A3-1)

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc05-slice06-deferred-surfacing`, branched off
> `arc05-slice05-affects-stale-doc` (slices 01–05 unmerged; the prompt's named
> fallback). Rebase onto `main` once 01–05 merge.

## Per-row walk

**D-1 — `deferred` marker — done (attested).** `Deferral { because, reenter_when }`
on `Frontmatter` (`with_deferred`/`deferred()`), `#[serde(skip_serializing_if =
"Option::is_none")]`. `deferred_marker_round_trip` → ok: round-trips; absent ⇒
`None` and no `deferred:` key. `reenter_when` references one of the node's own
`desired_facts` by id (reuses slice01 — no inline probe).

**D-2 — re-entry predicate reconcile-evaluated — done (attested).**
`project_deferred` resolves `reenter_when` to the fact's outcome over the same
`run_corpus` drift uses: `Holds` → `Reentry::Ready`; `Drifted`/`Error`/missing →
`Reentry::Waiting { describe }`. `deferred_ready_when_reenter_fact_holds`
(`true`→ready) + `deferred_waiting_when_reenter_fact_drifts` (`false`→waiting) → ok.

**D-3 — fill the `Deferred` slot + rollup — done (attested).** `odm_core::rollup::
Deferred` reshaped `{ nodes: Vec<DeferredNode> }` + `Rollup::with_deferred`;
`odm rollup` renders a `## Deferred` section (because + status); none deferred →
no section (no fabricated data). `rollup_surfaces_deferred` +
`rollup_no_deferred_section_when_none` → ok.

**D-4 — orient — done (attested).** orient's `DEFERRED` section (same shared
`reconcile_views` projector); none → no section. `orient_surfaces_deferred` +
`orient_no_deferred_when_none` → ok.

**D-5 — store-overlay, no index change — done (attested).** The marker is carried
on `NodeReport` (enriched in `run_node` from the store frontmatter) and read via
`reconcile_views`/`run_corpus` — never `index_frontmatters`. `git diff --stat --
crates/odm-index/` is empty; no `IndexRecord`/adapter/`FORMAT_VERSION` change. The
A4 invariant stays honored by non-triggering.

**D-6 — additive `--json` + dangling check — done (attested).** `rollup/v1`'s
empty `deferred` slot is populated; `orient/v1` gains a `deferred` key — both
additive (shape-lock tests updated, no version bump). A dangling `reenter_when`
is `Violation::DanglingReenterWhen` (Warning via slice05's `violation_severity`),
never a panic. `rollup_json_includes_deferred` + `orient_json_includes_deferred`
+ `check_flags_dangling_reenter_when` → ok.

**D-7 — gates — done (attested).** clippy `-D warnings` → 0; no `unsafe`;
coverage (line) all touched files ≥ 90 (frontmatter 99.5%, core/rollup 98.8%,
check 99.3%, runner 97.2%, cli/reconcile 95.6%, cli/rollup 98.7%, orient 95.4%,
json 95.5%, commands 91.7%). Full workspace green (42 suites).

Rows: 7. Done: 7. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item shipped: the `deferred` marker + round-trip, re-entry
predicate evaluation (ready/waiting), the filled `Deferred` slot surfaced in both
rollup and orient via one shared projector, additive `--json`, store-overlay
read (no index change), and the dangling-`reenter_when` check finding. Every
"out" item stayed out: **no `next`/graph withholding** of deferred nodes (the
documented limitation), no freshness/probes work (07/08), no index change.

Disclosed (non-silent) consequential edits, all in the diff:
1. `compute_drift` was refactored to `reconcile_views` (project both `Drift` and
   `Deferred` from one `run_corpus`) — so drift + deferred share one probe run.
   `rollup`/`orient` call sites updated accordingly.
2. `NodeReport` gained a `deferred: Option<Deferral>` field (as slice04 added
   identity/`describe`), and `run_corpus` now keeps a node if it has facts **or**
   a deferred marker (so a deferred node always surfaces).
3. The `orient/v1` shape-lock test's key set gained `deferred`; the frontmatter
   proptest `modeled` list gained `deferred` (an 8-char key the generator can
   produce — a latent flake, now guarded).

## Deviations / decisions flagged

- **`orient/v1` gains a new top-level key** (`deferred`), where slice04's drift
  was populating an *existing* empty slot. The ledger D-6 sanctions this ("absent
  in A3 → populated; no version bump"); adding a key is backward-compatible for
  consumers pinned on `orient/v1`. Flagged as the one place "additive" means a
  new key rather than a filled slot.
- **`DanglingReenterWhen` is a Warning, not an Error** — unlike other dangling
  edges (which are Errors). Per the ledger (reuses `violation_severity` → Warning):
  the node is still validly parked, so it's advisory. Flagged (the inconsistency
  with `DanglingEdge` is deliberate; a later slice could reconsider).
- **No amendment** to the marker shape or the `reenter_when`-by-id coupling was
  needed — the `fact_id` reference read cleanly.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice06 deliver A-6 and the A-12 mechanism?** Yes to A-6 ("slice06
closed"). A-12 ("deferred nodes surfaced with a checkable re-entry predicate —
the Q-A3-1 deferral cashed") is a **compose** row *reproduced at arc scale*:
slice06 lands its full mechanism (marker + predicate eval + rollup/orient
surfacing + additive JSON). Recorded as mechanism-complete, to be reproduced at
arc-close (never inherited from a slice).

**2. What it reveals for slices 07/08 (the freshness rework, ODD-0019).**
  - **The re-entry predicate is exactly an ODD-0019 probe** — today `project_deferred`
    evaluates it via `run_corpus` (store-read, on-demand), the clean disclosed
    interim. When 07/08 land the incremental drift snapshot, the re-entry
    predicate should fold into the **same** machinery: its outcome becomes a
    cached, incrementally-refreshed fact, and "ready to re-enter" is read from the
    snapshot, not recomputed. A re-entry predicate that is *input-derived*
    (ODD-0019's dominant class — e.g. "the blocking file now exists") becomes
    always-fresh for free; a *volatile* one carries the honest "last checked"
    staleness. **Recommendation:** 07/08 route deferred re-entry through the drift
    snapshot alongside drift — `reconcile_views` is already the single seam to
    make incremental.
  - **`reconcile_views` is the consolidation point** the freshness work needs:
    both views now flow through one `run_corpus`; making that call incremental
    upgrades drift *and* deferred at once.

**3. The `next`-withholding limitation (documented, out of scope).** A deferred,
dependency-ready node still appears in `next` (surfacing-only this slice).
Withholding needs the index-backed graph reader to see the marker → indexing it →
the A4 invariant + a `FORMAT_VERSION` bump (the rejected alternative). This is the
one place deferred would justify touching the index; **recommend it be settled
with the freshness rework** (07/08), where the marker's incremental-snapshot home
is decided anyway — bundling the "should `next` withhold deferred?" decision there
avoids a one-off index change now.

**4. Honesty nuance for a follow.** `project_deferred` collapses `Drifted` and
`Error` re-entry outcomes both into `Waiting` (per D-2). Once the freshness model
distinguishes "checked, not ready" from "couldn't check" with a staleness stamp,
the deferred view could surface "waiting (last checked Xm ago)" for volatile
re-entry predicates — a small honesty upgrade, naturally part of 07/08.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-6 row evidence + a v2.0 version-history entry), not only here.
