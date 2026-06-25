# Closing report — Arc 03 / Slice 03: orient / brief + bare-`odm`

> CC implementation closing report. Status: **proposed done** → CDC verifies the
> cargo rows via CI / a local 1.85+ toolchain. Impl commit `f3d9f21`; docs commit
> (this report + ledger) follows.

## What this slice built

`odm orient` (alias `brief`) — the cheap, actionable read a fresh session leads
with (ODD-0015 §2: full situational awareness from one call). Where `odm rollup`
writes the full structural view, `orient` composes a **terse** view over the
*same* slice02 `Rollup` model plus the existing `check` aggregation, in order:
**vision → current focus → ready/blocked → integrity → drift**. Bare `odm` runs
it and never bare-errors. This is the MVP's last *capability* slice; slice04
(`--json` + polish) closes the arc.

Key reuse (no reimplementation): `Rollup::assemble` supplies the tree / ready /
blocked / status (D-3); `commands::integrity_findings` — a thin `pub(crate)`
wrapper over the existing `aggregate` — supplies the check findings (slice02
ruling 3); the rollup display atoms (`label` / `dep_label` / `status_inline`) are
shared. orient walks no graph and re-checks nothing.

## Per-row ledger walk (11 rows)

- **O-1 — done.** `orient` calls `Rollup::assemble` and renders the six sections
  in order; `orient_section_order` asserts strictly-increasing section offsets.
- **O-2 — done.** `extract_vision` is a pure helper: `# Vision` section
  (case-insensitive ATX, bounded by the next same-or-higher heading) else the
  lead section, trimmed and truncated to 15 non-empty lines with a
  `… (full vision: odm show <n>)` marker. Six unit tests
  (`vision_extraction_rule_*`).
- **O-3 — done.** Leads with `VISION  #<n> <name>` + the excerpt, not the whole
  body; `orient_leads_with_vision` also asserts the raw `# Vision` heading is not
  echoed.
- **O-4 — done.** Resolves project/arc from `Context`; the focus block shows the
  current arc with its gate-sequence status vector (`orient_uses_context`).
- **O-5 — done.** Ready frontier carries the soft-sat ⚠ from `ReadyNode.soft`;
  blocked nodes name reasons. Two tests cover the ready ⚠ and all three blocked
  reason kinds (unsatisfied / low-evidence / blocked-by).
- **O-6 — done.** Surfaces every `check` **Error** inline (the INTEGRITY block);
  `orient_surfaces_integrity` asserts an orphan appears with its node. Reuses
  `aggregate` via `integrity_findings`.
- **O-7 — done.** Drift reads "not yet tracked (A5)", consistent with the rollup
  (`orient_drift_placeholder`).
- **O-8 — done.** Bare `odm` (real binary, no args) exits 0 and prints the orient
  view; `bare_odm_orients` in the `oxur-odm` `assert_cmd` suite.
- **O-9 — done.** Three no-project branches, each exit 0: none → create
  affordance; one → orient on it; many → list + `odm use project` prompt
  (`orient_no_project_fallback`).
- **O-10 — done.** `brief` is a clap `visible_alias` of `orient` (same `Orient`
  variant); `brief_aliases_orient` asserts byte-identical output.
- **O-11 — done.** clippy `--all-targets --all-features --workspace -- -D warnings`
  → exit 0; no `unsafe` (grep → no matches); line coverage **odm-core 98.69%**,
  **odm-cli 93.44%** (orient.rs 97.78%), both ≥ 90%; `fmt --check` clean; full
  workspace `cargo test` → **218 passed, 0 failed**.

## Deviations from the slice doc (flagged, not buried)

None material. Two design choices worth surfacing:

1. **`orient` takes only `out` (no `err`).** Its entire output — including the
   no-project affordances — is the user-facing view on stdout, and it never
   bare-errors, so there are no diagnostics to route to stderr. (Other mutators
   take `err` for confirmations; orient is a pure query.) Flagging in case the CDC
   wants the affordances on stderr instead.
2. **Integrity surfaces *all* check Errors, not only orphan + cycle.** The slice
   doc says "at least every Error (orphan, cycle-without-tear)"; the
   implementation surfaces every `Severity::Error` finding (also dangling edges,
   self-supersede, supersession-cycle, decomposition-drift). This is a superset of
   the requirement — more is unmissable, not less — and reuses the existing
   severity classification rather than re-filtering by code.

## Uncertainties / things CDC should look at

- **Six rare/defensive lines in `orient.rs` remain uncovered** (named for
  honesty): the arc-without-gate-set focus arm (`125` — every configured type has
  a gate-set in practice), the empty-ready placeholder (`138`–`139` — a corpus
  with a project always has at least that project ready), and a
  write/`find_in_tree` recursion arm (`206`, `208`, `238`). orient.rs is otherwise
  97.78% line; the crate is 93.44%.
- **Current focus shows only the arc, not the project-level status.** Per the
  slice doc (focus = "the current arc + its status vector"). If no arc is
  selected, the block prints a `odm use arc <ref>` hint. Flag if the focus should
  also summarise the project's own gate status.
- **Vision `odm show <n>` marker uses the project number.** `odm show` accepts
  id | number | name-prefix, so the number is a valid, stable ref. Flag if a name
  or full id would read better.

## Iterations

One. Two minor in-slice corrections before the first close: a test expectation
fixed (a level-2 heading under `# Vision` is a *subsection*, correctly kept — the
rule, not the code, and the test was wrong), and ULID seed tags moved off the
Crockford-excluded letters (I/L/O/U). One coverage-driven test added for the
blocked reason variants.

On close, the MVP capability is **feature-complete**: `odm orient` gives full
situational awareness from one call and bare `odm` orients. slice04 (`--json` +
polish) closes the arc.
