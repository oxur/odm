//! `odm migrate` — import a legacy ODD corpus into the node model (Arc 06
//! slice01). The mapping + safety live in the `odm-migrate` crate; this module
//! resolves the gate-set, invokes it, and renders the [`MigrationReport`] as an
//! Oxur-themed table ([`oxur_term::table::OxurTable`], RH C-1).

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use odm_core::NodeType;
use odm_migrate::coverage::{CoverageReport, DocClass};
use odm_migrate::mapping::{DocGates, canonical_design_gates, canonical_research_gates};
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

/// The plan-set derivation: **reconciles, then imports** a `design-vX.Y.Z/`
/// plan set (project + arcs + slices) as work nodes.
///
/// Reached only through `migrate` now. `self-host` was folded into it by RH C-5
/// (F-14) and the surviving top-level spelling was removed by C-4, per ODD-0023
/// §5 — one verb, whose derivation the tree's shape selects.
///
/// Runs the full migration-fidelity flow in **load-bearing order**
/// (arc-migration-fidelity s06 F-2/F-3): [`odm_migrate::selfhost::repair`]
/// **first** — repairs stub bodies and gated-backfills `source` onto every
/// existing faithful node (the project excluded, ODD-0025 §2.3) — **then**
/// [`odm_migrate::selfhost::self_host`], so the import's source-keyed
/// idempotence (s05) finds every already-reconciled node by its now-populated
/// `source.paths`, not just the ones that already had one. Reordering these
/// would reintroduce exactly the gap s06 closed — a reconciled-before-import
/// invariant, not an incidental sequence.
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
    let report = odm_migrate::selfhost::self_host(store, &plan_root, mode)
        .with_context(|| format!("self-hosting the plan set at {}", plan_root.display()))?;

    render_repair(&repair_report, out)?;
    render_self_host(&report, out)?;
    let status = format!(
        "{}: {} reconciled, {} created, {} skipped{}",
        if report.dry_run { "self-host (dry-run)" } else { "self-host" },
        repair_report.repaired_count(),
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
