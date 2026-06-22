//! The on-disk node layout: `nodes/YYYY/MM/<ULID>.md`.
//!
//! The path is a **pure function of the id**: the `YYYY/MM` shard comes from the
//! id's creation month (read from the ULID timestamp), so a node file never
//! moves on retitle, reparent, or gate change, and locating a node by id is
//! O(1) — no scan or lookup index needed.

use std::path::{Path, PathBuf};

use chrono::Datelike;
use odm_core::Id;

/// The directory, relative to the store root, that holds all node files.
pub const NODES_DIR: &str = "nodes";

/// The file extension for node files.
pub const NODE_EXT: &str = "md";

/// Returns the path of a node, relative to the store root:
/// `nodes/<YYYY>/<MM>/<ULID>.md`.
///
/// `<YYYY>/<MM>` is the id's creation month (UTC), and `<ULID>` is the id's
/// canonical string. This is total and allocation-cheap; it touches no
/// filesystem.
#[must_use]
pub fn relative_path(id: Id) -> PathBuf {
    let created = id.created_at();
    PathBuf::from(NODES_DIR)
        .join(format!("{:04}", created.year()))
        .join(format!("{:02}", created.month()))
        .join(format!("{id}.{NODE_EXT}"))
}

/// Returns the absolute path of a node under `root`:
/// `<root>/nodes/<YYYY>/<MM>/<ULID>.md`.
#[must_use]
pub fn path_in(root: &Path, id: Id) -> PathBuf {
    root.join(relative_path(id))
}
