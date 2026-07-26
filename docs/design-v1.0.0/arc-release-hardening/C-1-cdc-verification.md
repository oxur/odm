# C-1 — CDC verification (chunk close)

> **Chunk:** RH C-1 — adopt Oxur themed output in odm via `oxur-term` · **Verifies:** RH-1 ·
> **Covers finding:** F-1 · **Branch:** `rh-c1-adopt-oxur-term` · **Date:** 2026-07-26
> **Verifier:** CDC (odm session), independent of the implementation. **Environment note:** the
> CDC sandbox has no edition-2024 cargo, so cargo/executable rows are **attested-by-CC →
> reproduced-on-CI**; CDC reproduces *structural* rows by source + on-disk inspection, and the
> runtime render by a captured `odm list`.

## Verdict

**C-1 delivered, RH-1 `attested`.** The themed-output capability is in place and reproduced
visually; the remaining gap to `reproduced`/`done` is mechanical (commit + toolchain/CI green),
not design. One disclosed deviation, no defects, one new finding bubbled up (F-15).

## Per-criterion walk (against `cc-prompt-c1-adopt-oxur-term.md`)

| # | Acceptance criterion | Result | Strength | Evidence |
|---|----------------------|--------|----------|----------|
| 1 | **Deps hygiene:** `oxur-term` added; `oxur-cli` removed from odm | **PASS** | reproduced (inspection) | `odm/Cargo.toml` has only `oxur-term = { git…, tag = "0.2.1" }` (+ comment on why `oxur-cli` was shed); `odm-cli/Cargo.toml` has `oxur-term.workspace = true`, `tabled.workspace = true` kept only for the re-exported `Tabled` derive's absolute `::tabled::` paths (documented). |
| 2 | **Route-B invariant:** `oxur-cli` / compiler-REPL stack absent from odm's graph | **PASS (structural)** / *CC to confirm via `cargo tree`* | attested-pending | No `oxur-cli` in `Cargo.toml`; `oxur-term`'s deps are `tabled`/`colored`/`serde`/`toml`/`anyhow`. Ask CC to attach `cargo tree -p odm-cli`. |
| 3 | **Tables through `oxur_term`** (`commands.rs`, `migrate.rs`) | **PASS** | reproduced (inspection) | `odm-cli/src/table.rs` centralises rendering (`oxur_term::table::{Builder, TableStyleConfig}`; `TableStyleConfig::default().apply_to_table`); `commands.rs`/`migrate.rs` route through it. **No raw `tabled::{Table,settings::Style,builder}` left in `odm-cli/src`** (grep clean). |
| 4 | **Status lines through terminal helpers** | **PASS** | reproduced (inspection) | `odm-cli/src/term.rs` wraps `oxur_term::common::output`; `lib.rs` calls `oxur_term::common::output::error` directly. |
| 5 | **Themed `odm list` renders** (the F-1 ask) | **PASS** | reproduced (runtime) | 2026-07-26 capture: warm-orange header + rows, 59 nodes, on `rh-c1-adopt-oxur-term`. The "no colours" complaint is visibly resolved. |
| 6 | **Columns/content unchanged vs pre-C-1** (rendering-only) | **PASS** | reproduced (runtime) | Capture shows the same NUMBER/TYPE/NAME/ID columns, `odd` types, number-prefixed names — restructuring correctly deferred to C-2/C-3. |
| 7 | `cargo build` / `test` / `clippy -D warnings` / `fmt` green | **PENDING** | attested-by-CC → CI | Not runnable in the CDC sandbox. CC to attest on local 1.85+; CI reproduces. |
| 8 | `odm self-host` + `odm check` green after the swap | **PENDING** | attested-by-CC | `list` renders on the 59-node corpus (capture), but a `check`-green run is the row that closes it. |

## Disclosed deviation (not a defect)

The cc-prompt named the high-level `OxurTable::new(rows).render()` API. The implementation uses
the **lower-level** `Builder` + `TableStyleConfig::default().apply_to_table` path instead — the
same path `oxur-odm` used — because `OxurTable` does not yet expose **theme injection** or a
**text-carrying footer**, which odm's `list`/`migrate` output needs. `odm-cli/src/table.rs`
documents this and flags the upstream follow-up (`OxurTable::with_theme(…)` + a text footer would
let odm collapse back onto `OxurTable`). Disclosed, rationale recorded — **candidate `oxur-term`
enhancement ticket**, not a C-1 blocker.

## Bubble-up to the arc

- **F-1 dispositioned:** Route B (`oxur-term`) delivered; themed output landed as F-1 asked.
- **F-15 surfaced (Batch 2), not dropped:** `odm list` shows retired/superseded nodes
  undifferentiated (node 1605's L-2 tombstone reads as live A6 slice 05). Routed to **C-3**
  (default-exclude + `--all`; status marker via F-7). It is *not* a C-1 defect — C-1 is
  rendering-only and this predates it; the themed `list` merely made the corpus legible enough
  to notice.
- **Silent-drop diff (C-1 scope):** none. Delivered = themed rendering + terminal helpers + deps
  hygiene; explicitly out (C-2/C-3/C-4/C-5) untouched, as scoped.

## Remaining to flip RH-1 → `reproduced`/`done`

1. **Commit** the staged diff (currently uncommitted on `rh-c1-adopt-oxur-term`): `Cargo.toml`,
   `Cargo.lock`, `crates/odm-cli/{Cargo.toml, src/{table.rs (new), term.rs, commands.rs,
   migrate.rs, lib.rs, reconcile.rs, rollup.rs}}`, `CLAUDE.md`.
2. **CC attest** `cargo build`/`test`/`clippy -D warnings`/`fmt` green + attach `cargo tree -p odm-cli`.
3. **`odm check` green** on the corpus after the swap.
4. **CI green** → RH-1 `attested` → `reproduced`, `done`.
5. Then bubble to `arc-plan.md` (already recorded, v1.2) and proceed to C-2 (type taxonomy —
   foundational, gates C-3).
