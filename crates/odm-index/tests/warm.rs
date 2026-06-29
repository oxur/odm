//! Tests for the warm-path reconcile (Arc 04 slice03 — ODD-0014 §3.2).
//!
//! Mtimes are set explicitly (not left to wall-clock) so the racy `>=` cases are
//! deterministic. Test names carry the substrings the ledger Verify commands
//! filter on (`warm_rebuild_on_load_failure`, `warm_clean_file_skipped_not_reparsed`,
//! `warm_changed_file_updated`, `warm_new_file_inserted`, `warm_deleted_file_removed`,
//! `warm_racy_same_size_edit_caught`, `warm_racy_unchanged_stays_clean`,
//! `warm_racy_entries_size_zeroed_on_write`, `warm_restamp_and_persist_on_change`,
//! `warm_no_change_no_rewrite`, `warm_returns_delta`).

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_index::record::IndexRecord;
use odm_index::{Load, Snapshot, build_records, reconcile};
use odm_store::Store;
use tempfile::TempDir;

/// A stamp far enough in the future that every real file mtime is `< stamp`, so
/// files are non-racy (the cheap signal alone decides).
const FUTURE: i64 = 32_503_680_000; // ~year 3000

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 26).unwrap()
}

fn index_path(root: &Path) -> PathBuf {
    root.join(".odm").join("index")
}

fn node_doc(id: Id, body: &str) -> Document {
    Document::new(
        Frontmatter::new(id, 1, NodeType::Slice, "T", day(), day(), Origin::Planned),
        body,
    )
}

/// Seeds a node file at its id-derived path; returns the absolute path.
fn seed(store: &Store, id: Id, body: &str) -> PathBuf {
    store.persist(&node_doc(id, body)).expect("seed persist")
}

/// Builds a prior snapshot over the current corpus, stamps it with
/// `index_timestamp`, and persists it. Returns the persisted snapshot.
fn persist_prior(store: &Store, index_timestamp: i64) -> Snapshot {
    let records = build_records(store).expect("cold build");
    let snapshot = Snapshot::new(index_timestamp, records);
    snapshot.persist(&index_path(store.root())).expect("persist prior");
    snapshot
}

/// Sets a file's whole-second mtime (for deterministic racy/non-racy framing).
fn set_mtime_secs(path: &Path, secs: i64) {
    let t = UNIX_EPOCH + Duration::from_secs(secs as u64);
    OpenOptions::new().write(true).open(path).unwrap().set_modified(t).unwrap();
}

fn record_for(snapshot: &Snapshot, id: Id) -> &IndexRecord {
    snapshot.records.iter().find(|r| r.id == id).expect("a record for the id")
}

// ----- W-1: a missing/corrupt snapshot triggers a full cold rebuild ----------

#[test]
fn warm_rebuild_on_load_failure() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    seed(&store, Id::new(), "# T\nbody\n");
    seed(&store, Id::new(), "# T\nbody2\n");
    // No index file exists yet → RebuildNeeded(Missing) → cold rebuild.

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert!(r.delta.rebuilt, "a missing index rebuilds cold");
    assert_eq!(r.delta.new.len(), 2, "every node reported as new on rebuild");
    assert_eq!(r.snapshot.records.len(), 2);
    assert!(index_path(dir.path()).exists(), "the rebuilt index is persisted");
}

// ----- W-2: an unchanged (non-racy) file is clean — reused, not re-parsed ----

#[test]
fn warm_clean_file_skipped_not_reparsed() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();
    let path = seed(&store, id, "# T\nbody\n");
    set_mtime_secs(&path, 1_000_000); // well below FUTURE → non-racy
    let prior = persist_prior(&store, FUTURE);

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert_eq!(r.delta.clean, 1);
    assert!(r.delta.new.is_empty() && r.delta.changed.is_empty() && r.delta.deleted.is_empty());
    assert!(!r.delta.rebuilt);
    // No re-stamp ⇒ no rewrite; the record is reused byte-identically.
    assert_eq!(r.snapshot.index_timestamp, prior.index_timestamp);
    assert_eq!(record_for(&r.snapshot, id), record_for(&prior, id));
}

// ----- W-3: a changed file (cheap signal) is re-read + updated ---------------

#[test]
fn warm_changed_file_updated() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();
    let path = seed(&store, id, "# T\nshort\n");
    set_mtime_secs(&path, 1_000_000);
    let prior = persist_prior(&store, FUTURE);

    // A different-size edit → the cheap signal (size) trips CHANGED.
    std::fs::write(&path, node_doc(id, "# T\nmuch longer body now\n").emit().unwrap()).unwrap();

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert_eq!(r.delta.changed, vec![id]);
    assert_ne!(
        record_for(&r.snapshot, id).content_hash,
        record_for(&prior, id).content_hash,
        "the record was re-hashed"
    );
}

// ----- W-4: a new file is inserted -------------------------------------------

#[test]
fn warm_new_file_inserted() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let existing = Id::new();
    set_mtime_secs(&seed(&store, existing, "# T\na\n"), 1_000_000);
    persist_prior(&store, FUTURE);

    let added = Id::new();
    set_mtime_secs(&seed(&store, added, "# T\nb\n"), 1_000_000);

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert_eq!(r.delta.new, vec![added]);
    assert_eq!(r.snapshot.records.len(), 2);
}

// ----- W-5: a deleted file's record is removed -------------------------------

#[test]
fn warm_deleted_file_removed() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let keep = Id::new();
    let gone = Id::new();
    set_mtime_secs(&seed(&store, keep, "# T\na\n"), 1_000_000);
    let gone_path = seed(&store, gone, "# T\nb\n");
    set_mtime_secs(&gone_path, 1_000_000);
    persist_prior(&store, FUTURE);

    std::fs::remove_file(&gone_path).unwrap();

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert_eq!(r.delta.deleted, vec![gone]);
    assert!(r.snapshot.records.iter().all(|rec| rec.id == keep), "only the kept node remains");
}

// ----- W-6: the racy case — a same-size in-place edit is caught by the hash ---

#[test]
fn warm_racy_same_size_edit_caught() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();

    // Two valid node files of EQUAL byte length (bodies differ char-for-char).
    let a = node_doc(id, "# T\nAAAA\n").emit().unwrap();
    let b = node_doc(id, "# T\nBBBB\n").emit().unwrap();
    assert_eq!(a.len(), b.len(), "the edit must be same-size");

    let path = store.path_of(id);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, &a).unwrap();
    let mtime = build_records(&store).unwrap()[0].mtime_secs;
    // Frame it as racy: index_timestamp == the file's mtime ⇒ mtime_secs >= ts.
    persist_prior(&store, mtime);

    // Edit in place to equal-size B, then reset mtime so the cheap signal MATCHES
    // (same size, same mtime_secs, same mode). Stat-only would call this CLEAN.
    std::fs::write(&path, &b).unwrap();
    set_mtime_secs(&path, mtime);

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert_eq!(r.delta.changed, vec![id], "the content-hash fallback catches the racy edit");
    assert_eq!(r.delta.clean, 0, "it is NOT classified clean (would be, under stat-only)");
}

// ----- W-6b: a racy-but-unchanged file stays clean ---------------------------

#[test]
fn warm_racy_unchanged_stays_clean() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();
    seed(&store, id, "# T\nbody\n");
    let mtime = build_records(&store).unwrap()[0].mtime_secs;
    persist_prior(&store, mtime); // racy (mtime_secs >= index_timestamp)

    // Touch nothing; reconcile must hash, find the content identical, stay clean.
    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert_eq!(r.delta.clean, 1);
    assert!(r.delta.changed.is_empty(), "identical content under a racy stat stays clean");
}

// ----- W-7: same-size-edit defense — still-racy entries get size zeroed -------

#[test]
fn warm_racy_entries_size_zeroed_on_write() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();
    let path = seed(&store, id, "# T\nbody\n");
    set_mtime_secs(&path, 1_000_000);
    persist_prior(&store, FUTURE);

    // Push the file's mtime into the future (>= the new stamp `now`): the
    // reconcile re-stamps to now, so this entry is "still racy" on write and its
    // recorded size is zeroed (forcing a cheap mismatch next run). The mtime
    // change also trips CHANGED, which is what makes the write happen.
    set_mtime_secs(&path, FUTURE);

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert!(r.delta.is_changed(), "the future-mtime edit forces a write");
    assert_eq!(record_for(&r.snapshot, id).size, 0, "a still-racy entry's size is zeroed on write");

    // The zeroing is durable (re-loadable).
    match Snapshot::load(&index_path(dir.path())).unwrap() {
        Load::Loaded(s) => assert_eq!(record_for(&s, id).size, 0),
        Load::RebuildNeeded(r) => panic!("expected a clean reload, got {r:?}"),
    }
}

// ----- W-8: re-stamp + persist on change; no rewrite on a no-change run ------

#[test]
fn warm_restamp_and_persist_on_change() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();
    let path = seed(&store, id, "# T\nshort\n");
    set_mtime_secs(&path, 1_000_000);
    let prior = persist_prior(&store, FUTURE);

    std::fs::write(&path, node_doc(id, "# T\nlonger body\n").emit().unwrap()).unwrap();
    let r = reconcile(&store, &index_path(dir.path())).unwrap();

    assert!(r.delta.is_changed());
    assert_ne!(r.snapshot.index_timestamp, prior.index_timestamp, "re-stamped on change");
    // The persisted snapshot reflects the update.
    match Snapshot::load(&index_path(dir.path())).unwrap() {
        Load::Loaded(s) => assert_eq!(s, r.snapshot, "the updated snapshot was persisted"),
        Load::RebuildNeeded(reason) => panic!("expected a clean reload, got {reason:?}"),
    }
}

#[test]
fn warm_no_change_no_rewrite() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();
    let path = seed(&store, id, "# T\nbody\n");
    set_mtime_secs(&path, 1_000_000);
    let prior = persist_prior(&store, FUTURE);

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert!(!r.delta.is_changed(), "an unchanged corpus produces no change");
    // No re-stamp is the observable proxy for "did not rewrite the file".
    assert_eq!(r.snapshot.index_timestamp, prior.index_timestamp);
}

// ----- W-9: the reconcile returns a new/changed/deleted/clean delta ----------

#[test]
fn warm_returns_delta() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let clean = Id::new();
    let to_change = Id::new();
    let to_delete = Id::new();
    set_mtime_secs(&seed(&store, clean, "# T\nstays\n"), 1_000_000);
    let change_path = seed(&store, to_change, "# T\nshort\n");
    set_mtime_secs(&change_path, 1_000_000);
    let delete_path = seed(&store, to_delete, "# T\ngone\n");
    set_mtime_secs(&delete_path, 1_000_000);
    persist_prior(&store, FUTURE);

    // Mutate: change one (size), delete one, add one new.
    std::fs::write(&change_path, node_doc(to_change, "# T\na longer body here\n").emit().unwrap())
        .unwrap();
    std::fs::remove_file(&delete_path).unwrap();
    let added = Id::new();
    set_mtime_secs(&seed(&store, added, "# T\nfresh\n"), 1_000_000);

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    assert_eq!(r.delta.new, vec![added]);
    assert_eq!(r.delta.changed, vec![to_change]);
    assert_eq!(r.delta.deleted, vec![to_delete]);
    assert_eq!(r.delta.clean, 1, "the untouched node");
}

// ===== slice07: meta-changed vs. body-only in the delta =====================

// ----- E-1: the delta separates a meaning-change from a body-only edit -------

#[test]
fn delta_distinguishes_meta_changed_from_body_only() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let body_only = Id::new();
    let meta = Id::new();
    let pb = seed(&store, body_only, "# T\nshort\n");
    let pm = seed(&store, meta, "# T\nshort\n");
    set_mtime_secs(&pb, 1_000_000);
    set_mtime_secs(&pm, 1_000_000);
    persist_prior(&store, FUTURE);

    // body_only: same frontmatter, longer body → content changes, meaning does not.
    std::fs::write(&pb, node_doc(body_only, "# T\nmuch longer body now\n").emit().unwrap())
        .unwrap();
    // meta: a renamed node — `title` is a meta field, so `meta_hash` changes.
    let renamed = Document::new(
        Frontmatter::new(meta, 1, NodeType::Slice, "RENAMED", day(), day(), Origin::Planned),
        "# T\nshort\n",
    );
    std::fs::write(&pm, renamed.emit().unwrap()).unwrap();

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    // Both records were rebuilt (their files changed) …
    assert!(r.delta.changed.contains(&body_only) && r.delta.changed.contains(&meta));
    // … but only the renamed node's *meaning* changed.
    assert_eq!(r.delta.meta_changed, vec![meta], "only the meaning-change is meta-changed");
    assert!(!r.delta.meta_changed.contains(&body_only), "the body-only edit is not meta-changed");
}

// ----- E-4: a body-only edit still refreshes the record (content_hash + stat) -

#[test]
fn body_only_edit_refreshes_record() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let id = Id::new();
    let path = seed(&store, id, "# T\nshort\n");
    set_mtime_secs(&path, 1_000_000);
    let prior = persist_prior(&store, FUTURE);

    // Body-only edit: identical frontmatter, different body.
    std::fs::write(
        &path,
        node_doc(id, "# T\ntotally different prose, same meaning\n").emit().unwrap(),
    )
    .unwrap();

    let r = reconcile(&store, &index_path(dir.path())).unwrap();
    let before = record_for(&prior, id);
    let after = record_for(&r.snapshot, id);
    assert_ne!(
        after.content_hash, before.content_hash,
        "content_hash refreshed (the file changed)"
    );
    assert_eq!(after.meta_hash, before.meta_hash, "meta_hash unchanged (meaning is stable)");
    assert!(r.delta.changed.contains(&id), "the record was rebuilt …");
    assert!(!r.delta.meta_changed.contains(&id), "… but it is body-only, not meta-changed");
}
