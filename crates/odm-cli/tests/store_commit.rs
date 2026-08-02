//! `odm store commit` — persists the store worktree's pending node changes on
//! the orphan branch (arc-store-lifecycle, slice 01). Test ids (`F-N`) match
//! `docs/design-v1.0.0/arc-store-lifecycle/slice01-store-commit/ledger.md`.
//!
//! Fixture, class-(a) per the slice-doc: a **real** orphan-branch worktree,
//! stood up exactly as `odm store init` would (`init::bootstrap`), rather
//! than a hand-placed store — the point of this slice is what lands in real
//! git history.

use std::path::Path;
use std::process::Command;

use clap::Parser as _;
use odm_cli::Cli;
use odm_store::init::{self, Plan};
use odm_store::{Store, StoreHome};
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

/// A real git repo at `dir`, with one commit — `bootstrap` shells out to
/// `git worktree add`, which needs a resolvable `HEAD` (mirrors
/// `odm-store::init`'s own `git_init` test helper).
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
/// `odm store init` would — an unborn orphan branch, no commit yet.
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

// ----- F-1 / F-6: exists, persists, on the orphan branch, code branch untouched

#[test]
fn commit_persists_a_new_node_on_the_orphan_branch_and_leaves_the_code_branch_alone() {
    let dir = bootstrapped_store();
    let store = dir.path().join(".worktrees").join("odm");
    assert_eq!(commit_count(&store), 0, "the orphan branch starts unborn");

    run(dir.path(), &["node", "new", "slice", "A slice"]);
    let r = run(dir.path(), &["store", "commit"]);
    assert_eq!(r.code, Some(0), "commit succeeded:\n{}", r.err);

    assert_eq!(commit_count(&store), 1, "the orphan branch gained exactly one commit");
    // Same repo, different checked-out branch: this proves the code branch's
    // own history (the `initial` commit from `git_init`) was not touched.
    assert_eq!(commit_count(dir.path()), 1, "the code branch gained no commit");
}

// ----- F-2: auto-summary message + `-m` override -----------------------------

#[test]
fn default_message_reads_the_node_delta_in_odm_terms() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);

    let r = run(dir.path(), &["store", "commit", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["message"], "store: +1 slice", "{v}");
}

#[test]
fn dash_m_overrides_the_default_message() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);

    let r = run(dir.path(), &["store", "commit", "-m", "a custom message", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["message"], "a custom message", "{v}");
}

// ----- F-3: idempotent no-op when clean ---------------------------------------

#[test]
fn a_second_commit_on_a_clean_worktree_is_a_no_op() {
    let dir = bootstrapped_store();
    let store = dir.path().join(".worktrees").join("odm");
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));
    assert_eq!(commit_count(&store), 1);

    let r = run(dir.path(), &["store", "commit", "--json"]);
    assert_eq!(r.code, Some(0), "a clean worktree is not an error:\n{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["committed"], false, "{v}");
    assert_eq!(v["message"], "nothing to commit", "{v}");
    assert_eq!(commit_count(&store), 1, "no empty commit was made");
}

#[test]
fn a_clean_no_op_reports_in_plain_text_too() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "commit"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(r.err.contains("nothing to commit"), "{}", r.err);
}

// ----- F-4: `--dry-run` writes nothing -----------------------------------------

#[test]
fn dry_run_reports_the_delta_and_writes_no_commit() {
    let dir = bootstrapped_store();
    let store = dir.path().join(".worktrees").join("odm");
    run(dir.path(), &["node", "new", "slice", "A slice"]);

    let r = run(dir.path(), &["store", "commit", "--dry-run", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["committed"], false, "{v}");
    assert_eq!(v["delta"]["created"], 1, "{v}");
    assert_eq!(v["message"], "store: +1 slice", "{v}");

    // The branch was unborn coming out of bootstrap; a dry run must not have
    // given it a commit.
    assert_eq!(commit_count(&store), 0, "HEAD is still unborn — dry-run wrote nothing");

    // A real commit afterward still sees the same pending change.
    let real = run(dir.path(), &["store", "commit"]);
    assert_eq!(real.code, Some(0));
    assert_eq!(commit_count(&store), 1);
}

#[test]
fn dry_run_reports_in_plain_text_too() {
    let dir = bootstrapped_store();
    run(dir.path(), &["node", "new", "slice", "A slice"]);

    let r = run(dir.path(), &["store", "commit", "--dry-run"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(r.err.contains("dry-run"), "{}", r.err);
    assert!(r.err.contains("store: +1 slice"), "{}", r.err);
}

// ----- F-5: `--json` shape, on a real run and a no-op run ---------------------

#[test]
fn json_shape_is_correct_on_a_real_run() {
    let dir = bootstrapped_store();
    seed(dir.path());

    let r = run(dir.path(), &["store", "commit", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");

    assert_eq!(v["committed"], true, "{v}");
    assert!(v["sha"].is_string(), "{v}");
    assert_eq!(v["branch"], "odm", "{v}");
    assert!(v["message"].is_string(), "{v}");
    assert_eq!(v["delta"]["created"], 3, "{v}");
    assert_eq!(v["delta"]["modified"], 0, "{v}");
    assert_eq!(v["delta"]["removed"], 0, "{v}");
    assert_eq!(v["delta"]["by_type"]["project"]["created"], 1, "{v}");
    assert_eq!(v["delta"]["by_type"]["arc"]["created"], 1, "{v}");
    assert_eq!(v["delta"]["by_type"]["slice"]["created"], 1, "{v}");
}

#[test]
fn json_shape_is_correct_on_a_no_op_run() {
    let dir = bootstrapped_store();
    seed(dir.path());
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "commit", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");

    assert_eq!(v["committed"], false, "{v}");
    assert!(v["sha"].is_null(), "{v}");
    assert_eq!(v["branch"], "odm", "{v}");
    assert_eq!(v["message"], "nothing to commit", "{v}");
    assert_eq!(v["delta"]["created"], 0, "{v}");
}

// ----- Delta classification: modified + removed, not just created ------------

#[test]
fn a_later_commit_captures_modifications_and_removals_by_type() {
    let dir = bootstrapped_store();
    seed(dir.path());
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let store_root = StoreHome::resolve(dir.path()).store_root;
    let mut paths = Store::open(&store_root).node_paths().unwrap();
    paths.sort();
    assert_eq!(paths.len(), 3, "project, arc, slice");

    // Modify the first node file's body.
    let mut text = std::fs::read_to_string(&paths[0]).unwrap();
    text.push_str("\nAn added line.\n");
    std::fs::write(&paths[0], text).unwrap();

    // Remove the second node file outright.
    std::fs::remove_file(&paths[1]).unwrap();

    let r = run(dir.path(), &["store", "commit", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["committed"], true, "{v}");
    assert_eq!(v["delta"]["created"], 0, "{v}");
    assert_eq!(v["delta"]["modified"], 1, "{v}");
    assert_eq!(v["delta"]["removed"], 1, "{v}");
}

// ----- Guard: no `[store]` section means no orphan branch to commit to -------

#[test]
fn without_a_store_section_commit_refuses_rather_than_touching_the_repo_root() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("odm.toml"), "author_name = \"Ada\"\n").unwrap();

    let r = run(dir.path(), &["store", "commit"]);
    assert_eq!(r.code, None, "no `[store]` section is an error, not a silent commit");
}
