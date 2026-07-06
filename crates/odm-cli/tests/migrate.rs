//! In-process tests for the `odm migrate` command surface. Drives
//! [`odm_cli::dispatch`] against a temp store. Test names carry the substrings
//! the ledger Verify commands filter on: `migrate_command_exists` (slice01 M-1),
//! `check_green_on_migrated_odm_docs` (slice02 N-2).

use std::path::{Path, PathBuf};

use clap::Parser;
use odm_cli::Cli;
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

// ----- N-2: `odm check` is green on the migrated real ODD corpus -------------

#[test]
fn check_green_on_migrated_odm_docs() {
    // Import odm's real `docs/design` into a fresh store (migrate never mutates
    // the legacy tree — proven in odm-migrate's never-delete test), then assert
    // `odm check` is green (exit 0) on the imported `odd` graph.
    let store_dir = TempDir::new().unwrap();
    let (ok, _out, err) = run(store_dir.path(), &["migrate", real_docs().to_str().unwrap()]);
    assert!(ok, "migrate real docs dispatches cleanly:\n{err}");
    assert!(err.contains("created"), "some ODDs imported:\n{err}");

    let (code, out) = run_code(store_dir.path(), &["check"]);
    assert_eq!(code, Some(0), "check is green on the imported odd corpus:\n{out}");
}
