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

use odm_store::init::{self, Mode};
use odm_store::{InitPlan, StoreHome};
use serde::Serialize;

use crate::term;

/// The `--json` shape for a completed (or planned) `init`.
#[derive(Serialize)]
struct InitJson {
    /// Which arm ran — `"bootstrap"` today; `"attach"`/`"sync"` in slice 03.
    mode: &'static str,
    /// Whether this was a plan rather than a change.
    dry_run: bool,
    /// The resolved store root.
    store_root: String,
    /// The orphan branch holding the store.
    branch: String,
    /// The worktree directory (the same path as `store_root`, named as the
    /// worktree it is).
    worktree: String,
    /// The git that created it, when one ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    git_version: Option<String>,
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

    match init::detect(&repo_root, &plan.store_root(), &plan.location.branch_name) {
        Mode::Bootstrap => {}
        other => {
            let (what, arm) = match &other {
                Mode::ExistsLocally => ("a local branch already holds a store".to_string(), "sync"),
                Mode::ExistsOnRemote(name) => (format!("{name} already holds a store"), "attach"),
                Mode::Bootstrap => unreachable!("handled above"),
            };
            term::warning(
                err,
                &format!(
                    "{}: `{}` {what}. Nothing was changed — {arm} arrives in slice 03; \
                     re-creating the branch here would discard what is on it.",
                    "store init", plan.location.branch_name
                ),
            )?;
            return Ok(());
        }
    }

    let done = init::bootstrap(&plan)?;

    if json {
        let view = InitJson {
            mode: "bootstrap",
            dry_run,
            store_root: done.store_root.display().to_string(),
            branch: done.branch.clone(),
            worktree: done.store_root.display().to_string(),
            git_version: done.git_version.map(|v| v.to_string()),
        };
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
