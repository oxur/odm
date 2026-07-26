//! RH C-3 — the `odm list` overhaul, driven through `dispatch` in-process.
//!
//! One test per finding: F-4 (no NUMBER), F-5 (DATE first + `--date`), F-7
//! (STATUS after TYPE), F-8 (containment tree + the document group), F-6
//! (de-numbered names), F-9 (`--width` elision), F-15 (retired excluded by
//! default, shown under `--all`).

use std::path::Path;

use clap::Parser as _;
use odm_cli::Cli;
use tempfile::TempDir;

/// The `[gates.*]` config the seeded corpus needs for a meaningful STATUS.
const GATES: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
[gates.design]
sequence = [\"draft\", \"under-review\", \"revised\", \"accepted\", \"active\", \"final\"]
";

struct Run {
    ok: bool,
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let ok = odm_cli::dispatch(cli, root, &mut out, &mut err).is_ok();
    Run { ok, out: String::from_utf8(out).unwrap() }
}

/// A project → arc → two slices, one design doc, and one retired slice.
fn seed(root: &Path) {
    std::fs::write(root.join("odm.toml"), GATES).unwrap();
    run(root, &["new", "project", "Root project"]);
    run(root, &["new", "arc", "Arc 01 — First arc", "--parent", "1"]);
    run(root, &["new", "slice", "Slice 01 — Alpha", "--parent", "2"]);
    run(root, &["new", "slice", "Slice 02 — Beta", "--parent", "2"]);
    run(root, &["new", "design", "A design document"]);
    run(root, &["set-gate", "3", "built"]);
    // #6: retired, so it must not appear in a default listing (F-15).
    run(root, &["new", "slice", "Slice 03 — Tombstone", "--parent", "2"]);
    run(root, &["retire", "6", "--because", "created against a stale list"]);
}

/// The visible text of a run, ANSI stripped.
fn plain(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ----- F-4: the NUMBER column is gone ---------------------------------------

#[test]
fn list_has_no_number_column() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let r = run(dir.path(), &["list"]);
    assert!(r.ok);
    let out = plain(&r.out);
    assert!(!out.contains("NUMBER"), "no NUMBER column:\n{out}");
}

// ----- F-5: DATE leads, and `--date` switches which date --------------------

#[test]
fn list_leads_with_date_and_honours_the_date_flag() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list"]).out);
    let header = out.lines().nth(1).expect("a header row");
    let columns: Vec<&str> = header.split('│').map(str::trim).collect();
    assert_eq!(columns, ["DATE", "TYPE", "STATUS", "NAME", "ID"], "C-3 column order");

    // Both forms dispatch and render a table.
    for which in ["created", "updated"] {
        let r = run(dir.path(), &["list", "--date", which]);
        assert!(r.ok, "--date={which} dispatches");
        assert!(plain(&r.out).contains("DATE"), "--date={which} renders");
    }
}

// ----- F-7: STATUS is the furthest-reached gate -----------------------------

#[test]
fn list_status_shows_the_furthest_reached_gate() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list"]).out);

    // #3 reached `built` (the second of planned/built/tested).
    let alpha = out.lines().find(|l| l.contains("Alpha")).expect("Alpha row");
    assert!(alpha.contains("built"), "furthest gate shown: {alpha}");
    // #4 has reached nothing.
    let beta = out.lines().find(|l| l.contains("Beta")).expect("Beta row");
    assert!(beta.contains('—'), "no gate reached renders as a dash: {beta}");
}

// ----- F-8: containment tree, with documents as their own group -------------

#[test]
fn list_renders_the_containment_tree_and_a_document_group() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list"]).out);

    // The arc is a child of the project, the slices children of the arc.
    let arc = out.lines().find(|l| l.contains("First arc")).expect("arc row");
    assert!(arc.contains("└─") || arc.contains("├─"), "arc is branched: {arc}");
    let alpha = out.lines().find(|l| l.contains("Alpha")).expect("Alpha row");
    assert!(alpha.contains("├─") || alpha.contains("└─"), "slice is branched: {alpha}");
    // The slice is indented deeper than its arc.
    let indent = |line: &str| line.find('─').unwrap_or(0);
    assert!(indent(alpha) > indent(arc), "slices nest under their arc");

    // Documents follow, under their own header, not inside the tree.
    let docs_header = out.find("documents").expect("a documents group header");
    let doc_row = out.find("A design document").expect("the design row");
    assert!(docs_header < doc_row, "the document group precedes its rows");
    assert!(out.find("work").unwrap() < docs_header, "work group comes first");
}

// ----- F-6: displayed names are de-numbered ---------------------------------

#[test]
fn list_denumbers_displayed_names() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list"]).out);
    assert!(out.contains("Alpha"), "the name still shows:\n{out}");
    assert!(!out.contains("Slice 01"), "the number-reference is stripped:\n{out}");
    assert!(!out.contains("Arc 01"), "the arc's number-reference is stripped:\n{out}");

    // Display-only: the stored name is untouched (`--json` is the raw record).
    let json = run(dir.path(), &["list", "--json"]).out;
    assert!(json.contains("Slice 01 — Alpha"), "the stored name is unchanged:\n{json}");
}

// ----- F-9: `--width` elides the name column --------------------------------

#[test]
fn list_elides_names_past_the_width_limit() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list", "--width", "12"]).out);
    assert!(out.contains(" ..."), "a long name is elided:\n{out}");
    assert!(!out.contains("A design document"), "the full name is not shown:\n{out}");

    // A generous width leaves everything intact.
    let wide = plain(&run(dir.path(), &["list", "--width", "200"]).out);
    assert!(wide.contains("A design document"), "no elision when it fits:\n{wide}");
}

// ----- F-15: retired nodes are excluded by default, shown under `--all` -----

#[test]
fn list_excludes_retired_nodes_by_default() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list"]).out);
    assert!(!out.contains("Tombstone"), "retired node hidden by default:\n{out}");
    assert!(out.contains("withdrawn"), "the summary says rows were withheld:\n{out}");
}

#[test]
fn list_all_shows_retired_nodes_marked_and_dimmed() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let r = run(dir.path(), &["list", "--all"]);
    assert!(r.ok);
    let out = plain(&r.out);
    let row = out.lines().find(|l| l.contains("Tombstone")).expect("the retired row");
    assert!(row.contains("retired"), "STATUS reads retired: {row}");

    // The row is dimmed: its cells carry the bright-black foreground, which no
    // live row does.
    let raw_row = r.out.lines().find(|l| l.contains("Tombstone")).expect("the raw row");
    assert!(raw_row.contains("\u{1b}[90m"), "the retired row renders dimmed");
    let live_row = r.out.lines().find(|l| l.contains("Alpha")).expect("a live row");
    assert!(!live_row.contains("\u{1b}[90m"), "a live row is not dimmed");
}

#[test]
fn list_json_is_unfiltered_by_all() {
    // The machine path hands over every node and its `retired` field; the
    // default-hiding is a human-view affordance.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let json = run(dir.path(), &["list", "--json"]).out;
    assert!(json.contains("Tombstone"), "JSON carries the retired node:\n{json}");
}
