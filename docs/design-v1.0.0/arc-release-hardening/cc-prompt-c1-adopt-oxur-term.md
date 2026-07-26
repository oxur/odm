# cc-prompt — RH C-1: Adopt Oxur themed output in odm (via `oxur-term`)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-1 · **Covers:** `F-1`
> **Decision of record:** `adr-c1-oxur-table-re-extraction.md` (Route B). **Settled scope:**
> the crate was extracted as **`oxur-term`** (table **+** `common` terminal helpers), not
> table-only — so themed status messages are **in scope** here, not deferred.
> **Dependency (already published):** `oxur-term = { git = "https://github.com/oxur/oxur", tag = "0.2.1" }`.
> **Do NOT also depend on `oxur-cli`** — `oxur-term` supersedes it for odm, and
> `oxur-cli --no-default-features` still fails to compile (crate-level `config` is ungated →
> `config::paths` needs the binary-only `dirs`). Route B = depend on `oxur-term` alone.

## Goal

Replace odm's plain raw-`tabled` + `writeln!` CLI output with Oxur's themed rendering, so
`odm list` (and the other tabular commands) render in the established **warm-orange theme** —
the `F-1` UAT ask, and what odm's own ODD-0012/0013 §11 already spec.

## `oxur-term` API (what to call)

- `oxur_term::table::{OxurTable, TableStyleConfig, Tabled, Builder, Cell, TabledColor}`
  - `OxurTable::new(rows).with_title(t).with_footer().render() -> String`
  - `#[derive(oxur_term::table::Tabled)]` on the row struct (drop the direct `tabled` derive path)
  - `TableStyleConfig::default()` = warm-orange theme
  - low-level helpers under `oxur_term::table::helpers::*` if a command needs per-cell colour
- `oxur_term::common::output::{success, error, info, warning}` for status lines
- `oxur_term::common::progress::ProgressTracker` if a long op wants progress

## Scope

**In:**
1. **Deps hygiene (do first):** add `oxur-term.workspace = true` to `odm-cli/Cargo.toml`; and
   **remove the `oxur-cli` entry from `odm/Cargo.toml`** `[workspace.dependencies]` (redundant +
   won't compile with `--no-default-features`). Confirm `cargo tree -p odm-cli` shows `oxur-term`
   pulling only `tabled`/`colored`/`serde`/`toml`/`anyhow` — and **no** `oxur-cli`, `oxur-lang`,
   `clap`-via-oxur, `tokio`, etc.
2. Route the table sites through `OxurTable` + `TableStyleConfig` (warm-orange):
   - `odm-cli/src/commands.rs` — the `list` table (`#[derive(Tabled)]` row struct with
     NUMBER/TYPE/NAME/ID columns; currently `tabled::{Table, settings::Style}`).
   - `odm-cli/src/migrate.rs` — the `tabled::builder::Builder` + `Style` table.
   - any other command that prints a table (`orient`/`rollup`/`show` if applicable).
3. Route status/informational lines through `oxur_term::common::output::{success,error,info,warning}`
   where odm currently hand-rolls them (the themed-terminal half of `F-1`).

**Out (later chunks — do not do here):** C-2 type taxonomy (`odd`→`design`, add `research`),
C-3 `list` overhaul (columns/tree/widths/elision), C-4 renames, C-5 `self-host`→`migrate`.
**C-1 changes rendering only** — keep the current columns, rows, and command surface; C-3
restructures them.

## Steps

1. Deps hygiene (scope item 1). Build to confirm the graph is clean.
2. Swap the `commands.rs` `list` table + `migrate.rs` table to `OxurTable` with
   `TableStyleConfig::default()`. Preserve columns/rows exactly (rendering swap, not a content
   change).
3. Swap hand-rolled status lines to `oxur_term::common::output::*`.
4. `cargo build`; `cargo test` (workspace); `cargo clippy --all-targets -- -D warnings`;
   `cargo fmt --check`. No `unsafe`, no new warnings.
5. **Reflexive check:** re-run `odm self-host` + `odm check` — corpus stays green, and
   `odm list` now renders themed.

## Acceptance / ledger

- Feeds **RH-1** (C-1 closed). Closes `attested`-on-close, `reproduced` on CI-green.
- `odm list` renders in the warm-orange themed table on a real terminal — attach a capture to
  the closing report (the `F-1` "no colours" complaint visibly resolved).
- odm's dependency graph gains **only** `oxur-term` + its 4 transitive deps; `oxur-cli` and the
  compiler/REPL stack are **absent** from `cargo tree` (the Route-B invariant).
- Columns/content unchanged vs. pre-C-1 (diff the row data; restructuring is C-3).

## Method / housekeeping

- One branch, one mergeable diff; five-iteration cap. CC implements on local 1.85+; cargo rows
  attested-by-CC → reproduced-on-CI.
- **Base branch (settle at start):** builds on the A6 slice04 self-host tip
  (`arc06-slice04-self-host-cutover`, off `release/1.0.x`, unmerged) — branch off it directly,
  or merge green-A6-so-far to `release/1.0.x` first and branch from there.
- On close: bubble up to `arc-release-hardening/arc-plan.md` (F-1 dispositioned; RH-1 done).
