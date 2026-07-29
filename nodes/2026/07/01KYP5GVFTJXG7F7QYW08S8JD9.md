---
id: 01KYP5GVFTJXG7F7QYW08S8JD9
number: 523257600
type: artifact
schema: artifact/v1.1
name: 'Closing report — Slice 03 (Arc 05): `odm reconcile` (on demand)'
created: 2026-06-30
updated: 2026-06-30
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice03-reconcile-command/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKKA9T9G4T0GJVRG4H
---
# Closing report — Slice 03 (Arc 05): `odm reconcile` (on demand)

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc05-slice03-reconcile-command`, branched off
> `arc05-slice02-file-probe-and-runner` (slices 01–02 had not merged to `main`;
> the prompt's named fallback). Rebase onto `main` once 01/02 merge.

## Per-row walk

**R-1 — `odm reconcile` reports drift; clean corpus → no drift, exit 0 — done
(attested).** `Command::Reconcile { strict, json }` dispatches to
`reconcile::reconcile`, which invokes `Runner::run_corpus` (slice02 store-read)
and renders the `CorpusReport` to stdout. A holding fact and a factless corpus
both print `reconcile: no drift …` and exit `0` — no fabricated data, no file
written (`ROLLUP.md` is slice04). `reconcile_clean_reports_no_drift_exit_0` → ok.

**R-2 — drifted fact reported with identity + expected/observed, non-zero exit —
done (attested).** A drift renders `#<number> <name> / <fact_id>: <describe>`
plus `expected:` / `observed:` lines, and exits `1`.
`reconcile_drift_reported_nonzero` asserts all five identity/finding fields and
the exit code.

**R-3 — probe `Error` surfaced distinctly (Warning), `--strict`-gated — done
(attested).** A probe error renders an `[error]` entry with its `reason`,
distinct from `[drift]`; the header counts it as `couldn't-check`. Exit `0`
without `--strict`, `1` with. `reconcile_probe_error_surfaced_warning` +
`reconcile_strict_fails_on_probe_error` → ok. The slice02 `Error ≠ Drifted` split
now carries all the way to the exit code.

**R-4 — exit/severity is a pure fn of `OutcomeCounts` — done (attested).**
`fn verdict(&OutcomeCounts) -> Verdict { severity, exit, strict_exit }` is pure
(no I/O); the command is a thin wrapper. `reconcile_exit_severity_is_pure_fn_of_counts`
is a `#[cfg(test)]` table over clean / holds-only / drift / probe-error / both.

**R-5 — `--json` `reconcile/v1`, `Serialize` added additively — done
(attested).** `--json` emits `{schema:"reconcile/v1", ok, counts, nodes:[…]}`,
a 1:1 projection of the same `NodeView`s the human output uses. `ProbeOutcome`
gained `Serialize` internally tagged on `kind` (`holds`/`drifted`/`error`), plus
the runner report types — additive-stable, explicit always-serialized fields.
`reconcile_json_schema` + `reconcile_json_clean_is_ok` → ok; greps confirm the
marker and `Serialize`.

**R-6 — gates — done (attested).** clippy `-D warnings` → exit 0; no `unsafe`;
coverage (line) `odm-cli/src/reconcile.rs` 97.10%, `odm-reconcile` file.rs 97.01%
/ runner.rs 96.92% / shell.rs 100% — all ≥ 90. Full workspace `cargo test` green
(41 suites, no regression).

Rows: 6. Done: 6. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item of `slice-doc.md` shipped: the `odm reconcile` command
(invoking the slice02 `run_corpus`), human output (drift + probe-error with
identity, distinct severities, "no drift" when clean), `--json` `reconcile/v1`
with `Serialize` added additively, and the pure counts→severity/exit mapping
consistent with `check`. Every "out" item stayed out: no rollup/orient drift
wiring (slice04), no `--schedule` (slice07), no `affects`/deferred. **No new index
reader** — `run_corpus` reads the store; no `IndexRecord`/adapter/`FORMAT_VERSION`
change (invariant honored by non-triggering; verified by the slice02 G-5 grep
still clean).

## Deviations / decisions flagged

1. **Output uses `writeln!`, not `oxur_cli::common::output`.** The slice-doc (and
   CLAUDE.md) name `oxur_cli` output helpers + `table`. In fact `odm-cli` has **no
   `oxur-cli` dependency** — `check`/`rollup`/`orient` all render with plain
   `writeln!` (+ `tabled` for tables). Reconcile matches the *actual* in-tree
   convention (`writeln!`, mirroring `check`) rather than introducing a new
   dependency and an inconsistent idiom. Flagged, not silently diverged; if the
   project wants `oxur-cli` adopted, that is a separate, workspace-wide change.
2. **The probe-error exit default was implemented as recommended** (Warning,
   `--strict`-gated), not flipped to fail-by-default. No amendment.
3. **The command loads the corpus twice** — once inside `run_corpus` (to run the
   probes) and once to label the report with node number/name and fact
   `describe`. See the bubble-up: the slice02 report carries ids only, so a
   second store read is needed for the render identity. For an infrequent,
   I/O-bound-on-probes command this is negligible, but it is a real API seam worth
   addressing — flagged for slice04 (which renders the same drift).

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice03 deliver A-3?** Yes. A-3 ("slice03 — `odm reconcile` on demand —
closed") is discharged: drift is visible and actionable from the CLI, with
`check`-consistent severities/exit codes and a `reconcile/v1` JSON contract.
A-3 stays `attested` until CDC reproduces.

**2. What it reveals for slice04 (drift in rollup/orient).**
  - **The report-needs-render-identity seam.** `CorpusReport`/`NodeReport` carry
    `node_id`/`fact_id` only — to render `#number name` and a fact's `describe`,
    slice03 re-loads the store and joins. slice04 will render the *same* drift in
    the rollup/orient views and will hit the identical need. **Recommendation:**
    factor the join — either a small helper in `odm-reconcile` that enriches a
    report with identity from the loaded docs, or have slice04's rollup builder
    (which already holds the corpus) do the join once — so the logic is written
    once, not copied. Cheap now, avoids divergence later.
  - **The drift→rollup path is settled (arc-plan v1.5 / slice-doc):** rollup runs
    an on-demand corpus reconcile and folds the result in; it does **not** cache
    drift in the index. slice03 confirms the store-read `run_corpus` is the right
    entry point for that — the rollup builder can call the same runner. The A4
    invariant stays un-triggered.
  - **Reusable severity model.** The pure `verdict`/`OutcomeCounts` mapping is the
    natural input to the rollup's drift summary (counts → a "drift: N drifted, M
    couldn't-check" line, or "no drift" when clean). slice04 can reuse the counts
    rather than re-deriving severity.

**3. Wire-contract note.** `reconcile/v1` is now a versioned contract (a shape
test pins it). `ProbeOutcome`'s tagged `kind` representation is part of it —
evolve additively only (new variant/field, never rename/remove). If ODD-0013
§7.1 maintains a canonical schema list, `reconcile/v1` should be added there
(noted for the arc-close / a docs slice; no §7.1 code change this slice).

No row required an amendment; the ratified probe-error default fit cleanly. The
`writeln!`-vs-`oxur_cli` and double-load items above are flagged deviations, not
silent drops.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-3 row evidence + a v1.6 version-history entry), not only here.
