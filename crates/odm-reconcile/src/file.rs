//! The file probe (slice02): check a declared file against an expectation
//! (exists / content-hash / size).

use std::path::PathBuf;

use odm_core::FileExpect;
use sha2::{Digest as _, Sha256};

use crate::probe::{Probe, ProbeOutcome};

/// The **file** probe: check a file (resolved relative to the repo root) against
/// a [`FileExpect`] and map the result to a [`ProbeOutcome`].
///
/// The slice01 `Error ≠ Drifted` distinction carries, and is load-bearing here:
///
/// - **`Drifted`** — reality was observed and diverges from the declaration: the
///   file is missing when `exists: true` (or present when `exists: false`), the
///   size differs, or the content hash differs. These are real, reportable
///   declared-vs-observed findings.
/// - **`Error`** — the path could **not be evaluated**: stat-ing it failed for a
///   reason other than "not found" (a permission error, an invalid path), or the
///   content could not be read to hash it (e.g. the path is a directory). "The
///   file is gone" is drift; "I couldn't read it at all" is error.
///
/// Content hashing reuses the workspace `sha2` (SHA-256), the same algorithm the
/// index uses — not a reimplementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileProbe {
    root: PathBuf,
    path: String,
    expect: FileExpect,
}

impl FileProbe {
    /// Builds a file probe. `path` is resolved relative to `root` (the repo
    /// root / odm working directory).
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, path: impl Into<String>, expect: FileExpect) -> Self {
        Self { root: root.into(), path: path.into(), expect }
    }
}

impl Probe for FileProbe {
    fn evaluate(&self) -> ProbeOutcome {
        let full = self.root.join(&self.path);

        // Stat the path. Distinguish "not found" (a presence question — may be
        // drift) from any other error (genuinely "couldn't check" → Error).
        let metadata = match std::fs::metadata(&full) {
            Ok(metadata) => Some(metadata),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(err) => {
                return ProbeOutcome::Error {
                    reason: format!("could not stat `{}`: {err}", self.path),
                };
            }
        };

        // Presence check first (`exists`, default true).
        if !self.expect.exists {
            return match metadata {
                None => ProbeOutcome::Holds,
                Some(_) => ProbeOutcome::Drifted {
                    expected: "file absent".to_string(),
                    observed: "file present".to_string(),
                },
            };
        }
        let Some(metadata) = metadata else {
            return ProbeOutcome::Drifted {
                expected: format!("file present at `{}`", self.path),
                observed: "file missing".to_string(),
            };
        };

        // Size check (optional).
        if let Some(want) = self.expect.size {
            if metadata.len() != want {
                return ProbeOutcome::Drifted {
                    expected: format!("size {want} bytes"),
                    observed: format!("size {} bytes", metadata.len()),
                };
            }
        }

        // Content-hash check (optional). Reading the bytes is where a directory
        // or an unreadable file becomes an Error rather than drift.
        if let Some(want) = &self.expect.sha256 {
            let bytes = match std::fs::read(&full) {
                Ok(bytes) => bytes,
                Err(err) => {
                    return ProbeOutcome::Error {
                        reason: format!("could not read `{}`: {err}", self.path),
                    };
                }
            };
            let got = hex_sha256(&bytes);
            if &got != want {
                return ProbeOutcome::Drifted {
                    expected: format!("sha256 {want}"),
                    observed: format!("sha256 {got}"),
                };
            }
        }

        ProbeOutcome::Holds
    }
}

/// Lowercase-hex SHA-256 of `bytes`, using the workspace `sha2`.
fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// Truncates `s` to at most `max` bytes (on a char boundary), appending `…`
/// when it was shortened. Shared with the shell probe's observed-output report.
pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}
