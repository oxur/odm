//! Tests for the store layer.
//!
//! Test names contain the substrings the ledger Verify commands filter on
//! (`path_from_ulid`, `locate_by_id`, `persist_reload_roundtrip`,
//! `atomic_write_no_partial`, `full_scan_load`, `gix_stage_commit`,
//! `config_layered_load`, `missing_dir_selfheal`).

use std::fs;
use std::path::Path;
use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_store::{Repo, Store, StoreConfig, atomic, layout};
use tempfile::TempDir;

// Spec example ULID → 2016-07-30T23:54:10Z, so its shard is nodes/2016/07.
const SAMPLE_ULID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

fn doc_with_id(id: Id, number: u32, name: &str) -> Document {
    let fm = Frontmatter::new(
        id,
        number,
        NodeType::Slice,
        name,
        day(2026, 6, 20),
        day(2026, 6, 21),
        Origin::Planned,
    );
    Document::new(fm, format!("# {name}\n\nBody for {name}.\n"))
}

// ----- J-1: path = nodes/YYYY/MM/<ULID>.md, month from the ULID --------------

#[test]
fn path_from_ulid_uses_creation_month() {
    let id = Id::from_str(SAMPLE_ULID).unwrap();
    let rel = layout::relative_path(id);
    assert_eq!(rel, Path::new("nodes").join("2016").join("07").join(format!("{SAMPLE_ULID}.md")));
}

// ----- J-2: locate-by-id is O(1), a pure function of the id ------------------

#[test]
fn locate_by_id_is_pure_function_no_scan() {
    let root = TempDir::new().unwrap();
    let store = Store::open(root.path());
    assert_eq!(store.root(), root.path());
    let id = Id::from_str(SAMPLE_ULID).unwrap();
    // path_of touches no filesystem and needs no existing file.
    let p1 = store.path_of(id);
    let p2 = store.path_of(id);
    assert_eq!(p1, p2);
    assert_eq!(p1, root.path().join(layout::relative_path(id)));
    assert!(!p1.exists()); // computed without the file existing
}

// ----- J-3: persist → reload round-trips a node set identically -------------

#[test]
fn persist_reload_roundtrip_preserves_node_set() {
    let root = TempDir::new().unwrap();
    let store = Store::open(root.path());

    let mut originals: Vec<Document> =
        (0..5).map(|i| doc_with_id(Id::new(), i, &format!("node {i}"))).collect();
    for doc in &originals {
        store.persist(doc).expect("persist");
    }

    let mut reloaded = store.load_all().expect("load_all");
    originals.sort_by_key(|d| d.frontmatter().id());
    reloaded.sort_by_key(|d| d.frontmatter().id());
    assert_eq!(reloaded, originals);

    // Single-node locate + load also round-trips.
    let one = &originals[0];
    let got = store.load(one.frontmatter().id()).expect("load one");
    assert_eq!(&got, one);
}

// ----- J-4: atomic write leaves no partial/corrupt file ---------------------

#[test]
fn atomic_write_no_partial_or_temp_leftover() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("node.md");

    // Successful write: exact content, and no temp file left behind.
    atomic::write(&target, b"OLD").expect("write old");
    assert_eq!(fs::read(&target).unwrap(), b"OLD");
    atomic::write(&target, b"NEW longer contents").expect("overwrite");
    assert_eq!(fs::read(&target).unwrap(), b"NEW longer contents");
    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "no .tmp file should remain");
}

#[test]
fn atomic_write_no_partial_on_failure_preserves_existing() {
    let dir = TempDir::new().unwrap();
    // `blocker` is a regular file; writing "through" it as if it were a
    // directory must fail at create_dir_all and leave the file untouched.
    let blocker = dir.path().join("blocker");
    fs::write(&blocker, b"ORIGINAL").unwrap();

    let bad_target = blocker.join("sub").join("x.md");
    let err = atomic::write(&bad_target, b"SHOULD NOT LAND");
    assert!(err.is_err(), "write under a file-parent must fail");
    // The pre-existing data is intact; no partial/corrupt file appeared.
    assert_eq!(fs::read(&blocker).unwrap(), b"ORIGINAL");
}

// ----- J-5: full-scan load reads every .md under nodes/ ----------------------

#[test]
fn full_scan_load_reads_all_nodes() {
    let root = TempDir::new().unwrap();
    let store = Store::open(root.path());
    for i in 0..7 {
        store.persist(&doc_with_id(Id::new(), i, &format!("n{i}"))).unwrap();
    }
    // A stray non-.md file under nodes/ is ignored.
    let stray = root.path().join(layout::NODES_DIR).join("README.txt");
    fs::write(&stray, b"not a node").unwrap();

    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 7);
}

// ----- J-6: gix stage + commit; status reflects changes ----------------------

#[test]
fn gix_stage_commit_and_status() {
    let dir = TempDir::new().unwrap();
    let repo = Repo::init(dir.path()).expect("init repo");

    // Empty repo, no commits: clean.
    assert!(repo.is_clean().unwrap());

    // Add a node file, then commit: status goes dirty → clean.
    let store = Store::open(dir.path());
    store.persist(&doc_with_id(Id::new(), 1, "first")).unwrap();
    assert!(!repo.is_clean().unwrap(), "uncommitted file is dirty");

    repo.commit_all("add first node").expect("commit");
    assert!(repo.is_clean().unwrap(), "clean right after commit");

    // A new change is detected again.
    store.persist(&doc_with_id(Id::new(), 2, "second")).unwrap();
    assert!(!repo.is_clean().unwrap(), "new file is dirty");
    let commit_id = repo.commit_all("add second node").expect("commit 2");
    assert_eq!(commit_id.len(), 40, "sha1 hex commit id");
    assert!(repo.is_clean().unwrap());
}

// ----- J-7: odm.toml via confyg layered search ------------------------------

#[test]
fn config_layered_load_from_start_dir() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("odm.toml"),
        "author_name = \"Ada\"\nauthor_email = \"ada@example.com\"\n",
    )
    .unwrap();

    let cfg = StoreConfig::load(dir.path()).expect("load config");
    assert_eq!(cfg.author_name, "Ada");
    assert_eq!(cfg.author_email, "ada@example.com");
}

#[test]
fn config_layered_load_defaults_when_absent() {
    let dir = TempDir::new().unwrap();
    let cfg = StoreConfig::load(dir.path()).expect("load defaults");
    assert_eq!(cfg, StoreConfig::default());
}

// ----- J-8: missing nodes/ dir self-heals -----------------------------------

#[test]
fn missing_dir_selfheal_load_then_write() {
    let root = TempDir::new().unwrap();
    let store = Store::open(root.path());

    // No nodes/ dir yet: empty load, not an error.
    assert!(store.load_all().unwrap().is_empty());
    assert!(!root.path().join(layout::NODES_DIR).exists());

    // First write creates the directory tree.
    let doc = doc_with_id(Id::new(), 1, "first");
    let path = store.persist(&doc).unwrap();
    assert!(path.exists());
    assert!(root.path().join(layout::NODES_DIR).exists());
    assert_eq!(store.load_all().unwrap().len(), 1);
}
