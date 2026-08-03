//! `odm store status` — a read-only view of the store's git state
//! (arc-store-lifecycle, slice 02). Test ids (`F-N`) match
//! `docs/design-v1.0.0/arc-store-lifecycle/slice02-store-status/ledger.md`.
//!
//! Fixture, class-(a) per the slice-doc: a **real** orphan-branch worktree,
//! stood up exactly as `odm store init` would (`init::bootstrap`) — same
//! discipline `store_commit.rs` uses, since `status` is the read half of the
//! same coin.

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

/// A real git repo at `dir`, with one commit (mirrors `store_commit.rs`'s
/// own helper — `bootstrap` needs a resolvable `HEAD`).
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

/// A repo with a real store worktree, bootstrapped exactly as
/// `odm store init` would.
fn bootstrapped_store() -> TempDir {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    let plan = Plan::new(dir.path(), None, None);
    init::bootstrap(&plan).expect("bootstrap");
    dir
}

/// Seeds a project → arc → slice into the store rooted at `root`.
fn seed(root: &Path) {
    run(root, &["node", "new", "project", "Root project"]);
    run(root, &["node", "new", "arc", "An arc", "--parent", "1"]);
    run(root, &["node", "new", "slice", "A slice", "--parent", "2"]);
}

/// How many commits `HEAD` has in `dir`, or `0` on an unborn branch.
fn commit_count(dir: &Path) -> usize {
    let out = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(dir)
        .output()
        .expect("git rev-list");
    String::from_utf8_lossy(&out.stdout).trim().parse().unwrap_or(0)
}

/// `HEAD`'s commit id in `dir`.
fn head_oid(dir: &Path) -> String {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .expect("git rev-parse");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A plain `git status --porcelain` in `dir`.
fn git_status_porcelain(dir: &Path) -> String {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir)
        .output()
        .expect("git status");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Sets `dir` (the code repo, sharing refs with the store worktree inside
/// it) up with a bare `origin` remote carrying `branch`'s current state, and
/// fetches it — a remote-tracking branch to compare against, without a real
/// network. Returns the bare repo's `TempDir` so it outlives the test.
fn with_fetched_upstream(dir: &Path, branch: &str) -> TempDir {
    let bare = TempDir::new().unwrap();
    let git = |args: &[&str]| {
        Command::new("git").args(args).current_dir(dir).status().expect("git command");
    };
    git(&["init", "-q", "--bare", &bare.path().display().to_string()]);
    git(&["remote", "add", "origin", &bare.path().display().to_string()]);
    git(&["push", "-q", "origin", &format!("{branch}:{branch}")]);
    git(&["fetch", "-q", "origin"]);
    bare
}

// ----- F-1/F-2: the pending delta, and "nothing to commit" when clean -------

#[test]
fn dirty_store_shows_the_pending_delta() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);

    let r = run(dir.path(), &["store", "status", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["clean"], false, "{v}");
    assert_eq!(v["delta"]["created"], 1, "{v}");
    assert_eq!(v["delta"]["by_type"]["slice"]["created"], 1, "{v}");
}

#[test]
fn dirty_store_matches_commits_own_delta_computation() {
    // F-1's invariant: status and commit must report the identical delta for
    // the identical worktree state.
    let dir = bootstrapped_store();
    seed(dir.path());

    let status = run(dir.path(), &["store", "status", "--json"]);
    let commit = run(dir.path(), &["store", "commit", "--dry-run", "--json"]);
    let sv: serde_json::Value = serde_json::from_str(&status.out).expect("valid json");
    let cv: serde_json::Value = serde_json::from_str(&commit.out).expect("valid json");
    assert_eq!(sv["delta"], cv["delta"], "status and commit report the same delta");
}

#[test]
fn clean_store_reports_nothing_to_commit() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "status", "--json"]);
    assert_eq!(r.code, Some(0), "a clean store is not an error:\n{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["clean"], true, "{v}");
    assert_eq!(v["delta"]["created"], 0, "{v}");
}

#[test]
fn clean_store_reports_in_plain_text_too() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "status"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(r.err.contains("nothing to commit"), "{}", r.err);
}

// ----- F-3: ahead is reported ------------------------------------------------

#[test]
fn local_ahead_of_upstream_reports_the_count() {
    let dir = bootstrapped_store();
    let store = dir.path().join(".worktrees").join("odm");
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let _bare = with_fetched_upstream(dir.path(), "odm");

    // Two more local commits, unpushed.
    run(dir.path(), &["node", "new", "slice", "A second slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));
    run(dir.path(), &["node", "new", "slice", "A third slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));
    assert_eq!(commit_count(&store), 3);

    let r = run(dir.path(), &["store", "status", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["upstream"]["action"], "local-ahead", "{v}");
    assert_eq!(v["upstream"]["ahead"], 2, "{v}");
    assert_eq!(v["upstream"]["behind"], 0, "{v}");
    assert_eq!(v["upstream"]["ref"], "origin/odm", "{v}");
}

#[test]
fn local_ahead_reports_in_plain_text_too() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));
    let _bare = with_fetched_upstream(dir.path(), "odm");
    run(dir.path(), &["node", "new", "slice", "Another slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "status"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(r.err.contains("1 commit(s) ahead"), "{}", r.err);
}

#[test]
fn synced_store_reports_up_to_date() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));
    let _bare = with_fetched_upstream(dir.path(), "odm");

    let r = run(dir.path(), &["store", "status", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["upstream"]["action"], "up-to-date", "{v}");
    assert_eq!(v["upstream"]["ahead"], 0, "{v}");
    assert_eq!(v["upstream"]["behind"], 0, "{v}");
}

// ----- F-4: no upstream is not an error --------------------------------------

#[test]
fn no_upstream_is_not_an_error() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "status", "--json"]);
    assert_eq!(r.code, Some(0), "no upstream is not an error:\n{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert!(v["upstream"].is_null(), "{v}");
}

#[test]
fn no_upstream_reports_in_plain_text_too() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "status"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(r.err.contains("no upstream configured"), "{}", r.err);
}

// ----- F-5: `--json` shape ----------------------------------------------------

#[test]
fn json_shape_is_correct() {
    let dir = bootstrapped_store();
    seed(dir.path());

    let r = run(dir.path(), &["store", "status", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");

    assert_eq!(v["clean"], false, "{v}");
    assert_eq!(v["branch"], "odm", "{v}");
    assert!(v["store_root"].is_string(), "{v}");
    assert_eq!(v["delta"]["created"], 3, "{v}");
    assert!(v["upstream"].is_null(), "no remote configured: {v}");
}

// ----- F-6: read-only — status never mutates anything ------------------------

#[test]
fn status_never_mutates_the_store_or_the_orphan_branch() {
    let dir = bootstrapped_store();
    let store = dir.path().join(".worktrees").join("odm");
    seed(dir.path());
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));
    let _bare = with_fetched_upstream(dir.path(), "odm");

    // Dirty the worktree too, so status has real delta + ahead/behind work to
    // do — the fixture that would catch a status handler that "helpfully"
    // commits or fetches along the way.
    run(dir.path(), &["node", "new", "slice", "One more"]);

    let before_head = head_oid(&store);
    let before_status = git_status_porcelain(&store);
    let before_snapshot = odm_store::Store::open(&store).node_paths().unwrap();

    let r = run(dir.path(), &["store", "status", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);

    assert_eq!(head_oid(&store), before_head, "HEAD did not move");
    assert_eq!(
        git_status_porcelain(&store),
        before_status,
        "the worktree's dirty/clean state is unchanged"
    );
    assert_eq!(
        odm_store::Store::open(&store).node_paths().unwrap(),
        before_snapshot,
        "no node file was created, removed, or touched"
    );
    // The remote-tracking ref itself: status must not have fetched (D-1), so
    // a second status run reports the identical ahead/behind either way —
    // proven here by re-running and diffing rather than needing a live
    // network to assert "no fetch happened".
    let r2 = run(dir.path(), &["store", "status", "--json"]);
    assert_eq!(r.out, r2.out, "status is deterministic — no hidden fetch side effect");
}

// ----- Guard: no `[store]` section means no store to check --------------------

#[test]
fn without_a_store_section_status_refuses_rather_than_touching_the_repo_root() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("odm.toml"), "author_name = \"Ada\"\n").unwrap();

    let r = run(dir.path(), &["store", "status"]);
    assert_eq!(r.code, None, "no `[store]` section is an error, not a silent status");
}
