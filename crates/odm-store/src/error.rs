//! The store's error type.

use std::path::PathBuf;

use odm_core::frontmatter::FrontmatterError;

/// An error from the store layer.
///
/// Self-contained: the underlying `gix` error types are flattened into strings
/// so the git backend stays an implementation detail of [`crate::git`].
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// An I/O operation failed, with the path it concerned.
    #[error("io error at {path}: {source}")]
    Io {
        /// The path the operation concerned.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// A node file could not be parsed or emitted.
    #[error("frontmatter error in {path}: {source}")]
    Frontmatter {
        /// The node file.
        path: PathBuf,
        /// The underlying frontmatter error.
        source: FrontmatterError,
    },

    /// A git operation failed.
    #[error("git error: {0}")]
    Git(String),

    /// Loading or parsing `odm.toml` failed.
    #[error("config error: {0}")]
    Config(String),
}

impl StoreError {
    /// Builds an [`StoreError::Io`] tagged with the path it concerned.
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        StoreError::Io { path: path.into(), source }
    }

    /// Builds a [`StoreError::Frontmatter`] tagged with the node file.
    pub(crate) fn frontmatter(path: impl Into<PathBuf>, source: FrontmatterError) -> Self {
        StoreError::Frontmatter { path: path.into(), source }
    }
}

/// A `Result` whose error is a [`StoreError`].
pub type Result<T> = std::result::Result<T, StoreError>;
