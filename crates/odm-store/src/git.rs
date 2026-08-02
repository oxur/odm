//! Git integration via [`gix`] — pure-Rust, no shelling out (ODD-0013 Q-2).
//!
//! The store commits node files by building a tree directly from the working
//! directory and writing a commit object — it never *reads* the on-disk
//! index to decide what to commit. "Status" (odm's own — [`Repo::is_clean`],
//! [`Repo::tree_delta`]) is expressed as a comparison between the current
//! worktree tree and `HEAD`'s tree (equal ⇒ clean), which is race-free and
//! needs no index file; that stays true after arc-store-lifecycle s04.
//!
//! s04 added one thing on the *write* side: after a commit, [`Repo::commit_all`]
//! rewrites the on-disk index to match the new `HEAD` tree, purely so a plain
//! `git status` — standard git tooling, not odm's own status path — reads
//! clean immediately afterward instead of misreporting the just-committed
//! files as staged-deletions plus untracked.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use gix::ObjectId;
use gix::bstr::BString;
use gix::objs::Tree;
use gix::objs::tree::{Entry, EntryKind};

use crate::error::{Result, StoreError};

/// How a single file under a watched directory (e.g. `nodes/`) differs
/// between `HEAD`'s tree and the current worktree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// Present in the worktree, absent from `HEAD`.
    Created,
    /// Present in both, with a different blob id.
    Modified,
    /// Present in `HEAD`, absent from the worktree.
    Removed,
}

/// One file's change, as found by [`Repo::tree_delta`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    /// The path, relative to the work directory, `/`-separated (e.g.
    /// `nodes/2026/07/01J....md`).
    pub path: String,
    /// What happened to it.
    pub kind: ChangeKind,
    /// The file's content **as it was in `HEAD`** — only present for
    /// [`ChangeKind::Removed`], since that content no longer exists on disk
    /// for a caller to read directly.
    pub removed_content: Option<Vec<u8>>,
}

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
        let mut repo = gix::init(path).map_err(git_err)?;
        ensure_identity(&mut repo)?;
        Ok(Self { repo })
    }

    /// Opens an existing repository at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Git`] if `path` is not a repository.
    pub fn open(path: &Path) -> Result<Self> {
        let mut repo = gix::open(path).map_err(git_err)?;
        ensure_identity(&mut repo)?;
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
    /// `HEAD` (if any). Then syncs the on-disk git index to that same tree
    /// (arc-store-lifecycle s04), so a plain `git status` on the store
    /// worktree reads clean immediately afterward — without this, the index
    /// is left at its pre-commit state and misreports the just-committed
    /// files as staged-deletions plus untracked, even though `HEAD` already
    /// has the full tree. Returns the new commit id as a hex string.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Git`] / [`StoreError::Io`] if reading the worktree,
    /// writing git objects, or syncing the index fails.
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
        self.sync_index_to_tree(tree)?;
        Ok(id.detach().to_string())
    }

    /// Rewrites the on-disk git index to match `tree` exactly, so a plain
    /// `git status` on the store worktree reads clean against the commit
    /// that tree belongs to.
    ///
    /// [`Self::commit_all`] never reads or updates the index while building a
    /// commit — it walks the filesystem directly (see the module doc) — so
    /// without this the index silently drifts from `HEAD` on every commit.
    /// This is purely a compatibility sync for standard git tooling: it does
    /// not feed odm's own status ([`Self::tree_delta`], [`Self::is_clean`]),
    /// which stays a race-free worktree-tree-vs-`HEAD` comparison and never
    /// reads the index.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Git`] if the index cannot be built from `tree` or
    /// written to disk.
    fn sync_index_to_tree(&self, tree: ObjectId) -> Result<()> {
        let mut index = self.repo.index_from_tree(&tree).map_err(git_err)?;
        index.write(gix::index::write::Options::default()).map_err(git_err)?;
        Ok(())
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

    /// The files under `dir` (relative to the work directory, e.g. `"nodes"`)
    /// that differ between `HEAD`'s tree and the current worktree.
    ///
    /// Restricted to `dir` rather than diffing the whole worktree, because
    /// callers want a delta over one subtree (the node files), not every file
    /// a commit would carry (e.g. `config.toml`).
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Git`] / [`StoreError::Io`] if reading the tree or
    /// the worktree fails.
    pub fn tree_delta(&self, dir: &str) -> Result<Vec<FileChange>> {
        let work_dir =
            self.repo.work_dir().ok_or_else(|| StoreError::Git("bare repository".into()))?;

        let mut before: BTreeMap<String, ObjectId> = BTreeMap::new();
        if let Some(head_tree_id) = self.head_tree_id() {
            let tree = self.repo.find_tree(head_tree_id).map_err(git_err)?;
            let decoded = tree.decode().map_err(git_err)?;
            if let Some(entry) = decoded.entries.iter().find(|e| e.filename == dir)
                && entry.mode.is_tree()
            {
                self.collect_tree_blobs(entry.oid.to_owned(), dir, &mut before)?;
            }
        }

        let mut after: BTreeMap<String, ObjectId> = BTreeMap::new();
        let dir_path = work_dir.join(dir);
        if dir_path.is_dir() {
            self.collect_fs_blobs(&dir_path, dir, &mut after)?;
        }

        let mut changes: Vec<FileChange> = after
            .iter()
            .filter_map(|(path, id)| match before.get(path) {
                None => Some(FileChange {
                    path: path.clone(),
                    kind: ChangeKind::Created,
                    removed_content: None,
                }),
                Some(old_id) if old_id != id => Some(FileChange {
                    path: path.clone(),
                    kind: ChangeKind::Modified,
                    removed_content: None,
                }),
                Some(_) => None,
            })
            .collect();

        for (path, id) in &before {
            if after.contains_key(path) {
                continue;
            }
            let content = self.repo.find_blob(*id).map_err(git_err)?.data.clone();
            changes.push(FileChange {
                path: path.clone(),
                kind: ChangeKind::Removed,
                removed_content: Some(content),
            });
        }

        changes.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(changes)
    }

    /// Recursively collects blob paths (relative to the work dir, `/`-joined)
    /// and their object ids from an existing git tree.
    fn collect_tree_blobs(
        &self,
        tree_id: ObjectId,
        prefix: &str,
        out: &mut BTreeMap<String, ObjectId>,
    ) -> Result<()> {
        let tree = self.repo.find_tree(tree_id).map_err(git_err)?;
        let decoded = tree.decode().map_err(git_err)?;
        for entry in &decoded.entries {
            let full = format!("{prefix}/{}", entry.filename);
            if entry.mode.is_tree() {
                self.collect_tree_blobs(entry.oid.to_owned(), &full, out)?;
            } else {
                out.insert(full, entry.oid.to_owned());
            }
        }
        Ok(())
    }

    /// Recursively collects blob paths (relative to the work dir, `/`-joined)
    /// and their content-addressed ids from the filesystem, writing each file
    /// as a blob (content-addressed and idempotent, matching [`Self::write_tree`]).
    fn collect_fs_blobs(
        &self,
        dir: &Path,
        prefix: &str,
        out: &mut BTreeMap<String, ObjectId>,
    ) -> Result<()> {
        for dirent in fs::read_dir(dir).map_err(|e| StoreError::io(dir, e))? {
            let dirent = dirent.map_err(|e| StoreError::io(dir, e))?;
            let name = dirent.file_name();
            let path = dirent.path();
            let file_type = dirent.file_type().map_err(|e| StoreError::io(&path, e))?;
            let full = format!("{prefix}/{}", name.to_string_lossy());
            if file_type.is_dir() {
                self.collect_fs_blobs(&path, &full, out)?;
            } else if file_type.is_file() {
                let bytes = fs::read(&path).map_err(|e| StoreError::io(&path, e))?;
                let blob = self.repo.write_blob(&bytes).map_err(git_err)?.detach();
                out.insert(full, blob);
            }
        }
        Ok(())
    }

    /// Recursively writes `dir` as a git tree, returning the tree's id. Files
    /// become blobs; subdirectories become subtrees; `.git`, empty
    /// directories, and anything the worktree's own `.gitignore` excludes are
    /// skipped.
    ///
    /// The exclude check matters as much as the `.git` skip: `commit_all`
    /// walks the filesystem directly rather than staging through git's index,
    /// so without it a gitignored, derived cache (odm's own `.odm/index` and
    /// `.odm/drift` — ODD-0022's own `.gitignore` marks them expendable)
    /// would get baked permanently into the orphan branch's history the
    /// moment it exists on disk, the first time anything commits.
    fn write_tree(&self, dir: &Path) -> Result<ObjectId> {
        let work_dir =
            self.repo.work_dir().ok_or_else(|| StoreError::Git("bare repository".into()))?;
        let index = gix::index::State::new(self.repo.object_hash());
        let mut excludes = self
            .repo
            .excludes(&index, None, gix::worktree::stack::state::ignore::Source::default())
            .map_err(git_err)?;
        self.write_tree_excluding(dir, work_dir, &mut excludes)
    }

    /// [`Self::write_tree`]'s recursion, carrying the exclude stack down so
    /// it is built once per commit rather than once per directory.
    fn write_tree_excluding(
        &self,
        dir: &Path,
        work_dir: &Path,
        excludes: &mut gix::AttributeStack<'_>,
    ) -> Result<ObjectId> {
        let mut entries: Vec<Entry> = Vec::new();
        for dirent in fs::read_dir(dir).map_err(|e| StoreError::io(dir, e))? {
            let dirent = dirent.map_err(|e| StoreError::io(dir, e))?;
            let name = dirent.file_name();
            if name == ".git" {
                continue;
            }
            let path = dirent.path();
            let file_type = dirent.file_type().map_err(|e| StoreError::io(&path, e))?;
            let relative = path.strip_prefix(work_dir).unwrap_or(&path);
            let mode = Some(if file_type.is_dir() {
                gix::index::entry::Mode::DIR
            } else {
                gix::index::entry::Mode::FILE
            });
            let excluded = excludes
                .at_path(relative, mode)
                .map_err(|e| StoreError::io(&path, e))?
                .is_excluded();
            if excluded {
                continue;
            }
            let filename = BString::from(name.to_string_lossy().as_bytes());

            if file_type.is_dir() {
                let sub = self.write_tree_excluding(&path, work_dir, excludes)?;
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

/// Ensures the repository has a committer identity in its (in-memory) config.
///
/// `commit_as` sets the author/committer on the *commit object* from the
/// `SignatureRef` we pass, but updating `HEAD` also writes a **reflog** entry
/// whose identity comes from `repo.committer()` — i.e. from git config
/// (`user.name`/`user.email`), not from that signature. On a machine with no
/// global gitconfig (notably CI), that identity is absent and the reflog write
/// fails with "The reflog could not be created or updated". Seeding a default
/// identity here makes commits work regardless of the ambient git config. The
/// change is in-memory only; it never touches the repository's `.git/config`.
fn ensure_identity(repo: &mut gix::Repository) -> Result<()> {
    let mut config = repo.config_snapshot_mut();
    config.set_value(&gix::config::tree::User::NAME, "odm").map_err(git_err)?;
    config.set_value(&gix::config::tree::User::EMAIL, "odm@localhost").map_err(git_err)?;
    config.commit().map_err(git_err)?;
    Ok(())
}

/// Flattens any `gix` error into a [`StoreError::Git`] string, so the git
/// backend type never escapes this module.
fn git_err<E: std::fmt::Display>(error: E) -> StoreError {
    StoreError::Git(error.to_string())
}
