//! The incremental drift reconcile (arc05 slice07, ODD-0019 §3.2): fingerprint
//! each input-derived fact's inputs, re-probe **only** what changed, carry the
//! rest from the persisted [`DriftSnapshot`], and leave volatile facts untouched
//! (stamped "last checked").
//!
//! This is the freshness core: cost proportional to the change, not the corpus.
//! It is the **library mechanism** — slice08 wires it into `reconcile_views` and
//! renders honest staleness. It does **not** touch `odm-index`; the input
//! fingerprint mirrors the ODD-0014 racy-correct *pattern* over the file probe's
//! own [`hex_sha256`](crate::file::hex_sha256), not the index's record machinery.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use odm_core::{ProbeClass, ProbeSpec};
use odm_store::{Store, StoreError};

use crate::file::hex_sha256;
use crate::runner::evaluate_spec;
use crate::snapshot::{DriftSnapshot, FactEntry, FactState, InputFingerprint, Load};

/// How a reconcile treats each fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Explicit `odm reconcile`: re-probe **every** fact (input-derived *and*
    /// volatile), refreshing all fingerprints and last-checked stamps.
    Full,
    /// A bare command's cheap pass: re-probe an input-derived fact **only** when
    /// an input changed (else carry its cached outcome); **do not run** volatile
    /// facts — carry their last outcome + last-checked stamp.
    Incremental,
}

/// An error running the incremental reconcile.
#[derive(Debug, thiserror::Error)]
pub enum IncrementalError {
    /// The corpus could not be loaded from the store.
    #[error("loading the corpus for drift reconcile")]
    Corpus(#[source] StoreError),
    /// The drift snapshot could not be read or written.
    #[error("drift snapshot I/O")]
    Snapshot(#[from] crate::snapshot::DriftError),
}

/// Explicit full reconcile: re-probe every fact and persist. The substrate for
/// `odm reconcile` (slice08).
///
/// # Errors
///
/// Returns [`IncrementalError`] if the corpus or snapshot I/O fails.
pub fn reconcile_full(
    store: &Store,
    snapshot_path: &Path,
) -> Result<DriftSnapshot, IncrementalError> {
    reconcile(store, snapshot_path, Mode::Full)
}

/// Incremental reconcile: re-probe only changed input-derived facts, carry the
/// rest (and all volatile facts) from the prior snapshot. The cheap
/// every-command path (slice08).
///
/// # Errors
///
/// Returns [`IncrementalError`] if the corpus or snapshot I/O fails.
pub fn reconcile_incremental(
    store: &Store,
    snapshot_path: &Path,
) -> Result<DriftSnapshot, IncrementalError> {
    reconcile(store, snapshot_path, Mode::Incremental)
}

/// The reconcile core (see [`Mode`]). Loads the prior snapshot (self-healing:
/// missing/corrupt → empty), processes every node's declared facts, and persists
/// the updated snapshot atomically.
///
/// # Errors
///
/// Returns [`IncrementalError`] if the corpus or snapshot I/O fails.
pub fn reconcile(
    store: &Store,
    snapshot_path: &Path,
    mode: Mode,
) -> Result<DriftSnapshot, IncrementalError> {
    let prior = match DriftSnapshot::load(snapshot_path)? {
        Load::Loaded(snapshot) => snapshot,
        Load::RebuildNeeded(_) => DriftSnapshot::default(),
    };
    let documents = store.load_all().map_err(IncrementalError::Corpus)?;
    let root = store.root();
    let now = now_unix_secs();

    let mut facts = Vec::new();
    for document in &documents {
        let frontmatter = document.frontmatter();
        let node_id = frontmatter.id();
        for fact in frontmatter.desired_facts() {
            let prior_entry = prior.entry(node_id, &fact.id);
            let entry = match fact.probe.class() {
                ProbeClass::InputDerived => {
                    process_input_derived(root, node_id, fact, &fact.probe, prior_entry, mode, now)
                }
                ProbeClass::Volatile => {
                    match process_volatile(root, node_id, fact, &fact.probe, prior_entry, mode, now)
                    {
                        Some(entry) => entry,
                        // Incremental + never-checked volatile → no entry yet
                        // (an explicit `reconcile` will check it first).
                        None => continue,
                    }
                }
            };
            facts.push(entry);
        }
    }

    let snapshot = DriftSnapshot::new(now, facts);
    snapshot.persist(snapshot_path)?;
    Ok(snapshot)
}

/// Processes one input-derived fact: in `Full`, always re-probe; in
/// `Incremental`, re-probe only if an input changed since the prior snapshot,
/// else carry the cached entry.
fn process_input_derived(
    root: &Path,
    node_id: odm_core::Id,
    fact: &odm_core::DesiredFact,
    spec: &ProbeSpec,
    prior: Option<&FactEntry>,
    mode: Mode,
    now: i64,
) -> FactEntry {
    // Carry the cached entry when incremental and no input changed.
    if mode == Mode::Incremental {
        if let Some(prior_entry) = prior {
            if let FactState::InputDerived { inputs } = &prior_entry.state {
                if !inputs.iter().any(|fp| input_changed(fp, root)) {
                    return prior_entry.clone();
                }
            }
        }
    }
    // Re-probe: evaluate + re-capture the input fingerprints.
    let outcome = evaluate_spec(spec, root);
    let inputs = spec.inputs().iter().map(|rel| capture_input(root, rel, now)).collect();
    FactEntry {
        node_id,
        fact_id: fact.id.clone(),
        outcome,
        state: FactState::InputDerived { inputs },
    }
}

/// Processes one volatile fact: in `Full`, run it and stamp `last_checked = now`;
/// in `Incremental`, carry the prior entry unchanged (not run), or `None` if it
/// was never checked.
fn process_volatile(
    root: &Path,
    node_id: odm_core::Id,
    fact: &odm_core::DesiredFact,
    spec: &ProbeSpec,
    prior: Option<&FactEntry>,
    mode: Mode,
    now: i64,
) -> Option<FactEntry> {
    match mode {
        Mode::Full => {
            let outcome = evaluate_spec(spec, root);
            Some(FactEntry {
                node_id,
                fact_id: fact.id.clone(),
                outcome,
                state: FactState::Volatile { last_checked: now },
            })
        }
        // Not run — carry the prior outcome + last-checked stamp verbatim.
        Mode::Incremental => prior.cloned(),
    }
}

/// Captures an input's racy-correct fingerprint at `captured_at`: `lstat` for
/// size + mtime, and the content hash when the file exists (the authority on the
/// racy window). A missing input is a valid captured state.
pub(crate) fn capture_input(root: &Path, rel_path: &str, captured_at: i64) -> InputFingerprint {
    let full = root.join(rel_path);
    match std::fs::symlink_metadata(&full) {
        Ok(meta) => {
            let content_hash = std::fs::read(&full).ok().map(|bytes| hex_sha256(&bytes));
            InputFingerprint {
                rel_path: rel_path.to_string(),
                exists: true,
                size: meta.len(),
                mtime_secs: mtime_secs(&meta),
                content_hash,
                captured_at,
            }
        }
        Err(_) => InputFingerprint {
            rel_path: rel_path.to_string(),
            exists: false,
            size: 0,
            mtime_secs: 0,
            content_hash: None,
            captured_at,
        },
    }
}

/// Whether an input changed since its fingerprint was captured — racy-correct
/// (ODD-0014 §3.2, **never stat-only**): the cheap size/mtime signal decides the
/// common case, but an input still in the racy window (`mtime_secs >=
/// captured_at`) is re-hashed and the **hash is the authority**, catching a
/// same-tick, same-size edit that stat alone would miss (the C2 failure).
pub(crate) fn input_changed(stored: &InputFingerprint, root: &Path) -> bool {
    let full = root.join(&stored.rel_path);
    let meta = match std::fs::symlink_metadata(&full) {
        Ok(meta) => Some(meta),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        // Can't stat (permission, etc.): treat as changed to force a re-probe
        // rather than trust a stale cache.
        Err(_) => return true,
    };
    let Some(meta) = meta else {
        // Missing now: changed iff it existed at capture.
        return stored.exists;
    };
    if !stored.exists {
        return true; // absent → present.
    }
    let size = meta.len();
    let mtime = mtime_secs(&meta);
    if size != stored.size || mtime != stored.mtime_secs {
        return true; // cheap signal differs.
    }
    // Cheap signal matches; if racy, the hash is the authority.
    if mtime >= stored.captured_at {
        match (std::fs::read(&full), &stored.content_hash) {
            (Ok(bytes), Some(stored_hash)) => &hex_sha256(&bytes) != stored_hash,
            // Couldn't re-read, or nothing to compare against → conservatively changed.
            _ => true,
        }
    } else {
        false // cheap match + outside the racy window → trust stat.
    }
}

/// Current Unix time in whole seconds (0 before the epoch — unreachable).
fn now_unix_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Whole-second mtime of `meta` (0 if unavailable).
fn mtime_secs(meta: &std::fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // K-2: a same-tick, same-size input edit is caught (would be missed
    // stat-only). Deterministic: capture with `captured_at` == the file's mtime
    // so the input is guaranteed to be in the racy window.
    #[test]
    fn input_fingerprint_catches_racy_same_size_edit() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("in.txt"), b"AAAA").unwrap();
        let mtime = mtime_secs(&std::fs::symlink_metadata(dir.path().join("in.txt")).unwrap());

        // Capture in the racy window (captured_at == mtime).
        let fp = capture_input(dir.path(), "in.txt", mtime);
        assert!(fp.exists && fp.size == 4 && fp.content_hash.is_some());
        assert!(!input_changed(&fp, dir.path()), "unedited input is unchanged");

        // Same-size, same-second edit: stat is identical, but the hash differs.
        std::fs::write(dir.path().join("in.txt"), b"BBBB").unwrap();
        let post = std::fs::symlink_metadata(dir.path().join("in.txt")).unwrap();
        assert_eq!(post.len(), 4, "same size");
        assert!(input_changed(&fp, dir.path()), "racy same-size edit must be caught");
    }

    // A non-racy input (captured well after its mtime) is trusted by the cheap
    // signal; a real edit (mtime advances) is caught.
    #[test]
    fn input_fingerprint_trusts_stat_outside_racy_window() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("in.txt"), b"AAAA").unwrap();
        let mtime = mtime_secs(&std::fs::symlink_metadata(dir.path().join("in.txt")).unwrap());
        // Captured far in the future → not racy (mtime < captured_at).
        let fp = capture_input(dir.path(), "in.txt", mtime + 10_000);
        assert!(!input_changed(&fp, dir.path()), "clean, non-racy input is unchanged");

        // A missing input that existed at capture is a change.
        std::fs::remove_file(dir.path().join("in.txt")).unwrap();
        assert!(input_changed(&fp, dir.path()), "deleted input is a change");
    }
}
