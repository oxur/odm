//! `odm store …` — the store's own lifecycle (ODD-0023's third tier).
//!
//! Born here rather than as a top-level `init` that would later be renamed:
//! ODD-0023 puts store-lifecycle commands under `odm store`, and the operator's
//! sequencing is that a command should arrive in its final home rather than
//! move once users have learned it. `init` is the group's first member; the
//! rest of the reorg — the `node` group, the top-level renames — is RH C-4 and
//! is deliberately absent here.

use std::io::Write;
use std::path::Path;

use odm_store::init::{self, Mode, SyncAction};
use odm_store::rename::Decision;
use odm_store::{InitPlan, RenamePlan, StoreHome};
use serde::Serialize;

use crate::term;

/// The `--json` shape for a completed (or planned) `init`, whichever arm ran.
#[derive(Serialize)]
struct InitJson {
    /// `bootstrap` | `attach` | `sync-*` — the arm and, for a sync, its outcome.
    mode: String,
    /// Whether this was a plan rather than a change.
    dry_run: bool,
    /// The resolved store root.
    store_root: String,
    /// The branch holding the store.
    branch: String,
    /// The worktree directory (the same path as `store_root`, named as the
    /// worktree it is).
    worktree: String,
    /// The git that created it, when one ran (bootstrap).
    #[serde(skip_serializing_if = "Option::is_none")]
    git_version: Option<String>,
    /// The remote fetched from, when one was (attach of a remote-only branch).
    #[serde(skip_serializing_if = "Option::is_none")]
    fetched_from: Option<String>,
    /// The upstream compared against, when there was one (sync).
    #[serde(skip_serializing_if = "Option::is_none")]
    upstream: Option<String>,
    /// How many local commits are unpushed (`sync-local-ahead`).
    #[serde(skip_serializing_if = "Option::is_none")]
    local_ahead: Option<usize>,
}

impl InitJson {
    /// The fields every arm reports; each arm fills in its own besides.
    fn new(mode: impl Into<String>, dry_run: bool, store_root: &Path, branch: &str) -> Self {
        let path = store_root.display().to_string();
        Self {
            mode: mode.into(),
            dry_run,
            store_root: path.clone(),
            branch: branch.to_string(),
            worktree: path,
            git_version: None,
            fetched_from: None,
            upstream: None,
            local_ahead: None,
        }
    }
}

/// Runs `odm store init`: stands up the store home, or reports what it would.
///
/// Only the **bootstrap** arm is implemented. A repo that already has the
/// branch — locally or on a remote — stops untouched with a message naming the
/// arm that will handle it, because re-orphaning a branch someone's planning
/// data already lives on would destroy it.
///
/// # Errors
///
/// Returns an error (exit code `2`) if git is unavailable, the worktree cannot
/// be created, or the scaffold cannot be written.
pub(crate) fn init(
    root: &Path,
    worktree: Option<&str>,
    branch: Option<&str>,
    dry_run: bool,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    // The repo root, not the cwd: the locator and `.gitignore` belong at the
    // top of the repository, wherever `init` was invoked from.
    let home = StoreHome::resolve(root);
    let repo_root = home.repo_root.clone();

    let mut plan = InitPlan::new(&repo_root, worktree, branch);
    plan.dry_run = dry_run;

    // A branch with no worktree is a half-finished or hand-removed init. It is
    // *detected*, not repaired: re-creating the worktree could paper over a
    // state someone made deliberately (ODD-0022 §6 defers `--force`).
    if init::needs_repair(&repo_root, &plan.store_root(), &plan.location.branch_name) {
        term::warning(
            err,
            &format!(
                "store init: branch {:?} exists but {} does not — a half-finished or removed \
                 worktree. Nothing was changed; repairing it automatically could discard a state \
                 you made on purpose. Re-create it with `git worktree add {} {}`, or wait for \
                 `--force`.",
                plan.location.branch_name,
                plan.store_root().display(),
                plan.store_root().display(),
                plan.location.branch_name,
            ),
        )?;
        return Ok(());
    }

    match init::detect(&repo_root, &plan.store_root(), &plan.location.branch_name) {
        Mode::Bootstrap => bootstrap(&plan, &repo_root, dry_run, json, out, err),
        Mode::ExistsOnRemote(remote) => attach(&plan, Some(&remote), dry_run, json, out, err),
        Mode::ExistsLocally => sync(&plan, dry_run, json, out, err),
    }
}

/// The **bootstrap** arm: stand the home up from nothing (slice 02).
fn bootstrap(
    plan: &InitPlan,
    repo_root: &Path,
    dry_run: bool,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let done = init::bootstrap(plan)?;

    if json {
        let mut view = InitJson::new("bootstrap", dry_run, &done.store_root, &done.branch);
        view.git_version = done.git_version.map(|v| v.to_string());
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    let store_root = done.store_root.display();
    if dry_run {
        term::info(
            err,
            &format!(
                "store init (dry-run): would create the worktree {store_root} on a new orphan \
                 branch {:?}, write [store] to {}, scaffold {}/ + nodes/, and gitignore \
                 /.worktrees/ — nothing written",
                done.branch,
                repo_root.join("odm.toml").display(),
                odm_store::home::OPERATIONAL_FILE,
            ),
        )?;
        return Ok(());
    }

    term::success(
        err,
        &format!("store init: {store_root} on orphan branch {:?} — the store is live", done.branch),
    )?;
    Ok(())
}

/// The **attach** arm: check out an *existing* branch into the worktree.
///
/// The store arrives with the branch, so nothing is scaffolded and nothing is
/// re-orphaned — that would overwrite a teammate's data with defaults.
fn attach(
    plan: &InitPlan,
    remote: Option<&str>,
    dry_run: bool,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let done = init::attach(plan, remote)?;

    if json {
        let mut view = InitJson::new("attach", dry_run, &done.store_root, &done.branch);
        view.fetched_from = done.fetched_from.clone();
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    let store_root = done.store_root.display();
    if dry_run {
        let fetching =
            done.fetched_from.map_or_else(String::new, |r| format!(" (fetching {r} first)"));
        term::info(
            err,
            &format!(
                "store init (dry-run): would attach {store_root} to the existing branch \
                 {:?}{fetching} — checking it out, not re-creating it; the store's config and \
                 nodes arrive with the branch. Nothing written",
                done.branch,
            ),
        )?;
        return Ok(());
    }

    term::success(
        err,
        &format!(
            "store init: attached {store_root} to the existing branch {:?} — the store came \
             with it",
            done.branch
        ),
    )?;
    Ok(())
}

/// The **sync** arm: fast-forward an existing local store, or explain why not.
fn sync(
    plan: &InitPlan,
    dry_run: bool,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let done = init::sync(plan)?;

    if json {
        let mut view = InitJson::new(done.action.mode(), dry_run, &done.store_root, &done.branch);
        view.upstream = done.upstream.clone();
        if let SyncAction::LocalAhead(n) = done.action {
            view.local_ahead = Some(n);
        }
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    let prefix = if dry_run { "store init (dry-run)" } else { "store init" };
    let upstream = done.upstream.clone().unwrap_or_else(|| "its upstream".to_string());
    let branch = &done.branch;
    match done.action {
        SyncAction::FastForward if dry_run => term::info(
            err,
            &format!(
                "{prefix}: would fast-forward {branch:?} to {upstream} — nothing written (the \
                 upstream was fetched so this preview matches the real run; remote-tracking \
                 refs are all that moved)"
            ),
        )?,
        SyncAction::FastForward => term::success(
            err,
            &format!("{prefix}: fast-forwarded {branch:?} to {upstream} — the store is fresh"),
        )?,
        SyncAction::UpToDate => term::success(
            err,
            &format!("{prefix}: {branch:?} is already up to date with {upstream}"),
        )?,
        SyncAction::LocalAhead(n) => term::info(
            err,
            &format!(
                "{prefix}: {branch:?} is {n} commit(s) ahead of {upstream} — nothing to pull; \
                 push when you are ready"
            ),
        )?,
        // The one outcome that must never be resolved automatically.
        SyncAction::Diverged => term::warning(
            err,
            &format!(
                "{prefix}: {branch:?} and {upstream} have diverged — each has commits the other \
                 lacks. Nothing was changed. odm will not merge or rebase a shared branch for \
                 you: that rewrites history other clones already have. Reconcile them in {}, \
                 then re-run.",
                done.store_root.display()
            ),
        )?,
        SyncAction::NoUpstream => term::warning(
            err,
            &format!(
                "{prefix}: nothing to sync from — {branch:?} has no upstream (no remote, or it \
                 does not carry the branch). The local store is untouched and fine."
            ),
        )?,
    }
    Ok(())
}

/// The `--json` shape for a rename.
#[derive(Serialize)]
struct RenameJson {
    /// What happened: `renamed` | `no-op` | `collision`.
    outcome: &'static str,
    /// Whether this was a plan rather than a change.
    dry_run: bool,
    /// The worktree directory before.
    old_worktree: String,
    /// The worktree directory after — observed, not assumed.
    new_worktree: String,
    /// The branch before.
    old_branch: String,
    /// The branch after.
    new_branch: String,
    /// The resolved store root after.
    store_root: String,
    /// Set when the renamed branch is shared, and the remote keeps the old name.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    published_warning: bool,
    /// What blocked the rename, when something did.
    #[serde(skip_serializing_if = "Option::is_none")]
    collision: Option<String>,
}

/// Runs `odm store rename`.
///
/// # Errors
///
/// Returns an error (exit code `2`) if there is no store to rename, or a git
/// operation fails.
pub(crate) fn rename(
    root: &Path,
    new_worktree: Option<&str>,
    new_branch: Option<&str>,
    dry_run: bool,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let home = StoreHome::resolve(root);
    let Some(current) = home.location.clone() else {
        anyhow::bail!(
            "no `[store]` in {} — this repo's store is the repo root, so there is no worktree to \
             rename. Run `odm store init` to give it a home first.",
            home.repo_root.join("odm.toml").display()
        )
    };

    let plan = RenamePlan {
        repo_root: home.repo_root.clone(),
        current,
        new_worktree: new_worktree.map(str::to_string),
        new_branch: new_branch.map(str::to_string),
        dry_run,
    };
    let done = odm_store::rename::rename(&plan)?;

    if json {
        let outcome = match &done.decision {
            Decision::NoOp => "no-op",
            Decision::Collision(_) => "collision",
            Decision::Proceed { .. } => "renamed",
        };
        let view = RenameJson {
            outcome,
            dry_run,
            old_worktree: done.old_root.display().to_string(),
            new_worktree: done.new_root.display().to_string(),
            old_branch: done.old_branch.clone(),
            new_branch: done.new_branch.clone(),
            store_root: done.new_root.display().to_string(),
            published_warning: done.published_warning,
            collision: match &done.decision {
                Decision::Collision(c) => Some(c.describe()),
                _ => None,
            },
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    match &done.decision {
        Decision::NoOp => {
            term::success(err, "store rename: already named that — nothing to do")?;
            return Ok(());
        }
        Decision::Collision(c) => {
            term::warning(
                err,
                &format!(
                    "store rename: {} — nothing was changed. Choose another name, or move what \
                     is in the way first.",
                    c.describe()
                ),
            )?;
            return Ok(());
        }
        Decision::Proceed { .. } => {}
    }

    // Said before the outcome line: a local branch rename silently desyncs a
    // shared branch, and that is worth knowing whether or not the rename works.
    if done.published_warning {
        term::warning(
            err,
            &format!(
                "store rename: {:?} is published — this renames it **locally only**, and the \
                 remote keeps the old name. Renaming a shared store branch is a team \
                 coordination event (push the new name, delete the old, everyone re-points); \
                 odm will not do that for you.",
                done.old_branch
            ),
        )?;
    }

    let what = describe(&done);
    if dry_run {
        term::info(err, &format!("store rename (dry-run): would rename {what} — nothing written"))?;
        return Ok(());
    }
    term::success(err, &format!("store rename: {what}"))?;
    Ok(())
}

/// A one-line description of what a rename moved.
fn describe(done: &odm_store::Renamed) -> String {
    let mut parts = Vec::new();
    if done.old_root != done.new_root {
        parts.push(format!("{} → {}", done.old_root.display(), done.new_root.display()));
    }
    if done.old_branch != done.new_branch {
        parts.push(format!("branch {:?} → {:?}", done.old_branch, done.new_branch));
    }
    parts.join(", ")
}
