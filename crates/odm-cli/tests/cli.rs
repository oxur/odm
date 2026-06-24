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
use std::str::FromStr;

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
    /// The exit code dispatch returned, or `None` if it errored (which `run`
    /// maps to exit code 2).
    code: Option<u8>,
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
    Run {
        ok: result.is_ok(),
        code: result.as_ref().ok().copied(),
        out: String::from_utf8(out).unwrap(),
        err: err_text,
    }
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

// ===========================================================================
// check v1
// ===========================================================================

const MISSING_ID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FZZ";

/// Seeds a single slice node with the given number/name and edge tweaks.
fn seed_slice(root: &Path, number: u32, name: &str, edit: impl FnOnce(&mut Frontmatter)) {
    seed(root, |id| {
        let mut fm =
            Frontmatter::new(id, number, NodeType::Slice, name, day(), day(), Origin::Planned);
        edit(&mut fm);
        Document::new(fm, "body\n")
    });
}

// ----- L-1: missing required field ------------------------------------------

#[test]
fn check_missing_field_is_flagged() {
    let dir = TempDir::new().unwrap();
    seed_slice(dir.path(), 1, "   ", |_| {}); // whitespace-only name
    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1));
    assert!(r.out.contains("missing-field"), "out: {}", r.out);
}

// ----- L-2: dangling part_of ------------------------------------------------

#[test]
fn check_dangling_part_of_is_flagged() {
    let dir = TempDir::new().unwrap();
    seed_slice(dir.path(), 1, "Orphan", |fm| {
        fm.edges_mut().part_of = Some(Id::from_str(MISSING_ID).unwrap());
    });
    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1));
    assert!(r.out.contains("dangling-part_of"), "out: {}", r.out);
}

// ----- L-3: dangling edge ---------------------------------------------------

#[test]
fn check_dangling_edge_is_flagged() {
    let dir = TempDir::new().unwrap();
    seed_slice(dir.path(), 1, "Node", |fm| {
        fm.edges_mut().supersedes = Some(Supersedes {
            node: Id::from_str(MISSING_ID).unwrap(),
            kind: SupersedeKind::Obsoletes,
        });
    });
    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1));
    assert!(r.out.contains("dangling-edge"), "out: {}", r.out);
}

// ----- L-4: supersession-chain integrity ------------------------------------

#[test]
fn check_supersession_chain_is_flagged() {
    let dir = TempDir::new().unwrap();
    // Self-supersede: edges.supersedes points at the node's own id.
    seed(dir.path(), |id| {
        let mut fm = Frontmatter::new(id, 1, NodeType::Odd, "Loop", day(), day(), Origin::Planned);
        fm.edges_mut().supersedes = Some(Supersedes { node: id, kind: SupersedeKind::Updates });
        Document::new(fm, "body\n")
    });
    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1));
    assert!(r.out.contains("self-supersede"), "out: {}", r.out);
}

// ----- L-5: clean corpus passes with exit 0 ---------------------------------

#[test]
fn check_clean_passes() {
    let dir = TempDir::new().unwrap();
    // A total tree: project <- arc <- slice (every non-root resolves to a
    // parent). v2 recomposition is stricter than v1 — a top-level arc with no
    // project parent is now an orphan, so the corpus must be a real tree.
    let mut project_id = Id::new();
    seed(dir.path(), |id| {
        project_id = id;
        let fm = Frontmatter::new(id, 1, NodeType::Project, "Odm", day(), day(), Origin::Planned);
        Document::new(fm, "body\n")
    });
    let mut arc_id = Id::new();
    seed(dir.path(), |id| {
        arc_id = id;
        let mut fm = Frontmatter::new(id, 2, NodeType::Arc, "Arc", day(), day(), Origin::Planned);
        fm.edges_mut().part_of = Some(project_id);
        Document::new(fm, "body\n")
    });
    seed_slice(dir.path(), 3, "Child", |fm| {
        fm.edges_mut().part_of = Some(arc_id);
    });
    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(0));
    assert!(r.out.contains("check: ok"), "out: {}", r.out);
}

// ----- L-6: exit codes 0 / 1 / 2 --------------------------------------------

#[test]
fn check_exit_codes_v1() {
    let dir = TempDir::new().unwrap();
    // 0: clean (empty corpus is clean).
    assert_eq!(run(dir.path(), &["check"]).code, Some(0));
    // 1: violations.
    seed_slice(dir.path(), 1, "", |_| {});
    assert_eq!(run(dir.path(), &["check"]).code, Some(1));
    // 2: a usage error is clap's domain — it rejects unknown flags before
    // dispatch is ever reached (the binary then exits 2).
    assert!(Cli::try_parse_from(["odm", "check", "--bogus"]).is_err());
}

// ----- L-7: errors-as-affordances -------------------------------------------

#[test]
fn check_errors_name_fix_v1() {
    let dir = TempDir::new().unwrap();
    seed_slice(dir.path(), 1, "", |_| {}); // empty name → fixable with `odm rename`
    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1));
    assert!(r.out.contains("fix:"), "every finding names a fix; out: {}", r.out);
    assert!(r.out.contains("odm rename"), "empty-name fix is a real command; out: {}", r.out);
}

// ----- L-8: --json report stable schema (v2 envelope) -----------------------

#[test]
fn check_json_v1() {
    let dir = TempDir::new().unwrap();
    seed_slice(dir.path(), 1, "Detached", |fm| {
        fm.edges_mut().part_of = Some(Id::from_str(MISSING_ID).unwrap());
    });
    let r = run(dir.path(), &["check", "--json"]);
    assert_eq!(r.code, Some(1));
    let value: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");
    assert_eq!(value["ok"], false);
    assert!(value["errors"].as_u64().unwrap() >= 1);
    let findings = value["findings"].as_array().expect("findings array");
    // Each finding carries the stable v2 keys.
    let f = &findings[0];
    let mut keys: Vec<&String> = f.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(keys, ["code", "detail", "fix", "name", "node", "number", "severity"]);
    // The dangling-part_of violation is present (a v1 link-integrity finding).
    assert!(findings.iter().any(|f| f["code"] == "dangling-part_of" && f["severity"] == "error"));

    // A clean corpus reports ok=true, empty findings.
    let clean = TempDir::new().unwrap();
    let cr = run(clean.path(), &["check", "--json"]);
    assert_eq!(cr.code, Some(0));
    let cv: serde_json::Value = serde_json::from_str(&cr.out).unwrap();
    assert_eq!(cv["ok"], true);
    assert_eq!(cv["errors"], 0);
    assert_eq!(cv["warnings"], 0);
    assert!(cv["findings"].as_array().unwrap().is_empty());
}

// ===========================================================================
// derived order: next / blocked / path --json (H-12)
// ===========================================================================

use odm_core::frontmatter::Dependency;
use odm_core::gates::GateSets;
use odm_core::status::Evidence;

const DERIVED_TOML: &str = "\
[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]

[satisfaction]
threshold = \"reproduced\"
";

/// Seeds A (#1) depends_on B (#2); B has reached its terminal gate but only at
/// `attested` (below the `reproduced` threshold ⇒ soft-satisfied).
fn seed_derived(root: &Path) {
    std::fs::write(root.join("odm.toml"), DERIVED_TOML).unwrap();
    let gates = GateSets::from_toml_str(DERIVED_TOML).unwrap();
    let gset = gates.for_type(NodeType::Slice).unwrap().clone();

    let mut b_id = Id::new();
    seed(root, |id| {
        b_id = id;
        let mut fm = Frontmatter::new(id, 2, NodeType::Slice, "B", day(), day(), Origin::Planned);
        fm.status_mut().set_gate(&gset, "tested", None, Evidence::Attested, day()).unwrap();
        Document::new(fm, "body\n")
    });
    seed(root, |id| {
        let mut fm = Frontmatter::new(id, 1, NodeType::Slice, "A", day(), day(), Origin::Planned);
        fm.edges_mut().depends_on = vec![Dependency::Bare(b_id)];
        Document::new(fm, "body\n")
    });
}

#[test]
fn json_schema_derived_order() {
    let dir = TempDir::new().unwrap();
    seed_derived(dir.path());

    // next --json: A is ready (B complete), soft-flagged at attested.
    let next = run(dir.path(), &["next", "--json"]);
    assert_eq!(next.code, Some(0));
    let value: serde_json::Value = serde_json::from_str(&next.out).expect("valid JSON");
    let arr = value.as_array().expect("array");
    let a = arr.iter().find(|n| n["number"] == 1).expect("A in next");
    let mut keys: Vec<&String> = a.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(keys, ["effective_evidence", "node", "number", "soft"]);
    assert_eq!(a["effective_evidence"], "attested"); // carries the evidence level
    assert_eq!(a["soft"][0]["evidence"], "attested");
    assert!(!arr.iter().any(|n| n["number"] == 2), "B is complete, not in next");

    // blocked 1 --json: the low-evidence dependency, with threshold.
    let blocked = run(dir.path(), &["blocked", "1", "--json"]);
    assert_eq!(blocked.code, Some(0));
    let value: serde_json::Value = serde_json::from_str(&blocked.out).unwrap();
    let reason = &value.as_array().unwrap()[0];
    assert_eq!(reason["kind"], "soft-satisfied");
    assert_eq!(reason["evidence"], "attested");
    assert_eq!(reason["threshold"], "reproduced");

    // path 1 2 --json: the dependency path A -> B.
    let path = run(dir.path(), &["path", "1", "2", "--json"]);
    assert_eq!(path.code, Some(0));
    let value: serde_json::Value = serde_json::from_str(&path.out).unwrap();
    assert_eq!(value["path"].as_array().unwrap().len(), 2);

    // Human output also surfaces the soft flag.
    let next_human = run(dir.path(), &["next"]);
    assert!(next_human.out.contains("⚠ dep") && next_human.out.contains("evidence=attested"));
}

// ===========================================================================
// check v2: aggregate every graph-level invariant (M-1 .. M-9)
// ===========================================================================

const V2_TOML: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\"]

[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\"]

[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]

[satisfaction]
threshold = \"reproduced\"
";

fn fmn(id: Id, number: u32, ty: NodeType, name: &str) -> Frontmatter {
    Frontmatter::new(id, number, ty, name, day(), day(), Origin::Planned)
}

/// Persists a fully-built frontmatter (controlled id) to the store.
fn put(root: &Path, fm: Frontmatter) {
    Store::open(root).persist(&Document::new(fm, "body\n")).unwrap();
}

/// The slice gate-set from [`V2_TOML`], for stamping status in tests.
fn slice_gset() -> odm_core::gates::GateSet {
    GateSets::from_toml_str(V2_TOML).unwrap().for_type(NodeType::Slice).unwrap().clone()
}

/// Seeds a minimal total tree project(1) <- arc(2) and returns (project, arc).
fn tree(root: &Path) -> (Id, Id) {
    let p = Id::new();
    let a = Id::new();
    put(root, fmn(p, 1, NodeType::Project, "Project"));
    let mut arc = fmn(a, 2, NodeType::Arc, "Arc");
    arc.edges_mut().part_of = Some(p);
    put(root, arc);
    (p, a)
}

// ----- M-1: aggregate schema + link-integrity (v1) --------------------------

#[test]
fn check_schema_and_links() {
    let dir = TempDir::new().unwrap();
    let (_p, a) = tree(dir.path());
    // A blank-name slice (missing-field) and a slice with a dangling depends_on.
    let mut blank = fmn(Id::new(), 3, NodeType::Slice, "   ");
    blank.edges_mut().part_of = Some(a);
    put(dir.path(), blank);
    let mut dangling = fmn(Id::new(), 4, NodeType::Slice, "Dep");
    dangling.edges_mut().part_of = Some(a);
    dangling.edges_mut().depends_on = vec![Dependency::Bare(Id::from_str(MISSING_ID).unwrap())];
    put(dir.path(), dangling);

    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1));
    assert!(r.out.contains("missing-field"), "schema check aggregated; out: {}", r.out);
    assert!(r.out.contains("dangling-edge"), "link-integrity aggregated; out: {}", r.out);
}

// ----- M-2: cycle-without-tear fails; passes once torn ----------------------

#[test]
fn check_cycle_requires_tear() {
    let dir = TempDir::new().unwrap();
    let (p, a) = tree(dir.path());
    let x = Id::new();
    let y = Id::new();
    let slice_with = |id, num, name, dep, tear: Option<Id>| {
        let mut fm = fmn(id, num, NodeType::Slice, name);
        fm.edges_mut().part_of = Some(a);
        fm.edges_mut().depends_on = vec![Dependency::Bare(dep)];
        if let Some(t) = tear {
            fm.edges_mut().tears = vec![Dependency::Bare(t)];
        }
        fm
    };
    let _ = p;
    // X depends_on Y and Y depends_on X — an ordering cycle.
    put(dir.path(), slice_with(x, 3, "X", y, None));
    put(dir.path(), slice_with(y, 4, "Y", x, None));

    let cyclic = run(dir.path(), &["check"]);
    assert_eq!(cyclic.code, Some(1));
    assert!(cyclic.out.contains("cycle"), "cycle reported; out: {}", cyclic.out);

    // Tear X -> Y: the cycle is broken, the corpus passes.
    put(dir.path(), slice_with(x, 3, "X", y, Some(y)));
    let torn = run(dir.path(), &["check"]);
    assert_eq!(torn.code, Some(0), "torn cycle passes; out: {}", torn.out);
}

// ----- M-3: out-of-order / staleness ----------------------------------------

#[test]
fn check_staleness() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("odm.toml"), V2_TOML).unwrap();
    let (_p, a) = tree(dir.path());
    let y = Id::new();
    let x = Id::new();
    // Y: an unsatisfied dependency (no terminal gate reached).
    let mut y_fm = fmn(y, 4, NodeType::Slice, "Y");
    y_fm.edges_mut().part_of = Some(a);
    put(dir.path(), y_fm);
    // X: advanced to `built` while depending on the unsatisfied Y → stale.
    let mut x_fm = fmn(x, 3, NodeType::Slice, "X");
    x_fm.edges_mut().part_of = Some(a);
    x_fm.edges_mut().depends_on = vec![Dependency::Bare(y)];
    x_fm.status_mut().set_gate(&slice_gset(), "built", None, Evidence::Reproduced, day()).unwrap();
    put(dir.path(), x_fm);

    let r = run(dir.path(), &["check"]);
    // Warning-only ⇒ exit 0 in normal mode, but the staleness is reported.
    assert_eq!(r.code, Some(0), "out: {}", r.out);
    assert!(r.out.contains("staleness"), "out: {}", r.out);
}

// ----- M-4: recomposition (orphan/stub/decomposition drift) -----------------

#[test]
fn check_recomposition() {
    let dir = TempDir::new().unwrap();
    // A slice with no containment parent: an orphan (recomposition not total).
    put(dir.path(), fmn(Id::new(), 1, NodeType::Slice, "Lonely"));

    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1));
    assert!(r.out.contains("orphan"), "recomposition aggregated; out: {}", r.out);
}

// ----- M-4b: the other recomposition variants (stub/drift/advance-without) --

#[test]
fn check_recomposition_variants() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("odm.toml"), V2_TOML).unwrap();
    let arc_gset =
        GateSets::from_toml_str(V2_TOML).unwrap().for_type(NodeType::Arc).unwrap().clone();

    let p = Id::new();
    put(dir.path(), fmn(p, 1, NodeType::Project, "P"));

    // Q: an arc advanced to `in-progress` with zero children → undeveloped stub.
    let q = Id::new();
    let mut q_fm = fmn(q, 2, NodeType::Arc, "Q");
    q_fm.edges_mut().part_of = Some(p);
    q_fm.status_mut()
        .set_gate(&arc_gset, "in-progress", None, Evidence::Reproduced, day())
        .unwrap();
    put(dir.path(), q_fm);

    // R: an arc that reached terminal `complete` with a child but never affirmed
    // `decomposed` → advanced-without-decomposition.
    let r = Id::new();
    let mut r_fm = fmn(r, 3, NodeType::Arc, "R");
    r_fm.edges_mut().part_of = Some(p);
    r_fm.status_mut().set_gate(&arc_gset, "complete", None, Evidence::Reproduced, day()).unwrap();
    put(dir.path(), r_fm);
    let s = Id::new();
    let mut s_fm = fmn(s, 4, NodeType::Slice, "S");
    s_fm.edges_mut().part_of = Some(r);
    put(dir.path(), s_fm);

    // T: an arc that affirmed `decomposed` against a child that is not present,
    // while a different child is → decomposition drift (added U, removed Z).
    let t = Id::new();
    let mut t_fm = fmn(t, 5, NodeType::Arc, "T");
    t_fm.edges_mut().part_of = Some(p);
    t_fm.affirm_decomposed(vec![Id::from_str(MISSING_ID).unwrap()], day());
    put(dir.path(), t_fm);
    let u = Id::new();
    let mut u_fm = fmn(u, 6, NodeType::Slice, "U");
    u_fm.edges_mut().part_of = Some(t);
    put(dir.path(), u_fm);

    let out = run(dir.path(), &["check"]);
    assert_eq!(out.code, Some(1));
    assert!(out.out.contains("undeveloped-stub"), "out: {}", out.out);
    assert!(out.out.contains("advanced-without-decomposition"), "out: {}", out.out);
    assert!(out.out.contains("decomposition-drift"), "out: {}", out.out);
}

// ----- M-5: below-threshold (soft-satisfied) dependencies -------------------

/// Seeds a total tree with A (#4) depends_on B (#3), where B reached its
/// terminal gate only at `attested` (below the `reproduced` threshold).
fn seed_soft_tree(root: &Path) -> (Id, Id) {
    std::fs::write(root.join("odm.toml"), V2_TOML).unwrap();
    let (_p, a) = tree(root);
    let b = Id::new();
    let mut b_fm = fmn(b, 3, NodeType::Slice, "B");
    b_fm.edges_mut().part_of = Some(a);
    b_fm.status_mut().set_gate(&slice_gset(), "tested", None, Evidence::Attested, day()).unwrap();
    put(root, b_fm);
    let aa = Id::new();
    let mut a_fm = fmn(aa, 4, NodeType::Slice, "A");
    a_fm.edges_mut().part_of = Some(a);
    a_fm.edges_mut().depends_on = vec![Dependency::Bare(b)];
    put(root, a_fm);
    (aa, b)
}

#[test]
fn check_soft_satisfied() {
    let dir = TempDir::new().unwrap();
    seed_soft_tree(dir.path());

    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(0), "warning-only ⇒ passes in normal mode; out: {}", r.out);
    assert!(r.out.contains("soft-satisfied"), "out: {}", r.out);
}

// ----- M-6: exit codes 0 / 1 / 2 --------------------------------------------

#[test]
fn check_exit_codes() {
    let dir = TempDir::new().unwrap();
    // 0: a clean total tree.
    tree(dir.path());
    assert_eq!(run(dir.path(), &["check"]).code, Some(0));

    // 1: an error (an orphan slice).
    let bad = TempDir::new().unwrap();
    put(bad.path(), fmn(Id::new(), 1, NodeType::Slice, "Orphan"));
    assert_eq!(run(bad.path(), &["check"]).code, Some(1));

    // 2: a usage error is clap's domain (rejected before dispatch; the binary
    // then exits 2). Verified in-process: odm-cli is library-only, so there is
    // no binary to spawn — `dispatch` returns the exact code `run` maps to
    // `ExitCode`. (Deviation from the cc-prompt's `assert_cmd` note — flagged.)
    assert!(Cli::try_parse_from(["odm", "check", "--bogus"]).is_err());
}

// ----- M-7: --strict promotes warnings to failures --------------------------

#[test]
fn check_strict_mode() {
    let dir = TempDir::new().unwrap();
    seed_soft_tree(dir.path()); // warning-only corpus (soft-satisfied)

    // Normal mode: warnings do not fail.
    assert_eq!(run(dir.path(), &["check"]).code, Some(0));
    // Strict mode: the same warning fails the run.
    let strict = run(dir.path(), &["check", "--strict"]);
    assert_eq!(strict.code, Some(1), "out: {}", strict.out);
    assert!(strict.out.contains("soft-satisfied"));
}

// ----- M-8: every finding names the exact fix -------------------------------

#[test]
fn check_errors_name_fix() {
    let dir = TempDir::new().unwrap();
    // A mix: an orphan (error) and a soft-satisfied dep (warning).
    seed_soft_tree(dir.path());
    put(dir.path(), fmn(Id::new(), 9, NodeType::Slice, "Stray")); // orphan

    let r = run(dir.path(), &["check", "--json"]);
    assert_eq!(r.code, Some(1));
    let value: serde_json::Value = serde_json::from_str(&r.out).unwrap();
    let findings = value["findings"].as_array().unwrap();
    assert!(findings.len() >= 2);
    for f in findings {
        let fix = f["fix"].as_str().expect("fix string");
        assert!(!fix.trim().is_empty(), "every finding names a fix: {f}");
    }
    // Human output prints a `fix:` line per finding too.
    assert!(run(dir.path(), &["check"]).out.contains("fix:"));
}

// ----- M-9: --json report, stable schema ------------------------------------

#[test]
fn check_json_schema() {
    let dir = TempDir::new().unwrap();
    seed_soft_tree(dir.path()); // a warning
    put(dir.path(), fmn(Id::new(), 9, NodeType::Slice, "Stray")); // an error (orphan)

    let r = run(dir.path(), &["check", "--json"]);
    assert_eq!(r.code, Some(1));
    let value: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");

    // Stable envelope.
    let mut top: Vec<&String> = value.as_object().unwrap().keys().collect();
    top.sort();
    assert_eq!(top, ["errors", "findings", "ok", "warnings"]);
    assert_eq!(value["ok"], false);
    assert!(value["errors"].as_u64().unwrap() >= 1);
    assert!(value["warnings"].as_u64().unwrap() >= 1);

    // Stable per-finding schema.
    let findings = value["findings"].as_array().unwrap();
    for f in findings {
        let mut keys: Vec<&String> = f.as_object().unwrap().keys().collect();
        keys.sort();
        assert_eq!(keys, ["code", "detail", "fix", "name", "node", "number", "severity"]);
        assert!(f["severity"] == "error" || f["severity"] == "warning");
    }
    // Both families are present and labeled by severity.
    assert!(findings.iter().any(|f| f["code"] == "orphan" && f["severity"] == "error"));
    assert!(findings.iter().any(|f| f["code"] == "soft-satisfied" && f["severity"] == "warning"));
}

// ===========================================================================
// slice07: CLI graph mutators — link / unlink / set-gate / tear (M-1 .. M-13)
// ===========================================================================

/// The on-disk content of the node with the given human `number`.
fn file_for(root: &Path, number: u32) -> String {
    for p in md_paths(root) {
        let c = std::fs::read_to_string(&p).unwrap();
        if c.contains(&format!("number: {number}\n")) {
            return c;
        }
    }
    panic!("no node #{number} on disk");
}

/// The ULID of the node with the given human `number` (via `show --json`).
fn id_of(root: &Path, number: u32) -> String {
    let r = run(root, &["show", &number.to_string(), "--json"]);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("show --json");
    v["id"].as_str().unwrap().to_string()
}

// ----- M-1: link adds the edge on the source; reverse is derived ------------

#[test]
fn link_adds_edge() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "A"]);
    run(dir.path(), &["new", "slice", "B"]);

    let r = run(dir.path(), &["link", "1", "depends_on", "2"]);
    assert!(r.ok && r.err.contains("linked"), "err: {}", r.err);

    // The edge is written on the source A (#1)...
    let a = file_for(dir.path(), 1);
    assert!(a.contains("depends_on") && a.contains(&id_of(dir.path(), 2)));
    // ...and NOT mirrored onto the target B (#2): reverse edges stay derived.
    let b = file_for(dir.path(), 2);
    assert!(!b.contains("depends_on"), "reverse edge must not be written; B: {b}");
}

// ----- M-2: link covers every source-stored edge kind -----------------------

#[test]
fn link_edge_kinds() {
    let dir = TempDir::new().unwrap();
    for (n, ty, name) in [
        (1, "slice", "Src"),
        (2, "slice", "Dep"),
        (3, "slice", "Block"),
        (4, "slice", "Out"),
        (5, "odd", "Doc"),
        (6, "odd", "Affected"),
        (7, "arc", "Parent"),
    ] {
        let _ = n;
        run(dir.path(), &["new", ty, name]);
    }

    assert!(run(dir.path(), &["link", "1", "depends_on", "2", "--satisfied-at", "tested"]).ok);
    assert!(run(dir.path(), &["link", "1", "blocked_by", "3"]).ok);
    assert!(run(dir.path(), &["link", "1", "consumes", "4"]).ok);
    assert!(run(dir.path(), &["link", "1", "verifies", "5"]).ok);
    assert!(run(dir.path(), &["link", "1", "affects", "6"]).ok);
    assert!(run(dir.path(), &["link", "1", "part_of", "7"]).ok);

    let src = file_for(dir.path(), 1);
    for field in ["depends_on", "blocked_by", "consumes", "verifies", "affects", "part_of"] {
        assert!(src.contains(field), "missing `{field}` in source; file:\n{src}");
    }
    // `--satisfied-at` is recorded as a qualified dependency.
    assert!(src.contains("satisfied_at") && src.contains("tested"));
}

// ----- M-3: part_of is single-parent (replace, not append) ------------------

#[test]
fn link_part_of_single_parent() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "Child"]);
    run(dir.path(), &["new", "arc", "P1"]);
    run(dir.path(), &["new", "arc", "P2"]);

    assert!(run(dir.path(), &["link", "1", "part_of", "2"]).ok);
    assert!(run(dir.path(), &["link", "1", "part_of", "3"]).ok); // replaces

    // show --json exposes a single parent; it is P2 (#3), not P1 (#2).
    let r = run(dir.path(), &["show", "1", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&r.out).unwrap();
    assert_eq!(v["part_of"], id_of(dir.path(), 3));
    // The old parent's id is gone from the file (replaced, not appended).
    assert!(!file_for(dir.path(), 1).contains(&id_of(dir.path(), 2)));
}

// ----- M-4: unlink removes the edge; absent edge is a clear no-op ------------

#[test]
fn unlink_removes_edge() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "A"]);
    run(dir.path(), &["new", "slice", "B"]);
    run(dir.path(), &["link", "1", "depends_on", "2"]);

    let r = run(dir.path(), &["unlink", "1", "depends_on", "2"]);
    assert!(r.ok && r.err.contains("unlinked"));
    assert!(!file_for(dir.path(), 1).contains("depends_on"));

    // Unlinking again is a clear no-op (not an error).
    let again = run(dir.path(), &["unlink", "1", "depends_on", "2"]);
    assert!(again.ok && again.err.contains("no-op"), "err: {}", again.err);
}

// ----- M-5: endpoint resolution by id | number | name-prefix ----------------

#[test]
fn mutator_ref_resolution() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "Alpha"]);
    run(dir.path(), &["new", "slice", "Beta"]);

    // By unique name-prefix, by number, and by full id — all resolve.
    assert!(run(dir.path(), &["link", "Alph", "depends_on", "Bet"]).ok);
    assert!(run(dir.path(), &["unlink", "1", "depends_on", "2"]).ok);
    let id2 = id_of(dir.path(), 2);
    assert!(run(dir.path(), &["link", &id_of(dir.path(), 1), "depends_on", &id2]).ok);

    // An unresolvable endpoint fails with an affordance.
    let bad = run(dir.path(), &["link", "1", "depends_on", "99"]);
    assert!(!bad.ok && bad.err.contains("no node with number 99") && bad.err.contains("odm list"));
}

// ----- M-6: set-gate via Status::set_gate -----------------------------------

#[test]
fn set_gate_cli() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("odm.toml"), V2_TOML).unwrap();
    run(dir.path(), &["new", "slice", "A"]);

    // Default evidence is `asserted`; the first-reach date is recorded.
    let r = run(dir.path(), &["set-gate", "1", "built"]);
    assert!(r.ok && r.err.contains("set gate \"built\"=asserted"), "err: {}", r.err);
    let f = file_for(dir.path(), 1);
    assert!(f.contains("built:") && f.contains("evidence: asserted"));
    assert!(f.contains("evidence_dates"), "slice05.1 first-reach recorded; file:\n{f}");

    // An out-of-set gate is rejected with an affordance.
    let bad = run(dir.path(), &["set-gate", "1", "deployed"]);
    assert!(!bad.ok && bad.err.contains("unknown gate"));
    assert!(bad.err.contains("allowed") && bad.err.contains("odm set-gate"));

    // Explicit evidence + actor are recorded.
    assert!(
        run(dir.path(), &["set-gate", "1", "tested", "--evidence", "reproduced", "--by", "ci"]).ok
    );
    let f = file_for(dir.path(), 1);
    assert!(f.contains("evidence: reproduced") && f.contains("ci"));
}

// ----- M-7: tear via Tear::new ----------------------------------------------

#[test]
fn tear_cli() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "slice", "A"]);
    run(dir.path(), &["new", "slice", "B"]);

    let r = run(dir.path(), &["tear", "1", "depends_on", "2", "--because", "assumed for now"]);
    assert!(r.ok && r.err.contains("tore"), "err: {}", r.err);
    assert!(file_for(dir.path(), 1).contains("tears:"));

    // An empty rationale is rejected with an affordance.
    let bad = run(dir.path(), &["tear", "1", "depends_on", "2", "--because", "   "]);
    assert!(!bad.ok && bad.err.contains("needs a rationale") && bad.err.contains("--because"));
}

// ----- M-8: new --parent sets part_of ---------------------------------------

#[test]
fn new_with_parent() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["new", "project", "Odm"]);
    let r = run(dir.path(), &["new", "arc", "Substrate", "--parent", "1"]);
    assert!(r.ok && r.err.contains("part_of"), "err: {}", r.err);

    let shown = run(dir.path(), &["show", "2", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&shown.out).unwrap();
    assert_eq!(v["part_of"], id_of(dir.path(), 1));
}

// ----- M-8b: decomposed affirms (wraps affirm_decomposed) -------------------

#[test]
fn decomposed_cli() {
    let dir = TempDir::new().unwrap();
    // project P(1) <- arc Q(2) <- slice S(3).
    run(dir.path(), &["new", "project", "P"]);
    run(dir.path(), &["new", "arc", "Q", "--parent", "1"]);
    run(dir.path(), &["new", "slice", "S", "--parent", "2"]);

    // Affirm Q's decomposition against an explicit child set; records + persists.
    let r = run(dir.path(), &["decomposed", "2", "--children", "3"]);
    assert!(r.ok && r.err.contains("affirmed decomposition"), "err: {}", r.err);
    let q = file_for(dir.path(), 2);
    assert!(q.contains("decomposed:") && q.contains(&id_of(dir.path(), 3)));

    // With no --children, it affirms against the current containment children.
    run(dir.path(), &["new", "slice", "S2", "--parent", "2"]); // #4, a second child
    let r2 = run(dir.path(), &["decomposed", "2"]);
    assert!(r2.ok && r2.err.contains("2 child(ren)"), "err: {}", r2.err);

    // --dry-run announces but writes nothing new.
    assert!(run(dir.path(), &["decomposed", "2", "--dry-run"]).err.contains("would affirm"));

    // A non-parent-capable node (a slice) cannot be decomposed.
    let bad = run(dir.path(), &["decomposed", "3"]);
    assert!(!bad.ok && bad.err.contains("only a project or arc"));
}

// ----- M-9: --dry-run writes nothing; --yes runs ----------------------------

#[test]
fn mutators_dry_run_and_yes() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("odm.toml"), V2_TOML).unwrap();
    run(dir.path(), &["new", "slice", "A"]);
    run(dir.path(), &["new", "slice", "B"]);

    // link --dry-run: announced, but nothing written.
    let dry = run(dir.path(), &["link", "1", "depends_on", "2", "--dry-run"]);
    assert!(dry.ok && dry.err.contains("would link"));
    assert!(!file_for(dir.path(), 1).contains("depends_on"));

    // --yes runs (non-interactive); the edge is written.
    assert!(run(dir.path(), &["link", "1", "depends_on", "2", "--yes"]).ok);
    assert!(file_for(dir.path(), 1).contains("depends_on"));

    // set-gate / tear --dry-run write nothing either.
    assert!(run(dir.path(), &["set-gate", "1", "built", "--dry-run"]).ok);
    assert!(!file_for(dir.path(), 1).contains("built:"));
    assert!(run(dir.path(), &["tear", "1", "depends_on", "2", "--because", "x", "--dry-run"]).ok);
    assert!(!file_for(dir.path(), 1).contains("tears:"));
}

// ----- M-10: mutations persist atomically and round-trip on reload ----------

#[test]
fn mutation_roundtrip() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("odm.toml"), V2_TOML).unwrap();
    run(dir.path(), &["new", "arc", "Parent"]);
    run(dir.path(), &["new", "slice", "Leaf"]);

    run(dir.path(), &["link", "2", "part_of", "1"]);
    run(dir.path(), &["link", "2", "depends_on", "1", "--satisfied-at", "complete"]);
    run(dir.path(), &["set-gate", "2", "built", "--evidence", "reproduced"]);

    // A fresh dispatch reloads from disk: every mutation survives the round-trip
    // and the file still parses (queries succeed, `check` does not choke).
    let shown = run(dir.path(), &["show", "2", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&shown.out).unwrap();
    assert_eq!(v["part_of"], id_of(dir.path(), 1));
    let f = file_for(dir.path(), 2);
    assert!(f.contains("satisfied_at: complete") && f.contains("built:"));
    assert!(run(dir.path(), &["list"]).ok); // reload parses cleanly
}

// ----- M-11: a CLI-built graph answers next/blocked (self-host smoke) -------

#[test]
fn cli_built_graph_queries() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("odm.toml"), V2_TOML).unwrap();

    // Build the graph purely through the CLI: A depends_on B.
    run(dir.path(), &["new", "slice", "A"]);
    run(dir.path(), &["new", "slice", "B"]);
    assert!(run(dir.path(), &["link", "1", "depends_on", "2"]).ok);

    // B is ready (no deps); A is blocked on the unsatisfied B.
    let next1 = run(dir.path(), &["next"]);
    assert!(next1.out.contains('B') && !next1.out.contains("A\n"), "next: {}", next1.out);
    let blocked1 = run(dir.path(), &["blocked", "1"]);
    assert!(blocked1.out.contains("unsatisfied dependency"), "blocked: {}", blocked1.out);

    // Satisfy B by recording its terminal gate at the threshold evidence.
    assert!(run(dir.path(), &["set-gate", "2", "tested", "--evidence", "reproduced"]).ok);

    // Now B is complete (out of `next`) and A is ready (its dep is satisfied).
    let next2 = run(dir.path(), &["next"]);
    assert!(next2.out.contains("A") && !next2.out.contains('B'), "next: {}", next2.out);
    let blocked2 = run(dir.path(), &["blocked", "1"]);
    assert!(blocked2.out.contains("nothing holding"), "blocked: {}", blocked2.out);
}

// ----- M-12: every mutator failure names the exact fix ----------------------

#[test]
fn mutator_errors_name_fix() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("odm.toml"), V2_TOML).unwrap();
    run(dir.path(), &["new", "slice", "A"]);

    // Unresolvable endpoint → names `odm list`.
    let r1 = run(dir.path(), &["link", "1", "depends_on", "404"]);
    assert!(!r1.ok && r1.err.contains("odm list"));

    // Out-of-set gate → names `odm set-gate`.
    let r2 = run(dir.path(), &["set-gate", "1", "shipped"]);
    assert!(!r2.ok && r2.err.contains("odm set-gate"));

    // Empty tear rationale → names `odm tear ... --because`.
    run(dir.path(), &["new", "slice", "B"]);
    let r3 = run(dir.path(), &["tear", "1", "depends_on", "2", "--because", ""]);
    assert!(!r3.ok && r3.err.contains("--because"));
}

// ----- mutator edge cases (error/dry-run branches) --------------------------

#[test]
fn mutator_edge_cases() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("odm.toml"), V2_TOML).unwrap();
    run(dir.path(), &["new", "slice", "A"]);
    run(dir.path(), &["new", "slice", "B"]);

    // Self-link is rejected (both the generic and the `part_of` phrasings).
    assert!(
        run(dir.path(), &["link", "1", "depends_on", "1"]).err.contains("cannot link to itself")
    );
    assert!(run(dir.path(), &["link", "1", "part_of", "1"]).err.contains("be `part_of` itself"));

    // `--satisfied-at` only applies to `depends_on`.
    let bad = run(dir.path(), &["link", "1", "blocked_by", "2", "--satisfied-at", "tested"]);
    assert!(!bad.ok && bad.err.contains("--satisfied-at"));

    // unlink --dry-run announces but writes nothing.
    run(dir.path(), &["link", "1", "depends_on", "2"]);
    assert!(
        run(dir.path(), &["unlink", "1", "depends_on", "2", "--dry-run"])
            .err
            .contains("would unlink")
    );
    assert!(file_for(dir.path(), 1).contains("depends_on"), "dry-run unlink must not remove");

    // set-gate / tear dry-run messages.
    assert!(
        run(dir.path(), &["set-gate", "1", "built", "--dry-run"]).err.contains("would set gate")
    );
    assert!(
        run(dir.path(), &["tear", "1", "depends_on", "2", "--because", "x", "--dry-run"])
            .err
            .contains("would tear")
    );

    // new --parent --dry-run notes the parent and writes nothing.
    let np = run(dir.path(), &["new", "slice", "Z", "--parent", "2", "--dry-run"]);
    assert!(np.err.contains("would create") && np.err.contains("part_of"));

    // set-gate on a type with no configured gate-set → affordance.
    run(dir.path(), &["new", "note", "N"]);
    let ng = run(dir.path(), &["set-gate", "3", "anything"]);
    assert!(!ng.ok && ng.err.contains("no gate-set"));

    // new --parent with an unresolvable parent fails before writing.
    assert!(!run(dir.path(), &["new", "slice", "Orphan", "--parent", "404"]).ok);
}
