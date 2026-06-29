//! `odm-index` — the persisted, derived **stat-cache** for odm (Arc 04).
//!
//! The index makes "which files define which nodes?" and metadata filter/sort
//! fast at scale **without a database, an FTS engine, or a daemon** (ODD-0014):
//! a sorted, versioned, checksummed snapshot under `.odm/` that the first run
//! builds and subsequent runs `lstat`-compare against, touching only the delta.
//! It is **derived and rebuildable** — it carries no authority; a corrupt or
//! stale file is detected on load and rebuilt from the node files.
//!
//! This first slice lands the **on-disk foundation** only:
//!
//! - [`IndexRecord`] — the per-node entry (identity, stat-cache fields,
//!   fingerprints, extracted metadata);
//! - [`Snapshot`] — the file format (header + `postcard` body + trailing
//!   checksum), with crash-safe atomic persistence (reusing
//!   [`odm_store::atomic::write`]) and self-healing load
//!   ([`Load::RebuildNeeded`]).
//!
//! Building records from a corpus walk (slice02), warm-path change detection
//! (slice03), and the in-memory filter/sort maps (slice04) build on this.

#![deny(missing_docs)]

pub mod adapter;
pub mod build;
mod hash;
pub mod maps;
pub mod record;
pub mod snapshot;
pub mod warm;

use std::path::{Path, PathBuf};

pub use adapter::frontmatters_from_records;
pub use build::{BuildError, build, build_records};
pub use maps::IndexMaps;
pub use record::{
    Decomposition, Digest, EdgeKind, EdgeQualifier, EdgeRef, GateState, IndexRecord, SupersedeKind,
};
pub use snapshot::{
    FORMAT_VERSION, HashAlgo, Header, IndexError, Load, MAGIC, RebuildReason, Snapshot,
};
pub use warm::{Delta, Reconciliation, WarmError, reconcile};

/// The index snapshot's path under a store root: `<root>/.odm/index`.
///
/// The `.odm/` directory holds derived, gitignored CLI state (the context file
/// lives there too); the index is never the source of truth.
#[must_use]
pub fn default_index_path(root: &Path) -> PathBuf {
    root.join(".odm").join("index")
}
