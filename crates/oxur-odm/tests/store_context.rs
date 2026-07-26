//! The CLI context follows the store, not the invocation directory (RH C-5).
//!
//! `use` writes a node-id selection and `orient` reads it back. Both resolve
//! `.odm/context.json`, and for as long as the store *was* the invocation root
//! they agreed by accident — so a test in an un-redirected repository proves
//! nothing about which root either one uses.
//!
//! These run against a **redirected** store (`odm store init`, worktree home),
//! where the two roots differ. That is the only arrangement in which the bug
//! this pins was observable: `use` reported success, wrote the selection under
//! the store, and `orient` — reading the invocation root — printed "no current
//! arc". Success and failure looked identical from the outside.

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

/// A git repository with one commit on the code branch.
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

fn stdout(cmd: assert_cmd::assert::Assert) -> String {
    String::from_utf8_lossy(&cmd.get_output().stdout).to_string()
}

/// Just the CURRENT FOCUS block of an `orient` render.
///
/// The whole output is the wrong thing to assert against: every node's name
/// also appears in READY, so `output.contains("Test arc")` is satisfied by a
/// completely empty focus. The section has to be isolated for the assertion to
/// mean what it says.
fn focus_block(orient: &str) -> String {
    orient
        .split("CURRENT FOCUS")
        .nth(1)
        .unwrap_or_else(|| panic!("orient renders a CURRENT FOCUS section; got:\n{orient}"))
        .split("\n\n")
        .next()
        .unwrap_or_default()
        .to_string()
}

/// A repository whose store is redirected into a worktree, holding one project
/// and one arc — the shape odm itself has after the cutover.
fn repo_with_redirected_store() -> (TempDir, std::path::PathBuf) {
    let dir = repo();
    odm(dir.path()).args(["store", "init"]).assert().success();
    let store = dir.path().join(".worktrees").join("odm");
    odm(dir.path()).args(["new", "project", "Test project"]).assert().success();
    odm(dir.path()).args(["new", "arc", "Test arc"]).assert().success();
    (dir, store)
}

#[test]
fn use_writes_the_context_into_the_store_not_the_invocation_root() {
    let (dir, store) = repo_with_redirected_store();

    odm(dir.path()).args(["use", "arc", "Test arc"]).assert().success();

    // Both halves: it is in the store *and* it is not at the invocation root.
    // Asserting only the first would pass with the context written to both, and
    // asserting only the second would pass with it written to neither.
    assert!(store.join(".odm").join("context.json").is_file(), "the context lives in the store");
    assert!(
        !dir.path().join(".odm").join("context.json").exists(),
        "nothing is written at the invocation root — the context follows the corpus it names"
    );
}

#[test]
fn orient_reads_a_selection_use_just_wrote() {
    let (dir, _store) = repo_with_redirected_store();

    odm(dir.path()).args(["use", "arc", "Test arc"]).assert().success();
    let out = stdout(odm(dir.path()).args(["orient"]).assert().success());
    let focus = focus_block(&out);

    assert!(focus.contains("Test arc"), "the focus block names the selected arc; got:\n{focus}");
    assert!(
        !focus.contains("no current arc"),
        "orient must not report an empty focus after a successful `use`; got:\n{focus}"
    );
}

#[test]
fn context_and_orient_agree_on_the_selection() {
    let (dir, _store) = repo_with_redirected_store();

    odm(dir.path()).args(["use", "arc", "Test arc"]).assert().success();

    // `context` read the store root even before the fix, so it agreeing with
    // `orient` is the actual invariant: the two commands cannot disagree about
    // what is selected.
    let context = stdout(odm(dir.path()).args(["context"]).assert().success());
    let focus = focus_block(&stdout(odm(dir.path()).args(["orient"]).assert().success()));
    assert!(context.contains("Test arc"), "context reports the arc; got:\n{context}");
    assert!(focus.contains("Test arc"), "orient's focus reports the same arc; got:\n{focus}");
}

#[test]
fn an_unselected_store_still_reports_an_empty_focus() {
    let (dir, _store) = repo_with_redirected_store();

    // The negative case still has to work: no `use`, no focus. Without this the
    // fix could "pass" by reporting some arc unconditionally.
    let focus = focus_block(&stdout(odm(dir.path()).args(["orient"]).assert().success()));
    assert!(focus.contains("no current arc"), "an unselected store has no focus; got:\n{focus}");
}
