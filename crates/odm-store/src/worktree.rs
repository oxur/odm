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
//! The exception is deliberately narrow: **`init`-time only**. Every
//! steady-state read and write stays on `gix`, so the subprocess dependency
//! exists at `init` and never during ordinary use. If `gix` grows worktree
//! support, this module is the only thing to delete.
//!
//! **Scope widened deliberately, twice.** §5 originally ratified the exception
//! for *creating* a worktree. Attach and ff-sync (slice 03) added
//! `worktree add <dir> <branch>`, `fetch` and `merge --ff-only`; rename (slice
//! 04) adds `worktree move` and `branch -m`, both named in §5. They live here
//! rather than anywhere else so the boundary stays one module wide, and the
//! invariant that matters is unchanged: **setup-time only (`init`/`rename`),
//! steady state on `gix`**.
//!
//! ## The two paths
//!
//! `git worktree add --orphan` landed in **git 2.42**, where the branch is
//! named with `-b` (the synopsis is
//! `worktree add … [--orphan] [(-b|-B) <branch>] <path>` — the branch is never
//! a bare positional). Below 2.42, the same end state is reached in two steps — add a detached worktree, then orphan the
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

use std::path::{Path, PathBuf};
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
            // `-b` is not optional decoration: in the 2.42+ synopsis
            // (`worktree add … [--orphan] [(-b|-B) <branch>] <path>`) the branch
            // is *never* a bare positional. Without it git reads the branch as
            // `<path>` and the dir as `<commit-ish>`, and refuses:
            // "fatal: '--orphan' and '<commit-ish>' cannot be used together".
            add: vec![
                "worktree".into(),
                "add".into(),
                "--orphan".into(),
                "-b".into(),
                branch.into(),
                dir,
            ],
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

/// Adds `dir` as a worktree checking out an **existing** `branch`.
///
/// Deliberately *not* `--orphan`: the branch already carries someone's store,
/// and re-orphaning it would discard that history. Not version-gated either —
/// plain `worktree add` long predates 2.42.
///
/// # Errors
///
/// [`StoreError::Git`] if git is absent or the command fails.
pub fn attach(repo_root: &Path, dir: &Path, branch: &str) -> Result<()> {
    run(
        repo_root,
        &["worktree".into(), "add".into(), dir.to_string_lossy().into_owned(), branch.into()],
    )
}

/// Fetches `remote`, so ancestry is judged against a current upstream.
///
/// # Errors
///
/// [`StoreError::Git`] if the fetch fails (no network, no such remote).
pub fn fetch(repo_root: &Path, remote: &str) -> Result<()> {
    run(repo_root, &["fetch".into(), remote.into()])
}

/// Fast-forwards the branch checked out in `worktree_dir`.
///
/// `--ff-only` is the whole point: it fails rather than creating a merge
/// commit, so a sync can never rewrite or merge a published branch.
///
/// # Errors
///
/// [`StoreError::Git`] if the merge is not a fast-forward, or git fails.
pub fn merge_ff_only(worktree_dir: &Path) -> Result<()> {
    run(worktree_dir, &["merge".into(), "--ff-only".into()])
}

/// Resolves `rev` to a commit id, or `None` when it does not exist.
///
/// A missing ref is a normal answer here — "there is no upstream" is a state to
/// report, not a failure — so it is `None` rather than an error.
///
/// # Errors
///
/// [`StoreError::Git`] only if git itself cannot be run.
pub fn rev_parse(cwd: &Path, rev: &str) -> Result<Option<String>> {
    let output =
        capture(cwd, &["rev-parse".into(), "--verify".into(), "--quiet".into(), rev.into()])?;
    Ok(output.map(|text| text.trim().to_string()).filter(|id| !id.is_empty()))
}

/// Whether `ancestor` is an ancestor of `descendant` (a commit is its own).
///
/// # Errors
///
/// [`StoreError::Git`] if git cannot be run. A `false` answer is git's exit
/// code 1, not an error.
pub fn is_ancestor(cwd: &Path, ancestor: &str, descendant: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .current_dir(cwd)
        .output()
        .map_err(|e| StoreError::Git(format!("running `git merge-base`: {e}")))?;
    Ok(output.status.success())
}

/// How many commits `range` (e.g. `upstream..local`) contains.
///
/// # Errors
///
/// [`StoreError::Git`] if git cannot be run.
pub fn count_commits(cwd: &Path, range: &str) -> Result<usize> {
    let output = capture(cwd, &["rev-list".into(), "--count".into(), range.into()])?;
    Ok(output.and_then(|t| t.trim().parse().ok()).unwrap_or(0))
}

/// Moves a worktree from `old` to `new`.
///
/// **`git worktree move` behaves like `mv`:** given an *existing* directory as
/// the destination it moves the worktree *inside* it (`new/<basename>`) rather
/// than *to* it, and reports success. So the caller must reject a target that
/// already exists before calling this — git will not do it — and must read the
/// resulting location back rather than assume it (see [`path_of_branch`]).
///
/// # Errors
///
/// [`StoreError::Git`] if git is absent or the move fails.
pub fn move_worktree(repo_root: &Path, old: &Path, new: &Path) -> Result<()> {
    run(
        repo_root,
        &[
            "worktree".into(),
            "move".into(),
            old.to_string_lossy().into_owned(),
            new.to_string_lossy().into_owned(),
        ],
    )
}

/// Renames a **local** branch. The upstream configuration follows it; the
/// remote's own branch is untouched.
///
/// Works on an unborn branch too — the case a freshly bootstrapped store is in
/// — because it rewrites the symbolic HEAD rather than moving a ref.
///
/// # Errors
///
/// [`StoreError::Git`] if git is absent or the rename fails (no such branch,
/// or the target name is taken).
pub fn rename_branch(cwd: &Path, old: &str, new: &str) -> Result<()> {
    run(cwd, &["branch".into(), "-m".into(), old.into(), new.into()])
}

/// Where git says the worktree holding `branch` actually is.
///
/// Parsed from `worktree list --porcelain`, which names the branch even when it
/// is unborn. This is how a rename learns the *observed* state rather than
/// trusting the path it asked for — the difference matters precisely because
/// `worktree move` can put the tree somewhere other than the argument given.
///
/// # Errors
///
/// [`StoreError::Git`] if git cannot be run.
pub fn path_of_branch(repo_root: &Path, branch: &str) -> Result<Option<PathBuf>> {
    let Some(text) = capture(repo_root, &["worktree".into(), "list".into(), "--porcelain".into()])?
    else {
        return Ok(None);
    };
    let wanted = format!("branch refs/heads/{branch}");
    let mut current: Option<&str> = None;
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current = Some(path);
        } else if line.trim() == wanted {
            return Ok(current.map(PathBuf::from));
        }
    }
    Ok(None)
}

/// Whether a local branch of this name exists.
///
/// # Errors
///
/// [`StoreError::Git`] if git cannot be run.
pub fn branch_exists(repo_root: &Path, branch: &str) -> Result<bool> {
    Ok(rev_parse(repo_root, &format!("refs/heads/{branch}"))?.is_some()
        || path_of_branch(repo_root, branch)?.is_some())
}

/// The branch currently checked out in `worktree_dir`, if any.
///
/// # Errors
///
/// [`StoreError::Git`] if git cannot be run.
pub fn current_branch(worktree_dir: &Path) -> Result<Option<String>> {
    let out = capture(worktree_dir, &["branch".into(), "--show-current".into()])?;
    Ok(out.map(|t| t.trim().to_string()).filter(|b| !b.is_empty()))
}

/// Whether `branch` has an upstream configured, or a remote carries it — the
/// signal that renaming it locally will desynchronise a shared branch.
///
/// # Errors
///
/// [`StoreError::Git`] if git cannot be run.
pub fn is_published(repo_root: &Path, branch: &str) -> Result<bool> {
    if rev_parse(repo_root, &format!("{branch}@{{upstream}}"))?.is_some() {
        return Ok(true);
    }
    let Some(text) = capture(
        repo_root,
        &["for-each-ref".into(), "--format=%(refname)".into(), "refs/remotes/".into()],
    )?
    else {
        return Ok(false);
    };
    Ok(text.lines().any(|r| r.trim().ends_with(&format!("/{branch}"))))
}

/// Runs `git`, returning its stdout, or `None` when it exits non-zero.
///
/// Used for the queries whose "failure" is a legitimate answer — an absent ref,
/// an empty range — where an error would force every caller to re-interpret it.
fn capture(cwd: &Path, args: &[String]) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| StoreError::Git(format!("running `git {}`: {e}", args.join(" "))))?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()))
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
                    // The branch is named with `-b`; as a bare positional git
                    // reads it as the path and rejects the command.
                    "-b".into(),
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
