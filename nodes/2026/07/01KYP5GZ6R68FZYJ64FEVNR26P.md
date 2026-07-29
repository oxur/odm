---
id: 01KYP5GZ6R68FZYJ64FEVNR26P
number: 513704300
type: artifact
schema: artifact/v1.1
name: 'CDC Verification — Arc 05 / Slice 08: freshness on every command + honest staleness'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice08-freshness-wiring/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKT5WD5R931YMT2FNE
---
# CDC Verification — Arc 05 / Slice 08: freshness on every command + honest staleness

> Independent verification of CC's closed ledger (impl + close on
> `arc05-slice08-freshness-wiring`, commits `f4b5ac7` + `2b5f34b`), per LEDGER-DISCIPLINE
> v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo rows route to CI /
> a local 1.85+ run. **Arc capstone — the A5 arc-close runs after this.**

## Environment constraint (disclosed)

CDC sandbox has no 1.85+ toolchain; CC built + ran on local 1.95.0 (attested → reproduced on
CI). Branch cut off `arc05-slice07-…` (01–07 unmerged) — rebase onto `main` when they merge.

## Row dispositions

**Row count:** 7 opened, 7 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **L-1** — `reconcile_views` → `reconcile_incremental(store, default_drift_path)`
  (`reconcile.rs:101–102`); `orient_runs_zero_volatile_probes` + `orient_reprobes_changed_input`
  present (counting probe stays 0 on bare orient). The slice04 regression is dissolved. ✔
- **L-2** — `odm reconcile` → `reconcile_full` (`reconcile.rs:216`);
  `reconcile_command_runs_full_refreshes_volatile` present (volatile probe 1→2, `last_checked`
  re-stamped). ✔
- **L-3** — `Freshness`/`FreshnessJson` + `UncheckedFactJson` (`json.rs:207–265`);
  `staleness_suffix`/`humanize_age` render "last checked Xs/Xm ago"; never-checked →
  "not yet checked — run `odm reconcile`". `--json` additive (`freshness:{kind,at}` +
  `unchecked[]`); schema markers unchanged. Tests present. ✔ (see ruling 3)
- **L-4** — `default_drift_path(root) = <root>/.odm/drift` (`incremental.rs:27`); `.gitignore`
  carries `.odm/` (line 28). ✔
- **L-5** — cutoff keys on `content_fingerprint` = sha256(`RollupJson`) (corpus+drift+deferred,
  absolute stamps); `rollup_regenerates_on_drift_change` + `rollup_skips_when_drift_and_corpus_unchanged`
  present. Settles slice04 finding #2 — a persisted rollup can't hide stale drift. ✔
- **L-6** — `next` graph-pure doc-comment (`commands.rs:1481`: "which nodes are graph-ready …
  deliberately [unaffected by deferred]"); `next_unaffected_by_deferred_marker` present. ✔
- **L-7 (the invariant guard + no unsafe)** — `git diff --stat -- crates/odm-index/` →
  **empty**; grep `unsafe` → none. **A5's zero-index-change streak holds across all 8
  slices.** ✔

**Attested by CC (local 1.95.0), pending CI:** clippy `-D warnings` → 0; touched-path line
coverage ≥ 93.5% (cli reconcile.rs 93.48% the floor); full workspace green (44 suites);
12 new slice08 tests. → **PENDING CI.**

## Rulings on CC's flagged items

1. **Volatile deferred re-entry now needs an explicit `reconcile` — a real slice06→08
   behavior change (7 pre-existing tests updated). Accepted, correct, and honestly bubbled.**
   This is the *direct, honest consequence* of the freshness model, not a regression: bare
   commands run the incremental path, and volatile facts (including a **volatile** re-entry
   predicate — a `shell` probe without `inputs`) are not auto-run — they carry last state +
   honest staleness until `odm reconcile`. An **input-derived** re-entry predicate (a `file`
   probe, or `shell`+`inputs`) *still* auto-refreshes incrementally. So a deferred node with a
   volatile re-entry reads "waiting (last checked Xm ago)" on bare `odm` and flips to "ready"
   after an explicit reconcile — exactly ODD-0019 §3.4. CC updated the 7 tests to the new
   correct behavior (not disabled/weakened) and disclosed it as a bubble-up. This is the
   discipline working: a cross-slice behavior change surfaced, not buried. ✔
2. **Two corpus reads per bare command (`reconcile_incremental` `load_all`s to probe, then
   `reconcile_views` `load_all`s again for render identity — number/name/describe the lean
   snapshot omits). Accepted as an interim; flagged follow.** It is O(corpus) *frontmatter*
   reads (not probes — the expensive thing we eliminated), and bare orient already walks the
   corpus once; the second walk is the overhead. Negligible for odm's own corpus; at scale
   it's an optimization (unify the two loads, or have the snapshot carry render-identity like
   the slice04 report enrichment). Not blocking — recorded as a **post-arc / A4-follow
   optimization**, mirroring the slice03 double-load lineage. (Worth carrying to the arc
   bubble-up.)
3. **Absolute stamps stored, relative "Xm ago" rendered.** Accepted — a clean choice: the
   snapshot/`content_fingerprint` use the absolute `last_checked`, so the drift-aware cutoff
   is **stable across wall-clock ticks** (regenerates on a real drift change, byte-identical
   when nothing moved). Modeling never-checked as a distinct `UncheckedFact` (no outcome) makes
   "no fabricated fresh" fall out of the type — there's no path that prints "fresh" for
   something never probed. Good.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-8: the freshness wiring + honest staleness + both loose ends
  settled. ODD-0019 is realized end-to-end; the slice04 regression is gone.
- **Silent-drop diff honest?** ✔ — 7/7; the volatile-re-entry behavior change, the two-reads
  cost, and the absolute-stamp choice are all disclosed.
- **Findings + arc-plan?** ✔ — CC propagated the bubble-up itself (arc-plan A-8 + A-9
  mechanism-complete + v2.2). **All 8 slices delivered → the arc-close is next.**

## Verdict

**Arc 05 / Slice 08 CDC-verified on structure; all three flags ruled; cargo rows pending
CI.** The capstone lands ODD-0019 end-to-end: bare `odm` runs the incremental pass (zero
volatile probes — the slice04 regression dissolved), volatile facts show honest "last checked
Xm ago", `odm reconcile` is the explicit refresh, the `ROLLUP.md` cutoff is drift-aware (no
more hidden stale drift), and `next` stays graph-pure. **A5's zero-index-change streak held
across all eight slices** — the store-vs-index boundary drawn in slice02 proved right the
whole way. A-8 attested-on-close; flips `done` on CI green. **A5 at 8/8 — all slices
delivered.**

**Next: the A5 arc-close** — a CDC-assembled arc-scale unit (not CC's to run): the
composition check across A-1…A-13, the class-(b) rows (A-8…A-12) *reproduced at arc scale*,
`arc05-reconciliation/closing-report.md`, an **independent gate** (fresh-context subagent, as
in A4), and the bubble-up to `project-plan.md`. One flagged follow to carry into it: the
two-reads-per-bare-command optimization.

CDC: planning thread, 2026-07-06. Iterations used: 1.
