//! The index snapshot: a versioned, checksummed, atomically-written file holding
//! a sorted set of [`IndexRecord`]s (ODD-0014 §3.1 header, §3.3 persistence).
//!
//! # On-disk layout
//!
//! ```text
//! ┌────────────┬───────────────┬───────────┬───────────────────┬──────────────┐
//! │ MAGIC (8)  │ version (u16) │ algo (u8) │ body (postcard)   │ checksum(32) │
//! └────────────┴───────────────┴───────────┴───────────────────┴──────────────┘
//! └──────────────────────── checksummed prefix ─────────────────┘
//! ```
//!
//! `MAGIC`, the format version, and the hash-algorithm id sit at fixed offsets so
//! a foreign or stale file is rejected cheaply *before* any deserialization. The
//! `body` is a `postcard`-encoded `{ index_timestamp, record_count, records }`
//! (postcard is the most compact serde binary format with a documented stable
//! wire format — ODD-0014 §3.3). The trailing 32-byte **checksum** is SHA-256
//! over everything before it (git ends its index with a checksum too): any
//! corruption, truncation, magic/version mismatch, or decode failure on load is
//! surfaced as a typed [`RebuildReason`] — **never a silent bad parse** — so the
//! caller rebuilds from the node files (the index carries no authority).
//!
//! The write reuses [`odm_store::atomic::write`] (temp + fsync + rename +
//! dir-fsync); the sequence is not reimplemented here.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::record::IndexRecord;

/// Magic sentinel at the start of every snapshot file.
pub const MAGIC: [u8; 8] = *b"ODMINDEX";

/// The current snapshot format version. Bumped on any incompatible change to the
/// layout or the record shape; an older file is detected and rebuilt.
pub const FORMAT_VERSION: u16 = 1;

/// The hash algorithm used for the trailing checksum (and, by convention, the
/// record fingerprints). A fast non-crypto hash (xxh3) is a deferred perf option
/// (ODD-0014 §3.1); SHA-256 reuses the workspace `sha2` and is more than adequate
/// for a local, derived, rebuildable cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgo {
    /// SHA-256 (algorithm id `1`).
    Sha256,
}

impl HashAlgo {
    /// The one-byte on-disk id.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            HashAlgo::Sha256 => 1,
        }
    }

    /// Parses the on-disk id, or `None` for an unknown algorithm.
    #[must_use]
    pub const fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(HashAlgo::Sha256),
            _ => None,
        }
    }
}

/// The hash algorithm this build writes.
pub const HASH_ALGO: HashAlgo = HashAlgo::Sha256;

/// Byte length of the fixed header prefix (`MAGIC` + version + algo).
const PREFIX_LEN: usize = 8 + 2 + 1;
/// Byte length of the trailing checksum.
const CHECKSUM_LEN: usize = 32;
/// The shortest possible well-formed file (prefix + checksum, empty body).
const MIN_LEN: usize = PREFIX_LEN + CHECKSUM_LEN;

/// The snapshot header (everything but the record body and the trailing
/// checksum). All six header elements ODD-0014 §3.1 calls for are reachable:
/// [`MAGIC`] + this struct's fields + the checksum verified on [`Snapshot::decode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// The format version the snapshot was written with.
    pub format_version: u16,
    /// The hash algorithm in use.
    pub hash_algo: HashAlgo,
    /// When the index was stamped (Unix seconds) — slice03's racy `>=` reference.
    pub index_timestamp: i64,
    /// The number of records in the body.
    pub record_count: u64,
}

/// A loaded (or to-be-written) index snapshot: a stamp plus its records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// When the index was stamped (Unix seconds). slice03 compares a file's
    /// `mtime_secs >=` this value to decide whether `stat` can be trusted.
    pub index_timestamp: i64,
    /// The records, one per tracked node file.
    pub records: Vec<IndexRecord>,
}

/// The serialized body shape, written by reference (no clone) on encode.
#[derive(Serialize)]
struct BodyRef<'a> {
    index_timestamp: i64,
    record_count: u64,
    records: &'a [IndexRecord],
}

/// The serialized body shape, owned on decode.
#[derive(Deserialize)]
struct BodyOwned {
    index_timestamp: i64,
    record_count: u64,
    records: Vec<IndexRecord>,
}

/// The outcome of loading a snapshot file: a valid snapshot, or a typed signal
/// that the caller should rebuild from the node files.
#[derive(Debug, Clone)]
pub enum Load {
    /// A valid snapshot was read.
    Loaded(Snapshot),
    /// The file is missing, corrupt, or stale; rebuild it. Carries the reason.
    RebuildNeeded(RebuildReason),
}

/// Why a snapshot could not be loaded as-is and must be rebuilt. Every variant
/// is an *expected, self-healing* outcome — not a hard error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RebuildReason {
    /// No snapshot file exists yet (first run).
    #[error("no index snapshot yet")]
    Missing,
    /// The file is shorter than the smallest well-formed snapshot.
    #[error("index snapshot is truncated")]
    TooShort,
    /// The trailing checksum did not match the body — corruption or a torn write.
    #[error("index snapshot checksum mismatch (corrupt)")]
    BadChecksum,
    /// The leading magic bytes are not an odm index.
    #[error("not an odm index snapshot (bad magic)")]
    BadMagic,
    /// The format version is not the one this build understands.
    #[error("index snapshot format version {found} is unsupported")]
    VersionMismatch {
        /// The version found on disk.
        found: u16,
    },
    /// The hash-algorithm id is not one this build knows.
    #[error("index snapshot uses unknown hash algorithm id {0}")]
    UnknownHashAlgo(u8),
    /// The body could not be deserialized.
    #[error("index snapshot body did not deserialize")]
    Decode,
    /// The header's record count disagreed with the decoded records.
    #[error("index snapshot record count mismatch")]
    CountMismatch,
}

/// A hard error persisting or reading a snapshot (distinct from the self-healing
/// [`RebuildReason`]).
#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    /// The snapshot could not be serialized.
    #[error("serializing the index snapshot")]
    Encode(#[from] postcard::Error),
    /// The atomic write failed.
    #[error("writing the index snapshot")]
    Store(#[from] odm_store::StoreError),
    /// The snapshot file could not be read (an error other than "not found").
    #[error("reading the index snapshot at {path}")]
    Read {
        /// The path that failed to read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// SHA-256 over `bytes`, as a fixed 32-byte array.
fn checksum_of(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&out);
    digest
}

impl Snapshot {
    /// Creates a snapshot from a stamp and its records.
    #[must_use]
    pub fn new(index_timestamp: i64, records: Vec<IndexRecord>) -> Self {
        Self { index_timestamp, records }
    }

    /// The snapshot header (constants + stamp + record count).
    #[must_use]
    pub fn header(&self) -> Header {
        Header {
            format_version: FORMAT_VERSION,
            hash_algo: HASH_ALGO,
            index_timestamp: self.index_timestamp,
            record_count: self.records.len() as u64,
        }
    }

    /// Serializes the snapshot to its on-disk bytes (header + postcard body +
    /// trailing checksum).
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::Encode`] if the body cannot be serialized (not
    /// reachable for records produced by this crate).
    pub fn encode(&self) -> Result<Vec<u8>, IndexError> {
        let body = postcard::to_allocvec(&BodyRef {
            index_timestamp: self.index_timestamp,
            record_count: self.records.len() as u64,
            records: &self.records,
        })?;

        let mut out = Vec::with_capacity(PREFIX_LEN + body.len() + CHECKSUM_LEN);
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.push(HASH_ALGO.as_u8());
        out.extend_from_slice(&body);
        let checksum = checksum_of(&out);
        out.extend_from_slice(&checksum);
        Ok(out)
    }

    /// Decodes on-disk bytes into a snapshot, verifying the checksum, magic,
    /// format version, hash-algo id, and record count first.
    ///
    /// # Errors
    ///
    /// Returns the [`RebuildReason`] for the first check that fails — the caller
    /// rebuilds rather than trusting a bad parse.
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
        if found != FORMAT_VERSION {
            return Err(RebuildReason::VersionMismatch { found });
        }
        let algo_byte = signed[10];
        HashAlgo::from_u8(algo_byte).ok_or(RebuildReason::UnknownHashAlgo(algo_byte))?;

        let body: BodyOwned =
            postcard::from_bytes(&signed[PREFIX_LEN..]).map_err(|_| RebuildReason::Decode)?;
        if body.record_count != body.records.len() as u64 {
            return Err(RebuildReason::CountMismatch);
        }
        Ok(Self { index_timestamp: body.index_timestamp, records: body.records })
    }

    /// Persists the snapshot to `path` with a crash-safe atomic write, reusing
    /// [`odm_store::atomic::write`] (temp + fsync + rename + dir-fsync).
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::Encode`] if serialization fails or
    /// [`IndexError::Store`] if the write fails.
    pub fn persist(&self, path: &Path) -> Result<(), IndexError> {
        let bytes = self.encode()?;
        odm_store::atomic::write(path, &bytes)?;
        Ok(())
    }

    /// Loads a snapshot from `path`, self-healing: a missing, corrupt, or stale
    /// file yields [`Load::RebuildNeeded`] rather than an error.
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::Read`] only for an I/O error *other* than the file
    /// being absent (which is a normal first-run [`RebuildReason::Missing`]).
    pub fn load(path: &Path) -> Result<Load, IndexError> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Load::RebuildNeeded(RebuildReason::Missing));
            }
            Err(source) => return Err(IndexError::Read { path: path.to_path_buf(), source }),
        };
        match Self::decode(&bytes) {
            Ok(snapshot) => Ok(Load::Loaded(snapshot)),
            Err(reason) => Ok(Load::RebuildNeeded(reason)),
        }
    }
}
