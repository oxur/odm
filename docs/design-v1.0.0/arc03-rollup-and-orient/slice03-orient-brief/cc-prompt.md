# CC Prompt — Slice 03 (Arc 03): orient / brief + bare-`odm`

Compose `odm orient` (alias `brief`) over the slice02 `Rollup` model so one cheap call
gives a fresh session full situational awareness — **vision → current focus →
ready/blocked → integrity → drift** — and make bare `odm` run it. This is the MVP's
headline affordance and its last capability slice (slice04 = `--json` + polish).

> **Start condition:** slice02 (rollup model) CDC-verified / CI-green. Also relies on
> arc01 slice05 (`use`/`context`) and slice01 (the `oxur-odm` `assert_cmd` suite). If
> slice02 isn't in, hold.

## Read first
1. `slice03-orient-brief/ledger.md` (11 rows).
2. `slice-doc.md` (same dir) and the **arc-plan** (`../arc-plan.md`) — D-1/D-1a
   (vision source + the settled extraction rule), Q-A3-2 (drift).
3. `../slice02-rollup-generation/cdc-verification.md` — rulings 2 (soft-sat ⚠ on the
   ready frontier) and 3 (surface `check` integrity inline).
4. The pieces to reuse: `crates/odm-core/src/rollup.rs` (`Rollup::assemble` +
   `ReadyNode.soft`), `crates/odm-cli/src/context.rs` (`Context::load`),
   `crates/odm-core/src/frontmatter.rs` (`Document::body()`), and the `check`
   aggregation in `crates/odm-cli/src/commands.rs` (for the integrity section).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `14-cli-tools/02-argument-parsing.md`,
  `14-cli-tools/03-error-handling.md`, `02-api-design.md`.
- `/collaboration-framework` → LEDGER_DISCIPLINE.

## Task

1. **`orient` composition** (`odm-cli`, e.g. `orient.rs`): load `Context`, load the
   corpus, `Rollup::assemble(...)` (reuse — **do not re-derive**), and render a *terse*
   orient view (not the full `ROLLUP.md`) in order: vision → current focus (current
   arc + status) → ready/blocked → integrity → drift.
2. **Vision** (D-1/D-1a): load the current project's `Document`; lead with its `name` +
   the vision section of `body()`. **Extraction rule:** `# Vision` heading
   (case-insensitive ATX) → that section (to the next same-or-higher heading); else the
   lead section (body before the first ATX heading); truncate to ~15 non-empty lines
   with `… (full vision: odm show <project>)` when cut. Make it a pure, tested helper.
3. **Ready/blocked** (slice02 ruling 2): surface the ready frontier with the soft-sat
   ⚠ from `ReadyNode.soft`; blocked nodes with named reasons.
4. **Integrity** (slice02 ruling 3): reuse the existing `check` aggregation and surface
   every **Error** (orphan, cycle-without-tear) inline, so a structural break is
   unmissable. Don't reimplement check.
5. **Drift**: "not yet tracked (A5)" (Q-A3-2).
6. **Bare `odm` orients**: no subcommand → `orient`; **never bare-errors**.
7. **No-current-project fallback** (all exit 0): none → affordance to `odm new
   project`; one → orient on it; many w/o context → list + prompt `odm use project`.
8. **`brief`** = alias of `orient` (identical output).

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Reuse, don't reimplement:** orient composes the slice02 model + the check
  aggregation; it does not re-derive the graph or re-walk integrity.
- **`--json` is OUT** (slice04 pins the schema). Human-readable output only.
- Bare `odm` and every no-project branch **exit 0** (errors-as-affordances, never
  bare-error). No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test -p odm-core -p odm-cli` + the `oxur-odm` `assert_cmd` row + clippy +
coverage; `ledger.md` evidence per row; `closing-report.md` (per-row walk for all 11,
What Worked, uncertainties named). Feature branch (`arc03-slice03-orient-brief`); not
`main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration
cap; your `done` is *proposed done* → CDC verifies (cargo rows via CI / local 1.85+).
On close, the MVP capability is feature-complete (slice04 closes the arc).
