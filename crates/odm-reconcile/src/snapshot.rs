//! The persisted `.odm/` **drift snapshot** (arc05 slice07, ODD-0019 §3.3): a
//! versioned, checksummed, atomically-written file holding, per fact, the last
//! [`ProbeOutcome`] plus its freshness state (an input fingerprint for
//! input-derived facts, a last-checked timestamp for volatile ones).
//!
//! # Why a separate artifact (not the index)
//!
//! Drift is a **derived, time-varying** fact — it mirrors *probe results*, not
//! file state. The A4 index mirrors file state; embedding drift in it would bloat
//! every record and trip the adapter-fidelity invariant (ODD-0019 §7). So the
//! drift snapshot is a **sibling** of the index under `.odm/`, on the same
//! header + atomic-write discipline (which this module *mirrors* — it owns its
//! `MAGIC` `ODMDRIFT`, and does **not** import the index's record-coupled
//! `Snapshot`).
//!
//! # On-disk layout
//!
//! ```text
//! ┌────────────┬───────────────┬───────────────────┬──────────────┐
//! │ MAGIC (8)  │ version (u16) │ body (postcard)   │ checksum(32) │
//! └────────────┴───────────────┴───────────────────┴──────────────┘
//! └──────────────── checksummed prefix ─────────────┘
//! ```
//!
//! `MAGIC` + version sit at fixed offsets so a foreign/stale file is rejected
//! cheaply before deserialization; the trailing SHA-256 catches corruption/torn
//! writes. Any mismatch, truncation, or decode failure surfaces as a typed
//! [`RebuildReason`] — never a silent bad parse — and the caller rebuilds (the
//! snapshot carries no authority; it is derived and rebuildable).
//!
//! # Body format
//!
//! The body is **JSON** (not the index's postcard). Deliberate: [`ProbeOutcome`]
//! is an internally-tagged enum (`{"kind":…}`, the `reconcile/v1` contract) and
//! [`Id`] serializes as a string — both need a *self-describing* format, which
//! postcard is not. The index's records are postcard-friendly; drift's are not,
//! so the drift snapshot uses JSON under the same header/checksum/atomic
//! discipline. (A binary body would need postcard-specific DTOs; not worth it for
//! a small, derived, rebuildable cache.)

use std::path::{Path, PathBuf};

use odm_core::Id;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::probe::ProbeOutcome;

/// Magic sentinel at the start of every drift-snapshot file. Distinct from the
/// index's `ODMINDEX` so the two `.odm/` artifacts can never be confused.
pub const MAGIC: [u8; 8] = *b"ODMDRIFT";

/// The current drift-snapshot format version. Bumped on any incompatible change;
/// an older file loads as [`RebuildReason::VersionMismatch`] and is rebuilt.
pub const SNAPSHOT_VERSION: u16 = 1;

/// Byte length of the fixed header prefix (`MAGIC` + version).
const PREFIX_LEN: usize = 8 + 2;
/// Byte length of the trailing checksum.
const CHECKSUM_LEN: usize = 32;
/// The shortest possible well-formed file (prefix + checksum, empty body).
const MIN_LEN: usize = PREFIX_LEN + CHECKSUM_LEN;

/// A racy-correct fingerprint of one probe input file (ODD-0014 discipline):
/// size + whole-second mtime + a content hash. The hash is the authority on the
/// racy window; size/mtime are the cheap common-case signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputFingerprint {
    /// The input path, relative to the repo root.
    pub rel_path: String,
    /// Whether the input existed when captured (a missing input is a valid state
    /// whose appearance/disappearance is a change).
    pub exists: bool,
    /// Size in bytes (`0` when absent).
    pub size: u64,
    /// Whole-second mtime (Unix seconds; `0` when absent).
    pub mtime_secs: i64,
    /// Lowercase-hex SHA-256 of the contents — `Some` for an existing file
    /// (captured at probe time), `None` when the input is absent.
    pub content_hash: Option<String>,
    /// Unix seconds this fingerprint was captured — the **racy reference**: on a
    /// later check, an input whose `mtime_secs >= captured_at` is in the racy
    /// window and its content is re-hashed rather than trusted by stat (ODD-0014
    /// §3.2). Carried per-fingerprint so an unchanged entry keeps its own racy
    /// context regardless of the snapshot-wide write stamp.
    pub captured_at: i64,
}

/// A fact's freshness state in the snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactState {
    /// Input-derived: the outcome is cached until one of these inputs changes.
    InputDerived {
        /// The fingerprints of the fact's declared inputs at last probe time.
        inputs: Vec<InputFingerprint>,
    },
    /// Volatile: no filesystem signal — carries when it was last actually run
    /// (Unix seconds), for honest "last checked" staleness.
    Volatile {
        /// Unix seconds the fact was last actually probed.
        last_checked: i64,
    },
}

/// One fact's persisted drift entry: its identity, last outcome, and freshness
/// state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactEntry {
    /// The declaring node.
    pub node_id: Id,
    /// The fact's node-local id.
    pub fact_id: String,
    /// The last observed outcome.
    pub outcome: ProbeOutcome,
    /// How its freshness is tracked.
    pub state: FactState,
}

/// A loaded-or-to-be-written drift snapshot: a write stamp plus its fact entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DriftSnapshot {
    /// When the snapshot was written (Unix seconds) — the racy `>=` reference for
    /// input fingerprints (as the index's `index_timestamp` is).
    pub stamp: i64,
    /// One entry per tracked fact.
    pub facts: Vec<FactEntry>,
}

/// The serialized body shape (written by reference on encode).
#[derive(Serialize)]
struct BodyRef<'a> {
    stamp: i64,
    fact_count: u64,
    facts: &'a [FactEntry],
}

/// The serialized body shape (owned on decode).
#[derive(Deserialize)]
struct BodyOwned {
    stamp: i64,
    fact_count: u64,
    facts: Vec<FactEntry>,
}

/// The outcome of loading a snapshot file: a valid snapshot, or a typed signal
/// that the caller should rebuild.
#[derive(Debug, Clone)]
pub enum Load {
    /// A valid snapshot was read.
    Loaded(DriftSnapshot),
    /// The file is missing or corrupt; rebuild it. Carries the reason.
    RebuildNeeded(RebuildReason),
}

/// Why a snapshot could not be loaded as-is — every variant is an *expected,
/// self-healing* outcome, not a hard error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RebuildReason {
    /// No snapshot file exists yet (first run).
    #[error("no drift snapshot yet")]
    Missing,
    /// The file is shorter than the smallest well-formed snapshot.
    #[error("drift snapshot is truncated")]
    TooShort,
    /// The trailing checksum did not match — corruption or a torn write.
    #[error("drift snapshot checksum mismatch (corrupt)")]
    BadChecksum,
    /// The leading magic bytes are not an odm drift snapshot.
    #[error("not an odm drift snapshot (bad magic)")]
    BadMagic,
    /// The format version is not the one this build understands.
    #[error("drift snapshot format version {found} is unsupported")]
    VersionMismatch {
        /// The version found on disk.
        found: u16,
    },
    /// The body could not be deserialized.
    #[error("drift snapshot body did not deserialize")]
    Decode,
    /// The header's fact count disagreed with the decoded facts.
    #[error("drift snapshot fact count mismatch")]
    CountMismatch,
}

/// A hard error persisting or reading a snapshot (distinct from the self-healing
/// [`RebuildReason`]).
#[derive(Debug, thiserror::Error)]
pub enum DriftError {
    /// The snapshot could not be serialized.
    #[error("serializing the drift snapshot")]
    Encode(#[from] serde_json::Error),
    /// The atomic write failed.
    #[error("writing the drift snapshot")]
    Store(#[from] odm_store::StoreError),
    /// The snapshot file could not be read (an error other than "not found").
    #[error("reading the drift snapshot at {path}")]
    Read {
        /// The path that failed to read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// SHA-256 over `bytes`, as a fixed 32-byte array (the snapshot checksum).
fn checksum_of(bytes: &[u8]) -> [u8; 32] {
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&Sha256::digest(bytes));
    digest
}

impl DriftSnapshot {
    /// Creates a snapshot from a write stamp and its fact entries.
    #[must_use]
    pub fn new(stamp: i64, facts: Vec<FactEntry>) -> Self {
        Self { stamp, facts }
    }

    /// The entry for `(node_id, fact_id)`, if present.
    #[must_use]
    pub fn entry(&self, node_id: Id, fact_id: &str) -> Option<&FactEntry> {
        self.facts.iter().find(|e| e.node_id == node_id && e.fact_id == fact_id)
    }

    /// Serializes to on-disk bytes (header + postcard body + trailing checksum).
    ///
    /// # Errors
    ///
    /// Returns [`DriftError::Encode`] if the body cannot be serialized.
    pub fn encode(&self) -> Result<Vec<u8>, DriftError> {
        let body = serde_json::to_vec(&BodyRef {
            stamp: self.stamp,
            fact_count: self.facts.len() as u64,
            facts: &self.facts,
        })?;
        let mut out = Vec::with_capacity(PREFIX_LEN + body.len() + CHECKSUM_LEN);
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&SNAPSHOT_VERSION.to_le_bytes());
        out.extend_from_slice(&body);
        let checksum = checksum_of(&out);
        out.extend_from_slice(&checksum);
        Ok(out)
    }

    /// Decodes on-disk bytes, verifying checksum, magic, version, and fact count.
    ///
    /// # Errors
    ///
    /// Returns the [`RebuildReason`] for the first check that fails.
    pub fn decode(bytes: &[u8]) -> Result<Self, RebuildReason> {
        if bytes.len() < MIN_LEN {
            return Err(RebuildReason::TooShort);
        }
        let (signed, checksum) = bytes.split_at(bytes.len() - CHECKSUM_LEN);
        if checksum_of(signed)[..] != checksum[..] {
            return Err(RebuildReason::BadChecksum);
        }
        if signed[0..8] != MAGIC {
            return Err(RebuildReason::BadMagic);
        }
        let found = u16::from_le_bytes([signed[8], signed[9]]);
        if found != SNAPSHOT_VERSION {
            return Err(RebuildReason::VersionMismatch { found });
        }
        let body: BodyOwned =
            serde_json::from_slice(&signed[PREFIX_LEN..]).map_err(|_| RebuildReason::Decode)?;
        if body.fact_count != body.facts.len() as u64 {
            return Err(RebuildReason::CountMismatch);
        }
        Ok(Self { stamp: body.stamp, facts: body.facts })
    }

    /// Persists to `path` with a crash-safe atomic write (reusing
    /// [`odm_store::atomic::write`]).
    ///
    /// # Errors
    ///
    /// Returns [`DriftError::Encode`]/[`DriftError::Store`] on failure.
    pub fn persist(&self, path: &Path) -> Result<(), DriftError> {
        let bytes = self.encode()?;
        odm_store::atomic::write(path, &bytes)?;
        Ok(())
    }

    /// Loads a snapshot from `path`, self-healing: a missing or corrupt file
    /// yields [`Load::RebuildNeeded`] rather than an error.
    ///
    /// # Errors
    ///
    /// Returns [`DriftError::Read`] only for an I/O error other than "not found".
    pub fn load(path: &Path) -> Result<Load, DriftError> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Load::RebuildNeeded(RebuildReason::Missing));
            }
            Err(source) => return Err(DriftError::Read { path: path.to_path_buf(), source }),
        };
        match Self::decode(&bytes) {
            Ok(snapshot) => Ok(Load::Loaded(snapshot)),
            Err(reason) => Ok(Load::RebuildNeeded(reason)),
        }
    }
}
