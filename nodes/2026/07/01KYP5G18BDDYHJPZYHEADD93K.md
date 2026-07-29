---
id: 01KYP5G18BDDYHJPZYHEADD93K
number: 583747000
type: artifact
schema: artifact/v1.1
name: 'cc-prompt — RH C-1: Adopt Oxur table styling in odm (via `oxur-table`)'
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/cc-prompt-c1-adopt-oxur-table.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# cc-prompt — RH C-1: Adopt Oxur table styling in odm (via `oxur-table`)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-1 · **Covers:** `F-1`
> **Decision of record:** `adr-c1-oxur-table-re-extraction.md` (Route B — depend on the
> standalone `oxur-table` crate, **not** `oxur-cli`).
> **Hard dependency:** the `oxur-table` crate must exist first (oxur repo; ADR §6a). If it is
> not yet published/available, this chunk is **blocked** — do not vendor or reimplement the
> theme (that is ADR Option C, explicitly rejected).

## Goal

Replace odm's plain raw-`tabled` + `writeln!` CLI output with Oxur's themed table rendering,
so `odm list` (and the other tabular commands) render in the established **warm-orange theme** —
the F-1 UAT ask, and what odm's own ODD-0012/0013 §11 already spec ("oxur-cli/tabled").

## Scope

**In:**
- Add `oxur-table` as a dependency of `odm-cli` (distribution per ADR §7.2 — crates.io version
  or git dep pinned to a tag; confirm which at start).
- Route odm's table construction through `oxur_table::OxurTable` + `TableStyleConfig`
  (`OxurTable::new(rows).with_title(..).with_footer().render()`), with the **warm-orange
  default theme**.
- Wire the theme into the commands that currently emit tables (start with `list`; include
  `orient`/`rollup`/`show` wherever a table is printed).
- Terminal status helpers (`success`/`error`/`info`/`warning`): **gated on ADR §7.1.** If the
  shared crate ships `common::output`, use it; otherwise defer themed status messages to a
  follow-up and keep odm's current output for now (disclose in the closing report — do not
  silently drop).

**Out (belongs to later chunks, do not do here):**
- C-2 type taxonomy (`odd`→`design`, add `research`), C-3 `list` overhaul (columns, tree,
  widths, elision), C-4 renames, C-5 `self-host`→`migrate`. This chunk changes **rendering
  only**, not columns/content/命令 surface. Keep the current column set; C-3 restructures it.

## Steps

1. Confirm `oxur-table` availability + the dep form (crates.io vs git tag). Add it to
   `odm-cli/Cargo.toml`. Confirm the 3–4-crate dep tree (`cargo tree` — expect
   `tabled`/`colored`/`serde`(+`toml`?), **no** `oxur-lang`/`clap`/`tokio`).
2. Find odm's current table-emitting code (raw `tabled::Builder` + `writeln!`). Replace with
   `OxurTable` construction; apply `TableStyleConfig::default()` (warm-orange).
3. Preserve current columns/rows exactly (rendering swap, not a content change).
4. `cargo build`; `cargo test` (workspace); `cargo clippy --all-targets -- -D warnings`;
   `cargo fmt --check`. No `unsafe`, no new warnings.
5. **Reflexive check:** re-run `odm self-host` and `odm check` — the regenerated corpus stays
   green and `odm list` now renders themed (RH-7 is the arc-scale version of this).

## Acceptance / ledger

- Feeds **RH-1** (C-1 closed). Row closes `attested`-on-close, `reproduced` on CI-green.
- `odm list` renders in the warm-orange themed table on a real terminal (attach a capture to
  the closing report — the F-1 "no colours" complaint is visibly resolved).
- odm's dependency graph gains **only** `oxur-table` + its 3-ish transitive deps; `oxur-cli`
  and the compiler/REPL stack are **absent** from `cargo tree` (the whole point of Route B).
- Columns/content unchanged vs. pre-C-1 (diff the row data; restructuring is C-3).

## Method / housekeeping

- One branch, one mergeable diff; five-iteration cap. CC implements on local 1.85+; cargo rows
  attested-by-CC → reproduced-on-CI.
- **Base branch (open — settle at start):** the C-1 code builds on the A6 slice04 self-host tip
  (`arc06-slice04-self-host-cutover`, off `release/1.0.x`, unmerged) — branch off it directly,
  or merge green-A6-so-far to `release/1.0.x` first and branch from there. (This was flagged in
  the RH arc-plan for C-1 to decide.)
- On close: bubble up to `arc-release-hardening/arc-plan.md` (F-1 dispositioned; RH-1 done;
  note whether §7.1 terminal-helpers landed or deferred).
