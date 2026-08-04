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

use anyhow::Context;
use odm_store::init::{self, Mode, SyncAction};
use odm_store::rename::Decision;
use odm_store::{InitPlan, NodeDelta, RenamePlan, Repo, StoreHome, worktree};
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

/// The `--json` shape for a `store commit` (F-5): a real commit, a dry-run
/// preview, or a clean no-op all use the same shape, distinguished by
/// `committed`.
#[derive(Serialize)]
struct CommitJson {
    /// Whether a commit was actually written.
    committed: bool,
    /// The new commit's id, when one was written.
    sha: Option<String>,
    /// The orphan branch the commit landed on (or would).
    branch: String,
    /// The message used (auto-generated or `-m`-supplied).
    message: String,
    /// The node-file delta the message was derived from.
    delta: NodeDelta,
}

/// The auto-summary message (D-1): the node-delta summary when there is one,
/// else a generic fallback for a dirty worktree whose changes are all
/// non-node files (e.g. `config.toml`).
fn auto_message(delta: &NodeDelta) -> String {
    if delta.is_empty() { "store: config/settings update".to_string() } else { delta.summary() }
}

/// Runs `odm store commit`: persists the store worktree's pending node
/// changes as a commit on the orphan branch (arc-store-lifecycle s01).
///
/// Idempotent (F-3): a clean worktree exits `0` with "nothing to commit" and
/// writes no empty commit. The default message is the node delta (F-2);
/// `-m` overrides it. `--dry-run` (F-4) reports the delta and message and
/// writes nothing — the orphan branch's `HEAD` is untouched.
///
/// # Errors
///
/// Returns an error (exit code `2`) if there is no store worktree to commit
/// to, or a git operation fails.
pub(crate) fn commit(
    root: &Path,
    message: Option<&str>,
    dry_run: bool,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let home = StoreHome::resolve(root);
    let Some(location) = home.location.clone() else {
        anyhow::bail!(
            "no `[store]` in {} — there is no store worktree to commit; the store is the repo \
             root. Run `odm store init` first.",
            home.repo_root.join("odm.toml").display()
        )
    };
    let branch = location.branch_name.clone();
    let store_root = home.store_root.clone();

    let repo = Repo::open(&store_root).with_context(|| {
        format!(
            "opening the store worktree at {} — has `odm store init` run?",
            store_root.display()
        )
    })?;

    let delta = odm_store::delta::compute(&repo, &store_root)?;
    let clean = repo.is_clean()?;

    if clean {
        if json {
            let view = CommitJson {
                committed: false,
                sha: None,
                branch,
                message: "nothing to commit".to_string(),
                delta,
            };
            writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
            return Ok(());
        }
        term::success(err, "store commit: nothing to commit — the store worktree is clean")?;
        return Ok(());
    }

    let message = message.map(str::to_string).unwrap_or_else(|| auto_message(&delta));

    if dry_run {
        if json {
            let view = CommitJson { committed: false, sha: None, branch, message, delta };
            writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
            return Ok(());
        }
        term::info(
            err,
            &format!(
                "store commit (dry-run): would commit on {branch:?} — {message:?} — nothing \
                 written"
            ),
        )?;
        return Ok(());
    }

    let sha = repo.commit_all(&message)?;

    if json {
        let view = CommitJson { committed: true, sha: Some(sha.clone()), branch, message, delta };
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }
    term::success(err, &format!("store commit: {sha} on {branch:?} — {message:?}"))?;
    Ok(())
}

/// The `--json` shape for `store status` (F-5): the pending delta `commit`
/// would write, plus ahead/behind vs. the configured upstream.
#[derive(Serialize)]
struct StatusJson {
    /// Whether the store worktree has no pending node changes.
    clean: bool,
    /// The orphan branch this reports on.
    branch: String,
    /// The store's root directory.
    store_root: String,
    /// The node-file delta `commit` would write.
    delta: NodeDelta,
    /// Ahead/behind vs. the configured upstream, or `None` when there is none.
    upstream: Option<UpstreamJson>,
}

/// The upstream half of [`StatusJson`].
#[derive(Serialize)]
struct UpstreamJson {
    /// The upstream ref compared against, e.g. `origin/odm-store`.
    #[serde(rename = "ref")]
    reference: String,
    /// `up-to-date` | `local-ahead` | `upstream-ahead` | `diverged`.
    action: &'static str,
    /// Commits local has that upstream lacks.
    ahead: usize,
    /// Commits upstream has that local lacks.
    behind: usize,
}

/// The `--json` `upstream.action` for a [`SyncAction`] (never called for
/// `NoUpstream` — that case reports `upstream: null` instead).
fn action_str(action: &SyncAction) -> &'static str {
    match action {
        SyncAction::FastForward => "upstream-ahead",
        SyncAction::UpToDate => "up-to-date",
        SyncAction::LocalAhead(_) => "local-ahead",
        SyncAction::Diverged => "diverged",
        SyncAction::NoUpstream => "no-upstream",
    }
}

/// The plain-text delta half of the status line: `commit`'s own wording
/// (`NodeDelta::summary`, or its `auto_message` fallback for a dirty
/// worktree with no *node* changes — e.g. only `config.toml` touched) minus
/// the `"store: "` prefix, since `status` already opens with
/// `"store status: "`. `"nothing to commit"` when the worktree is fully
/// clean, matching `commit`'s own clean-path wording — driven by `clean`
/// (`Repo::is_clean`), not `delta.is_empty()`, since the two can diverge
/// exactly the way `commit`'s `auto_message` fallback exists to handle.
fn delta_phrase(clean: bool, delta: &NodeDelta) -> String {
    if clean {
        return "nothing to commit".to_string();
    }
    let full = auto_message(delta);
    full.strip_prefix("store: ").unwrap_or(&full).to_string()
}

/// The plain-text upstream half of the status line.
fn upstream_phrase(action: &SyncAction, upstream_ref: &str, ahead: usize, behind: usize) -> String {
    match action {
        SyncAction::UpToDate => format!("up to date with {upstream_ref}"),
        SyncAction::FastForward => {
            format!("{behind} commit(s) behind {upstream_ref} — run `store sync` to fast-forward")
        }
        SyncAction::LocalAhead(_) => {
            format!("{ahead} commit(s) ahead of {upstream_ref} — push when ready")
        }
        SyncAction::Diverged => format!(
            "diverged from {upstream_ref} ({ahead} ahead, {behind} behind) — reconcile manually"
        ),
        SyncAction::NoUpstream => "no upstream configured".to_string(),
    }
}

/// Runs `odm store status`: a read-only view of the store's git state
/// (arc-store-lifecycle s02) — the pending node delta `commit` would write
/// (F-1/F-2), plus ahead/behind vs. the configured upstream (F-3/F-4).
///
/// **Never mutates anything** (F-6): no commit, no fetch, no index or
/// worktree write. The upstream comparison reads whatever the last fetch
/// left behind (D-1) — `store sync` is the verb that fetches; `status` stays
/// a pure, side-effect-free read.
///
/// # Errors
///
/// Returns an error (exit code `2`) if there is no store worktree to check,
/// or a git operation fails.
pub(crate) fn status(
    root: &Path,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let home = StoreHome::resolve(root);
    let Some(location) = home.location.clone() else {
        anyhow::bail!(
            "no `[store]` in {} — there is no store worktree to check; the store is the repo \
             root. Run `odm store init` first.",
            home.repo_root.join("odm.toml").display()
        )
    };
    let branch = location.branch_name.clone();
    let store_root = home.store_root.clone();
    let repo_root = home.repo_root.clone();

    let repo = Repo::open(&store_root).with_context(|| {
        format!(
            "opening the store worktree at {} — has `odm store init` run?",
            store_root.display()
        )
    })?;

    let delta = odm_store::delta::compute(&repo, &store_root)?;
    let clean = repo.is_clean()?;

    // Same remote-tracking plumbing `init`'s sync arm uses, but with no fetch
    // (D-1) — `status` reports against whatever the last fetch left behind.
    // Reads the configured remote (slice 06), same fallback `sync` uses, so
    // the two commands never disagree about which remote "ahead/behind"
    // means.
    let remote = location.remote.as_deref().unwrap_or(init::DEFAULT_REMOTE);
    let upstream_ref = format!("{remote}/{branch}");
    let local = worktree::rev_parse(&repo_root, &branch)?;
    let upstream = worktree::rev_parse(&repo_root, &upstream_ref)?;

    let (action, ahead, behind) = match (&local, &upstream) {
        (Some(l), Some(u)) => {
            let ahead = worktree::count_commits(&repo_root, &format!("{u}..{l}"))?;
            let behind = worktree::count_commits(&repo_root, &format!("{l}..{u}"))?;
            let ancestry = init::Ancestry {
                local_is_ancestor: worktree::is_ancestor(&repo_root, l, u)?,
                upstream_is_ancestor: worktree::is_ancestor(&repo_root, u, l)?,
                local_ahead: ahead,
            };
            (init::sync_action(Some(ancestry)), ahead, behind)
        }
        _ => (init::sync_action(None), 0, 0),
    };

    if json {
        let view = StatusJson {
            clean,
            branch,
            store_root: store_root.display().to_string(),
            delta,
            upstream: (action != SyncAction::NoUpstream).then(|| UpstreamJson {
                reference: upstream_ref.clone(),
                action: action_str(&action),
                ahead,
                behind,
            }),
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    let msg = format!(
        "store status: {} — {}",
        delta_phrase(clean, &delta),
        upstream_phrase(&action, &upstream_ref, ahead, behind)
    );
    match action {
        SyncAction::Diverged | SyncAction::FastForward => term::warning(err, &msg)?,
        _ => term::success(err, &msg)?,
    }
    Ok(())
}

/// The `--json` shape for `store sync` (F-6): the action taken — or, under
/// `--dry-run`, the action the ancestry calls for — plus the upstream state.
#[derive(Serialize)]
struct SyncJson {
    /// `pushed` | `pulled` | `up-to-date` | `diverged` | `no-upstream`.
    action: &'static str,
    /// Whether this was a preview rather than a real push/pull.
    dry_run: bool,
    /// The orphan branch this acted on.
    branch: String,
    /// The store's root directory.
    store_root: String,
    /// The upstream compared/synced against, or `None` when there is none.
    upstream: Option<SyncUpstreamJson>,
}

/// The upstream half of [`SyncJson`].
#[derive(Serialize)]
struct SyncUpstreamJson {
    /// The upstream ref, e.g. `origin/odm-store`.
    #[serde(rename = "ref")]
    reference: String,
    /// Commits local has that upstream lacks, after this run — or, under
    /// `--dry-run`, unchanged from before it (nothing was pushed).
    ahead: usize,
    /// Commits upstream has that local lacks, after this run — or, under
    /// `--dry-run`, unchanged from before it (nothing was pulled).
    behind: usize,
}

/// `store sync`'s action for a [`SyncAction`] classification: what happened,
/// or — under `--dry-run` — what the classification calls for. Distinct
/// vocabulary from `status`'s [`action_str`]: `status` describes a state
/// (`upstream-ahead`), `sync` describes an outcome (`pulled`).
fn sync_action_str(action: &SyncAction) -> &'static str {
    match action {
        SyncAction::FastForward => "pulled",
        SyncAction::LocalAhead(_) => "pushed",
        SyncAction::UpToDate => "up-to-date",
        SyncAction::Diverged => "diverged",
        SyncAction::NoUpstream => "no-upstream",
    }
}

/// The `--json` shape for `store set-remote` (F-6).
#[derive(Serialize)]
struct SetRemoteJson {
    /// The remote name now configured.
    remote: String,
    /// Its URL.
    url: String,
    /// Whether it was auto-detected (no argument given).
    auto_detected: bool,
    /// The store branch.
    branch: String,
    /// The store root path.
    store_root: String,
}

/// Sets `remote = "<name>"` in the code branch's `odm.toml` `[store]`
/// section, preserving every other key and every comment (D-5).
///
/// Edits with `toml_edit::DocumentMut`, the same approach
/// `write_additional_paths` uses for the operational config: `odm.toml` is a
/// short, hand-editable file, and a `toml::Value` round-trip through
/// `to_string_pretty` would silently discard its one comment (and any a user
/// adds later) — `toml_edit` touches only the key this function owns.
/// `[store]` is assumed already present (`set_remote` bails earlier when it
/// isn't), so no auto-vivify guard is needed here.
///
/// # Errors
///
/// Returns an error if `odm.toml` cannot be read, parsed, or written.
fn write_remote_to_locator(repo_root: &Path, remote: &str) -> anyhow::Result<()> {
    let path = repo_root.join("odm.toml");
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("parsing {} as TOML", path.display()))?;
    doc["store"]["remote"] = toml_edit::value(remote);
    std::fs::write(&path, doc.to_string()).with_context(|| format!("writing {}", path.display()))
}

/// Runs `odm store set-remote`: configures which git remote `store sync`
/// targets (arc-store-lifecycle s06).
///
/// **Explicit** (`set-remote <name>`): validates the remote exists
/// (`git remote get-url`) before writing it — a typo should fail here, not
/// silently at the next `sync`. **Auto-detect** (`set-remote`, no argument,
/// D-1): exactly one configured remote is used without a prompt; zero or
/// multiple remotes is a clear error naming what's available. Re-running
/// with a different name overwrites — this is a config pointer, not a
/// destructive action.
///
/// # Errors
///
/// Returns an error (exit code `2`) if there is no store worktree, the named
/// remote doesn't exist, auto-detection finds zero or multiple remotes, or
/// the locator cannot be updated.
pub(crate) fn set_remote(
    root: &Path,
    remote: Option<&str>,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let home = StoreHome::resolve(root);
    let Some(location) = home.location.clone() else {
        anyhow::bail!(
            "no `[store]` in {} — there is no store to configure a remote for. Run `odm store \
             init` first.",
            home.repo_root.join("odm.toml").display()
        )
    };
    let repo_root = home.repo_root.clone();

    let (chosen, auto_detected) = match remote {
        Some(name) => {
            if worktree::remote_url(&repo_root, name)?.is_none() {
                let available = worktree::list_remotes(&repo_root)?;
                anyhow::bail!(
                    "remote {name:?} does not exist — available remotes: {}",
                    if available.is_empty() {
                        "(none configured)".to_string()
                    } else {
                        available.join(", ")
                    }
                );
            }
            (name.to_string(), false)
        }
        None => {
            let remotes = worktree::list_remotes(&repo_root)?;
            match remotes.len() {
                0 => anyhow::bail!("no remotes configured for this repository"),
                1 => (remotes[0].clone(), true),
                _ => anyhow::bail!(
                    "multiple remotes configured — specify one: {}",
                    remotes.join(", ")
                ),
            }
        }
    };
    let url = worktree::remote_url(&repo_root, &chosen)?
        .ok_or_else(|| anyhow::anyhow!("remote {chosen:?} vanished between validation and use"))?;

    write_remote_to_locator(&repo_root, &chosen)?;

    if json {
        let view = SetRemoteJson {
            remote: chosen,
            url,
            auto_detected,
            branch: location.branch_name.clone(),
            store_root: home.store_root.display().to_string(),
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    let suffix = if auto_detected { " (auto-detected)" } else { "" };
    term::success(err, &format!("store set-remote: using {chosen:?} ({url}){suffix}"))?;
    Ok(())
}

/// Runs `odm store sync`: pushes or pulls the orphan-branch store to/from its
/// remote (arc-store-lifecycle s03) — the last leg of the lifecycle, closing
/// the gap that otherwise forces raw git.
///
/// A single bidirectional verb: the action follows from the ancestry, fetched
/// first — **always, even under `--dry-run`** (D-2) — so the preview matches
/// what a real run would do; a preview against a stale remote-tracking ref
/// could show "up to date" when a real run would pull, which is worse than no
/// preview. Fetching only updates remote-tracking refs; it never moves the
/// local branch or writes into the worktree.
///
/// Upstream ahead **fast-forwards** — never a merge commit. Local ahead
/// **pushes** — never `--force`. **Diverged stops and changes nothing**
/// (ODD-0022 §6): odm never resolves a shared branch's diverged history on
/// someone's behalf.
///
/// **D-3:** a dirty worktree does not block a push (it only sends committed
/// work), but does block a pull — `git merge --ff-only` against uncommitted
/// changes is exactly the kind of surprising git error odm exists to avoid,
/// so a fast-forward first checks [`Repo::is_clean`] and refuses with
/// guidance instead of letting git's own error surface.
///
/// # Errors
///
/// Returns an error (exit code `2`) if there is no store worktree to sync,
/// the worktree is dirty ahead of a fast-forward, or a git operation fails.
pub(crate) fn sync_cmd(
    root: &Path,
    dry_run: bool,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let home = StoreHome::resolve(root);
    let Some(location) = home.location.clone() else {
        anyhow::bail!(
            "no `[store]` in {} — there is no store worktree to sync; the store is the repo \
             root. Run `odm store init` first.",
            home.repo_root.join("odm.toml").display()
        )
    };
    let branch = location.branch_name.clone();
    let store_root = home.store_root.clone();
    let repo_root = home.repo_root.clone();
    // Slice 06: read the configured sync remote, falling back to
    // DEFAULT_REMOTE for stores created before `set-remote` existed.
    let remote = location.remote.clone().unwrap_or_else(|| init::DEFAULT_REMOTE.to_string());

    // D-2: fetch always, even under --dry-run. A failure (no remote, no
    // network) is not fatal — it leaves no upstream to compare, which the
    // NoUpstream arm below already handles. Tracked (not just discarded)
    // because a *successful* fetch is what distinguishes "remote doesn't
    // carry this branch yet" (first-push, D-4) from "no remote at all".
    let fetch_ok = worktree::fetch(&repo_root, &remote).is_ok();

    let upstream_ref = format!("{remote}/{branch}");
    let local = worktree::rev_parse(&repo_root, &branch)?;
    let upstream = worktree::rev_parse(&repo_root, &upstream_ref)?;

    let (action, mut ahead, mut behind) = match (&local, &upstream) {
        (Some(l), Some(u)) => {
            let ahead = worktree::count_commits(&repo_root, &format!("{u}..{l}"))?;
            let behind = worktree::count_commits(&repo_root, &format!("{l}..{u}"))?;
            let ancestry = init::Ancestry {
                local_is_ancestor: worktree::is_ancestor(&repo_root, l, u)?,
                upstream_is_ancestor: worktree::is_ancestor(&repo_root, u, l)?,
                local_ahead: ahead,
            };
            (init::sync_action(Some(ancestry)), ahead, behind)
        }
        // D-4: the remote is reachable (fetch succeeded) but doesn't carry
        // this branch yet — a first-push, not "no upstream". Reuse
        // `LocalAhead` so the existing push arm below handles it unchanged;
        // the count is every commit on the branch (there is nothing on the
        // remote yet to diff against).
        (Some(_local), None) if fetch_ok => {
            let count = worktree::count_commits(&repo_root, &branch)?.max(1);
            (SyncAction::LocalAhead(count), count, 0)
        }
        _ => (init::sync_action(None), 0, 0),
    };

    if !dry_run {
        match &action {
            SyncAction::FastForward => {
                let repo = Repo::open(&store_root).with_context(|| {
                    format!(
                        "opening the store worktree at {} — has `odm store init` run?",
                        store_root.display()
                    )
                })?;
                if !repo.is_clean()? {
                    anyhow::bail!(
                        "store sync: {branch:?} has uncommitted changes — a fast-forward pull \
                         would conflict with them. Commit first (`odm store commit`), then \
                         retry."
                    );
                }
                worktree::merge_ff_only(&store_root, &upstream_ref)?;
                behind = 0;
            }
            SyncAction::LocalAhead(_) => {
                worktree::push(&repo_root, &remote, &branch)?;
                ahead = 0;
            }
            SyncAction::UpToDate | SyncAction::Diverged | SyncAction::NoUpstream => {}
        }
    }

    if json {
        let view = SyncJson {
            action: sync_action_str(&action),
            dry_run,
            branch,
            store_root: store_root.display().to_string(),
            upstream: (action != SyncAction::NoUpstream).then(|| SyncUpstreamJson {
                reference: upstream_ref.clone(),
                ahead,
                behind,
            }),
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    let prefix = if dry_run { "store sync (dry-run)" } else { "store sync" };
    match &action {
        SyncAction::FastForward if dry_run => term::info(
            err,
            &format!(
                "{prefix}: would fast-forward {branch:?} to {upstream_ref} — nothing written \
                 (the upstream was fetched so this preview matches the real run; \
                 remote-tracking refs are all that moved)"
            ),
        )?,
        SyncAction::FastForward => term::success(
            err,
            &format!("{prefix}: fast-forwarded {branch:?} to {upstream_ref} — the store is fresh"),
        )?,
        SyncAction::LocalAhead(n) if dry_run => term::info(
            err,
            &format!("{prefix}: would push {n} commit(s) to {upstream_ref} — nothing written"),
        )?,
        SyncAction::LocalAhead(n) => {
            term::success(err, &format!("{prefix}: pushed {n} commit(s) to {upstream_ref}"))?
        }
        SyncAction::UpToDate => term::success(
            err,
            &format!("{prefix}: {branch:?} is already up to date with {upstream_ref}"),
        )?,
        SyncAction::Diverged => term::warning(
            err,
            &format!(
                "{prefix}: {branch:?} and {upstream_ref} have diverged — each has commits the \
                 other lacks ({ahead} ahead, {behind} behind). Nothing was changed. odm will not \
                 merge or rebase a shared branch for you: that rewrites history other clones \
                 already have. Reconcile them in {}, then re-run.",
                store_root.display()
            ),
        )?,
        SyncAction::NoUpstream => term::warning(
            err,
            &format!(
                "{prefix}: nothing to sync — {branch:?} has no upstream (no remote, or it does \
                 not carry the branch). The local store is untouched and fine."
            ),
        )?,
    }
    Ok(())
}
