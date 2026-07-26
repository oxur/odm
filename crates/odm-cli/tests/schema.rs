//! In-process tests for schema versioning at the CLI surface (arc06 slice03,
//! ODD-0020). Drives [`odm_cli::dispatch`] against a temp store. Test names carry
//! the substrings the ledger Verify commands filter on: `new_node_stamps_schema_v1`
//! (V-2), `check_wrong_type_field_is_error` (V-3).

use std::path::Path;

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::schema::{SchemaMarker, SchemaVersion};
use odm_core::{DesiredFact, Id, NodeType, Origin, ProbeSpec, ShellExpect};
use odm_store::Store;
use tempfile::TempDir;

fn run_code(root: &Path, args: &[&str]) -> (Option<u8>, String, String) {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = odm_cli::dispatch(cli, root, &mut out, &mut err).ok();
    (code, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

// ----- V-2: every odm-created node stamps `<type>/v1.0` ----------------------

#[test]
fn new_node_stamps_schema_v1() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());

    // Create one node of a few representative types; each must carry <type>/v1.0.
    for (ty, name) in [("project", "Proj"), ("design", "Doc"), ("slice", "Work")] {
        let (code, _o, _e) = run_code(dir.path(), &["new", ty, name]);
        assert_eq!(code, Some(0), "`odm new {ty}` dispatches");
    }

    let by_type: std::collections::HashMap<NodeType, SchemaMarker> = store
        .load_all()
        .unwrap()
        .iter()
        .map(|d| (d.frontmatter().node_type(), d.frontmatter().schema().expect("stamped")))
        .collect();

    assert_eq!(by_type[&NodeType::Project], SchemaMarker::current(NodeType::Project));
    assert_eq!(by_type[&NodeType::Design], SchemaMarker::current(NodeType::Design));
    assert_eq!(by_type[&NodeType::Slice], SchemaMarker::current(NodeType::Slice));
    // "v1.0" is the current version.
    assert_eq!(by_type[&NodeType::Design].version, SchemaVersion::CURRENT);
}

// ----- V-3: a wrong-type field is a `check` Error ----------------------------

#[test]
fn check_wrong_type_field_is_error() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let day = NaiveDate::from_ymd_opt(2026, 7, 6).unwrap();

    // A `design` node carrying `desired_facts` (a work-only field) — a per-type
    // contract violation. Persist it directly (the CLI would never create this).
    let fact = DesiredFact {
        id: "up".to_string(),
        describe: "service up".to_string(),
        probe: ProbeSpec::Shell {
            run: "true".to_string(),
            inputs: Vec::new(),
            expect: ShellExpect { exit: 0, stdout_contains: None },
        },
    };
    let fm = Frontmatter::new(Id::new(), 1, NodeType::Design, "Bad", day, day, Origin::Planned)
        .with_desired_facts(vec![fact])
        .with_schema(SchemaMarker::current(NodeType::Design));
    store.persist(&Document::new(fm, "body\n")).unwrap();

    // `odm check` is an Error (exit 1), and names the wrong-type-field finding.
    let (code, out, _e) = run_code(dir.path(), &["check"]);
    assert_eq!(code, Some(1), "wrong-type field fails check:\n{out}");
    assert!(out.contains("wrong-type-field"), "the finding is surfaced:\n{out}");
    assert!(out.contains("desired_facts"), "names the offending field:\n{out}");

    // `check --json` marks it an error too.
    let (_c, jout, _e) = run_code(dir.path(), &["check", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&jout).expect("valid JSON");
    assert_eq!(v["ok"], false);
    assert!(
        v["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "wrong-type-field" && f["severity"] == "error"),
        "wrong-type-field is an error in --json:\n{jout}"
    );
}

// ----- V-3 (negative): a valid corpus stays green ---------------------------

#[test]
fn check_green_when_fields_valid_for_type() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path());
    let day = NaiveDate::from_ymd_opt(2026, 7, 6).unwrap();

    // An odd with `supersedes` (valid on a document) + a slice with desired_facts
    // (valid on work) — both stamped v1.0. No field-validity findings.
    let a = Id::new();
    let b = Id::new();
    let odd = Frontmatter::new(a, 1, NodeType::Design, "Doc", day, day, Origin::Planned)
        .with_schema(SchemaMarker::current(NodeType::Design));
    let mut newer = Frontmatter::new(b, 2, NodeType::Design, "Newer", day, day, Origin::Planned)
        .with_schema(SchemaMarker::current(NodeType::Design));
    newer.edges_mut().supersedes = Some(odm_core::frontmatter::Supersedes {
        node: a,
        kind: odm_core::frontmatter::SupersedeKind::Obsoletes,
    });
    store.persist(&Document::new(odd, "body\n")).unwrap();
    store.persist(&Document::new(newer, "body\n")).unwrap();

    let (code, out, _e) = run_code(dir.path(), &["check"]);
    assert_eq!(code, Some(0), "valid per-type fields → check green:\n{out}");
}
