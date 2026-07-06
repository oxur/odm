//! `odm migrate` — import a legacy ODD corpus into the node model (Arc 06
//! slice01). The mapping + safety live in the `odm-migrate` crate; this module
//! resolves the gate-set, invokes it, and renders the [`MigrationReport`] with
//! `writeln!` + `tabled` (no `oxur-cli` — the A5/slice03 finding).

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use odm_core::NodeType;
use odm_migrate::mapping::canonical_odd_gates;
use odm_migrate::{Created, MigrationReport, Mode};
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
        "{}: {} created, {} skipped, {} warning(s){}",
        if report.dry_run { "migrate (dry-run)" } else { "migrate" },
        report.created_count(),
        report.skipped_count(),
        report.warnings.len(),
        if report.dry_run { " — nothing written" } else { "" },
    )?;
    Ok(())
}

/// Resolves the legacy path against `root` when it is relative (the CWD-rooted
/// convention), leaving an absolute path untouched.
fn resolve(root: &Path, legacy_path: &str) -> PathBuf {
    let p = Path::new(legacy_path);
    if p.is_absolute() { p.to_path_buf() } else { root.join(p) }
}

/// Renders the report: a plan/result table (create/skip rows) + any warnings.
fn render(report: &MigrationReport, out: &mut dyn Write) -> anyhow::Result<()> {
    if report.created.is_empty() && report.skipped.is_empty() {
        writeln!(out, "migrate: no legacy documents found.")?;
        return Ok(());
    }

    let verb = if report.dry_run { "would create" } else { "created" };
    let mut builder = Builder::default();
    builder.push_record(["action", "#", "name / path", "note"]);
    for c in &report.created {
        builder.push_record([verb, &c.number.to_string(), &c.name, &created_note(c)]);
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
