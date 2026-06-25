//! In-process tests for the `odm rollup` command and its rendered `ROLLUP.md`.
//!
//! Each case drives [`odm_cli::dispatch`] against a temp store (with an
//! `odm.toml` gate config) and asserts on the rendered file. Test names carry
//! the substrings the slice02 ledger Verify commands filter on
//! (`rollup_ready_blocked`, `rollup_active_tears_rationale`, `rollup_origin_view`,
//! `rollup_drift_placeholder`, `rollup_omits_deferred_until_a5`,
//! `rollup_command_regenerates`, `rollup_header_and_dry_run`).

use std::path::Path;

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use tempfile::TempDir;

const CONFIG: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]

[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]

[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

struct Run {
    ok: bool,
    out: String,
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
    NaiveDate::from_ymd_opt(2026, 6, 25).unwrap()
}

/// Writes the gate config so status/satisfaction are configured.
fn write_config(root: &Path) {
    std::fs::write(root.join("odm.toml"), CONFIG).unwrap();
}

/// Reads the generated rollup file.
fn read_rollup(root: &Path) -> String {
    std::fs::read_to_string(root.join("ROLLUP.md")).expect("ROLLUP.md should exist")
}

/// Seeds a node directly through the library (to set fields the CLI cannot,
/// e.g. `origin` or `part_of`/`depends_on` in one shot).
fn seed(root: &Path, build: impl FnOnce() -> Document) {
    Store::open(root).persist(&build()).expect("seed persist");
}

fn fm(number: u32, ty: NodeType, name: &str, origin: Origin) -> Frontmatter {
    Frontmatter::new(Id::new(), number, ty, name, day(), day(), origin)
}

// ----- R-4: ready/blocked rendered; blocked names its unsatisfied edges -----

#[test]
fn rollup_ready_blocked_named_edges() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    // Build P -> Q -> {early, late}, with `late` depending on `early`.
    run(root, &["new", "project", "Proj"]);
    run(root, &["new", "arc", "Arc one", "--parent", "1"]);
    run(root, &["new", "slice", "Early", "--parent", "2"]);
    run(root, &["new", "slice", "Late", "--parent", "2"]);
    run(root, &["link", "Late", "depends_on", "Early"]);

    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);

    // `Early` (no deps) is ready; `Late` (unsatisfied dep) is blocked and names
    // the unsatisfied edge.
    let ready = md.split("## Blocked").next().unwrap();
    assert!(ready.contains("Early"), "Early should be in the Ready section:\n{md}");

    let blocked = md.split("## Blocked").nth(1).unwrap();
    assert!(blocked.contains("Late"), "Late should be blocked:\n{md}");
    assert!(
        blocked.contains("unsatisfied: #3 Early"),
        "the blocked entry must name its unsatisfied edge:\n{md}"
    );
}

// ----- R-5: active tears render with their rationale ------------------------

#[test]
fn rollup_active_tears_rationale_rendered() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    // A <-> B cycle, broken by tearing A depends_on B with a rationale.
    run(root, &["new", "slice", "Aaa"]);
    run(root, &["new", "slice", "Bbb"]);
    run(root, &["link", "Aaa", "depends_on", "Bbb"]);
    run(root, &["link", "Bbb", "depends_on", "Aaa"]);
    run(root, &["tear", "Aaa", "depends_on", "Bbb", "--because", "cut the A-B cycle"]);

    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);

    let tears = md.split("## Active tears").nth(1).unwrap();
    assert!(tears.contains("depends_on"), "tear edge rendered:\n{md}");
    assert!(tears.contains("because: cut the A-B cycle"), "tear rationale rendered:\n{md}");
}

// ----- R-6: provenance view labels nodes by origin --------------------------

#[test]
fn rollup_origin_view_groups_by_provenance() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    // One planned (via CLI), one discovered, one amendment (via seed).
    run(root, &["new", "slice", "Planned slice"]);
    seed(root, || {
        Document::new(fm(2, NodeType::Slice, "Found slice", Origin::Discovered), "# x\n")
    });
    seed(root, || {
        Document::new(fm(3, NodeType::Slice, "Amended slice", Origin::Amendment), "# y\n")
    });

    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);

    let prov = md.split("## Provenance").nth(1).unwrap();
    let planned = prov.split("### Discovered").next().unwrap();
    let discovered =
        prov.split("### Discovered").nth(1).unwrap().split("### Amendment").next().unwrap();
    let amendment = prov.split("### Amendment").nth(1).unwrap();

    assert!(planned.contains("Planned slice"), "planned group:\n{md}");
    assert!(discovered.contains("Found slice"), "discovered group:\n{md}");
    assert!(amendment.contains("Amended slice"), "amendment group:\n{md}");
}

// ----- R-7: drift section is present but reads "not yet tracked (A5)" --------

#[test]
fn rollup_drift_placeholder_no_fake_data() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    run(root, &["new", "slice", "Only"]);

    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);

    assert!(md.contains("## Drift"), "a Drift section is structurally present:\n{md}");
    let drift = md.split("## Drift").nth(1).unwrap();
    assert!(drift.contains("Not yet tracked (A5)"), "drift reads the A5 placeholder:\n{md}");
}

// ----- R-8: no deferred surfacing — no section, no `deferred` status ---------

#[test]
fn rollup_omits_deferred_until_a5() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    run(root, &["new", "project", "Proj"]);
    run(root, &["new", "arc", "Arc", "--parent", "1"]);
    run(root, &["set-gate", "Arc", "in-progress", "--evidence", "reproduced"]);

    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);

    // No deferred section and no `deferred` status variant anywhere (Q-A3-1).
    assert!(!md.to_lowercase().contains("deferred"), "no deferred surfacing in A3:\n{md}");
}

// ----- R-9: full-scan regenerate is idempotent (same corpus → same bytes) ---

#[test]
fn rollup_command_regenerates_idempotently() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    run(root, &["new", "project", "Proj"]);
    run(root, &["new", "arc", "Arc", "--parent", "1"]);
    run(root, &["new", "slice", "Slice", "--parent", "2"]);

    assert!(run(root, &["rollup"]).ok);
    let first = read_rollup(root);
    assert!(run(root, &["rollup"]).ok);
    let second = read_rollup(root);

    assert_eq!(first, second, "regenerating an unchanged corpus yields identical bytes");
}

// ----- R-10: generated header + --dry-run writes nothing --------------------

#[test]
fn rollup_header_and_dry_run() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    run(root, &["new", "slice", "Only"]);

    // --dry-run: previews to stdout, writes no file.
    let dry = run(root, &["rollup", "--dry-run"]);
    assert!(dry.ok);
    assert!(dry.out.contains("do not edit"), "preview carries the generated header:\n{}", dry.out);
    assert!(dry.out.contains("odm rollup"), "header names the regenerating command:\n{}", dry.out);
    assert!(!root.join("ROLLUP.md").exists(), "--dry-run must not write the file");
    assert!(dry.err.contains("nothing written"), "dry-run reports nothing written:\n{}", dry.err);

    // A real run writes the file, header and all.
    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);
    assert!(md.contains("do not edit") && md.contains("odm rollup"), "header in file:\n{md}");
}

// ----- Coverage: an empty corpus renders every section's empty placeholder --

#[test]
fn rollup_empty_corpus_renders_placeholders() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);

    assert!(md.contains("_(no nodes)_"), "empty tree placeholder:\n{md}");
    assert!(md.contains("_(nothing ready)_"), "empty ready placeholder:\n{md}");
    assert!(md.contains("_(nothing blocked)_"), "empty blocked placeholder:\n{md}");
    assert!(md.contains("## Active tears"), "tears section present:\n{md}");
    // All three provenance groups render their empty placeholder.
    assert_eq!(md.matches("_(none)_").count(), 4, "tears + 3 empty origin groups:\n{md}");
}

// ----- Coverage: a tree node with no gate-set renders without a status tail -

#[test]
fn rollup_tree_node_without_gateset_has_no_status() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    // `note` has no `[gates.note]` sequence, so its status vector is empty.
    seed(root, || Document::new(fm(1, NodeType::Note, "Loose note", Origin::Planned), "# n\n"));

    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);
    let tree = md.split("## Ready").next().unwrap();
    assert!(tree.contains("note #1 Loose note"), "the note appears in the tree:\n{md}");
    assert!(
        tree.contains("- note #1 Loose note\n"),
        "a gate-less node renders with no ` — status` tail:\n{md}"
    );
}

// ----- Coverage: soft-satisfied (ready) + low-evidence/externally-blocked ---

#[test]
fn rollup_soft_and_block_reason_rendering() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    // Dependencies:
    //   Soft   — reached its terminal gate but only at `attested` (< reproduced).
    //   Hard   — never advanced (unsatisfied).
    //   Ext    — incomplete external block target.
    //   Reader — depends_on Soft and Hard, blocked_by Ext.
    //   Easy   — depends_on Soft only ⇒ soft-satisfied ⇒ ready with a soft flag.
    run(root, &["new", "slice", "Soft dep"]);
    run(root, &["new", "slice", "Hard dep"]);
    run(root, &["new", "slice", "Ext block"]);
    run(root, &["new", "slice", "Reader"]);
    run(root, &["new", "slice", "Easy"]);
    run(root, &["set-gate", "Soft dep", "tested", "--evidence", "attested"]);
    run(root, &["link", "Reader", "depends_on", "Soft dep"]);
    run(root, &["link", "Reader", "depends_on", "Hard dep"]);
    run(root, &["link", "Reader", "blocked_by", "Ext block"]);
    run(root, &["link", "Easy", "depends_on", "Soft dep"]);

    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);

    // Ready section: Easy is ready, flagged with its soft dependency.
    let ready = md.split("## Blocked").next().unwrap();
    assert!(ready.contains("Easy"), "Easy is ready:\n{md}");
    assert!(ready.contains("soft: #1 Soft dep at evidence=attested"), "soft dep flagged:\n{md}");

    // Blocked section: Reader names all three reason kinds.
    let blocked = md.split("## Blocked").nth(1).unwrap();
    assert!(blocked.contains("Reader"), "Reader is blocked:\n{md}");
    assert!(blocked.contains("unsatisfied: #2 Hard dep"), "unsatisfied edge named:\n{md}");
    assert!(
        blocked.contains("low-evidence: #1 Soft dep at evidence=attested (needs reproduced)"),
        "soft-satisfied reason named:\n{md}"
    );
    assert!(blocked.contains("blocked-by: #3 Ext block"), "external block named:\n{md}");
}
