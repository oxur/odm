//! The **only** module that runs the `git` binary (ODD-0022 §5).
//!
//! ## Why the exception exists
//!
//! Everything else in odm talks to git through `gix`, in-process. But `gix`
//! 0.66 does not implement `git worktree add`, and creating a worktree with an
//! orphan branch is precisely what standing up a store home requires. Rather
//! than hand-roll worktree administration against `gix`'s plumbing — writing
//! `.git/worktrees/<name>/` metadata, the `gitdir`/`commondir` links, and the
//! unborn-HEAD dance ourselves — this one operation shells out to the `git`
//! the user already has.
//!
//! The exception is deliberately narrow: **setup only**. Every steady-state
//! read and write stays on `gix` (ledger L-14), so the subprocess dependency
//! exists at `init` and never during ordinary use. If `gix` grows worktree
//! support, this module is the only thing to delete.
//!
//! ## The two paths
//!
//! `git worktree add --orphan` landed in **git 2.42**. Below that, the same end
//! state is reached in two steps — add a detached worktree, then orphan the
//! branch inside it — so odm works on stock and LTS git rather than requiring a
//! recent one. The version is detected once and mapped to a [`Plan`], which is
//! a pure function and therefore testable without running anything.
//!
//! Either way the branch is left **unborn and empty**: no commit, no tree,
//! nothing shared with the code branch. The first `odm` write is what gives it
//! a root commit, and that is what makes the history disjoint by construction
//! rather than by convention.
//!
//! *Empty* takes an extra step on the old path. `git checkout --orphan`
//! deliberately **keeps the working tree and index** — it is designed for
//! "start a new branch from these files" — so on its own it would leave the
//! code branch's entire checkout staged in the store. `git rm -rf .` clears
//! both, matching what `worktree add --orphan` does natively. Without it the
//! two paths produce different stores, which is exactly the kind of divergence
//! a fallback must not have.

use std::path::Path;
use std::process::Command;

use crate::error::{Result, StoreError};

/// The first git release with `worktree add --orphan`.
const ORPHAN_FLAG_SINCE: GitVersion = GitVersion { major: 2, minor: 42 };

/// A git version, to the precision this module cares about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GitVersion {
    /// The major version.
    pub major: u32,
    /// The minor version.
    pub minor: u32,
}

impl GitVersion {
    /// Parses the `git --version` line (`git version 2.39.5 (Apple Git-154)`).
    ///
    /// Tolerant of the vendor suffixes real installations carry: it takes the
    /// first dotted number it finds and reads two components from it.
    #[must_use]
    pub fn parse(output: &str) -> Option<Self> {
        let token =
            output.split_whitespace().find(|t| t.starts_with(|c: char| c.is_ascii_digit()))?;
        let mut parts = token.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next().unwrap_or("0").parse().unwrap_or(0);
        Some(Self { major, minor })
    }

    /// Whether this version supports `worktree add --orphan`.
    #[must_use]
    pub fn has_orphan_flag(self) -> bool {
        self >= ORPHAN_FLAG_SINCE
    }
}

impl std::fmt::Display for GitVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// How to create the worktree, given the available git.
///
/// Kept as data rather than branching inside the runner so the mapping can be
/// asserted in a unit test without a git binary present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// git ≥ 2.42: one command does it.
    Orphan {
        /// `worktree add --orphan <branch> <dir>`.
        add: Vec<String>,
    },
    /// git < 2.42: add a detached worktree, orphan the branch inside it, then
    /// clear the checkout that `--orphan` carries over.
    DetachThenOrphan {
        /// `worktree add --detach <dir>`, run in the repo.
        add: Vec<String>,
        /// `checkout --orphan <branch>`, run **in the worktree**.
        checkout: Vec<String>,
        /// `rm -rf .`, run in the worktree — empties the index and the tree
        /// that `checkout --orphan` preserves. `--ignore-unmatch` keeps an
        /// already-empty checkout from being an error.
        clear: Vec<String>,
    },
}

/// The argv (after `git`) for creating `dir` as a worktree holding an orphan
/// `branch`, chosen for `version`.
#[must_use]
pub fn plan(version: GitVersion, dir: &Path, branch: &str) -> Plan {
    let dir = dir.to_string_lossy().into_owned();
    if version.has_orphan_flag() {
        Plan::Orphan {
            add: vec!["worktree".into(), "add".into(), "--orphan".into(), branch.into(), dir],
        }
    } else {
        Plan::DetachThenOrphan {
            add: vec!["worktree".into(), "add".into(), "--detach".into(), dir],
            checkout: vec!["checkout".into(), "--orphan".into(), branch.into()],
            clear: vec![
                "rm".into(),
                "-rf".into(),
                "--ignore-unmatch".into(),
                "--quiet".into(),
                ".".into(),
            ],
        }
    }
}

/// The installed git's version.
///
/// # Errors
///
/// [`StoreError::Git`] naming the fix if `git` cannot be run or its version
/// cannot be read — never a panic, because "no git on PATH" is a user
/// situation with an obvious remedy, not a bug.
pub fn version() -> Result<GitVersion> {
    let output = Command::new("git").arg("--version").output().map_err(|e| {
        StoreError::Git(format!(
            "could not run `git` ({e}). `odm store init` needs the git binary to create the \
             store's worktree — install git, or ensure it is on PATH"
        ))
    })?;
    let text = String::from_utf8_lossy(&output.stdout);
    GitVersion::parse(&text).ok_or_else(|| {
        StoreError::Git(format!("could not read a version from `git --version` output: {text:?}"))
    })
}

/// Creates `dir` as a git worktree of the repository at `repo_root`, holding a
/// newly-orphaned `branch`.
///
/// The branch is left **unborn** — no commit — so its history is disjoint from
/// every code branch by construction.
///
/// # Errors
///
/// [`StoreError::Git`] if git is absent, too old to offer either path, or the
/// command fails; the message carries git's own stderr, which is usually the
/// most useful thing to show.
pub fn create(repo_root: &Path, dir: &Path, branch: &str) -> Result<GitVersion> {
    let version = version()?;
    match plan(version, dir, branch) {
        Plan::Orphan { add } => run(repo_root, &add)?,
        Plan::DetachThenOrphan { add, checkout, clear } => {
            run(repo_root, &add)?;
            // Both of these run *inside* the new worktree: that is what makes
            // the branch belong to it rather than to the repo's checkout.
            run(dir, &checkout)?;
            // `checkout --orphan` keeps the files it came with; drop them so
            // the store starts empty, as the modern path does.
            run(dir, &clear)?;
        }
    }
    Ok(version)
}

/// Runs `git` with `args` in `cwd`, mapping a non-zero exit to an error that
/// carries git's stderr.
fn run(cwd: &Path, args: &[String]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| StoreError::Git(format!("running `git {}`: {e}", args.join(" "))))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(StoreError::Git(format!("`git {}` failed: {}", args.join(" "), stderr.trim())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ----- L-2: the version → argv mapping ----------------------------------

    #[test]
    fn test_modern_git_uses_the_single_orphan_command() {
        let p = plan(GitVersion { major: 2, minor: 42 }, &PathBuf::from("/w/odm"), "odm");
        assert_eq!(
            p,
            Plan::Orphan {
                add: vec![
                    "worktree".into(),
                    "add".into(),
                    "--orphan".into(),
                    "odm".into(),
                    "/w/odm".into()
                ]
            }
        );
    }

    #[test]
    fn test_old_git_falls_back_to_detach_then_orphan() {
        let p = plan(GitVersion { major: 2, minor: 39 }, &PathBuf::from("/w/odm"), "plan");
        let Plan::DetachThenOrphan { add, checkout, clear } = p else {
            panic!("git 2.39 must take the fallback");
        };
        assert_eq!(add, vec!["worktree", "add", "--detach", "/w/odm"]);
        assert_eq!(checkout, vec!["checkout", "--orphan", "plan"]);
        // The step that makes the fallback match the modern path: `--orphan`
        // keeps the old checkout, so it has to be cleared.
        assert_eq!(clear, vec!["rm", "-rf", "--ignore-unmatch", "--quiet", "."]);
    }

    #[test]
    fn test_the_boundary_is_2_42() {
        assert!(!GitVersion { major: 2, minor: 41 }.has_orphan_flag());
        assert!(GitVersion { major: 2, minor: 42 }.has_orphan_flag());
        assert!(GitVersion { major: 3, minor: 0 }.has_orphan_flag());
    }

    // ----- version parsing tolerates real-world output ----------------------

    #[test]
    fn test_version_parses_vendor_suffixed_output() {
        assert_eq!(
            GitVersion::parse("git version 2.39.5 (Apple Git-154)"),
            Some(GitVersion { major: 2, minor: 39 })
        );
        assert_eq!(
            GitVersion::parse("git version 2.45.2"),
            Some(GitVersion { major: 2, minor: 45 })
        );
        assert_eq!(GitVersion::parse("git version 3"), Some(GitVersion { major: 3, minor: 0 }));
        assert_eq!(GitVersion::parse("no version here"), None);
    }

    #[test]
    fn test_versions_order_by_major_then_minor() {
        assert!(GitVersion { major: 2, minor: 9 } < GitVersion { major: 2, minor: 42 });
        assert!(GitVersion { major: 2, minor: 42 } < GitVersion { major: 3, minor: 0 });
    }
}
