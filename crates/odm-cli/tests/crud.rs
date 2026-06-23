//! End-to-end CLI tests (assert_cmd), driving the `odm-cli` binary against a
//! temp store. Test names contain the substrings the ledger Verify commands
//! filter on (`new_persists`, `new_idempotent`, `list_filters`, `show_node`,
//! `rename_keeps_id_and_path`, `retire_preserves_file`, `supersede_with_kind`,
//! `context_use_and_show`, `dry_run_and_yes`, `json_schema_crud`).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use tempfile::TempDir;

/// An `odm-cli` invocation rooted at `dir`.
fn odm(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("odm-cli").expect("odm-cli binary");
    cmd.current_dir(dir);
    cmd
}

/// Counts `.md` files under `nodes/`.
fn node_count(dir: &Path) -> usize {
    walk_md(&dir.join("nodes"))
}

fn walk_md(dir: &Path) -> usize {
    let mut n = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                n += walk_md(&p);
            } else if p.extension().is_some_and(|x| x == "md") {
                n += 1;
            }
        }
    }
    n
}

// ----- K-1: new persists ----------------------------------------------------

#[test]
fn new_persists_a_node() {
    let dir = TempDir::new().unwrap();
    odm(dir.path())
        .args(["new", "slice", "Store layer"])
        .assert()
        .success()
        .stderr(predicates::str::contains("created slice #1"));
    assert_eq!(node_count(dir.path()), 1);
}

// ----- K-2: new is idempotent describe-or-create ----------------------------

#[test]
fn new_idempotent_does_not_duplicate() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "slice", "Same"]).assert().success();
    odm(dir.path())
        .args(["new", "slice", "Same"])
        .assert()
        .success()
        .stderr(predicates::str::contains("exists: slice #1"));
    assert_eq!(node_count(dir.path()), 1, "re-running new must not duplicate");
}

// ----- K-3: list filters ----------------------------------------------------

#[test]
fn list_filters_by_type() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "slice", "A slice"]).assert().success();
    odm(dir.path()).args(["new", "arc", "An arc"]).assert().success();

    // Unfiltered: both.
    odm(dir.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("A slice").and(predicates::str::contains("An arc")));

    // Filter by type=arc: only the arc.
    odm(dir.path()).args(["list", "--type", "arc"]).assert().success().stdout(
        predicates::str::contains("An arc").and(predicates::str::contains("A slice").not()),
    );
}

// ----- K-4: show -----------------------------------------------------------

#[test]
fn show_node_renders_details_and_children() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "arc", "Parent arc"]).assert().success();
    odm(dir.path()).args(["show", "1"]).assert().success().stdout(
        predicates::str::contains("arc #1 Parent arc").and(predicates::str::contains("id:")),
    );
}

// ----- K-5: rename keeps id and path ----------------------------------------

#[test]
fn rename_keeps_id_and_path() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "slice", "Before"]).assert().success();

    // Capture the id + file path before rename.
    let before_files = list_md_paths(&dir.path().join("nodes"));
    assert_eq!(before_files.len(), 1);
    let path_before = before_files[0].clone();
    let id_before = path_before.file_stem().unwrap().to_string_lossy().to_string();

    odm(dir.path()).args(["rename", "1", "After"]).assert().success();

    let after_files = list_md_paths(&dir.path().join("nodes"));
    assert_eq!(after_files.len(), 1, "rename must not create a new file");
    assert_eq!(after_files[0], path_before, "the on-disk path is unchanged");

    // The id is unchanged and the new name is present.
    odm(dir.path())
        .args(["show", &id_before])
        .assert()
        .success()
        .stdout(predicates::str::contains("After").and(predicates::str::contains(&id_before)));
}

// ----- K-6: retire preserves the file ---------------------------------------

#[test]
fn retire_preserves_file_not_deleted() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "odd", "Old design"]).assert().success();
    let path = list_md_paths(&dir.path().join("nodes"))[0].clone();

    odm(dir.path())
        .args(["retire", "1", "--because", "superseded by 0013"])
        .assert()
        .success()
        .stderr(predicates::str::contains("retired #1"));

    assert!(path.exists(), "retire must not delete the file");
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("retired:"), "retirement is recorded in frontmatter");
    assert!(content.contains("superseded by 0013"));
}

// ----- K-7: supersede with kind ---------------------------------------------

#[test]
fn supersede_with_kind_records_edge() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "odd", "Old"]).assert().success();
    odm(dir.path()).args(["new", "odd", "New"]).assert().success();

    odm(dir.path())
        .args(["supersede", "1", "--with", "2", "--kind", "obsoletes"])
        .assert()
        .success()
        .stderr(predicates::str::contains("#2 supersedes #1"));

    // The edge is on the newer node (#2), pointing at the old one, with kind.
    odm(dir.path()).args(["show", "2", "--json"]).assert().success().stdout(
        predicates::str::contains("\"supersedes\"").and(predicates::str::contains("obsoletes")),
    );
}

// ----- K-8: use + context ---------------------------------------------------

#[test]
fn context_use_and_show() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "project", "Odm"]).assert().success();
    odm(dir.path()).args(["new", "arc", "Substrate"]).assert().success();

    odm(dir.path()).args(["use", "project", "1"]).assert().success();
    odm(dir.path()).args(["use", "arc", "2"]).assert().success();

    odm(dir.path()).args(["context"]).assert().success().stdout(
        predicates::str::contains("project: #1 Odm")
            .and(predicates::str::contains("arc:     #2 Substrate")),
    );

    // `use project` on an arc is rejected.
    odm(dir.path()).args(["use", "project", "2"]).assert().failure();
}

// ----- K-9: --dry-run writes nothing; --yes runs ----------------------------

#[test]
fn dry_run_and_yes() {
    let dir = TempDir::new().unwrap();

    // --dry-run on `new` writes nothing.
    odm(dir.path())
        .args(["new", "slice", "Ghost", "--dry-run"])
        .assert()
        .success()
        .stderr(predicates::str::contains("would create"));
    assert_eq!(node_count(dir.path()), 0, "--dry-run must not persist");

    // --yes runs the mutation non-interactively.
    odm(dir.path()).args(["new", "slice", "Real", "--yes"]).assert().success();
    assert_eq!(node_count(dir.path()), 1);

    // --dry-run on rename leaves the name unchanged.
    odm(dir.path()).args(["rename", "1", "Renamed", "--dry-run"]).assert().success();
    odm(dir.path())
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Real"));
}

// ----- K-10: --json stable schema -------------------------------------------

#[test]
fn json_schema_crud_is_stable() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "slice", "Schema check"]).assert().success();

    let output = odm(dir.path()).args(["show", "1", "--json"]).output().unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");

    // Stable, documented key set.
    let obj = value.as_object().expect("JSON object");
    let mut keys: Vec<&String> = obj.keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "component",
            "id",
            "name",
            "number",
            "origin",
            "part_of",
            "reserved",
            "retired",
            "supersedes",
            "tags",
            "type",
        ]
    );
    // Stable field values (id is a 26-char ULID, not pinned).
    assert_eq!(obj["type"], "slice");
    assert_eq!(obj["number"], 1);
    assert_eq!(obj["name"], "Schema check");
    assert_eq!(obj["origin"], "planned");
    assert_eq!(obj["reserved"], false);
    assert_eq!(obj["id"].as_str().unwrap().len(), 26);
}

// --- helpers ---------------------------------------------------------------

fn list_md_paths(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    collect_md(dir, &mut out);
    out
}

fn collect_md(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_md(&p, out);
            } else if p.extension().is_some_and(|x| x == "md") {
                out.push(p);
            }
        }
    }
}
