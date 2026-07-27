//! arc-store-home slice 04 — `odm store rename`, end to end against real git.
//!
//! The store root is *derived* from worktree-dir + branch-name, so a rename has
//! to move the git side and rewrite the locator in step. These tests check both
//! halves every time — that git moved, **and** that odm can still find the
//! corpus afterwards — because either alone passing would hide the failure that
//! matters: a store that exists but cannot be resolved.

use std::path::Path;
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use tempfile::TempDir;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(dir).output().expect("git runs");
    assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn odm(dir: &Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("odm").expect("the odm binary is built");
    cmd.current_dir(dir);
    cmd
}

/// A repo with a bootstrapped store holding one node.
fn store_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "--initial-branch=main"]);
    git(dir.path(), &["config", "user.name", "Test"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    std::fs::write(dir.path().join("README.md"), "# code\n").unwrap();
    git(dir.path(), &["add", "README.md"]);
    git(dir.path(), &["commit", "-m", "initial"]);
    odm(dir.path()).args(["store", "init"]).assert().success();
    odm(dir.path()).args(["node", "new", "project", "Kept project"]).assert().success();
    dir
}

/// The `[store]` section of the locator.
fn locator(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("odm.toml")).unwrap_or_default()
}

/// Asserts the store resolves and still holds the seeded node.
fn resolves_with_the_corpus(dir: &Path) {
    let out = odm(dir).args(["node", "list"]).assert().success().get_output().stdout.clone();
    assert!(
        String::from_utf8_lossy(&out).contains("Kept project"),
        "the corpus is still reachable after the rename"
    );
    odm(dir).arg("check").assert().success().code(0);
}

// ----- L-2/L-4: worktree-only rename ----------------------------------------

#[test]
fn renaming_the_worktree_moves_it_and_updates_the_locator() {
    let dir = store_repo();
    odm(dir.path()).args(["store", "rename", "--worktree", "planning"]).assert().success();

    assert!(dir.path().join(".worktrees/planning").is_dir(), "the new path exists");
    assert!(!dir.path().join(".worktrees/odm").exists(), "the old path is gone");

    // git's own view agrees.
    let listed = git(dir.path(), &["worktree", "list"]);
    assert!(listed.contains(".worktrees/planning"), "git knows the new path:\n{listed}");

    let text = locator(dir.path());
    assert!(text.contains("worktree_name = \"planning\""), "locator updated:\n{text}");
    assert!(text.contains("branch_name = \"odm\""), "the branch was left alone:\n{text}");
    resolves_with_the_corpus(dir.path());
}

// ----- L-3/L-4: branch-only rename ------------------------------------------

#[test]
fn renaming_the_branch_updates_the_worktree_and_the_locator() {
    let dir = store_repo();
    odm(dir.path()).args(["store", "rename", "--branch", "plan"]).assert().success();

    let store = dir.path().join(".worktrees/odm");
    assert_eq!(
        git(&store, &["branch", "--show-current"]),
        "plan",
        "the worktree is on the new branch"
    );

    let text = locator(dir.path());
    assert!(text.contains("branch_name = \"plan\""), "locator updated:\n{text}");
    assert!(text.contains("worktree_name = \"odm\""), "the directory was left alone:\n{text}");
    resolves_with_the_corpus(dir.path());
}

// ----- L-2/L-3/L-4: the bare positional renames both ------------------------

#[test]
fn the_bare_form_renames_both_halves_together() {
    let dir = store_repo();
    odm(dir.path()).args(["store", "rename", "planning"]).assert().success();

    let store = dir.path().join(".worktrees/planning");
    assert!(store.is_dir());
    assert_eq!(git(&store, &["branch", "--show-current"]), "planning");

    let text = locator(dir.path());
    assert!(text.contains("worktree_name = \"planning\""));
    assert!(text.contains("branch_name = \"planning\""));
    assert_eq!(text.matches("[store]").count(), 1, "one store block, not two:\n{text}");
    resolves_with_the_corpus(dir.path());
}

#[test]
fn renames_compose_across_repeated_runs() {
    // The locator must survive being rewritten more than once.
    let dir = store_repo();
    odm(dir.path()).args(["store", "rename", "one"]).assert().success();
    odm(dir.path()).args(["store", "rename", "two"]).assert().success();
    odm(dir.path()).args(["store", "rename", "three"]).assert().success();

    let text = locator(dir.path());
    assert_eq!(text.matches("[store]").count(), 1, "no stacking:\n{text}");
    assert!(text.contains("worktree_name = \"three\""));
    assert!(dir.path().join(".worktrees/three").is_dir());
    resolves_with_the_corpus(dir.path());
}

// ----- L-7: collisions stop, they do not clobber ----------------------------

#[test]
fn a_collision_with_an_existing_directory_stops_and_changes_nothing() {
    let dir = store_repo();
    // Something already occupies the target. `git worktree move` would happily
    // move *into* it, so odm has to refuse first.
    std::fs::create_dir_all(dir.path().join(".worktrees/taken")).unwrap();
    std::fs::write(dir.path().join(".worktrees/taken/theirs.txt"), "not ours\n").unwrap();

    odm(dir.path())
        .args(["store", "rename", "--worktree", "taken"])
        .assert()
        .success()
        .stderr(predicates::str::contains("already exists"));

    assert!(dir.path().join(".worktrees/odm").is_dir(), "the store did not move");
    assert!(
        dir.path().join(".worktrees/taken/theirs.txt").exists(),
        "and the occupant was not disturbed"
    );
    assert!(locator(dir.path()).contains("worktree_name = \"odm\""), "the locator is unchanged");
    resolves_with_the_corpus(dir.path());
}

#[test]
fn a_collision_with_an_existing_branch_stops() {
    let dir = store_repo();
    git(dir.path(), &["branch", "taken", "main"]);

    odm(dir.path())
        .args(["store", "rename", "--branch", "taken"])
        .assert()
        .success()
        .stderr(predicates::str::contains("already exists"));

    let store = dir.path().join(".worktrees/odm");
    assert_eq!(git(&store, &["branch", "--show-current"]), "odm", "the branch did not move");
    assert_eq!(
        git(dir.path(), &["rev-parse", "taken"]),
        git(dir.path(), &["rev-parse", "main"]),
        "and the existing branch was not clobbered"
    );
}

// ----- L-8: no-op ------------------------------------------------------------

#[test]
fn renaming_to_the_current_name_is_a_clean_no_op() {
    let dir = store_repo();
    let before = locator(dir.path());

    odm(dir.path())
        .args(["store", "rename", "odm"])
        .assert()
        .success()
        .stderr(predicates::str::contains("nothing to do"));

    assert_eq!(locator(dir.path()), before, "the locator was not rewritten");
    resolves_with_the_corpus(dir.path());
}

// ----- L-9: in-flight work survives the move --------------------------------

#[test]
fn uncommitted_work_survives_the_move() {
    let dir = store_repo();
    // A node created but never committed to the store branch — the ordinary
    // state, since odm does not auto-commit.
    let scratch = dir.path().join(".worktrees/odm/scratch.txt");
    std::fs::write(&scratch, "in flight\n").unwrap();

    odm(dir.path()).args(["store", "rename", "moved"]).assert().success();

    let moved = dir.path().join(".worktrees/moved/scratch.txt");
    assert!(moved.is_file(), "the uncommitted file came along");
    assert_eq!(std::fs::read_to_string(moved).unwrap(), "in flight\n", "byte-for-byte");
    resolves_with_the_corpus(dir.path());
}

// ----- L-5: the always-resolvable invariant, under a partial failure --------

#[test]
fn a_partial_failure_still_leaves_the_store_resolvable() {
    // The worktree move succeeds, then git rejects the branch name. Written
    // naively the locator would never be updated, so it would name a directory
    // that no longer exists — and resolution self-heals an empty store at the
    // stale path, giving a *green* `check` over an invisible corpus. That is
    // the worst failure this slice can produce, so it is pinned here.
    let dir = store_repo();

    odm(dir.path())
        .args(["store", "rename", "--worktree", "moved", "--branch", "bad..name"])
        .assert()
        .failure()
        // The error says what *did* happen, not just what broke.
        .stderr(predicates::str::contains("intact and resolvable"));

    // git did move the worktree…
    assert!(dir.path().join(".worktrees/moved").is_dir(), "the move stands");
    assert!(!dir.path().join(".worktrees/odm").exists());
    // …and the locator followed it, rather than being left behind.
    let text = locator(dir.path());
    assert!(text.contains("worktree_name = \"moved\""), "locator mirrors reality:\n{text}");
    assert!(text.contains("branch_name = \"odm\""), "the branch did not change:\n{text}");

    // The corpus is still there — the assertion that would have failed before.
    resolves_with_the_corpus(dir.path());
}

// ----- L-6: the published-branch guard --------------------------------------

#[test]
fn renaming_a_published_branch_warns_that_it_is_local_only() {
    let dir = store_repo();
    let remote = TempDir::new().unwrap();
    git(remote.path(), &["init", "--bare", "--initial-branch=main"]);
    git(dir.path(), &["remote", "add", "origin", &remote.path().to_string_lossy()]);

    // Publish the store branch.
    let store = dir.path().join(".worktrees/odm");
    git(&store, &["add", "-A"]);
    git(&store, &["commit", "-m", "store: initial"]);
    git(&store, &["push", "-u", "origin", "odm"]);

    odm(dir.path())
        .args(["store", "rename", "--branch", "plan"])
        .assert()
        .success()
        .stderr(predicates::str::contains("locally only"));

    // Local moved; the remote did not.
    assert_eq!(git(&store, &["branch", "--show-current"]), "plan");
    let remotes = git(dir.path(), &["for-each-ref", "--format=%(refname)", "refs/remotes/"]);
    assert!(remotes.contains("origin/odm"), "origin still has the old name:\n{remotes}");
    assert!(!remotes.contains("origin/plan"), "and odm did not invent a remote rename");
    resolves_with_the_corpus(dir.path());
}

// ----- L-10: `--dry-run` and `--json` ---------------------------------------

#[test]
fn dry_run_reports_and_touches_nothing() {
    let dir = store_repo();
    let before = locator(dir.path());

    odm(dir.path()).args(["store", "rename", "planning", "--dry-run"]).assert().success();

    assert!(dir.path().join(".worktrees/odm").is_dir(), "the store did not move");
    assert!(!dir.path().join(".worktrees/planning").exists(), "the target was not created");
    assert_eq!(locator(dir.path()), before, "the locator is untouched");
    assert_eq!(git(dir.path(), &["branch", "--list", "planning"]), "", "no branch was created");
}

#[test]
fn json_reports_old_and_new() {
    let dir = store_repo();
    let out = odm(dir.path())
        .args(["store", "rename", "planning", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");

    assert_eq!(v["outcome"], "renamed");
    assert_eq!(v["old_branch"], "odm");
    assert_eq!(v["new_branch"], "planning");
    assert!(v["old_worktree"].as_str().unwrap().ends_with(".worktrees/odm"));
    assert!(v["new_worktree"].as_str().unwrap().ends_with(".worktrees/planning"));
    assert_eq!(v["store_root"], v["new_worktree"], "store_root is the observed new path");
}

#[test]
fn json_reports_a_collision_rather_than_claiming_success() {
    let dir = store_repo();
    std::fs::create_dir_all(dir.path().join(".worktrees/taken")).unwrap();

    let out = odm(dir.path())
        .args(["store", "rename", "--worktree", "taken", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["outcome"], "collision");
    assert!(v["collision"].as_str().unwrap().contains("taken"), "it names what collided: {v}");
}

// ----- a store that was never given a home ----------------------------------

#[test]
fn renaming_without_a_store_section_explains_rather_than_failing_obscurely() {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "--initial-branch=main"]);
    git(dir.path(), &["config", "user.name", "Test"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    odm(dir.path()).args(["node", "new", "project", "P"]).assert().success();

    odm(dir.path())
        .args(["store", "rename", "planning"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("odm store init"));
}
