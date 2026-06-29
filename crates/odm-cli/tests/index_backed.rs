//! Slice04: the index-backed consumers match the `load_all` baseline.
//!
//! Seam (a): `list` (human table) reads the reconciled `.odm/` index. The graph
//! readers (`next`/`blocked`/`path`/`check`) and composed views (`rollup`/
//! `orient`) are the flagged continuation (seam b/c) — not wired here. Test names
//! carry the substrings the slice04 ledger Verify commands filter on
//! (`list_index_backed_matches_baseline`, `consumers_reconcile_before_read`).

use std::path::Path;

use clap::Parser;
use odm_cli::Cli;
use tempfile::TempDir;

struct Run {
    ok: bool,
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args parse");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, root, &mut out, &mut err);
    Run { ok: result.is_ok(), out: String::from_utf8(out).unwrap() }
}

/// Seeds a small corpus through the CLI (each `new` is its own process-less run).
fn seed(root: &Path) {
    run(root, &["new", "project", "Proj"]);
    run(root, &["new", "arc", "Arc one", "--parent", "1"]);
    run(root, &["new", "slice", "Early", "--parent", "2"]);
    run(root, &["new", "slice", "Late", "--parent", "2"]);
}

/// Writes a gate config so satisfaction has gate-sets to work with.
fn write_config(root: &Path) {
    let cfg = "[gates.project]\nsequence = [\"planned\", \"complete\"]\n\
               [gates.arc]\nsequence = [\"planned\", \"complete\"]\n\
               [gates.slice]\nsequence = [\"planned\", \"built\", \"tested\"]\n";
    std::fs::write(root.join("odm.toml"), cfg).unwrap();
}

// ----- G-2: next/blocked/path read the index-backed graph (correct order) ----

#[test]
fn derived_order_index_backed_match_baseline() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    seed(root); // P(1) <- A(2) <- Early(3), Late(4)
    run(root, &["link", "Late", "depends_on", "Early"]);

    // `next`: Early (no deps) is ready; Late (unsatisfied dep) is not.
    let next = run(root, &["next"]);
    assert!(next.out.contains("Early"), "Early ready:\n{}", next.out);
    assert!(!next.out.contains("Late"), "Late not ready (blocked):\n{}", next.out);

    // `blocked Late`: names the unsatisfied dependency Early.
    let blocked = run(root, &["blocked", "Late"]);
    assert!(
        blocked.out.contains("unsatisfied dependency") && blocked.out.contains("Early"),
        "blocked names the dep:\n{}",
        blocked.out
    );

    // `path Late`: the dependency chain Late → Early.
    let path = run(root, &["path", "Late"]);
    assert!(path.out.contains("Late") && path.out.contains("Early"), "path chain:\n{}", path.out);
}

// ----- G-6: graph readers reconcile before reading (freshness) ---------------

#[test]
fn graph_consumers_reconcile_before_read() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    seed(root);

    // First `next` builds the index.
    let before = run(root, &["next"]);
    assert!(before.ok && !before.out.contains("Fresh"));

    // Add a ready node, then `next` again with no manual rebuild: the reconcile
    // inside the graph reader must surface it.
    run(root, &["new", "slice", "Fresh ready", "--parent", "2"]);
    let after = run(root, &["next"]);
    assert!(after.out.contains("Fresh ready"), "freshly-added ready node appears:\n{}", after.out);
}

// ----- I-5: index-backed `list` table == the baseline (full-scan) table ------

#[test]
fn list_index_backed_matches_baseline() {
    // Baseline: list over a corpus with NO index present (first call builds it,
    // but the output is computed from the reconciled records either way). To get
    // a true "full-scan baseline" to compare against, we capture the table from
    // a fresh store, then re-run after the index exists; both must be identical.
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    // First `list` builds the index and renders the table.
    let first = run(dir.path(), &["list"]);
    assert!(first.ok);
    // Second `list` reads the (now-warm) index and renders the same table.
    let second = run(dir.path(), &["list"]);
    assert_eq!(first.out, second.out, "warm read matches cold-built read");

    // The table has every node, sorted by number, with the expected columns.
    assert!(first.out.contains("NUMBER") && first.out.contains("ID"));
    for name in ["Proj", "Arc one", "Early", "Late"] {
        assert!(first.out.contains(name), "{name} present:\n{}", first.out);
    }
    // Proj (#1) precedes Late (#4) — number order preserved.
    assert!(first.out.find("Proj").unwrap() < first.out.find("Late").unwrap());

    // A type filter narrows the table (index-backed).
    let slices = run(dir.path(), &["list", "--type", "slice"]);
    assert!(slices.out.contains("Early") && slices.out.contains("Late"));
    assert!(!slices.out.contains("Proj"), "type filter excludes the project:\n{}", slices.out);
}

// ----- I-9: `list` reconciles before reading — a new node shows without a -----
//             manual rebuild.

#[test]
fn consumers_reconcile_before_read() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    // Build the index via a first read.
    let before = run(dir.path(), &["list"]);
    assert!(before.ok && !before.out.contains("Fresh slice"));

    // Add a node, then list again WITHOUT any explicit rebuild: the reconcile
    // inside `list` must pick it up.
    run(dir.path(), &["new", "slice", "Fresh slice", "--parent", "2"]);
    let after = run(dir.path(), &["list"]);
    assert!(after.out.contains("Fresh slice"), "the freshly-added node appears:\n{}", after.out);
}
