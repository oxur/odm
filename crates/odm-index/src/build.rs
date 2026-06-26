//! The cold path: walk the corpus once, populate one [`IndexRecord`] per node
//! file, and assemble a [`Snapshot`] (ODD-0014 §2.1 FIRST-full).
//!
//! This is the O(corpus) pass paid on the first run; the cheap incremental warm
//! path (`lstat`-compare the delta) is slice03. It **reuses** every dependency
//! rather than re-deriving it: the corpus walk from
//! [`odm_store::Store::node_paths`], the frontmatter parse + accessors from
//! `odm-core`, the hashing from [`crate::hash`], and the snapshot/persist from
//! slice01 ([`Snapshot::persist`]). `odm-index` adds only the *assembly*.

use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use odm_core::frontmatter::{Dependency, Document, Edges, SupersedeKind as CoreSupersedeKind};
use odm_store::Store;
use serde::Serialize;

use crate::hash::sha256;
use crate::record::{Digest, EdgeKind, EdgeQualifier, EdgeRef, IndexRecord, SupersedeKind};
use crate::snapshot::Snapshot;

/// An error encountered while building the index from the corpus.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// The corpus walk (`node_paths`) failed.
    #[error("walking the corpus")]
    Walk(#[source] odm_store::StoreError),
    /// A node file could not be read.
    #[error("reading node file {path}")]
    Read {
        /// The file that failed to read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A node file's metadata (`lstat`) could not be read.
    #[error("statting node file {path}")]
    Stat {
        /// The file that failed to stat.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A node file's bytes are not valid UTF-8.
    #[error("node file {path} is not valid UTF-8")]
    Utf8 {
        /// The offending file.
        path: PathBuf,
    },
    /// A node file's frontmatter did not parse.
    #[error("parsing node file {path}")]
    Parse {
        /// The offending file.
        path: PathBuf,
        /// The underlying parse error.
        #[source]
        source: odm_core::frontmatter::FrontmatterError,
    },
    /// The metadata fingerprint could not be serialized (not reachable for data
    /// produced by this crate).
    #[error("serializing metadata for the meta-hash")]
    Encode(#[from] postcard::Error),
}

/// Builds the index records from a full corpus walk, sorted by id. Deterministic
/// (no clock) — the caller stamps the snapshot. A missing/empty `nodes/` yields
/// an empty set, not an error.
///
/// # Errors
///
/// Returns a [`BuildError`] for the first file that cannot be walked, read,
/// stat'd, decoded, or parsed.
pub fn build_records(store: &Store) -> Result<Vec<IndexRecord>, BuildError> {
    let root = store.root();
    let paths = store.node_paths().map_err(BuildError::Walk)?;

    let mut records = Vec::with_capacity(paths.len());
    for path in &paths {
        records.push(build_one(root, path)?);
    }
    records.sort_by_key(|r| r.id);
    Ok(records)
}

/// Builds one record for the node file at `path` (absolute), relative to `root`:
/// `lstat` + read + UTF-8 + parse + assemble. This is the single per-file build
/// seam — the cold build calls it for every file, and the warm path (slice03)
/// calls it for each NEW / CHANGED file (reuse, not a second copy).
///
/// # Errors
///
/// Returns a [`BuildError`] if the file cannot be read, stat'd, decoded, or
/// parsed.
pub(crate) fn build_one(root: &Path, path: &Path) -> Result<IndexRecord, BuildError> {
    let bytes = std::fs::read(path)
        .map_err(|source| BuildError::Read { path: path.to_path_buf(), source })?;
    let meta = std::fs::symlink_metadata(path)
        .map_err(|source| BuildError::Stat { path: path.to_path_buf(), source })?;
    let text =
        std::str::from_utf8(&bytes).map_err(|_| BuildError::Utf8 { path: path.to_path_buf() })?;
    let doc = Document::parse(text)
        .map_err(|source| BuildError::Parse { path: path.to_path_buf(), source })?;

    let rel_path = path.strip_prefix(root).unwrap_or(path).to_string_lossy().into_owned();
    build_record(&doc, rel_path, &meta, &bytes)
}

/// Builds a full snapshot from a cold corpus walk, stamped with the current time
/// (`index_timestamp` = now at build, per ODD-0014 §3.2). Persist it with
/// [`Snapshot::persist`].
///
/// # Errors
///
/// Returns a [`BuildError`] if the walk or any per-file step fails.
pub fn build(store: &Store) -> Result<Snapshot, BuildError> {
    let records = build_records(store)?;
    Ok(Snapshot::new(now_unix_secs(), records))
}

/// The current time as whole Unix seconds (the index stamp). Falls back to `0`
/// only if the clock is before the Unix epoch (unreachable in practice). Shared
/// with the warm path's re-stamp (slice03).
pub(crate) fn now_unix_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Assembles one record from a parsed document plus its on-disk stat + bytes.
fn build_record(
    doc: &Document,
    rel_path: String,
    meta: &Metadata,
    bytes: &[u8],
) -> Result<IndexRecord, BuildError> {
    let fm = doc.frontmatter();
    let (mtime_secs, mtime_nsec) = mtime_parts(meta);
    let (inode, mode) = ino_mode(meta);

    let node_type = fm.node_type();
    // `gates` = reached gate names (slice01 finding #1: odm has no scalar state).
    // `reached()` iterates the status BTreeMap, so this is already gate-name sorted.
    let gates: Vec<String> = fm.status().reached().map(|(name, _)| name.to_string()).collect();
    let tags: Vec<String> = fm.tags().to_vec();
    let edges = map_edges(fm.edges());
    let title = fm.name().to_string();
    let updated = fm.updated();

    let content_hash = sha256(bytes);
    let meta_hash = meta_fingerprint(&MetaInput {
        node_type,
        gates: &gates,
        tags: &tags,
        edges: &edges,
        title: &title,
    })?;

    Ok(IndexRecord {
        id: fm.id(),
        rel_path,
        mtime_secs,
        mtime_nsec,
        size: meta.len(),
        inode,
        mode,
        content_hash,
        meta_hash,
        node_type,
        gates,
        tags,
        edges,
        title,
        updated,
    })
}

/// The whole-second + sub-second mtime, from the portable `modified()` time.
/// Shared with the warm path's cheap-signal comparison (slice03).
pub(crate) fn mtime_parts(meta: &Metadata) -> (i64, u32) {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or((0, 0), |d| (d.as_secs() as i64, d.subsec_nanos()))
}

/// The inode and mode from a `lstat`. Unix carries both; elsewhere they are `0`
/// (the index treats them as opportunistic — they are never a correctness
/// signal on their own, ODD-0014 §4).
#[cfg(unix)]
pub(crate) fn ino_mode(meta: &Metadata) -> (u64, u32) {
    use std::os::unix::fs::MetadataExt as _;
    (meta.ino(), meta.mode())
}

/// Non-Unix fallback: no inode/mode available.
#[cfg(not(unix))]
pub(crate) fn ino_mode(_meta: &Metadata) -> (u64, u32) {
    (0, 0)
}

/// Maps a node's domain [`Edges`] into the index's [`EdgeRef`]s, in the canonical
/// frontmatter edge order, preserving each kind's qualifier (B-5).
fn map_edges(edges: &Edges) -> Vec<EdgeRef> {
    let mut out = Vec::new();

    if let Some(parent) = edges.part_of {
        out.push(EdgeRef { target: parent, kind: EdgeKind::PartOf, qualifier: None });
    }
    for dep in &edges.depends_on {
        let (target, qualifier) = match dep {
            Dependency::Bare(id) => (*id, None),
            Dependency::Qualified { node, satisfied_at } => {
                (*node, Some(EdgeQualifier::SatisfiedAt(satisfied_at.clone())))
            }
        };
        out.push(EdgeRef { target, kind: EdgeKind::DependsOn, qualifier });
    }
    for &target in &edges.blocked_by {
        out.push(EdgeRef { target, kind: EdgeKind::BlockedBy, qualifier: None });
    }
    for &target in &edges.verifies {
        out.push(EdgeRef { target, kind: EdgeKind::Verifies, qualifier: None });
    }
    for &target in &edges.consumes {
        out.push(EdgeRef { target, kind: EdgeKind::Consumes, qualifier: None });
    }
    for &target in &edges.affects {
        out.push(EdgeRef { target, kind: EdgeKind::Affects, qualifier: None });
    }
    if let Some(supersedes) = &edges.supersedes {
        let kind = match supersedes.kind {
            CoreSupersedeKind::Obsoletes => SupersedeKind::Obsoletes,
            CoreSupersedeKind::Updates => SupersedeKind::Updates,
        };
        out.push(EdgeRef {
            target: supersedes.node,
            kind: EdgeKind::Supersedes,
            qualifier: Some(EdgeQualifier::Supersede(kind)),
        });
    }
    for torn in &edges.tears {
        let target = match &torn.edge {
            Dependency::Bare(id) => *id,
            Dependency::Qualified { node, .. } => *node,
        };
        out.push(EdgeRef {
            target,
            kind: EdgeKind::Tears,
            qualifier: Some(EdgeQualifier::Because(torn.because.clone())),
        });
    }
    out
}

/// The semantic-metadata fields the `meta_hash` covers (B-6). Deliberately
/// **excludes** the stat fields, `content_hash`, and `updated` (bookkeeping):
/// the meta-hash is the *derived/meaning* fingerprint slice05's early cutoff
/// compares — a body-only edit must not change it.
#[derive(Serialize)]
struct MetaInput<'a> {
    node_type: odm_core::NodeType,
    gates: &'a [String],
    tags: &'a [String],
    edges: &'a [EdgeRef],
    title: &'a str,
}

/// SHA-256 over a **canonical** (order-stable) encoding of the extracted
/// metadata: `tags` and `edges` are sorted so the fingerprint is invariant to
/// incidental ordering, and `gates` is already gate-name sorted. Identical
/// metadata ⇒ identical `meta_hash` across runs.
fn meta_fingerprint(input: &MetaInput<'_>) -> Result<Digest, BuildError> {
    let mut tags = input.tags.to_vec();
    tags.sort();
    let mut edges = input.edges.to_vec();
    edges.sort();

    let canonical = MetaInput {
        node_type: input.node_type,
        gates: input.gates,
        tags: &tags,
        edges: &edges,
        title: input.title,
    };
    let bytes = postcard::to_allocvec(&canonical)?;
    Ok(sha256(&bytes))
}
