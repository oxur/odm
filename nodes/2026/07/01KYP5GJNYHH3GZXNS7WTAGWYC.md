---
id: 01KYP5GJNYHH3GZXNS7WTAGWYC
number: 502265800
type: artifact
schema: artifact/v1.1
name: 'Slice 03 (Arc 03): orient / brief + bare-`odm`'
created: 2026-06-25
updated: 2026-06-25
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc03-rollup-and-orient/slice03-orient-brief/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKV4JZXW92JYVG969F
---
# Slice 03 (Arc 03): orient / brief + bare-`odm`

> Per LEDGER_DISCIPLINE. Cargo rows reproduced on a local 1.95.0 toolchain (the
> 1.85+ floor is met here); CDC re-runs them via CI / a local 1.85+ toolchain for
> the independent gate. CDC-authored acceptance rows; CC filled Status/Evidence/
> Notes per commit. Five-iteration cap (closed in 1).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| O-1 | `orient` composes over the slice02 `Rollup` model (reuses `Rollup::assemble`, not re-derived) and renders the sections in order: vision → current focus → ready/blocked → integrity → drift | `cargo test -p odm-cli orient_section_order` → ok | serious | 0013 §4.1/§6 / arc-plan D-3 | done | `f3d9f21`; `orient_section_order` → 1 passed (asserts the six section offsets are strictly increasing). `orient` calls `Rollup::assemble` (`orient.rs`), builds no graph itself. | Terse orient view, distinct from the full `ROLLUP.md`. |
| O-2 | Vision extraction: `# Vision` section (case-insensitive ATX) if present, else the lead section (body before the first ATX heading); truncated to a line budget with a `odm show` continuation marker | `cargo test -p odm-cli vision_extraction_rule` → ok | serious | arc-plan D-1a | done | `f3d9f21`; `vision_extraction_rule_*` → 6 passed (heading-preferred, case-insensitive + nested subheading, lead-section fallback, truncation-with-marker, title-only ⇒ empty, under-budget ⇒ no marker). | Pure helper `extract_vision` over `Document.body()`; unit-tested in `orient.rs`. |
| O-3 | `orient` leads with the current project's `name` + the extracted vision — **not** the whole body | `cargo test -p odm-cli orient_leads_with_vision` → ok | serious | arc-plan D-1 | done | `f3d9f21`; `orient_leads_with_vision` → 1 passed (asserts `VISION  #1 Proj` + vision text, and that the raw `# Vision` heading is not dumped). | Loads the project `Document` (frontmatter + body). |
| O-4 | The current project/arc is resolved from `.odm/context.json` (`Context`); `orient` shows the current arc + its status vector after vision | `cargo test -p odm-cli orient_uses_context` → ok | serious | arc01 slice05 / 0013 §7 | done | `f3d9f21`; `orient_uses_context` → 1 passed (after `use arc` + `set-gate`, the focus block shows `arc #2 Arc one` and `in-progress=reproduced`). | Reuses `Context::load`; arc status read from the model tree. |
| O-5 | Ready/blocked surface from the model: the ready frontier shows the **soft-sat ⚠** (`ReadyNode.soft`); blocked nodes name their reasons | `cargo test -p odm-cli orient_ready_blocked_softsat` → ok | serious | 0013 §4.4 / slice02 ruling 2 | done | `f3d9f21`; `orient_ready_blocked_softsat*` → 2 passed (ready node carries `⚠ soft dep … at evidence=attested`; a blocked node renders all three reason kinds: unsatisfied / low-evidence / blocked-by). | Soft signal travels with the ready node (ruling 2). |
| O-6 | `orient` surfaces `check` integrity findings inline (every **Error**: orphan, cycle-without-tear) so a structural break is unmissable | `cargo test -p odm-cli orient_surfaces_integrity` → ok | serious | slice02 ruling 3 / 0015 §2 | done | `f3d9f21`; `orient_surfaces_integrity` → 1 passed (the INTEGRITY block shows `[orphan] … #5 Orphan`). | Reuses `commands::integrity_findings` → `aggregate` — check is not reimplemented. |
| O-7 | The drift line reads "not yet tracked (A5)" — consistent with the rollup, no fabricated data | `cargo test -p odm-cli orient_drift_placeholder` → ok | correctness | arc-plan Q-A3-2 | done | `f3d9f21`; `orient_drift_placeholder` → 1 passed (DRIFT block reads `not yet tracked (A5)`). | Consistent with slice02's rollup drift placeholder. |
| O-8 | Bare `odm` (no subcommand) runs `orient` and **never bare-errors** | `cargo test -p oxur-odm --test cli bare_odm_orients` → ok | serious | 0013 §7 | done | `f3d9f21`; `bare_odm_orients` → 1 passed (the real binary with no args exits 0 and prints the orient view). | Binary-level (real process / `ExitCode`); subcommand is `Option<Command>` defaulting to Orient. |
| O-9 | No-current-project fallback (all exit 0): none → affordance to `odm new project`; exactly one → orient on it; multiple w/o context → list + prompt `odm use project <ref>` | `cargo test -p odm-cli orient_no_project_fallback` → ok | serious | 0013 §7 (never bare-errors) | done | `f3d9f21`; `orient_no_project_fallback` → 1 passed (all three branches asserted, each `code == Some(0)`). | Three branches; each exits 0. |
| O-10 | `brief` is an alias of `orient` (identical output) | `cargo test -p odm-cli brief_aliases_orient` → ok | polish | 0013 §7 | done | `f3d9f21`; `brief_aliases_orient` → 1 passed (byte-identical output). | clap `#[command(visible_alias = "brief")]` → same `Orient` variant. |
| O-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-core + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | done | `f3d9f21`; clippy `--all-targets --all-features --workspace` → exit 0; `unsafe` grep → no matches (exit 1); `cargo llvm-cov` **line**: odm-core **98.69%**, odm-cli **93.44%** (orient.rs 97.78%). | `fmt --check` clean; full workspace `cargo test` → 218 passed. |

## What Worked

- **Composing over the slice02 model paid the dividend it was built for.** `orient`
  is a thin reader: `Rollup::assemble` for the structural view and
  `commands::integrity_findings` (a wrapper over the existing `aggregate`) for
  integrity. orient re-derives nothing — the graph is walked once, in odm-core.
- **The two slice02 CDC rulings landed cleanly.** Ruling 2 (soft-sat ⚠ on the
  ready frontier) fell out for free because the model already carries
  `ReadyNode.soft`; ruling 3 (surface `check` errors inline) only needed a small
  `pub(crate)` wrapper, not a re-walk.
- **Vision extraction as a pure helper** made the fiddly part (ATX levels,
  same-or-higher-heading boundary, line budget, marker) fully unit-testable in
  isolation — six cases, no store needed.
- **`Option<Command>` + a clap `visible_alias`** gave bare-`odm`-orients and the
  `brief` alias with no bespoke dispatch logic and no risk to existing
  subcommands (the binary suite still green).
- **Reusing the rollup display atoms** (`label`/`dep_label`/`status_inline`) kept
  orient's rendering consistent with `ROLLUP.md` and avoided a second copy.

## Closure

Closed at commit `f3d9f21` on 2026-06-25 (CC implementation). CDC verification:
pending (cargo rows via CI / local 1.85+). Total rows: 11. Done: 11. Deferred: 0.
No-op: 0.
