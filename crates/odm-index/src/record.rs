//! The index record: one entry per tracked node file (ODD-0014 §3.1).
//!
//! An [`IndexRecord`] is the persisted, derived stat-cache entry for a node — the
//! shape the cold-path build (slice02) fills from a corpus walk and the warm path
//! (slice03) `lstat`-compares against. It carries three layers:
//!
//! 1. **identity** — the node's [`Id`] (its filename stem, the canonical key) and
//!    its `rel_path` (for I/O and deletion detection);
//! 2. **stat-cache fields** — `mtime`/`size`/`inode`/`mode`, the cheap
//!    change-detection signal (git/hg/jj's pattern); and
//! 3. **fingerprints + extracted metadata** — a `content_hash` (raw bytes) and a
//!    `meta_hash` (normalized metadata, the early-cutoff key), plus the metadata
//!    needed for in-memory filter/sort without re-parsing.
//!
//! This module defines the *type only*; populating a record from a real file is
//! slice02's job.

use chrono::NaiveDate;
use odm_core::{Id, NodeType};
use serde::{Deserialize, Serialize};

/// The 32-byte digest produced by the index's hash algorithm (SHA-256 today).
pub type Digest = [u8; 32];

/// One tracked node file's index entry (ODD-0014 §3.1).
///
/// Two records are equal iff every field matches; the type round-trips through
/// the snapshot format byte-for-byte (`encode ∘ decode = identity`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexRecord {
    // --- identity ---
    /// The node's stable id — equal to the filename stem; the canonical key.
    pub id: Id,
    /// The node file's path relative to the store root
    /// (`nodes/YYYY/MM/<ULID>.md`), for I/O and deletion detection.
    pub rel_path: String,

    // --- stat cache (change detection; mirrors git/hg/jj) ---
    /// Whole-second mtime. The correctness-grade mtime signal (ODD-0014 §4: do
    /// **not** rely on `mtime_nsec` for correctness).
    pub mtime_secs: i64,
    /// Sub-second mtime, recorded for completeness / opportunistic compare only —
    /// never a proof of cleanliness (ODD-0014 §4).
    pub mtime_nsec: u32,
    /// File size in bytes (full `u64` — no git-style 32-bit truncation).
    pub size: u64,
    /// Device inode, a rename-vs-edit aid; `0` when unavailable (e.g. network FS).
    pub inode: u64,
    /// File mode (type / exec bits).
    pub mode: u32,

    // --- fingerprints ---
    /// Hash of the raw file bytes — the input fingerprint (did the file change?).
    pub content_hash: Digest,
    /// Hash of the normalized extracted metadata — the derived fingerprint that
    /// powers early cutoff (did the file's *meaning* change?), slice05.
    pub meta_hash: Digest,

    // --- extracted metadata for in-memory filter/sort (no re-parse needed) ---
    /// The node type (`project`/`arc`/`slice`/`odd`/…).
    pub node_type: NodeType,
    /// The node's state in the gate model: the names of the gates it has reached,
    /// sorted. odm has no single lifecycle `state` — status is a multi-gate
    /// vector — so this is the reached-gate set, which supports "filter by
    /// gate / state" (ODD-0014 §3.1's generic `state` mapped to odm's model).
    pub gates: Vec<String>,
    /// Free-form filter tags.
    pub tags: Vec<String>,
    /// The node's outgoing dependency-relevant edges (id + kind), for graph build.
    pub edges: Vec<EdgeRef>,
    /// The node's human title/name.
    pub title: String,
    /// The node's last-updated date.
    pub updated: NaiveDate,
}

/// One outgoing edge of a node, reduced to what the index needs: the target id
/// and the edge kind. The cold-path build (slice02) maps a node's frontmatter
/// edges into these.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeRef {
    /// The target node's id.
    pub target: Id,
    /// The kind of edge.
    pub kind: EdgeKind,
}

/// The odm edge taxonomy (ODD-0013 §3), as the index records it.
///
/// This deliberately **mirrors** `odm_core`'s edge kinds rather than reusing that
/// enum directly: the index is a derived cache with its own versioned on-disk
/// format, so it owns the wire representation of an edge kind and can evolve it
/// (behind the snapshot format-version) independently of the domain model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Containment: source is `part_of` the target (the hierarchy tree).
    PartOf,
    /// The source needs the target satisfied before it is ready.
    DependsOn,
    /// A hard external block on the source.
    BlockedBy,
    /// The source verifies the target.
    Verifies,
    /// The source consumes a concrete output of the target.
    Consumes,
    /// The source (a decision/doc) affects the target.
    Affects,
    /// Lineage: the source supersedes the target.
    Supersedes,
    /// A `depends_on` deliberately assumed to break a cycle.
    Tears,
}
