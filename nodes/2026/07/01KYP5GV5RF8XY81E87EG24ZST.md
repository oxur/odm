---
id: 01KYP5GV5RF8XY81E87EG24ZST
number: 551612300
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 03 (Arc 05): `odm reconcile` (on demand)'
created: 2026-06-30
updated: 2026-06-30
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice03-reconcile-command/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKKA9T9G4T0GJVRG4H
---
# CC Prompt — Slice 03 (Arc 05): `odm reconcile` (on demand)

Third slice of A5. slice01 landed the model + `Probe` trait + shell probe; slice02 added the
`file` probe + the probe-runner (`Runner::run_corpus` → `CorpusReport` / `OutcomeCounts`,
store-read). Put a **command** on top: `odm reconcile` runs the corpus runner and reports
drift — human + `--json` (`reconcile/v1`), severities + exit codes consistent with `check`.
No rollup/orient wiring, no scheduling this slice (slices 04 / 07).

> **Start condition:** slices 01–02 CDC-verified (CI flips them to reproduced). Branch off
> `main`: **`arc05-slice03-reconcile-command`** (not `main`). If 01/02 are unmerged, branch
> off slice02's branch and flag for rebase (as slice02 did).

## Read first
1. `slice03-reconcile-command/ledger.md` (6 rows) + `slice-doc.md` (same dir) — especially
   **"The one design decision to ratify"** (probe-error exit semantics: `check`-consistent,
   Warning + `--strict`). Implement as recommended; if you disagree, amend with reasoning.
2. `../arc-plan.md` — A5 capability, Arc Ledger (this slice closes **A-3**), v1.5 (slice02
   close + the slice04 open question). slice03 does **not** touch rollup — that's slice04.
3. slice02's API: `crates/odm-reconcile/src/runner.rs` (`Runner`, `run_corpus`,
   `CorpusReport`, `NodeReport`, `FactResult`, `OutcomeCounts`) — you render these.
4. The A3 CLI patterns: `crates/odm-cli/` command dispatch + `crates/odm-cli/tests/`
   (e.g. `cli.rs`, `rollup.rs`); how `--json` + schema markers (`check/v1`, `rollup/v1`,
   `orient/v1`) and exit codes are done; `oxur_cli::common::output` + `oxur_cli::table`.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design + error handling.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc** at close; **propagate the bubble-up into `arc-plan.md` (A-3 row +
  version entry) yourself** — as you did for slice02).

## Task
1. **`odm reconcile` command** (R-1, R-2): invoke `Runner::run_corpus`, render the report to
   stdout. Clean corpus → "no drift" + exit 0 (no fabricated data). Drifted fact → identity
   (node number+name, `fact_id`, `describe`) + `expected`/`observed` + non-zero exit.
2. **Probe-error severity** (R-3): surface probe `Error` distinctly from drift (Warning);
   exit 0 without `--strict`, non-zero with `--strict`. (Mirrors `check`.)
3. **Exit/severity mapping as a pure fn of `OutcomeCounts`** (R-4): factor it so it's
   unit-testable without I/O (counts → severity, exit, exit-under-strict).
4. **`--json` `reconcile/v1`** (R-5): 1:1 projection of the report; add `Serialize` to
   `ProbeOutcome` + the report types (slice02 left it off — add it here, **additive-stable**:
   explicit, always-serialized fields, a versioned `reconcile/v1` marker like the A3 schemas).
5. **Gates** (R-6): clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If a row is wrong/impossible, or you disagree with the
  probe-error exit default, raise an amendment — don't quietly diverge.
- **Stay in scope.** No rollup/orient drift wiring (slice04), no `--schedule` (slice07), no
  `affects`/deferred. `odm reconcile` standalone only.
- **No new index reader.** Call `run_corpus` (store-read); make **no** `IndexRecord`/adapter/
  `FORMAT_VERSION` change. The A4 invariant stays honored by non-triggering (slice-doc).
- **`Error ≠ Drifted` carries to the exit code** — a probe-error and a drift must remain
  distinguishable in both output and exit semantics; don't collapse them.
- **`Serialize` is a wire contract** — once `reconcile/v1` ships, treat the shape as versioned
  (additive only); pin it in a test + (if A3 did) the relevant ODD schema section.

## Deliverables
The command + human/`--json` output + the exit/severity mapping, with `ledger.md` evidence
per row (`attested`); a `closing-report.md` — per-row walk **plus the Bubble-up to the arc**
(did slice03 deliver A-3; what it reveals for slice04's rollup-drift; the silent-drop diff)
— **and propagate that bubble-up into `arc-plan.md` (A-3 + a version entry)**. Feature branch
`arc05-slice03-reconcile-command`; not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-3) per LEDGER-DISCIPLINE v2.0 §A.
