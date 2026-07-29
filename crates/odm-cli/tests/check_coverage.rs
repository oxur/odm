//! In-process tests for `odm check`'s doc-coverage rule (arc-migration-
//! fidelity slice09, F-5, ODD-0025 §5): any `.md` under the configured
//! `[coverage] scan_root` with no covering node is an Error. Test names carry
//! the ledger's F-5 substring: `check_coverage`.
//!
//! **Fixture-only, by design (this slice's hard rule):** the rule fires only
//! when `[coverage] scan_root` is configured — absent by default, so it never
//! activates against a store that hasn't opted in (the live `.worktrees/odm`
//! store has no such key yet; turning it on there is a follow-on live-run
//! slice's job).

use std::fs;
use std::path::Path;

use chrono::NaiveDate;
use clap::Parser as _;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Frontmatter, Source};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use tempfile::TempDir;

/// The gate-sets a work corpus needs to validate at all.
const GATES: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

struct Run {
    code: Option<u8>,
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = odm_cli::dispatch(cli, root, &mut out, &mut err).ok();
    Run { code, out: String::from_utf8(out).unwrap() }
}

/// A flat store (no `[store]` redirection) whose `config.toml` opts into the
/// doc-coverage rule over `docs/` (relative to `root`).
fn fixture_with_scan_root(scan_root: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("config.toml"),
        format!("{GATES}[coverage]\nscan_root = \"{scan_root}\"\n"),
    )
    .unwrap();
    dir
}

fn write_doc(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// Persists a node whose `source.paths` covers `relative_to_scan_root` — the
/// exact key `coverage::run` matches against when the fixture root carries no
/// `.git` (its anchor falls back to the scan root itself, s08).
fn seed_covering_node(root: &Path, relative_to_scan_root: &str) {
    let today = NaiveDate::from_ymd_opt(2026, 7, 28).unwrap();
    let fm = Frontmatter::new(
        Id::new(),
        1,
        NodeType::Artifact,
        "Covering node",
        today,
        today,
        Origin::Planned,
    )
    .with_source(Source {
        paths: vec![relative_to_scan_root.into()],
        class: "other".to_string(),
        normalization: "trim+lf".to_string(),
        migrated_by: "test".to_string(),
        migrated_on: today,
    });
    let document = Document::new(fm, "# Covering node\n".to_string());
    Store::open(root).persist(&document).expect("seed persist");
}

// ----- F-5: an uncovered doc under the scan root is an Error ----------------

#[test]
fn check_coverage_flags_an_uncovered_doc_as_an_error() {
    let dir = fixture_with_scan_root("docs");
    write_doc(dir.path(), "docs/orphan.md", "# Orphan\n\nNo node covers this.\n");

    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1), "an uncovered doc is a hard Error: {}", r.out);
    assert!(r.out.contains("uncovered-doc"), "the rule fires: {}", r.out);
    assert!(r.out.contains("orphan.md"), "and names the file: {}", r.out);
}

// ----- F-5: covering the doc clears the finding ------------------------------

#[test]
fn check_coverage_is_green_once_the_doc_is_covered() {
    let dir = fixture_with_scan_root("docs");
    write_doc(dir.path(), "docs/covered.md", "# Covered\n\nA node covers this.\n");
    seed_covering_node(dir.path(), "covered.md");

    let r = run(dir.path(), &["check"]);
    assert!(!r.out.contains("uncovered-doc"), "covered, so silent: {}", r.out);
    assert_eq!(r.code, Some(0), "no uncovered docs remain: {}", r.out);
}

// ----- F-5: absent `[coverage] scan_root` is a deliberate no-op -------------

#[test]
fn check_coverage_rule_is_a_noop_without_scan_root_configured() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config.toml"), GATES).unwrap();
    write_doc(dir.path(), "docs/orphan.md", "# Orphan\n");

    let r = run(dir.path(), &["check"]);
    assert!(
        !r.out.contains("uncovered-doc"),
        "no `[coverage] scan_root` configured -- the rule stays inert: {}",
        r.out
    );
    assert_eq!(r.code, Some(0));
}

// ----- F-5: the coverage key resolves identically for two config spellings -

#[test]
fn check_coverage_result_is_identical_from_a_relative_and_an_absolute_scan_root() {
    let relative_dir = fixture_with_scan_root("docs");
    write_doc(relative_dir.path(), "docs/orphan.md", "# Orphan\n");
    let relative_result = run(relative_dir.path(), &["check"]);

    let absolute_dir = TempDir::new().unwrap();
    let absolute_scan_root = absolute_dir.path().join("docs");
    fs::write(
        absolute_dir.path().join("config.toml"),
        format!("{GATES}[coverage]\nscan_root = \"{}\"\n", absolute_scan_root.display()),
    )
    .unwrap();
    write_doc(absolute_dir.path(), "docs/orphan.md", "# Orphan\n");
    let absolute_result = run(absolute_dir.path(), &["check"]);

    assert_eq!(relative_result.code, absolute_result.code);
    assert!(relative_result.out.contains("orphan.md") && absolute_result.out.contains("orphan.md"));
}
