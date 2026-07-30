//! `odm migrate` — import a legacy ODD corpus into the node model (Arc 06
//! slice01). The mapping + safety live in the `odm-migrate` crate; this module
//! resolves the gate-set, invokes it, and renders the [`MigrationReport`] as an
//! Oxur-themed table ([`oxur_term::table::OxurTable`], RH C-1).

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_migrate::coverage::{CoverageReport, DocClass};
use odm_migrate::mapping::{
    CanonicalizeReport, DocGates, ReconcileSourceReport, backfill_source, canonical_design_gates,
    canonical_research_gates, canonicalize_source_paths, reconcile_source,
};
use odm_migrate::synthesis::{Attestation, apply_project_vision};
use odm_migrate::{Created, MigrationReport, Mode, SelfHostReport};
use odm_store::{Store, StoreHome};

use crate::commands;
use crate::table::Themed;
use crate::term;

/// Runs `odm migrate <legacy-path> [--dry-run]`: reads the legacy corpus, imports
/// it into `store`, and renders the plan/result. The report (the answer) goes to
/// `out`; a one-line status goes to `err`.
///
/// # Errors
///
/// Returns an error (exit code `2`) if the gate config cannot be loaded or the
/// migration hits a store I/O failure. Per-document problems are reported in the
/// table, not raised.
pub(crate) struct Options {
    /// Force a derivation instead of detecting it (F-14).
    pub forced: Option<odm_migrate::Corpus>,
    /// Re-derive the existing plan nodes in place (C-5).
    pub replan: bool,
    /// Report and write nothing.
    pub dry_run: bool,
    /// Report the coverage/gap inventory over `legacy_path` (a docs root) and
    /// exit — read-only (arc-migration-fidelity slice01).
    pub coverage: bool,
    /// Mint the artifact-family corpus over `legacy_path` (a docs root) and
    /// exit (arc-migration-fidelity slice09/slice10).
    pub artifacts: bool,
    /// Mint the note corpus over `legacy_path` (a dev-docs root) and exit
    /// (arc-migration-fidelity slice10).
    pub notes: bool,
    /// Re-cast the project node as the vision synthesis and exit
    /// (arc-migration-fidelity slice12/slice13).
    pub vision: bool,
}

pub(crate) fn migrate(
    store: &Store,
    root: &Path,
    legacy_path: &str,
    options: Options,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let dry_run = options.dry_run;
    let legacy = resolve(root, legacy_path);

    // Coverage is a report, not a derivation: it never creates, upgrades, or
    // re-derives a node, so it short-circuits before any of that machinery.
    if options.coverage {
        return coverage(store, &legacy, out, err);
    }
    // Artifact/note mint-all are their own derivations — a different node
    // family, over a different (usually broader) root than the plan-set or
    // legacy-ODD walks below — so each short-circuits the same way `coverage`
    // does.
    if options.artifacts {
        return artifacts(store, &legacy, dry_run, out, err);
    }
    if options.notes {
        return notes(store, &legacy, dry_run, out, err);
    }
    if options.vision {
        return vision(store, &legacy, dry_run, out, err);
    }

    // The re-stamp is a different operation from an import: it rewrites nodes
    // that already exist rather than creating any, so it short-circuits here.
    if options.replan {
        return replan(store, root, &legacy, dry_run, out, err);
    }

    // One verb, two derivations — the shape decides unless told otherwise.
    let corpus = options.forced.unwrap_or_else(|| odm_migrate::detect_corpus(&legacy));
    if corpus == odm_migrate::Corpus::Plan {
        return self_host_inner(store, &legacy, dry_run, out, err);
    }
    let (gates, _) = commands::load_gate_config(root)?;
    // A repo may configure either document gate-set; each falls back to its
    // canonical sequence independently (C-2: `research` mirrors `design` today,
    // but nothing here assumes it always will).
    let doc_gates = DocGates {
        design: gates.for_type(NodeType::Design).cloned().unwrap_or_else(canonical_design_gates),
        research: gates
            .for_type(NodeType::Research)
            .cloned()
            .unwrap_or_else(canonical_research_gates),
    };

    let mode = Mode::from_dry_run(dry_run);

    // The design/research counterpart to `self_host_inner`'s repair-then-import
    // order (arc-migration-fidelity s10, gate wiring — no new detection logic,
    // `backfill_source` is s09's): reconcile every existing sourceless
    // design/research node against its legacy file **first**, then the
    // ordinary create pass below picks up anything genuinely new (an
    // as-yet-unmigrated ODD). Both steps already honor `--dry-run` identically.
    let backfill_report = backfill_source(store, &legacy, mode)
        .with_context(|| format!("backfilling source onto {}", legacy.display()))?;
    render_backfill(&backfill_report, out)?;

    // `backfill_source`'s complement (arc-migration-fidelity s12): a node
    // that already carries a `source` is invisible to the backfill pass
    // above. This re-establishes fidelity for it — re-discovering a moved
    // legacy file by number and rewriting `source.paths`, and/or
    // re-snapshotting a body that no longer matches the current legacy
    // content — the migration-fidelity reconcile (not the unrelated
    // `odm reconcile` / `odm-reconcile` desired-facts command).
    let reconcile_report = reconcile_source(store, &legacy, mode)
        .with_context(|| format!("reconciling source for {}", legacy.display()))?;
    render_reconcile_source(&reconcile_report, out)?;

    // `backfill_source`'s complement (arc-migration-fidelity s10 iteration 1):
    // a node that already carries a `source` — however it got one — is
    // invisible to the backfill pass above; this canonicalizes its path form
    // in place if it isn't already repo-content-root-relative. Runs after
    // `reconcile_source` above: that pass already writes canonical paths for
    // any node it touches, so this only ever has leftover pure-path-form
    // cases (an absolute-but-still-resolving path) to canonicalize.
    let canonicalize_report = canonicalize_source_paths(store, &legacy, mode)
        .with_context(|| format!("canonicalizing source.paths under {}", legacy.display()))?;
    render_canonicalize(&canonicalize_report, out)?;

    let report = odm_migrate::migrate_with_gates(store, &legacy, mode, &doc_gates)
        .with_context(|| format!("migrating the legacy corpus at {}", legacy.display()))?;

    render(&report, out)?;
    let status = format!(
        "{}: {} reconciled, {} re-snapshotted, {} source-reconciled, {} path(s) canonicalized, \
         {} created, {} upgraded, {} skipped, {} warning(s){}",
        if report.dry_run { "migrate (dry-run)" } else { "migrate" },
        backfill_report.repaired_count(),
        backfill_report.reconciled_count(),
        reconcile_report.reconciled_count(),
        canonicalize_report.rewritten_count(),
        report.created_count(),
        report.upgraded_count(),
        report.skipped_count(),
        report.warnings.len(),
        if report.dry_run { " — nothing written" } else { "" },
    );
    // A dry run planned; a real run changed the store.
    if report.dry_run {
        term::info(err, &status)?
    } else {
        term::success(err, &status)?
    }
    Ok(())
}

/// The plan-set derivation: **reconciles, then imports** a `design-vX.Y.Z/`
/// plan set (project + arcs + slices) as work nodes.
///
/// Reached only through `migrate` now. `self-host` was folded into it by RH C-5
/// (F-14) and the surviving top-level spelling was removed by C-4, per ODD-0023
/// §5 — one verb, whose derivation the tree's shape selects.
///
/// Runs the full migration-fidelity flow (arc-migration-fidelity s06 F-2/F-3,
/// corrected s07 F-1 — CDC v2.5 finding): [`odm_migrate::selfhost::repair`]
/// **first** — repairs stub bodies and gated-backfills `source` onto every
/// existing faithful node (the project excluded, ODD-0025 §2.3) — **then**
/// [`odm_migrate::selfhost::self_host`] (import the missing arcs/slices).
///
/// The order is **not** load-bearing for *correctness*: both `repair` and
/// `self_host`'s own coordinate→source transition route through the same
/// [`odm_migrate::selfhost::reconcile_source`] gate (s06's two-path fix), so a
/// coordinate-matched node reconciles identically whichever of the two calls
/// reaches it first — `self_host` alone (no `repair` call) converges on the
/// same final store state. Running `repair` first is a **composability +
/// report-attribution** choice: it gives `repair` a real caller (previously
/// only tests exercised it, the v2.2 finding) and lets the rendered report
/// name what was reconciled versus imported, in that order, for a human
/// reading the output.
///
/// Nothing here depends on that order for safety — it just reads more
/// clearly.
///
/// **No new flag**: both steps run unconditionally as part of the self-host
/// path (a deliberate default-on choice — the command inventory documents no
/// `--repair`/`--reconcile` opt-in, and `repair` is idempotent + gated + safe,
/// so gating it behind a flag would only add a foot-gun: a plain `odm migrate`
/// that silently skips reconciliation). Both steps already honor `--dry-run`
/// identically (preview counts, write nothing).
fn self_host_inner(
    store: &Store,
    plan_root: &Path,
    dry_run: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let plan_root = plan_root.to_path_buf();
    let mode = Mode::from_dry_run(dry_run);

    let repair_report = odm_migrate::selfhost::repair(store, &plan_root, mode)
        .with_context(|| format!("reconciling the plan set at {}", plan_root.display()))?;
    // `repair`'s complement (arc-migration-fidelity s12): a work node that
    // already carries a `source` — however faithful it was at import time —
    // is invisible to `repair` (which only ever touches sourceless nodes).
    // This re-snapshots it if its source has since moved or its body has
    // drifted (ODD-0025 §2.9's living-plan-node policy — no special
    // exclusion for a still-changing source).
    let reconcile_report =
        odm_migrate::selfhost::reconcile(store, &plan_root, mode).with_context(|| {
            format!("reconciling drifted plan-set sources at {}", plan_root.display())
        })?;
    let report = odm_migrate::selfhost::self_host(store, &plan_root, mode)
        .with_context(|| format!("self-hosting the plan set at {}", plan_root.display()))?;

    render_repair(&repair_report, out)?;
    render_reconcile(&reconcile_report, out)?;
    render_self_host(&report, out)?;
    let status = format!(
        "{}: {} reconciled, {} source-reconciled, {} created, {} skipped{}",
        if report.dry_run { "self-host (dry-run)" } else { "self-host" },
        repair_report.repaired_count(),
        reconcile_report.reconciled_count(),
        report.created_count(),
        report.skipped_count(),
        if report.dry_run { " — nothing written" } else { "" },
    );
    if report.dry_run {
        term::info(err, &status)?
    } else {
        term::success(err, &status)?
    }
    Ok(())
}

/// The reconcile (repair/backfill) table's columns.
const RECONCILE_COLUMNS: [&str; 4] = ["ACTION", "#", "NAME", "ID"];

/// Renders the reconcile pass: every existing node `repair` touched (a stub
/// body replaced, or a faithful node gated-backfilled with `source`).
/// Silent when there was nothing to reconcile — a fresh store's first
/// self-host run has no existing nodes, so this table would otherwise render
/// empty on every ordinary run.
fn render_repair(
    report: &odm_migrate::selfhost::RepairReport,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    if report.repaired.is_empty() {
        return Ok(());
    }
    let verb = if report.dry_run { "would reconcile" } else { "reconciled" };
    let title = if report.dry_run { "RECONCILE (DRY RUN)" } else { "RECONCILE" };
    let mut table = Themed::new(title, &RECONCILE_COLUMNS);
    for r in &report.repaired {
        table.row([verb.to_string(), r.number.to_string(), r.name.clone(), r.id.to_string()]);
    }
    table.summary(format!(
        "Total: {} {}",
        report.repaired_count(),
        if report.dry_run { "to reconcile" } else { "reconciled" }
    ));
    writeln!(out, "{}", table.render())?;
    Ok(())
}

/// Renders the design/research `source`-backfill pass
/// ([`odm_migrate::mapping::backfill_source`]): a `backfilled` row per
/// sourceless node whose stub body was replaced outright, plus a
/// `reconciled` row per sourceless node whose non-stub body no longer
/// matched its current legacy source and was re-snapshotted to it
/// (arc-migration-fidelity s12 F-1 — before s12 this second case was a
/// `drift (skipped)` row; see [`odm_migrate::mapping::BackfillReport`]'s
/// field docs). Silent when there is nothing to report.
fn render_backfill(
    report: &odm_migrate::mapping::BackfillReport,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    if report.repaired.is_empty() && report.reconciled.is_empty() {
        return Ok(());
    }
    let backfilled_verb = if report.dry_run { "would backfill" } else { "backfilled" };
    let reconciled_verb = if report.dry_run { "would reconcile" } else { "reconciled" };
    let title = if report.dry_run { "BACKFILL (DRY RUN)" } else { "BACKFILL" };
    let mut table = Themed::new(title, &RECONCILE_COLUMNS);
    for r in &report.repaired {
        table.row([
            backfilled_verb.to_string(),
            r.number.to_string(),
            r.name.clone(),
            r.id.to_string(),
        ]);
    }
    for r in &report.reconciled {
        table.row([
            reconciled_verb.to_string(),
            r.number.to_string(),
            r.name.clone(),
            r.id.to_string(),
        ]);
    }
    table.summary(format!(
        "Total: {} {}, {} {}",
        report.repaired_count(),
        if report.dry_run { "to backfill" } else { "backfilled" },
        report.reconciled_count(),
        if report.dry_run { "to reconcile" } else { "reconciled" },
    ));
    writeln!(out, "{}", table.render())?;
    Ok(())
}

/// Renders [`odm_migrate::mapping::reconcile_source`]'s pass over
/// already-sourced design/research nodes (arc-migration-fidelity s12 F-1/
/// F-2): a row per node whose stored path was re-discovered by number
/// (moved), body was re-snapshotted (drifted), or both. Silent when there is
/// nothing to report.
fn render_reconcile_source(
    report: &ReconcileSourceReport,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    if report.reconciled.is_empty() {
        return Ok(());
    }
    let verb = if report.dry_run { "would reconcile" } else { "reconciled" };
    let title = if report.dry_run { "SOURCE RECONCILE (DRY RUN)" } else { "SOURCE RECONCILE" };
    let mut table = Themed::new(title, &RECONCILE_COLUMNS);
    for r in &report.reconciled {
        let note = match (r.path_moved, r.body_drifted) {
            (true, true) => "moved+drifted",
            (true, false) => "moved",
            (false, true) => "drifted",
            (false, false) => "unchanged", // unreachable in practice (F-6's no-op skips it)
        };
        table.row([
            format!("{verb} ({note})"),
            r.number.to_string(),
            r.name.clone(),
            r.id.to_string(),
        ]);
    }
    table.summary(format!(
        "Total: {} {}",
        report.reconciled_count(),
        if report.dry_run { "to reconcile" } else { "reconciled" },
    ));
    writeln!(out, "{}", table.render())?;
    Ok(())
}

/// Renders [`odm_migrate::selfhost::reconcile`]'s pass over already-sourced
/// `arc`/`slice` nodes (arc-migration-fidelity s12 F-1/F-2): a row per node
/// whose plan-set source was re-discovered by `(type, number)` (moved), body
/// was re-snapshotted (drifted), or both. Silent when there is nothing to
/// report.
fn render_reconcile(
    report: &odm_migrate::selfhost::ReconcileReport,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    if report.reconciled.is_empty() {
        return Ok(());
    }
    let verb = if report.dry_run { "would reconcile" } else { "reconciled" };
    let title = if report.dry_run { "SOURCE RECONCILE (DRY RUN)" } else { "SOURCE RECONCILE" };
    let mut table = Themed::new(title, &RECONCILE_COLUMNS);
    for r in &report.reconciled {
        let note = match (r.path_moved, r.body_drifted) {
            (true, true) => "moved+drifted",
            (true, false) => "moved",
            (false, true) => "drifted",
            (false, false) => "unchanged", // unreachable in practice (F-6's no-op skips it)
        };
        table.row([
            format!("{verb} ({note})"),
            r.number.to_string(),
            r.name.clone(),
            r.id.to_string(),
        ]);
    }
    table.summary(format!(
        "Total: {} {}",
        report.reconciled_count(),
        if report.dry_run { "to reconcile" } else { "reconciled" },
    ));
    writeln!(out, "{}", table.render())?;
    Ok(())
}

/// Renders [`odm_migrate::mapping::canonicalize_source_paths`]'s pass: a
/// reconcile row per node whose `source.paths` form was rewritten to
/// repo-content-root-relative (arc-migration-fidelity s10 iteration 1).
/// Silent when there is nothing to report.
fn render_canonicalize(report: &CanonicalizeReport, out: &mut dyn Write) -> anyhow::Result<()> {
    if report.rewritten.is_empty() {
        return Ok(());
    }
    let verb = if report.dry_run { "would canonicalize" } else { "canonicalized" };
    let title = if report.dry_run { "CANONICALIZE (DRY RUN)" } else { "CANONICALIZE" };
    let mut table = Themed::new(title, &RECONCILE_COLUMNS);
    for r in &report.rewritten {
        table.row([verb.to_string(), r.number.to_string(), r.name.clone(), r.id.to_string()]);
    }
    table.summary(format!(
        "Total: {} {}",
        report.rewritten_count(),
        if report.dry_run { "to canonicalize" } else { "canonicalized" }
    ));
    writeln!(out, "{}", table.render())?;
    Ok(())
}

/// The `self-host` cutover table's columns.
const SELF_HOST_COLUMNS: [&str; 4] = ["ACTION", "#", "NAME", "ID"];

/// Renders the self-host cutover report: a create/skip table of the work nodes.
fn render_self_host(report: &SelfHostReport, out: &mut dyn Write) -> anyhow::Result<()> {
    if report.created.is_empty() && report.skipped.is_empty() {
        writeln!(out, "self-host: no plan-set nodes found.")?;
        return Ok(());
    }
    let verb = if report.dry_run { "would create" } else { "created" };
    let title = if report.dry_run { "SELF-HOST (DRY RUN)" } else { "SELF-HOST" };
    let mut table = Themed::new(title, &SELF_HOST_COLUMNS);
    for c in &report.created {
        table.row([verb.to_string(), c.number.to_string(), c.name.clone(), c.id.to_string()]);
    }
    for s in &report.skipped {
        table.row([
            "skip".to_string(),
            s.number.map_or_else(|| "—".to_string(), |n| n.to_string()),
            "(already exists)".to_string(),
            s.reason.to_string(),
        ]);
    }
    table.summary(format!(
        "Total: {} {}, {} skipped",
        report.created_count(),
        if report.dry_run { "to create" } else { "created" },
        report.skipped_count()
    ));
    writeln!(out, "{}", table.render())?;
    Ok(())
}

/// The `migrate --coverage` arm: runs the read-only coverage/gap detectors
/// (arc-migration-fidelity slice01) over `docs_root` and prints the report.
///
/// The report is Markdown, not a themed table — it is meant to be captured
/// verbatim as the arc's `coverage-report.md` work-list (`odm migrate <docs>
/// --coverage > coverage-report.md`), and a 200+ row inventory reads better as
/// grouped Markdown than as one giant table.
fn coverage(
    store: &Store,
    docs_root: &Path,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let report = odm_migrate::coverage::run(store, docs_root)
        .with_context(|| format!("running coverage over {}", docs_root.display()))?;

    render_coverage(&report, out)?;

    let status = format!(
        "coverage: {}/{} docs covered, {} arc dir(s) + {} slice dir(s) unrepresented, \
         {} stub bod{}, {} node(s) missing provenance",
        report.doc_coverage.covered_count(),
        report.doc_coverage.total(),
        report.representation.missing_arcs.len(),
        report.representation.missing_slices.len(),
        report.stubs.len(),
        if report.stubs.len() == 1 { "y" } else { "ies" },
        report.provenance_missing.len(),
    );
    term::info(err, &status)?;
    Ok(())
}

/// Renders the coverage report as Markdown: a summary, then the four detectors
/// in turn (doc-coverage grouped by class, representation, stub bodies,
/// provenance-absence).
fn render_coverage(report: &CoverageReport, out: &mut dyn Write) -> anyhow::Result<()> {
    let dc = &report.doc_coverage;
    let rep = &report.representation;

    writeln!(out, "# Coverage report\n")?;
    writeln!(out, "## Summary\n")?;
    writeln!(
        out,
        "- Source docs: {} total, {} covered, {} uncovered",
        dc.total(),
        dc.covered_count(),
        dc.uncovered_count()
    )?;
    writeln!(
        out,
        "- Representation: {}/{} arc dirs, {}/{} slice dirs",
        rep.represented_arc_dirs(),
        rep.total_arc_dirs,
        rep.represented_slice_dirs(),
        rep.total_slice_dirs
    )?;
    writeln!(out, "- Stub bodies: {}", report.stubs.len())?;
    writeln!(out, "- Provenance absent: {}\n", report.provenance_missing.len())?;

    writeln!(out, "## 1. Doc coverage — uncovered by class\n")?;
    for class in DocClass::all() {
        let rows: Vec<_> = dc.uncovered().filter(|e| e.class == class).collect();
        if rows.is_empty() {
            continue;
        }
        writeln!(out, "### {} ({})\n", class.as_str(), rows.len())?;
        for entry in rows {
            writeln!(out, "- `{}` — {}", entry.path.display(), entry.basis)?;
        }
        writeln!(out)?;
    }

    writeln!(out, "## 2. Representation gap\n")?;
    writeln!(out, "### Missing arc directories ({})\n", rep.missing_arcs.len())?;
    for name in &rep.missing_arcs {
        writeln!(out, "- `{name}`")?;
    }
    writeln!(out, "\n### Missing slice directories ({})\n", rep.missing_slices.len())?;
    for path in &rep.missing_slices {
        writeln!(out, "- `{path}`")?;
    }

    writeln!(out, "\n## 3. Stub bodies ({})\n", report.stubs.len())?;
    for stub in &report.stubs {
        writeln!(out, "- #{} {} — {}", stub.number, stub.node_type.as_str(), stub.name)?;
    }

    writeln!(out, "\n## 4. Provenance absence ({})\n", report.provenance_missing.len())?;
    for entry in &report.provenance_missing {
        writeln!(out, "- #{} {} — {}", entry.number, entry.node_type.as_str(), entry.name)?;
    }
    Ok(())
}

/// The `migrate --artifacts` arm: mints an `artifact` node for every
/// supporting doc under `docs_root` not already covered (arc-migration-
/// fidelity slice09/slice10, ODD-0025 §2.6 mint-all).
fn artifacts(
    store: &Store,
    docs_root: &Path,
    dry_run: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mode = Mode::from_dry_run(dry_run);
    let report = odm_migrate::artifact::mint_artifacts(store, docs_root, mode)
        .with_context(|| format!("minting artifacts over {}", docs_root.display()))?;

    render_artifacts(&report, out)?;
    let status = format!(
        "{}: {} artifact(s) minted{}",
        if report.dry_run { "migrate --artifacts (dry-run)" } else { "migrate --artifacts" },
        report.minted_count(),
        if report.dry_run { " — nothing written" } else { "" },
    );
    if report.dry_run {
        term::info(err, &status)?
    } else {
        term::success(err, &status)?
    }
    Ok(())
}

const ARTIFACT_COLUMNS: [&str; 4] = ["ACTION", "PATH", "CONTAINED BY", "ID"];

/// Renders the artifact mint-all report: one row per minted (or, dry-run,
/// would-mint) artifact.
fn render_artifacts(
    report: &odm_migrate::artifact::ArtifactReport,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    if report.minted.is_empty() {
        writeln!(out, "migrate --artifacts: nothing to mint (every supporting doc is covered).")?;
        return Ok(());
    }
    let verb = if report.dry_run { "would mint" } else { "minted" };
    let title = if report.dry_run { "ARTIFACTS (DRY RUN)" } else { "ARTIFACTS" };
    let mut table = Themed::new(title, &ARTIFACT_COLUMNS);
    for m in &report.minted {
        table.row([
            verb.to_string(),
            m.path.display().to_string(),
            m.contained_by.map_or_else(|| "—".to_string(), |id| id.to_string()),
            m.id.to_string(),
        ]);
    }
    table.summary(format!(
        "Total: {} {}",
        report.minted_count(),
        if report.dry_run { "to mint" } else { "minted" }
    ));
    writeln!(out, "{}", table.render())?;
    Ok(())
}

/// The `migrate --notes` arm: mints a `note` node for every dev doc under
/// `dev_root` not already covered (arc-migration-fidelity slice10, operator
/// decision 2026-07-28).
fn notes(
    store: &Store,
    dev_root: &Path,
    dry_run: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mode = Mode::from_dry_run(dry_run);
    let report = odm_migrate::notes::mint_notes(store, dev_root, mode)
        .with_context(|| format!("minting notes over {}", dev_root.display()))?;

    render_notes(&report, out)?;
    let status = format!(
        "{}: {} note(s) minted{}",
        if report.dry_run { "migrate --notes (dry-run)" } else { "migrate --notes" },
        report.minted_count(),
        if report.dry_run { " — nothing written" } else { "" },
    );
    if report.dry_run {
        term::info(err, &status)?
    } else {
        term::success(err, &status)?
    }
    Ok(())
}

const NOTE_COLUMNS: [&str; 4] = ["ACTION", "PATH", "TAG", "ID"];

/// Renders the note mint-all report: one row per minted (or, dry-run,
/// would-mint) note.
fn render_notes(
    report: &odm_migrate::notes::NoteReport,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    if report.minted.is_empty() {
        writeln!(out, "migrate --notes: nothing to mint (every dev doc is covered).")?;
        return Ok(());
    }
    let verb = if report.dry_run { "would mint" } else { "minted" };
    let title = if report.dry_run { "NOTES (DRY RUN)" } else { "NOTES" };
    let mut table = Themed::new(title, &NOTE_COLUMNS);
    for m in &report.minted {
        table.row([
            verb.to_string(),
            m.path.display().to_string(),
            m.tag.clone().unwrap_or_else(|| "—".to_string()),
            m.id.to_string(),
        ]);
    }
    table.summary(format!(
        "Total: {} {}",
        report.minted_count(),
        if report.dry_run { "to mint" } else { "minted" }
    ));
    writeln!(out, "{}", table.render())?;
    Ok(())
}

/// The reserved `number` the 1:1 `project-plan` clone [`vision`] mints —
/// distinct from every `arc`/`slice`/`artifact`/`note` number band this
/// crate assigns (`arc_number`'s lowest is `PROJECT_NUMBER + 100 = 1100`;
/// `artifact`/`note` start at 500,000,000/700,000,000) and confirmed free on
/// the live corpus before use. There is only ever one such node, so a fixed
/// constant — not a hash-derived band — is the simplest correct choice.
const VISION_PLAN_NUMBER: u32 = 1001;

/// The `migrate --vision` arm (arc-migration-fidelity s12/s13, MF-7's live
/// half): re-casts the project node as an **editorial-merge synthesis**
/// superseding a freshly-established **1:1 `project-plan` node**, via
/// [`apply_project_vision`].
///
/// The existing project node — `self_host`'s own 1:1 mint — is **cloned**
/// into a new node (fresh id, reserved number [`VISION_PLAN_NUMBER`]) that
/// becomes the permanent, immutable 1:1 record; the *original* project
/// node's own identity (id **and** number) is then re-cast in place as the
/// synthesis, so `orient`'s project lookup and every existing reference to
/// it keeps resolving to "the project," now vision-bearing. Idempotent: a
/// project node that already carries `source.synthesis` is left alone.
///
/// # Errors
///
/// Returns an error if no project node exists yet (run the plan-set import
/// first), if `plan_root`'s `project-plan.md` has no vision section, or on a
/// store I/O failure.
fn vision(
    store: &Store,
    plan_root: &Path,
    dry_run: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let corpus = store.load_all().context("loading the corpus for the vision re-cast")?;
    let Some(project) = corpus.iter().find(|d| d.frontmatter().node_type() == NodeType::Project)
    else {
        bail!("no project node found — run `odm migrate {} --plan` first", plan_root.display());
    };
    let fm = project.frontmatter();

    if fm.source().is_some_and(|s| s.synthesis.is_some()) {
        term::info(err, "migrate --vision: the project is already a synthesis — nothing to do")?;
        return Ok(());
    }

    // The project node never carries `source` (ODD-0025 §2.3 — its body is
    // meant to become the synthesis, so self-host structurally excludes it
    // from the ordinary source-population path). The 1:1 record this mints
    // is therefore read fresh from `project-plan.md`, the same way every
    // other migrated node's body is sourced — never from `project.body()`,
    // which may carry stale text from an earlier `--replan` restamp.
    let today = chrono::Utc::now().date_naive();
    let project_plan_path = plan_root.join("project-plan.md");
    let project_plan_body = std::fs::read_to_string(&project_plan_path)
        .with_context(|| format!("reading {}", project_plan_path.display()))?;
    let anchor = odm_migrate::fidelity::anchor_for(plan_root);
    let (created, updated) =
        odm_migrate::fidelity::git_derived_dates(&anchor, &project_plan_path, today);
    let plan_source_path =
        PathBuf::from(odm_migrate::fidelity::relativize(&anchor, &project_plan_path));

    let plan_id = Id::new();
    let mut plan_fm = Frontmatter::new(
        plan_id,
        VISION_PLAN_NUMBER,
        NodeType::Project,
        fm.name().to_string(),
        created,
        updated,
        Origin::Planned,
    );
    plan_fm.stamp_schema();
    if !fm.tags().is_empty() {
        plan_fm = plan_fm.with_tags(fm.tags().to_vec());
    }
    if let Some(component) = fm.component() {
        plan_fm = plan_fm.with_component(component);
    }
    plan_fm = plan_fm.with_source(odm_migrate::fidelity::build_source(
        vec![plan_source_path.clone()],
        "project-plan",
        today,
    ));
    let plan_document = Document::new(plan_fm, project_plan_body.clone());
    odm_migrate::fidelity::verify_body_hash(
        &project_plan_body,
        plan_document.body(),
        format!("#{VISION_PLAN_NUMBER} (project-plan 1:1)"),
    )?;

    let attestation = Attestation {
        by: "odm-migrate".to_string(),
        statement: "distills project-plan.md's Definition-of-done section verbatim".to_string(),
        on: today,
    };
    let (vision_fm, vision_body) = apply_project_vision(
        plan_id,
        plan_source_path,
        &project_plan_body,
        plan_root,
        fm.id(),
        fm.number(),
        fm.created(),
        today,
        attestation,
    )
    .with_context(|| format!("building the vision synthesis from {}", plan_root.display()))?;
    let vision_document = Document::new(vision_fm, vision_body);

    if !dry_run {
        store.persist(&plan_document).context("persisting the 1:1 project-plan node")?;
        store.persist(&vision_document).context("persisting the vision synthesis")?;
    }

    let verb = if dry_run { "would mint" } else { "minted" };
    writeln!(
        out,
        "migrate --vision: {verb} #{VISION_PLAN_NUMBER} (1:1 project-plan) and re-cast #{} as the \
         vision synthesis superseding it.",
        fm.number()
    )?;
    let status = format!(
        "migrate --vision{}: 1:1 node minted, project re-cast{}",
        if dry_run { " (dry-run)" } else { "" },
        if dry_run { " — nothing written" } else { "" },
    );
    if dry_run {
        term::info(err, &status)?
    } else {
        term::success(err, &status)?
    }
    Ok(())
}

/// Resolves the legacy path against `root` when it is relative (the CWD-rooted
/// convention), leaving an absolute path untouched.
fn resolve(root: &Path, legacy_path: &str) -> PathBuf {
    let p = Path::new(legacy_path);
    if p.is_absolute() { p.to_path_buf() } else { root.join(p) }
}

/// The `migrate` plan/result table's columns.
const MIGRATE_COLUMNS: [&str; 4] = ["ACTION", "#", "NAME / PATH", "NOTE"];

/// Renders the report: a plan/result table (create/upgrade/skip rows) + warnings.
fn render(report: &MigrationReport, out: &mut dyn Write) -> anyhow::Result<()> {
    if report.created.is_empty() && report.skipped.is_empty() && report.upgraded.is_empty() {
        writeln!(out, "migrate: no legacy documents found.")?;
        return Ok(());
    }

    let verb = if report.dry_run { "would create" } else { "created" };
    let upgrade_verb = if report.dry_run { "would upgrade" } else { "upgraded" };
    let title = if report.dry_run { "MIGRATE (DRY RUN)" } else { "MIGRATE" };
    let mut table = Themed::new(title, &MIGRATE_COLUMNS);
    for c in &report.created {
        table.row([verb.to_string(), c.number.to_string(), c.name.clone(), created_note(c)]);
    }
    for u in &report.upgraded {
        table.row([
            upgrade_verb.to_string(),
            u.number.to_string(),
            u.name.clone(),
            format!("→ {}", u.schema),
        ]);
    }
    for s in &report.skipped {
        table.row([
            "skip".to_string(),
            s.number.map_or_else(|| "—".to_string(), |n| n.to_string()),
            s.path.display().to_string(),
            s.reason.to_string(),
        ]);
    }
    table.summary(format!(
        "Total: {} {}, {} {}, {} skipped",
        report.created_count(),
        if report.dry_run { "to create" } else { "created" },
        report.upgraded_count(),
        if report.dry_run { "to upgrade" } else { "upgraded" },
        report.skipped_count()
    ));
    writeln!(out, "{}", table.render())?;

    if !report.warnings.is_empty() {
        writeln!(out, "\nwarnings:")?;
        for w in &report.warnings {
            writeln!(out, "  - {w}")?;
        }
    }
    Ok(())
}

/// The "note" cell for a created row: its type and identity, plus a retirement
/// flag.
fn created_note(c: &Created) -> String {
    let what = format!("{} {}", c.node_type.as_str(), c.id);
    if c.retired { format!("{what} (retired)") } else { what }
}

/// The `migrate --replan` arm: re-derive the existing plan nodes in place.
///
/// This is how a *derivation fix* reaches a corpus that was already minted.
/// Ordinary import cannot: it is idempotent on `(type, number)`, so it skips
/// every node that exists. Deriving into an empty store would apply the new
/// logic but mint fresh ULIDs, breaking every edge — so the corpus is rewritten
/// in place, ids untouched.
fn replan(
    store: &Store,
    root: &Path,
    plan_root: &Path,
    dry_run: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let home = StoreHome::resolve(root);
    let derived = odm_migrate::replan::derive_plan(&home.repo_root, plan_root)
        .with_context(|| format!("deriving the plan at {}", plan_root.display()))?;
    let vision = odm_migrate::replan::vision_from_plan(plan_root);

    let mode = Mode::from_dry_run(dry_run);
    let changes = odm_migrate::replan::restamp(store, &derived, vision.as_deref(), mode)
        .context("re-stamping the plan nodes")?;

    render_replan(&changes, dry_run, out)?;

    let summary = odm_migrate::replan::summarize(&changes);
    let line = format!(
        "{}: {} node(s) re-derived — {summary}{}",
        if dry_run { "replan (dry-run)" } else { "replan" },
        changes.len(),
        if dry_run { " — nothing written" } else { "" },
    );
    if dry_run {
        term::info(err, &line)?
    } else {
        term::success(err, &line)?
    }
    Ok(())
}

/// The `--replan` table's columns.
const REPLAN_COLUMNS: [&str; 4] = ["#", "NAME", "CREATED", "NOTE"];

/// Renders what the re-stamp changed, node by node — the diff an operator
/// should read before letting it touch the live corpus.
fn render_replan(
    changes: &[odm_migrate::replan::Restamped],
    dry_run: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    if changes.is_empty() {
        writeln!(out, "replan: every node already matches the plan.")?;
        return Ok(());
    }
    let title = if dry_run { "REPLAN (DRY RUN)" } else { "REPLAN" };
    let mut table = Themed::new(title, &REPLAN_COLUMNS);
    for change in changes {
        let name = change
            .name
            .as_ref()
            .map_or_else(|| "—".to_string(), |(old, new)| format!("{old} → {new}"));
        let created =
            change.created.map_or_else(|| "—".to_string(), |(old, new)| format!("{old} → {new}"));
        let mut notes = Vec::new();
        if change.moved {
            notes.push("relocated");
        }
        if change.vision {
            notes.push("vision");
        }
        table.row([change.number.to_string(), name, created, notes.join(", ")]);
    }
    table.summary(format!("Total: {} node(s) re-derived", changes.len()));
    writeln!(out, "{}", table.render())?;
    Ok(())
}
