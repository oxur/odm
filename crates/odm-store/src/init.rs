//! Standing up a store home: `odm store init`, bootstrap arm (ODD-0022 §4.3).
//!
//! Bootstrap is the *no store anywhere* case. The other two arms — **attach**
//! (a branch exists on a remote) and **sync** (a store already exists locally)
//! — are slice 03, and this module stops rather than guessing when it sees
//! them: re-orphaning a branch that already carries someone's planning data
//! would destroy it, so the detection is a safety boundary, not a convenience.
//!
//! The steps, in order, are ODD-0022 §4.3's:
//!
//! 1. worktree + orphan branch (via [`crate::worktree`], the one git shell-out)
//! 2. `[store]` written to the code-branch `odm.toml` — after which slice 01's
//!    resolution redirects every command into the new home
//! 3. the store scaffolded *inside* the worktree: `config.toml` + empty `nodes/`
//! 4. `/.worktrees/` git-ignored on the code branch
//!
//! Ordering matters: the locator is written only once the worktree exists, so a
//! failed `init` never leaves an `odm.toml` pointing at nothing.

use std::path::{Path, PathBuf};

use crate::error::{Result, StoreError};
use crate::home::{
    DEFAULT_STORE_NAME, DEFAULT_WORKTREE_BASE, LOCATOR_FILE, OPERATIONAL_FILE, StoreLocation,
};
use crate::worktree::{self, GitVersion};

/// The operational config a fresh store is scaffolded with — the gate-sets
/// ODD-0013 §5.1 defines, plus the display default.
///
/// This is the *store's* config from birth (`config.toml` inside the worktree);
/// the code-branch `odm.toml` stays locator-only, so the two never hold
/// overlapping keys that could disagree.
pub const DEFAULT_OPERATIONAL_CONFIG: &str = "\
# odm store configuration — the operational half (ODD-0022 §4.2).
#
# This file lives *in* the store, so it is versioned with the data it governs
# and cannot drift from it. The code branch's `odm.toml` only says where the
# store is.

# Where design documents live, for `odm migrate`.
docs_directory = \"./docs/design\"

# Per-type gate-sets (ODD-0013 §5.1).
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]

[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]

[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]

[gates.design]
sequence = [\"draft\", \"under-review\", \"revised\", \"accepted\", \"active\", \"final\"]

[gates.research]
sequence = [\"draft\", \"under-review\", \"revised\", \"accepted\", \"active\", \"final\"]

# Display preferences: `max_width` bounds `odm list`'s NAME column.
[display]
max_width = 64
";

/// The gitignore entry keeping the worktree off the code branch.
const GITIGNORE_ENTRY: &str = "/.worktrees/";

/// What `init` found, and therefore which arm applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// No `<branch>` anywhere — stand the home up from nothing.
    Bootstrap,
    /// A local branch already exists: the store is here, so this is a **sync**
    /// (slice 03), not a bootstrap.
    ExistsLocally,
    /// Only a remote has it: this is an **attach** (slice 03).
    ExistsOnRemote(String),
}

impl Mode {
    /// Whether this is the arm slice 02 implements.
    #[must_use]
    pub fn is_bootstrap(&self) -> bool {
        matches!(self, Mode::Bootstrap)
    }
}

/// Classifies the repo at `repo_root` for a store branch named `branch`, whose
/// home would be `store_root`.
///
/// A pure read, so it stays on `gix`: the shell-out exception is for *creating*
/// a worktree, and widening it to reads would erode the boundary it is supposed
/// to hold. A repo `gix` cannot open is treated as a bootstrap candidate — the
/// worktree step will produce git's own error if it truly is not a repo, which
/// is a better message than anything invented here.
///
/// **An existing store root counts as "exists locally", even with no ref.** A
/// freshly created orphan branch is *unborn*: it has no commit, so
/// `refs/heads/<branch>` does not exist until the first write, and a
/// refs-only check would therefore fail to see the very home `init` just
/// created — and then try to create it again. The directory is the reliable
/// signal for that window; the ref is the reliable signal afterwards. Both are
/// checked, because either alone has a blind spot.
#[must_use]
pub fn detect(repo_root: &Path, store_root: &Path, branch: &str) -> Mode {
    if store_root.exists() {
        return Mode::ExistsLocally;
    }
    let Ok(repo) = gix::open(repo_root) else {
        return Mode::Bootstrap;
    };
    if repo.try_find_reference(&format!("refs/heads/{branch}")).ok().flatten().is_some() {
        return Mode::ExistsLocally;
    }
    // Any remote carrying the branch means someone else's store exists.
    if let Ok(platform) = repo.references()
        && let Ok(iter) = platform.prefixed("refs/remotes/")
    {
        for reference in iter.flatten() {
            let name = reference.name().as_bstr().to_string();
            if name.ends_with(&format!("/{branch}")) {
                return Mode::ExistsOnRemote(name);
            }
        }
    }
    Mode::Bootstrap
}

/// What a bootstrap did, or — under [`Plan::dry_run`] — would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bootstrapped {
    /// The store root: `<repo>/<worktree_base>/<worktree_name>`.
    pub store_root: PathBuf,
    /// The orphan branch created there.
    pub branch: String,
    /// The `[store]` locator written to the code branch.
    pub location: StoreLocation,
    /// The git that ran, when one did (`None` under a dry run).
    pub git_version: Option<GitVersion>,
}

/// The inputs to a bootstrap.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The repository root — where the locator and `.gitignore` are written.
    pub repo_root: PathBuf,
    /// The names to use; defaults when the caller passes none.
    pub location: StoreLocation,
    /// Report what would happen and write nothing.
    pub dry_run: bool,
}

impl Plan {
    /// A bootstrap of `repo_root` with the default names, overridden where the
    /// caller supplies one.
    #[must_use]
    pub fn new(
        repo_root: impl Into<PathBuf>,
        worktree: Option<&str>,
        branch: Option<&str>,
    ) -> Self {
        let name = worktree.unwrap_or(DEFAULT_STORE_NAME).to_string();
        Self {
            repo_root: repo_root.into(),
            location: StoreLocation {
                worktree_base: DEFAULT_WORKTREE_BASE.to_string(),
                worktree_name: name,
                branch_name: branch.unwrap_or(DEFAULT_STORE_NAME).to_string(),
            },
            dry_run: false,
        }
    }

    /// The store root this plan would create.
    #[must_use]
    pub fn store_root(&self) -> PathBuf {
        self.location.store_root(&self.repo_root)
    }
}

/// Runs the bootstrap.
///
/// # Errors
///
/// [`StoreError::Git`] if the worktree cannot be created (git missing, or its
/// own failure), or [`StoreError::Io`] if a scaffolded file cannot be written.
/// Returns an error rather than proceeding if the store root already exists —
/// `init` creates a home, it never writes into one it did not make.
pub fn bootstrap(plan: &Plan) -> Result<Bootstrapped> {
    let store_root = plan.store_root();

    if plan.dry_run {
        return Ok(Bootstrapped {
            store_root,
            branch: plan.location.branch_name.clone(),
            location: plan.location.clone(),
            git_version: None,
        });
    }

    if store_root.exists() {
        return Err(StoreError::Git(format!(
            "{} already exists; `odm store init` will not write into a directory it did not \
             create — remove it, or choose another name with --worktree",
            store_root.display()
        )));
    }

    // 1. The worktree + orphan branch. First, because everything after it is
    //    only meaningful once the home exists.
    if let Some(parent) = store_root.parent() {
        std::fs::create_dir_all(parent).map_err(|e| StoreError::io(parent, e))?;
    }
    let git_version = worktree::create(&plan.repo_root, &store_root, &plan.location.branch_name)?;

    // 2. The locator — now that there is something to point at.
    write_locator(&plan.repo_root, &plan.location)?;

    // 3. The store's own config + an empty `nodes/`.
    let config = store_root.join(OPERATIONAL_FILE);
    std::fs::write(&config, DEFAULT_OPERATIONAL_CONFIG).map_err(|e| StoreError::io(&config, e))?;
    let nodes = store_root.join(crate::layout::NODES_DIR);
    std::fs::create_dir_all(&nodes).map_err(|e| StoreError::io(&nodes, e))?;

    // 4. Keep the worktree off the code branch.
    ensure_gitignored(&plan.repo_root)?;

    Ok(Bootstrapped {
        store_root,
        branch: plan.location.branch_name.clone(),
        location: plan.location.clone(),
        git_version: Some(git_version),
    })
}

/// Appends the `[store]` section to the code-branch `odm.toml`, preserving
/// whatever else is there.
///
/// Append rather than rewrite: the locator file may already carry operational
/// settings (every pre-migration repo does), and `init` has no business
/// discarding them — the C-5 cutover is what moves them into the store.
fn write_locator(repo_root: &Path, location: &StoreLocation) -> Result<()> {
    let path = repo_root.join(LOCATOR_FILE);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut text = existing;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    if !text.is_empty() {
        text.push('\n');
    }
    text.push_str(&format!(
        "# Where the odm store lives (ODD-0022 §4.2). Written by `odm store init`.\n\
         [store]\n\
         worktree_base = {:?}\n\
         worktree_name = {:?}\n\
         branch_name = {:?}\n",
        location.worktree_base, location.worktree_name, location.branch_name
    ));
    std::fs::write(&path, text).map_err(|e| StoreError::io(&path, e))
}

/// Ensures the code branch ignores the worktree directory. Idempotent: a
/// `.gitignore` already carrying the entry is left untouched, so re-running
/// `init` cannot accumulate duplicates.
fn ensure_gitignored(repo_root: &Path) -> Result<()> {
    let path = repo_root.join(".gitignore");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if existing.lines().any(|line| line.trim() == GITIGNORE_ENTRY) {
        return Ok(());
    }
    let mut text = existing;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&format!(
        "\n# odm's store worktree — a separate branch, never staged here.\n{GITIGNORE_ENTRY}\n"
    ));
    std::fs::write(&path, text).map_err(|e| StoreError::io(&path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plan_defaults_and_overrides() {
        let plan = Plan::new("/repo", None, None);
        assert_eq!(plan.location.worktree_name, "odm");
        assert_eq!(plan.location.branch_name, "odm");
        assert_eq!(plan.store_root(), Path::new("/repo/.worktrees/odm"));

        let plan = Plan::new("/repo", Some("planning"), Some("plan"));
        assert_eq!(plan.store_root(), Path::new("/repo/.worktrees/planning"));
        assert_eq!(plan.location.branch_name, "plan");
    }

    #[test]
    fn test_locator_append_preserves_existing_keys() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(LOCATOR_FILE), "author_name = \"Ada\"\n").unwrap();
        write_locator(dir.path(), &StoreLocation::default()).unwrap();

        let text = std::fs::read_to_string(dir.path().join(LOCATOR_FILE)).unwrap();
        assert!(text.contains("author_name = \"Ada\""), "existing keys survive:\n{text}");
        assert!(text.contains("[store]"), "the locator was added:\n{text}");
        assert!(text.contains("branch_name = \"odm\""), "with its names:\n{text}");
    }

    #[test]
    fn test_gitignore_entry_is_idempotent() {
        let dir = TempDir::new().unwrap();
        ensure_gitignored(dir.path()).unwrap();
        let once = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        ensure_gitignored(dir.path()).unwrap();
        let twice = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();

        assert_eq!(once, twice, "a second run adds nothing");
        assert_eq!(twice.matches(GITIGNORE_ENTRY).count(), 1, "exactly one entry:\n{twice}");
    }

    #[test]
    fn test_gitignore_appends_to_an_existing_file() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(".gitignore"), "/target\n").unwrap();
        ensure_gitignored(dir.path()).unwrap();
        let text = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(text.contains("/target"), "existing rules survive:\n{text}");
        assert!(text.contains(GITIGNORE_ENTRY));
    }

    #[test]
    fn test_dry_run_reports_without_touching_anything() {
        let dir = TempDir::new().unwrap();
        let mut plan = Plan::new(dir.path(), None, None);
        plan.dry_run = true;

        let out = bootstrap(&plan).unwrap();
        assert_eq!(out.store_root, dir.path().join(".worktrees").join("odm"));
        assert_eq!(out.git_version, None, "a dry run does not even ask git");
        assert!(!out.store_root.exists(), "nothing created");
        assert!(!dir.path().join(LOCATOR_FILE).exists(), "no locator written");
        assert!(!dir.path().join(".gitignore").exists(), "no gitignore written");
    }

    #[test]
    fn test_bootstrap_refuses_an_existing_store_root() {
        let dir = TempDir::new().unwrap();
        let plan = Plan::new(dir.path(), None, None);
        std::fs::create_dir_all(plan.store_root()).unwrap();

        let err = bootstrap(&plan).unwrap_err();
        assert!(
            err.to_string().contains("already exists"),
            "it refuses rather than writing in: {err}"
        );
    }

    #[test]
    fn test_detect_reports_bootstrap_for_a_repo_without_the_branch() {
        // A non-repo also classifies as bootstrap: git itself will produce the
        // authoritative error if it is not one.
        let dir = TempDir::new().unwrap();
        let store = dir.path().join(".worktrees").join("odm");
        assert_eq!(detect(dir.path(), &store, "odm"), Mode::Bootstrap);
        assert!(Mode::Bootstrap.is_bootstrap());
        assert!(!Mode::ExistsLocally.is_bootstrap());
        assert!(!Mode::ExistsOnRemote("origin/odm".into()).is_bootstrap());
    }

    #[test]
    fn test_detect_sees_an_existing_store_even_with_an_unborn_branch() {
        // The window `init` itself creates: the orphan branch has no commit
        // yet, so no `refs/heads/<branch>` exists — but the store plainly does.
        let dir = TempDir::new().unwrap();
        let store = dir.path().join(".worktrees").join("odm");
        std::fs::create_dir_all(&store).unwrap();
        assert_eq!(detect(dir.path(), &store, "odm"), Mode::ExistsLocally);
    }
}
