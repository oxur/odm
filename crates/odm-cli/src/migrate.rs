//! `odm migrate` — import a legacy ODD corpus into the node model (Arc 06
//! slice01). The mapping + safety live in the `odm-migrate` crate; this module
//! resolves the gate-set, invokes it, and renders the [`MigrationReport`] with
//! `writeln!` + `tabled` (no `oxur-cli` — the A5/slice03 finding).

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use odm_core::NodeType;
use odm_migrate::mapping::canonical_odd_gates;
use odm_migrate::{Created, MigrationReport, Mode, SelfHostReport};
use odm_store::Store;
use tabled::builder::Builder;
use tabled::settings::Style;

use crate::commands;

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
    // Prefer a repo-configured `[gates.odd]`; fall back to the canonical set.
    let (gates, _) = commands::load_gate_config(root)?;
    let odd = gates.for_type(NodeType::Odd).cloned().unwrap_or_else(canonical_odd_gates);

    let mode = Mode::from_dry_run(dry_run);
    let report = odm_migrate::migrate_with_gates(store, &legacy, mode, &odd)
        .with_context(|| format!("migrating the legacy corpus at {}", legacy.display()))?;

    render(&report, out)?;
    writeln!(
        err,
        "{}: {} created, {} upgraded, {} skipped, {} warning(s){}",
        if report.dry_run { "migrate (dry-run)" } else { "migrate" },
        report.created_count(),
        report.upgraded_count(),
        report.skipped_count(),
        report.warnings.len(),
        if report.dry_run { " — nothing written" } else { "" },
    )?;
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
    writeln!(
        err,
        "{}: {} created, {} skipped{}",
        if report.dry_run { "self-host (dry-run)" } else { "self-host" },
        report.created_count(),
        report.skipped_count(),
        if report.dry_run { " — nothing written" } else { "" },
    )?;
    Ok(())
}

/// Renders the self-host cutover report: a create/skip table of the work nodes.
fn render_self_host(report: &SelfHostReport, out: &mut dyn Write) -> anyhow::Result<()> {
    if report.created.is_empty() && report.skipped.is_empty() {
        writeln!(out, "self-host: no plan-set nodes found.")?;
        return Ok(());
    }
    let verb = if report.dry_run { "would create" } else { "created" };
    let mut builder = Builder::default();
    builder.push_record(["action", "#", "name", "id"]);
    for c in &report.created {
        builder.push_record([verb, &c.number.to_string(), &c.name, &c.id.to_string()]);
    }
    for s in &report.skipped {
        let num = s.number.map_or_else(|| "—".to_string(), |n| n.to_string());
        builder.push_record(["skip", &num, "(already exists)", &s.reason.to_string()]);
    }
    let mut table = builder.build();
    table.with(Style::sharp());
    writeln!(out, "{table}")?;
    Ok(())
}

/// Resolves the legacy path against `root` when it is relative (the CWD-rooted
/// convention), leaving an absolute path untouched.
fn resolve(root: &Path, legacy_path: &str) -> PathBuf {
    let p = Path::new(legacy_path);
    if p.is_absolute() { p.to_path_buf() } else { root.join(p) }
}

/// Renders the report: a plan/result table (create/upgrade/skip rows) + warnings.
fn render(report: &MigrationReport, out: &mut dyn Write) -> anyhow::Result<()> {
    if report.created.is_empty() && report.skipped.is_empty() && report.upgraded.is_empty() {
        writeln!(out, "migrate: no legacy documents found.")?;
        return Ok(());
    }

    let verb = if report.dry_run { "would create" } else { "created" };
    let upgrade_verb = if report.dry_run { "would upgrade" } else { "upgraded" };
    let mut builder = Builder::default();
    builder.push_record(["action", "#", "name / path", "note"]);
    for c in &report.created {
        builder.push_record([verb, &c.number.to_string(), &c.name, &created_note(c)]);
    }
    for u in &report.upgraded {
        builder.push_record([
            upgrade_verb,
            &u.number.to_string(),
            &u.name,
            &format!("→ {}", u.schema),
        ]);
    }
    for s in &report.skipped {
        let num = s.number.map_or_else(|| "—".to_string(), |n| n.to_string());
        builder.push_record(["skip", &num, &s.path.display().to_string(), &s.reason.to_string()]);
    }
    let mut table = builder.build();
    table.with(Style::sharp());
    writeln!(out, "{table}")?;

    if !report.warnings.is_empty() {
        writeln!(out, "\nwarnings:")?;
        for w in &report.warnings {
            writeln!(out, "  - {w}")?;
        }
    }
    Ok(())
}

/// The "note" cell for a created row: its identity, plus a retirement flag.
fn created_note(c: &Created) -> String {
    if c.retired { format!("odd {} (retired)", c.id) } else { format!("odd {}", c.id) }
}
