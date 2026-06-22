//! `odm.toml` loading via a layered search (cwd → repo root → user config).

use std::path::{Path, PathBuf};

use confyg::Confygery;
use confyg::searchpath::Finder;
use serde::Deserialize;

use crate::error::{Result, StoreError};

/// Store configuration, loaded from `odm.toml`.
///
/// Every field has a default, so a missing or partial `odm.toml` is fine.
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
    /// Loads `odm.toml` by searching, in priority order: `start` (typically the
    /// current directory), the enclosing git repository root, then the user
    /// config directory. The first `odm.toml` found wins; if none is found, the
    /// defaults are returned.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Config`] if a located `odm.toml` cannot be read or
    /// does not deserialize.
    pub fn load(start: &Path) -> Result<Self> {
        let mut finder = Finder::new();
        finder.add_path(&start.to_string_lossy());
        if let Some(root) = repo_root(start) {
            finder.add_path(&root.to_string_lossy());
        }
        if let Some(user) = user_config_dir() {
            finder.add_path(&user.to_string_lossy());
        }

        match finder.find("odm.toml") {
            Ok(found) => Confygery::new()
                .and_then(|mut c| {
                    c.add_file(&found)?;
                    c.build::<StoreConfig>()
                })
                .map_err(|e| StoreError::Config(e.to_string())),
            // Not found anywhere → defaults (not an error).
            Err(_) => Ok(StoreConfig::default()),
        }
    }
}

/// Walks up from `start` looking for a directory containing `.git`.
fn repo_root(start: &Path) -> Option<PathBuf> {
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
fn user_config_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg).join("odm"));
        }
    }
    std::env::var("HOME").ok().map(|home| PathBuf::from(home).join(".config").join("odm"))
}
