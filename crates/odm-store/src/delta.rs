//! The pending node-file delta between `HEAD` and a store worktree.
//!
//! Factored out of `store commit` (arc-store-lifecycle s01) so `store status`
//! (s02) can reuse the same computation without recomputing it against a
//! different notion of "changed" — both read [`Repo::tree_delta`] over
//! [`layout::NODES_DIR`] and classify by [`NodeType`].

use std::collections::BTreeMap;
use std::path::Path;

use odm_core::frontmatter::Document;
use serde::Serialize;

use crate::error::Result;
use crate::git::{ChangeKind, FileChange, Repo};
use crate::layout;

/// Created/modified/removed counts for one node type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct TypeCounts {
    /// Node files of this type present in the worktree but not `HEAD`.
    pub created: usize,
    /// Node files of this type present in both, with different content.
    pub modified: usize,
    /// Node files of this type present in `HEAD` but not the worktree.
    pub removed: usize,
}

/// The pending node-file delta between `HEAD` and the current worktree.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct NodeDelta {
    /// Total node files created.
    pub created: usize,
    /// Total node files modified.
    pub modified: usize,
    /// Total node files removed.
    pub removed: usize,
    /// Per-type breakdown, keyed by [`odm_core::NodeType::as_str`] (or
    /// `"unknown"` for a file that could not be read/parsed as a node —
    /// counted rather than dropped, so the total never silently under-reports).
    pub by_type: BTreeMap<String, TypeCounts>,
}

impl NodeDelta {
    /// Whether there is no node-file change at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.created == 0 && self.modified == 0 && self.removed == 0
    }

    /// Renders the D-1 auto-summary message, e.g.
    /// `store: +2 slice, ~3 arc, ~2 design, -1 project`.
    ///
    /// Types are walked in [`Self::by_type`]'s key order (alphabetical) for a
    /// deterministic message. Empty deltas render as `store: no node changes`
    /// — callers with a non-empty worktree but no node changes (e.g. only
    /// `config.toml` touched) should prefer their own fallback wording.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.is_empty() {
            return "store: no node changes".to_string();
        }
        let mut parts = Vec::new();
        for (ty, counts) in &self.by_type {
            if counts.created > 0 {
                parts.push(format!("+{} {ty}", counts.created));
            }
            if counts.modified > 0 {
                parts.push(format!("~{} {ty}", counts.modified));
            }
            if counts.removed > 0 {
                parts.push(format!("-{} {ty}", counts.removed));
            }
        }
        format!("store: {}", parts.join(", "))
    }
}

/// Computes the node delta between `HEAD` and `repo`'s worktree.
///
/// `store_root` is the worktree's root (equal to `repo`'s work directory) —
/// passed explicitly rather than re-derived, since the caller already has it
/// from [`crate::StoreHome`].
///
/// # Errors
///
/// Returns [`crate::StoreError::Git`] / [`crate::StoreError::Io`] if reading
/// git objects or the worktree fails.
pub fn compute(repo: &Repo, store_root: &Path) -> Result<NodeDelta> {
    let changes = repo.tree_delta(layout::NODES_DIR)?;
    let mut delta = NodeDelta::default();
    for change in &changes {
        let ty = classify(store_root, change);
        let counts = delta.by_type.entry(ty).or_default();
        match change.kind {
            ChangeKind::Created => {
                delta.created += 1;
                counts.created += 1;
            }
            ChangeKind::Modified => {
                delta.modified += 1;
                counts.modified += 1;
            }
            ChangeKind::Removed => {
                delta.removed += 1;
                counts.removed += 1;
            }
        }
    }
    Ok(delta)
}

/// The node type of a changed file — `"unknown"` if it cannot be read or
/// parsed as a node (an unrelated file under `nodes/`, or a corrupt one).
fn classify(store_root: &Path, change: &FileChange) -> String {
    let text = match &change.removed_content {
        Some(bytes) => String::from_utf8_lossy(bytes).into_owned(),
        None => std::fs::read_to_string(store_root.join(&change.path)).unwrap_or_default(),
    };
    Document::parse(&text)
        .map(|d| d.frontmatter().node_type().as_str().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_renders_created_modified_removed_by_type() {
        let mut by_type = BTreeMap::new();
        by_type.insert("arc".to_string(), TypeCounts { created: 0, modified: 3, removed: 0 });
        by_type.insert("slice".to_string(), TypeCounts { created: 2, modified: 0, removed: 0 });
        let delta = NodeDelta { created: 2, modified: 3, removed: 0, by_type };
        assert_eq!(delta.summary(), "store: ~3 arc, +2 slice");
    }

    #[test]
    fn test_summary_of_empty_delta() {
        assert_eq!(NodeDelta::default().summary(), "store: no node changes");
        assert!(NodeDelta::default().is_empty());
    }

    #[test]
    fn test_classify_falls_back_to_unknown_for_unparseable_content() {
        // A `Removed` change carries its old content inline, so the fallback
        // is exercisable without a live repo.
        let change = FileChange {
            path: "nodes/2026/07/garbage.md".to_string(),
            kind: ChangeKind::Removed,
            removed_content: Some(b"not frontmatter at all".to_vec()),
        };
        assert_eq!(classify(Path::new("/nonexistent"), &change), "unknown");
    }
}
