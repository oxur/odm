//! arc-store-home slice 03 — `odm store init`'s attach and ff-sync arms.
//!
//! Driven against **real git with a real remote**: a local bare repository
//! serves as `origin`, which gives genuine fetch/ancestry/fast-forward
//! behaviour with no network. Mocking git here would test the mock — the whole
//! point of these arms is what git actually does with a shared branch.
//!
//! The shape of every test is A → origin → B: repo A bootstraps and pushes,
//! repo B clones and attaches, and then the two take turns advancing.

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

/// Runs `git`, returning whether it succeeded (for the checks whose *failure*
/// is the assertion — e.g. "these histories have no merge-base").
fn git_ok(dir: &Path, args: &[&str]) -> bool {
    Command::new("git").args(args).current_dir(dir).output().expect("git runs").status.success()
}

fn odm(dir: &Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("odm").expect("the odm binary is built");
    cmd.current_dir(dir);
    cmd
}

/// Configures identity on a fresh repo (needed for commits in CI).
fn identify(dir: &Path) {
    git(dir, &["config", "user.name", "Test"]);
    git(dir, &["config", "user.email", "test@example.com"]);
}

/// A bare repository to act as `origin`.
fn bare_remote() -> TempDir {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "--bare", "--initial-branch=main"]);
    dir
}

/// Repo A: a code repo wired to `origin`, bootstrapped, with its `odm` branch
/// pushed. Returns the repo and the store root inside it.
fn origin_with_pushed_store(remote: &Path) -> TempDir {
    let a = TempDir::new().unwrap();
    git(a.path(), &["init", "--initial-branch=main"]);
    identify(a.path());
    std::fs::write(a.path().join("README.md"), "# code\n").unwrap();
    git(a.path(), &["add", "README.md"]);
    git(a.path(), &["commit", "-m", "initial"]);
    git(a.path(), &["remote", "add", "origin", &remote.to_string_lossy()]);
    git(a.path(), &["push", "-u", "origin", "main"]);

    odm(a.path()).args(["store", "init"]).assert().success();
    // Give the orphan branch its first commit, so there is a store to share.
    odm(a.path()).args(["new", "project", "Shared project"]).assert().success();
    let store = a.path().join(".worktrees").join("odm");
    git(&store, &["add", "-A"]);
    git(&store, &["commit", "-m", "store: initial"]);
    git(&store, &["push", "-u", "origin", "odm"]);
    a
}

/// Repo B: a clone of `origin` that has never seen the store.
fn clone_of(remote: &Path) -> TempDir {
    let b = TempDir::new().unwrap();
    git(b.path(), &["clone", &remote.to_string_lossy(), "."]);
    identify(b.path());
    b
}

// ----- L-1/L-2/L-3/L-4: attach ----------------------------------------------

#[test]
fn attach_checks_out_the_existing_branch_without_re_orphaning() {
    let remote = bare_remote();
    let _a = origin_with_pushed_store(remote.path());
    let b = clone_of(remote.path());

    odm(b.path()).args(["store", "init"]).assert().success();
    let store = b.path().join(".worktrees").join("odm");

    // L-1/L-3: the remote-only branch was fetched and checked out.
    assert!(store.is_dir(), "the worktree exists in the clone");
    assert_eq!(git(&store, &["branch", "--show-current"]), "odm");

    // L-2: it *shares history* with what A pushed — it was not re-orphaned.
    assert!(
        git_ok(b.path(), &["merge-base", "odm", "origin/odm"]),
        "attach checked the branch out; a re-orphan would have no merge-base"
    );

    // L-4: the store rode in with the branch, rather than being scaffolded.
    assert!(store.join("config.toml").is_file());
    assert!(store.join("nodes").is_dir());
    let listed = odm(b.path()).arg("list").assert().success().get_output().stdout.clone();
    assert!(
        String::from_utf8_lossy(&listed).contains("Shared project"),
        "A's node is present in B's attached store"
    );
    odm(b.path()).arg("check").assert().success().code(0);
}

#[test]
fn attach_does_not_overwrite_the_stores_own_config() {
    // A's config is *not* the scaffold default; attach must leave it alone.
    let remote = bare_remote();
    let a = origin_with_pushed_store(remote.path());
    let a_store = a.path().join(".worktrees").join("odm");
    std::fs::write(a_store.join("config.toml"), "# A's own config\n[display]\nmax_width = 99\n")
        .unwrap();
    git(&a_store, &["add", "-A"]);
    git(&a_store, &["commit", "-m", "store: custom config"]);
    git(&a_store, &["push", "origin", "odm"]);

    let b = clone_of(remote.path());
    odm(b.path()).args(["store", "init"]).assert().success();

    let config =
        std::fs::read_to_string(b.path().join(".worktrees/odm/config.toml")).expect("config");
    assert!(config.contains("max_width = 99"), "A's config survived attach:\n{config}");
    assert!(!config.contains("[gates.project]"), "it was not replaced by the scaffold default");
}

#[test]
fn attach_top_ups_are_idempotent() {
    let remote = bare_remote();
    let _a = origin_with_pushed_store(remote.path());
    let b = clone_of(remote.path());

    odm(b.path()).args(["store", "init"]).assert().success();
    let ignore = std::fs::read_to_string(b.path().join(".gitignore")).unwrap_or_default();
    assert_eq!(ignore.matches("/.worktrees/").count(), 1, "exactly one entry:\n{ignore}");
}

// ----- L-6/L-8/L-12: ff-sync ------------------------------------------------

#[test]
fn sync_fast_forwards_when_upstream_is_ahead() {
    let remote = bare_remote();
    let a = origin_with_pushed_store(remote.path());
    let b = clone_of(remote.path());
    odm(b.path()).args(["store", "init"]).assert().success();

    // A adds a node and pushes.
    odm(a.path()).args(["new", "arc", "Later work", "--parent", "1"]).assert().success();
    let a_store = a.path().join(".worktrees").join("odm");
    git(&a_store, &["add", "-A"]);
    git(&a_store, &["commit", "-m", "store: later work"]);
    git(&a_store, &["push", "origin", "odm"]);

    // B re-inits: this is the sync arm, and it should fast-forward.
    odm(b.path()).args(["store", "init"]).assert().success();

    let listed = odm(b.path()).arg("list").assert().success().get_output().stdout.clone();
    assert!(
        String::from_utf8_lossy(&listed).contains("Later work"),
        "B picked up A's new node by fast-forward"
    );
}

#[test]
fn sync_is_a_clean_no_op_when_up_to_date() {
    let remote = bare_remote();
    let _a = origin_with_pushed_store(remote.path());
    let b = clone_of(remote.path());
    odm(b.path()).args(["store", "init"]).assert().success();

    let before = git(&b.path().join(".worktrees").join("odm"), &["rev-parse", "HEAD"]);
    odm(b.path())
        .args(["store", "init", "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("sync-up-to-date"));
    let after = git(&b.path().join(".worktrees").join("odm"), &["rev-parse", "HEAD"]);
    assert_eq!(before, after, "nothing moved");
}

// ----- L-9: the safety invariant --------------------------------------------

#[test]
fn sync_stops_on_divergence_and_touches_nothing() {
    let remote = bare_remote();
    let a = origin_with_pushed_store(remote.path());
    let b = clone_of(remote.path());
    odm(b.path()).args(["store", "init"]).assert().success();
    let b_store = b.path().join(".worktrees").join("odm");

    // B commits locally…
    odm(b.path()).args(["new", "arc", "B's work", "--parent", "1"]).assert().success();
    git(&b_store, &["add", "-A"]);
    git(&b_store, &["commit", "-m", "store: B's work"]);
    let b_head = git(&b_store, &["rev-parse", "HEAD"]);

    // …while A pushes something different. Now the histories have parted.
    odm(a.path()).args(["new", "arc", "A's work", "--parent", "1"]).assert().success();
    let a_store = a.path().join(".worktrees").join("odm");
    git(&a_store, &["add", "-A"]);
    git(&a_store, &["commit", "-m", "store: A's work"]);
    git(&a_store, &["push", "origin", "odm"]);

    odm(b.path())
        .args(["store", "init"])
        .assert()
        .success()
        .stderr(predicates::str::contains("diverged"));

    // The invariant: nothing was merged, rebased, or moved.
    assert_eq!(git(&b_store, &["rev-parse", "HEAD"]), b_head, "B's branch did not move");
    let listed = odm(b.path()).arg("list").assert().success().get_output().stdout.clone();
    let listed = String::from_utf8_lossy(&listed);
    assert!(listed.contains("B's work"), "B's own work is intact");
    assert!(!listed.contains("A's work"), "and A's was not merged in behind B's back");
}

// ----- L-10/L-11: nothing to do, reported as such ---------------------------

#[test]
fn sync_without_an_upstream_warns_and_succeeds() {
    // A repo with no remote at all: bootstrap, then re-init.
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "--initial-branch=main"]);
    identify(dir.path());
    std::fs::write(dir.path().join("README.md"), "# code\n").unwrap();
    git(dir.path(), &["add", "README.md"]);
    git(dir.path(), &["commit", "-m", "initial"]);
    odm(dir.path()).args(["store", "init"]).assert().success();
    odm(dir.path()).args(["new", "project", "P"]).assert().success();

    odm(dir.path())
        .args(["store", "init"])
        .assert()
        .success()
        .stderr(predicates::str::contains("nothing to sync from"));
    odm(dir.path()).arg("check").assert().success().code(0);
}

#[test]
fn sync_reports_local_ahead_without_erroring() {
    let remote = bare_remote();
    let _a = origin_with_pushed_store(remote.path());
    let b = clone_of(remote.path());
    odm(b.path()).args(["store", "init"]).assert().success();
    let b_store = b.path().join(".worktrees").join("odm");

    odm(b.path()).args(["new", "arc", "Unpushed", "--parent", "1"]).assert().success();
    git(&b_store, &["add", "-A"]);
    git(&b_store, &["commit", "-m", "store: unpushed"]);

    let out = odm(b.path())
        .args(["store", "init", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["mode"], "sync-local-ahead");
    assert_eq!(v["local_ahead"], 1, "the count of unpushed commits: {v}");
}

// ----- L-14: `--dry-run` and `--json` on every arm ---------------------------

#[test]
fn dry_run_touches_nothing_on_the_attach_arm() {
    let remote = bare_remote();
    let _a = origin_with_pushed_store(remote.path());
    let b = clone_of(remote.path());

    odm(b.path())
        .args(["store", "init", "--dry-run", "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"mode\": \"attach\""));

    assert!(!b.path().join(".worktrees").exists(), "no worktree was created");
    assert_eq!(git(b.path(), &["branch", "--list", "odm"]), "", "no local branch was created");
}

#[test]
fn dry_run_touches_nothing_on_the_sync_arm() {
    let remote = bare_remote();
    let a = origin_with_pushed_store(remote.path());
    let b = clone_of(remote.path());
    odm(b.path()).args(["store", "init"]).assert().success();
    let b_store = b.path().join(".worktrees").join("odm");
    let before = git(&b_store, &["rev-parse", "HEAD"]);

    // Upstream advances, so a real run *would* fast-forward.
    odm(a.path()).args(["new", "arc", "Later", "--parent", "1"]).assert().success();
    let a_store = a.path().join(".worktrees").join("odm");
    git(&a_store, &["add", "-A"]);
    git(&a_store, &["commit", "-m", "store: later"]);
    git(&a_store, &["push", "origin", "odm"]);

    odm(b.path())
        .args(["store", "init", "--dry-run", "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("sync-fast-forwarded"));

    assert_eq!(git(&b_store, &["rev-parse", "HEAD"]), before, "the dry run moved nothing");
}

// ----- L-13: repair is detected, never performed ----------------------------

#[test]
fn a_missing_worktree_is_reported_not_silently_repaired() {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "--initial-branch=main"]);
    identify(dir.path());
    std::fs::write(dir.path().join("README.md"), "# code\n").unwrap();
    git(dir.path(), &["add", "README.md"]);
    git(dir.path(), &["commit", "-m", "initial"]);
    odm(dir.path()).args(["store", "init"]).assert().success();
    odm(dir.path()).args(["new", "project", "P"]).assert().success();

    // Commit so the branch exists as a ref, then remove the worktree directory
    // the way an impatient `rm -rf` would.
    let store = dir.path().join(".worktrees").join("odm");
    git(&store, &["add", "-A"]);
    git(&store, &["commit", "-m", "store: initial"]);
    std::fs::remove_dir_all(&store).unwrap();

    odm(dir.path())
        .args(["store", "init"])
        .assert()
        .success()
        .stderr(predicates::str::contains("--force"));

    assert!(!store.exists(), "odm did not silently re-create it");
    assert_ne!(git(dir.path(), &["rev-parse", "odm"]), "", "and the branch is untouched");
}
