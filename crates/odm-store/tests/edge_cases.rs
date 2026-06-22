//! Edge-case and error-path tests for the store layer.

use std::fs;
use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_store::{Repo, Store, StoreConfig, StoreError, atomic, layout};
use tempfile::TempDir;

fn sample_doc(id: Id) -> Document {
    let fm = Frontmatter::new(
        id,
        1,
        NodeType::Note,
        "n",
        NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
        NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
        Origin::Planned,
    );
    Document::new(fm, "body\n")
}

// ----- store load error paths ----------------------------------------------

#[test]
fn load_missing_node_is_io_error() {
    let root = TempDir::new().unwrap();
    let store = Store::open(root.path());
    let err = store.load(Id::new()).unwrap_err();
    assert!(matches!(err, StoreError::Io { .. }));
    // Display carries the path context.
    assert!(err.to_string().contains("io error"));
}

#[test]
fn load_corrupt_node_is_frontmatter_error() {
    let root = TempDir::new().unwrap();
    let store = Store::open(root.path());
    let id = Id::new();
    store.persist(&sample_doc(id)).unwrap();
    // Corrupt the persisted file (no frontmatter fence).
    fs::write(store.path_of(id), b"not a node file").unwrap();
    let err = store.load(id).unwrap_err();
    assert!(matches!(err, StoreError::Frontmatter { .. }));
    assert!(err.to_string().contains("frontmatter error"));
}

#[test]
fn load_all_propagates_parse_error() {
    let root = TempDir::new().unwrap();
    let store = Store::open(root.path());
    // A .md file under nodes/ that isn't valid frontmatter.
    let bad = root.path().join(layout::NODES_DIR).join("2026").join("06");
    fs::create_dir_all(&bad).unwrap();
    fs::write(bad.join("garbage.md"), b"no fence").unwrap();
    assert!(matches!(store.load_all(), Err(StoreError::Frontmatter { .. })));
}

#[cfg(unix)]
#[test]
fn load_all_surfaces_unreadable_dir() {
    use std::os::unix::fs::PermissionsExt;

    let root = TempDir::new().unwrap();
    let store = Store::open(root.path());
    let id = Id::new();
    store.persist(&sample_doc(id)).unwrap();
    let month_dir = store.path_of(id).parent().unwrap().to_path_buf();

    // Make the month directory unreadable so the scan yields an error.
    fs::set_permissions(&month_dir, fs::Permissions::from_mode(0o000)).unwrap();
    let result = store.load_all();
    // Restore before asserting so the TempDir can clean up regardless.
    fs::set_permissions(&month_dir, fs::Permissions::from_mode(0o755)).unwrap();

    assert!(matches!(result, Err(StoreError::Io { .. })));
}

// ----- git edge cases -------------------------------------------------------

#[test]
fn open_existing_repo_and_work_dir() {
    let dir = TempDir::new().unwrap();
    Repo::init(dir.path()).unwrap();
    let repo = Repo::open(dir.path()).expect("open existing repo");
    assert!(repo.work_dir().is_some());
}

#[test]
fn open_non_repo_is_git_error() {
    let dir = TempDir::new().unwrap();
    let err = Repo::open(dir.path()).unwrap_err();
    assert!(matches!(err, StoreError::Git(_)));
    assert!(err.to_string().contains("git error"));
}

#[test]
fn commit_skips_empty_subdirectories() {
    let dir = TempDir::new().unwrap();
    let repo = Repo::init(dir.path()).unwrap();
    // An empty directory in the worktree must not break tree-building, and git
    // does not track it.
    fs::create_dir(dir.path().join("empty")).unwrap();
    fs::write(dir.path().join("file.md"), b"hi").unwrap();
    repo.commit_all("with empty dir").expect("commit");
    assert!(repo.is_clean().unwrap());
}

// ----- atomic write: rename-failure cleanup ---------------------------------

#[test]
fn atomic_write_rename_failure_cleans_temp() {
    let dir = TempDir::new().unwrap();
    // Target path is a *non-empty directory*, so the final rename fails.
    let target = dir.path().join("occupied");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("inner"), b"keep").unwrap();

    let err = atomic::write(&target, b"data");
    assert!(err.is_err(), "rename over a non-empty dir must fail");
    // The temp sibling was cleaned up, and the directory's content is intact.
    assert!(fs::read(target.join("inner")).unwrap() == b"keep");
    let temps: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
        .collect();
    assert!(temps.is_empty(), "temp file must be cleaned up on failure");
}

#[cfg(unix)]
#[test]
fn atomic_write_temp_failure_in_readonly_dir() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("ro");
    fs::create_dir(&sub).unwrap();
    // Make the directory non-writable so creating the temp file fails.
    fs::set_permissions(&sub, fs::Permissions::from_mode(0o555)).unwrap();

    let err = atomic::write(&sub.join("x.md"), b"data");
    assert!(matches!(err, Err(StoreError::Io { .. })));

    // Restore write perms so the TempDir can be cleaned up.
    fs::set_permissions(&sub, fs::Permissions::from_mode(0o755)).unwrap();
}

// ----- config: repo-root layer + malformed ----------------------------------

#[test]
fn config_found_at_repo_root_from_subdir() {
    let root = TempDir::new().unwrap();
    // Mark this as a repo root and place odm.toml there.
    fs::create_dir(root.path().join(".git")).unwrap();
    fs::write(root.path().join("odm.toml"), "author_name = \"RepoRoot\"\n").unwrap();
    let subdir = root.path().join("nodes").join("2026");
    fs::create_dir_all(&subdir).unwrap();

    let cfg = StoreConfig::load(&subdir).expect("load from subdir");
    assert_eq!(cfg.author_name, "RepoRoot");
    // Unset field falls back to default.
    assert_eq!(cfg.author_email, StoreConfig::default().author_email);
}

#[test]
fn config_malformed_is_config_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("odm.toml"), "this is := not valid toml ==").unwrap();
    let err = StoreConfig::load(dir.path()).unwrap_err();
    assert!(matches!(err, StoreError::Config(_)));
}

// ----- round-trip sanity for a known id path --------------------------------

#[test]
fn persist_uses_id_derived_path() {
    let root = TempDir::new().unwrap();
    let store = Store::open(root.path());
    let id = Id::from_str("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
    let path = store.persist(&sample_doc(id)).unwrap();
    assert!(path.ends_with(layout::relative_path(id)));
    assert!(path.exists());
}
