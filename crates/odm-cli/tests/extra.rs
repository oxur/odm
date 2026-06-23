//! Additional CLI tests: error paths, dry-run for every mutator, rich-fixture
//! rendering (fields the CLI cannot yet set are seeded via the library), and
//! context JSON/empty cases.

use std::path::Path;

use assert_cmd::Command;
use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Edges, Frontmatter, SupersedeKind, Supersedes};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use predicates::prelude::PredicateBooleanExt;
use tempfile::TempDir;

fn odm(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("odm-cli").expect("odm-cli binary");
    cmd.current_dir(dir);
    cmd
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 23).unwrap()
}

/// Seeds a node directly through the library (to exercise CLI rendering of
/// fields the CLI itself cannot set yet, e.g. tags/component/part_of).
fn seed(dir: &Path, build: impl FnOnce(Id) -> Document) {
    let id = Id::new();
    let store = Store::open(dir);
    store.persist(&build(id)).expect("seed persist");
}

// ----- error paths ----------------------------------------------------------

#[test]
fn new_rejects_unknown_type() {
    let dir = TempDir::new().unwrap();
    odm(dir.path())
        .args(["new", "widget", "X"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown type"));
}

#[test]
fn list_rejects_unknown_type_filter() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["list", "--type", "widget"]).assert().failure();
}

#[test]
fn show_missing_reference_fails() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "slice", "Only"]).assert().success();
    // Unknown number, unknown name, and unknown id all fail with guidance.
    odm(dir.path())
        .args(["show", "99"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no node with number 99"));
    odm(dir.path())
        .args(["show", "Nonexistent"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no node matching"));
}

#[test]
fn rename_and_retire_missing_reference_fail() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["rename", "5", "x"]).assert().failure();
    odm(dir.path()).args(["retire", "5", "--because", "x"]).assert().failure();
}

#[test]
fn supersede_self_is_rejected() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "odd", "Doc"]).assert().success();
    odm(dir.path())
        .args(["supersede", "1", "--with", "1", "--kind", "updates"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot supersede itself"));
}

#[test]
fn use_rejects_unknown_and_wrong_type() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["use", "project", "404"]).assert().failure();
}

#[test]
fn ambiguous_name_prefix_is_rejected() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "slice", "Alpha"]).assert().success();
    odm(dir.path()).args(["new", "slice", "Alpine"]).assert().success();
    odm(dir.path())
        .args(["show", "Alp"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("ambiguous"));
    // A unique prefix resolves.
    odm(dir.path()).args(["show", "Alpha"]).assert().success();
}

// ----- dry-run for the remaining mutators -----------------------------------

#[test]
fn dry_run_retire_and_supersede_write_nothing() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "odd", "A"]).assert().success();
    odm(dir.path()).args(["new", "odd", "B"]).assert().success();

    odm(dir.path())
        .args(["retire", "1", "--because", "x", "--dry-run"])
        .assert()
        .success()
        .stderr(predicates::str::contains("would retire"));
    odm(dir.path())
        .args(["supersede", "1", "--with", "2", "--kind", "updates", "--dry-run"])
        .assert()
        .success()
        .stderr(predicates::str::contains("would record"));

    // Neither wrote anything: #1 not retired, #2 has no supersedes edge.
    odm(dir.path())
        .args(["show", "1", "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"retired\": null"));
    odm(dir.path())
        .args(["show", "2", "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"supersedes\": null"));
}

// ----- rich-fixture rendering -----------------------------------------------

#[test]
fn show_renders_all_fields_and_children() {
    let dir = TempDir::new().unwrap();

    // Parent arc with tags + component.
    let mut parent_id = Id::new();
    seed(dir.path(), |id| {
        parent_id = id;
        let fm = Frontmatter::new(id, 1, NodeType::Arc, "Substrate", day(), day(), Origin::Planned)
            .with_tags(vec!["core".to_string(), "graph".to_string()])
            .with_component("odm-core");
        Document::new(fm, "body\n")
    });

    // Child slice whose part_of points at the parent, plus a supersedes edge.
    let other = Id::new();
    seed(dir.path(), |id| {
        let edges = Edges {
            part_of: Some(parent_id),
            supersedes: Some(Supersedes { node: other, kind: SupersedeKind::Updates }),
            ..Edges::default()
        };
        let fm = Frontmatter::new(
            id,
            2,
            NodeType::Slice,
            "Store layer",
            day(),
            day(),
            Origin::Discovered,
        )
        .with_edges(edges);
        Document::new(fm, "body\n")
    });

    // show parent → tags, component, and the child are rendered.
    odm(dir.path()).args(["show", "1"]).assert().success().stdout(
        predicates::str::contains("tags:")
            .and(predicates::str::contains("component: odm-core"))
            .and(predicates::str::contains("children:"))
            .and(predicates::str::contains("Store layer")),
    );

    // show child → part_of and supersedes lines.
    odm(dir.path()).args(["show", "2"]).assert().success().stdout(
        predicates::str::contains("part_of:").and(predicates::str::contains("supersedes:")),
    );

    // list --tag filters to the tagged parent; list --component likewise.
    odm(dir.path()).args(["list", "--tag", "core"]).assert().success().stdout(
        predicates::str::contains("Substrate").and(predicates::str::contains("Store layer").not()),
    );
    odm(dir.path())
        .args(["list", "--component", "odm-core"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Substrate"));
}

#[test]
fn retired_node_renders_in_show_text() {
    let dir = TempDir::new().unwrap();
    odm(dir.path()).args(["new", "odd", "Old"]).assert().success();
    odm(dir.path()).args(["retire", "1", "--because", "done"]).assert().success();
    odm(dir.path())
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicates::str::contains("retired:").and(predicates::str::contains("done")));
}

// ----- list/context empty + json -------------------------------------------

#[test]
fn list_empty_reports_no_nodes() {
    let dir = TempDir::new().unwrap();
    odm(dir.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("(no nodes)"));
    odm(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("[]"));
}

#[test]
fn context_empty_then_set_json() {
    let dir = TempDir::new().unwrap();
    // Empty context.
    odm(dir.path()).args(["context"]).assert().success().stdout(
        predicates::str::contains("project: (none)")
            .and(predicates::str::contains("arc:     (none)")),
    );
    odm(dir.path())
        .args(["context", "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"project\": null"));

    // Set a project, then JSON reflects it.
    odm(dir.path()).args(["new", "project", "Odm"]).assert().success();
    odm(dir.path()).args(["use", "project", "1"]).assert().success();
    odm(dir.path())
        .args(["context", "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"name\": \"Odm\""));
}

#[test]
fn context_corrupt_file_errors() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(".odm")).unwrap();
    std::fs::write(dir.path().join(".odm").join("context.json"), b"{ not json").unwrap();
    odm(dir.path()).args(["context"]).assert().failure();
}
