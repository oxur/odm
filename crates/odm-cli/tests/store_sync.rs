//! `odm store sync` — push/pull the orphan-branch store to/from its remote
//! (arc-store-lifecycle, slice 03). Test ids (`F-N`) match
//! `docs/design-v1.0.0/arc-store-lifecycle/slice03-store-sync/ledger.md`.
//!
//! Fixture, class-(a) per the slice-doc: a **real** bare-repo remote (`git
//! init --bare`, real push/fetch, no mock) — the same discipline
//! `store_status.rs`'s ahead/behind fixtures use, extended with a second
//! clone so the remote can be advanced independently of the store under test.

use std::path::Path;
use std::process::Command;

use clap::Parser as _;
use odm_cli::Cli;
use odm_store::init::{self, Plan};
use tempfile::TempDir;

struct Run {
    code: Option<u8>,
    out: String,
    err: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = odm_cli::dispatch(cli, root, &mut out, &mut err).ok();
    Run { code, out: String::from_utf8(out).unwrap(), err: String::from_utf8(err).unwrap() }
}

fn git_init(dir: &Path) {
    let git = |args: &[&str]| {
        Command::new("git").args(args).current_dir(dir).status().expect("git command");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    std::fs::write(dir.join(".gitkeep"), "").unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "initial"]);
}

fn bootstrapped_store() -> TempDir {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    let plan = Plan::new(dir.path(), None, None);
    init::bootstrap(&plan).expect("bootstrap");
    dir
}

/// A repo with a real store worktree, one commit on the store branch, and a
/// bare-repo `origin` remote already carrying that commit (pushed + fetched)
/// — the common starting point for the push/pull/divergence/no-op fixtures.
fn bootstrapped_store_with_upstream() -> (TempDir, TempDir) {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "project", "Root project"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let bare = TempDir::new().unwrap();
    let git = |args: &[&str]| {
        Command::new("git").args(args).current_dir(dir.path()).status().expect("git command");
    };
    git(&["init", "-q", "--bare", &bare.path().display().to_string()]);
    git(&["remote", "add", "origin", &bare.path().display().to_string()]);
    git(&["push", "-q", "origin", "odm:odm"]);
    git(&["fetch", "-q", "origin"]);
    (dir, bare)
}

/// Advances `bare`'s `branch` by one commit, independently of any other
/// clone — via a throwaway second clone, so the fixture simulates "someone
/// else pushed" rather than mutating the bare repo's refs directly.
fn advance_bare_repo(bare: &Path, branch: &str) {
    let clone = TempDir::new().unwrap();
    Command::new("git")
        .args(["clone", "-q", &bare.display().to_string(), "."])
        .current_dir(clone.path())
        .status()
        .expect("git clone");
    let git = |args: &[&str]| {
        Command::new("git").args(args).current_dir(clone.path()).status().expect("git command");
    };
    git(&["checkout", "-q", branch]);
    std::fs::write(clone.path().join("advanced.txt"), "advanced").unwrap();
    git(&["add", "."]);
    git(&["-c", "user.email=t@t.com", "-c", "user.name=T", "commit", "-q", "-m", "advance"]);
    git(&["push", "-q", "origin", branch]);
}

/// `HEAD`'s commit id for `branch` in `dir` — works against a bare repo too.
fn branch_oid(dir: &Path, branch: &str) -> String {
    let out = Command::new("git")
        .args(["rev-parse", branch])
        .current_dir(dir)
        .output()
        .expect("git rev-parse");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

// ----- F-1: pull (ff-only) ----------------------------------------------------

#[test]
fn upstream_ahead_pulls_and_fast_forwards() {
    let (dir, bare) = bootstrapped_store_with_upstream();
    advance_bare_repo(bare.path(), "odm");

    let r = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["action"], "pulled", "{v}");
    assert_eq!(v["dry_run"], false, "{v}");

    let store = dir.path().join(".worktrees").join("odm");
    assert_eq!(
        branch_oid(&store, "HEAD"),
        branch_oid(bare.path(), "odm"),
        "local now matches what the remote had"
    );
}

#[test]
fn pull_reports_in_plain_text_too() {
    let (dir, bare) = bootstrapped_store_with_upstream();
    advance_bare_repo(bare.path(), "odm");

    let r = run(dir.path(), &["store", "sync"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(r.err.contains("fast-forwarded"), "{}", r.err);
}

// ----- F-2: push ----------------------------------------------------------------

#[test]
fn local_ahead_pushes_to_the_remote() {
    let (dir, bare) = bootstrapped_store_with_upstream();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let store = dir.path().join(".worktrees").join("odm");
    let local_head = branch_oid(&store, "HEAD");
    assert_ne!(local_head, branch_oid(bare.path(), "odm"), "not yet pushed");

    let r = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["action"], "pushed", "{v}");

    assert_eq!(
        branch_oid(bare.path(), "odm"),
        local_head,
        "the bare remote now has the local commit"
    );
}

#[test]
fn push_reports_in_plain_text_too() {
    let (dir, _bare) = bootstrapped_store_with_upstream();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "sync"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(r.err.contains("pushed 1 commit"), "{}", r.err);
}

// ----- F-3: divergence stops, changes nothing (also proves push never forces) ----
// `worktree::push` builds a plain `git push`, so a push into a remote that has
// moved fails rather than clobbering it; `diverged_store_changes_nothing`
// below is the end-to-end proof — sync classifies the state as `Diverged`
// *before* ever attempting a push, and the remote is asserted unchanged.

#[test]
fn diverged_store_changes_nothing() {
    let (dir, bare) = bootstrapped_store_with_upstream();
    // Advance both sides independently.
    advance_bare_repo(bare.path(), "odm");
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let store = dir.path().join(".worktrees").join("odm");
    let local_before = branch_oid(&store, "HEAD");
    let remote_before = branch_oid(bare.path(), "odm");
    assert_ne!(local_before, remote_before, "the two sides have parted");

    let r = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(r.code, Some(0), "diverged is reported, not an error:\n{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["action"], "diverged", "{v}");

    assert_eq!(branch_oid(&store, "HEAD"), local_before, "local HEAD did not move");
    assert_eq!(branch_oid(bare.path(), "odm"), remote_before, "remote HEAD did not move");
}

#[test]
fn diverged_reports_in_plain_text_too() {
    let (dir, bare) = bootstrapped_store_with_upstream();
    advance_bare_repo(bare.path(), "odm");
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "sync"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(r.err.contains("diverged"), "{}", r.err);
    assert!(r.err.contains("Nothing was changed"), "{}", r.err);
}

// ----- F-4: no-op cases -----------------------------------------------------------

#[test]
fn up_to_date_is_a_no_op() {
    let (dir, _bare) = bootstrapped_store_with_upstream();

    let r = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["action"], "up-to-date", "{v}");
    assert_eq!(v["upstream"]["ahead"], 0, "{v}");
    assert_eq!(v["upstream"]["behind"], 0, "{v}");
}

#[test]
fn no_upstream_is_not_an_error() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(r.code, Some(0), "no upstream is not an error:\n{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["action"], "no-upstream", "{v}");
    assert!(v["upstream"].is_null(), "{v}");
}

// ----- F-5: `--dry-run` fetches + compares but writes nothing --------------------

#[test]
fn dry_run_reports_would_pull_but_leaves_local_untouched() {
    let (dir, bare) = bootstrapped_store_with_upstream();
    advance_bare_repo(bare.path(), "odm");

    let store = dir.path().join(".worktrees").join("odm");
    let before = branch_oid(&store, "HEAD");

    let r = run(dir.path(), &["store", "sync", "--dry-run", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["action"], "pulled", "{v}");
    assert_eq!(v["dry_run"], true, "{v}");

    assert_eq!(branch_oid(&store, "HEAD"), before, "dry-run wrote nothing locally");

    // A real sync afterward still sees (and now performs) the same pull.
    let real = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(real.code, Some(0), "{}", real.err);
    assert_eq!(branch_oid(&store, "HEAD"), branch_oid(bare.path(), "odm"));
}

#[test]
fn dry_run_reports_would_push_but_leaves_remote_untouched() {
    let (dir, bare) = bootstrapped_store_with_upstream();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let remote_before = branch_oid(bare.path(), "odm");

    let r = run(dir.path(), &["store", "sync", "--dry-run", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["action"], "pushed", "{v}");
    assert_eq!(v["dry_run"], true, "{v}");

    assert_eq!(branch_oid(bare.path(), "odm"), remote_before, "dry-run pushed nothing");

    // A real sync afterward still performs the push.
    let real = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(real.code, Some(0), "{}", real.err);
    let store = dir.path().join(".worktrees").join("odm");
    assert_eq!(branch_oid(bare.path(), "odm"), branch_oid(&store, "HEAD"));
}

#[test]
fn dry_run_reports_in_plain_text_too() {
    let (dir, bare) = bootstrapped_store_with_upstream();
    advance_bare_repo(bare.path(), "odm");

    let r = run(dir.path(), &["store", "sync", "--dry-run"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(r.err.contains("dry-run"), "{}", r.err);
    assert!(r.err.contains("would fast-forward"), "{}", r.err);
}

// ----- F-6: `--json` shape --------------------------------------------------------

#[test]
fn json_shape_is_correct() {
    let (dir, _bare) = bootstrapped_store_with_upstream();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");

    assert_eq!(v["action"], "pushed", "{v}");
    assert_eq!(v["dry_run"], false, "{v}");
    assert_eq!(v["branch"], "odm", "{v}");
    assert!(v["store_root"].is_string(), "{v}");
    assert_eq!(v["upstream"]["ref"], "origin/odm", "{v}");
    assert_eq!(v["upstream"]["ahead"], 0, "{v}");
    assert_eq!(v["upstream"]["behind"], 0, "{v}");
}

// ----- Guard: no `[store]` section means no store to sync ------------------------

#[test]
fn without_a_store_section_sync_refuses_rather_than_touching_the_repo_root() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("odm.toml"), "author_name = \"Ada\"\n").unwrap();

    let r = run(dir.path(), &["store", "sync"]);
    assert_eq!(r.code, None, "no `[store]` section is an error, not a silent sync");
}
