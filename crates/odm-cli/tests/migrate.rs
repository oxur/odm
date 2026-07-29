//! In-process tests for the `odm migrate` command surface. Drives
//! [`odm_cli::dispatch`] against a temp store. Test names carry the substrings
//! the ledger Verify commands filter on: `migrate_command_exists` (slice01 M-1),
//! `check_green_on_migrated_odm_docs` (slice02 N-2).

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use tempfile::TempDir;

/// The absolute path of a fixture corpus under the workspace `test-data/`.
fn fixtures(name: &str) -> PathBuf {
    // odm-cli manifest dir is crates/odm-cli; the fixtures live at the root.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data").join(name)
}

/// The workspace path of the live `docs/design` corpus.
fn real_docs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design")
}

fn run(root: &Path, args: &[&str]) -> (bool, String, String) {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let ok = odm_cli::dispatch(cli, root, &mut out, &mut err).is_ok();
    (ok, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

/// Like [`run`] but returns the intended exit code (for `check`, which returns
/// its own code rather than erroring).
fn run_code(root: &Path, args: &[&str]) -> (Option<u8>, String) {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = odm_cli::dispatch(cli, root, &mut out, &mut err).ok();
    (code, String::from_utf8(out).unwrap())
}

#[test]
fn migrate_command_exists() {
    let store_dir = TempDir::new().unwrap();
    let legacy = fixtures("legacy");

    // Dry-run first: the command parses, runs, and writes nothing.
    let (ok, out, err) = run(store_dir.path(), &["migrate", legacy.to_str().unwrap(), "--dry-run"]);
    assert!(ok, "migrate --dry-run dispatches cleanly");
    assert!(out.contains("would create"), "dry-run plans creations:\n{out}");
    assert!(err.contains("dry-run"), "status names dry-run:\n{err}");
    assert!(!store_dir.path().join("nodes").exists(), "dry-run wrote nothing");

    // Commit: the six fixture docs become nodes.
    let (ok, _out, err) = run(store_dir.path(), &["migrate", legacy.to_str().unwrap()]);
    assert!(ok, "migrate dispatches cleanly");
    assert!(err.contains("6 created"), "reports six created:\n{err}");
    let store = Store::open(store_dir.path());
    assert_eq!(store.load_all().unwrap().len(), 6, "six nodes persisted");
}

#[test]
fn migrate_command_renders_warnings_and_skips() {
    // The edge corpus surfaces a skip table and a dangling-supersedes warning.
    let store_dir = TempDir::new().unwrap();
    let edge = fixtures("legacy-edge");
    let (ok, out, _err) = run(store_dir.path(), &["migrate", edge.to_str().unwrap()]);
    assert!(ok, "migrate over the edge corpus dispatches cleanly");
    assert!(out.contains("skip"), "skip rows rendered:\n{out}");
    assert!(out.contains("warnings:"), "warnings section rendered:\n{out}");
    assert!(out.contains("dangling"), "the dangling supersession is shown:\n{out}");
}

#[test]
fn migrate_command_reports_empty_corpus() {
    // An empty directory → the "no legacy documents" path (not an error).
    let store_dir = TempDir::new().unwrap();
    let empty = TempDir::new().unwrap();
    let (ok, out, _err) = run(store_dir.path(), &["migrate", empty.path().to_str().unwrap()]);
    assert!(ok);
    assert!(out.contains("no legacy documents found"), "empty-corpus message:\n{out}");
}

// ----- s10: sourceless nodes are backfilled in the same `migrate` pass ------

#[test]
fn migrate_backfills_a_sourceless_node_and_still_imports_the_rest() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A pre-source design node matching the legacy fixture's real #1
    // (test-data/legacy/01-draft/0001-early-draft.md) -- the pre-slice03
    // shape `backfill_source` exists to repair.
    let today = NaiveDate::from_ymd_opt(2026, 7, 28).unwrap();
    let fm =
        Frontmatter::new(Id::new(), 1, NodeType::Design, "Stub #1", today, today, Origin::Planned);
    store.persist(&Document::new(fm, "# Stub #1\n".to_string())).unwrap();

    let legacy = fixtures("legacy");
    let (ok, _out, err) = run(store_dir.path(), &["migrate", legacy.to_str().unwrap()]);
    assert!(ok, "migrate dispatches cleanly:\n{err}");
    assert!(err.contains("reconciled"), "the sourceless node is reported reconciled:\n{err}");
    assert!(err.contains("5 created"), "the other five fixture docs still import:\n{err}");

    let node = store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|d| d.frontmatter().number() == 1)
        .expect("#1 present");
    assert!(node.frontmatter().source().is_some(), "backfilled with source");
    assert!(
        node.body().contains("The first sketch"),
        "the stub body was replaced with the real legacy content: {:?}",
        node.body()
    );
}

// ----- s09/s10: `migrate --artifacts` mints the supporting-doc corpus -------

#[test]
fn migrate_artifacts_mints_supporting_docs() {
    let store_dir = TempDir::new().unwrap();
    let plan_set = fixtures("plan-set");

    // Self-host first so containment has something to resolve against.
    let (ok, _out, err) = run(store_dir.path(), &["migrate", plan_set.to_str().unwrap()]);
    assert!(ok, "self-host dispatches cleanly:\n{err}");

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", plan_set.to_str().unwrap(), "--artifacts"]);
    assert!(ok, "migrate --artifacts dispatches cleanly:\n{err}");
    assert!(err.contains("artifact(s) minted"), "status names what happened:\n{err}");
    assert!(out.contains("ARTIFACTS"), "the mint table is rendered:\n{out}");

    let store = Store::open(store_dir.path());
    let minted = store
        .load_all()
        .unwrap()
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == odm_core::NodeType::Artifact)
        .count();
    assert!(minted > 0, "at least one artifact was minted");
}

#[test]
fn migrate_artifacts_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let plan_set = fixtures("plan-set");
    run(store_dir.path(), &["migrate", plan_set.to_str().unwrap()]);

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", plan_set.to_str().unwrap(), "--artifacts", "--dry-run"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(out.contains("would mint"), "dry-run plans mints:\n{out}");
    assert!(err.contains("nothing written"), "status names dry-run:\n{err}");

    let store = Store::open(store_dir.path());
    let artifact_count = store
        .load_all()
        .unwrap()
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == odm_core::NodeType::Artifact)
        .count();
    assert_eq!(artifact_count, 0, "dry-run minted nothing");
}

// ----- s10: `migrate --notes` mints the dev-doc corpus ----------------------

#[test]
fn migrate_notes_mints_dev_docs() {
    let store_dir = TempDir::new().unwrap();
    let dev = TempDir::new().unwrap();
    std::fs::write(dev.path().join("0001-early-thoughts.md"), "# Early thoughts\nbody\n").unwrap();
    std::fs::create_dir_all(dev.path().join("research")).unwrap();
    std::fs::write(dev.path().join("research/0001-survey.md"), "# Survey\nbody\n").unwrap();

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", dev.path().to_str().unwrap(), "--notes"]);
    assert!(ok, "migrate --notes dispatches cleanly:\n{err}");
    assert!(err.contains("note(s) minted"), "status names what happened:\n{err}");
    assert!(out.contains("NOTES"), "the mint table is rendered:\n{out}");
    assert!(out.contains("research"), "the subdirectory tag is shown:\n{out}");

    let store = Store::open(store_dir.path());
    let notes: Vec<_> = store
        .load_all()
        .unwrap()
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == odm_core::NodeType::Note)
        .collect();
    assert_eq!(notes.len(), 2, "both dev docs minted");
    assert!(
        notes.iter().all(|n| n.frontmatter().edges().part_of.is_none()),
        "notes are uncontained"
    );
}

#[test]
fn migrate_notes_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let dev = TempDir::new().unwrap();
    std::fs::write(dev.path().join("0001-a.md"), "# A\nbody\n").unwrap();

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", dev.path().to_str().unwrap(), "--notes", "--dry-run"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(out.contains("would mint"), "dry-run plans mints:\n{out}");
    assert!(err.contains("nothing written"), "status names dry-run:\n{err}");

    let store = Store::open(store_dir.path());
    assert!(store.load_all().unwrap().is_empty(), "dry-run minted nothing");
}

#[test]
fn migrate_artifacts_and_notes_report_nothing_to_mint_once_covered() {
    let store_dir = TempDir::new().unwrap();
    let plan_set = fixtures("plan-set");
    run(store_dir.path(), &["migrate", plan_set.to_str().unwrap()]);
    run(store_dir.path(), &["migrate", plan_set.to_str().unwrap(), "--artifacts"]);

    let (ok, out, _err) =
        run(store_dir.path(), &["migrate", plan_set.to_str().unwrap(), "--artifacts"]);
    assert!(ok);
    assert!(out.contains("nothing to mint"), "a second run reports nothing left:\n{out}");

    let dev = TempDir::new().unwrap();
    std::fs::write(dev.path().join("0001-a.md"), "# A\nbody\n").unwrap();
    run(store_dir.path(), &["migrate", dev.path().to_str().unwrap(), "--notes"]);
    let (ok, out, _err) =
        run(store_dir.path(), &["migrate", dev.path().to_str().unwrap(), "--notes"]);
    assert!(ok);
    assert!(out.contains("nothing to mint"), "a second notes run reports nothing left:\n{out}");
}

#[test]
fn migrate_artifacts_and_notes_conflict() {
    assert!(
        Cli::try_parse_from(["odm", "migrate", "x", "--artifacts", "--notes"]).is_err(),
        "--artifacts and --notes are mutually exclusive"
    );
    assert!(
        Cli::try_parse_from(["odm", "migrate", "x", "--coverage", "--artifacts"]).is_err(),
        "--coverage and --artifacts are mutually exclusive"
    );
}

// ----- N-2: `odm check` is green on the migrated real ODD corpus -------------

#[test]
fn check_green_on_migrated_odm_docs() {
    // Import odm's real `docs/design` into a fresh store (migrate never mutates
    // the legacy tree — proven in odm-migrate's never-delete test), then assert
    // `odm check` is green (exit 0) on the imported document graph.
    let store_dir = TempDir::new().unwrap();
    let (ok, _out, err) = run(store_dir.path(), &["migrate", real_docs().to_str().unwrap()]);
    assert!(ok, "migrate real docs dispatches cleanly:\n{err}");
    assert!(err.contains("created"), "some ODDs imported:\n{err}");

    let (code, out) = run_code(store_dir.path(), &["validate"]);
    assert_eq!(code, Some(0), "check is green on the imported odd corpus:\n{out}");
}
