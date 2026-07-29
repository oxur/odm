//! In-process tests for `odm check`'s `source.paths` portability guard
//! (arc-migration-fidelity slice10 iteration 1, ODD-0025 §2.2): any
//! `source.paths` entry that is not repo-content-root-relative — absolute, or
//! anchored at a `.worktrees/` superproject root — is a hard Error, code
//! `absolute-source-path`.
//!
//! **Unconditional, unlike doc-coverage:** no `[coverage] scan_root` (or any
//! other config key) gates this rule — it always runs, because there is no
//! legitimate absolute `source.paths` case.

use std::fs;
use std::path::Path;

use chrono::NaiveDate;
use clap::Parser as _;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Frontmatter, Source};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use tempfile::TempDir;

const GATES: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

struct Run {
    code: Option<u8>,
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = odm_cli::dispatch(cli, root, &mut out, &mut err).ok();
    Run { code, out: String::from_utf8(out).unwrap() }
}

fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config.toml"), GATES).unwrap();
    dir
}

fn seed_design_node(root: &Path, name: &str, source_path: &str) {
    let today = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap();
    let fm = Frontmatter::new(Id::new(), 1, NodeType::Design, name, today, today, Origin::Planned)
        .with_source(Source {
            paths: vec![source_path.into()],
            class: "odd".to_string(),
            normalization: "trim+lf".to_string(),
            migrated_by: "test".to_string(),
            migrated_on: today,
            synthesis: None,
            attestation: None,
        });
    let document = Document::new(fm, "# Doc\n\nBody.\n".to_string());
    Store::open(root).persist(&document).expect("seed persist");
}

// ----- an absolute `source.paths` entry is a hard Error ---------------------

#[test]
fn check_flags_an_absolute_source_path_as_an_error() {
    let dir = fixture();
    seed_design_node(
        dir.path(),
        "Absolute-path doc",
        "/Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design/04-accepted/0022-x.md",
    );

    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1), "an absolute source.paths is a hard Error: {}", r.out);
    assert!(r.out.contains("absolute-source-path"), "the rule fires: {}", r.out);
    assert!(r.out.contains("Absolute-path doc"), "and names the node: {}", r.out);
}

// ----- a `.worktrees/`-anchored (superproject-root) entry is also an Error --

#[test]
fn check_flags_a_worktrees_anchored_source_path_as_an_error() {
    let dir = fixture();
    seed_design_node(
        dir.path(),
        "Worktree-anchored doc",
        ".worktrees/1.0.x/docs/design/04-accepted/0023-x.md",
    );

    let r = run(dir.path(), &["check"]);
    assert_eq!(r.code, Some(1), "a .worktrees/-anchored path is a hard Error: {}", r.out);
    assert!(r.out.contains("absolute-source-path"), "the rule fires: {}", r.out);
}

// ----- a content-root-relative entry is green --------------------------------

#[test]
fn check_is_green_for_a_relative_source_path() {
    let dir = fixture();
    seed_design_node(dir.path(), "Relative-path doc", "docs/design/04-accepted/0022-x.md");

    let r = run(dir.path(), &["check"]);
    assert!(!r.out.contains("absolute-source-path"), "no finding for a relative path: {}", r.out);
    assert_eq!(r.code, Some(0), "clean corpus, exit 0: {}", r.out);
}
