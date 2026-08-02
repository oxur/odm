---
id: 01KYZTRFN66FX6MMS6X86KFN2G
number: 595180100
type: artifact
schema: artifact/v1.1
name: 'Slice 15 (Migration Fidelity) — CDC verification: Reconcile completeness + vision collapse'
created: 2026-08-01
updated: 2026-08-01
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice15-reconcile-completeness/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-01
edges:
  part_of: 01KYZTRDEQ5KD76YCNNB7DHEG3
---
# Slice 15 (Migration Fidelity) — CDC verification: Reconcile completeness + vision collapse

> **Arc:** Migration Fidelity · **Slice:** 15 · **Verifier:** CDC (independent) · **Date:** 2026-08-01 ·
> **Method:** LEDGER-DISCIPLINE v2.0 §A. **Fixture slice — no live class-(b) row.** Code + fixtures + the
> ODD-0025 amendment reproduced by direct read on `release/1.0.x`; runtime execution
> (`cargo`/`clippy`/`llvm-cov`) attested-by-CC → CI.
> **Under review:** `ledger.md` (F-1…F-10), commit `1c779ec` (`release/1.0.x`). `odm` store HEAD unchanged
> at `e06fffe` (reproduced — no live mutation).

## Verdict

**PASS — CDC-verified.** The project-vision pair is collapsed cleanly into one faithful 1:1 project node,
the reconcile-exclusion is rekeyed on `source.synthesis` (so the project reconciles like any plan node and
genuine syntheses stay protected), artifacts and notes are mint-or-reconcile, and the `is_excluded` guard
is in. The collapse is id/number/part_of-preserving and idempotent; the synthesis *library* is kept. CC
found and fixed three adjacent issues under the same theme (a second vision-mutation path in `replan`, a
D-2 escape-hatch regression, and an L-3b retired-project no-vision bug) and honestly flagged one risk —
**which I confirm, characterize precisely, and elevate to a freeze precondition (CDC-F15-1, §4)**: it is a
`Warning`, not a hard `check` failure, but firing the collapse without first adding a `# Vision` section to
`project-plan.md` would lose the curated vision and blank `orient`. No live mutation (`odm@e06fffe`).

## 1. Collapse (F-1/F-2) — reproduced

`collapse.rs::collapse_project_vision` (new, +536):

- Finds the synthesis-bearing project node (`node_type == Project && source.synthesis.is_some()`); **absent
  → `CollapseReport{collapsed: None}`, a no-op** — so a re-run over an already-collapsed store does nothing
  (idempotent, F-6, test `…idempotent`).
- Re-snapshots that node's body from `project-plan.md` **under the §2.1 hard body-hash gate**
  (`verify_body_hash`), restores its name to the base's real project name, clears `supersedes`, rebuilds
  `source` with **no `synthesis` key** — **`id`/`number`/`edges.part_of` untouched** (clone of the existing
  fm), so the surviving node is the *same* `#1000` and its **12 arc `part_of` children need no rewrite**.
- Retires the 1:1 base (`#1001`) supersede-don't-delete, reason recorded.
- Validates the supersede shape (exactly one target, target is a Project) with typed errors.

`--vision` removed from the clap surface and `apply_project_vision` un-wired from the migrate path
(migrate.rs −333 net); the `synthesis` module itself is kept for future genuine merges (F-2). **Bonus,
disclosed:** CC also removed a *second* vision-body mutation path in `replan::restamp` (−81) that would
have re-introduced a curated body post-collapse — a real catch, verified gone.

## 2. Reconcile completeness (F-3/F-4/F-5) — reproduced

- **F-3 exclusion rekey.** `selfhost.rs::is_synthesis(doc) = source().synthesis.is_some()` replaces
  `node_type == Project` at the three sites (backfill :333, `reconcile_source` :~548, the loop). Post-collapse
  the project carries no synthesis → it reconciles; a genuine synthesis stays excluded. **Both directions
  tested:** `reconcile_reconciles_a_drifted_project_body` **and**
  `reconcile_excludes_a_synthesis_bearing_project`. CC disclosed + fixed a regression this exposed in the
  D-2 escape-hatch self-host path — verified.
- **F-4 mint-or-reconcile.** `artifact.rs`/`notes.rs` gain a `Reconciled{Artifact,Note}` bucket — a
  drifted already-covered node re-snapshots in place, **`id` unchanged ("a reconcile never re-mints")**.
  Tests: `artifact_mint_reconciles_a_drifted_already_minted_artifact`,
  `notes_mint_reconciles_a_drifted_already_minted_note`, each with idempotence + dry-run-writes-nothing.
  This closes `#509907700` (slice10 ledger) and the same latent gap in the note family.
- **F-5 `is_excluded` guard.** `artifact.rs:237` + `notes.rs:129` apply `legacy::is_excluded` — `index.md`
  / `templates/*` never minted regardless of root.

## 3. Model + collateral (F-7/F-8) — reproduced

ODD-0025 **v1.3** amends §2.3 (project vision reversed out of the synthesis model — records the operator
decision, the id/number-preserving collapse, `#1001` retired) and the reconcile-exclusion note (§2.3/§2.9:
the sole exclusion is now `source.synthesis`, was `node_type == Project`). Well-formed, cited,
amend-not-work-around. CC also fixed a collateral L-3b bug (a retired project wrongly flagged `no-vision`)
— reproduced: the `check` loop now `continue`s on `retired().is_some()` (commands.rs:1830). 72/72 suites,
clippy/fmt/no-`unsafe`/coverage → CI.

## 4. Finding CDC-F15-1 (elevated freeze precondition) — add a `# Vision` section to `project-plan.md`

CC flagged "whether the live `project-plan.md` states a `# Vision` heading." **It does not** — reproduced:
its headings are `# odm v1.0.0 — Project Plan …`, `## 1. Definition of done & boundaries`, `## 2 … 5`,
`## Version History` — no `Vision` heading at any level. The collapse sets `#1000`'s body to
`project-plan.md` **verbatim** (hash-gated), so post-collapse the project node has no `# Vision` section.

Consequences, precisely (I checked the severity so this isn't over-stated):

- **`check`: a `no-vision` *Warning*, not an Error** (commands.rs:1830) — so `check` **stays green**; this
  is *not* a hard blocker.
- **But the curated vision is lost.** The `# Vision` statement currently living in `#1000`'s synthesis body
  ("`odm` is a markdown/git-native … In scope … Non-goals …") disappears when `#1000` is re-snapshotted to
  `project-plan.md` and `#1001` is retired — it exists in neither node nor the source.
- **`orient` blanks the vision.** `orient::vision_section(...).unwrap_or_else(lead_section)` (orient.rs:378)
  falls back to the plan's lead when no `# Vision` section exists — so `orient` shows the plan title/intro,
  not a vision, plus the "add a `# Vision` section" hint.

**Resolution (the correct completion of the collapse, not a workaround):** before the freeze, **add a
`# Vision` section to `project-plan.md`** — lifting the content from `#1000`'s current body. In the new
model the vision *is* a section of the 1:1 project plan: `check` expects it, `orient` renders it, and the
curated text is preserved in the source where it belongs. This is a source-side prep step, exactly like the
s14 store-config fix — do it, then re-run the freeze dry-run. **This is CDC-F15-1 and it joins the freeze
runbook as a step-0 item.**

## 5. Ledger — CDC disposition

F-1…F-8 reproduced in code + non-vacuous fixtures (structural / attested→CI on execution). F-9 (no live
mutation) reproduced — `odm@e06fffe` unchanged; `orient` view correctly left to the LLM arc. F-10
attested→CI. **10 rows, no silent drops.** The three CC-found adjacent fixes (replan mutation path,
escape-hatch regression, L-3b) are disclosed, not smuggled — good discipline. CDC adds one finding
(CDC-F15-1, §4).

## 6. Bubble-up (PM Part IV)

- **Did s15 deliver?** Yes — the pair is collapsed to one faithful 1:1 project node, everything ingested
  reconciles, index/template can't be minted, the synthesis capability is preserved-but-un-wired;
  `odm@e06fffe` untouched.
- **Arc-plan change:** flip s15 → **CDC-verified PASS** (v2.30). **The arc-close resumes**, with the freeze
  runbook now carrying **two step-0 items**: (1) *[done]* store-config `docs_directory = "./docs"`; (2)
  **add a `# Vision` section to `project-plan.md`** (CDC-F15-1) — then the dry-run-adjudicated freeze
  collapses the project + closes all 4 drifts → P-12 demo → Migration Fidelity closes.

## Closure

s15 **CDC-verified PASS** on 2026-08-01 (`release/1.0.x` `1c779ec`; `odm@e06fffe` unchanged). Clean collapse
(id/number/part_of-preserving, idempotent), `source.synthesis`-keyed reconcile, artifacts+notes
mint-or-reconcile, the guard, ODD-0025 v1.3 — every row with a non-vacuous test; three adjacent bugs caught
and disclosed. One elevated precondition: **`project-plan.md` needs a `# Vision` section before the freeze**
(CDC-F15-1) — else the collapse loses the vision and blanks `orient` (a warning, not a check failure). The
arc-close resumes.

_Verified by: CDC (independent), 2026-08-01 — against `release/1.0.x` (`1c779ec`); `odm@e06fffe` unchanged.
Code/fixtures/ODD reproduced by direct read; execution rows attested-by-CC → CI._

---

## Addendum — Iteration 1 (wire the collapse into `migrate --all`) — CDC re-verified

> 2026-08-01 · commit `3617813` (`release/1.0.x`) · `odm@e06fffe` unchanged.

The s15 (`1c779ec`) freeze dry-run adjudication caught that `collapse_project_vision` had **no CLI
caller** — the collapse was correct but unreachable, so `migrate --all` never fired it (CDC iteration-1
prompt). **The wire lands and re-verifies clean — invocation path traced, not just logic:**

- `migrate.rs::all()` now calls `odm_migrate::collapse::collapse_project_vision(store, plan_root, mode)`
  per plan root, **guarded on `plan_root.join("project-plan.md").is_file()`** (the D-2 escape-hatch shape —
  an `arc*`-only plan root — is never handed to the collapse; CC cites the same bug class s15 F-3 fixed),
  **before `self_host_inner`** so the just-collapsed 1:1 project reconciles in the same pass, `mode` threaded
  for dry-run.
- `render_collapse` emits a `COLLAPSE` / `COLLAPSE (DRY RUN)` table (re-cast project + retired base + a
  one-pair summary); no-op when `report.collapsed` is `None` (nothing to collapse).
- **Compose fixture** `migrate_all_collapses_a_vision_pair_and_then_reconciles_it_in_the_same_pass`
  (non-vacuous): asserts the collapsed project's body equals the *amended* `project-plan.md` after one
  `--all` run — i.e. collapse **then** reconcile compose. Plus `migrate_all_collapse_dry_run_writes_nothing`.
- 72/72 suites, clippy/fmt clean; `.worktrees/odm` untouched.

**Disposition:** iteration-1 code **CDC-verified**. s15 stays **CDC-verified PASS** (this was a reachability
wire, not a logic change). **Final live sign-off is the freeze dry-run re-run** (operator-run; the binary is
macOS): the adjudication gate is now that a **`COLLAPSE (DRY RUN)`** section appears — `#1000` "would
re-cast", `#1001` "would retire" — on top of the already-confirmed reconciles, with still no `no-vision`,
no index/template mint, and nothing unexpected. When that preview is clean, the freeze is clear to fire.
