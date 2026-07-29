//! RH C-3 — the `odm list` overhaul, driven through `dispatch` in-process.
//!
//! One test per finding: F-4 (no NUMBER), F-5 (DATE first + `--date`), F-7
//! (STATUS after TYPE), F-8 (containment tree + the document group), F-6
//! (de-numbered names), F-9 (`--width` elision), F-15 (retired excluded by
//! default, shown under `--all`).

use std::path::Path;

use chrono::NaiveDate;
use clap::Parser as _;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
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
    run(root, &["node", "new", "project", "Root project"]);
    run(root, &["node", "new", "arc", "Arc 01 — First arc", "--parent", "1"]);
    run(root, &["node", "new", "slice", "Slice 01 — Alpha", "--parent", "2"]);
    run(root, &["node", "new", "slice", "Slice 02 — Beta", "--parent", "2"]);
    run(root, &["node", "new", "design", "A design document"]);
    run(root, &["node", "set-gate", "3", "built"]);
    run(root, &["node", "set-gate", "5", "draft"]);
    // #6: retired, so it must not appear in a default listing (F-15).
    run(root, &["node", "new", "slice", "Slice 03 — Tombstone", "--parent", "2"]);
    run(root, &["node", "retire", "6", "--because", "created against a stale list"]);
}

/// Persists an `artifact` node directly (the CLI's `node new` has no verb for
/// it — artifacts only ever arrive via migration/mint). `parent` sets
/// `part_of`, when given.
fn seed_artifact(root: &Path, number: u32, name: &str, parent: Option<Id>) {
    let today = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap();
    let mut fm = Frontmatter::new(
        Id::new(),
        number,
        NodeType::Artifact,
        name,
        today,
        today,
        Origin::Planned,
    );
    if let Some(parent) = parent {
        fm.edges_mut().part_of = Some(parent);
    }
    Store::open(root).persist(&Document::new(fm, format!("# {name}\n"))).expect("seed persist");
}

/// The id of the node with `number`, from a freshly-seeded store.
fn id_of(root: &Path, number: u32) -> Id {
    Store::open(root)
        .load_all()
        .expect("load_all")
        .into_iter()
        .find(|d| d.frontmatter().number() == number)
        .unwrap_or_else(|| panic!("#{number} not found"))
        .frontmatter()
        .id()
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
    let r = run(dir.path(), &["node", "list"]);
    assert!(r.ok);
    let out = plain(&r.out);
    assert!(!out.contains("NUMBER"), "no NUMBER column:\n{out}");
}

// ----- F-5: DATE leads, and `--date` switches which date --------------------

#[test]
fn list_leads_with_date_and_honours_the_date_flag() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["node", "list"]).out);
    let header = out.lines().nth(1).expect("a header row");
    let columns: Vec<&str> = header.split('│').map(str::trim).collect();
    assert_eq!(columns, ["DATE", "TYPE", "STATUS", "NAME", "ID"], "C-3 column order");

    // Both forms dispatch and render a table.
    for which in ["created", "updated"] {
        let r = run(dir.path(), &["node", "list", "--date", which]);
        assert!(r.ok, "--date={which} dispatches");
        assert!(plain(&r.out).contains("DATE"), "--date={which} renders");
    }
}

// ----- F-19: STATUS is the normalized state (was the raw gate, F-7) ---------

#[test]
fn list_status_shows_the_normalized_state_not_the_raw_gate() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["node", "list"]).out);

    // #3 reached `built` — the middle of planned/built/tested, so `active`.
    let alpha = out.lines().find(|l| l.contains("Alpha")).expect("Alpha row");
    assert!(alpha.contains("active"), "mid-ladder reads active: {alpha}");
    assert!(!alpha.contains("built"), "the raw gate is no longer the column: {alpha}");
    // #4 has reached nothing, so it sits at the start of its ladder.
    let beta = out.lines().find(|l| l.contains("Beta")).expect("Beta row");
    assert!(beta.contains("planned"), "nothing reached reads planned: {beta}");
}

#[test]
fn list_status_is_comparable_across_node_types() {
    // The point of F-19: a done slice and a done arc must read alike, though
    // their terminal gates are spelled `tested` and `verified`.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    run(dir.path(), &["node", "set-gate", "3", "tested"]);
    run(dir.path(), &["node", "set-gate", "2", "verified"]);
    let out = plain(&run(dir.path(), &["node", "list"]).out);

    let alpha = out.lines().find(|l| l.contains("Alpha")).expect("Alpha row");
    let arc = out.lines().find(|l| l.contains("First arc")).expect("arc row");
    assert!(alpha.contains("done"), "slice at `tested` reads done: {alpha}");
    assert!(arc.contains("done"), "arc at `verified` reads done too: {arc}");
}

#[test]
fn list_status_filter_accepts_both_vocabularies() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    run(dir.path(), &["node", "set-gate", "3", "tested"]);

    // The derived word the column shows...
    let derived = plain(&run(dir.path(), &["node", "list", "--status", "done"]).out);
    assert!(derived.contains("Alpha"), "--status done finds it: {derived}");
    // ...and the raw gate underneath it, which F-15 shipped and must not break.
    let raw = plain(&run(dir.path(), &["node", "list", "--status", "tested"]).out);
    assert!(raw.contains("Alpha"), "--status tested still works: {raw}");
    // A state it is not in matches neither way.
    let other = plain(&run(dir.path(), &["node", "list", "--status", "planned"]).out);
    assert!(!other.contains("Alpha"), "and does not over-match: {other}");
}

// ----- F-8: containment tree, with documents as their own group -------------

#[test]
fn list_renders_the_containment_tree_and_a_document_group() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["node", "list"]).out);

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

// ----- s10: a promoted `artifact` is tree-nested, not left in reference -----

#[test]
fn list_promotes_a_slice_attached_artifact_into_the_tree() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let alpha_id = id_of(dir.path(), 3); // Slice 01 — Alpha
    seed_artifact(dir.path(), 100, "Alpha's ledger", Some(alpha_id));

    let out = plain(&run(dir.path(), &["node", "list", "--all"]).out);
    let alpha = out.lines().find(|l| l.contains("Alpha")).expect("Alpha row");
    let ledger = out.lines().find(|l| l.contains("Alpha's ledger")).expect("ledger row");

    let indent = |line: &str| line.find('─').unwrap_or(0);
    assert!(indent(ledger) > indent(alpha), "the artifact nests under its slice: {ledger:?}");
    assert!(ledger.contains(" — "), "status shows the em-dash, unchanged: {ledger:?}");

    // No divider needed above it — it's part of the plan tree, not reference.
    let ledger_pos = out.lines().position(|l| l == ledger).unwrap();
    let rule_pos = out.lines().position(|l| l.contains("──────"));
    assert!(
        rule_pos.is_none_or(|r| ledger_pos < r),
        "a slice-promoted artifact sits above the divider, in the tree"
    );
}

#[test]
fn list_promotes_an_arc_attached_artifact_into_the_tree() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let arc_id = id_of(dir.path(), 2); // First arc
    seed_artifact(dir.path(), 101, "Chunk C-1 cc-prompt", Some(arc_id));

    let out = plain(&run(dir.path(), &["node", "list", "--all"]).out);
    let arc = out.lines().find(|l| l.contains("First arc")).expect("arc row");
    let alpha = out.lines().find(|l| l.contains("Alpha")).expect("Alpha row (an arc child)");
    let chunk = out.lines().find(|l| l.contains("Chunk C-1 cc-prompt")).expect("chunk row");

    // Indent *within the NAME cell* (the 4th `│`-delimited field) — comparing
    // whole-line offsets is fragile against the TYPE/STATUS columns' own
    // widths, which vary with the widest cell content across the table
    // (`artifact` is wider than `slice`), not with tree depth.
    let name_cell_indent = |line: &str| {
        let cell = line.split('│').nth(3).unwrap_or_default();
        cell.find('─').unwrap_or(0)
    };
    assert!(
        name_cell_indent(chunk) > name_cell_indent(arc),
        "the artifact nests under its arc: {chunk:?}"
    );
    assert_eq!(
        name_cell_indent(chunk),
        name_cell_indent(alpha),
        "an arc-attached artifact sits at the same depth as the arc's slice children"
    );
}

#[test]
fn list_does_not_promote_a_top_level_artifact() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    seed_artifact(dir.path(), 102, "A loose report", None);

    let out = plain(&run(dir.path(), &["node", "list", "--all"]).out);
    let report = out.lines().find(|l| l.contains("A loose report")).expect("report row");
    assert!(!report.contains('─'), "an uncontained artifact is not tree-rendered: {report:?}");

    // It stays below the divider, in the reference group, alongside the design doc.
    let rule = out.lines().position(|l| l.contains("──────")).expect("a divider row");
    let report_pos = out.lines().position(|l| l == report).unwrap();
    assert!(report_pos > rule, "an uncontained artifact stays in reference: {report:?}");
}

#[test]
fn list_promoted_artifact_type_cell_is_cyan_unpromoted_is_uncoloured() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let alpha_id = id_of(dir.path(), 3);
    seed_artifact(dir.path(), 100, "Alpha's ledger", Some(alpha_id));
    seed_artifact(dir.path(), 102, "A loose report", None);

    let raw = run(dir.path(), &["node", "list", "--all"]).out;
    let row_for = |name: &str| {
        raw.lines().find(|l| l.contains(name)).unwrap_or_else(|| panic!("{name} row")).to_string()
    };
    assert!(
        row_for("Alpha's ledger").contains("\u{1b}[38;2;150;224;248m"),
        "a promoted artifact gets the cyan-leaning blue"
    );
    assert!(
        !row_for("A loose report").contains("\u{1b}[38;2;150;224;248m"),
        "an unpromoted artifact stays uncoloured, same as before"
    );
}

#[test]
fn group_plan_includes_promoted_artifacts_reference_excludes_them() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let alpha_id = id_of(dir.path(), 3);
    seed_artifact(dir.path(), 100, "Alpha's ledger", Some(alpha_id));
    seed_artifact(dir.path(), 102, "A loose report", None);

    let plan = plain(&run(dir.path(), &["node", "list", "--group", "plan", "--all"]).out);
    assert!(plan.contains("Alpha's ledger"), "a promoted artifact counts as plan:\n{plan}");
    assert!(!plan.contains("A loose report"), "an unpromoted one does not:\n{plan}");

    let reference = plain(&run(dir.path(), &["node", "list", "--group", "reference", "--all"]).out);
    assert!(
        !reference.contains("Alpha's ledger"),
        "a promoted artifact is not reference:\n{reference}"
    );
    assert!(reference.contains("A loose report"), "an unpromoted one still is:\n{reference}");
}

// ----- F-6: displayed names are de-numbered ---------------------------------

#[test]
fn list_denumbers_displayed_names() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["node", "list"]).out);
    assert!(out.contains("Alpha"), "the name still shows:\n{out}");
    assert!(!out.contains("Slice 01"), "the number-reference is stripped:\n{out}");
    assert!(!out.contains("Arc 01"), "the arc's number-reference is stripped:\n{out}");

    // Display-only: the stored name is untouched (`--json` is the raw record).
    let json = run(dir.path(), &["node", "list", "--json"]).out;
    assert!(json.contains("Slice 01 — Alpha"), "the stored name is unchanged:\n{json}");
}

// ----- F-9: `--width` elides the name column --------------------------------

#[test]
fn list_elides_names_past_the_width_limit() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["node", "list", "--width", "12"]).out);
    assert!(out.contains(" ..."), "a long name is elided:\n{out}");
    assert!(!out.contains("A design document"), "the full name is not shown:\n{out}");

    // A generous width leaves everything intact.
    let wide = plain(&run(dir.path(), &["node", "list", "--width", "200"]).out);
    assert!(wide.contains("A design document"), "no elision when it fits:\n{wide}");
}

// ----- F-15: retired nodes are excluded by default, shown under `--all` -----

#[test]
fn list_excludes_retired_nodes_by_default() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["node", "list"]).out);
    assert!(!out.contains("Tombstone"), "retired node hidden by default:\n{out}");
    assert!(out.contains("withdrawn"), "the summary says rows were withheld:\n{out}");
}

#[test]
fn list_all_shows_retired_nodes_marked_and_dimmed() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let r = run(dir.path(), &["node", "list", "--all"]);
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

    let shown = plain(&run(dir.path(), &["node", "list", "--all"]).out);
    let beta_all = shown.lines().find(|l| l.contains("Beta")).expect("Beta row");
    let tomb = shown.lines().find(|l| l.contains("Tombstone")).expect("Tombstone row");
    assert!(beta_all.contains("├─"), "with the tombstone shown, Beta is not last: {beta_all}");
    assert!(tomb.contains("└─"), "the tombstone closes the branch: {tomb}");

    let hidden = plain(&run(dir.path(), &["node", "list"]).out);
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
    run(dir.path(), &["node", "retire", "2", "--because", "folded into another arc"]);

    let out = plain(&run(dir.path(), &["node", "list"]).out);
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
    let raw = run(dir.path(), &["node", "list", "--group", "reference"]).out;
    let doc = raw.lines().find(|l| l.contains("A design document")).expect("the design row");
    assert!(doc.contains("\u{1b}[33m"), "a `draft` status is yellow: {doc:?}");
    assert!(!doc.contains("\u{1b}[1m"), "the state carries no bold, as in 0.3.5: {doc:?}");
}

#[test]
fn work_gate_statuses_carry_the_matching_palette_slots() {
    // The work sequences postdate the 0.3.5 palette, so they reuse its slots:
    // early yellow, under way cyan, done green, verified-live bright green.
    // F-19 folded `complete`/`tested`/`verified` into one `done`, so the
    // bright-green "verified live" shade no longer has a state to itself — the
    // rung stays exact in `node show` and `--json`. What the column keeps is
    // one distinct colour per normalized state.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    run(dir.path(), &["node", "set-gate", "4", "planned"]);
    run(dir.path(), &["node", "set-gate", "1", "verified"]);
    let raw = run(dir.path(), &["node", "list"]).out;

    let row_for = |name: &str| {
        raw.lines().find(|l| l.contains(name)).unwrap_or_else(|| panic!("{name} row")).to_string()
    };
    // The normalized vocabulary reuses the slots the gate palette had tuned:
    // under way cyan, not started yellow, done green.
    assert!(row_for("Alpha").contains("\u{1b}[36m"), "`active` is cyan (under way)");
    assert!(row_for("Beta").contains("\u{1b}[33m"), "`planned` is yellow (not started)");
    assert!(row_for("Root project").contains("\u{1b}[32m"), "`done` is green");
}

#[test]
fn type_cells_carry_one_hue_per_node_type() {
    // One hue per type, so a long listing sorts by kind at a glance. Truecolor,
    // because violet and orange have no basic ANSI slot.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let raw = run(dir.path(), &["node", "list"]).out;
    let row_for = |name: &str| {
        raw.lines().find(|l| l.contains(name)).unwrap_or_else(|| panic!("{name} row")).to_string()
    };
    assert!(row_for("Root project").contains("\u{1b}[38;2;212;110;197m"), "project is dim magenta");
    assert!(row_for("First arc").contains("\u{1b}[38;2;167;139;250m"), "arc is violet");
    assert!(row_for("Alpha").contains("\u{1b}[38;2;122;162;247m"), "slice is blue");
    assert!(row_for("A design document").contains("\u{1b}[38;2;240;128;74m"), "design is orange");
}

#[test]
fn date_and_id_are_muted_but_keep_the_alternating_band() {
    // The date and the id are context; muting them lets type/status/name carry
    // the eye. The muting is per-band, so the alternating stripe survives.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let raw = run(dir.path(), &["node", "list"]).out;
    let data: Vec<&str> = raw.lines().skip(2).filter(|l| l.contains("20")).collect();

    let first_fg = |line: &str| {
        let i = line.find("\u{1b}[38;2;").expect("a truecolor foreground");
        line[i..].split('m').next().unwrap().to_string()
    };
    // Two different muted tones, one per band — and neither is the full-strength
    // band colour the NAME column still carries.
    let a = first_fg(data[0]);
    let b = first_fg(data[1]);
    assert_ne!(a, b, "the bands stay distinguishable when muted");
    assert!(a.contains("147;125;99"), "band A date is muted: {a}");
    assert!(b.contains("147;108;67"), "band B date is muted: {b}");
    assert!(raw.contains("\u{1b}[38;2;254;215;170m"), "NAME keeps the full band colour");
}

#[test]
fn dimming_wins_over_the_state_colour_on_a_withdrawn_row() {
    // A retired row is dimmed whole; the status colour must not fight it.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let raw = run(dir.path(), &["node", "list", "--all"]).out;
    let row = raw.lines().find(|l| l.contains("Tombstone")).expect("the retired row");
    assert!(row.contains("\u{1b}[90m"), "the row is dimmed: {row:?}");
}

// ----- `--group`: one family at a time -------------------------------------

#[test]
fn group_plan_shows_only_work_nodes() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["node", "list", "--group", "plan"]).out);
    assert!(out.contains("Root project"), "the plan shows:\n{out}");
    assert!(out.contains("Alpha"), "its slices show:\n{out}");
    assert!(!out.contains("A design document"), "reference material does not:\n{out}");
    assert!(!out.contains("──────"), "one group needs no dividing rule:\n{out}");
}

#[test]
fn group_reference_shows_only_document_nodes() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["node", "list", "--group", "reference"]).out);
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
    let plain_plan = plain(&run(dir.path(), &["node", "list", "--group", "plan"]).out);
    assert!(!plain_plan.contains("Tombstone"), "still excluded by default:\n{plain_plan}");
    let with_all = plain(&run(dir.path(), &["node", "list", "--group", "plan", "--all"]).out);
    assert!(with_all.contains("Tombstone"), "--all still applies within a group:\n{with_all}");
}

// ----- `--status`: look at exactly what a default listing withholds ---------

#[test]
fn status_filter_shows_only_the_matching_nodes() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let out = plain(&run(dir.path(), &["node", "list", "--status", "built"]).out);
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
    let out = plain(&run(dir.path(), &["node", "list", "--status", "retired"]).out);
    assert!(out.contains("Tombstone"), "the retired node shows:\n{out}");
    assert!(!out.contains("Alpha"), "live work does not:\n{out}");
}

#[test]
fn status_filter_is_case_insensitive_and_reports_an_empty_result() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let upper = plain(&run(dir.path(), &["node", "list", "--status", "RETIRED"]).out);
    assert!(upper.contains("Tombstone"), "the match ignores case:\n{upper}");

    let none = plain(&run(dir.path(), &["node", "list", "--status", "nonesuch"]).out);
    assert!(none.contains("no nodes with status"), "an empty result says why:\n{none}");
}

#[test]
fn list_json_is_unfiltered_by_all() {
    // The machine path hands over every node and its `retired` field; the
    // default-hiding is a human-view affordance.
    let dir = TempDir::new().unwrap();
    seed(dir.path());
    let json = run(dir.path(), &["node", "list", "--json"]).out;
    assert!(json.contains("Tombstone"), "JSON carries the retired node:\n{json}");
}
