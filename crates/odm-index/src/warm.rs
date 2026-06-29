//! The warm path: reconcile an existing snapshot against the corpus at delta
//! cost, **correctly** under the racy-git case (ODD-0014 §3.2, the algorithm;
//! §2.3, the racy lesson + same-size-edit defense).
//!
//! This is the correctness core of A4. The cheap signal — whole-second `mtime` +
//! `size` + `mode` — decides the common case, but a file whose `mtime_secs >=`
//! the index timestamp is **racily clean**: `stat` cannot be trusted (the edit
//! and the index write could share a clock tick), so its content is hashed and
//! the **hash is the authority** (§4: stat-only is a correctness bug; nanosecond
//! mtime is not a correctness signal). On write, still-racy entries have their
//! recorded `size` zeroed (git's belt-and-suspenders) so a future same-tick,
//! same-size edit forces a cheap mismatch.
//!
//! Everything is **reused**: [`Snapshot::load`]/[`Snapshot::persist`] for I/O,
//! [`build`](crate::build::build) for the rebuild path,
//! [`build_one`](crate::build::build_one) for re-parsing a NEW/CHANGED file, and
//! [`Store::node_paths`] for the walk. The warm path adds only *classification*.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use odm_core::Id;
use odm_store::Store;

use crate::build::{self, BuildError, build_one, ino_mode, mtime_parts, now_unix_secs};
use crate::hash::sha256;
use crate::record::IndexRecord;
use crate::snapshot::{Load, Snapshot};

/// What a warm reconcile changed, for downstream consumers (slice04/05/07).
///
/// New/changed/deleted are carried as **id sets** (slice07's early cutoff acts on
/// exactly these); `clean` is a **count** (the large, do-nothing majority — its
/// ids carry no downstream signal, so retaining them would be waste). `rebuilt`
/// is `true` when the snapshot was absent/corrupt/stale and rebuilt cold.
///
/// `meta_changed` is the **semantic** subset of `changed` (ODD-0014 §2.4/§2.5): a
/// changed record whose new `meta_hash` differs from the prior record's. The
/// complement — `changed` ids **not** in `meta_changed` — are *body-only* edits
/// (the file changed, its meaning did not), the early-cutoff signal slice07 reads.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Delta {
    /// A full cold rebuild happened (load returned `RebuildNeeded`).
    pub rebuilt: bool,
    /// Ids of files with no prior record (inserted).
    pub new: Vec<Id>,
    /// Ids of files whose content changed (re-parsed + updated).
    pub changed: Vec<Id>,
    /// Ids (⊆ `changed`) whose *meaning* changed (`meta_hash` differs from prior).
    /// `changed` minus this set is the body-only edits (early cutoff, §2.4).
    pub meta_changed: Vec<Id>,
    /// Ids of records whose file is gone (removed).
    pub deleted: Vec<Id>,
    /// Count of files that were clean (skipped; record reused).
    pub clean: usize,
}

impl Delta {
    /// Whether the reconcile produced any change (and therefore persisted).
    #[must_use]
    pub fn is_changed(&self) -> bool {
        self.rebuilt || !self.new.is_empty() || !self.changed.is_empty() || !self.deleted.is_empty()
    }
}

/// The outcome of a warm reconcile: the up-to-date snapshot (persisted iff it
/// changed) plus the [`Delta`] describing what moved.
#[derive(Debug, Clone)]
pub struct Reconciliation {
    /// The current snapshot. Persisted to disk iff [`Delta::is_changed`].
    pub snapshot: Snapshot,
    /// What changed.
    pub delta: Delta,
}

/// An error reconciling the index.
#[derive(Debug, thiserror::Error)]
pub enum WarmError {
    /// Loading or persisting the snapshot failed.
    #[error("index snapshot I/O")]
    Index(#[from] crate::snapshot::IndexError),
    /// The corpus walk failed.
    #[error("walking the corpus")]
    Walk(#[source] odm_store::StoreError),
    /// A node file could not be stat'd.
    #[error("statting node file {path}")]
    Stat {
        /// The file that failed to stat.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A node file could not be read (for the racy content-hash).
    #[error("reading node file {path}")]
    Read {
        /// The file that failed to read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Building/re-parsing a record failed.
    #[error("building an index record")]
    Build(#[from] BuildError),
}

/// Reconciles the snapshot at `index_path` against `store`'s corpus (ODD-0014
/// §3.2). Loads the prior snapshot (rebuilding cold if it is absent, corrupt, or
/// stale), classifies every node file, removes records for deleted files, and —
/// **only if anything changed** — re-stamps `index_timestamp = now`, applies the
/// same-size-edit defense, and persists. Returns the current snapshot + a
/// [`Delta`].
///
/// # Errors
///
/// Returns a [`WarmError`] if the walk, a stat/read, a rebuild/re-parse, or the
/// persist fails.
pub fn reconcile(store: &Store, index_path: &Path) -> Result<Reconciliation, WarmError> {
    let prior = match Snapshot::load(index_path)? {
        Load::Loaded(snapshot) => snapshot,
        // Absent / corrupt / stale → full cold rebuild (not an error).
        Load::RebuildNeeded(_) => return rebuild_cold(store, index_path),
    };

    let root = store.root();
    let paths = store.node_paths().map_err(WarmError::Walk)?;

    // Cache the prior records by their on-disk path (a node's filename is its id,
    // so path identity == record identity); consume them as we match.
    let mut cache: HashMap<String, IndexRecord> =
        prior.records.into_iter().map(|r| (r.rel_path.clone(), r)).collect();

    let mut next: Vec<IndexRecord> = Vec::with_capacity(paths.len());
    let mut delta = Delta::default();

    for path in &paths {
        let rel_path = path.strip_prefix(root).unwrap_or(path).to_string_lossy().into_owned();
        let meta = std::fs::symlink_metadata(path)
            .map_err(|source| WarmError::Stat { path: path.clone(), source })?;

        match cache.remove(&rel_path) {
            None => {
                // NEW: no cached record.
                let record = build_one(root, path)?;
                delta.new.push(record.id);
                next.push(record);
            }
            Some(record) => {
                let (mtime_secs, _) = mtime_parts(&meta);
                let (_, mode) = ino_mode(&meta);
                let cheap_differs = meta.len() != record.size
                    || mtime_secs != record.mtime_secs
                    || mode != record.mode;

                if cheap_differs {
                    // CHANGED (cheap signal): re-read + re-hash + re-parse.
                    let updated = build_one(root, path)?;
                    note_change(&mut delta, &record, &updated);
                    next.push(updated);
                } else if mtime_secs >= prior.index_timestamp {
                    // RACILY CLEAN: stat cannot be trusted — the hash is the authority.
                    let bytes = std::fs::read(path)
                        .map_err(|source| WarmError::Read { path: path.clone(), source })?;
                    if sha256(&bytes) == record.content_hash {
                        delta.clean += 1;
                        next.push(record);
                    } else {
                        let updated = build_one(root, path)?;
                        note_change(&mut delta, &record, &updated);
                        next.push(updated);
                    }
                } else {
                    // CLEAN: cheap signal matches and the file is older than the
                    // index stamp — skip, reuse the record, no re-read/re-parse.
                    delta.clean += 1;
                    next.push(record);
                }
            }
        }
    }

    // DELETED: any cached record not matched by a walked file.
    delta.deleted = cache.into_values().map(|r| r.id).collect();
    delta.deleted.sort_unstable();

    next.sort_by_key(|r| r.id);

    if delta.is_changed() {
        let stamp = now_unix_secs();
        zero_racy_sizes(&mut next, stamp);
        let snapshot = Snapshot::new(stamp, next);
        snapshot.persist(index_path)?;
        Ok(Reconciliation { snapshot, delta })
    } else {
        // No change → no rewrite: keep the prior stamp, persist nothing.
        let snapshot = Snapshot::new(prior.index_timestamp, next);
        Ok(Reconciliation { snapshot, delta })
    }
}

/// The full cold rebuild taken when the snapshot is absent/corrupt/stale: build
/// from the corpus, persist, and report every record as `new` under `rebuilt`.
fn rebuild_cold(store: &Store, index_path: &Path) -> Result<Reconciliation, WarmError> {
    let snapshot = build::build(store)?;
    snapshot.persist(index_path)?;
    let new = snapshot.records.iter().map(|r| r.id).collect();
    Ok(Reconciliation { delta: Delta { rebuilt: true, new, ..Delta::default() }, snapshot })
}

/// Records a CHANGED file on the delta: always in `changed` (the record was
/// re-built), and additionally in `meta_changed` when its `meta_hash` differs
/// from the prior record's — i.e. its *meaning* changed, not just its bytes
/// (ODD-0014 §2.4/§2.5). A body-only edit lands in `changed` but **not**
/// `meta_changed`, which is the early-cutoff signal.
fn note_change(delta: &mut Delta, prior: &IndexRecord, updated: &IndexRecord) {
    delta.changed.push(updated.id);
    if updated.meta_hash != prior.meta_hash {
        delta.meta_changed.push(updated.id);
    }
}

/// The same-size-edit defense (ODD-0014 §2.3): zero the recorded `size` of any
/// entry still racy w.r.t. the new `stamp` (`mtime_secs >= stamp`), so a future
/// same-tick, same-size in-place edit trips the cheap `size` mismatch and is
/// re-hashed rather than trusted.
fn zero_racy_sizes(records: &mut [IndexRecord], stamp: i64) {
    for record in records.iter_mut() {
        if record.mtime_secs >= stamp {
            record.size = 0;
        }
    }
}
