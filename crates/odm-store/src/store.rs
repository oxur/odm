//! The node store: persist and load node files under a root directory.

use std::path::{Path, PathBuf};

use odm_core::Id;
use odm_core::frontmatter::Document;
use walkdir::WalkDir;

use crate::error::{Result, StoreError};
use crate::{atomic, layout};

/// A node store rooted at a directory. Node files live at
/// `<root>/nodes/YYYY/MM/<ULID>.md` (see [`crate::layout`]).
///
/// The root need not exist yet — it is created on the first write, and loading
/// from a store with no `nodes/` directory yields an empty set rather than an
/// error.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Opens a store rooted at `root`. Does not touch the filesystem.
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The store root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The absolute path where the node with `id` is (or would be) stored.
    /// A pure function of the id — no filesystem access, O(1).
    #[must_use]
    pub fn path_of(&self, id: Id) -> PathBuf {
        layout::path_in(&self.root, id)
    }

    /// Persists `document` to its id-derived path with a crash-safe atomic
    /// write, creating any missing directories. Returns the file path.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Frontmatter`] if the document cannot be emitted, or
    /// [`StoreError::Io`] if the write fails.
    pub fn persist(&self, document: &Document) -> Result<PathBuf> {
        let id = document.frontmatter().id();
        let path = self.path_of(id);
        let text = document.emit().map_err(|e| StoreError::frontmatter(&path, e))?;
        atomic::write(&path, text.as_bytes())?;
        Ok(path)
    }

    /// Loads the single node with `id` from its id-derived path.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Io`] if the file cannot be read, or
    /// [`StoreError::Frontmatter`] if it cannot be parsed.
    pub fn load(&self, id: Id) -> Result<Document> {
        let path = self.path_of(id);
        let text = std::fs::read_to_string(&path).map_err(|e| StoreError::io(&path, e))?;
        Document::parse(&text).map_err(|e| StoreError::frontmatter(&path, e))
    }

    /// Loads every node by scanning `<root>/nodes/` for `.md` files and parsing
    /// each. A missing `nodes/` directory is not an error — it yields an empty
    /// list (self-healing).
    ///
    /// Results are sorted by id (creation order) for determinism.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Io`] / [`StoreError::Frontmatter`] for the first
    /// file that cannot be read or parsed.
    pub fn load_all(&self) -> Result<Vec<Document>> {
        let nodes_dir = self.root.join(layout::NODES_DIR);
        if !nodes_dir.exists() {
            return Ok(Vec::new());
        }

        let mut docs = Vec::new();
        for entry in WalkDir::new(&nodes_dir).into_iter() {
            let entry = entry.map_err(|e| {
                let path = e.path().unwrap_or(&nodes_dir).to_path_buf();
                StoreError::io(path, e.into())
            })?;
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.extension().is_none_or(|ext| ext != layout::NODE_EXT)
            {
                continue;
            }
            let text = std::fs::read_to_string(path).map_err(|e| StoreError::io(path, e))?;
            let doc = Document::parse(&text).map_err(|e| StoreError::frontmatter(path, e))?;
            docs.push(doc);
        }

        docs.sort_by_key(|d| d.frontmatter().id());
        Ok(docs)
    }
}
