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
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use serde::{Deserialize, Serialize};

/// The 32-byte digest produced by the index's hash algorithm (SHA-256 today).
pub type Digest = [u8; 32];

/// A reached gate and the evidence level it was reached at — the index's view of
/// a node's status (slice04). The graph build reads the evidence to compute
/// evidence-leveled satisfaction off the index; carrying only gate *names* (as
/// slice02 did) was not enough. Reached-dates stay out (those are A7 telemetry,
/// not what satisfaction needs).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GateState {
    /// The reached gate's name.
    pub gate: String,
    /// The evidence level the gate was reached at.
    pub evidence: Evidence,
}

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
    /// The node's human number (for display; the `list` table column).
    pub number: u32,
    /// How the node arose (`planned`/`discovered`/`amendment`) — the rollup's
    /// provenance view (slice06). Reuses `odm_core::Origin` (string-serialized,
    /// no `skip_serializing_if`, so postcard-safe).
    pub origin: Origin,
    /// The node type (`project`/`arc`/`slice`/`odd`/…).
    pub node_type: NodeType,
    /// The node's state in the gate model: each reached gate with its evidence
    /// level, gate-name sorted. odm has no single lifecycle `state` — status is a
    /// multi-gate vector — so this is the reached-gate set (ODD-0014 §3.1's
    /// generic `state` mapped to odm's model). Carries evidence (slice04) so the
    /// graph build can compute satisfaction off the index.
    pub gates: Vec<GateState>,
    /// Free-form filter tags.
    pub tags: Vec<String>,
    /// Optional subsystem/component filter label.
    pub component: Option<String>,
    /// The node's outgoing dependency-relevant edges (id + kind), for graph build.
    pub edges: Vec<EdgeRef>,
    /// The parent's guarded "decomposition complete" assertion, if affirmed —
    /// `check`'s recomposition checks (drift / advanced-without) read it
    /// (slice06). Absent until affirmed.
    pub decomposed: Option<Decomposition>,
    /// The node's human title/name.
    pub title: String,
    /// The node's last-updated date.
    pub updated: NaiveDate,
}

/// A parent's affirmed "decomposition complete" assertion (ODD-0013 §4.5), as the
/// index records it: the date and the affirmed child set.
///
/// This **mirrors** `odm_core`'s `Decomposition` rather than embedding it: that
/// type's `children` field carries `#[serde(skip_serializing_if)]`, which
/// **desyncs a non-self-describing format like postcard** (the slice02 lesson) —
/// so the index owns a copy whose fields are always serialized.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Decomposition {
    /// The date the child set was affirmed complete.
    pub on: NaiveDate,
    /// The affirmed child ids (sorted + de-duplicated, as odm-core affirms them).
    pub children: Vec<Id>,
}

/// One outgoing edge of a node: the target id, the edge kind, and any
/// kind-specific qualifier. The cold-path build (slice02) maps a node's
/// frontmatter edges into these.
///
/// The `qualifier` preserves the edge details slice04's index-backed graph build
/// and `orient` need so they need not re-read frontmatter (slice02 B-5): a
/// `depends_on`'s `satisfied_at` gate (for satisfaction), a tear's `because`
/// (for the active-tears listing), and a supersede's kind. Edges with no
/// qualifier (most kinds) carry `None`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EdgeRef {
    /// The target node's id.
    pub target: Id,
    /// The kind of edge.
    pub kind: EdgeKind,
    /// The kind-specific qualifier, if any. Always serialized (postcard is a
    /// non-self-describing format — a skipped field would desync the stream).
    pub qualifier: Option<EdgeQualifier>,
}

/// The kind-specific extra carried by an [`EdgeRef`] — the edge detail beyond
/// `(target, kind)` that downstream consumers (graph build, `orient`) need.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EdgeQualifier {
    /// A `depends_on`'s satisfying gate (`satisfied_at`): the gate at which the
    /// dependency counts as satisfied.
    SatisfiedAt(String),
    /// A `supersedes` edge's kind (obsoletes vs. updates).
    Supersede(SupersedeKind),
    /// A tear's required rationale (`because`).
    Because(String),
}

/// Whether a `supersedes` edge replaces (obsoletes) or amends (updates) its
/// target. Mirrors `odm_core`'s supersede kind in the index's own wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupersedeKind {
    /// The old node is replaced.
    Obsoletes,
    /// The old node is amended (still relevant).
    Updates,
}

/// The odm edge taxonomy (ODD-0013 §3), as the index records it.
///
/// This deliberately **mirrors** `odm_core`'s edge kinds rather than reusing that
/// enum directly: the index is a derived cache with its own versioned on-disk
/// format, so it owns the wire representation of an edge kind and can evolve it
/// (behind the snapshot format-version) independently of the domain model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
