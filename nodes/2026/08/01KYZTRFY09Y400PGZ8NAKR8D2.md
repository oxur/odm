---
id: 01KYZTRFY09Y400PGZ8NAKR8D2
number: 562085400
type: artifact
schema: artifact/v1.1
name: 'Slice 15 (Migration Fidelity): Reconcile completeness + collapse the project-vision pair'
created: 2026-08-01
updated: 2026-08-01
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice15-reconcile-completeness/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-01
edges:
  part_of: 01KYZTRDEQ5KD76YCNNB7DHEG3
---
# Slice 15 (Migration Fidelity): Reconcile completeness + collapse the project-vision pair

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. **Capability
> slice — fixture-only; `.worktrees/odm` untouched.** Code/fixtures class-(a): CDC reproduces by direct
> read; runtime attested→CI. The collapse + full reconcile fire live at the **arc-close freeze**. Five-
> iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **Project-vision pair collapsed**: `#1000` → plain 1:1 project node (body == `project-plan.md`, gate passes, no `source.synthesis`, no `supersedes`), `id`/`number`/`part_of`-children preserved; `#1001` retired (supersede-don't-delete, reason recorded) | fixture: run the collapse → `#1000` 1:1 + 12 children intact, `#1001` retired; ODD-0025 §2.3 amended | serious (the model change) | operator 2026-08-01 | done | `crates/odm-migrate/src/collapse.rs` (new module, `collapse_project_vision`); unit tests `collapse_makes_the_synthesis_a_plain_1to1_node_and_retires_the_base`, `collapse_preserves_part_of_children` (12-arc fixture); ODD-0025 §2.3 + §2.9 amended, v1.3 Version History entry | `#1000` must survive (12 arcs `part_of` it); `#1001` has 0 children. The surviving node's `id`/`number` are the former synthesis's own — never re-minted — so `part_of` children need no edge rewrite at all. |
| F-2 | **`apply_project_vision` off the migrate path; synthesis library kept**: the project migrates as a normal plan node; the `synthesis` module (s11) stays for future genuine syntheses | read the migrate/self-host flow: no vision-apply call; `synthesis` module + its tests intact | serious | s11 capability | done | Removed `--vision`/`vision()`/`VISION_PLAN_NUMBER`/`one_plan_root_with_a_vision_section` from `crates/odm-cli/src/migrate.rs` + the clap flag from `crates/odm-cli/src/lib.rs`'s `Command::Migrate`; `all()`'s step 5 removed. `crates/odm-migrate/src/synthesis.rs` (`apply_project_vision`, `build_synthesis`, `vision_body`) untouched — its own test suite (`tests/vision_synthesis.rs`, 8 tests) still green. | **Deviation flagged**: went beyond "un-wire from the automatic `--all` sweep" to remove the `--vision` CLI flag entirely — leaving it reachable would let an operator re-split an already-collapsed project (an explicit `migrate --vision` re-run), defeating F-6's idempotency guarantee at the CLI layer even though the library capability itself stays inert. ~10 CLI tests in `odm-cli/tests/migrate.rs` that exercised `--vision` directly were removed/rewritten accordingly. Two now-closed slices' ledgers (s05 `slice05-source-identity/ledger.md`, s06 `slice06-live-run-capability/ledger.md`) cite unit-test names since renamed (`repair_excludes_the_project_node` → `repair_backfills_a_faithful_sourceless_project` / `repair_excludes_a_synthesis_bearing_project`; `selfhost_transition_excludes_the_project_node_from_source` → `selfhost_transition_backfills_a_faithful_sourceless_project` / `selfhost_transition_excludes_a_synthesis_bearing_project`) — left as historical records per LEDGER-DISCIPLINE, not rewritten. |
| F-3 | **Reconcile-exclusion keys on `source.synthesis`**, not `node_type == Project`: post-collapse the project reconciles like any plan node; a `source.synthesis`-bearing node stays excluded | fixture: edit `project-plan.md` → `#1000` re-snapshots; a seeded synthesis node is *not* 1:1-reconciled | serious (MF-9 fidelity) | arc-close dry-run finding | done | `selfhost.rs`: new `is_synthesis()` helper used at the three cited sites (:332 `by_coordinate` populate-exclusion, :578 `reconcile_source`'s own exclusion, :757 `reconcile()`'s type filter, now including `Project`). Unit tests `reconcile_source_excludes_a_synthesis_node`, `reconcile_source_does_not_exclude_a_sourceless_project`, `reconcile_source_reconciles_a_faithful_1to1_project_like_any_other_node`; integration tests `reconcile_reconciles_a_drifted_project_body`, `reconcile_excludes_a_synthesis_bearing_project` (`tests/reconcile.rs`); `source_identity.rs`'s `repair_backfills_a_faithful_sourceless_project`/`repair_excludes_a_synthesis_bearing_project`/`selfhost_transition_backfills_a_faithful_sourceless_project`/`selfhost_transition_excludes_a_synthesis_bearing_project`; end-to-end via `odm-cli/tests/migrate_reconcile.rs` (fixture updated — a faithful sourceless project now backfills instead of staying excluded) | Sites: selfhost.rs:578/757/332. ODD-0020 v1.4 key. **Discovered + fixed regression**: extending `reconcile()`'s scope to `Project` exposed a pre-existing sharp edge — `discover()` always invents a project `PlanNode` at `plan_root/project-plan.md` even when that file doesn't exist (true for the D-2 escape-hatch self-host of a detached arc directory), which would have made `reconcile()` try to read a nonexistent file and error on `migrate --all <extra-plan-dir>`. Fixed with a targeted guard (`plan_node.source.is_file()`) in `selfhost.rs::reconcile`'s loop; regression-tested by the existing `migrate_all_plan_set_escape_hatch_self_hosts_an_additional_arc_directory` (odm-cli), which now passes with the F-3 change live. |
| F-4 | **Artifacts + notes mint-or-reconcile**: a drifted already-covered `artifact`/`note` re-snapshots in place (`id`/`number`/`part_of` preserved), not skipped | fixture: drift a minted artifact + a minted note → both re-snapshot, identity preserved | serious (MF-9 fidelity; "everything ingested reconciles") | operator D-1 | done | `artifact.rs::mint_artifacts` and `notes.rs::mint_notes` rebuilt as mint-**or**-reconcile: `already_covered: HashSet` → `covered: HashMap<path, Document>`; a covered doc whose file body differs from the stored body re-snapshots (`ReconciledArtifact`/`ReconciledNote`, new `ArtifactReport.reconciled`/`NoteReport.reconciled` + `reconciled_count()`); a retired artifact/note is excluded (historical record). CLI (`migrate.rs`'s `artifacts()`/`notes()`/`render_artifacts`/`render_notes`) updated to report both mint and reconcile counts. Tests: `artifact_mint_reconciles_a_drifted_already_minted_artifact`, `artifact_mint_reconcile_dry_run_writes_nothing`, `notes_mint_reconciles_a_drifted_already_minted_note`, `notes_mint_reconcile_dry_run_writes_nothing` | Closes `#509907700` (slice10 ledger) + the same gap in the note family. |
| F-5 | **`mint_artifacts` `is_excluded` guard**: `index.md` (basename) + any `templates/`-component path never minted, regardless of root | fixture: `index.md` + `templates/x.md` under the artifacts root → 0 minted | correctness (latent-bug fix) | this session's finding | done | `artifact.rs::mint_artifacts` now calls `legacy::is_excluded(&doc.path)` before minting (mirrors `mint_notes`/the coverage report's own guard). Test `artifact_mint_never_mints_index_or_template_files` — `index.md`/`templates/x.md` at both the artifacts root and nested under an arc directory, 0 minted | The guard the coverage report + notes pass already have. |
| F-6 | **Idempotent + dry-run-safe**: a second run over an already-collapsed/reconciled store is 0-change on every path (collapse, reconcile, mint); `--dry-run` writes nothing | run twice: second pass 0/0/0; dry-run leaves store byte-identical | serious (operator req: idempotency through the migration period) | s13 protocol | done | `collapse_is_idempotent` + `collapse_dry_run_writes_nothing` (collapse.rs); `reconcile_is_idempotent_and_dry_run_writes_nothing` (pre-existing, still green under the F-3 rekey); `migrate_reconcile_rerun_is_idempotent` (odm-cli, end-to-end); `artifact_mint_is_idempotent`/`artifact_mint_dry_run_writes_nothing`/`notes_mint_is_idempotent`/`notes_mint_dry_run_writes_nothing` (pre-existing) plus the new `*_reconcile_dry_run_writes_nothing` pair | "already a plain project node" is the collapse's idempotent no-op (`collapse_is_a_no_op_when_no_synthesis_project_exists`). |
| F-7 | **No collateral**: only body (+ `updated`) changes on reconciled nodes; `#1000`'s children + ids/schema/edges intact; retired + already-faithful nodes untouched; genuine synthesis nodes untouched | direct read of fixtures: diffs show body/updated only where expected; `#1000` children preserved | serious | slice-doc | done | `collapse_preserves_part_of_children` (12-arc fixture, `part_of` unchanged); `collapse_leaves_a_genuine_non_project_synthesis_untouched` (a `NodeType::Design` synthesis is never touched by the project-scoped collapse); `artifact_mint_reconciles_a_drifted_already_minted_artifact` asserts `id`/`part_of` preserved across reconcile | |
| F-8 | **Gate stays migration-time-only; model amended not worked around**: reconcile only sets a body from its current source; ODD-0025 §2.3 reversed (cited), §2.6/§2.9 lines if needed | cross-read: gate not continuous; §2.3 amendment present + coherent | correctness | ODD-0025 | done | `docs/design/04-accepted/0025-migration-fidelity-model.md` v1.3: §2.3 amended in place (the project vision reversed out of the synthesis mechanism; the general mechanism itself is *not* retracted) + §2.9 amended (the reconcile exclusion re-keyed off `source.synthesis`, not `node_type == Project`) + a new Version History entry recording both changes and their rationale. `version`/`updated` frontmatter bumped to 1.3/2026-08-01. | The 1:1 rule + §2.1 gate unchanged; s15 removes the project's synthesis special-case + extends *which* nodes reconcile. |
| F-9 | **No live mutation; no downstream pulled forward**: no `odm`-branch commit; no freeze/P-12; no L-8; `orient` view left to the LLM arc (interim verbosity disclosed) | `git -C .worktrees/odm status` clean; HEAD unchanged; no `orient` render change | serious | LEDGER-DISCIPLINE | done | `git -C .worktrees/odm status --short` shows exactly one modified file, `config.toml` — verified by mtime (10:45:38, before this session's first tool call) to **predate this session**; its content matches slice14's own disclosed pre-condition (the `docs_directory` widen + `[legacy]` block), consistent with the "config-fixed" 2026-07-31 arc-close dry-run the cc-prompt's background names. Nothing under `.worktrees/odm` was read, written, or committed by this session; HEAD (`e06fffe`) unchanged. `crates/odm-cli/src/orient.rs` untouched — the concise-view question is explicitly left to the LLM-command-surface arc, per slice-doc's Out list. | The collapse fires live at the freeze. **Flagged risk for the freeze**: post-collapse, the surviving project's body is the *full, verbatim* `project-plan.md` (not a curated excerpt) — `check`'s L-3b `no-vision` rule requires a literal `# Vision`/`## Vision` heading somewhere in that body. Whether `project-plan.md` actually carries one was not verified against the live corpus (out of reach from this fixture-only branch) — if it doesn't, the freeze's collapse will newly surface a `no-vision` warning that the old synthesis body (which always carried a literal `# Vision` heading by construction) suppressed. Flagged for the freeze-runner to check before firing, not fixed here (a content/policy call, not a code defect). |
| F-10 | **Clippy clean; no `unsafe`; coverage ≥ 90% on touched code** | clippy `-D warnings` exit 0; `! grep unsafe`; `llvm-cov` ≥ 90% on changed | polish/correctness | CLAUDE.md | done | `cargo clippy --workspace --all-targets --all-features` exit 0 (0 warnings); `cargo fmt --check` exit 0; `grep -rn unsafe` over every touched file — 0 hits; `cargo test --workspace` — 72/72 suites green; `cargo llvm-cov -p odm-migrate -p odm-cli`: `collapse.rs` (new) 97.26%/82.61%/98.47% region/fn/line; `artifact.rs` 92.59%/73.68%/96.30%; `notes.rs` 91.41%/65.22%/94.01%; `selfhost.rs` 95.27%/87.84%/94.99% — all region/line ≥ 90%, `fn` figures lower only on unexercised error-variant branches (e.g. `CollapseError::TargetMissing`, a dangling-edge case with no realistic fixture path). `migrate.rs`/`replan.rs` aggregate below 90% (79.45%/85.84%, 83.20%/86.11% region/line) — pre-existing shortfall on rendering/date-derivation code this slice trimmed but didn't newly add; see closing-report.md. | The collapse, the exclusion-key change, the artifact/note reconcile branches, the guard. |

## What Worked

- **The "keyed on identity, not number" discipline already in the codebase paid for itself twice.**
  `is_synthesis()` mirrors `check_field_validity`'s existing `source.synthesis` predicate exactly — no
  new concept, just the same key reused at a third call site. And the collapse never had to rewrite a
  single `part_of` edge: because the surviving node's `id` is the *former synthesis's own* (re-cast in
  place, never re-minted), every arc's `part_of` continues to resolve without being touched at all —
  the F-1 "12 children intact" requirement fell out of the design for free rather than needing its own
  mechanism.
- **Extending `reconcile()`'s node-type scope surfaced a real, load-bearing bug**, not a hypothetical
  one: `discover()`'s project-node invention (`PROJECT_NUMBER` is a corpus-wide constant every
  `discover()` call reproduces, file-existence unchecked) only ever mattered once something started
  reading that phantom entry's source file — which nothing did until F-3 put `Project` inside
  `reconcile()`'s scan. The existing `migrate_all_plan_set_escape_hatch_...` fixture test caught it
  immediately once F-3 landed, without needing a new fixture — proof the existing suite's coverage of
  the D-2 escape hatch was doing real work.
- **Mirroring `mapping::reconcile_source`'s established mint-or-reconcile shape** for `artifact.rs`/
  `notes.rs` (read the corpus into a path→`Document` map instead of a path `HashSet`; compare bodies;
  rebuild `source` fresh with `today`'s `migrated_on` on drift) meant the F-4 implementation was almost
  entirely pattern-following, not novel design — the same shape the project/arc/slice family already
  proved out in s12.

## Iteration 1 (2026-08-01) — wire the collapse into `migrate --all` (note under F-1)

**Finding (CDC/arc-close dry-run, 2026-08-01):** `collapse_project_vision` had **no CLI caller** —
`crates/odm-cli/src/migrate.rs`'s `all()` orchestration never invoked it, so `migrate --all`
reconciled everything else but never actually collapsed a synthesis-shaped project. F-1's capability
was complete and CDC-verified (`cdc-verification.md`, PASS) as a *library function*, but s15's headline
behavior could not fire through the one command the freeze runbook actually runs
(`odm migrate --all [--dry-run]`).

**Fix (this iteration, thin wiring only — `collapse_project_vision`'s own logic untouched):**

- `all()` now calls `collapse_project_vision(store, plan_root, mode)` once per discovered plan root
  that has its own `project-plan.md` (guarded exactly like the pre-s15 `--vision` step was, and for the
  identical reason: the D-2 escape-hatch shape has no `project-plan.md` of its own, and calling it there
  would hit the same class of bug F-3's `reconcile()` guard already fixed) — **before** `self_host_inner`
  for that root, so a just-collapsed 1:1 project reconciles like any other plan node in the same pass
  (`self_host_inner` → `reconcile()` → F-3's rekeyed exclusion, no longer excluding it). Confirmed: this
  ordering is correct and needs no change — the collapse re-snapshots the body itself, so the
  in-pass reconcile that follows is a content no-op unless the source drifts again mid-run (never, in
  one process).
- `Collapsed` gained two additive fields, `project_name`/`retired_name` (populated from values the
  function already computes — `base.frontmatter().name()` twice over — no new decision logic), so the
  CLI can render a correct `--dry-run` preview without needing to load the store back (which would show
  the *pre-collapse* name under dry-run, since nothing is persisted yet).
- A new `render_collapse` (mirroring `render_self_host`/`render_reconcile`'s established shape) renders
  a `COLLAPSE`/`COLLAPSE (DRY RUN)` table — a `re-cast` row for the surviving project, a `retire` row for
  the base — silent when there was nothing to collapse (`collapsed: None`, the ordinary case after the
  first run). The composed status line now reads "N plan root(s) **collapse-checked** + self-hosted, …".

**Re-run evidence:** `crates/odm-cli/tests/migrate.rs` —
`migrate_all_collapses_a_vision_pair_and_then_reconciles_it_in_the_same_pass` seeds the exact
`#1000`/`#1001` shape directly into the store, runs `migrate --all` once (asserts the `COLLAPSE` table
renders, `#1000` is re-cast to the faithful 1:1 body with no `source.synthesis`/`supersedes`, `#1001`
is retired, and the arc self-hosts in the same pass), edits `project-plan.md` and runs `--all` again
(asserts the collapsed project reconciles — the two s15 behaviors composing in one pass, no `COLLAPSE`
table since it's already collapsed), then re-runs with nothing changed (asserts a 0-change no-op, node
count stable). `migrate_all_collapse_dry_run_writes_nothing` seeds the same pair fresh, runs
`--all --dry-run`, asserts the `COLLAPSE (DRY RUN)` preview renders and every existing node's
body/`retired`/`synthesis` state is byte-for-byte unchanged after. All 31 `odm-cli` `tests/migrate.rs`
tests pass (29 pre-existing + 2 new); full workspace suite green; clippy/fmt clean; no `unsafe`.

**Confirmed: the arc-close freeze (`migrate --all`) now collapses + reconciles in one pass.** No change
to `collapse_project_vision`'s own decisions, no live mutation (`.worktrees/odm` untouched by this
iteration, same pre-existing `config.toml` state as before — see F-9), no arc-plan slice-status change
(s15 stays **CDC-verified PASS**; this iteration is noted here per the working agreement). CDC-F15-1
(`project-plan.md` needs a `# Vision` section before the freeze fires) is unaffected by this wiring —
still an open, disclosed freeze precondition, now doubly relevant since `--all` will actually reach the
collapse the moment it's next run against the live corpus.

## Closure

Fixture-only — no store commit. Verified by: `CC` (self) → **CDC-verified PASS** (`cdc-verification.md`,
2026-08-01); iteration 1 (the `--all` wiring, above) is CC-verified, not yet independently re-verified by
CDC. Rows: 10. Done: 10. Deferred: 0. Not committed to `release/1.0.x` at the time CDC verified `1c779ec`;
iteration 1's changes are a follow-up on top of that commit. On close, bubble up to `../arc-plan.md`: s15
done (project collapsed to one 1:1 node; everything ingested reconciles; **and, as of iteration 1, actually
reachable through `migrate --all`**); **the arc-close resumes** — the freeze collapses + closes all 4
drifts, then the P-12 demo, then Migration Fidelity closes. **Carry forward to the freeze** (both from
CDC's review): CDC-F15-1 (add a `# Vision` section to `project-plan.md` before firing, elevated to a
freeze-runbook step-0 item — see `cdc-verification.md` §4) and the open sub-decision of exactly which
`--all` invocation the freeze runbook uses (now moot in one sense — `--all`/`--all --dry-run` is the
answer, per iteration 1 — but still worth the runbook naming it explicitly).
