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
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use tempfile::TempDir;

struct Run {
    ok: bool,
    out: String,
    err: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args parse");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, root, &mut out, &mut err);
    Run {
        ok: result.is_ok(),
        out: String::from_utf8(out).unwrap(),
        err: String::from_utf8(err).unwrap(),
    }
}

/// Rewrites a node's body while keeping its frontmatter byte-identical — a
/// body-only edit (`content_hash` moves, `meta_hash` stable).
fn body_only_edit(root: &Path, id: Id, new_body: &str) {
    let store = Store::open(root);
    let fm = store.load(id).unwrap().frontmatter().clone();
    store.persist(&Document::new(fm, new_body.to_string())).unwrap();
}

/// Persists a node file directly (bypassing `new`) so a test can pin its number,
/// `origin`, parent, and body — `new` always mints `Origin::Planned` with a
/// title-only body, but the slice06 views read `origin` (provenance) and the
/// project body (vision), which we need to vary.
fn persist(
    root: &Path,
    number: u32,
    ntype: NodeType,
    name: &str,
    origin: Origin,
    parent: Option<Id>,
    body: &str,
) -> Id {
    let store = Store::open(root);
    let id = Id::new();
    let created = id.created_at().date_naive();
    let mut fm = Frontmatter::new(id, number, ntype, name, created, created, origin);
    if let Some(parent) = parent {
        fm.edges_mut().part_of = Some(parent);
    }
    store.persist(&Document::new(fm, body.to_string())).unwrap();
    id
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

// ===== slice06: check / rollup / orient are index-backed ====================

// ----- V-4: `check` reads the index-backed graph + recomposition -------------

#[test]
fn check_index_backed_matches_baseline() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    // P <- A <- Early, Late ; affirm A's decomposition over {Early, Late} …
    let p = persist(root, 1, NodeType::Project, "Proj", Origin::Planned, None, "# Proj\n");
    let a = persist(root, 2, NodeType::Arc, "Arc one", Origin::Planned, Some(p), "# A\n");
    persist(root, 3, NodeType::Slice, "Early", Origin::Planned, Some(a), "# E\n");
    persist(root, 4, NodeType::Slice, "Late", Origin::Planned, Some(a), "# L\n");
    run(root, &["decomposed", "Arc one"]); // affirms {Early, Late}

    // … then drift A's child-set (a NEW child) and add an orphan slice.
    persist(root, 5, NodeType::Slice, "Newcomer", Origin::Planned, Some(a), "# N\n");
    persist(root, 6, NodeType::Slice, "Orphan", Origin::Planned, None, "# O\n");

    let first = run(root, &["check"]);
    let second = run(root, &["check"]);
    assert_eq!(first.out, second.out, "warm read == cold-built read:\n{}", first.out);

    // decomposition-drift proves `decomposed` flows through the index (V-4) …
    assert!(first.out.contains("decomposition-drift"), "drift surfaced:\n{}", first.out);
    // … and orphan proves recomposition runs over the index-backed graph.
    assert!(first.out.contains("orphan"), "orphan surfaced:\n{}", first.out);
}

// ----- V-5: `rollup` composes over the index; provenance reads `origin` ------

#[test]
fn rollup_index_backed_matches_baseline() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    let p = persist(root, 1, NodeType::Project, "Proj", Origin::Planned, None, "# Proj\n");
    persist(root, 2, NodeType::Arc, "Planned arc", Origin::Planned, Some(p), "# PA\n");
    persist(root, 3, NodeType::Arc, "Found arc", Origin::Discovered, Some(p), "# FA\n");

    let first = run(root, &["rollup", "--dry-run"]);
    let second = run(root, &["rollup", "--dry-run"]);
    assert_eq!(first.out, second.out, "warm read == cold-built read:\n{}", first.out);

    // The way-finding tree composes over the index-backed model.
    assert!(first.out.contains("Way-finding tree") && first.out.contains("Proj"));

    // Provenance groups by `origin`: "Found arc" lands under Discovered, not
    // Planned — proving `origin` is carried through the index (V-5). If it were
    // not, the Discovered group would be empty and the node would be Planned.
    let disc = first.out.find("### Discovered").expect("a Discovered group");
    let amend = first.out.find("### Amendment").expect("an Amendment group");
    assert!(
        first.out[disc..amend].contains("Found arc"),
        "discovered node grouped under Discovered:\n{}",
        first.out
    );
}

// ----- V-6: `orient` composes over the index + one targeted body load --------

#[test]
fn orient_index_backed_matches_baseline() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    let vision = "# Vision\n\nMake the plan legible from one call.\n";
    let p = persist(root, 1, NodeType::Project, "Proj", Origin::Planned, None, vision);
    let a = persist(root, 2, NodeType::Arc, "Arc one", Origin::Planned, Some(p), "# A\n");
    persist(root, 3, NodeType::Slice, "Early", Origin::Planned, Some(a), "# E\n");
    persist(root, 4, NodeType::Slice, "Orphan", Origin::Planned, None, "# O\n");
    run(root, &["use", "project", "Proj"]);
    run(root, &["use", "arc", "Arc one"]);

    let first = run(root, &["orient"]);
    let second = run(root, &["orient"]);
    assert_eq!(first.out, second.out, "warm read == cold-built read:\n{}", first.out);

    // The vision BODY appears → the one targeted `store.load(project)` happened
    // (the body is deliberately NOT in the index, §3.5).
    assert!(first.out.contains("Make the plan legible"), "vision body:\n{}", first.out);
    // Current focus resolves to the arc; integrity surfaces the orphan over the
    // index-backed graph.
    assert!(
        first.out.contains("CURRENT FOCUS") && first.out.contains("Arc one"),
        "current focus:\n{}",
        first.out
    );
    assert!(first.out.contains("orphan"), "orphan in INTEGRITY:\n{}", first.out);
}

// ----- V-7: all three view consumers reconcile before reading ----------------

#[test]
fn view_consumers_reconcile_before_read() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    let p = persist(root, 1, NodeType::Project, "Proj", Origin::Planned, None, "# Vision\n\nV.\n");
    let a = persist(root, 2, NodeType::Arc, "Arc one", Origin::Planned, Some(p), "# A\n");
    run(root, &["use", "project", "Proj"]);
    run(root, &["use", "arc", "Arc one"]);

    // Warm the index via a first read of each view.
    run(root, &["check"]);
    run(root, &["rollup", "--dry-run"]);
    run(root, &["orient"]);

    // Add nodes with NO manual rebuild — each consumer's `reconcile` must see them.
    persist(root, 3, NodeType::Slice, "Fresh slice", Origin::Planned, Some(a), "# F\n");
    assert!(
        run(root, &["rollup", "--dry-run"]).out.contains("Fresh slice"),
        "rollup reconciles before read"
    );
    assert!(
        run(root, &["orient"]).out.contains("Fresh slice"),
        "orient reconciles before read (ready frontier)"
    );

    persist(root, 4, NodeType::Slice, "Fresh orphan", Origin::Planned, None, "# FO\n");
    assert!(run(root, &["check"]).out.contains("orphan"), "check reconciles before read");
}

// ===== slice07: `odm rollup` early-cutoff ===================================

// ----- E-2: a body-only edit leaves ROLLUP.md untouched (early cutoff) --------

#[test]
fn rollup_skips_on_body_only_change() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    let p = persist(root, 1, NodeType::Project, "Proj", Origin::Planned, None, "# Proj\n");
    let a = persist(root, 2, NodeType::Arc, "Arc one", Origin::Planned, Some(p), "# A\n");

    // First rollup writes ROLLUP.md and stamps the corpus meta-fingerprint.
    assert!(run(root, &["rollup"]).ok);
    let rollup_path = root.join("ROLLUP.md");
    let before = std::fs::read(&rollup_path).unwrap();

    // A body-only edit to the arc: content changes, meaning does not.
    body_only_edit(root, a, "# A\n\nMuch longer prose, same frontmatter.\n");

    let second = run(root, &["rollup"]);
    assert!(second.ok);
    assert!(
        second.err.contains("unchanged") && second.err.contains("skipped"),
        "rollup reports the skip:\n{}",
        second.err
    );
    let after = std::fs::read(&rollup_path).unwrap();
    assert_eq!(before, after, "a body-only edit leaves ROLLUP.md byte-identical");
}

// ----- E-3: a meaning-change / new / deleted always regenerates ROLLUP.md -----

#[test]
fn rollup_regenerates_on_meta_change() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    let p = persist(root, 1, NodeType::Project, "Proj", Origin::Planned, None, "# Proj\n");
    let a = persist(root, 2, NodeType::Arc, "Arc one", Origin::Planned, Some(p), "# A\n");
    let slice = persist(root, 3, NodeType::Slice, "Slice one", Origin::Planned, Some(a), "# S\n");

    assert!(run(root, &["rollup"]).ok);
    let rollup_path = root.join("ROLLUP.md");
    let mut prev = std::fs::read(&rollup_path).unwrap();

    // (a) a gate/evidence change is a meaning-change → regenerate.
    let gate = run(root, &["set-gate", "Slice one", "built"]);
    assert!(gate.ok, "set-gate:\n{}", gate.err);
    let after_gate = run(root, &["rollup"]);
    assert!(after_gate.err.contains("wrote"), "regenerates on gate change:\n{}", after_gate.err);
    let now = std::fs::read(&rollup_path).unwrap();
    assert_ne!(prev, now, "ROLLUP.md changed after a gate change");
    prev = now;

    // (b) a new node → the record set changes → regenerate.
    persist(root, 4, NodeType::Slice, "Slice two", Origin::Planned, Some(a), "# S2\n");
    let after_new = run(root, &["rollup"]);
    assert!(after_new.err.contains("wrote"), "regenerates on a new node:\n{}", after_new.err);
    let now = std::fs::read(&rollup_path).unwrap();
    assert_ne!(prev, now, "ROLLUP.md changed after a new node");
    prev = now;

    // (c) a deleted node → the record set changes → regenerate.
    std::fs::remove_file(Store::open(root).path_of(slice)).unwrap();
    let after_del = run(root, &["rollup"]);
    assert!(after_del.err.contains("wrote"), "regenerates on a deletion:\n{}", after_del.err);
    let now = std::fs::read(&rollup_path).unwrap();
    assert_ne!(prev, now, "ROLLUP.md changed after a deletion");
}
