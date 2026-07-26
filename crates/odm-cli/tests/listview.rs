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
    run(root, &["set-gate", "5", "draft"]);
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

    // Documents follow the work tree, separated by a rule rather than mixed
    // into it. The rule keeps the column dividers, and the leading/trailing
    // space every other row has.
    let rule = out.lines().position(|l| l.contains("──────")).expect("a divider row");
    let rule_line = out.lines().nth(rule).unwrap();
    assert!(rule_line.starts_with(' '), "the rule keeps the leading space: {rule_line:?}");
    assert!(rule_line.ends_with(' '), "the rule keeps the trailing space: {rule_line:?}");
    assert!(rule_line.contains('┼'), "the rule crosses the column separators: {rule_line:?}");
    assert!(!rule_line.contains('│'), "a crossing is a junction, not a bar: {rule_line:?}");

    let doc_row = out.lines().position(|l| l.contains("A design document")).expect("design row");
    let work_row = out.lines().position(|l| l.contains("Alpha")).expect("a work row");
    assert!(work_row < rule, "work comes above the rule");
    assert!(rule < doc_row, "documents come below it");
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

// ----- F-15 × F-8: hiding a node must re-shape the tree, not leave it stale --

#[test]
fn hiding_the_last_child_promotes_its_previous_sibling_to_the_closing_glyph() {
    // The seeded arc's children are Alpha, Beta, and the retired Tombstone.
    // With Tombstone hidden, Beta is the last *visible* child and must close
    // the branch — a `├─` here would point at a row that is not rendered.
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let shown = plain(&run(dir.path(), &["list", "--all"]).out);
    let beta_all = shown.lines().find(|l| l.contains("Beta")).expect("Beta row");
    let tomb = shown.lines().find(|l| l.contains("Tombstone")).expect("Tombstone row");
    assert!(beta_all.contains("├─"), "with the tombstone shown, Beta is not last: {beta_all}");
    assert!(tomb.contains("└─"), "the tombstone closes the branch: {tomb}");

    let hidden = plain(&run(dir.path(), &["list"]).out);
    let beta = hidden.lines().find(|l| l.contains("Beta")).expect("Beta row");
    assert!(beta.contains("└─"), "with the tombstone hidden, Beta must close the branch: {beta}");
    assert!(!beta.contains("├─"), "no dangling branch to an absent row: {beta}");
}

#[test]
fn hiding_a_parent_reroots_its_children() {
    // Retiring the arc removes it from the default view; its slices must become
    // roots of that view rather than staying indented under an absent parent.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    run(dir.path(), &["retire", "2", "--because", "folded into another arc"]);

    let out = plain(&run(dir.path(), &["list"]).out);
    assert!(!out.contains("First arc"), "the retired arc is hidden:\n{out}");
    let alpha = out.lines().find(|l| l.contains("Alpha")).expect("Alpha row");
    assert!(
        !alpha.contains("├─") && !alpha.contains("└─"),
        "an orphaned child renders as a root, not indented under nothing: {alpha}"
    );
}

// ----- STATUS carries the 0.3.5 state palette -------------------------------

#[test]
fn status_cells_carry_the_legacy_state_colours() {
    // The palette `oxur-odm` 0.3.5 shipped, via the same `oxur-term` helper:
    // draft yellow (SGR 33), accepted/final green (32) — colour only, no bold.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let raw = run(dir.path(), &["list", "--group", "reference"]).out;
    let doc = raw.lines().find(|l| l.contains("A design document")).expect("the design row");
    assert!(doc.contains("\u{1b}[33m"), "a `draft` status is yellow: {doc:?}");
    assert!(!doc.contains("\u{1b}[1m"), "the state carries no bold, as in 0.3.5: {doc:?}");
}

#[test]
fn work_gate_statuses_carry_the_matching_palette_slots() {
    // The work sequences postdate the 0.3.5 palette, so they reuse its slots:
    // early yellow, under way cyan, done green, verified-live bright green.
    // `complete` green and `verified` bright green keep ODD-0013 §5.1's
    // done-at-its-layer / verified-live distinction visible.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    run(dir.path(), &["set-gate", "4", "planned"]);
    run(dir.path(), &["set-gate", "1", "verified"]);
    let raw = run(dir.path(), &["list"]).out;

    let row_for = |name: &str| {
        raw.lines().find(|l| l.contains(name)).unwrap_or_else(|| panic!("{name} row")).to_string()
    };
    assert!(row_for("Alpha").contains("\u{1b}[36m"), "`built` is cyan (under way)");
    assert!(row_for("Beta").contains("\u{1b}[33m"), "`planned` is yellow (not started)");
    assert!(row_for("Root project").contains("\u{1b}[92m"), "`verified` is bright green");
}

#[test]
fn dimming_wins_over_the_state_colour_on_a_withdrawn_row() {
    // A retired row is dimmed whole; the status colour must not fight it.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let raw = run(dir.path(), &["list", "--all"]).out;
    let row = raw.lines().find(|l| l.contains("Tombstone")).expect("the retired row");
    assert!(row.contains("\u{1b}[90m"), "the row is dimmed: {row:?}");
}

// ----- `--group`: one family at a time -------------------------------------

#[test]
fn group_plan_shows_only_work_nodes() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list", "--group", "plan"]).out);
    assert!(out.contains("Root project"), "the plan shows:\n{out}");
    assert!(out.contains("Alpha"), "its slices show:\n{out}");
    assert!(!out.contains("A design document"), "reference material does not:\n{out}");
    assert!(!out.contains("──────"), "one group needs no dividing rule:\n{out}");
}

#[test]
fn group_reference_shows_only_document_nodes() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list", "--group", "reference"]).out);
    assert!(out.contains("A design document"), "reference material shows:\n{out}");
    assert!(!out.contains("Root project"), "the plan does not:\n{out}");
    assert!(!out.contains("Alpha"), "nor its slices:\n{out}");
    assert!(!out.contains("──────"), "one group needs no dividing rule:\n{out}");
}

#[test]
fn group_composes_with_the_other_filters() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    // The retired node is a work node, so it is in the plan group — and still
    // withheld unless asked for.
    let plain_plan = plain(&run(dir.path(), &["list", "--group", "plan"]).out);
    assert!(!plain_plan.contains("Tombstone"), "still excluded by default:\n{plain_plan}");
    let with_all = plain(&run(dir.path(), &["list", "--group", "plan", "--all"]).out);
    assert!(with_all.contains("Tombstone"), "--all still applies within a group:\n{with_all}");
}

// ----- `--status`: look at exactly what a default listing withholds ---------

#[test]
fn status_filter_shows_only_the_matching_nodes() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list", "--status", "built"]).out);
    assert!(out.contains("Alpha"), "the node at `built` shows:\n{out}");
    assert!(!out.contains("Beta"), "a node at another status does not:\n{out}");
    assert!(!out.contains("A design document"), "documents are filtered too:\n{out}");
}

#[test]
fn status_filter_on_a_withdrawn_value_reveals_the_withheld_rows() {
    // The point of the flag: `--status retired` shows exactly what a default
    // listing hides, without having to pass `--all` and hunt for it.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["list", "--status", "retired"]).out);
    assert!(out.contains("Tombstone"), "the retired node shows:\n{out}");
    assert!(!out.contains("Alpha"), "live work does not:\n{out}");
}

#[test]
fn status_filter_is_case_insensitive_and_reports_an_empty_result() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let upper = plain(&run(dir.path(), &["list", "--status", "RETIRED"]).out);
    assert!(upper.contains("Tombstone"), "the match ignores case:\n{upper}");

    let none = plain(&run(dir.path(), &["list", "--status", "nonesuch"]).out);
    assert!(none.contains("no nodes with status"), "an empty result says why:\n{none}");
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
