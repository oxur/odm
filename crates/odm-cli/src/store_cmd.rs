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
use odm_store::{InitPlan, StoreHome};
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
