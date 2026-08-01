# Slice 15 closing report — Reconcile completeness + collapse the project-vision pair

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 15 · **Feeds:** MF-9 (the
> corrected freeze), the arc-close final reconcile · **Realizes:** ODD-0025 §2.3 (reversed, v1.3) +
> §2.9 (amended) · **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (F-1…F-10) ·
> **Implemented by:** CC · **Date:** 2026-08-01
> **Branch:** `release/1.0.x` only — fixture-only, per the hard rule. **No `odm`-branch commit this
> slice** — `.worktrees/odm` carries one pre-existing, uncommitted, pre-session change
> (`config.toml`, mtime before this session started — see Verification) left exactly as found; nothing
> under it was read, written, or committed by this session. **Evidence class:** fixture-attested
> (LEDGER-DISCIPLINE v2.0 §B class-(a)) throughout. **Changes are uncommitted** on `release/1.0.x`,
> left for operator/CDC review before commit.

## What shipped

**The project-vision pair collapses to one faithful 1:1 node (F-1/F-2).** A new
`crates/odm-migrate/src/collapse.rs` module (`collapse_project_vision`) finds the `NodeType::Project`
node carrying `source.synthesis`, resolves its single `supersedes` target (the 1:1 base), then:
re-snapshots the synthesis node's body **from `project-plan.md` verbatim** under the same hard
body-hash gate every migrated node passes, restores its name to the base's real project name (not the
synthesis's `"Vision"` label), and rebuilds its `source` with the `synthesis` key dropped and the
`supersedes` edge cleared — **in place**, never re-minting `id`/`number`. The node it superseded is
**retired** (`Frontmatter::retire`, supersede-don't-delete), never deleted. Because the surviving
node's identity is literally the former synthesis's own, every `part_of` child (all 12 arcs, in the
real corpus) keeps resolving without a single edge being touched — F-1's "children intact"
requirement holds by construction, not by a separate repair pass. Idempotent: a store with no
`source.synthesis`-bearing project is a 0-change no-op (`collapsed: None`).

`apply_project_vision`/`--vision` are **off the migrate path** (F-2): the CLI flag, its `vision()`
handler, `VISION_PLAN_NUMBER`, and the `--all` sweep's vision step are all removed from
`crates/odm-cli/src/{lib,migrate}.rs`. The `synthesis` module itself — `build_synthesis`,
`apply_project_vision`, `vision_body`, the `concatenation`/`editorial-merge`/`other` regimes — is
**untouched**, still exercised by its own `tests/vision_synthesis.rs` (8 tests, all green): a genuine
future synthesis (project-shaped or not) still has a working mechanism to use.

**Reconcile-exclusion re-keyed on `source.synthesis` (F-3).** `selfhost.rs` gains an `is_synthesis()`
helper (mirroring `odm_core::check::check_field_validity`'s identical predicate) used at the three
cited sites: the `by_coordinate` populate-exclusion (:332), `reconcile_source`'s own exclusion (:578),
and `reconcile()`'s node-type filter (:757, now including `Project` alongside `Arc`/`Slice`).
Post-collapse the project carries no `source.synthesis`, so it flows through exactly like any other
plan node — a drifted `project-plan.md` re-snapshots on the next `reconcile()`/`migrate` pass, closing
the structural gap that orphaned `#1001` in the arc-close dry-run.

**Artifacts and notes become mint-or-reconcile (F-4), with the `is_excluded` guard (F-5).**
`artifact.rs::mint_artifacts` and `notes.rs::mint_notes` now build a path→`Document` map of everything
already covered (not just a `HashSet` of covered paths); a covered doc whose current file body differs
from the stored body is re-snapshotted in place (`ReconciledArtifact`/`ReconciledNote`, new
`reconciled`/`reconciled_count()` on both reports), identity preserved, retired nodes excluded. The
CLI (`artifacts()`/`notes()`/`render_artifacts`/`render_notes` in `migrate.rs`) reports both mint and
reconcile counts. Separately, `mint_artifacts` gained the same `legacy::is_excluded` guard
`mint_notes`/the coverage report already had — an `index.md` or `templates/`-component path that
happens to classify `DocClass::Other` (outside `docs/design/`/`docs/dev/`) was, before this fix,
mintable as an artifact; it no longer is.

**ODD-0025 amended (F-8), not worked around.** §2.3 records the project-vision reversal in place
(the general synthesis mechanism is *not* retracted — only the project's own application of it); §2.9
records the reconcile-exclusion rekey. A new Version History v1.3 entry ties both to this slice and
its rationale. `version`/`updated` frontmatter bumped to 1.3/2026-08-01.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | Project-vision pair collapsed | **done** | `collapse.rs`; id/number-preserving re-cast + retire; 12-`part_of`-child fixture proves no edge rewrite needed. |
| F-2 | `apply_project_vision` off the migrate path; synthesis kept | **done** | `--vision` CLI surface fully removed (a deliberate widening beyond "un-wire from `--all`," flagged below); `synthesis.rs` + its tests untouched. |
| F-3 | Reconcile-exclusion keyed on `source.synthesis` | **done** | Three sites rekeyed; regression found + fixed in the same pass (see Deviations). |
| F-4 | Artifacts + notes mint-or-reconcile | **done** | Both families rebuilt on the `mapping::reconcile_source` shape; identity/containment preserved across reconcile. |
| F-5 | `mint_artifacts` `is_excluded` guard | **done** | `legacy::is_excluded` reused; fixture proves index/templates never mint, nested or at the root. |
| F-6 | Idempotent + dry-run-safe | **done** | Every new/changed path has an idempotent-rerun and a dry-run-writes-nothing fixture. |
| F-7 | No collateral | **done** | 12-child fixture + a genuine non-project synthesis fixture prove nothing else moves. |
| F-8 | Gate migration-time-only; amended not worked around | **done** | ODD-0025 §2.3/§2.9 amended in place, v1.3 Version History entry. |
| F-9 | No live mutation; no downstream pulled forward | **done, risk flagged** | `.worktrees/odm` never opened this session. A content risk for the freeze is flagged, not fixed (see Deviations). |
| F-10 | Clippy/unsafe/coverage | **done** | Clean workspace-wide; see Verification for exact figures. |

**Rows: 10. Done: 10. Deferred: 0. No-op: 0.**

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace` | 0 failed — 72/72 test suites green across every crate (`odm-graph`, `odm-core`, `odm-store`, `odm-migrate`, `odm-index`, `odm-reconcile`, `odm-cli`, `oxur-odm`) |
| `cargo clippy --workspace --all-targets --all-features` | clean (0 warnings beyond the pre-existing, unrelated `proc-macro-error2` future-incompat notice) |
| `cargo fmt --check` | clean |
| `unsafe` in every file touched (`collapse.rs`, `artifact.rs`, `notes.rs`, `selfhost.rs`, `replan.rs`, `migrate.rs`, `lib.rs`, `commands.rs`) | none |
| `cargo llvm-cov -p odm-migrate -p odm-cli` (region / fn / line %) | `collapse.rs` (new module) **97.26 / 82.61 / 98.47**; `artifact.rs` **92.59 / 73.68 / 96.30**; `notes.rs` **91.41 / 65.22 / 94.01**; `selfhost.rs` **95.27 / 87.84 / 94.99**; `synthesis.rs` (untouched, re-verified) **98.95 / 100.00 / 99.62**; `migrate.rs` (odm-cli) **79.45 / 67.80 / 85.84**; `replan.rs` **83.20 / 73.33 / 86.11** |
| `.worktrees/odm` | not touched by this session — `git -C .worktrees/odm status --short` shows exactly one modified file (`config.toml`), verified via mtime (10:45:38, before this session's first tool call) to predate this session; content matches slice14's own disclosed pre-condition (the `docs_directory` widen + `[legacy]` block), almost certainly the "config-fixed" 2026-07-31 arc-close dry-run prep the cc-prompt's background section names — left exactly as found, not committed, not reverted |

**On the two files below 90% region/line (`migrate.rs`, `replan.rs`):** both are pre-existing shortfalls,
not new gaps this slice introduced. `migrate.rs`'s uncovered lines are concentrated in rendering
functions (`render_replan`, error-message formatting) this slice trimmed (removed the `vision` note
column) but did not add to; the same file was already below 90% in slice14's closing report
(80.97/68.57/88.49 then) for the identical reason (rendering/error-path code, disclosed there too).
`replan.rs`'s uncovered lines are `restamp()`'s core name/date-correction loop — logic this slice did
not change beyond deleting the vision branch inside it; `is_change()`/`summarize()`, the two functions
this slice's edits actually touched the *logic* of, are both directly unit-tested
(`test_restamped_reports_whether_anything_changed`, `test_summary_counts_each_kind_of_change`).
Every genuinely **new** unit this slice added (`collapse.rs` in full; the mint-or-reconcile branches
in `artifact.rs`/`notes.rs`; `is_synthesis()` and the three rekeyed exclusion sites in `selfhost.rs`)
clears 90%+ on region and line coverage.

## Deviations / findings (flagged, per the working agreement)

### F-2: the `--vision` CLI flag was removed outright, not just un-wired from `--all`

The cc-prompt's literal ask was "remove `apply_project_vision` from the migrate/self-host path" —
read narrowly, that could mean only dropping the automatic vision step from `--all`'s sweep while
leaving a standalone `migrate --vision` reachable. I went further: removed the flag, its handler,
`VISION_PLAN_NUMBER`, and `one_plan_root_with_a_vision_section` entirely. Rationale: leaving
`--vision` reachable would let an operator explicitly re-split an already-collapsed project (a
one-command undo of F-1), which directly contradicts F-6's idempotency guarantee at the CLI surface
even though the underlying library capability (`synthesis.rs`) stays inert either way. This forced a
larger test-surface change than anticipated: ~10 CLI tests in `odm-cli/tests/migrate.rs` that
exercised `--vision` directly were removed or rewritten (`migrate_replan_resolves_plan_roots_from_config`
lost its dependency on the now-deleted `write_vision_plan_set` fixture helper and was rewritten with an
inline plan). Two now-**closed** slices' ledgers cite unit-test names that no longer exist verbatim
(`slice05-source-identity/ledger.md`'s `repair_excludes_the_project_node`;
`slice06-live-run-capability/ledger.md`'s `selfhost_transition_excludes_the_project_node_from_source`)
— both renamed to reflect the new F-3 behavior (`repair_backfills_a_faithful_sourceless_project` /
`repair_excludes_a_synthesis_bearing_project`; `selfhost_transition_backfills_a_faithful_sourceless_project`
/ `selfhost_transition_excludes_a_synthesis_bearing_project`). Per LEDGER-DISCIPLINE, closed-slice
ledgers are historical records and were **not** rewritten to match — this is a disclosed, expected,
one-way divergence between a closed ledger's citation and current `git grep`-ability, not a defect.

### F-3: extending `reconcile()` to the project surfaced a real bug in the D-2 escape hatch

`selfhost::discover()` always invents a `Project` `PlanNode` at `plan_root/project-plan.md`, whether or
not that file exists — true for every plan root **except** the D-2 escape-hatch self-host of a
detached arc directory (`migrate --all <extra-plan-dir>`, s14), which deliberately has no
`project-plan.md` of its own. Before F-3, this never mattered: `reconcile()` only scanned
`Arc`/`Slice`, so the phantom project entry was never looked up. The moment F-3 added `Project` to that
scan, the pre-existing, already-persisted (real) project node — matched purely by `(type, number)`
against *this* call's `plan_by_key`, with no check that this `plan_root` is actually where that node's
source lives — would be handed a plan_root pointing at a nonexistent file and error trying to read it.
The existing `migrate_all_plan_set_escape_hatch_self_hosts_an_additional_arc_directory` fixture (no
new test needed) caught this the moment F-3 landed. Fixed with a targeted guard in
`selfhost.rs::reconcile`'s loop: a matched `Project` plan-node is skipped unless its `project-plan.md`
actually exists on disk — the same "this plan_root doesn't own this node" signal `self_host`'s own
`by_source` matching already relies on implicitly. This is a narrow, targeted fix, not a redesign of
`reconcile()`'s matching discipline; the same theoretical risk exists for `Arc`/`Slice` if two
independent plan roots ever produced colliding numbers, which is out of this slice's scope to
address (no fixture has ever hit it, and `discover()` never invents an arc/slice entry the way it
does the project).

### F-9 (flagged, not fixed): the collapsed project's real body may trip `check`'s `no-vision` rule

`crates/odm-cli/src/commands.rs`'s L-3b rule (`no-vision`) requires a project's body to contain a
literal `# Vision`/`## Vision` heading. Before this slice, that always held by construction — the
synthesis body `apply_project_vision` built always carried one (`replan::VISION_HEADING`, prepended
by `synthesis::vision_body`). Post-collapse, the surviving project's body is the **full, verbatim**
`project-plan.md` — whether that file happens to contain a literal `# Vision`/`## Vision` heading
anywhere is a fact about the *live* `.worktrees/odm/project-plan.md`, which this fixture-only branch
cannot read. If it doesn't, the freeze's collapse will newly surface a `no-vision` warning that never
fired before. This is flagged for whoever runs the freeze to check ahead of time (grep the real
`project-plan.md` for a `# Vision` heading; if absent, either add one, accept the resulting warning as
expected until the LLM-command-surface arc's concise vision view lands, or bring the question back for
a model decision) — not fixed here, since it is a content/policy call about the real corpus, not a
code defect this slice can resolve from a fixture.

A related, smaller fix **was** made and is not flagged: `#1001`'s retirement (no incoming `supersedes`
edge once the surviving node drops its own) would otherwise have newly tripped the *same* L-3b rule on
a pure historical-record tombstone — L-3b's exemption filter now also checks `retired()` (read off the
freshly store-loaded document, since `retired` is not part of the index-backed `Frontmatter`
projection L-3b otherwise reads — discovered by a first attempt at this fix silently no-op'ing against
the index-reconstructed frontmatter instead). Fixture: `l3b_a_retired_project_is_exempt_from_the_vision_rule`.

### A second, independently-discovered body-mutation path removed: `replan::restamp`'s vision patch

`replan.rs::restamp` (the `migrate --replan` arm) used to *also* inject a `# Vision` section directly
into the project's body when it didn't already contain `VISION_HEADING` — a second, older mechanism
(predating `apply_project_vision`) that the module's own doc comment says was "replaced," but whose
code path was never actually removed. Left in place, running `--replan` against a freshly-collapsed
project (whose body is the plain `project-plan.md`, no `# Vision` heading unless the source file
itself has one) would silently re-inject a heading into the 1:1 body — breaking the exact invariant
this slice exists to establish, via a path outside the cc-prompt's named file list. Removed: the
`vision: Option<&str>` parameter, the `Restamped.vision` field, and the `with_vision` helper; `restamp`
now only ever touches `name`/`created`/`updated`. `replan.rs::vision_from_plan`/`VISION_HEADING`
themselves are untouched — `synthesis.rs::vision_body` still depends on them for a future genuine
synthesis. Flagged here because it's outside the literal cc-prompt scope, discovered while reasoning
through what else could still write to a project's body.

### Sub-decisions (per the working agreement, flagged rather than silently decided)

1. **Retire-vs-remove `#1001`: retire** (per the cc-prompt's own instruction — "supersede-don't-delete,
   reason recorded"). `Frontmatter::retire(reason, date)` is the existing, established mechanism (used
   identically elsewhere, e.g. `replan.rs`'s tombstone handling); no new retirement concept introduced.
2. **A minimal `orient` vision-view: not built**, per the slice-doc's explicit Out-of-scope call —
   `orient.rs` is untouched. The interim-verbosity consequence stands as disclosed there.
3. **No CLI entry point for `collapse_project_vision`** — it's a library-only capability this slice,
   fixture-tested directly (not through a flag). The freeze needs *some* way to invoke it against
   `.worktrees/odm`; whether that's a new `migrate --collapse` flag, a one-off script, or direct library
   use from whatever tooling runs the freeze is left open — not decided here, since the cc-prompt's
   Task list only asked for the capability + un-wiring the old one, not a new wiring.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s15 collapse the pair and make everything reconcile?** Yes, fixture-proven across all ten ledger
rows. `#1000`-shaped (a `source.synthesis`-bearing project) collapses to one faithful 1:1 node,
`id`/`number`/`part_of` preserved, its former base retired; the reconcile exclusion is now keyed on
the actual reason a node must never be forced through the 1:1 gate (`source.synthesis`), not on
`node_type`, so a plain project reconciles like any other plan node and artifacts/notes join the
reconcile-eligible set.

**Silent-drop diff vs. the slice-doc's In/Out list:** none against **In** — F-1 through F-5 (as F-6
through F-10 are cross-cutting verification/model criteria, not separate deliverables) are all
delivered. Against **Out**: the live re-run stayed out (no `.worktrees/odm` touch, confirmed);
`orient`'s concise vision view stayed out (untouched, per the explicit deferral to the LLM-command-
surface arc); the numeric-display-handle retirement stayed out (a different arc's punch item,
untouched). Two things were added **beyond** the literal cc-prompt text, both flagged above rather
than silently folded in: the full `--vision` CLI removal (F-2, beyond "un-wire from `--all`") and the
`replan::restamp` vision-patch removal (a second body-mutation path the cc-prompt's file list didn't
name). Both are judged in-scope by extension — they exist to protect the exact invariant (the project
is always exactly one 1:1 node) this slice's F-1 establishes — but are called out explicitly rather
than presented as if they were always part of the ask.

**What this slice revealed that the arc-plan didn't anticipate:**

1. **The D-2 escape hatch's interaction with F-3** (above) — a latent sharp edge in `discover()`'s
   always-invents-a-project-entry behavior, exposed only once `reconcile()`'s scope grew to include
   `Project`. Fixed narrowly; the same class of risk (two plan roots, colliding numbers) remains
   theoretically open for `Arc`/`Slice`, unaddressed, and not newly introduced by this slice.
2. **A live content risk for the freeze** (F-9, above): whether `project-plan.md` states a vision
   heading is unverified from this branch and should be checked before the freeze fires the collapse
   live, or the `no-vision` check will start firing where it previously didn't.
3. **A second, previously-unnoticed vision-body-mutation path** (`replan::restamp`) existed alongside
   `apply_project_vision` and would have silently undermined the collapse's 1:1 guarantee on the next
   `--replan` run had it not been found and removed in the same pass.

**Confirm the freeze collapses + closes all 4 drifts next.** With F-1/F-3 landed, the arc-close's
final reconcile-and-freeze — run live against `.worktrees/odm` — should now: (a) collapse the
synthesis-shaped project (`#1000`/`#1001` in the live corpus) to one faithful 1:1 node via
`collapse_project_vision` (invocation mechanism TBD, sub-decision 3 above); (b) reconcile the
collapsed project along with the two previously-orphaned drifts the 2026-07-31 dry-run could not close
(`#1001` itself, no longer applicable post-collapse since it's retired not reconciled; `#509907700`,
now reachable via F-4's artifact mint-or-reconcile); (c) leave the other 2 of 4 drifts closed as the
dry-run already showed. **This slice blocks that freeze**, per its own framing — s15 is done; the
arc-close resumes: the freeze, then the P-12 self-host demo, then Migration Fidelity closes.
