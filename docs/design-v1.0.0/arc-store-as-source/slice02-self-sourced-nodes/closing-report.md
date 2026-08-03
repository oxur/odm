# Closing Report — Slice 02 (Store-as-Source): self-sourced planning nodes

> Verified by: CC (this session). F-1…F-5, F-7, F-10, F-11 attested (unit + real end-to-end
> `odm-cli` integration tests). F-8/F-9 attested (amendment stubs written, not yet folded into the
> accepted ODDs). F-4/F-6's real-corpus leg deferred to CDC/operator — `.worktrees/odm` is not
> checked out in this implementation worktree, the same blocker every slice this session has hit.
> Closed 2026-08-03 on `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative.

**F-1 (schema).** `Origin::Authored` (a new value on the existing enum, `origin.rs`).
`Source`'s migration-only fields (`normalization`/`migrated_by`/`migrated_on`) became `Option<_>` —
`None` on an authored node, unchanged for a migrated one (every existing construction site updated,
~30 call sites across `odm-core`/`odm-migrate`/`odm-cli`, purely mechanical `Some(...)` wrapping
plus one new field). Added `source.migrated_from: Vec<PathBuf>` for the conversion marker (F-10),
and `Source::authored(migrated_from) -> Self` / `Source::is_authored(&self) -> bool` as the
canonical constructor/predicate. A new structural check,
`Violation::InconsistentAuthoredSource`, enforces the two-way agreement between `origin` and
`source.class` — this is what makes "authored is a value, never an absence" (ODD-0026 §2.1) a
checked invariant, not just a convention.

**F-2/F-3 (check).** Investigated first, before writing code: does `check`/`recompose` actually
gate on "no external source" anywhere today? No — `recompose.rs`'s `UndevelopedStub` is purely a
parent/child-count signal, unrelated to `source`; there was no literal "missing-source" check to
suppress. So F-2/F-3 reduced to: add the one new consistency check (F-1), and prove every existing
family (link-integrity, cycles, required-fields) still fires on an authored node exactly as before
— which it does, since none of that code was touched. Six fixtures across `odm-core/tests/check.rs`
cover both directions.

**F-4/F-5 (migrate/reconcile never churn an authored node) — the substantive code change.**
Reading `self_host`'s existing three-tier identity index (`by_source` → `by_coordinate` → create)
before writing anything surfaced a real gap: an authored node's `source.paths` is always empty, so
it was invisible to `by_source` (zero iterations) *and* to `by_coordinate` (it has `Some(source)`,
so the sourceless-fallback arm never fires). Without a fix, the first `migrate --all` after this
lands would have silently **re-minted every authored node as a duplicate**, the moment its `./docs`
counterpart was still present — true for every node until slice04's cutover. Fixed with a fourth
index, `by_coordinate_authored`, checked before falling through to `to_create`; matched nodes are
recorded (for `part_of` resolution) and reported via a new `SkipReason::Authored`, untouched
otherwise. `selfhost::reconcile` (the "already-sourced" reconciler) gained an explicit
`origin() == Authored` skip alongside its existing sourceless skip.
`mapping::reconcile_source`/`backfill_source` (the design/research family) needed **no change** —
confirmed by code read: an authored node's empty `paths` already makes `.first()` return `None`,
the existing "nothing to resolve from" skip. Proven end-to-end: an authored node survives a real
`self_host` + `reconcile` pass with its `./docs` counterpart actively drifting, body and source both
byte-identical after.

**F-6 (the corpus conversion mechanism).** `selfhost::convert_to_authored` — a new, explicit,
`--dry-run`-able, idempotent pass, re-classifying every genuinely-migrated `project`/`arc`/`slice`
to authored, preserving the former `source.paths` in `source.migrated_from` (sub-decision (i)).
**Deliberately not folded into `migrate --all`** — see the "judgment call" note below. Wired as
`odm migrate --to-authored`, conflicting with every other migrate mode the same way `--coverage`/
`--artifacts`/`--notes`/`--all` conflict with each other. Six fixtures
(`convert_to_authored.rs`) plus a real end-to-end CLI test reproducing SS-3's exact acceptance
shape (convert → `check` exit 0 with `./docs` still present) on a synthetic plan set. The **real**
leg — the actual 400+-node corpus — could not run: `.worktrees/odm` is not checked out in this
worktree (the odm store lives on the orphan `odm` branch per ODD-0022, outside this code
worktree's git history). This is the identical blocker slice04/slice06 hit earlier this session,
carried forward honestly rather than silently assumed passing.

**F-7 (no regression).** The pre-existing slice05-era drift fixture
(`repair_surfaces_a_drifted_non_stub_body_as_a_hash_mismatch`) passes unmodified — `verify_body_hash`
and its call sites were never touched, only wrapped with new authored-guards *around* them. Added
one slice02-specific fixture proving the new authored-recognition logic doesn't accidentally mask
the gate for an unrelated migrated sibling in the same run
(`a_drifted_migrated_node_still_hard_fails_alongside_an_untouched_authored_one`).

**F-8/F-9 (amendments).** Written as stubs in this slice's own directory
(`F-8-amendment-ODD-0013.md`, `F-9-amendment-ODD-0025.md`), following the RH `C-2-amendment-*`
precedent: precise before/after text for each affected section, plus a version-history entry ready
to fold in. **Not applied to the accepted ODD documents** — per the cc-prompt's own instruction
("Write the ODD-0013 and ODD-0025 amendment docs in this slice directory") and the RH precedent
(amendments are proposed by the implementing slice, folded in and accepted as a separate operator/
CDC action). Marked `attested`, not `done`, in the ledger for exactly this reason.

**F-10 (the sub-decision).** Resolved **(i)** — re-classify as authored, preserve provenance via
`source.migrated_from`. This is CC's call, made without a fresh `AskUserQuestion` round-trip:
the slice-doc's own stated lean, and the direct, low-risk application of ODD-0026 fork A (already
operator-ratified: "provenance is enduring — don't drop it"). Option (ii) would leave dangling
`source.paths` with no marker after slice04's cutover, directly contradicting fork A's spirit.
Flagged prominently here and in the amendment stub, per the slice-doc's "flagged, not silently
chosen" instruction — the operator can override before this lands on the real corpus.

**F-11.** `make format` (reflow only), `make lint` (clippy `-D warnings` + rustfmt, clean), `make
test` (full workspace, all crates + doctests) all green.

## A judgment call worth stating plainly: `--to-authored` is separate from `--all`

Once a node converts, `reconcile` stops re-verifying it against `./docs` — by design, since it's
self-sourced now (F-5). That is a **real, disclosed behavior change** to the operator's current
edit-`arc-plan.md`-then-`migrate --all` workflow, which has been the primary planning interface all
session. Folding the conversion into `--all` unconditionally would make every node `--all` touches
— old and freshly-self-hosted alike — authored by the end of the same run, silently freezing further
`./docs` edits to it until slice03's native edit commands exist. Instead, `--to-authored` is its own
explicit, `--dry-run`-able flag: the operator triggers the transition knowingly, previews it first,
and nothing about their ordinary `migrate --all` habit changes until they choose to run it. This is
a design decision the slice-doc didn't spell out explicitly; recorded here, in the flag's own doc
comment, and in `convert_to_authored`'s doc comment so it isn't only discoverable by reading code.

## Scope discipline

Diff: `odm-core/src/{origin,frontmatter,check,rollup}.rs` (schema + the new check + the `Provenance`
grouping gaining an `authored` bucket, which two existing JSON/text shape-lock tests needed
updating for — a correct, intended consequence of a new `Origin` variant, not a regression);
`odm-migrate/src/selfhost.rs` (the fourth identity index, the `reconcile` guard, the new
`convert_to_authored` pass); `odm-migrate/src/lib.rs` (`SkipReason::Authored`);
`odm-cli/src/{lib,migrate,commands,json,rollup}.rs` (the `--to-authored` flag + its render function,
the CLI's `Violation` rendering gaining an arm). No change to `mapping.rs` (design/research family —
confirmed safe by construction, not touched). No change to `recompose.rs`/`delta.rs`. No store-as-
source model change beyond what ODD-0026 already decided (that's slice 01, already accepted). No
CLI authoring commands (`node new --from-file` etc. — slice 03, explicitly out of scope). `./docs`
untouched, not deleted (slice 04).

## Iterations

Two passes. First: schema + check + migrate/reconcile guards + the conversion mechanism +
fixtures, all green in isolation. Second (self-caught, not CDC- or operator-reported): the real
end-to-end CLI test for F-2 failed because `check_authored_source` was wired into the wrong
function (`check()`, index-backed — `source` isn't in the index) instead of `content_validity()`
(store-loaded, the same reason `check_source_paths` lives there); moved it, added the CLI's
`Violation` rendering arm it was also missing, all green after. Well inside the five-iteration cap.

## Bubble-up → `../arc-plan.md`

- **Slice 02 done** (SS-2/SS-3 fixture-proven, SS-3's real-corpus leg deferred to CDC; SS-5
  regression-fixtured), delivering **the core enabler**: an authored node is now a first-class,
  checked, migrate-safe citizen of the store. Slice 03 (native authoring commands) can build on
  authored nodes being valid; the author-vs-odm field boundary it needs is recorded in the ODD-0013
  amendment stub, ready to fold in alongside slice03's own schema work.
- **What implementing it revealed that the arc-plan/ODD-0026 didn't fully anticipate**: (a) the
  index-backed vs store-loaded split in `odm_core::check` is a sharp edge any future
  `source`-dependent (or generally non-index-carried-field-dependent) check will hit — worth a
  standing note in `check.rs`'s own module doc (added) and worth CDC flagging in any future
  check-authoring guidance; (b) `self_host`'s matching index needed a fourth tier, not just a skip
  — this is a detail ODD-0026 §2.1 didn't need to specify (it's an implementation consequence of
  "authored nodes have no source.paths"), but it's exactly the kind of gap that would have shipped
  a silent duplicate-minting bug on the real corpus without being read for explicitly; (c) the
  `--to-authored`-vs-`--all` separation (above) is a judgment call ODD-0026 left implicit — worth
  folding into ODD-0026 or a future amendment if the operator confirms the reasoning.
- **Silent-drop check:** all 11 ledger rows reached a final status (9 done, 2 attested with a
  disclosed caveat — F-8/F-9 not yet folded into the accepted ODDs; F-4/F-6's real-corpus legs
  disclosed within their otherwise-`done` rows). Nothing dropped or left `open`.
- **For the operator's next real cycle**: once ready, `odm migrate --to-authored --dry-run` on the
  real store previews the conversion; a real run, then `odm migrate --all`, then `odm check`
  reproduces SS-3/F-6's live leg. Fold `F-8-amendment-ODD-0013.md`/`F-9-amendment-ODD-0025.md` into
  the accepted ODDs when convenient — code already implements what they specify.
