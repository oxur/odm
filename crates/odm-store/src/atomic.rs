//! Crash-safe atomic file writes (ODD-0014).
//!
//! A write is staged to a sibling temp file, `fsync`'d, then `rename`'d over the
//! target, and finally the parent directory is `fsync`'d so the rename itself is
//! durable. A rename is atomic on POSIX, so a reader of the target always sees
//! either the complete old contents or the complete new contents — never a
//! partial write. If any step fails, the temp file is removed and the existing
//! target is left untouched.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::error::{Result, StoreError};

/// Atomically writes `bytes` to `path`, creating parent directories as needed.
///
/// # Errors
///
/// Returns [`StoreError::Io`] if any filesystem step fails. On failure the
/// target file is never partially written, and the temp file is cleaned up.
pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| StoreError::io(parent, e))?;

    // Temp file in the same directory, so the rename stays on one filesystem.
    let tmp = temp_path(path);

    // Write + fsync the temp file; clean it up if anything fails.
    if let Err(e) = write_synced(&tmp, bytes) {
        let _ = fs::remove_file(&tmp);
        return Err(StoreError::io(&tmp, e));
    }

    // Atomically swap it into place.
    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(StoreError::io(path, e));
    }

    // fsync the directory so the rename survives a crash. Best-effort: not all
    // platforms permit opening a directory for this, so a failure here is not
    // fatal to the (already-completed) rename.
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }

    Ok(())
}

/// Writes `bytes` to `tmp` and flushes them to disk before returning.
fn write_synced(tmp: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

/// The temp path for a target: same directory, a dotted `.tmp` sibling.
fn temp_path(path: &Path) -> std::path::PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path.file_name().map_or_else(|| "node".into(), |n| n.to_string_lossy());
    parent.join(format!(".{name}.tmp"))
}
