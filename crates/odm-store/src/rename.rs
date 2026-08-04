//! `odm store rename` — moving the store's home without losing it
//! (ODD-0022 §4.5).
//!
//! The store root is *derived*: `<repo>/<worktree_base>/<worktree_name>`, with
//! the branch named alongside it in the same `[store]` locator (slice 01). So a
//! rename is never one operation — it is a git-side move and a locator rewrite
//! that must agree, and the whole risk of this slice is that they might not.
//!
//! ## The always-resolvable invariant
//!
//! The order is: **validate → git → write the locator last, from what git
//! actually did**.
//!
//! - A failure *before* any git op changes nothing, and the old locator is
//!   still correct.
//! - A failure *between* two git ops leaves the locator written to the
//!   **observed** state, not the intended one — so it still points at a store
//!   that exists, and the report says which half succeeded.
//!
//! Mirroring observation rather than intent is not defensive padding: `git
//! worktree move` behaves like `mv`, so given an existing destination it puts
//! the tree at `<dest>/<basename>` and reports success. A locator written from
//! the *requested* path would then point at nothing. (odm rejects that case up
//! front — see [`Decision::Collision`] — but the locator is written from
//! observation regardless, because "git did something other than what was
//! asked" is exactly the situation the invariant exists for.)
//!
//! ## What is deliberately not here
//!
//! Renaming a **published** branch across a team. `git branch -m` is local; the
//! remote keeps the old name. Reconciling that is a coordination event
//! (push-new, delete-old, everyone re-points), not something odm should do
//! behind a rename flag — so the rename warns and proceeds locally.

use std::path::{Path, PathBuf};

use crate::error::{Result, StoreError};
use crate::home::{LOCATOR_FILE, StoreLocation};
use crate::worktree;

/// What a rename should do, decided before anything is touched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// The requested names are already the current ones.
    NoOp,
    /// Something already occupies a target name; stop rather than clobber it.
    Collision(Collision),
    /// Go ahead, renaming whichever halves were asked for.
    Proceed {
        /// `(old, new)` worktree directory name, when it changes.
        worktree: Option<(String, String)>,
        /// `(old, new)` branch name, when it changes.
        branch: Option<(String, String)>,
    },
}

/// What was in the way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Collision {
    /// A directory already exists at the target worktree path.
    Worktree(String),
    /// A local branch of the target name already exists.
    Branch(String),
}

impl Collision {
    /// A message naming what collided and where.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Collision::Worktree(name) => {
                format!("a directory already exists at the target worktree {name:?}")
            }
            Collision::Branch(name) => format!("a local branch named {name:?} already exists"),
        }
    }
}

/// Decides what a rename should do — the whole decision, as a pure function.
///
/// The existence checks are passed in rather than performed here, so every
/// branch of the decision is testable without a repository. `None` for a target
/// means "leave this half alone".
#[must_use]
pub fn decide(
    current: &StoreLocation,
    new_worktree: Option<&str>,
    new_branch: Option<&str>,
    worktree_target_exists: bool,
    branch_target_exists: bool,
) -> Decision {
    // A target equal to the current name is not a rename of that half.
    let worktree = new_worktree
        .filter(|n| *n != current.worktree_name)
        .map(|n| (current.worktree_name.clone(), n.to_string()));
    let branch = new_branch
        .filter(|n| *n != current.branch_name)
        .map(|n| (current.branch_name.clone(), n.to_string()));

    if worktree.is_none() && branch.is_none() {
        return Decision::NoOp;
    }
    // Collisions are checked before anything moves: `git worktree move` will
    // not refuse an occupied destination, and `branch -m` clobbering is worse.
    if let Some((_, new)) = &worktree
        && worktree_target_exists
    {
        return Decision::Collision(Collision::Worktree(new.clone()));
    }
    if let Some((_, new)) = &branch
        && branch_target_exists
    {
        return Decision::Collision(Collision::Branch(new.clone()));
    }
    Decision::Proceed { worktree, branch }
}

/// The inputs to a rename.
#[derive(Debug, Clone)]
pub struct RenamePlan {
    /// The repository root, where the locator lives.
    pub repo_root: PathBuf,
    /// The store's current location, from the locator.
    pub current: StoreLocation,
    /// The requested worktree directory name, if it changes.
    pub new_worktree: Option<String>,
    /// The requested branch name, if it changes.
    pub new_branch: Option<String>,
    /// Report the plan and change nothing.
    pub dry_run: bool,
}

impl RenamePlan {
    /// The store root as it stands now.
    #[must_use]
    pub fn current_root(&self) -> PathBuf {
        self.current.store_root(&self.repo_root)
    }

    /// The store root the rename is aiming at.
    #[must_use]
    pub fn target_root(&self) -> PathBuf {
        self.target_location().store_root(&self.repo_root)
    }

    /// The location the locator would hold if everything succeeds.
    #[must_use]
    pub fn target_location(&self) -> StoreLocation {
        StoreLocation {
            worktree_base: self.current.worktree_base.clone(),
            worktree_name: self
                .new_worktree
                .clone()
                .unwrap_or_else(|| self.current.worktree_name.clone()),
            branch_name: self
                .new_branch
                .clone()
                .unwrap_or_else(|| self.current.branch_name.clone()),
            remote: self.current.remote.clone(),
        }
    }
}

/// What a rename did, or would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Renamed {
    /// The decision that was acted on.
    pub decision: Decision,
    /// Where the store was.
    pub old_root: PathBuf,
    /// Where the store is now — **observed**, not assumed.
    pub new_root: PathBuf,
    /// The branch before.
    pub old_branch: String,
    /// The branch now — observed.
    pub new_branch: String,
    /// Set when the branch is shared: the local rename does not follow it.
    pub published_warning: bool,
}

/// Runs the rename.
///
/// # Errors
///
/// [`StoreError::Git`] if the store cannot be found, or a git operation fails.
/// A failure before any git op leaves everything untouched.
pub fn rename(plan: &RenamePlan) -> Result<Renamed> {
    let old_root = plan.current_root();
    let old_branch = plan.current.branch_name.clone();

    if !old_root.exists() {
        return Err(StoreError::Git(format!(
            "no store at {} — nothing to rename. Run `odm store init` first",
            old_root.display()
        )));
    }

    let target = plan.target_root();
    let target_location = plan.target_location();
    let decision = decide(
        &plan.current,
        plan.new_worktree.as_deref(),
        plan.new_branch.as_deref(),
        // Only a *different* target path can collide; the current one is ours.
        target != old_root && target.exists(),
        target_location.branch_name != old_branch
            && worktree::branch_exists(&plan.repo_root, &target_location.branch_name)?,
    );

    // A rename that renames a shared branch desynchronises it from the remote,
    // which is a team matter rather than a local one. Reported, then done
    // locally — odm will not rename someone else's remote branch for them.
    let published = match &decision {
        Decision::Proceed { branch: Some(_), .. } => {
            worktree::is_published(&plan.repo_root, &old_branch)?
        }
        _ => false,
    };

    let unchanged = |decision| Renamed {
        decision,
        old_root: old_root.clone(),
        new_root: old_root.clone(),
        old_branch: old_branch.clone(),
        new_branch: old_branch.clone(),
        published_warning: published,
    };

    match &decision {
        Decision::NoOp => return Ok(unchanged(Decision::NoOp)),
        Decision::Collision(c) => return Ok(unchanged(Decision::Collision(c.clone()))),
        Decision::Proceed { .. } => {}
    }

    if plan.dry_run {
        return Ok(Renamed {
            decision,
            old_root,
            new_root: target,
            old_branch,
            new_branch: target_location.branch_name,
            published_warning: published,
        });
    }

    let Decision::Proceed { worktree: wt, branch: br } = &decision else {
        unreachable!("handled above");
    };

    // Move first, then rename the branch: the move is the operation that can
    // put the tree somewhere unexpected, so doing it first means the branch
    // rename runs against a location we have already observed.
    if wt.is_some() {
        worktree::move_worktree(&plan.repo_root, &old_root, &target)?;
    }
    // Where the tree *actually* is now — the basis for everything below.
    let observed_root = worktree::path_of_branch(&plan.repo_root, &old_branch)?
        .unwrap_or_else(|| if wt.is_some() { target.clone() } else { old_root.clone() });

    // From here the git state has already moved, so a failure must **not** skip
    // the locator write. That is precisely the case that would leave the
    // locator naming a directory which no longer exists — and resolution then
    // quietly self-heals an empty store at the stale path, giving a green
    // `check` over an invisible corpus, the worst failure this slice can have.
    // So the branch result is *held*, the locator is written from what was
    // observed either way, and only then is the error reported.
    let branch_result = match br {
        Some((old, new)) => worktree::rename_branch(&observed_root, old, new),
        None => Ok(()),
    };

    // Read both halves back rather than assuming either.
    let observed_branch = worktree::current_branch(&observed_root)?
        .unwrap_or_else(|| target_location.branch_name.clone());
    let final_root =
        worktree::path_of_branch(&plan.repo_root, &observed_branch)?.unwrap_or(observed_root);

    // The locator is written LAST, and from what was observed — so whatever
    // happened, it names a store that exists.
    write_locator(
        &plan.repo_root,
        &observed_location(&plan.current, &final_root, &observed_branch),
    )?;

    // Now the error, if there was one — naming what *did* succeed, so the
    // operator knows the store is intact and what is left to do.
    if let Err(e) = branch_result {
        let moved = if wt.is_some() {
            format!(
                "the worktree moved to {} and the locator now points there, so the store is \
                 intact and resolvable; ",
                final_root.display()
            )
        } else {
            String::new()
        };
        return Err(StoreError::Git(format!(
            "{moved}the branch rename failed: {e}. The branch is still {observed_branch:?} — \
             re-run `odm store rename --branch <name>` with a valid name"
        )));
    }

    Ok(Renamed {
        decision,
        old_root,
        new_root: final_root,
        old_branch,
        new_branch: observed_branch,
        published_warning: published,
    })
}

/// The locator describing an observed store: the branch as git reports it, and
/// the worktree directory name taken from the observed path.
fn observed_location(current: &StoreLocation, root: &Path, branch: &str) -> StoreLocation {
    let worktree_name = root
        .file_name()
        .map_or_else(|| current.worktree_name.clone(), |n| n.to_string_lossy().into_owned());
    StoreLocation {
        worktree_base: current.worktree_base.clone(),
        worktree_name,
        branch_name: branch.to_string(),
        remote: current.remote.clone(),
    }
}

/// Rewrites the `[store]` section of the locator, leaving every other key.
///
/// A rename must not discard the operational settings a pre-migration repo
/// still keeps in `odm.toml`, so the file is edited rather than replaced: the
/// old `[store]` block is dropped and a fresh one appended.
fn write_locator(repo_root: &Path, location: &StoreLocation) -> Result<()> {
    let path = repo_root.join(LOCATOR_FILE);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();

    let mut kept = String::new();
    let mut in_store = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            // Any new section ends the `[store]` block.
            in_store = trimmed == "[store]";
        }
        if !in_store {
            kept.push_str(line);
            kept.push('\n');
        }
    }
    // Drop the comment odm writes immediately above `[store]`, so re-renaming
    // does not stack copies of it.
    let kept = kept
        .lines()
        .filter(|l| !l.starts_with("# Where the odm store lives"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut text = kept.trim_end().to_string();
    if !text.is_empty() {
        text.push_str("\n\n");
    }
    text.push_str(&format!(
        "# Where the odm store lives (ODD-0022 §4.2). Written by `odm store`.\n\
         [store]\n\
         worktree_base = {:?}\n\
         worktree_name = {:?}\n\
         branch_name = {:?}\n",
        location.worktree_base, location.worktree_name, location.branch_name
    ));
    // Preserve `remote` across a rename — dropping it here would silently
    // un-configure `store sync`'s target the next time the store moves.
    if let Some(remote) = &location.remote {
        text.push_str(&format!("remote = {remote:?}\n"));
    }
    std::fs::write(&path, text).map_err(|e| StoreError::io(&path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn location(worktree: &str, branch: &str) -> StoreLocation {
        StoreLocation {
            worktree_base: ".worktrees".to_string(),
            worktree_name: worktree.to_string(),
            branch_name: branch.to_string(),
            remote: None,
        }
    }

    // ----- L-7/L-8: the decision, without a repository ----------------------

    #[test]
    fn test_renaming_to_the_current_names_is_a_no_op() {
        let cur = location("odm", "odm");
        assert_eq!(decide(&cur, Some("odm"), Some("odm"), false, false), Decision::NoOp);
        assert_eq!(decide(&cur, None, None, false, false), Decision::NoOp);
    }

    #[test]
    fn test_each_half_renames_independently() {
        let cur = location("odm", "odm");
        assert_eq!(
            decide(&cur, Some("planning"), None, false, false),
            Decision::Proceed { worktree: Some(("odm".into(), "planning".into())), branch: None }
        );
        assert_eq!(
            decide(&cur, None, Some("plan"), false, false),
            Decision::Proceed { worktree: None, branch: Some(("odm".into(), "plan".into())) }
        );
    }

    #[test]
    fn test_both_halves_rename_together() {
        let cur = location("odm", "odm");
        assert_eq!(
            decide(&cur, Some("plan"), Some("plan"), false, false),
            Decision::Proceed {
                worktree: Some(("odm".into(), "plan".into())),
                branch: Some(("odm".into(), "plan".into()))
            }
        );
    }

    #[test]
    fn test_an_occupied_worktree_target_is_a_collision() {
        let cur = location("odm", "odm");
        let d = decide(&cur, Some("taken"), None, true, false);
        assert_eq!(d, Decision::Collision(Collision::Worktree("taken".into())));
        assert!(matches!(d, Decision::Collision(_)), "and nothing proceeds");
    }

    #[test]
    fn test_an_existing_branch_target_is_a_collision() {
        let cur = location("odm", "odm");
        assert_eq!(
            decide(&cur, None, Some("taken"), false, true),
            Decision::Collision(Collision::Branch("taken".into()))
        );
    }

    #[test]
    fn test_an_unchanged_half_cannot_collide_with_itself() {
        // Passing the current name for one half must not read as a collision
        // just because that name exists — it is ours.
        let cur = location("odm", "odm");
        assert_eq!(
            decide(&cur, Some("plan"), Some("odm"), false, true),
            Decision::Proceed { worktree: Some(("odm".into(), "plan".into())), branch: None }
        );
    }

    #[test]
    fn test_collision_messages_name_what_is_in_the_way() {
        assert!(Collision::Worktree("x".into()).describe().contains("worktree \"x\""));
        assert!(Collision::Branch("y".into()).describe().contains("branch named \"y\""));
    }

    // ----- L-5: the locator mirrors the observed state ----------------------

    #[test]
    fn test_observed_location_takes_the_name_from_the_real_path() {
        let cur = location("odm", "odm");
        let observed =
            observed_location(&cur, Path::new("/repo/.worktrees/actually-here"), "actual-branch");
        assert_eq!(observed.worktree_name, "actually-here");
        assert_eq!(observed.branch_name, "actual-branch");
        assert_eq!(observed.worktree_base, ".worktrees", "the base is not invented");
    }

    #[test]
    fn test_locator_rewrite_replaces_the_store_block_and_keeps_the_rest() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join(LOCATOR_FILE),
            "author_name = \"Ada\"\n\n\
             # Where the odm store lives (ODD-0022 §4.2). Written by `odm store init`.\n\
             [store]\nworktree_base = \".worktrees\"\nworktree_name = \"odm\"\nbranch_name = \"odm\"\n\n\
             [gates.slice]\nsequence = [\"planned\"]\n",
        )
        .unwrap();

        write_locator(dir.path(), &location("planning", "plan")).unwrap();
        let text = std::fs::read_to_string(dir.path().join(LOCATOR_FILE)).unwrap();

        assert!(text.contains("author_name = \"Ada\""), "other keys survive:\n{text}");
        assert!(text.contains("[gates.slice]"), "other sections survive:\n{text}");
        assert!(text.contains("worktree_name = \"planning\""), "the new name is in:\n{text}");
        assert!(!text.contains("\"odm\""), "the old names are gone:\n{text}");
        assert_eq!(text.matches("[store]").count(), 1, "exactly one store block:\n{text}");
    }

    #[test]
    fn test_locator_rewrite_is_stable_across_repeated_renames() {
        let dir = TempDir::new().unwrap();
        write_locator(dir.path(), &location("a", "a")).unwrap();
        write_locator(dir.path(), &location("b", "b")).unwrap();
        write_locator(dir.path(), &location("c", "c")).unwrap();
        let text = std::fs::read_to_string(dir.path().join(LOCATOR_FILE)).unwrap();

        assert_eq!(text.matches("[store]").count(), 1, "no stacking:\n{text}");
        assert_eq!(
            text.matches("# Where the odm store lives").count(),
            1,
            "the comment does not accumulate:\n{text}"
        );
        assert!(text.contains("worktree_name = \"c\""));
    }
}
