# Slice 03 (Arc 03): orient / brief + bare-`odm`

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+ (sandbox has no
> 1.85 toolchain). CDC-authored acceptance rows; CC fills Status/Evidence/Notes per
> commit. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| O-1 | `orient` composes over the slice02 `Rollup` model (reuses `Rollup::assemble`, not re-derived) and renders the sections in order: vision → current focus → ready/blocked → integrity → drift | `cargo test -p odm-cli orient_section_order` → ok | serious | 0013 §4.1/§6 / arc-plan D-3 | open | | Terse orient view, distinct from the full `ROLLUP.md`. |
| O-2 | Vision extraction: `# Vision` section (case-insensitive ATX) if present, else the lead section (body before the first ATX heading); truncated to a line budget with a `odm show` continuation marker | `cargo test -p odm-cli vision_extraction_rule` → ok | serious | arc-plan D-1a | open | | Pure helper over the project `Document.body()`. |
| O-3 | `orient` leads with the current project's `name` + the extracted vision — **not** the whole body | `cargo test -p odm-cli orient_leads_with_vision` → ok | serious | arc-plan D-1 | open | | Loads the project `Document` (frontmatter + body). |
| O-4 | The current project/arc is resolved from `.odm/context.json` (`Context`); `orient` shows the current arc + its status vector after vision | `cargo test -p odm-cli orient_uses_context` → ok | serious | arc01 slice05 / 0013 §7 | open | | Reuses `Context::load`. |
| O-5 | Ready/blocked surface from the model: the ready frontier shows the **soft-sat ⚠** (`ReadyNode.soft`); blocked nodes name their reasons | `cargo test -p odm-cli orient_ready_blocked_softsat` → ok | serious | 0013 §4.4 / slice02 ruling 2 | open | | Soft signal travels with the ready node. |
| O-6 | `orient` surfaces `check` integrity findings inline (every **Error**: orphan, cycle-without-tear) so a structural break is unmissable | `cargo test -p odm-cli orient_surfaces_integrity` → ok | serious | slice02 ruling 3 / 0015 §2 | open | | Reuse the existing check aggregation — don't reimplement. |
| O-7 | The drift line reads "not yet tracked (A5)" — consistent with the rollup, no fabricated data | `cargo test -p odm-cli orient_drift_placeholder` → ok | correctness | arc-plan Q-A3-2 | open | | |
| O-8 | Bare `odm` (no subcommand) runs `orient` and **never bare-errors** | `cargo test -p oxur-odm --test cli bare_odm_orients` → ok | serious | 0013 §7 | open | | Binary-level (real process), reuses slice01's `assert_cmd` suite. |
| O-9 | No-current-project fallback (all exit 0): none → affordance to `odm new project`; exactly one → orient on it; multiple w/o context → list + prompt `odm use project <ref>` | `cargo test -p odm-cli orient_no_project_fallback` → ok | serious | 0013 §7 (never bare-errors) | open | | Three branches; each exits 0. |
| O-10 | `brief` is an alias of `orient` (identical output) | `cargo test -p odm-cli brief_aliases_orient` → ok | polish | 0013 §7 | open | | A distinct terser `brief` mode is out of scope. |
| O-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-core + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Total rows: 11.)_
