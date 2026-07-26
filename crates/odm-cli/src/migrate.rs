//! `odm migrate` — import a legacy ODD corpus into the node model (Arc 06
//! slice01). The mapping + safety live in the `odm-migrate` crate; this module
//! resolves the gate-set, invokes it, and renders the [`MigrationReport`] as an
//! Oxur-themed table ([`oxur_term::table::OxurTable`], RH C-1).

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use odm_core::NodeType;
use odm_migrate::mapping::{DocGates, canonical_design_gates, canonical_research_gates};
use odm_migrate::{Created, MigrationReport, Mode, SelfHostReport};
use odm_store::Store;

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
pub(crate) fn migrate(
    store: &Store,
    root: &Path,
    legacy_path: &str,
    dry_run: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let legacy = resolve(root, legacy_path);
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
    let report = odm_migrate::migrate_with_gates(store, &legacy, mode, &doc_gates)
        .with_context(|| format!("migrating the legacy corpus at {}", legacy.display()))?;

    render(&report, out)?;
    let status = format!(
        "{}: {} created, {} upgraded, {} skipped, {} warning(s){}",
        if report.dry_run { "migrate (dry-run)" } else { "migrate" },
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

/// Runs `odm self-host <plan-path> [--dry-run]`: imports odm's own `design-v1.0.0`
/// plan set (project + A1–A6 arcs + slices) into `store` as work nodes — the
/// loop-closer. The report goes to `out`; a one-line status to `err`.
///
/// # Errors
///
/// Returns an error (exit code `2`) if the cutover hits a store I/O failure.
pub(crate) fn self_host(
    store: &Store,
    root: &Path,
    plan_path: &str,
    dry_run: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let plan_root = resolve(root, plan_path);
    let mode = Mode::from_dry_run(dry_run);
    let report = odm_migrate::selfhost::self_host(store, &plan_root, mode)
        .with_context(|| format!("self-hosting the plan set at {}", plan_root.display()))?;

    render_self_host(&report, out)?;
    let status = format!(
        "{}: {} created, {} skipped{}",
        if report.dry_run { "self-host (dry-run)" } else { "self-host" },
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
