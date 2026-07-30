# Slice 13 closing report — Live reconcile + vision mint

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 13 · **Feeds:** MF-7 (vision live),
> MF-9 (fidelity on the live corpus) · **Realizes:** ODD-0025 §2.1/§2.3/§2.9 fired against
> `.worktrees/odm` · **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (F-1…F-10) ·
> **Implemented by:** CC · **Date:** 2026-07-30
> **Branches:** `odm` (the store — one commit, `26bea1d`, atop known-good `2fc25f5`) and `release/1.0.x`
> (the thin CLI wiring — four commits: `595f24f` wire, `dffd3d4` fix #1, `2533330` fix #2, `d949786`
> coverage). **Evidence class:** class-(b) (LEDGER-DISCIPLINE v2.0 §B) — the committed store is the
> evidence; CDC reproduces by direct read.

## What shipped

**The live store, reconciled and vision-minted.** `.worktrees/odm` moved from `2fc25f5` to `26bea1d`:
8 nodes reconciled in place (2 re-snapshotted, 5 source-reconciled — 3 by the design/research family, 2
by the plan-set self-host family), 4 nodes created (the 3 newly-authored slice11/12/13 plan nodes, plus
the new 1:1 `project-plan` node `#1001`). `#1000` (the project) is now the editorial-merge synthesis
superseding `#1001`, with a recorded attestation and a clean `supersedes` edge. Nothing outside `nodes/`
changed; the fire was preceded by an adjudicated dry-run and followed by a pre-commit idempotence
re-run, per the s07/s10 live-mutation protocol.

**The thin CLI wiring this required.** `self_host_inner` now calls s12's `selfhost::reconcile` between
`repair()` and `self_host()`; the default (design/research) `migrate` flow calls s12's
`mapping::reconcile_source` between `backfill_source` and `canonicalize_source_paths`. A new `--vision`
flag drives a new `vision()` orchestration function: it locates the project node, mints the faithful 1:1
`project-plan` node at a fixed reserved number (`VISION_PLAN_NUMBER = 1001`, verified free on the live
corpus before use), and calls s12's `apply_project_vision` to re-cast the project's own identity as the
synthesis. No new fidelity or synthesis logic — every fix this slice needed lived at this call-site
layer, never inside s12's library functions.

**Two real bugs, caught by the gate the cc-prompt insisted on.** The dry-run adjudication step is not
a formality this slice — it's what caught both of the following before either reached the live store:

1. `vision()`'s first draft required the project node's own `fm.source()` to build the 1:1 clone's
   source record. The project structurally never carries one (ODD-0025 §2.3 — its body is meant to
   *become* the synthesis, so self-host's source-population path excludes it outright, the same
   treatment a retired node gets). On the live corpus this made `--vision` fail immediately
   ("has no `source` yet"). Fixture tests never caught it, because self-hosting a *brand-new* plan set
   happens to stamp a source onto the project at creation time — masking the gap that only a corpus
   whose project node predates that code path (as odm's own does) exposes. Fixed to read
   `project-plan.md` fresh off disk instead, the same way every other migrated node is sourced
   (`dffd3d4`).
2. The idempotence re-run, done deliberately *before* committing, surfaced a second gap:
   `build_synthesis` never calls `stamp_schema()`. The re-cast `#1000` silently lost its
   `schema: project/v1.0` marker entirely — invisible until an unrelated `migrate` upgrade pass quietly
   patched a schema back in on the very next run, a collateral mutation a true no-op re-run must not
   produce. Fixed by stamping the synthesis frontmatter explicitly, mirroring the call already made for
   the 1:1 node (`2533330`). The store was reset to `2fc25f5` and re-fired clean rather than hand-patched.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | Pre-flight | **done** | Green build, clean store at `2fc25f5`, before-manifest captured (369 files, `43dac94b…`). |
| F-2 | Dry-run adjudicated | **done** | All three dry-runs matched the expected shape exactly; fingerprint stable; caught bug #1 before fire. |
| F-3 | Reconcile every drifted node | **done** | 7 nodes reconciled; recomputed gate → 0 drifted remain; id/edges/status preserved. |
| F-4 | Moved-source re-discovery live | **done** | ODD-0017/0018 `source.paths` rewritten to `04-accepted/`-relative, both resolve. |
| F-5 | Vision minted | **done** | `#1001` verbatim 1:1 (426/426 lines byte-identical); `#1000` synthesis with attestation + clean supersedes lineage. |
| F-6 | No collateral; idempotent | **done** | Idempotence checked *before* commit — this is what caught bug #2. Re-run after commit: 0/0/0. `orient`/`rollup` byte-stable. |
| F-7 | `check` green + guard intact | **done, deviation disclosed** | `absolute-source-path` clean (0 findings). `check` is exit 1 — 14 pre-existing `uncovered-doc` errors, confirmed identical at `2fc25f5` before this slice touched anything (not a regression). |
| F-8 | One revertible commit; rollback discipline | **done** | `26bea1d` atop `2fc25f5`; `reset --hard` exercised once for real, mid-slice, for the schema fix. No arc-close/P-12/L-8a touched. |
| F-9 | No model drift | **done** | No ODD-0025 edit; gate stayed migration-time-only; s12's §2.9 policy unchanged. |
| F-10 | Clippy/unsafe/coverage | **done, deviation disclosed** | Clippy clean, no `unsafe`, fmt clean throughout. New code's two previously-uncovered branches now tested; `migrate.rs`'s file-aggregate coverage (62.6%/67.7%/50%) stays below the 90% floor, dominated by pre-existing `--replan`/`coverage()` code out of this slice's "thin wiring only" scope. |

**Rows: 10. Done: 8. Done-with-disclosed-deviation: 2 (F-7, F-10 — both pre-existing conditions, not
regressions this slice introduced). Deferred: 0. No-op: 0.** No silent drops: the MF-9 composition
check, the P-12 demo, the final reconcile-and-freeze, and L-8a are all confirmed untouched (F-8).

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace --all-features` | 0 failed, checked after every code checkpoint (4 separate runs across the two fixes) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean (only the pre-existing, unrelated `proc-macro-error2` notice) |
| `cargo fmt --check` | clean |
| `unsafe` in `crates/odm-cli/src/migrate.rs` | none |
| `llvm-cov -p odm-cli` | `migrate.rs` new code (`vision()`, `render_reconcile()`) exercised by 7 fixture tests; file aggregate 62.6%/67.7%/50% — pre-existing gap, disclosed at F-10 |
| Dry-run fingerprint (sha256 composite, 369 files) | `43dac94b…` unchanged across all three dry-runs |
| Live fire vs. dry-run counts | exact match on all three invocations (2/3/0, 2/0/3, 1+1) |
| Pre-commit idempotence re-run | caught the schema-stamp bug; after the fix, 0 additional file changes |
| Post-commit idempotence re-run | `0/0/0` on all three commands; store `git status` empty |
| `odm orient` ×2 | byte-identical |
| `odm rollup` ×2 | `ROLLUP.md` byte-identical (second run self-reports "unchanged — skipped") |
| `odm check` `absolute-source-path` | 0 findings |
| `odm check` overall | exit 1 (14 pre-existing `uncovered-doc` errors — see F-7) |
| Pre-existing-vs-regression check for F-7 | scratch worktree at `2fc25f5`, `odm.toml` locator pointed at it temporarily, `check` run: identical 14 errors, identical exit 1 |

## Deviations / findings (flagged, per the working agreement)

### F-7: `check` is not exit 0 — a pre-existing, confirmed-not-a-regression gap

The live corpus has 14 `uncovered-doc` errors for slice11/12/13's own artifact-family docs
(`cc-prompt.md`/`ledger.md`/`closing-report.md`/`cdc-verification.md`/`slice-doc.md`), because
`--artifacts` mint-all hasn't run since s10 closed. This is not something s13's reconcile-or-vision work
caused: checked out `2fc25f5` in a disposable worktree (temporarily repointing `odm.toml`'s locator,
then reverting it) and ran `check` against that exact pre-fire state — identical 14 errors, identical
exit 1. Fixing it means running `--artifacts` mint-all, which is s09/s10's capability, not s12's or
s13's, and was never part of F-2's adjudicated dry-run set — pulling it in now would itself have been
the "no new capability" violation the cc-prompt warns against. Routed to the arc-close, which should
pick up a fresh `--artifacts` run before P-12's final demo.

### F-10: `migrate.rs`'s file-aggregate coverage stays below the 90% floor — pre-existing, not new surface

The uncovered regions concentrate almost entirely in `replan()`/`render_replan()` (the older `--replan`
mechanism, untouched this slice) and `coverage()` (the `--coverage` arm, likewise untouched) — not in
this slice's additions. The new code (`vision()`, `render_reconcile()`, the `self_host_inner`/`migrate()`
wiring) is exercised by 7 fixture tests, including two added specifically to close gaps this slice's own
work left (the no-project-node error path, tags/component carry-through onto the 1:1 node). One small,
disclosed gap remains: `render_reconcile`'s `(moved, false)`/`(moved, drifted)` note-format arms have no
fixture, since no test simultaneously moves and drifts a self-hosted plan node's file — the live corpus
hit that shape only via the *design/research* reconcile path (0017/0018), not the self-host one. Not
chased further, consistent with how the same kind of disclosure was made for `mapping.rs`/`selfhost.rs`
at s12's close.

### Two live bugs found and fixed — not silently absorbed

Both are described in full under "What shipped" above and carried as their own ledger Notes (F-2, F-5).
Named again here because LEDGER-DISCIPLINE calls for deviations to be visible in one place: neither bug
was in scope to *predict* going in — both were surfaces the adjudication protocol is specifically
designed to catch, and both were caught exactly where the protocol says they should be (the dry-run, and
the pre-commit idempotence check), not downstream.

### Post-close correction: a third bug, missed by CC's own verification, caught by the operator

This slice's original F-6/F-7 verification recorded `odm check` at 14 errors post-fire — a mis-read.
The real count was 15: `#1000`'s re-cast `edges.supersedes` is invalid on a `project` node per
ODD-0020 §2's work/document field split (`project` is a work type; `supersedes` is document-only).
`apply_project_vision`/`build_synthesis` sets `edges.supersedes` with no type check — a latent
conflict between ODD-0025 §2.3 and ODD-0020 §2 that nothing before this slice had exercised on the
same node through `check`. **The operator caught it**, not CC: asking why `find docs -name '*.md'
| wc -l` (388) didn't reconcile with `node list --all`'s footer (373) led to a fresh `check` run
that surfaced the miscount directly. Resolved same-day, operator-approved: `check_field_validity`
now exempts a work node carrying `source.synthesis` from the `supersedes`/`affects` checks, keyed
on that field rather than on `NodeType::Project`; ODD-0020 amended to v1.4; a fixture test proves
both directions (`release/1.0.x@83acedb`). No live data changed — the store's `supersedes` edge was
always correct under the intended model, only the validator's rule was wrong. `check` on the
committed `26bea1d` store now shows exactly the 14 errors the original evidence claimed. **This is
recorded as a real gap in CC's own verification discipline, not smoothed over**: the "F-6/F-7
verified on the committed store" claim in this report's Verification table was, at close time,
false by one finding — see `ledger.md`'s "Post-close correction" section for the full account.

**A related `[no-vision]` regression, same investigation, also resolved same day.** `#1000` and
`#1001` both triggered it: `apply_project_vision`'s body never carried the literal `# Vision`
heading `check`'s L-3b rule and `orient`'s excerpt require (fixed via a shared
`synthesis::vision_body()` helper; the live `#1000` refreshed in place, `odm@e1e94bf`); and L-3b
itself iterated every `NodeType::Project` node — safe before the vision mint (there was only ever
one), broken once it created a second, `#1001`, whose verbatim 1:1 body can never carry an
injected heading (fixed by exempting a superseded project node, `release/1.0.x@e4508e2`). `check`
on the committed store now shows 8 warnings (was 10), 14 errors (unchanged).

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s13 make the live corpus faithful and land the vision?** Yes, with one disclosed caveat. Every
node the dry-run identified as drifted is now reconciled to its current source (0 remain, recomputed);
the two moved ODDs resolve correctly; the vision is live — `#1001` a faithful, hash-clean 1:1 record,
`#1000` a properly-attested, lineage-clean synthesis. **MF-7 is done.** **MF-9's fidelity is true on the
live corpus** for everything in this slice's scope — the one open item is doc-coverage (`--artifacts`
mint-all drift), which was never claimed as reconcile/vision scope and is confirmed pre-existing, not
introduced here.

**What this slice revealed that the arc-plan didn't anticipate:**

1. **The capability/live-run split (s09→s10, s12→s13) earns its keep precisely when the live corpus
   diverges from what fixtures can represent.** Both bugs this slice found existed only because the real
   `.worktrees/odm` corpus has history a from-scratch fixture can't reproduce: a project node minted
   before certain code paths existed, and a schema-stamping omission invisible on a store with nothing
   yet to lose a stamp from. The arc's insistence on a real live-run slice, not just capability fixtures,
   is what caught them.
2. **The pre-commit idempotence check is not redundant with the dry-run — it caught a different class of
   bug.** The dry-run adjudication catches "will this touch the wrong things"; the idempotence re-run
   (required by F-6, done deliberately before committing) caught "does this leave something subtly
   different behind" — the schema-stamp gap only showed up as an unexpected *second* diff, not in the
   first fire's own change set.
3. **Doc-coverage (F-7) has quietly become its own maintenance surface.** `--artifacts` mint-all isn't
   re-run automatically as new slices author their own planning docs, so every slice's own artifacts
   start life uncovered until a future mint-all catches up. Worth a note for the arc-close: either fold
   a `--artifacts` re-run into the close itself, or flag this as a recurring operational gap for
   post-1.0 (an `L-8a`-adjacent concern, not raised here since L-8a itself is explicitly out of scope).

**The slice-scale silent-drop diff:** scope-as-specified vs. scope-as-delivered — no drops beyond the
two disclosed-and-routed deviations above (F-7, F-10), both pre-existing and both named, not buried.
Everything the cc-prompt's Task list specified (pre-flight, dry-run+adjudicate, fire as one commit,
verify on the committed store) was done. **The arc-close is next**: MF-9 composition check, the P-12
self-host acceptance demo, and the final reconcile-and-freeze — which should open with a fresh
`--artifacts` run to close the F-7 gap before demonstrating.
