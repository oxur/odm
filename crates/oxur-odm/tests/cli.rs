//! Binary-level end-to-end tests for the `odm` process (arc03/slice01, C-6/C-7).
//!
//! These are the first tests to drive the *built binary* through
//! [`assert_cmd`], so they exercise `odm_cli::run()` and the real
//! [`std::process::ExitCode`] — the path the in-process `dispatch` tests in
//! `odm-cli` cannot reach (they call `dispatch` and observe a `u8`). The real
//! binary reads the store from its current directory, so each test runs in a
//! fresh `TempDir`.

use assert_cmd::Command;
use tempfile::TempDir;

/// A fresh `odm` invocation rooted at `dir`'s path.
fn odm(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("odm").expect("the `odm` binary is built");
    cmd.current_dir(dir.path());
    cmd
}

/// Seeds a clean total tree: `project P(1) <- arc A(2) <- slice S(3)`.
fn seed_clean(dir: &TempDir) {
    odm(dir).args(["new", "project", "P"]).assert().success();
    odm(dir).args(["new", "arc", "A", "--parent", "1"]).assert().success();
    odm(dir).args(["new", "slice", "S", "--parent", "2"]).assert().success();
}

// ----- C-7: clean graph exits EXIT_OK (0) -----------------------------------

#[test]
fn check_exit_ok_on_clean_graph() {
    let dir = TempDir::new().unwrap();
    seed_clean(&dir);
    // The real process maps EXIT_OK → ExitCode(0).
    odm(&dir).arg("check").assert().success().code(0);
}

// ----- C-7: violating graph exits EXIT_VIOLATIONS (1) -----------------------

#[test]
fn check_exit_violations_on_dirty_graph() {
    let dir = TempDir::new().unwrap();
    // A parentless slice is an orphan (a hard error).
    odm(&dir).args(["new", "slice", "Orphan"]).assert().success();
    // The real process maps EXIT_VIOLATIONS → ExitCode(1).
    odm(&dir).arg("check").assert().failure().code(1);
}

// ----- C-6: --json shape on the real binary ---------------------------------

#[test]
fn check_json_shape_on_real_binary() {
    let dir = TempDir::new().unwrap();
    seed_clean(&dir);
    let output = odm(&dir).args(["check", "--json"]).assert().success().get_output().stdout.clone();
    let value: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON on stdout");

    // The stable v2 envelope (with the additive `tears` array).
    let mut keys: Vec<&String> = value.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(keys, ["errors", "findings", "ok", "tears", "warnings"]);
    assert_eq!(value["ok"], true);
    assert_eq!(value["errors"], 0);
}

// ----- C-6: a tear's rationale flows through the real binary into check ------

#[test]
fn tear_rationale_surfaces_in_real_check() {
    let dir = TempDir::new().unwrap();
    seed_clean(&dir); // P(1) <- A(2) <- S(3)
    odm(&dir).args(["new", "slice", "T", "--parent", "2"]).assert().success(); // T(4)

    // Create an ordering cycle S(3) <-> T(4), then tear S->T to break it.
    odm(&dir).args(["link", "3", "depends_on", "4"]).assert().success();
    odm(&dir).args(["link", "4", "depends_on", "3"]).assert().success();
    odm(&dir)
        .args(["tear", "3", "depends_on", "4", "--because", "T assumed ready"])
        .assert()
        .success();

    // The torn cycle passes, and `check` surfaces the persisted rationale.
    let human = odm(&dir).arg("check").assert().success().code(0).get_output().stdout.clone();
    let human = String::from_utf8(human).unwrap();
    assert!(human.contains("active tears"), "lists active tears:\n{human}");
    assert!(human.contains("T assumed ready"), "surfaces rationale:\n{human}");

    let json = odm(&dir).args(["check", "--json"]).assert().success().get_output().stdout.clone();
    let value: serde_json::Value = serde_json::from_slice(&json).unwrap();
    let tears = value["tears"].as_array().expect("tears array");
    assert_eq!(tears.len(), 1);
    assert_eq!(tears[0]["because"], "T assumed ready");
}

// ----- C-6: the real binary's error / version paths -------------------------

#[test]
fn usage_error_exits_two() {
    let dir = TempDir::new().unwrap();
    // clap rejects an unknown flag before dispatch → the binary exits 2.
    odm(&dir).args(["check", "--bogus"]).assert().failure().code(2);
}

#[test]
fn version_flag_succeeds() {
    let dir = TempDir::new().unwrap();
    odm(&dir).arg("--version").assert().success();
}

// ----- O-8: bare `odm` (no subcommand) orients and never bare-errors --------

#[test]
fn bare_odm_orients() {
    let dir = TempDir::new().unwrap();
    seed_clean(&dir); // project P(1) <- arc A(2) <- slice S(3): a single project

    // Bare `odm` dispatches to `orient` and exits 0 (no subcommand is not an
    // error) — the real process maps Ok(EXIT_OK) → ExitCode(0).
    let out = odm(&dir).assert().success().code(0).get_output().stdout.clone();
    let out = String::from_utf8(out).unwrap();
    assert!(out.contains("orient"), "bare odm runs orient:\n{out}");
    assert!(out.contains("VISION"), "orient leads with the vision section:\n{out}");
}
