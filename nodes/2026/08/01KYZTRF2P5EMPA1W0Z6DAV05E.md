---
id: 01KYZTRF2P5EMPA1W0Z6DAV05E
number: 570017700
type: artifact
schema: artifact/v1.1
name: Slice 14 closing report — Config-driven migrate roots + additional-paths
created: 2026-07-31
updated: 2026-07-31
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice14-migrate-config-roots/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-01
edges:
  part_of: 01KYZTRDEQE5EMGN59BPH3ASYR
---
# Slice 14 closing report — Config-driven migrate roots + additional-paths

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 14 · **Feeds:** MF-9 (the
> corrected command the arc-close freeze fires) · **Realizes:** ODD-0022 §4.2 (extended, v1.1) ·
> **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (F-1…F-10) · **Implemented by:** CC ·
> **Date:** 2026-07-31
> **Branch:** `release/1.0.x` only — fixture-only, per the hard rule. **No `odm`-branch commit this
> slice** — `.worktrees/odm` confirmed untouched throughout (opened only for read, once, while
> investigating F-6's `main`-branch history). **Evidence class:** fixture-attested (LEDGER-DISCIPLINE
> v2.0 §B class-(a)) throughout.

## What shipped

**The CLI is config-driven, not path-driven (F-1).** `migrate`'s required `legacy_path: String`
positional is gone; a new optional, comma-separated `[ADDITIONAL_PATHS]` positional takes its place,
parsed (trimmed, empties dropped) at the `lib.rs` dispatch layer. Every mode —
`--coverage`/`--artifacts`/`--notes`/`--vision`/`--all`/the bare default/`--replan` — resolves its
root(s) from the operational config instead.

**The restored `docs_directory` + `"design"` append (F-2), the actual bug the operator surfaced.**
`migrate --all`'s design/research step used to read `docs_directory` as-is
(`commands::configured_docs_directory`, no join) — a legacy (v0.3.5−) semantic that computed the
indexed corpus as `docs_directory` **plus a hard-coded `"design"`**. It only ever "worked" because
the live config happened to set `docs_directory = "./docs/design"` directly; the moment
`docs_directory` becomes the canonical, wider `"./docs"` (the parent the plan-set/`--artifacts` sweep
also needs), an as-is read would reconcile design/research over the **whole** docs tree — plan trees
and dev docs included — applying NN-state + §2.4 frontmatter rules to files that must never get them.
A new `commands::configured_design_directory` computes `docs_directory.join("design")` once, and
every design/research call site (the bare dispatch, `--all`) uses it instead of the raw value.

**A root map resolved entirely from config (F-3, D-1).** `docs_directory` doubles as the parent
umbrella `discover_plan_roots`/`--artifacts`/`--coverage` sweep, `docs_directory` + `"design"` is the
design/research corpus, and `dev_directory` (never derived from `docs_directory`) is `--notes`'s
independent root. D-1's alternative (a dedicated umbrella key) was considered and rejected: the
project's own layout (`docs/{design,design-v1.0.0,dev}`) already makes `docs_directory=./docs` name
the umbrella, and no fixture surfaced a case where reusing it broke down.

**A persistent additional-paths sweep (F-4/F-5/F-7).** `migrate --all`'s positional now unions with
`[legacy].additional_paths` (config), sorted and deduplicated, and writes the result back via
`toml_edit` (format-preserving — every comment and unrelated section in the operational config
survives byte-for-byte). Each *effective* additional (after set-subtracting anything already owned by
the design/dev/plan-set roots, in either overlap direction) is migrated via `--artifacts`'s generic
supporting-doc derivation, with a plan-set escape hatch (D-2) for one that turns out to be Plan-shaped
after all.

**The `[legacy]` config-home resolution fixed (F-6) — the second real bug, this one architectural, not
just a missing join.** `configured_directory`'s existing top-level-then-`[legacy]` fallback logic was
already correct; the actual defect was that the operator's `[legacy]` block (added to the code-branch
`odm.toml`, in preparation for this exact slice) sits in the *locator*, which stops being the file
`StoreHome::resolve` treats as operational the instant a store `config.toml` exists — and this repo's
own live store already has one. A fixture reproducing the real ODD-0022 §4.2 split (a genuine
`[store]`-redirected store with its own `config.toml`, plus a decoy `[legacy]` block in the locator)
proved the resolver was sound and pinpointed the fix as documentation + operator awareness, not new
code: ODD-0022 amended (v1.1) to record the `[legacy]` sub-table and this exact resolution rule.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | CLI positional redesign | **done** | No `<LEGACY_PATH>`; `[ADDITIONAL_PATHS]` optional, trimmed/empties-dropped; every mode config-driven. |
| F-2 | `docs_directory` + `"design"` append restored | **done** | The core bug; fixture proves design-only scoping, a loose file gets no design derivation. |
| F-3 | Root map from config (D-1) | **done** | D-1 (parent = `docs_directory`) adopted as recommended; no fixture argued otherwise. |
| F-4 | Additional-path processing (D-2) | **done, finding disclosed** | Artifacts-generic + plan-set escape hatch; the escape hatch's realistic scope (arcs, not a second whole project) discovered and disclosed, not silently routed around. |
| F-5 | `additional_paths` persistence | **done** | Union, sort, dedup, write-back proven across repeated runs; `--dry-run` writes no config. |
| F-6 | `[legacy]` config home + idempotency | **done** | Real split-store fixture proves the resolver was already correct; the defect was operator-side file placement — ODD-0022 amended to document it. |
| F-7 | Set-subtraction dedup | **done** | Both overlap directions (equals, nested-under) proven excluded from the generic pass, still persisted. |
| F-8 | Idempotent + dry-run-safe (incl. config) | **done** | Node count stable across runs; config byte-identical on `--dry-run`; write-back compares before writing, not relying on round-trip alone. |
| F-9 | No model drift / amend-not-work-around | **done** | 0 signature/behavior changes in `odm-migrate`; ODD-0022 amended (v1.1) for the two genuinely new model facts (`[legacy]` sub-table, `additional_paths`). |
| F-10 | Clippy/unsafe/coverage | **done** | Clean throughout; `commands.rs` new code ~90%+, `migrate.rs`'s new branches all covered — the file's aggregate stays below 90% only via pre-existing, untouched `--replan`-rendering/`--coverage`-rendering code. |

**Rows: 10. Done: 10. Deferred: 0. No-op: 0.** No silent drops: the live re-run (arc-close freeze),
the P-12 demo, and L-8/CDC-ARC-1 (the RH-era `version` backfill) are all confirmed untouched, per
scope.

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace --all-features` | 0 failed, checked after every code checkpoint (5 full runs across the slice) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean (only the pre-existing, unrelated `proc-macro-error2` notice) |
| `cargo fmt --check` | clean |
| `unsafe` in every file touched | none |
| `llvm-cov -p odm-cli` | `commands.rs` 89.32%/91.16%/93.29% (region/fn/line); `migrate.rs` 80.97%/68.57%/88.49% — up from 73.82%/60%/80.23% before the targeted `--replan`/error-path tests; remaining gap is pre-existing, untouched rendering code |
| `--help` output | `Usage: odm migrate [OPTIONS] [ADDITIONAL_PATHS]` — no `<LEGACY_PATH>` |
| `.worktrees/odm` | untouched — `git -C .worktrees/odm status --short` empty at every checkpoint; opened only for a read-only `git show`/`cat` while researching F-6 |
| Other `odm-cli` test files using the old positional (`migrate_reconcile.rs`, `selfhost.rs`) | found and updated to the config-driven fixture pattern; all pass |

## Deviations / findings (flagged, per the working agreement)

### F-4/D-2: the plan-set escape hatch cannot self-host a second, independent project

Confirmed empirically, not assumed: a fixture additional directory with its own full
`project-plan.md` tree produced **one** project node, not two, because `selfhost::PROJECT_NUMBER`
is a fixed constant (`1000`), not derived per plan-set — the second tree's project node collides
with the first's identity key and is silently skipped as already-existing. This is a pre-existing
architectural property of `self_host` (a store has exactly one project space), not a bug this slice
introduced or could reasonably fix within its scope (changing `PROJECT_NUMBER`'s derivation is a
`odm-migrate` model change, well outside "thin config-resolution wiring"). The escape hatch is
correctly scoped to an additional that folds *into* the same project — a detached arc directory —
which the final fixture (`migrate_all_plan_set_escape_hatch_self_hosts_an_additional_arc_directory`)
tests instead. Named here so a future slice attempting true multi-project support in one store knows
this constraint exists and where it lives.

### F-2/F-6: a live-fire pre-condition, disclosed ahead of the arc-close

`.worktrees/odm/config.toml`'s `docs_directory` is currently `"./docs/design"` — the narrow form,
correct under the pre-s14, un-appended resolution. F-2's restored append means the design/research
reconcile now resolves `docs_directory` **+** `"design"`; firing the corrected `--all` against this
config unchanged would look for `docs/design/design` (doesn't exist) and silently reconcile zero
design/research docs — not error, just quietly do nothing for that one step. **Not a silent-damage
risk**: the live-mutation protocol's mandatory dry-run would surface this immediately (0 reconciled
where dozens are expected, an obvious adjudication failure). Disclosed here so the arc-close isn't
surprised mid-adjudication: `docs_directory` should be updated to the parent `"./docs"` in
`.worktrees/odm/config.toml` *before* the live fire, matching the operator's own already-signaled
intent (the `[legacy]` block already added to the code-branch `odm.toml`, per F-6). `dev_directory`
needs no corresponding change (F-3: read as-is, never appended).

### Two `Store::open` usage bugs, found in the *fixture*, not the implementation

While building F-6's split-store fixture, two assertions failed for a reason that turned out to be
entirely in the test: `Store::open` takes a store root directly with no internal `StoreHome`
redirection (that redirection is `StoreHome::resolve`'s job, already done once for `dispatch()`
itself). Every other fixture in this test suite uses a flat store (no `[store]` split), so
`Store::open(store_dir.path())` and the *actual* resolved store root happen to coincide — this
slice's split-store fixture was the first to need `Store::open(&store_root)` explicitly. Fixed in
the test, not the implementation; named here because it's the kind of thing a future split-store
fixture will hit again if this isn't visible.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s14 make `migrate` config-driven, restore the append, and land persistent additionals?** Yes,
fixture-proven across all ten ledger rows. The three operator-surfaced defects are all fixed: the
ambiguous positional is gone, the `docs_directory` + `"design"` append is restored (the literal bug
that would have broken design/research reconciliation the moment the live config moves to the wider
form), and `additional_paths` is a first-class, persistent config affordance with correct
set-subtraction dedup against the design/dev/plan-set roots.

**What this slice revealed that the arc-plan didn't anticipate:**

1. **The `[legacy]` config-home defect (F-6) was a genuine, separate bug from the one the cc-prompt
   named first (F-2's append)** — not just a documentation gap. The operator's own preparatory edit
   (adding `[legacy]` to the code-branch `odm.toml`) was, itself, evidence of the defect: that block
   is invisible to `migrate` today, on this very repo, because a store `config.toml` already exists.
   Building the real split-store scenario as a fixture — rather than trusting the simpler,
   already-passing flat-store case — is what surfaced this.
2. **The plan-set escape hatch (D-2) has a real scope boundary** (a store's single project-number
   space) that the slice-doc's "its own plan set" wording didn't anticipate. Disclosed above, not
   silently narrowed without comment.
3. **A concrete, disclosed pre-condition for the arc-close's live fire**: `.worktrees/odm/config.toml`'s
   `docs_directory` must move from `"./docs/design"` to `"./docs"` before (or the dry-run will simply
   show) the corrected `--all` reconciles anything for design/research. Not a regression risk (the
   protocol's dry-run gate catches it), but worth knowing going in rather than discovering mid-adjudication.

**Confirmed against the already-written arc-close runbook.** `../closing-report.md` §7 (CDC,
`3bccd50`, independent of this slice) specifies the freeze exactly as `odm migrate --all --dry-run`
then `odm migrate --all` — **no positional path**. Under the pre-s14 CLI that syntax was a clap parse
error (`legacy_path` was required); CDC's runbook only parses under the config-driven contract this
slice delivers, which is a strong, independent confirmation that this redesign is what the arc-close
actually needs, not a speculative one. §7's adjudication expectation ("ONLY body re-snapshots of the
edited plan docs… no unexpected create") still holds under the corrected command: `--artifacts`/
`--notes` find nothing new (already fully covered by this session's earlier live `--all` fire), and
the additional-paths sweep is a no-op when the runbook's own invocation passes no positional and
none is yet configured. The one addition needed before §7 (A) runs is this slice's disclosed
pre-condition: update `.worktrees/odm/config.toml`'s `docs_directory` to `"./docs"` first, or the
dry-run's adjudication will show 0 design/research reconciled where the runbook expects the two
living-plan-tail nodes.

**The slice-scale silent-drop diff:** scope-as-specified vs. scope-as-delivered — no drops beyond the
two disclosed findings above (both named, not buried, both squarely "found while doing the work,"
not scope creep). Everything the cc-prompt's Task list specified (CLI surface, design append, root
map, additional-path processing + persistence, dedup, config-home fix, fixtures) was delivered.
**The arc-close resumes next**: `../closing-report.md` §7's runbook — the final reconcile-and-freeze
fires the *corrected* `migrate --all` live on `.worktrees/odm` (after updating `docs_directory` per
the disclosed pre-condition above),
then the P-12 self-host acceptance demo, then Migration Fidelity closes.
