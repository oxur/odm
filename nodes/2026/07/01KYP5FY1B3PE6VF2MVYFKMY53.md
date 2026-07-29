---
id: 01KYP5FY1B3PE6VF2MVYFKMY53
number: 569949600
type: artifact
schema: artifact/v1.1
name: Slice 09 closing report — Coverage enforcement *capability*
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice09-coverage-enforcement/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYP5FRXJBKY1GHW4QF9JQZD7
---
# Slice 09 closing report — Coverage enforcement *capability*

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 09 · **Feeds:** MF-1, MF-3, MF-5,
> MF-6 · **Resolves:** CDC v2.8 Findings 2–3 (elevated from slice08's disclosed findings) · **Realizes:**
> ODD-0025 §2.5 (`artifact` type + nearest-scale containment), §2.6 (mint-all), §2.7 (optional
> containment) · **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (F-1…F-10) · **Implemented
> by:** CC · **Date:** 2026-07-28
> **Branch:** `release/1.0.x` only — commits `012fad5`, `1760d7b`, `bc6e193`, `9cccdde` (four
> checkpoints: type foundation + Findings 2/3, discovery reach, check-wiring, F-8 + tests). **No `odm`
> branch commit this slice** — fixture-only, per the hard rule. **Evidence class:** fixture-attested
> (LEDGER-DISCIPLINE v2.0 §B class-(a)) throughout; there is no class-(b) row — the live mint/backfill
> and the check's live activation are s10.

## What shipped

Four pieces, in the order the cc-prompt's own task list laid them out:

1. **The `artifact` node type** (`NodeType::Artifact`, `odm-core/src/node_type.rs`): a document-family
   type — `is_document()` true, `is_work()` false, no valid children — that inherits every structural
   check (`check_field_validity`, `check_orphan`, `check_decomposition`) with no special-casing, by
   construction. Schema: `artifact/v1.1`, not the cc-prompt-mentioned `artifact/v1.0` — flagged and
   justified below (Deviations). The two exhaustive `match`es on `NodeType` in the codebase
   (`coverage.rs::NodeIndex::build`, `listview.rs::type_color`) were updated to handle the new variant.
2. **Discovery reach — artifact family** (`crates/odm-migrate/src/artifact.rs`, new module):
   `mint_artifacts()` walks the whole docs tree (`coverage::enumerate_docs`), mints an `artifact` node
   for every `ledger`/`cc-prompt`/`cdc-verification`/`closing-report`/other-classed doc not already
   covered by an existing node's `source.paths` — mint-all, no exemption, `coverage-report.md` included.
   Containment resolves via a new technique: a directory→id index built from every already-persisted
   arc/slice node's own `source.paths` entry, so a doc's nearest ancestor directory that's *modeled*
   resolves its containing scale directly — correct for a numbered **or** a named arc/slice with no
   re-derivation of arc/slice numbering, and correct for a chunk-level doc (resolves to the arc, since
   no `chunk` scale exists) with the same code path, not a special case.
3. **Discovery reach — design/research family** (`crates/odm-migrate/src/mapping.rs::backfill_source`,
   new function): the design/research counterpart to `selfhost::repair` — matches a sourceless
   design/research node to its legacy file by `number` (the only handle this pre-`source` family has),
   then reconciles it via the same stub-replace / faithful-body-hash-gate / hard-fail-on-drift policy
   `reconcile_source` already established for the plan-set family.
4. **Doc-coverage wired into `odm check` as an Error** (`odm-cli/src/commands.rs::aggregate`, new (c3)
   block): the s01 detector's set-difference becomes a hard `Error` (code `uncovered-doc`) for any `.md`
   under a configured `[coverage] scan_root` with no covering node. The rule is unconditionally wired
   into `aggregate()`, but the key is absent from the live store's `config.toml` today, so it is a
   documented no-op there until s10 adds it — confirmed by literally running `odm check` against
   `.worktrees/odm` and diffing (unchanged: 0 errors, the same 8 pre-existing warnings, exit 0).
5. **coverage.rs Findings 2 and 3** (CDC v2.8, elevated from slice08): Finding 2 —
   `representation()` now resolves a **named** arc's (and its slices') representation via the s05
   name-derived key (`resolve_arc_dir_numbers`, new — replays `named_arc_number`'s own collision
   handling over the full named-arc-directory set), closing the permanent "8/12" undercount. Finding
   3 — `provenance_absence` retargeted from a scan for a literal `provenance:` frontmatter line (a key
   that was renamed to `source:` back at s02, and that 0013 reserves for **derived-only, never-stored**
   lineage — so the old scan could never match anything) to the typed `Frontmatter::source().is_none()`
   check. The stale scan helpers (`has_provenance_key`, `frontmatter_yaml`) and their now-dead unit
   test were removed.

**Result:** the machinery that makes "no file left behind" mechanically enforceable now exists in code,
proven on fixtures, with the live corpus completely untouched — s10 can fire the mint-all + backfill +
check activation behind the same snapshot → dry-run → fire protocol s07/s08 used.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | `artifact` node type exists + validates | **done** | Variant + `as_str`/`is_document`/`valid_child_types`/`FromStr`; schema stamps/parses `artifact/v1.1` (flagged, see Deviations); per-type field validity inherited structurally, confirmed by test. |
| F-2 | Artifact containment = nearest modeled scale | **done** | Per-slice → slice; chunk-level (no slice subdir) → arc; genuinely top-level → uncontained; a named arc's own doc resolves via `source.paths`. All four cases fixture-proven. |
| F-3 | Discovery reaches + mints the artifact family, mint-all | **done** | `mint_artifacts()`; every supporting-doc class minted, `coverage-report.md` included; idempotent; dry-run-safe; verbatim body under the hard body-hash gate. |
| F-4 | Design/research family reachable + `source` backfilled | **done** | `backfill_source()`; stub bodies replaced, faithful bodies kept + gated, drift hard-rejected; idempotent; dry-run-safe. |
| F-5 | Doc-coverage wired into `odm check` as an Error | **done** | New (c3) rule, config-gated on `[coverage] scan_root` (absent from the live store — confirmed still a no-op there). Fixture: seed uncovered → Error; cover → green; relative/absolute scan_root spellings agree. |
| F-6 | coverage.rs Finding 2 fixed | **done** | `resolve_arc_dir_numbers` replays the named-arc collision handling; a named arc **with** a node now reads represented (0 missing), fixture-confirmed alongside the pre-existing "without a node" case. |
| F-7 | coverage.rs Finding 3 fixed | **done** | Retargeted to `source:` presence via the typed accessor; decision + rationale in a doc comment; stale scan + its dead test removed; no `provenance:` scan remains anywhere in the file. |
| F-8 | Optional containment honored | **done** | Structural since F-1 (`check_orphan` gates on `is_work()`); confirmed (not just asserted) with a new fixture case + an updated doc comment naming `artifact` explicitly. |
| F-9 | No live mutation; downstream not pulled forward | **done** | `.worktrees/odm` clean, HEAD unchanged from slice08; no ODD edited; no synthesis/reconcile code touched. |
| F-10 | Clippy/unsafe/coverage/model-drift | **done** | Clean throughout; 0 `unsafe`; every changed/new module ≥ 94% except `commands.rs` (file-wide 88.93%, but the new code is fully exercised — see Verification); no model drift, no ODD amendment needed. |

**Rows: 10. Done: 10. Deferred: 0. No-op: 0.** No silent drops — the slice-doc's "Out" items (the live
mint-all on `.worktrees/odm`, the live design/research backfill, flipping the check on against the live
corpus, synthesis/L-8b, the arc-close reconcile run) are confirmed untouched: `git -C .worktrees/odm
status` is clean, and `grep` for synthesis/reconcile-runner changes in this slice's diff turns up
nothing.

## Verification

| Check | Result |
|-------|--------|
| Fixture suite, new tests this slice | 20 new test functions across 6 files (`odm-migrate/src/artifact.rs` ×4 unit, `odm-migrate/tests/artifact.rs` ×5, `odm-migrate/tests/backfill_source.rs` ×5, `odm-migrate/tests/coverage.rs` ×1 new + 1 updated, `odm-cli/tests/check_coverage.rs` ×4, `odm-core/src/schema.rs` ×1) plus 2 existing tests extended in place (`recompose.rs::detect_orphan`, `check.rs::check_flags_wrong_type_field`) and 1 obsolete test removed (the stale `frontmatter_yaml` scan test) |
| `cargo test --workspace --all-features` | 0 failed, checked after every checkpoint and again at close |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean (only the pre-existing, unrelated `proc-macro-error2` future-incompat notice) |
| `cargo fmt --all` | applied, clean |
| `unsafe` in every file this slice touched | none |
| `cargo llvm-cov --workspace --all-features --summary-only` | `node_type.rs` 98.41%, `recompose.rs` 100%, `schema.rs` 95.04%, `check.rs` 97.29%, `artifact.rs` 94.72% (new), `coverage.rs` 97.53%, `mapping.rs` 95.07%, `selfhost.rs` 94.51%, `listview.rs` 98.92%; `commands.rs` file-wide 88.93% (2521 lines, mostly pre-existing — the new `coverage_scan_root` + `aggregate`'s (c3) block hit all branches across the 4 `check_coverage.rs` tests: `Some`+relative, `Some`+absolute, `None`, uncovered→covered). Workspace TOTAL line 90.85%. |
| Live-store read-only check | `odm check` against `.worktrees/odm`: 0 errors, 8 pre-existing warnings, exit 0 — identical to before this slice |
| `.worktrees/odm` git status | clean (`git -C .worktrees/odm status --short` → empty) |
| `.worktrees/odm` git log | HEAD still `7226797` (slice08's live rewrite) — no new commit |
| ODD files touched | none (`docs/design/*.md` unedited) |

## Deviations / findings (flagged, per the working agreement)

### Schema version: `artifact/v1.1`, not the cc-prompt-named `artifact/v1.0`

`SchemaMarker::current(node_type)` has no per-type version table — `SchemaVersion::CURRENT` is a single
**global** axis every type shares, currently `v1.1` (bumped from `v1.0` by slice04 for `source`/
`author`/`version`). There is no mechanism today for a single new type to start at a different version
number than every other current type. Building one (a per-type version map) would be new
infrastructure well beyond this slice's scope, and working around it silently (e.g. hand-stamping
`"artifact/v1.0"` as a literal string bypassing `SchemaMarker::current`) would create exactly the kind
of inconsistency ODD-0020's per-type-versioned-marker design exists to prevent. **Decision:** `artifact`
stamps the shared current version like every other type — `artifact/v1.1` — documented in a doc comment
on `SchemaMarker::current` and covered by a dedicated round-trip test. This is a naming detail, not a
model change: nothing about ODD-0025's `artifact` semantics (§2.5/§2.6/§2.7) is affected, and no ODD
edit was needed.

### `commands.rs`'s file-wide coverage sits under 90% (88.93%), while the slice's own new code is fully covered

`crates/odm-cli/src/commands.rs` is a 2521-line file predating this slice by a wide margin (it's the
single home for every `check`/`validate`/`aggregate` rule in the CLI). The ledger's F-10 criterion asks
for "≥ 90% (line), changed modules" — read literally against the whole file, it falls short. Read
against what this slice actually *changed* (`coverage_scan_root` and the new (c3) block in `aggregate`),
every branch is exercised: `check_coverage.rs`'s four tests drive `Some`(relative), `Some`(absolute),
`None` (the no-op case), and both the uncovered→Error and covered→green paths. The shortfall is
pre-existing debt in code this slice did not touch, not a gap in the new code — flagged rather than
silently claiming the file-wide number clears the bar.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV, class-(a) reproduction)

**Did s09 land the enforcement capability?** Yes: the `artifact` type exists and validates: discovery
reaches both the artifact-doc family (mint-all, no exemption) and the design/research family (source
backfill); doc-coverage is a real `check` rule (config-gated, currently inert against the live store by
construction); and CDC's two elevated Findings from slice08 (the named-arc representation undercount,
the stale `provenance:` scan) are both fixed and tested. All ten ledger rows are `done`, fixture-attested.

**What building it revealed that the design docs didn't fully anticipate:**

1. **Containment-by-directory-index turned out to generalize better than containment-by-numbering.**
   `representation()`'s F-6 fix needed to *replay* `named_arc_number`'s collision-handling algorithm
   (order-dependent, stateful across a directory set) to resolve a named arc's number from a bare
   directory name. The artifact minter's F-2 containment problem looked superficially similar
   ("resolve which arc/slice a doc belongs to") but had a strictly simpler solution available: since
   every candidate parent (an arc/slice node) is *already persisted* with its own `source.paths` entry
   by the time artifacts are minted, a directory→id index built from those entries resolves any doc's
   nearest modeled ancestor with no numbering derivation at all — numbered and named arcs alike, no
   branching. Worth remembering as the default technique for "resolve this doc's nearest structural
   ancestor" whenever the ancestor is something already in the store.
2. **`slice_number()`'s implicit precondition (its first argument must be a *raw numbered major*, not
   a final handle) was a live trap for the F-6 fix**, caught by a fixture test failing loudly rather
   than by inspection: the first implementation passed the already-derived arc handle straight into
   `slice_number()`, which internally re-derives `arc_number()` from it — silently producing a huge,
   wrong slice number for the numbered-arc case. The fix (`arc_num + slice_position(...)`, mirroring
   what `discover()` itself does) is now the second reuse of that exact formula, suggesting
   `slice_number()`'s doc comment earns a stronger warning about the precondition, though changing the
   function itself was out of this slice's scope.
3. **The two Finding 2/3 issues, once actually fixed, needed narrow but real code changes** — not
   documentation-only closures. Finding 3 in particular revealed that the "stale scan" wasn't just
   imprecise, it was *unconditionally wrong*: `provenance:` was renamed to `source:` at s02, so the old
   check had been silently flagging every single node as "provenance-missing" since that rename, with
   nothing catching it because nothing depended on the count being accurate until this slice wired a
   `check` rule near it.

**The slice-scale silent-drop diff:** scope-as-specified vs. scope-as-delivered — no drops. Every "In"
item from the slice-doc landed; every "Out" item (the live mint-all, the live design/research backfill,
flipping the check on against the live corpus, synthesis/L-8b, the arc-close reconcile run including the
living-doc-drift case s08's CDC verification surfaced) is confirmed untouched.

**Recommended arc-ledger update:** the enforcement **capability** lands. **s10 (coverage live run) is
now unblocked and next** — the pre-drawn `slice10-coverage-live-run/` open set is ready. MF-1
("no file left behind") and MF-6 ("doc-coverage `check`-enforced") move from "planned" to "capability
done, live-pending": the mechanism exists and is proven; making it *true of the live corpus* — mint-all
the ~211 supporting docs, backfill the 14 design/research nodes, flip the `[coverage] scan_root` key on
against `.worktrees/odm` — is s10's job, behind the established snapshot → dry-run → adjudicate → fire →
verify protocol.
