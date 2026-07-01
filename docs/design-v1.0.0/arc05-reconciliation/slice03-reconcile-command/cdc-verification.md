# CDC Verification — Arc 05 / Slice 03: `odm reconcile` (on demand)

> Independent verification of CC's closed ledger (impl + close on
> `arc05-slice03-reconcile-command`, commits `266fc38` + `00cdcce`), per
> LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo rows
> route to CI / a local 1.85+ run.

## Environment constraint (disclosed)

CDC sandbox has no 1.85+ toolchain; CC built + ran on local 1.95.0 (attested → reproduced on
CI). Branch cut off `arc05-slice02-…` (01/02 unmerged) — **rebase onto `main` when 01/02
merge** (tracked).

## Row dispositions

**Row count:** 6 opened, 6 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **R-1** — `odm-cli/src/reconcile.rs` invokes `Runner::run_corpus`; clean corpus / no facts
  → `reconcile: no drift …` + exit 0 (`reconcile.rs:111–113`), no file written.
  `reconcile_clean_reports_no_drift_exit_0` present. ✔ (see ruling 1 on the render path)
- **R-2** — `reconcile_drift_reported_nonzero`: a drifted fact prints `#number name` /
  `fact_id` / `describe` + `expected:`/`observed:`, exit 1. ✔
- **R-3** — `reconcile_probe_error_surfaced_warning` (probe-error → `[error]` + `reason`,
  Warning, exit 0) + `reconcile_strict_fails_on_probe_error` (`--strict` → exit 1). The
  recommended `--strict`-gated default was implemented, not flipped. ✔
- **R-4** — `fn verdict(&OutcomeCounts) -> Verdict` (`reconcile.rs:49`) is a pure function;
  `reconcile_exit_severity_is_pure_fn_of_counts` is a no-I/O table:
  clean→(0,0), drift→error(1,1), probe-error→warning(0,1), **drift dominates when both**.
  Exactly the mapping the slice-doc specified. ✔
- **R-5** — `RECONCILE_SCHEMA = "reconcile/v1"` (`reconcile.rs:22`); `--json` envelope
  `{schema, ok, counts, nodes[...]}`; `Serialize` added to `ProbeOutcome` (internally tagged
  on `kind`, additive) + `FactResult`/`NodeReport`/`CorpusReport`/`OutcomeCounts`
  (`runner.rs`, `probe.rs`). `reconcile_json_schema` + `reconcile_json_clean_is_ok`. ✔
- **Invariant (no row)** — `grep index_frontmatters|IndexRecord|FORMAT_VERSION` in
  `odm-reconcile/src` → none; reconcile reads via `run_corpus` (store). No new index reader;
  A4 invariant stays un-triggered. ✔
- **R-6 (no `unsafe`)** — grep empty in `reconcile.rs`. ✔

**Attested by CC (local 1.95.0), pending CI:** clippy `-D warnings` → 0; line coverage
**reconcile.rs 97.1%**; 41 suites green. → **PENDING CI.**

## Rulings on CC's flagged decisions

1. **Render via `writeln!` + `tabled`, not `oxur_cli` — CC was right, and it exposes a
   doc-drift I propagated.** My slice-doc (R-1/R-2) said "use `oxur_cli::common::output` +
   `oxur_cli::table`," carried from the odm `CLAUDE.md` convention. **The code disagrees with
   both:** `crates/odm-cli/Cargo.toml` has **no `oxur-cli` dependency** — it uses
   `tabled.workspace` directly, and `check`/`rollup`/`orient` all render with
   `Table::new(...).with(Style::sharp())` + `writeln!` (`commands.rs:313`). CC matched the
   **real in-tree convention** instead of introducing an unused dependency — correct call,
   correctly flagged (ledger R-1, closing-report, arc-plan). **Finding (spec-drift):** the
   odm `CLAUDE.md` "CLI output uses `oxur_cli::common::output`/`oxur_cli::table`" line is
   **stale/aspirational** — no crate depends on `oxur-cli`. → Recommend reconciling `CLAUDE.md`
   (drop the claim, or adopt oxur-cli deliberately in a dedicated slice); carried to the
   **A5 arc-close** as a doc-keeping item. *My planning error, owned:* I should stop
   repeating the `oxur_cli` guidance in slice-docs — the convention is `writeln!` + `tabled`.
2. **Double corpus load (a flagged API seam).** `run_corpus` loads docs to run the probes,
   then reconcile does a second `load_all` to join `number`/`name`/`describe` (the slice02
   report carries **ids only**). **Accepted** — negligible for this I/O-bound command, and
   honestly flagged with the right fix deferred to slice04: enrich the runner's report (or a
   shared join helper) so identity is carried once rather than re-loaded. This is a real
   seam, disclosed not buried; slice04 renders the same drift and should factor the join.
3. **One enriched `NodeView` feeds both human + `--json`.** Accepted — the D-3 single-source
   ethos (the two renderings cannot drift), consistent with A3's `--json` projections.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-3: `odm reconcile` renders drift with `check`-consistent
  severities/exit codes + `reconcile/v1` JSON. Drift is now *actionable* from the CLI.
- **Silent-drop diff honest?** ✔ — 6/6; the render-path deviation and the double-load seam
  are disclosed.
- **Findings + arc-plan?** ✔ — CC propagated the bubble-up itself (A-3 row + v1.6), including
  the slice04 render-identity seam, the settled store-read drift→rollup path, and a note that
  `reconcile/v1` should join ODD-0013 §7.1's schema list at arc-close. Plus the new
  doc-drift finding (ruling 1) for the arc-close doc-keeping list.

## Verdict

**Arc 05 / Slice 03 CDC-verified on structure; all flags ruled; cargo rows pending CI.**
`odm reconcile` is the user-facing face of the drift killer: `check`-consistent severities
via a pure `verdict(OutcomeCounts)`, a versioned `reconcile/v1` JSON contract, and the
`Error ≠ Drifted` distinction preserved all the way to the exit code. Two items carried to
slice04 (factor the render-identity join once) and the arc-close (reconcile `CLAUDE.md`'s
stale `oxur-cli` claim; add `reconcile/v1` to ODD-0013 §7.1). A-3 attested-on-close; flips to
`done` on CI green. **Half the arc is in (3/7); slice04 — drift in rollup/orient — is next,
and its store-read/no-index-cache resolution is already settled (arc-plan v1.5/v1.6).**

CDC: planning thread, 2026-06-30. Iterations used: 1.
