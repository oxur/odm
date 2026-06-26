//! The index's one hashing helper.
//!
//! SHA-256 (the workspace `sha2`) is used for both the snapshot's trailing
//! checksum and the per-record `content_hash` / `meta_hash` fingerprints. A fast
//! non-crypto hash (xxh3) is a deferred perf option (ODD-0014 §3.1); the
//! [`HashAlgo`](crate::snapshot::HashAlgo) id in the snapshot header makes that
//! swap a format-versioned change.

use sha2::{Digest as _, Sha256};

use crate::record::Digest;

/// SHA-256 of `bytes`, as a fixed 32-byte digest.
#[must_use]
pub(crate) fn sha256(bytes: &[u8]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&out);
    digest
}
