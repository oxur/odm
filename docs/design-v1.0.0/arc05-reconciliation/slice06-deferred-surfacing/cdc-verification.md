# CDC Verification — Arc 05 / Slice 06: deferred surfacing + re-entry predicate (Q-A3-1)

> Independent verification of CC's closed ledger (impl + close on
> `arc05-slice06-deferred-surfacing`, commits `7ce7f0d` + `34df7f9`), per
> LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo rows
> route to CI / a local 1.85+ run.

## Environment constraint (disclosed)

CDC sandbox has no 1.85+ toolchain; CC built + ran on local 1.95.0 (attested → reproduced on
CI). Branch cut off `arc05-slice05-…` (01–05 unmerged) — rebase onto `main` when they merge.

## Row dispositions

**Row count:** 7 opened, 7 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **D-1** — `Deferral { because, reenter_when }` on `Frontmatter` (`frontmatter.rs:448`),
  `with_deferred`/`deferred()`; `reenter_when` references a `desired_fact` by id; absent ⇒
  `None`/no key on emit (proptest `modeled` list extended). `deferred_marker_round_trip`
  present. ✔
- **D-2** — `project_deferred` (`reconcile.rs:111`) over `run_corpus`: referenced fact Holds
  → ready; Drift/Error → waiting. `deferred_ready_when_reenter_fact_holds` +
  `deferred_waiting_when_reenter_fact_drifts` present. ✔
- **D-3** — `odm_core::rollup::Deferred { nodes: Vec<DeferredNode { …, because, reentry:
  Reentry } > }` (`rollup.rs:237`) + `Rollup::with_deferred`; rollup renders `## Deferred`;
  none → no section. `rollup_surfaces_deferred` + `rollup_no_deferred_section_when_none`
  present. ✔
- **D-4** — orient `DEFERRED` section via the same projector; none → no section.
  `orient_surfaces_deferred` + `orient_no_deferred_when_none` present. ✔
- **D-5 (the invariant guard)** — `git diff --stat -- crates/odm-index/` → **empty**. The
  deferred projector reads the marker off `NodeReport` (enriched in `run_node` from the store
  frontmatter) via `reconcile_views`/`run_corpus`, never `index_frontmatters`. No
  `IndexRecord`/adapter/`FORMAT_VERSION` change — **A4 invariant honored by non-triggering.** ✔
- **D-6** — `ROLLUP_SCHEMA`/`ORIENT_SCHEMA` strings **unchanged** (`json.rs:26,28`); the
  `deferred` slot populated additively; `Violation::DanglingReenterWhen` (`check.rs:111`) →
  Warning via `violation_severity` (`commands.rs:1026`), never a panic.
  `{rollup,orient}_json_includes_deferred` + `check_flags_dangling_reenter_when` present. ✔
  (see ruling 3 on the new orient/v1 key)
- **D-7 (no `unsafe`)** — grep empty across the touched crates. ✔

**Attested by CC (local 1.95.0), pending CI:** clippy `-D warnings` → 0; touched-path line
coverage ≥ 91.7% (commands.rs 91.67% the floor); 42 suites green. → **PENDING CI.**

## Rulings on CC's flagged decisions

1. **`reconcile_views` — one `run_corpus`, two views (drift + deferred). Accepted, and the
   standout of the slice.** Refactoring `compute_drift` into `reconcile_views(store) →
   (Drift, Deferred)` means adding deferred **did not double the per-command probe cost** —
   both projections come from a single reconcile. This directly answers slice04's
   orient-runs-probes finding *and* gives 07/08 a **single seam** to make incremental. Exactly
   the right structural move, not scope creep.
2. **`NodeReport` enriched with the deferred marker; `run_corpus` keeps a node if it has
   facts *or* a deferred marker.** Accepted — a deferred node with no other facts still needs
   surfacing; parallels slice04's identity enrichment. Correct.
3. **orient/v1 gains a *new key* (vs slice04 populating an existing empty slot). Accepted —
   within the additive convention.** Adding a top-level key is backward-compatible for any
   consumer that ignores unknown fields (the same non-breaking rule that lets slice05 add a
   finding kind and slice04 fill a slot). The **schema version string is unchanged** (correct
   — a version bump signals a *breaking* change, which this is not); the shape-lock test was
   updated. Noted: this is A5's first *new key* on an A3 schema (vs filling a slot) — still
   additive, correctly handled.
4. **`DanglingReenterWhen` = Warning, not Error.** Accepted — a node with a typo'd
   `reenter_when` is *still validly parked*; the dangling reference is advisory, not a
   structural defect that should fail `check`. Reuses slice05's `violation_severity` arm.
   Consistent.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-6: the deferred marker + re-entry predicate + surfacing;
  A-12 **mechanism-complete** (reproduce at arc-scale, not inherited).
- **Silent-drop diff honest?** ✔ — 7/7; the `next`-withholding limitation, the orient/v1
  new-key, and the DanglingReenterWhen severity are all disclosed.
- **Findings + arc-plan?** ✔ — CC propagated the bubble-up itself (A-6 + A-12 + arc-plan
  v2.0), with three sharp 07/08 carries: (a) the re-entry predicate is an ODD-0019 probe →
  fold it into the incremental snapshot **via the `reconcile_views` seam** (already the single
  choke-point); (b) settle the `next`-withholding limitation where the marker's snapshot home
  is decided anyway (not a one-off index change now); (c) an honesty follow — distinguish
  "waiting (couldn't check)" from "waiting (drifted)" once staleness stamps land. All correct.

## Verdict

**Arc 05 / Slice 06 CDC-verified on structure; all four flags ruled; cargo rows pending CI.**
Q-A3-1 is cashed: a node parks itself with a checkable re-entry condition, and rollup/orient
answer "ready to resume?" via the *same reconcile* that answers drift — with zero index
change (D-5 empty diff) and the `reconcile_views` refactor keeping the per-command probe cost
flat. The slice04/05 patterns composed with nothing new invented. A-6 attested-on-close;
flips `done` on CI green. **A5 at 6/8 — the original slice set (01–06) is complete.**

**Only the freshness pair remains (07/08 — the ODD-0019 heart), then the arc-close.** And CC
has teed them up well: `reconcile_views` is the single seam to make incremental, and the
`next`-withholding + "couldn't-check" honesty questions are routed to land where the drift
snapshot's home is decided.

CDC: planning thread, 2026-07-02. Iterations used: 1.
