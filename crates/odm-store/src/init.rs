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
//! 4. `.gitignore` in the store, keeping the derived index out of its history
//! 5. `/.worktrees/` git-ignored on the code branch
//!
//! Ordering matters: the locator is written only once the worktree exists, so a
//! failed `init` never leaves an `odm.toml` pointing at nothing.

use std::path::{Path, PathBuf};

use crate::error::{Result, StoreError};
use crate::home::{
    DEFAULT_STORE_NAME, DEFAULT_WORKTREE_BASE, LOCATOR_FILE, OPERATIONAL_FILE, StoreLocation,
};
use crate::worktree::{self, GitVersion};

/// The store's own `.gitignore`, scaffolded at bootstrap.
///
/// **The caches are ignored; the context is not.** They sit side by side under
/// `.odm/`, but they are different kinds of thing:
///
/// - the `index` and `drift` snapshots are **derived**. They rebuild from
///   `nodes/`, change on nearly every command, and carry no information the
///   nodes do not — so tracking them would put a conflict generator on a branch
///   whose whole purpose is to be shared. The rule is written as
///   ignore-the-directory plus one exception, so a cache added later is ignored
///   by default rather than committed until someone notices.
/// - `context.json` is a **deliberate statement**: an operator saying which arc
///   the project is working on. That is exactly what a newcomer needs, and the
///   project's own success test is that a fresh session reaches full situational
///   awareness from `odm orient` alone. Ignoring it would mean every fresh clone
///   opens on "(no current arc)" — the store would carry the plan but not the
///   place in it.
///
/// The cost is real and accepted: two people working different arcs will contend
/// over one line. That is a coordination problem with a coordination answer, and
/// it is a better default than silently withholding the focus from everyone.
pub const STORE_GITIGNORE: &str = "\
# `.odm/` is derived state by default — the index and the drift snapshot are
# caches that rebuild from `nodes/`, and anything added later will be too.
# Ignoring the directory and re-admitting one file keeps that true without
# needing an edit each time a new cache appears; the reverse (listing the caches)
# fails open, and commits the next one by accident.
/.odm/*

# ...except the current focus, which is a deliberate statement about where the
# work is, not a cache of what the nodes already say. It travels with the store
# so a fresh session gets it from `odm orient`.
!/.odm/context.json
";

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

/// How local and upstream stand relative to each other.
///
/// Separated from the git calls that measure it so the decision table below is
/// a pure function: five cases, all testable without a repository, a remote, or
/// a network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ancestry {
    /// Local is an ancestor of upstream — upstream has moved on.
    pub local_is_ancestor: bool,
    /// Upstream is an ancestor of local — local has commits upstream lacks.
    pub upstream_is_ancestor: bool,
    /// How many commits local is ahead by.
    pub local_ahead: usize,
}

/// What a sync should do — the whole decision table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAction {
    /// Upstream is ahead: fast-forward to it.
    FastForward,
    /// Identical: nothing to do.
    UpToDate,
    /// Local has unpushed commits: report, change nothing.
    LocalAhead(usize),
    /// Histories have parted: stop, and let a human choose.
    Diverged,
    /// No upstream to compare against.
    NoUpstream,
}

impl SyncAction {
    /// The `--json` `mode` for this outcome.
    #[must_use]
    pub fn mode(&self) -> &'static str {
        match self {
            SyncAction::FastForward => "sync-fast-forwarded",
            SyncAction::UpToDate => "sync-up-to-date",
            SyncAction::LocalAhead(_) => "sync-local-ahead",
            SyncAction::Diverged => "sync-diverged",
            SyncAction::NoUpstream => "sync-no-upstream",
        }
    }
}

/// Chooses the sync action from the ancestry, or `None` when there is no
/// upstream at all.
///
/// **Divergence is a stop, never a merge or a rebase.** The store branch is
/// shared by construction — that is the point of ODD-0022 §6 — and rewriting or
/// merging published history corrupts every clone that already has it. odm
/// reports the situation and leaves the choice to a person; there is no
/// automatic resolution that is safe to make on someone else's behalf.
#[must_use]
pub fn sync_action(ancestry: Option<Ancestry>) -> SyncAction {
    let Some(a) = ancestry else {
        return SyncAction::NoUpstream;
    };
    match (a.local_is_ancestor, a.upstream_is_ancestor) {
        // Identical commit: each is an ancestor of the other.
        (true, true) => SyncAction::UpToDate,
        (true, false) => SyncAction::FastForward,
        (false, true) => SyncAction::LocalAhead(a.local_ahead),
        (false, false) => SyncAction::Diverged,
    }
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
                remote: None,
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

    // 0. Whatever pre-split (odm 0.3.x) settings the repo's own `odm.toml`
    //    already carries — read before anything below touches that file, so
    //    this captures the true pre-migration state, not a partially-written
    //    one.
    let legacy = detect_legacy_settings(&plan.repo_root);

    // 1. The worktree + orphan branch. First, because everything after it is
    //    only meaningful once the home exists.
    if let Some(parent) = store_root.parent() {
        std::fs::create_dir_all(parent).map_err(|e| StoreError::io(parent, e))?;
    }
    let git_version = worktree::create(&plan.repo_root, &store_root, &plan.location.branch_name)?;

    // 1a. Auto-set the sync remote when the repo has exactly one (D-3,
    //     arc-store-lifecycle slice 06) — makes `store sync` work out of the
    //     box for the common case. Multi-remote repos need explicit
    //     `store set-remote`; a caller-supplied remote is never overridden.
    let mut location = plan.location.clone();
    if location.remote.is_none() {
        let remotes = worktree::list_remotes(&plan.repo_root)?;
        if remotes.len() == 1 {
            location.remote = Some(remotes[0].clone());
        }
    }

    // 2. The locator — now that there is something to point at.
    write_locator(&plan.repo_root, &location)?;

    // 3. The store's own config + an empty `nodes/` — the defaults, plus a
    //    `[legacy]` block if step 0 found pre-split settings worth
    //    preserving as a historical record (arc-migration-fidelity s13).
    let config = store_root.join(OPERATIONAL_FILE);
    let config_text = match legacy.render() {
        Some(block) => format!("{DEFAULT_OPERATIONAL_CONFIG}{block}"),
        None => DEFAULT_OPERATIONAL_CONFIG.to_string(),
    };
    std::fs::write(&config, config_text).map_err(|e| StoreError::io(&config, e))?;
    let nodes = store_root.join(crate::layout::NODES_DIR);
    std::fs::create_dir_all(&nodes).map_err(|e| StoreError::io(&nodes, e))?;

    // 4. Keep the derived index out of the store's history — but not the
    //    context, which is a statement about where the work is rather than a
    //    cache of what the nodes already say (see `STORE_GITIGNORE`).
    let ignore = store_root.join(".gitignore");
    std::fs::write(&ignore, STORE_GITIGNORE).map_err(|e| StoreError::io(&ignore, e))?;

    // 5. Keep the worktree off the code branch.
    ensure_gitignored(&plan.repo_root)?;

    Ok(Bootstrapped {
        store_root,
        branch: location.branch_name.clone(),
        location,
        git_version: Some(git_version),
    })
}

/// The pre-split, odm-0.3.x operational keys `bootstrap` looks for in
/// whatever `odm.toml` already sits at `repo_root` — every pre-migration
/// repo has one, since the format predates the locator/config split
/// (ODD-0022 §4.2). Detected, never assumed: a repo with no `odm.toml`, or
/// one already in the new locator-only shape, has none of these.
struct LegacySettings {
    docs_directory: Option<String>,
    dev_directory: Option<String>,
    preserve_dustbin_structure: Option<bool>,
    auto_stage_git: Option<bool>,
}

impl LegacySettings {
    /// Whether anything was actually found — an all-`None` value is the same
    /// as "no legacy config", not an empty `[legacy]` block worth writing.
    fn is_empty(&self) -> bool {
        self.docs_directory.is_none()
            && self.dev_directory.is_none()
            && self.preserve_dustbin_structure.is_none()
            && self.auto_stage_git.is_none()
    }

    /// Renders as a `[legacy]` TOML block, or `None` if empty.
    fn render(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let mut out = String::from(
            "\n# The pre-split (odm 0.3.x) operational settings this repo's own\n\
             # `odm.toml` carried before `odm store init` — preserved here, not acted\n\
             # on, so a future migration pass has the historical record rather than\n\
             # having to re-derive or lose it (arc-migration-fidelity s13).\n\
             [legacy]\n",
        );
        if let Some(v) = &self.docs_directory {
            out.push_str(&format!("docs_directory = {v:?}\n"));
        }
        if let Some(v) = &self.dev_directory {
            out.push_str(&format!("dev_directory = {v:?}\n"));
        }
        if let Some(v) = self.preserve_dustbin_structure {
            out.push_str(&format!("preserve_dustbin_structure = {v}\n"));
        }
        if let Some(v) = self.auto_stage_git {
            out.push_str(&format!("auto_stage_git = {v}\n"));
        }
        Some(out)
    }
}

/// Reads whatever `odm.toml` already sits at `repo_root` (if any) and pulls
/// out the pre-split operational keys, before [`write_locator`] appends the
/// new `[store]` section to that same file.
///
/// A missing file, an unparsable one, or one with none of these keys (e.g.
/// already locator-only) all resolve the same way: nothing to carry forward.
fn detect_legacy_settings(repo_root: &Path) -> LegacySettings {
    let text = std::fs::read_to_string(repo_root.join(LOCATOR_FILE)).unwrap_or_default();
    let value: toml::Value = text.parse().unwrap_or(toml::Value::Table(Default::default()));
    LegacySettings {
        docs_directory: value.get("docs_directory").and_then(|v| v.as_str()).map(str::to_string),
        dev_directory: value.get("dev_directory").and_then(|v| v.as_str()).map(str::to_string),
        preserve_dustbin_structure: value
            .get("preserve_dustbin_structure")
            .and_then(toml::Value::as_bool),
        auto_stage_git: value.get("auto_stage_git").and_then(toml::Value::as_bool),
    }
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
    if let Some(remote) = &location.remote {
        text.push_str(&format!("remote = {remote:?}\n"));
    }
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

/// What an attach did, or would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attached {
    /// The store root the branch was checked out into.
    pub store_root: PathBuf,
    /// The branch attached to.
    pub branch: String,
    /// The remote it was fetched from, when it had to be fetched first.
    pub fetched_from: Option<String>,
}

/// What a sync found, and did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Synced {
    /// The store root.
    pub store_root: PathBuf,
    /// The branch.
    pub branch: String,
    /// The outcome.
    pub action: SyncAction,
    /// The upstream compared against, when there was one.
    pub upstream: Option<String>,
}

/// The default remote to fetch from.
pub const DEFAULT_REMOTE: &str = "origin";

/// Attaches to an existing branch: check it out into the worktree.
///
/// **Never scaffolds and never orphans.** `config.toml` and `nodes/` arrive
/// *with* the branch (ODD-0022 §4.3), so writing them here would overwrite a
/// teammate's store with defaults. The only writes are the two idempotent
/// top-ups a fresh clone might be missing: the `.gitignore` entry and the
/// locator.
///
/// # Errors
///
/// [`StoreError::Git`] if the fetch or the worktree checkout fails.
pub fn attach(plan: &Plan, remote_only: Option<&str>) -> Result<Attached> {
    let store_root = plan.store_root();
    let branch = plan.location.branch_name.clone();

    if plan.dry_run {
        return Ok(Attached { store_root, branch, fetched_from: remote_only.map(str::to_string) });
    }

    // A branch that exists only on a remote has to be fetched before it can be
    // checked out; git then DWIMs a local tracking branch of the same name.
    let fetched_from = match remote_only {
        Some(_) => {
            worktree::fetch(&plan.repo_root, DEFAULT_REMOTE)?;
            Some(DEFAULT_REMOTE.to_string())
        }
        None => None,
    };

    worktree::attach(&plan.repo_root, &store_root, &branch)?;

    // Top-ups only — both are normally already committed on the code branch.
    if !plan.repo_root.join(LOCATOR_FILE).exists() {
        write_locator(&plan.repo_root, &plan.location)?;
    }
    ensure_gitignored(&plan.repo_root)?;

    Ok(Attached { store_root, branch, fetched_from })
}

/// Freshens an existing local store from its upstream, fast-forward only.
///
/// # Errors
///
/// [`StoreError::Git`] if git cannot be run, or the fast-forward fails.
pub fn sync(plan: &Plan) -> Result<Synced> {
    let store_root = plan.store_root();
    let branch = plan.location.branch_name.clone();
    let remote = plan.location.remote.as_deref().unwrap_or(DEFAULT_REMOTE);
    let upstream_ref = format!("{remote}/{branch}");

    // Fetch first, so the comparison is against a current upstream rather than
    // a stale one — otherwise "up to date" could be a lie. A fetch failure (no
    // remote, no network) is not fatal: it simply leaves no upstream to
    // compare, which the table already handles.
    //
    // **This happens under `--dry-run` too**, deliberately. A dry run's whole
    // job is to predict the real run, and without a fetch it compares against
    // whatever the last fetch happened to leave behind — reporting
    // "up to date" for a store that a real run would fast-forward. A preview
    // that can differ from the thing it previews is worse than none. Fetching
    // is the one git operation here that cannot touch the store: it updates
    // remote-tracking refs only, moves no branch, and writes no file in the
    // worktree. Recorded in the slice's closing report rather than done
    // silently, since the prompt's wording was "touch nothing".
    let _ = worktree::fetch(&plan.repo_root, remote);

    let local = worktree::rev_parse(&plan.repo_root, &branch)?;
    let upstream = worktree::rev_parse(&plan.repo_root, &upstream_ref)?;

    let ancestry = match (&local, &upstream) {
        (Some(local), Some(up)) => Some(Ancestry {
            local_is_ancestor: worktree::is_ancestor(&plan.repo_root, local, up)?,
            upstream_is_ancestor: worktree::is_ancestor(&plan.repo_root, up, local)?,
            local_ahead: worktree::count_commits(&plan.repo_root, &format!("{up}..{local}"))?,
        }),
        _ => None,
    };
    let action = sync_action(ancestry);

    // Only the fast-forward changes anything, and only outside a dry run.
    if action == SyncAction::FastForward && !plan.dry_run {
        worktree::merge_ff_only(&store_root, &upstream_ref)?;
    }

    Ok(Synced {
        store_root,
        branch,
        action,
        upstream: upstream.is_some().then(|| upstream_ref.clone()),
    })
}

/// Whether the store branch exists but its worktree is gone — a half-finished
/// or manually-deleted `init`.
///
/// Detected but **not repaired** (ODD-0022 §6): re-creating the worktree here
/// could silently paper over a state a person deliberately made, so `init`
/// reports it and points at the future `--force`.
#[must_use]
pub fn needs_repair(repo_root: &Path, store_root: &Path, branch: &str) -> bool {
    if store_root.exists() {
        return false;
    }
    let Ok(repo) = gix::open(repo_root) else {
        return false;
    };
    repo.try_find_reference(&format!("refs/heads/{branch}")).ok().flatten().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Initializes a real git repo at `dir` with one commit — `bootstrap`
    /// (non-dry-run) shells out to `git worktree add --detach`, which needs a
    /// resolvable `HEAD`, not just a repo.
    fn git_init(dir: &Path) {
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(dir)
                .status()
                .expect("git command");
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "Test"]);
        std::fs::write(dir.join(".gitkeep"), "").unwrap();
        run(&["add", "."]);
        run(&["commit", "-q", "-m", "initial"]);
    }

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
    fn test_detect_legacy_settings_finds_the_pre_split_keys() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join(LOCATOR_FILE),
            "docs_directory = \"./docs\"\n\
             dev_directory = \"./docs/dev\"\n\
             preserve_dustbin_structure = true\n\
             auto_stage_git = false\n",
        )
        .unwrap();
        let legacy = detect_legacy_settings(dir.path());
        assert_eq!(legacy.docs_directory.as_deref(), Some("./docs"));
        assert_eq!(legacy.dev_directory.as_deref(), Some("./docs/dev"));
        assert_eq!(legacy.preserve_dustbin_structure, Some(true));
        assert_eq!(legacy.auto_stage_git, Some(false));
        assert!(!legacy.is_empty());
    }

    #[test]
    fn test_detect_legacy_settings_is_empty_with_no_odm_toml() {
        let dir = TempDir::new().unwrap();
        assert!(detect_legacy_settings(dir.path()).is_empty());
        assert!(detect_legacy_settings(dir.path()).render().is_none());
    }

    #[test]
    fn test_detect_legacy_settings_is_empty_for_a_locator_only_file() {
        // Already in the new, split shape — nothing pre-split to carry forward.
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join(LOCATOR_FILE),
            "[store]\nworktree_base = \"..\"\nworktree_name = \"odm\"\nbranch_name = \"odm\"\n",
        )
        .unwrap();
        assert!(detect_legacy_settings(dir.path()).is_empty());
    }

    #[test]
    fn test_bootstrap_ports_legacy_settings_into_the_stores_legacy_block() {
        let dir = TempDir::new().unwrap();
        git_init(dir.path());
        std::fs::write(
            dir.path().join(LOCATOR_FILE),
            "docs_directory = \"./docs\"\ndev_directory = \"./docs/dev\"\n",
        )
        .unwrap();
        let plan = Plan::new(dir.path(), None, None);
        let out = bootstrap(&plan).unwrap();

        let config = std::fs::read_to_string(out.store_root.join(OPERATIONAL_FILE)).unwrap();
        assert!(config.contains("[legacy]"), "the block is present:\n{config}");
        assert!(config.contains("docs_directory = \"./docs\""), "carries the old value:\n{config}");
        assert!(config.contains("dev_directory = \"./docs/dev\""), "and this one:\n{config}");
        assert!(
            config.contains("docs_directory = \"./docs/design\""),
            "the modern default is still present, unreplaced:\n{config}"
        );
    }

    #[test]
    fn test_bootstrap_writes_no_legacy_block_with_nothing_to_carry() {
        let dir = TempDir::new().unwrap();
        git_init(dir.path());
        let plan = Plan::new(dir.path(), None, None);
        let out = bootstrap(&plan).unwrap();
        let config = std::fs::read_to_string(out.store_root.join(OPERATIONAL_FILE)).unwrap();
        assert!(!config.contains("[legacy]"), "nothing to carry, nothing written:\n{config}");
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

    // ----- L-7: the ancestry decision table, without a repo or a network ----

    /// The ancestry for a given pair of flags.
    fn ancestry(local_is_ancestor: bool, upstream_is_ancestor: bool, ahead: usize) -> Ancestry {
        Ancestry { local_is_ancestor, upstream_is_ancestor, local_ahead: ahead }
    }

    #[test]
    fn test_sync_action_upstream_ahead_fast_forwards() {
        // local is an ancestor of upstream, and not vice versa.
        assert_eq!(sync_action(Some(ancestry(true, false, 0))), SyncAction::FastForward);
    }

    #[test]
    fn test_sync_action_identical_is_up_to_date() {
        // A commit is its own ancestor, so equality shows as both flags set.
        assert_eq!(sync_action(Some(ancestry(true, true, 0))), SyncAction::UpToDate);
    }

    #[test]
    fn test_sync_action_local_ahead_reports_the_count() {
        assert_eq!(sync_action(Some(ancestry(false, true, 3))), SyncAction::LocalAhead(3));
    }

    #[test]
    fn test_sync_action_neither_ancestor_is_diverged() {
        // The case that must never be auto-resolved.
        assert_eq!(sync_action(Some(ancestry(false, false, 2))), SyncAction::Diverged);
    }

    #[test]
    fn test_sync_action_without_upstream_is_nothing_to_sync_from() {
        assert_eq!(sync_action(None), SyncAction::NoUpstream);
    }

    #[test]
    fn test_sync_action_modes_are_the_json_contract() {
        assert_eq!(SyncAction::FastForward.mode(), "sync-fast-forwarded");
        assert_eq!(SyncAction::UpToDate.mode(), "sync-up-to-date");
        assert_eq!(SyncAction::LocalAhead(1).mode(), "sync-local-ahead");
        assert_eq!(SyncAction::Diverged.mode(), "sync-diverged");
        assert_eq!(SyncAction::NoUpstream.mode(), "sync-no-upstream");
    }

    // ----- L-13: repair is detected, not performed --------------------------

    #[test]
    fn test_needs_repair_is_false_when_the_store_is_present() {
        let dir = TempDir::new().unwrap();
        let store = dir.path().join(".worktrees").join("odm");
        std::fs::create_dir_all(&store).unwrap();
        assert!(!needs_repair(dir.path(), &store, "odm"));
    }

    #[test]
    fn test_needs_repair_is_false_without_a_branch() {
        // Store gone *and* no branch: that is a bootstrap, not a repair.
        let dir = TempDir::new().unwrap();
        let store = dir.path().join(".worktrees").join("odm");
        assert!(!needs_repair(dir.path(), &store, "odm"));
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
