//! In-process tests for the node CRUD surface.
//!
//! Each case parses args into a [`Cli`] and drives [`odm_cli::dispatch`]
//! against a temp store with captured output buffers — no spawned binary, no
//! global-cwd mutation. Test names contain the substrings the ledger Verify
//! commands filter on (`new_persists`, `new_idempotent`, `list_filters`,
//! `show_node`, `rename_keeps_id_and_path`, `retire_preserves_file`,
//! `supersede_with_kind`, `context_use_and_show`, `dry_run_and_yes`,
//! `json_schema_crud`).

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Edges, Frontmatter, SupersedeKind, Supersedes};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use tempfile::TempDir;

/// The result of one in-process command run.
struct Run {
    ok: bool,
    out: String,
    /// The diagnostics stream, mirroring what the user sees: the `err` buffer
    /// plus, on failure, the returned error rendered as `run` would print it.
    err: String,
}

/// Runs `odm <args>` in-process against a store rooted at `root`.
fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args should be structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, root, &mut out, &mut err);
    let mut err_text = String::from_utf8(err).unwrap();
    if let Err(e) = &result {
        err_text.push_str(&format!("error: {e:#}"));
    }
    Run { ok: result.is_ok(), out: String::from_utf8(out).unwrap(), err: err_text }
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 23).unwrap()
}

fn md_paths(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_md(&root.join("nodes"), &mut out);
    out
}

fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) {
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

/// Seeds a node directly through the library (to exercise CLI rendering of
/// fields the CLI itself cannot set yet, e.g. tags/component/part_of).
fn seed(root: &Path, build: impl FnOnce(Id) -> Document) {
    let store = Store::open(root);
    store.persist(&build(Id::new())).expect("seed persist");
}

// ----- K-1: new persists ----------------------------------------------------

#[test]
fn new_persists_a_node() {
    let dir = TempDir::new().unwrap();
    let r = run(dir.path(), &["new", "slice", "Store layer"]);
    assert!(r.ok);
    assert!(r.err.contains("created slice #1"), "stderr: {}", r.err);
    assert_eq!(md_paths(dir.path()).len(), 1);
}

// ----- K-2: idempotent describe-or-create -----------------------------------

#[test]
fn new_idempotent_does_not_duplicate() {
    let dir = TempDir::new().unwrap();
    assert!(run(dir.path(), &["new", "slice", "Same"]).ok);
    let r = run(dir.path(), &["new", "slice", "Same"]);
    assert!(r.ok);
    assert!(r.err.contains("exists: slice #1"), "stderr: {}", r.err);
    assert_eq!(md_paths(dir.path()).len(), 1, "re-running new must not duplicate");
}

// ----- K-3: list filters ----------------------------------------------------

#[test]
fn list_filters_by_type_tag_component() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "A slice"]);
    run(dir.path(), &["new", "arc", "An arc"]);

    let all = run(dir.path(), &["list"]);
    assert!(all.out.contains("A slice") && all.out.contains("An arc"));

    let arcs = run(dir.path(), &["list", "--type", "arc"]);
    assert!(arcs.out.contains("An arc") && !arcs.out.contains("A slice"));
}

// ----- K-4: show -----------------------------------------------------------

#[test]
fn show_node_renders_details() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "arc", "Parent arc"]);
    let r = run(dir.path(), &["show", "1"]);
    assert!(r.ok);
    assert!(r.out.contains("arc #1 Parent arc") && r.out.contains("id:"));
    assert!(r.out.contains("children:  (none)"));
}

// ----- K-5: rename keeps id and path ----------------------------------------

#[test]
fn rename_keeps_id_and_path() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "Before"]);

    let before = md_paths(dir.path());
    assert_eq!(before.len(), 1);
    let path = before[0].clone();
    let id = path.file_stem().unwrap().to_string_lossy().to_string();

    assert!(run(dir.path(), &["rename", "1", "After"]).ok);

    let after = md_paths(dir.path());
    assert_eq!(after.len(), 1, "rename must not create a new file");
    assert_eq!(after[0], path, "the on-disk path is unchanged");

    // Resolving by the original id still works and shows the new name.
    let shown = run(dir.path(), &["show", &id]);
    assert!(shown.out.contains("After") && shown.out.contains(&id));
}

// ----- K-6: retire preserves the file ---------------------------------------

#[test]
fn retire_preserves_file_not_deleted() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "odd", "Old design"]);
    let path = md_paths(dir.path())[0].clone();

    let r = run(dir.path(), &["retire", "1", "--because", "superseded by 0013"]);
    assert!(r.ok && r.err.contains("retired #1"));

    assert!(path.exists(), "retire must not delete the file");
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("retired:"), "retirement recorded in frontmatter");
    assert!(content.contains("superseded by 0013"));
}

// ----- K-7: supersede with kind ---------------------------------------------

#[test]
fn supersede_with_kind_records_edge() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "odd", "Old"]);
    run(dir.path(), &["new", "odd", "New"]);

    let r = run(dir.path(), &["supersede", "1", "--with", "2", "--kind", "obsoletes"]);
    assert!(r.ok && r.err.contains("#2 supersedes #1"));

    // The edge is on the newer node (#2), pointing at the old one, with kind.
    let shown = run(dir.path(), &["show", "2", "--json"]);
    assert!(shown.out.contains("\"supersedes\"") && shown.out.contains("obsoletes"));
}

// ----- K-8: use + context ---------------------------------------------------

#[test]
fn context_use_and_show() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "project", "Odm"]);
    run(dir.path(), &["new", "arc", "Substrate"]);

    assert!(run(dir.path(), &["use", "project", "1"]).ok);
    assert!(run(dir.path(), &["use", "arc", "2"]).ok);

    let ctx = run(dir.path(), &["context"]);
    assert!(ctx.out.contains("project: #1 Odm") && ctx.out.contains("arc:     #2 Substrate"));

    // `use project` on an arc is rejected.
    assert!(!run(dir.path(), &["use", "project", "2"]).ok);
}

// ----- K-9: --dry-run writes nothing; --yes runs ----------------------------

#[test]
fn dry_run_and_yes() {
    let dir = TempDir::new().unwrap();

    let dry = run(dir.path(), &["new", "slice", "Ghost", "--dry-run"]);
    assert!(dry.ok && dry.err.contains("would create"));
    assert_eq!(md_paths(dir.path()).len(), 0, "--dry-run must not persist");

    assert!(run(dir.path(), &["new", "slice", "Real", "--yes"]).ok);
    assert_eq!(md_paths(dir.path()).len(), 1);

    // Dry-run mutators on existing nodes also write nothing.
    run(dir.path(), &["new", "odd", "A"]);
    run(dir.path(), &["new", "odd", "B"]);
    assert!(run(dir.path(), &["retire", "2", "--because", "x", "--dry-run"]).ok);
    assert!(
        run(dir.path(), &["supersede", "2", "--with", "3", "--kind", "updates", "--dry-run"]).ok
    );
    assert!(run(dir.path(), &["show", "2", "--json"]).out.contains("\"retired\": null"));
    assert!(run(dir.path(), &["show", "3", "--json"]).out.contains("\"supersedes\": null"));
}

// ----- K-10: --json stable schema -------------------------------------------

#[test]
fn json_schema_crud_is_stable() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "Schema check"]);

    let r = run(dir.path(), &["show", "1", "--json"]);
    assert!(r.ok);
    let value: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");
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
    assert_eq!(obj["type"], "slice");
    assert_eq!(obj["number"], 1);
    assert_eq!(obj["name"], "Schema check");
    assert_eq!(obj["origin"], "planned");
    assert_eq!(obj["reserved"], false);
    assert_eq!(obj["id"].as_str().unwrap().len(), 26);
}

// ----- error paths ----------------------------------------------------------

#[test]
fn new_rejects_unknown_type() {
    let dir = TempDir::new().unwrap();
    let r = run(dir.path(), &["new", "widget", "X"]);
    assert!(!r.ok && r.err.contains("unknown type"));
}

#[test]
fn list_rejects_unknown_type_filter() {
    let dir = TempDir::new().unwrap();
    assert!(!run(dir.path(), &["list", "--type", "widget"]).ok);
}

#[test]
fn show_missing_reference_fails() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "Only"]);
    assert!(run(dir.path(), &["show", "99"]).err.contains("no node with number 99"));
    assert!(run(dir.path(), &["show", "Nonexistent"]).err.contains("no node matching"));
    // A well-formed but absent ULID resolves via the id branch and fails.
    assert!(!run(dir.path(), &["show", "01ARZ3NDEKTSV4RRFFQ69G5FAV"]).ok);
}

#[test]
fn rename_and_retire_missing_reference_fail() {
    let dir = TempDir::new().unwrap();
    assert!(!run(dir.path(), &["rename", "5", "x"]).ok);
    assert!(!run(dir.path(), &["retire", "5", "--because", "x"]).ok);
}

#[test]
fn supersede_self_is_rejected() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "odd", "Doc"]);
    let r = run(dir.path(), &["supersede", "1", "--with", "1", "--kind", "updates"]);
    assert!(!r.ok && r.err.contains("cannot supersede itself"));
}

#[test]
fn use_rejects_unknown_reference() {
    let dir = TempDir::new().unwrap();
    assert!(!run(dir.path(), &["use", "project", "404"]).ok);
}

#[test]
fn ambiguous_name_prefix_is_rejected() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "Alpha"]);
    run(dir.path(), &["new", "slice", "Alpine"]);
    assert!(run(dir.path(), &["show", "Alp"]).err.contains("ambiguous"));
    assert!(run(dir.path(), &["show", "Alpha"]).ok); // a unique prefix resolves
}

// ----- rich-fixture rendering (fields the CLI can't yet set) -----------------

#[test]
fn show_renders_all_fields_and_children() {
    let dir = TempDir::new().unwrap();

    let mut parent_id = Id::new();
    seed(dir.path(), |id| {
        parent_id = id;
        let fm = Frontmatter::new(id, 1, NodeType::Arc, "Substrate", day(), day(), Origin::Planned)
            .with_tags(vec!["core".to_string(), "graph".to_string()])
            .with_component("odm-core");
        Document::new(fm, "body\n")
    });
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

    let parent = run(dir.path(), &["show", "1"]);
    assert!(
        parent.out.contains("tags:")
            && parent.out.contains("component: odm-core")
            && parent.out.contains("children:")
            && parent.out.contains("Store layer")
    );

    let child = run(dir.path(), &["show", "2"]);
    assert!(child.out.contains("part_of:") && child.out.contains("supersedes:"));

    let tagged = run(dir.path(), &["list", "--tag", "core"]);
    assert!(tagged.out.contains("Substrate") && !tagged.out.contains("Store layer"));
    assert!(run(dir.path(), &["list", "--component", "odm-core"]).out.contains("Substrate"));
}

#[test]
fn retired_node_renders_in_show_text() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "odd", "Old"]);
    run(dir.path(), &["retire", "1", "--because", "done"]);
    let r = run(dir.path(), &["show", "1"]);
    assert!(r.out.contains("retired:") && r.out.contains("done"));
}

// ----- empty + json edges ---------------------------------------------------

#[test]
fn list_empty_reports_no_nodes() {
    let dir = TempDir::new().unwrap();
    assert!(run(dir.path(), &["list"]).out.contains("(no nodes)"));
    assert!(run(dir.path(), &["list", "--json"]).out.contains("[]"));
}

#[test]
fn context_empty_then_set_json() {
    let dir = TempDir::new().unwrap();
    let empty = run(dir.path(), &["context"]);
    assert!(empty.out.contains("project: (none)") && empty.out.contains("arc:     (none)"));
    assert!(run(dir.path(), &["context", "--json"]).out.contains("\"project\": null"));

    run(dir.path(), &["new", "project", "Odm"]);
    run(dir.path(), &["use", "project", "1"]);
    assert!(run(dir.path(), &["context", "--json"]).out.contains("\"name\": \"Odm\""));
}

#[test]
fn context_corrupt_file_errors() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(".odm")).unwrap();
    std::fs::write(dir.path().join(".odm").join("context.json"), b"{ not json").unwrap();
    assert!(!run(dir.path(), &["context"]).ok);
}
