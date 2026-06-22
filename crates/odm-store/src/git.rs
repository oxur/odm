//! Git integration via [`gix`] — pure-Rust, no shelling out (ODD-0013 Q-2).
//!
//! The store commits node files by building a tree directly from the working
//! directory and writing a commit object; it never goes through the on-disk
//! index. "Status" is therefore expressed as a comparison between the current
//! worktree tree and `HEAD`'s tree (equal ⇒ clean), which is race-free and
//! needs no index file.

use std::fs;
use std::path::Path;

use gix::ObjectId;
use gix::bstr::BString;
use gix::objs::Tree;
use gix::objs::tree::{Entry, EntryKind};

use crate::error::{Result, StoreError};

/// A handle to a git repository.
#[derive(Debug)]
pub struct Repo {
    repo: gix::Repository,
}

impl Repo {
    /// Initializes a new repository at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Git`] if the repository cannot be created.
    pub fn init(path: &Path) -> Result<Self> {
        let repo = gix::init(path).map_err(git_err)?;
        Ok(Self { repo })
    }

    /// Opens an existing repository at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Git`] if `path` is not a repository.
    pub fn open(path: &Path) -> Result<Self> {
        let repo = gix::open(path).map_err(git_err)?;
        Ok(Self { repo })
    }

    /// The repository's working directory.
    #[must_use]
    pub fn work_dir(&self) -> Option<&Path> {
        self.repo.work_dir()
    }

    /// Commits the current working-directory contents to `HEAD`.
    ///
    /// Builds a tree from every file under the work directory (excluding
    /// `.git`), writes it, and creates a commit whose parent is the previous
    /// `HEAD` (if any). Returns the new commit id as a hex string.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Git`] / [`StoreError::Io`] if reading the worktree
    /// or writing git objects fails.
    pub fn commit_all(&self, message: &str) -> Result<String> {
        let work_dir =
            self.repo.work_dir().ok_or_else(|| StoreError::Git("bare repository".into()))?;
        let tree = self.write_tree(work_dir)?;

        let parents: Vec<ObjectId> = self.head_commit_id().into_iter().collect();
        let now = gix::date::Time::now_utc();
        let sig = gix::actor::SignatureRef {
            name: "odm".into(),
            email: "odm@localhost".into(),
            time: now,
        };

        let id = self.repo.commit_as(sig, sig, "HEAD", message, tree, parents).map_err(git_err)?;
        Ok(id.detach().to_string())
    }

    /// Returns `true` if the working directory matches `HEAD`'s tree exactly.
    ///
    /// With no commits yet, an empty worktree is clean and a non-empty one is
    /// not.
    ///
    /// # Errors
    ///
    /// Returns an error if the worktree tree cannot be built.
    pub fn is_clean(&self) -> Result<bool> {
        let work_dir =
            self.repo.work_dir().ok_or_else(|| StoreError::Git("bare repository".into()))?;
        let worktree_tree = self.write_tree(work_dir)?;
        match self.head_tree_id() {
            Some(head_tree) => Ok(head_tree == worktree_tree),
            None => Ok(worktree_tree == self.repo.empty_tree().id().detach()),
        }
    }

    /// The id of the current `HEAD` commit, or `None` on an unborn branch.
    fn head_commit_id(&self) -> Option<ObjectId> {
        self.repo.head_id().ok().map(|id| id.detach())
    }

    /// The tree id of the current `HEAD` commit, or `None` if there is none.
    fn head_tree_id(&self) -> Option<ObjectId> {
        let commit = self.repo.head_commit().ok()?;
        commit.tree_id().ok().map(|id| id.detach())
    }

    /// Recursively writes `dir` as a git tree, returning the tree's id. Files
    /// become blobs; subdirectories become subtrees; `.git` and empty
    /// directories are skipped (git does not track empty directories).
    fn write_tree(&self, dir: &Path) -> Result<ObjectId> {
        let mut entries: Vec<Entry> = Vec::new();
        for dirent in fs::read_dir(dir).map_err(|e| StoreError::io(dir, e))? {
            let dirent = dirent.map_err(|e| StoreError::io(dir, e))?;
            let name = dirent.file_name();
            if name == ".git" {
                continue;
            }
            let path = dirent.path();
            let file_type = dirent.file_type().map_err(|e| StoreError::io(&path, e))?;
            let filename = BString::from(name.to_string_lossy().as_bytes());

            if file_type.is_dir() {
                let sub = self.write_tree(&path)?;
                // Skip empty subtrees (git has no concept of an empty directory).
                if sub != self.repo.empty_tree().id().detach() {
                    entries.push(Entry { mode: EntryKind::Tree.into(), filename, oid: sub });
                }
            } else if file_type.is_file() {
                let bytes = fs::read(&path).map_err(|e| StoreError::io(&path, e))?;
                let blob = self.repo.write_blob(&bytes).map_err(git_err)?.detach();
                entries.push(Entry { mode: EntryKind::Blob.into(), filename, oid: blob });
            }
        }
        // git requires tree entries in canonical order; `Entry: Ord` encodes it
        // (including the directory trailing-slash rule).
        entries.sort();
        let tree = Tree { entries };
        Ok(self.repo.write_object(&tree).map_err(git_err)?.detach())
    }
}

/// Flattens any `gix` error into a [`StoreError::Git`] string, so the git
/// backend type never escapes this module.
fn git_err<E: std::fmt::Display>(error: E) -> StoreError {
    StoreError::Git(error.to_string())
}
