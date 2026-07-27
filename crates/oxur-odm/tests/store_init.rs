//! arc-store-home slice 02 — `odm store init`, end to end against a real git.
//!
//! These run the built binary in a scratch repository, because the whole point
//! of the slice is a `git` shell-out: an in-process test would exercise
//! everything except the part that can actually fail. The assertions are made
//! against git itself (`branch --show-current`, `merge-base`) rather than
//! against odm's own report, so a bug in the reporting cannot mask a bug in the
//! creation.

use std::path::Path;
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use tempfile::TempDir;

/// Runs `git` in `dir`, returning trimmed stdout; panics with git's stderr.
fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(dir).output().expect("git runs");
    assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A git repository with one commit on the code branch — the state a user runs
/// `odm store init` from.
fn repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "--initial-branch=main"]);
    git(dir.path(), &["config", "user.name", "Test"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    std::fs::write(dir.path().join("README.md"), "# code\n").unwrap();
    git(dir.path(), &["add", "README.md"]);
    git(dir.path(), &["commit", "-m", "initial"]);
    dir
}

fn odm(dir: &Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("odm").expect("the odm binary is built");
    cmd.current_dir(dir);
    cmd
}

// ----- RH C-5: derived state stays out of the store's history ---------------

#[test]
fn the_scaffolded_store_ignores_the_index_but_keeps_the_context() {
    let dir = repo();
    odm(dir.path()).args(["store", "init"]).assert().success();
    let store = dir.path().join(".worktrees").join("odm");

    // Make odm produce both kinds of derived state, then ask git what it sees.
    odm(dir.path()).args(["node", "new", "project", "P"]).assert().success();
    odm(dir.path()).args(["validate"]).assert().success();
    assert!(store.join(".odm").is_dir(), "the run produced derived state to ignore");

    odm(dir.path()).args(["use", "project", "P"]).assert().success();
    assert!(store.join(".odm").join("index").is_file(), "there is an index to ignore");
    assert!(store.join(".odm").join("context.json").is_file(), "there is a context to keep");

    let untracked = git(&store, &["status", "--porcelain", "--untracked-files=all"]);
    assert!(
        !untracked.contains(".odm/index"),
        "the index is a derived cache and must not be offered for commit on a shared \
         branch; got:\n{untracked}"
    );
    assert!(
        !untracked.contains(".odm/drift"),
        "the drift snapshot is a cache too — the rule ignores the directory rather than \
         naming each cache, so a new one is covered on arrival; got:\n{untracked}"
    );
    // The other half, and the one that matters: ignoring the cache must not
    // also withhold the focus, which is what a fresh session needs from
    // `orient`. A blanket `/.odm/` would pass the assertion above and fail this.
    assert!(
        untracked.contains(".odm/context.json"),
        "the current focus travels with the store; got:\n{untracked}"
    );
    assert!(untracked.contains("nodes/"), "nodes are still tracked; got:\n{untracked}");
    assert!(untracked.contains("config.toml"), "the config is still tracked; got:\n{untracked}");
}

// ----- L-6/L-7/L-8/L-9: bootstrap stands the home up ------------------------

#[test]
fn init_bootstraps_the_store_home() {
    let dir = repo();
    odm(dir.path()).args(["store", "init"]).assert().success();
    let store = dir.path().join(".worktrees").join("odm");

    // L-6: the worktree exists, on the branch, with a *disjoint* history.
    assert!(store.is_dir(), "the worktree was created");
    assert_eq!(git(&store, &["branch", "--show-current"]), "odm", "on the orphan branch");
    let merge_base = Command::new("git")
        .args(["merge-base", "main", "odm"])
        .current_dir(dir.path())
        .output()
        .expect("git runs");
    assert!(
        !merge_base.status.success(),
        "orphan: no merge-base with the code branch, got {:?}",
        String::from_utf8_lossy(&merge_base.stdout)
    );

    // L-7: the locator now redirects resolution.
    let locator = std::fs::read_to_string(dir.path().join("odm.toml")).unwrap();
    assert!(locator.contains("[store]"), "locator written:\n{locator}");
    assert!(locator.contains("branch_name = \"odm\""), "with its names:\n{locator}");

    // L-8: the store is scaffolded, and the locator stays locator-only.
    assert!(store.join("config.toml").is_file(), "config.toml in the store");
    assert!(store.join("nodes").is_dir(), "empty nodes/ in the store");
    assert!(
        !locator.contains("[gates."),
        "operational keys did not leak into the locator:\n{locator}"
    );
    let config = std::fs::read_to_string(store.join("config.toml")).unwrap();
    assert!(config.contains("[gates.slice]"), "operational defaults scaffolded:\n{config}");

    // The store holds *only* the store. `git checkout --orphan` keeps the
    // checkout it came from, so the old-git fallback has to clear it — without
    // that, the code branch's files land here and get staged for the store's
    // first commit. Both paths must produce the same empty home.
    assert!(!store.join("README.md").exists(), "the code branch's files did not come along");
    let staged = git(&store, &["status", "--porcelain"]);
    assert!(
        staged.lines().all(|l| !l.contains("README.md")),
        "nothing from the code branch is staged in the store:\n{staged}"
    );

    // L-9: the worktree is ignored on the code branch.
    let ignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(ignore.contains("/.worktrees/"), "gitignored:\n{ignore}");
    assert!(
        git(dir.path(), &["status", "--porcelain"]).lines().all(|l| !l.contains(".worktrees")),
        "and therefore never staged"
    );
}

// ----- L-11: the home is live ------------------------------------------------

#[test]
fn nodes_land_in_the_new_home_and_check_is_green() {
    let dir = repo();
    odm(dir.path()).args(["store", "init"]).assert().success();
    let store = dir.path().join(".worktrees").join("odm");

    odm(dir.path()).args(["node", "new", "project", "A project"]).assert().success();
    odm(dir.path()).args(["node", "new", "arc", "An arc", "--parent", "1"]).assert().success();

    assert!(!dir.path().join("nodes").exists(), "nothing written at the repo root");
    let count = walkdir(&store.join("nodes"));
    assert_eq!(count, 2, "both nodes are under the store");

    odm(dir.path()).arg("check").assert().success().code(0);
}

/// Counts `.md` files under `dir`.
fn walkdir(dir: &Path) -> usize {
    let mut n = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                n += 1;
            }
        }
    }
    n
}

// ----- L-12: the context rides with the store -------------------------------

#[test]
fn context_is_written_under_the_store_root() {
    let dir = repo();
    odm(dir.path()).args(["store", "init"]).assert().success();
    odm(dir.path()).args(["node", "new", "project", "A project"]).assert().success();
    odm(dir.path()).args(["use", "project", "1"]).assert().success();

    let store = dir.path().join(".worktrees").join("odm");
    assert!(
        store.join(".odm").join("context.json").is_file(),
        "context rides with the store, like the index"
    );
    assert!(
        !dir.path().join(".odm").join("context.json").exists(),
        "and is not left at the invocation root"
    );
    // It reads back through the same resolution.
    odm(dir.path())
        .arg("project")
        .assert()
        .success()
        .stdout(predicates::str::contains("A project"));
}

// ----- L-10: `--dry-run` and `--json` ---------------------------------------

#[test]
fn dry_run_touches_nothing() {
    let dir = repo();
    odm(dir.path()).args(["store", "init", "--dry-run"]).assert().success();

    assert!(!dir.path().join(".worktrees").exists(), "no worktree");
    assert!(!dir.path().join("odm.toml").exists(), "no locator");
    assert!(!dir.path().join(".gitignore").exists(), "no gitignore");
    assert_eq!(git(dir.path(), &["branch", "--list", "odm"]), "", "no branch");
}

#[test]
fn json_reports_the_created_home() {
    let dir = repo();
    let out = odm(dir.path())
        .args(["store", "init", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");

    assert_eq!(v["mode"], "bootstrap");
    assert_eq!(v["dry_run"], false);
    assert_eq!(v["branch"], "odm");
    assert!(
        v["store_root"].as_str().unwrap().ends_with(".worktrees/odm"),
        "store_root reported: {v}"
    );
    assert!(v["git_version"].is_string(), "the git that ran is named: {v}");
}

// ----- L-5: an existing branch is never clobbered ---------------------------

#[test]
fn an_existing_branch_stops_without_touching_anything() {
    let dir = repo();
    // A branch exists but no worktree does. Slice 02 deferred this to "slice
    // 03"; slice 03 classifies it precisely — a half-finished or removed
    // worktree — and still refuses to act, pointing at the future `--force`.
    // The safety claim is unchanged and is what this test pins: an existing
    // branch is never re-orphaned or clobbered.
    git(dir.path(), &["branch", "odm"]);

    odm(dir.path())
        .args(["store", "init"])
        .assert()
        .success()
        .stderr(predicates::str::contains("--force"));

    assert!(!dir.path().join(".worktrees").exists(), "no worktree created");
    assert!(!dir.path().join("odm.toml").exists(), "no locator written");
    // The branch is untouched — still pointing where it did.
    assert_eq!(
        git(dir.path(), &["rev-parse", "odm"]),
        git(dir.path(), &["rev-parse", "main"]),
        "the existing branch was not re-orphaned"
    );
}

// ----- custom names ----------------------------------------------------------

#[test]
fn custom_worktree_and_branch_names_are_honoured() {
    let dir = repo();
    odm(dir.path())
        .args(["store", "init", "--worktree", "planning", "--branch", "plan"])
        .assert()
        .success();

    let store = dir.path().join(".worktrees").join("planning");
    assert!(store.is_dir(), "the named worktree was created");
    assert_eq!(git(&store, &["branch", "--show-current"]), "plan");

    // And resolution follows it: a node lands there.
    odm(dir.path()).args(["node", "new", "project", "P"]).assert().success();
    assert!(store.join("nodes").is_dir());
}

// ----- re-running init is refused, not destructive ---------------------------

#[test]
fn a_second_init_refuses_rather_than_clobbering() {
    let dir = repo();
    odm(dir.path()).args(["store", "init"]).assert().success();
    odm(dir.path()).args(["node", "new", "project", "A project"]).assert().success();

    // The branch now exists locally, so this is a sync — deferred, untouched.
    odm(dir.path()).args(["store", "init"]).assert().success();

    let store = dir.path().join(".worktrees").join("odm");
    assert_eq!(walkdir(&store.join("nodes")), 1, "the existing node survived");
}
