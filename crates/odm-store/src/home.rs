//! Where the store lives, and which file configures it (ODD-0022 §4.2).
//!
//! odm splits its configuration in two, because the two halves answer different
//! questions and belong to different histories:
//!
//! - **The locator** — `odm.toml` on the *code* branch. A `[store]` section
//!   saying **where** the store is. Committed with the code, minimal, and the
//!   only thing a fresh clone needs in order to find the store.
//! - **The operational config** — `config.toml` **inside** the store. Gate-sets,
//!   display settings, `docs_directory`, author: everything that governs the
//!   data, versioned *with* the data so the two cannot drift apart.
//!
//! Resolution is therefore two-stage: find `odm.toml` by the existing layered
//! search, read `[store]` from it, resolve the store root, then read that
//! store's `config.toml`.
//!
//! **Both halves are optional, and their absence is the status quo.** A repo
//! with no `[store]` keeps its store at the repo root; a store with no
//! `config.toml` reads its operational settings from `odm.toml`, exactly as
//! before. That is what lets an existing repo — odm's own included — keep
//! working untouched until it is migrated.
//!
//! This module resolves and reads. It creates nothing: no worktree, no branch,
//! no directory (that is slice 02's `init`).

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The file naming the store's location, on the code branch.
pub const LOCATOR_FILE: &str = "odm.toml";

/// The file configuring the store, inside the store.
pub const OPERATIONAL_FILE: &str = "config.toml";

/// The default base directory for worktrees, relative to the repo root.
pub const DEFAULT_WORKTREE_BASE: &str = ".worktrees";

/// The default worktree directory name, and the default branch name — kept
/// equal so the two never have to be reasoned about separately.
pub const DEFAULT_STORE_NAME: &str = "odm";

/// The `[store]` section of `odm.toml`: where the store home is.
///
/// Every field is optional; an empty `[store]` resolves to the documented
/// defaults (`.worktrees` / `odm` / `odm`), so a locator can be as short as the
/// section header.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct StoreLocation {
    /// The base directory holding worktrees, relative to the repo root.
    #[serde(default = "default_worktree_base")]
    pub worktree_base: String,
    /// The store's worktree directory, under [`Self::worktree_base`].
    #[serde(default = "default_store_name")]
    pub worktree_name: String,
    /// The orphan branch checked out in that worktree.
    ///
    /// Read but not acted on here — the branch is slice 02's business. It lives
    /// in this struct because the locator is where a rename has to update it
    /// (ODD-0022 §4.5), and splitting it elsewhere would invite the two to
    /// disagree.
    #[serde(default = "default_store_name")]
    pub branch_name: String,
}

fn default_worktree_base() -> String {
    DEFAULT_WORKTREE_BASE.to_string()
}

fn default_store_name() -> String {
    DEFAULT_STORE_NAME.to_string()
}

impl Default for StoreLocation {
    fn default() -> Self {
        Self {
            worktree_base: default_worktree_base(),
            worktree_name: default_store_name(),
            branch_name: default_store_name(),
        }
    }
}

impl StoreLocation {
    /// The store root this location points at, relative to `repo_root`:
    /// `<repo_root>/<worktree_base>/<worktree_name>`.
    #[must_use]
    pub fn store_root(&self, repo_root: &Path) -> PathBuf {
        repo_root.join(&self.worktree_base).join(&self.worktree_name)
    }
}

/// The deserialization shape for the locator file — only the `[store]` section
/// is read here, and every other key is ignored, since the operational settings
/// are read separately (and may live in a different file entirely).
#[derive(Debug, Default, Deserialize)]
struct LocatorFile {
    store: Option<StoreLocation>,
}

/// A resolved store home: where the store is, and which file configures it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreHome {
    /// The directory the locator was found in — the repo root in practice.
    pub repo_root: PathBuf,
    /// Where the nodes live. Equal to [`Self::repo_root`] when no `[store]`
    /// section directs otherwise.
    pub store_root: PathBuf,
    /// The `[store]` section, if the locator carried one.
    pub location: Option<StoreLocation>,
    /// The file the operational settings were read from, if one exists —
    /// `config.toml` in the store, else the locator itself.
    pub operational_path: Option<PathBuf>,
}

impl StoreHome {
    /// Resolves the store home starting from `start` (typically the current
    /// directory).
    ///
    /// Never fails and never touches the store: an unreadable or malformed
    /// locator resolves to the un-redirected default rather than refusing to
    /// run, because a broken `[store]` must not make the corpus unreachable.
    /// Callers that need to *report* a malformed config get that from
    /// [`crate::StoreConfig::load`], which parses the same file strictly.
    #[must_use]
    pub fn resolve(start: &Path) -> Self {
        let locator = find_locator(start);
        let repo_root = locator
            .as_ref()
            .and_then(|p| p.parent())
            .map_or_else(|| start.to_path_buf(), Path::to_path_buf);

        let location = locator
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|text| toml::from_str::<LocatorFile>(&text).ok())
            .and_then(|parsed| parsed.store);

        // No `[store]` ⇒ the store is the repo root, exactly as before.
        let store_root =
            location.as_ref().map_or_else(|| repo_root.clone(), |loc| loc.store_root(&repo_root));

        // The store's own `config.toml` wins; failing that, the locator doubles
        // as the operational config (the pre-migration arrangement).
        let in_store = store_root.join(OPERATIONAL_FILE);
        let operational_path = if in_store.is_file() { Some(in_store) } else { locator };

        Self { repo_root, store_root, location, operational_path }
    }

    /// The operational config's text, or an empty string when there is none.
    ///
    /// Returned as text because its readers parse different slices of it —
    /// `odm-core` takes the gate-sets and the satisfaction threshold, `odm-cli`
    /// the display settings — and each already parses from a string.
    #[must_use]
    pub fn operational_text(&self) -> String {
        self.operational_path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default()
    }

    /// Whether the store is redirected by a `[store]` section, as opposed to
    /// sitting at the repo root.
    #[must_use]
    pub fn is_redirected(&self) -> bool {
        self.location.is_some()
    }
}

/// Finds the locator file by the layered search: `start`, then the enclosing
/// repository root, then the user config directory.
fn find_locator(start: &Path) -> Option<PathBuf> {
    let mut candidates = vec![start.to_path_buf()];
    if let Some(root) = crate::config::repo_root(start) {
        candidates.push(root);
    }
    if let Some(user) = crate::config::user_config_dir() {
        candidates.push(user);
    }
    candidates.into_iter().map(|dir| dir.join(LOCATOR_FILE)).find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// A repo with an `odm.toml` containing `body`.
    fn repo_with(body: &str) -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(LOCATOR_FILE), body).unwrap();
        dir
    }

    // ----- L-1: the `[store]` section parses, all fields optional ------------

    #[test]
    fn test_store_section_parses_with_every_field() {
        let dir = repo_with(
            "[store]\nworktree_base = \"wt\"\nworktree_name = \"planning\"\nbranch_name = \"plan\"\n",
        );
        let home = StoreHome::resolve(dir.path());
        let loc = home.location.expect("the section is present");
        assert_eq!(loc.worktree_base, "wt");
        assert_eq!(loc.worktree_name, "planning");
        assert_eq!(loc.branch_name, "plan");
    }

    #[test]
    fn test_store_section_fields_default_individually() {
        // A bare `[store]` is enough: every field has a documented default.
        let home = StoreHome::resolve(repo_with("[store]\n").path());
        let loc = home.location.expect("the section is present");
        assert_eq!(loc.worktree_base, DEFAULT_WORKTREE_BASE);
        assert_eq!(loc.worktree_name, DEFAULT_STORE_NAME);
        assert_eq!(loc.branch_name, DEFAULT_STORE_NAME);

        // …and one named field does not drag the others to empty.
        let home = StoreHome::resolve(repo_with("[store]\nbranch_name = \"plan\"\n").path());
        let loc = home.location.unwrap();
        assert_eq!(loc.branch_name, "plan");
        assert_eq!(loc.worktree_base, DEFAULT_WORKTREE_BASE);
        assert_eq!(loc.worktree_name, DEFAULT_STORE_NAME);
    }

    // ----- L-2: `[store]` present ⇒ resolve to the worktree ------------------

    #[test]
    fn test_store_root_resolves_to_the_worktree_when_directed() {
        let dir = repo_with("[store]\n");
        let home = StoreHome::resolve(dir.path());
        assert_eq!(home.store_root, dir.path().join(".worktrees").join("odm"));
        assert!(home.is_redirected());
    }

    #[test]
    fn test_store_root_honours_custom_names() {
        let dir = repo_with("[store]\nworktree_base = \"wt\"\nworktree_name = \"planning\"\n");
        let home = StoreHome::resolve(dir.path());
        assert_eq!(home.store_root, dir.path().join("wt").join("planning"));
    }

    // ----- L-3: no `[store]` ⇒ the repo root, unchanged ----------------------

    #[test]
    fn test_store_root_is_the_repo_root_without_a_store_section() {
        let dir = repo_with("author_name = \"Ada\"\n");
        let home = StoreHome::resolve(dir.path());
        assert_eq!(home.store_root, dir.path());
        assert!(!home.is_redirected());
    }

    #[test]
    fn test_no_locator_at_all_resolves_to_the_starting_directory() {
        let dir = TempDir::new().unwrap();
        let home = StoreHome::resolve(dir.path());
        assert_eq!(home.store_root, dir.path());
        assert_eq!(home.operational_path, None);
        assert_eq!(home.operational_text(), "");
    }

    // ----- L-4 / L-5: operational config, and its fallback -------------------

    #[test]
    fn test_operational_config_comes_from_the_store_when_present() {
        let dir = repo_with("[store]\n");
        let store = dir.path().join(".worktrees").join("odm");
        fs::create_dir_all(&store).unwrap();
        fs::write(store.join(OPERATIONAL_FILE), "author_name = \"FromStore\"\n").unwrap();

        let home = StoreHome::resolve(dir.path());
        assert_eq!(home.operational_path.as_deref(), Some(store.join(OPERATIONAL_FILE).as_path()));
        assert!(home.operational_text().contains("FromStore"));
    }

    #[test]
    fn test_operational_config_falls_back_to_the_locator() {
        // The pre-migration arrangement: no `config.toml` in the store, so
        // `odm.toml` is still where the operational settings live.
        let dir = repo_with("[store]\nauthor_name = \"FromLocator\"\n");
        let home = StoreHome::resolve(dir.path());
        assert_eq!(home.operational_path.as_deref(), Some(dir.path().join(LOCATOR_FILE).as_path()));
        assert!(home.operational_text().contains("FromLocator"));
    }

    #[test]
    fn test_a_malformed_locator_still_resolves_rather_than_wedging() {
        // A broken `[store]` must not make the corpus unreachable; strict
        // parsing (and its error) belongs to `StoreConfig::load`.
        let dir = repo_with("this is := not valid toml ==");
        let home = StoreHome::resolve(dir.path());
        assert_eq!(home.store_root, dir.path());
        assert!(!home.is_redirected());
    }

    #[test]
    fn test_resolution_finds_the_locator_from_a_subdirectory() {
        let dir = repo_with("[store]\n");
        let deep = dir.path().join("nodes").join("2026");
        fs::create_dir_all(&deep).unwrap();

        let home = StoreHome::resolve(&deep);
        assert_eq!(home.repo_root, dir.path());
        assert_eq!(home.store_root, dir.path().join(".worktrees").join("odm"));
    }
}
