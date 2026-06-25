//! In-process tests for `odm orient` / `brief`.
//!
//! Each case seeds a corpus (via the library, to set bodies/edges/origin the CLI
//! cannot in one shot), optionally sets context, and drives [`odm_cli::dispatch`]
//! against the temp store. Test names carry the substrings the slice03 ledger
//! Verify commands filter on (`orient_section_order`, `orient_leads_with_vision`,
//! `orient_uses_context`, `orient_ready_blocked_softsat`,
//! `orient_surfaces_integrity`, `orient_drift_placeholder`,
//! `orient_no_project_fallback`, `brief_aliases_orient`).

use std::path::Path;
use std::str::FromStr;

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Dependency, Document, Frontmatter};
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
    code: Option<u8>,
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args should be structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, root, &mut out, &mut err);
    Run {
        ok: result.is_ok(),
        code: result.as_ref().ok().copied(),
        out: String::from_utf8(out).unwrap(),
    }
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 25).unwrap()
}

fn write_config(root: &Path) {
    std::fs::write(root.join("odm.toml"), CONFIG).unwrap();
}

fn id(tag: char) -> Id {
    Id::from_str(&format!("01ARZ3NDEKTSV4RRFFQ69G5F{tag}0")).unwrap()
}

fn persist(root: &Path, doc: Document) {
    Store::open(root).persist(&doc).expect("seed persist");
}

fn fm(tag: char, number: u32, ty: NodeType, name: &str, origin: Origin) -> Frontmatter {
    Frontmatter::new(id(tag), number, ty, name, day(), day(), origin)
}

/// Asserts `needle` appears in `hay` and returns its byte offset.
fn offset_of(hay: &str, needle: &str) -> usize {
    hay.find(needle).unwrap_or_else(|| panic!("expected {needle:?} in:\n{hay}"))
}

const VISION_BODY: &str = "# Vision\n\nMake the plan legible from one cheap call.\n";

/// A standing corpus: project P(1) ← arc A(2) ← slices Early(3), Late(4); Late
/// depends_on Early; plus a parentless Orphan(5) for the integrity section.
fn seed_full(root: &Path) {
    persist(
        root,
        Document::new(fm('P', 1, NodeType::Project, "Proj", Origin::Planned), VISION_BODY),
    );

    let mut a = fm('A', 2, NodeType::Arc, "Arc one", Origin::Planned);
    a.edges_mut().part_of = Some(id('P'));
    persist(root, Document::new(a, "# Arc one\n"));

    let mut early = fm('E', 3, NodeType::Slice, "Early", Origin::Planned);
    early.edges_mut().part_of = Some(id('A'));
    persist(root, Document::new(early, "# Early\n"));

    // Tags avoid the Crockford base32 exclusions (I, L, O, U): Late→T, Orphan→R.
    let mut late = fm('T', 4, NodeType::Slice, "Late", Origin::Planned);
    late.edges_mut().part_of = Some(id('A'));
    late.edges_mut().depends_on.push(Dependency::Bare(id('E')));
    persist(root, Document::new(late, "# Late\n"));

    persist(
        root,
        Document::new(fm('R', 5, NodeType::Slice, "Orphan", Origin::Planned), "# Orphan\n"),
    );
}

// ----- O-1: orient composes the model and renders sections in order ---------

#[test]
fn orient_section_order() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    seed_full(root);
    run(root, &["use", "project", "Proj"]);
    run(root, &["use", "arc", "Arc one"]);

    let r = run(root, &["orient"]);
    assert!(r.ok && r.code == Some(0), "orient exits 0");
    let o = &r.out;

    // vision → current focus → ready/blocked → integrity → drift.
    let vision = offset_of(o, "VISION");
    let focus = offset_of(o, "CURRENT FOCUS");
    let ready = offset_of(o, "READY");
    let blocked = offset_of(o, "BLOCKED");
    let integrity = offset_of(o, "INTEGRITY");
    let drift = offset_of(o, "DRIFT");
    assert!(
        vision < focus
            && focus < ready
            && ready < blocked
            && blocked < integrity
            && integrity < drift,
        "sections out of order:\n{o}"
    );
}

// ----- O-3: orient leads with the project name + vision, not the whole body -

#[test]
fn orient_leads_with_vision() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    // A single project ⇒ auto-resolved without context.
    persist(
        root,
        Document::new(fm('P', 1, NodeType::Project, "Proj", Origin::Planned), VISION_BODY),
    );

    let r = run(root, &["orient"]);
    assert!(r.ok);
    assert!(r.out.contains("VISION  #1 Proj"), "leads with the project name:\n{}", r.out);
    assert!(r.out.contains("Make the plan legible"), "shows the vision text:\n{}", r.out);
    // The vision heading itself is not echoed (we lead with the name).
    assert!(!r.out.contains("# Vision"), "the raw heading is not dumped:\n{}", r.out);
}

// ----- O-4: orient resolves context and shows the current arc + status ------

#[test]
fn orient_uses_context() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    seed_full(root);
    run(root, &["use", "arc", "Arc one"]);
    run(root, &["set-gate", "Arc one", "in-progress", "--evidence", "reproduced"]);

    let r = run(root, &["orient"]);
    assert!(r.ok);
    let focus = r.out.split("READY").next().unwrap();
    assert!(focus.contains("arc #2 Arc one"), "current arc shown:\n{}", r.out);
    assert!(
        focus.contains("in-progress=reproduced"),
        "arc status vector shown after vision:\n{}",
        r.out
    );
}

// ----- O-5: ready frontier carries the soft-sat ⚠; blocked names reasons -----

#[test]
fn orient_ready_blocked_softsat() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    persist(
        root,
        Document::new(fm('P', 1, NodeType::Project, "Proj", Origin::Planned), VISION_BODY),
    );

    // Softdep reaches its terminal gate only at `attested` (< reproduced).
    let mut soft = fm('S', 2, NodeType::Slice, "Softdep", Origin::Planned);
    soft.edges_mut().part_of = Some(id('P'));
    persist(root, Document::new(soft, "# Softdep\n"));
    // Easy depends_on Softdep ⇒ soft-satisfied ⇒ ready with a ⚠ flag.
    let mut easy = fm('E', 3, NodeType::Slice, "Easy", Origin::Planned);
    easy.edges_mut().part_of = Some(id('P'));
    easy.edges_mut().depends_on.push(Dependency::Bare(id('S')));
    persist(root, Document::new(easy, "# Easy\n"));

    run(root, &["set-gate", "Softdep", "tested", "--evidence", "attested"]);

    let r = run(root, &["orient"]);
    assert!(r.ok);
    let ready = r.out.split("BLOCKED").next().unwrap();
    assert!(ready.contains("slice #3 Easy"), "Easy is ready:\n{}", r.out);
    assert!(
        ready.contains("⚠ soft dep #2 Softdep at evidence=attested"),
        "soft-sat flag travels with the ready node:\n{}",
        r.out
    );
}

// ----- O-6: orient surfaces check integrity errors inline -------------------

#[test]
fn orient_surfaces_integrity() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    seed_full(root); // includes the parentless Orphan(5)

    let r = run(root, &["orient"]);
    assert!(r.ok);
    let integrity = r.out.split("DRIFT").next().unwrap().split("INTEGRITY").nth(1).unwrap();
    assert!(integrity.contains("[orphan]"), "orphan surfaced inline:\n{}", r.out);
    assert!(integrity.contains("#5 Orphan"), "names the offending node:\n{}", r.out);
}

// ----- O-5 (cont.): blocked nodes render every reason variant ---------------

#[test]
fn orient_ready_blocked_softsat_all_block_reasons() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    persist(
        root,
        Document::new(fm('P', 1, NodeType::Project, "Proj", Origin::Planned), VISION_BODY),
    );

    // Soft reaches terminal only at attested; Hard never; Ext is incomplete.
    let mut soft = fm('S', 2, NodeType::Slice, "Softdep", Origin::Planned);
    soft.edges_mut().part_of = Some(id('P'));
    persist(root, Document::new(soft, "# Softdep\n"));
    let mut hard = fm('H', 3, NodeType::Slice, "Harddep", Origin::Planned);
    hard.edges_mut().part_of = Some(id('P'));
    persist(root, Document::new(hard, "# Harddep\n"));
    let mut ext = fm('X', 4, NodeType::Slice, "Extdep", Origin::Planned);
    ext.edges_mut().part_of = Some(id('P'));
    persist(root, Document::new(ext, "# Extdep\n"));
    // Reader: depends_on Soft (soft) + Hard (unsatisfied), blocked_by Ext.
    let mut reader = fm('E', 5, NodeType::Slice, "Reader", Origin::Planned);
    reader.edges_mut().part_of = Some(id('P'));
    reader.edges_mut().depends_on.push(Dependency::Bare(id('S')));
    reader.edges_mut().depends_on.push(Dependency::Bare(id('H')));
    reader.edges_mut().blocked_by.push(id('X'));
    persist(root, Document::new(reader, "# Reader\n"));

    run(root, &["set-gate", "Softdep", "tested", "--evidence", "attested"]);

    let r = run(root, &["orient"]);
    assert!(r.ok);
    let blocked = r.out.split("BLOCKED").nth(1).unwrap().split("INTEGRITY").next().unwrap();
    assert!(blocked.contains("Reader"), "Reader is blocked:\n{}", r.out);
    assert!(blocked.contains("unsatisfied: #3 Harddep"), "unsatisfied reason:\n{}", r.out);
    assert!(
        blocked.contains("low-evidence: #2 Softdep at evidence=attested (needs reproduced)"),
        "soft-satisfied reason:\n{}",
        r.out
    );
    assert!(blocked.contains("blocked-by: #4 Extdep"), "external block reason:\n{}", r.out);
}

// ----- O-7: drift line reads the A5 placeholder -----------------------------

#[test]
fn orient_drift_placeholder() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    persist(
        root,
        Document::new(fm('P', 1, NodeType::Project, "Proj", Origin::Planned), VISION_BODY),
    );

    let r = run(root, &["orient"]);
    assert!(r.ok);
    let drift = r.out.split("DRIFT").nth(1).unwrap();
    assert!(drift.contains("not yet tracked (A5)"), "drift placeholder:\n{}", r.out);
}

// ----- O-9: no-current-project fallbacks, all exit 0 ------------------------

#[test]
fn orient_no_project_fallback() {
    // (a) No project: affordance to create one.
    let none = TempDir::new().unwrap();
    write_config(none.path());
    let r = run(none.path(), &["orient"]);
    assert!(r.ok && r.code == Some(0), "no-project orient still exits 0");
    assert!(r.out.contains("No project yet"), "create affordance:\n{}", r.out);
    assert!(r.out.contains("odm new project"), "names the command:\n{}", r.out);

    // (b) Exactly one project, no context: orient on it.
    let one = TempDir::new().unwrap();
    write_config(one.path());
    persist(
        one.path(),
        Document::new(fm('P', 1, NodeType::Project, "Solo", Origin::Planned), VISION_BODY),
    );
    let r = run(one.path(), &["orient"]);
    assert!(r.ok && r.code == Some(0));
    assert!(r.out.contains("VISION  #1 Solo"), "orients on the only project:\n{}", r.out);

    // (c) Multiple projects, no context: list + prompt to select.
    let many = TempDir::new().unwrap();
    write_config(many.path());
    persist(
        many.path(),
        Document::new(fm('A', 1, NodeType::Project, "Alpha", Origin::Planned), "# a\n"),
    );
    persist(
        many.path(),
        Document::new(fm('B', 2, NodeType::Project, "Beta", Origin::Planned), "# b\n"),
    );
    let r = run(many.path(), &["orient"]);
    assert!(r.ok && r.code == Some(0), "multi-project orient still exits 0");
    assert!(r.out.contains("none selected"), "lists projects:\n{}", r.out);
    assert!(r.out.contains("Alpha") && r.out.contains("Beta"), "names each project:\n{}", r.out);
    assert!(r.out.contains("odm use project"), "prompts a selection:\n{}", r.out);
}

// ----- O-10: `brief` is an alias of `orient` (identical output) -------------

#[test]
fn brief_aliases_orient() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    seed_full(root);
    run(root, &["use", "arc", "Arc one"]);

    let orient = run(root, &["orient"]);
    let brief = run(root, &["brief"]);
    assert!(orient.ok && brief.ok);
    assert_eq!(orient.out, brief.out, "brief output is byte-identical to orient");
}
