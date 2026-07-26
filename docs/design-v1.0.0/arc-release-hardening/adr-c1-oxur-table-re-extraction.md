# ADR — Re-extract `oxur-table` as a standalone crate

> **Status:** ACCEPTED + extracted (see §0 Decision update) · **Date:** 2026-07-25
> **Authors:** Duncan McGreggor + CDC (odm session)
> **Primary repo:** `oxur/oxur` (the change lands here) · **Driving consumer:** `oxur/odm`
> **Affected crates:** `oxur-cli` (source), new `oxur-table` (target), `oxur-odm` (back-compat consumer), `odm-cli` (new consumer)
> **Related:** ODD-0001 (Oxur Letter of Intent — lists `oxur-table` as a standalone crate); oxur design-0015 "oxur-table API (re)Design" (Final, 2025-12-31); `oxur-cli/src/table/README.md` (the late-2025 fold record); odm ODD-0012/0013 §11 (odm CLI output spec = "oxur-cli/tabled"); odm Release-Hardening arc `F-1` / chunk `C-1`.
>
> **This ADR is written to be picked up cold by an oxur-context session.** It carries the odm-side motivation, the full evidence, and a turn-key extraction plan against oxur-repo paths. You should not need the odm repo open to execute it.

## 0. Decision update (2026-07-25, post-extraction)

The decision was taken and executed. Recording the settled choices against the open questions
in §7, and the actual on-disk state (verified in the oxur repo):

- **§7.1 scope — RESOLVED: widen.** The crate was extracted as **`oxur-term`** (not table-only
  `oxur-table`), holding **both `table` and `common`** (`io`/`output`/`progress`). odm therefore
  gets themed status helpers (`success`/`error`/`info`/`warning`) from the same source. The name
  reflects the wider scope. *(Everywhere below that says "oxur-table", read "oxur-term".)*
- **§7.2 distribution — RESOLVED: git tag.** odm consumes it as
  `oxur-term = { git = "https://github.com/oxur/oxur", tag = "0.2.1" }`. (crates.io publish stays
  a later option.)
- **§7.3 `toml` — RESOLVED: yes.** `oxur-term`'s deps are `tabled`/`colored`/`serde`/`toml`/`anyhow`.
- **Extraction done (oxur repo):** `crates/oxur-term/` created (`src/lib.rs` + `table/*` +
  `common/*`); added to workspace members; `oxur-cli` now `pub use oxur_term::{common, table}`
  (back-compat re-export); workspace version bumped to `0.2.1`. The former in-repo consumer
  `oxur-odm` is no longer a workspace member, so the re-export is for external/future consumers.
- **Consequence for odm — depend on `oxur-term` ALONE.** Do **not** also depend on `oxur-cli`:
  `oxur-cli`'s crate-level `config` is still ungated (`lib.rs:68`), so `oxur-cli --no-default-features`
  still fails at `config::paths → dirs`. A stray `oxur-cli` entry in odm's `[workspace.dependencies]`
  should be removed (it's redundant now — `oxur-term` covers table + common). The optional
  config-gating hygiene fix in §6a remains optional and is not on odm's path.

Remaining work is the **odm-side C-1 consumption** — see `cc-prompt-c1-adopt-oxur-term.md`.

## 1. Context

**What odm is (one paragraph, for oxur context).** `odm` (`oxur/odm`, binary `odm`) is a markdown/git-native, dependency-ordered planning substrate — a sibling project to oxur that reuses oxur's CLI infrastructure. It is in v1.0.0 release hardening. Hands-on UAT of the self-hosted tool produced a top-priority finding (`F-1`): odm's CLI output is **plain, uncoloured** raw `tabled` + `writeln!`, and it should instead use **oxur's established warm-orange themed table styling** — the same look `oxur-odm` (odm's direct ancestor, still in this repo) already renders. odm's own design intent already mandates this: ODD-0012/0013 §11 spec odm's CLI output as "oxur-cli/tabled". The code drifted; this realigns it.

**The blocker that sent us here.** odm cannot simply depend on `oxur-cli` today. `oxur-cli`'s `default = ["binary"]` pulls the whole compiler stack (`oxur-lang`/`oxur-comp`/`oxur-repl`) plus `clap`, `tokio`, `crossterm`, `reedline`, etc. Depending with `default-features = false` *should* shed all of that and leave just the library surface — but it **does not compile** as-is (see §3). Rather than patch `oxur-cli`'s feature gating and take on a coupling to its REPL release cadence, we restore the original design: `oxur-table` as its own small crate. ODD-0001 and the Final design-0015 both describe it that way; `oxur-cli/src/table/README.md` records that it *was* its own crate until a late-2025 convenience fold into `oxur-cli/src/table`. This ADR reverses that fold.

## 2. Decision

**Extract the `table` module out of `oxur-cli` into a new standalone workspace crate, `oxur-table`, and have both `oxur-cli` (via a back-compat re-export) and `odm` depend on it.**

odm then depends **only on `oxur-table`** — a crate whose entire dependency surface is `tabled` + `colored` + `serde` — and never compiles `oxur-cli`. The `config`/`dirs` blocker below becomes irrelevant to odm.

## 3. Evidence (verified by reading `oxur-cli` source, 2026-07-25)

Facts an oxur CDC can re-confirm in ~2 minutes (commands in the appendix):

1. **The heavy tail is all optional.** In `crates/oxur-cli/Cargo.toml`, every one of `oxur-lang`, `oxur-comp`, `oxur-repl`, `clap`, `reedline`, `nu-ansi-term`, `terminal_size`, `crossterm`, `dirs`, `tokio`, `async-trait`, `metrics`, `uuid` is `optional = true` and pulled **only** by the `binary` feature. The always-on deps are just `tabled`, `serde`, `toml`, `anyhow`, `colored`.

2. **`table` needs almost nothing.** External imports across `src/table/*.rs` are only `tabled` (`Table`/`Tabled`/`Builder`/`settings::*`), `colored::*`, and `serde::Deserialize`. **The warm-orange theming rides on `colored`'s `ColoredString`, not on an ANSI crate** — so `nu-ansi-term`/`crossterm` are not needed for it. `table`'s only internal reach is `super::config` = its own `table::config` submodule (`TableStyleConfig`, the TOML theme), **not** the crate-level REPL `config`. `table` is a self-contained island.

3. **`common` is a second island.** `src/common/{io,output,progress}.rs` import only `std` + `colored`. `common::output` (`success`/`error`/`info`/`warning`) is 4 thin wrappers over `colored` (~152 lines incl. tests). It does not touch `config` either.

4. **The blocker that kills `oxur-cli --no-default-features`.** `src/lib.rs` declares `pub mod config;` **ungated**, and `src/config/paths.rs` calls `dirs::config_dir()` / `dirs::home_dir()` / `dirs::data_dir()` / `dirs::cache_dir()` **unconditionally** — but `dirs` is `optional` and only in `binary`. So building `oxur-cli` with `--no-default-features` fails to resolve `dirs` in `config::paths`. (`config` is titled "Configuration module for the Oxur REPL CLI" — it is REPL surface that was simply never feature-gated.)

5. **In-repo consumers of `table`.** Only `oxur-odm` uses `oxur_cli::table::*` — `TableStyleConfig` and `table::helpers::{parse_row_bg_colors,state_to_fg_color,get_data_row_bg_color,apply_cell_color}` in `commands/{list,show,info}.rs`. Everything else that imports `oxur_cli` uses `common::*` / `config::*` (unaffected). `crates/oxur/src/lib.rs` does `pub use oxur_cli;` (umbrella; unaffected).

6. **Editions align.** oxur workspace is `edition = "2021"`, `version = "0.2.0"`; odm is edition 2021 / Rust 1.75+. `oxur-table` inherits 2021 and builds anywhere — no edition-2024 toolchain needed (unlike the `binary` path).

## 4. Options considered

| | Option | Upfront work | Coupling for odm | Verdict |
|--|--------|--------------|------------------|---------|
| **A** | odm depends on `oxur-cli`, `default-features = false` | **Non-zero** — must first gate `config` (and its `lib.rs` re-exports) behind `binary`/a feature so `--no-default-features` compiles (evidence #4); then odm still compiles all of `common`+`config`+`table` and takes a git/path dep on the whole `oxur-cli` | High — odm's build tracks `oxur-cli`'s REPL churn (reedline/metrics/async) and release cadence | Rejected |
| **B** | **Extract `oxur-table`; odm depends on it directly** | Moderate — a near-mechanical crate carve-out (`table` is already an island, evidence #2) | **Minimal** — odm's dep surface is `tabled`+`colored`+`serde`; `oxur-cli` never enters odm's graph; the `config`/`dirs` issue is moot for odm | **Chosen** |
| **C** | Vendor/copy `table` into odm | Low now | None, but **forks the theme** — drift between odm and oxur styling, no shared source of truth | Rejected (violates "single established pattern", the whole point of F-1) |

**Why B over A.** A's only advantage was "no upstream work" — and evidence #4 removes it (A also requires touching oxur). Once both touch oxur, B wins on every axis that matters long-term: odm gets a tiny, stable dependency; the theme has one home; and it **restores the original ODD-0001 / design-0015 intent** rather than entrenching the late-2025 fold. Since `table` is already self-contained, B's extra cost over A is small.

## 5. Consequences

**Positive.** One source of truth for Oxur table styling. odm gets themed output with a 3-dep footprint and zero exposure to the compiler/REPL stack. `oxur-table` becomes a clean, publishable, independently-versioned artifact (a good candidate to be the first oxur crate on crates.io). oxur-cli slims by one responsibility.

**Neutral / to manage.**
- **Back-compat:** `oxur-odm` must keep working unchanged. Achieved by having `oxur-cli` re-export the crate: `pub use oxur_table as table;` (so `oxur_cli::table::helpers::…` keeps resolving). No `oxur-odm` edits required in this change.
- **Feature unification (note for the future):** odm is its own workspace and will not enable `oxur-cli/binary`. But if some future odm dependency ever enables it in the same build graph, the heavy stack unifies back in. Keep odm's graph free of `oxur-cli/binary`.
- **Distribution choice (open, §7):** crates.io publish vs git dep for odm's consumption.

**Negative.** One more workspace member to maintain and (optionally) publish. Small, and it was the design all along.

## 6. Implementation plan

### 6a. oxur repo — the extraction (this is the oxur CDC's slice)

Near-mechanical. Suggested steps:

1. **Create the crate.** `crates/oxur-table/` with `src/lib.rs`. Move the four files:
   `git mv crates/oxur-cli/src/table/{mod.rs,config.rs,helpers.rs,themes.rs} crates/oxur-table/src/` and rename `mod.rs` → `lib.rs`.
   - The submodule cross-refs (`use super::config::…` in `themes.rs`/`helpers.rs`) still resolve: as top-level modules under `lib.rs`, `super::config` == `crate::config`. Expect **zero or near-zero** edits; let `cargo build` point at any stragglers.

2. **`crates/oxur-table/Cargo.toml`** (workspace-inherited versions):
   ```toml
   [package]
   name = "oxur-table"
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   authors.workspace = true
   repository.workspace = true
   description = "Themed terminal table rendering for Oxur CLIs (tabled + colored)"

   [dependencies]
   tabled  = { workspace = true }
   colored = { workspace = true }
   serde   = { workspace = true }   # derive; used by TableStyleConfig
   # toml  = { workspace = true }   # ADD IFF table::config deserializes TOML strings directly — compiler will tell you
   ```
   (`dev-dependencies`: whatever the moved `#[cfg(test)]` modules need — likely none beyond std.)

3. **Register** `crates/oxur-table` in the root `Cargo.toml` `[workspace].members`.

4. **Rewire `oxur-cli`** for back-compat:
   - Add dep: `oxur-table = { version = "0.2.0", path = "../oxur-table" }` (non-optional — `oxur-odm` uses it through `oxur-cli`).
   - In `src/lib.rs`, replace `pub mod table;` with `pub use oxur_table as table;`.
   - Optional cleanup: if nothing else in `oxur-cli` imports `tabled` directly (grep `src/` outside the moved files), drop `tabled` from `oxur-cli/Cargo.toml`.

5. **Build & verify** (real toolchain): `cargo build`; `cargo test -p oxur-table`; **`cargo test -p oxur-odm`** (the back-compat consumer — its `list`/`show`/`info` must render identically); `cargo test` (workspace); `cargo clippy --all-targets -- -D warnings`; `cargo fmt --check`; `cargo tree -p oxur-table` shows only `tabled`/`colored`/`serde`(+`toml`?).

**Acceptance (oxur side):** workspace green; `oxur-odm` output byte-identical to before; `oxur-table` builds standalone with a 3–4-crate dep tree; `oxur_cli::table::*` paths still resolve via the re-export.

**Optional, separate hygiene fix (not required by this ADR):** gate `oxur-cli`'s crate-level `config` (and its `lib.rs` re-exports) behind `binary`/a feature so `oxur-cli --no-default-features` compiles for any future lib-only consumer (evidence #4). Decoupled from the extraction — do it only if you want it.

### 6b. odm repo — consume it (odm RH chunk C-1; see the companion cc-prompt)

Add `oxur-table` to `odm-cli`, replace odm's raw `tabled` + `writeln!` rendering with `oxur_table::OxurTable` + `TableStyleConfig` (warm-orange default), re-run `odm self-host`, verify themed `odm list`. Full assignment in `cc-prompt-c1-adopt-oxur-table.md`.

## 7. Open questions (decide at oxur-session kickoff)

1. **Crate scope — table only, or table + terminal helpers?** Duncan's F-1 said "table/**terminal**". `common::{io,output,progress}` are *also* config-free islands (evidence #3), so the clean options are: **(i)** widen this crate (or add a sibling, e.g. `oxur-cli-support` / `oxur-term`) to hold `common` too, so odm gets `success/error/info/warning` from the same shared source; or **(ii)** keep this crate table-only and have odm reimplement the ~40 lines of output wrappers over `colored`. **Recommendation:** decide now; (i) is only marginally more work and prevents a second round-trip when odm wants themed status messages. If you take (i), name the crate for the wider scope rather than `oxur-table`.
2. **Distribution to odm:** publish `oxur-table` to crates.io (cleanest; odm pins a version), or odm takes a git dep pinned to a rev/tag of `oxur/oxur`? `oxur-table` is small and stable — a natural first crates.io publish. Default if undecided: git dep pinned to a tag, upgrade to crates.io when ready.
3. **`toml` dep:** confirm whether `table::config` deserializes TOML strings itself (needs `toml`) or only accepts already-parsed `TableStyleConfig` (needs only `serde`). One `cargo build` answers it.

## Appendix — verify the evidence yourself (~2 min, oxur repo root)

```bash
# 1. heavy deps are all binary-gated; always-on = tabled/serde/toml/anyhow/colored
sed -n '/\[features\]/,/^$/p' crates/oxur-cli/Cargo.toml

# 2. table's only external deps + it never reaches crate::config
grep -rhoE 'use (std|serde|toml|anyhow|colored|tabled)[^;]*' crates/oxur-cli/src/table | sort -u
grep -rn 'crate::config' crates/oxur-cli/src/table        # expect: nothing

# 3. the --no-default-features blocker: ungated config + unconditional dirs
grep -n 'pub mod config' crates/oxur-cli/src/lib.rs
grep -n 'dirs::' crates/oxur-cli/src/config/paths.rs
grep -n 'dirs ' crates/oxur-cli/Cargo.toml                # optional = true, binary-only

# 4. only oxur-odm consumes table
grep -rn 'oxur_cli::table' --include=*.rs crates | grep -v crates/oxur-cli/
```
