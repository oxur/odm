//! In-process, end-to-end tests for the reconcile+import flow `odm migrate`'s
//! self-host path now runs (arc-migration-fidelity slice06, CDC v2.1/v2.2
//! findings). Test names carry the ledger Verify substrings:
//! `migrate_reconciles_before_importing` (F-2/F-3/F-4/F-6), `migrate_dry_run`
//! (F-6), `migrate_reconcile_rerun_is_idempotent` (F-7).

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::schema::SchemaVersion;
use odm_core::{Id, NodeType, Origin};
use odm_migrate::selfhost::{arc_number, slice_number};
use odm_store::Store;
use tempfile::TempDir;

/// The project (root) node's number — matches `selfhost::PROJECT_NUMBER`
/// (crate-private; mirrored the same way `tests/selfhost.rs` does).
const PROJECT_NUMBER: u32 = 1000;

fn run(root: &Path, args: &[&str]) -> (bool, String, String) {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let ok = odm_cli::dispatch(cli, root, &mut out, &mut err).is_ok();
    (ok, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Points the store's `docs_directory` at `plan_root` (arc-migration-
/// fidelity s14: `migrate` is config-driven, no more `<LEGACY_PATH>`
/// positional) — `plan_root` is Plan-shaped, so the self-host derivation
/// resolves it directly (`discover_plan_roots` recognizes `docs_root`
/// itself as the one plan root when it directly qualifies).
fn set_docs_directory(store_root: &Path, plan_root: &Path) {
    std::fs::write(
        store_root.join("config.toml"),
        format!("docs_directory = {:?}\n", plan_root.display().to_string()),
    )
    .unwrap();
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, 20).unwrap()
}

/// Persists a node directly, **unversioned and sourceless** — standing in for
/// a real corpus node that predates arc-migration-fidelity entirely (no
/// `schema:`, no `source:`), so the flow's schema-bump and backfill are
/// actually exercised rather than already-satisfied.
fn persist(store: &Store, number: u32, node_type: NodeType, name: &str, body: &str) -> Id {
    let id = Id::new();
    let fm = Frontmatter::new(id, number, node_type, name, day(), day(), Origin::Planned);
    let document = Document::new(fm, body.to_string());
    store.persist(&document).unwrap();
    id
}

fn nodes_by_key(store: &Store) -> std::collections::HashMap<(NodeType, u32), Document> {
    store
        .load_all()
        .unwrap()
        .into_iter()
        .map(|d| ((d.frontmatter().node_type(), d.frontmatter().number()), d))
        .collect()
}

fn snapshot_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else {
                    out.push(path);
                }
            }
        }
    }
    let mut paths = Vec::new();
    walk(root, &mut paths);
    for path in paths {
        let bytes = std::fs::read(&path).unwrap();
        out.push((path.strip_prefix(root).unwrap().to_path_buf(), bytes));
    }
    out.sort();
    out
}

/// The fixture plan-set: a faithful arc, a stub slice beneath it, and a
/// second arc + slice that don't exist in the store yet (the "missing arc"
/// shape) — written once, shared by every test below.
fn write_plan_set(root: &Path) {
    write(root, "project-plan.md", "# Test Project\n\nThe real plan-of-record content.\n");
    write(
        root,
        "arc01-alpha/arc-plan.md",
        "# Arc 01 — Alpha (plan-of-record)\n\nReal, faithful arc content.\n",
    );
    write(
        root,
        "arc01-alpha/slice01-aa/slice-doc.md",
        "# Slice 01 (Arc 01) — Aa\n\nReal slice content, not just a heading.\n",
    );
    write(
        root,
        "arc02-beta/arc-plan.md",
        "# Arc 02 — Beta (plan-of-record)\n\nReal arc02 content.\n",
    );
    write(
        root,
        "arc02-beta/slice01-cc/slice-doc.md",
        "# Slice 01 (Arc 02) — Cc\n\nReal arc02 slice content.\n",
    );
}

/// Seeds the store with the pre-slice06 shape the flow must reconcile: a
/// **synthesis-shaped project** (no 1:1 source, must stay that way), a
/// **faithful** arc01 (body already matches its source, just needs `source`
/// added), and a **stub** slice01 (needs its body replaced).
fn seed_pre_existing(store: &Store) -> (Id, Id, Id) {
    let project_id = persist(
        store,
        PROJECT_NUMBER,
        NodeType::Project,
        "odm",
        "# odm\n\n# Vision\n\nSynthesized vision text, not the source verbatim.\n",
    );
    let arc_id = persist(
        store,
        arc_number(1),
        NodeType::Arc,
        "Arc 01 — Alpha",
        "# Arc 01 — Alpha (plan-of-record)\n\nReal, faithful arc content.\n",
    );
    let slice_id =
        persist(store, slice_number(1, 1, None), NodeType::Slice, "Slice 01", "# Slice 01\n");
    (project_id, arc_id, slice_id)
}

// ----- F-2/F-3/F-4/F-6: the full flow, in order, on one fixture -------------

#[test]
fn migrate_reconciles_before_importing_the_whole_flow_in_one_pass() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write_plan_set(root);

    let store_dir = TempDir::new().unwrap();
    set_docs_directory(store_dir.path(), root);
    let (project_id, arc_id, slice_id) = seed_pre_existing(&Store::open(store_dir.path()));

    let (ok, out, err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "migrate dispatches cleanly:\n{out}\n{err}");

    // The status line names all three counts — reconcile ran (F-2), and
    // import ran too (missing arc02 + its slice), in the same pass.
    assert!(err.contains("reconciled") && err.contains("created"), "status:\n{err}");

    let store = Store::open(store_dir.path());
    let nodes = nodes_by_key(&store);

    // The project: untouched body, still no `source` — excluded by *every*
    // path this flow exercises (F-6).
    let project = &nodes[&(NodeType::Project, PROJECT_NUMBER)];
    assert_eq!(project.frontmatter().id(), project_id, "project identity preserved");
    assert_eq!(
        project.body(),
        "# odm\n\n# Vision\n\nSynthesized vision text, not the source verbatim.\n",
        "project body untouched"
    );
    assert!(project.frontmatter().source().is_none(), "project never gets a 1:1 source");

    // arc01: faithful body kept verbatim, `source` added, schema bumped.
    let arc = &nodes[&(NodeType::Arc, arc_number(1))];
    assert_eq!(arc.frontmatter().id(), arc_id, "arc identity preserved");
    assert_eq!(
        arc.body(),
        "# Arc 01 — Alpha (plan-of-record)\n\nReal, faithful arc content.\n",
        "faithful arc body is a content no-op"
    );
    assert!(arc.frontmatter().source().is_some(), "arc source backfilled");
    assert_eq!(arc.frontmatter().schema_version(), SchemaVersion::CURRENT);

    // slice01 (arc01's child): stub body replaced with the verbatim source.
    let slice = &nodes[&(NodeType::Slice, slice_number(1, 1, None))];
    assert_eq!(slice.frontmatter().id(), slice_id, "slice identity preserved");
    assert_eq!(
        slice.body(),
        "# Slice 01 (Arc 01) — Aa\n\nReal slice content, not just a heading.\n",
        "stub body replaced with the verbatim source"
    );
    assert!(slice.frontmatter().source().is_some(), "slice source backfilled");
    assert_eq!(slice.frontmatter().schema_version(), SchemaVersion::CURRENT);

    // arc02 + its slice: genuinely missing beforehand — imported fresh.
    let arc2 = &nodes[&(NodeType::Arc, arc_number(2))];
    assert!(arc2.frontmatter().source().is_some(), "newly-imported arc carries source");
    assert_eq!(arc2.frontmatter().schema_version(), SchemaVersion::CURRENT);
    let slice2 = &nodes[&(NodeType::Slice, slice_number(2, 1, None))];
    assert!(slice2.frontmatter().source().is_some());

    assert_eq!(nodes.len(), 5, "project + 2 arcs + 2 slices, no extras");
}

// ----- F-6: --dry-run mutates nothing ---------------------------------------

#[test]
fn migrate_dry_run_previews_reconcile_and_import_writing_nothing() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write_plan_set(root);

    let store_dir = TempDir::new().unwrap();
    set_docs_directory(store_dir.path(), root);
    seed_pre_existing(&Store::open(store_dir.path()));
    let before = snapshot_bytes(store_dir.path());

    let (ok, out, err) = run(store_dir.path(), &["migrate", "--dry-run"]);
    assert!(ok, "dry-run dispatches cleanly:\n{out}\n{err}");
    assert!(
        out.contains("would reconcile") || out.contains("RECONCILE"),
        "previews the reconcile pass:\n{out}"
    );
    assert!(out.contains("would create"), "previews the import:\n{out}");

    let after = snapshot_bytes(store_dir.path());
    assert_eq!(before, after, "--dry-run wrote nothing, store byte-identical");
}

// ----- F-7: a second run is idempotent — no duplicate, nothing re-stamped --

#[test]
fn migrate_reconcile_rerun_is_idempotent() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write_plan_set(root);

    let store_dir = TempDir::new().unwrap();
    set_docs_directory(store_dir.path(), root);
    seed_pre_existing(&Store::open(store_dir.path()));

    let (ok, _out, err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "first run: {err}");
    let store = Store::open(store_dir.path());
    let first_count = store.load_all().unwrap().len();
    let first_ids: std::collections::HashSet<Id> =
        store.load_all().unwrap().iter().map(|d| d.frontmatter().id()).collect();

    // Second run: every node is now source-bearing, so nothing is reconciled
    // and nothing is (re-)created — pure idempotence (arc-migration-fidelity
    // s05's source-keyed identity, exercised end-to-end through this flow).
    let (ok, _out, err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "second run: {err}");
    assert!(err.contains("0 reconciled"), "nothing left to reconcile:\n{err}");
    assert!(err.contains("0 created"), "nothing left to import:\n{err}");

    let second_ids: std::collections::HashSet<Id> =
        store.load_all().unwrap().iter().map(|d| d.frontmatter().id()).collect();
    assert_eq!(store.load_all().unwrap().len(), first_count, "no duplicate on re-run");
    assert_eq!(first_ids, second_ids, "identical node identities across runs");
}
