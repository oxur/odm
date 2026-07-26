//! The **operational** configuration — the settings that govern the data.
//!
//! Read from the store's own `config.toml` when it has one, and otherwise from
//! the `odm.toml` found by the layered search (cwd → repo root → user config).
//! Which file that is, is [`StoreHome`]'s decision; this module only parses.

use std::path::{Path, PathBuf};

use confyg::Confygery;
use serde::Deserialize;

use crate::error::{Result, StoreError};
use crate::home::StoreHome;

/// Store configuration — the author identity recorded on store commits.
///
/// Every field has a default, so a missing or partial config is fine.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct StoreConfig {
    /// Name recorded as the author/committer of store commits.
    pub author_name: String,
    /// Email recorded as the author/committer of store commits.
    pub author_email: String,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self { author_name: "odm".to_string(), author_email: "odm@localhost".to_string() }
    }
}

impl StoreConfig {
    /// Loads the operational config for the store `start` resolves to.
    ///
    /// The file is the store's `config.toml` when it has one, else the
    /// `odm.toml` found by the layered search — `start`, then the enclosing
    /// repository root, then the user config directory (ODD-0022 §4.2). With
    /// neither, the defaults are returned.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Config`] if the located file cannot be read or
    /// does not deserialize.
    pub fn load(start: &Path) -> Result<Self> {
        Self::from_home(&StoreHome::resolve(start))
    }

    /// [`Self::load`] against an already-resolved home, for a caller that has
    /// one (and so need not resolve twice).
    ///
    /// # Errors
    ///
    /// See [`Self::load`].
    pub fn from_home(home: &StoreHome) -> Result<Self> {
        let Some(path) = home.operational_path.as_ref() else {
            // No config anywhere → defaults (not an error).
            return Ok(StoreConfig::default());
        };
        Confygery::new()
            .and_then(|mut c| {
                c.add_file(&path.to_string_lossy())?;
                c.build::<StoreConfig>()
            })
            .map_err(|e| StoreError::Config(e.to_string()))
    }
}

/// Walks up from `start` looking for a directory containing `.git`.
pub(crate) fn repo_root(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(d) = dir {
        if d.join(".git").exists() {
            return Some(d.to_path_buf());
        }
        dir = d.parent();
    }
    None
}

/// The user config directory: `$XDG_CONFIG_HOME/odm` or `$HOME/.config/odm`.
pub(crate) fn user_config_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg).join("odm"));
        }
    }
    std::env::var("HOME").ok().map(|home| PathBuf::from(home).join(".config").join("odm"))
}
