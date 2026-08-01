//! In-process tests for the `odm migrate` command surface. Drives
//! [`odm_cli::dispatch`] against a temp store. Test names carry the substrings
//! the ledger Verify commands filter on: `migrate_command_exists` (slice01 M-1),
//! `check_green_on_migrated_odm_docs` (slice02 N-2).
//!
//! **s14: config-driven, not path-driven.** There is no more `<LEGACY_PATH>`
//! positional — every root (the design/research corpus, the dev corpus, the
//! plan-set(s)) resolves from the store's operational `config.toml`. Fixtures
//! therefore write that file (via [`set_docs_directory`]/[`set_dev_directory`]/
//! [`write_config`]) instead of passing a path on the command line.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use tempfile::TempDir;

/// The absolute path of a fixture corpus under the workspace `test-data/`.
fn fixtures(name: &str) -> PathBuf {
    // odm-cli manifest dir is crates/odm-cli; the fixtures live at the root.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data").join(name)
}

/// The workspace path of the live `docs/design` corpus.
fn real_docs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design")
}

fn run(root: &Path, args: &[&str]) -> (bool, String, String) {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let ok = odm_cli::dispatch(cli, root, &mut out, &mut err).is_ok();
    (ok, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

/// Like [`run`] but returns the intended exit code (for `check`, which returns
/// its own code rather than erroring).
fn run_code(root: &Path, args: &[&str]) -> (Option<u8>, String) {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = odm_cli::dispatch(cli, root, &mut out, &mut err).ok();
    (code, String::from_utf8(out).unwrap())
}

/// The work-node gate-sets every self-host-touching fixture needs to
/// validate at all.
const GATES: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

/// A path formatted as a TOML string literal (Rust's `Debug` quoting matches
/// TOML's double-quoted string escaping closely enough for the plain,
/// no-embedded-quote paths every fixture here uses).
fn quoted(p: &Path) -> String {
    format!("{:?}", p.display().to_string())
}

/// Writes `config.toml` at `root`: `extra` (the roots a test cares about)
/// plus the standard gate-sets.
fn write_config(root: &Path, extra: &str) {
    std::fs::write(root.join("config.toml"), format!("{extra}{GATES}")).unwrap();
}

/// Config with `docs_directory` only.
fn set_docs_directory(root: &Path, docs_directory: &Path) {
    write_config(root, &format!("docs_directory = {}\n", quoted(docs_directory)));
}

/// Config with `dev_directory` only (`--notes`' root — never derived from
/// `docs_directory`).
fn set_dev_directory(root: &Path, dev_directory: &Path) {
    write_config(root, &format!("dev_directory = {}\n", quoted(dev_directory)));
}

/// Config with both `docs_directory` and `dev_directory`.
fn set_docs_and_dev_directory(root: &Path, docs_directory: &Path, dev_directory: &Path) {
    write_config(
        root,
        &format!(
            "docs_directory = {}\ndev_directory = {}\n",
            quoted(docs_directory),
            quoted(dev_directory)
        ),
    );
}

/// Recursively copies `src` into `dst` (test-only; `std` has no built-in).
fn copy_dir_all(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let dest_path = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_all(&entry.path(), &dest_path);
        } else {
            std::fs::copy(entry.path(), &dest_path).unwrap();
        }
    }
}

/// A fresh `docs_directory` tempdir whose `design/` subtree is a copy of
/// `source` — s14 F-2's restored append means the design/research derivation
/// always resolves `docs_directory/design`, so a fixture corpus not already
/// named `design` on disk needs wrapping, not just pointing at directly.
fn docs_directory_wrapping_design(source: &Path) -> TempDir {
    let dir = TempDir::new().unwrap();
    copy_dir_all(source, &dir.path().join("design"));
    dir
}

#[test]
fn migrate_command_exists() {
    let store_dir = TempDir::new().unwrap();
    let docs_dir = docs_directory_wrapping_design(&fixtures("legacy"));
    set_docs_directory(store_dir.path(), docs_dir.path());

    // Dry-run first: the command parses, runs, and writes nothing.
    let (ok, out, err) = run(store_dir.path(), &["migrate", "--dry-run"]);
    assert!(ok, "migrate --dry-run dispatches cleanly");
    assert!(out.contains("would create"), "dry-run plans creations:\n{out}");
    assert!(err.contains("dry-run"), "status names dry-run:\n{err}");
    assert!(!store_dir.path().join("nodes").exists(), "dry-run wrote nothing");

    // Commit: the six fixture docs become nodes.
    let (ok, _out, err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "migrate dispatches cleanly");
    assert!(err.contains("6 created"), "reports six created:\n{err}");
    let store = Store::open(store_dir.path());
    assert_eq!(store.load_all().unwrap().len(), 6, "six nodes persisted");
}

#[test]
fn migrate_command_renders_warnings_and_skips() {
    // The edge corpus surfaces a skip table and a dangling-supersedes warning.
    let store_dir = TempDir::new().unwrap();
    let docs_dir = docs_directory_wrapping_design(&fixtures("legacy-edge"));
    set_docs_directory(store_dir.path(), docs_dir.path());

    let (ok, out, _err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "migrate over the edge corpus dispatches cleanly");
    assert!(out.contains("skip"), "skip rows rendered:\n{out}");
    assert!(out.contains("warnings:"), "warnings section rendered:\n{out}");
    assert!(out.contains("dangling"), "the dangling supersession is shown:\n{out}");
}

#[test]
fn migrate_command_reports_empty_corpus() {
    // An empty `design/` dir → the "no legacy documents" path (not an error).
    let store_dir = TempDir::new().unwrap();
    let empty = TempDir::new().unwrap();
    let docs_dir = docs_directory_wrapping_design(empty.path());
    set_docs_directory(store_dir.path(), docs_dir.path());

    let (ok, out, _err) = run(store_dir.path(), &["migrate"]);
    assert!(ok);
    assert!(out.contains("no legacy documents found"), "empty-corpus message:\n{out}");
}

// ----- s10: sourceless nodes are backfilled in the same `migrate` pass ------

#[test]
fn migrate_backfills_a_sourceless_node_and_still_imports_the_rest() {
    let store_dir = TempDir::new().unwrap();
    let docs_dir = docs_directory_wrapping_design(&fixtures("legacy"));
    set_docs_directory(store_dir.path(), docs_dir.path());

    let store = Store::open(store_dir.path());
    // A pre-source design node matching the legacy fixture's real #1
    // (test-data/legacy/01-draft/0001-early-draft.md) -- the pre-slice03
    // shape `backfill_source` exists to repair.
    let today = NaiveDate::from_ymd_opt(2026, 7, 28).unwrap();
    let fm =
        Frontmatter::new(Id::new(), 1, NodeType::Design, "Stub #1", today, today, Origin::Planned);
    store.persist(&Document::new(fm, "# Stub #1\n".to_string())).unwrap();

    let (ok, _out, err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "migrate dispatches cleanly:\n{err}");
    assert!(err.contains("reconciled"), "the sourceless node is reported reconciled:\n{err}");
    assert!(err.contains("5 created"), "the other five fixture docs still import:\n{err}");

    let node = store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|d| d.frontmatter().number() == 1)
        .expect("#1 present");
    assert!(node.frontmatter().source().is_some(), "backfilled with source");
    assert!(
        node.body().contains("The first sketch"),
        "the stub body was replaced with the real legacy content: {:?}",
        node.body()
    );
}

// ----- s13/s14: plain `migrate` on a plan set also reconciles a drifted node

#[test]
fn migrate_reconciles_a_drifted_already_sourced_arc_node() {
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    std::fs::write(plan_root.path().join(".git"), "gitdir: fake\n").unwrap();
    std::fs::write(plan_root.path().join("project-plan.md"), "# Test Project\n").unwrap();
    let arc_dir = plan_root.path().join("arc01-alpha");
    std::fs::create_dir_all(&arc_dir).unwrap();
    std::fs::write(arc_dir.join("arc-plan.md"), "# Arc 01 — Alpha\n\nOriginal.\n").unwrap();
    set_docs_directory(store_dir.path(), plan_root.path());

    let (ok, _out, err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "self-host dispatches cleanly:\n{err}");

    // The arc-plan.md keeps evolving, as a real, actively-worked arc's does.
    std::fs::write(arc_dir.join("arc-plan.md"), "# Arc 01 — Alpha\n\nAmended, live content.\n")
        .unwrap();

    let (ok, out, err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "second migrate dispatches cleanly:\n{err}");
    assert!(err.contains("source-reconciled"), "status names the reconcile pass:\n{err}");
    assert!(out.contains("SOURCE RECONCILE"), "the reconcile table is rendered:\n{out}");

    let store = Store::open(store_dir.path());
    let arc = store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Arc)
        .expect("arc node present");
    assert!(
        arc.body().contains("Amended, live content"),
        "the arc node's body was re-snapshotted: {}",
        arc.body()
    );
}

// ----- s09/s10: `migrate --artifacts` mints the supporting-doc corpus -------

#[test]
fn migrate_artifacts_mints_supporting_docs() {
    let store_dir = TempDir::new().unwrap();
    let plan_set = fixtures("plan-set");
    set_docs_directory(store_dir.path(), &plan_set);

    // Self-host first so containment has something to resolve against.
    let (ok, _out, err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "self-host dispatches cleanly:\n{err}");

    let (ok, out, err) = run(store_dir.path(), &["migrate", "--artifacts"]);
    assert!(ok, "migrate --artifacts dispatches cleanly:\n{err}");
    assert!(err.contains("artifact(s) minted"), "status names what happened:\n{err}");
    assert!(out.contains("ARTIFACTS"), "the mint table is rendered:\n{out}");

    let store = Store::open(store_dir.path());
    let minted = store
        .load_all()
        .unwrap()
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == odm_core::NodeType::Artifact)
        .count();
    assert!(minted > 0, "at least one artifact was minted");
}

#[test]
fn migrate_artifacts_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let plan_set = fixtures("plan-set");
    set_docs_directory(store_dir.path(), &plan_set);
    run(store_dir.path(), &["migrate"]);

    let (ok, out, err) = run(store_dir.path(), &["migrate", "--artifacts", "--dry-run"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(out.contains("would mint"), "dry-run plans mints:\n{out}");
    assert!(err.contains("nothing written"), "status names dry-run:\n{err}");

    let store = Store::open(store_dir.path());
    let artifact_count = store
        .load_all()
        .unwrap()
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == odm_core::NodeType::Artifact)
        .count();
    assert_eq!(artifact_count, 0, "dry-run minted nothing");
}

// ----- s10: `migrate --notes` mints the dev-doc corpus ----------------------

#[test]
fn migrate_notes_mints_dev_docs() {
    let store_dir = TempDir::new().unwrap();
    let dev = TempDir::new().unwrap();
    std::fs::write(dev.path().join("0001-early-thoughts.md"), "# Early thoughts\nbody\n").unwrap();
    std::fs::create_dir_all(dev.path().join("research")).unwrap();
    std::fs::write(dev.path().join("research/0001-survey.md"), "# Survey\nbody\n").unwrap();
    set_dev_directory(store_dir.path(), dev.path());

    let (ok, out, err) = run(store_dir.path(), &["migrate", "--notes"]);
    assert!(ok, "migrate --notes dispatches cleanly:\n{err}");
    assert!(err.contains("note(s) minted"), "status names what happened:\n{err}");
    assert!(out.contains("NOTES"), "the mint table is rendered:\n{out}");
    assert!(out.contains("research"), "the subdirectory tag is shown:\n{out}");

    let store = Store::open(store_dir.path());
    let notes: Vec<_> = store
        .load_all()
        .unwrap()
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == odm_core::NodeType::Note)
        .collect();
    assert_eq!(notes.len(), 2, "both dev docs minted");
    assert!(
        notes.iter().all(|n| n.frontmatter().edges().part_of.is_none()),
        "notes are uncontained"
    );
}

#[test]
fn migrate_notes_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let dev = TempDir::new().unwrap();
    std::fs::write(dev.path().join("0001-a.md"), "# A\nbody\n").unwrap();
    set_dev_directory(store_dir.path(), dev.path());

    let (ok, out, err) = run(store_dir.path(), &["migrate", "--notes", "--dry-run"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(out.contains("would mint"), "dry-run plans mints:\n{out}");
    assert!(err.contains("nothing written"), "status names dry-run:\n{err}");

    let store = Store::open(store_dir.path());
    assert!(store.load_all().unwrap().is_empty(), "dry-run minted nothing");
}

#[test]
fn migrate_artifacts_and_notes_report_nothing_to_mint_once_covered() {
    let store_dir = TempDir::new().unwrap();
    let plan_set = fixtures("plan-set");
    let dev = TempDir::new().unwrap();
    std::fs::write(dev.path().join("0001-a.md"), "# A\nbody\n").unwrap();
    set_docs_and_dev_directory(store_dir.path(), &plan_set, dev.path());

    run(store_dir.path(), &["migrate"]);
    run(store_dir.path(), &["migrate", "--artifacts"]);
    let (ok, out, _err) = run(store_dir.path(), &["migrate", "--artifacts"]);
    assert!(ok);
    assert!(out.contains("nothing to mint"), "a second run reports nothing left:\n{out}");

    run(store_dir.path(), &["migrate", "--notes"]);
    let (ok, out, _err) = run(store_dir.path(), &["migrate", "--notes"]);
    assert!(ok);
    assert!(out.contains("nothing to mint"), "a second notes run reports nothing left:\n{out}");
}

#[test]
fn migrate_artifacts_and_notes_conflict() {
    assert!(
        Cli::try_parse_from(["odm", "migrate", "--artifacts", "--notes"]).is_err(),
        "--artifacts and --notes are mutually exclusive"
    );
    assert!(
        Cli::try_parse_from(["odm", "migrate", "--coverage", "--artifacts"]).is_err(),
        "--coverage and --artifacts are mutually exclusive"
    );
}

// ----- s14 F-1: the CLI surface is config-driven, not path-driven ----------

#[test]
fn migrate_help_shows_no_legacy_path_and_an_optional_additional_paths() {
    let mut buf = Vec::new();
    // `Cli::command()` is clap's own generated help renderer — the same text
    // `--help` prints, without spawning a process.
    <Cli as clap::CommandFactory>::command()
        .find_subcommand_mut("migrate")
        .unwrap()
        .write_long_help(&mut buf)
        .unwrap();
    let help = String::from_utf8(buf).unwrap();
    assert!(!help.contains("LEGACY_PATH"), "no positional corpus path:\n{help}");
    assert!(help.contains("[ADDITIONAL_PATHS]"), "the additionals positional is optional:\n{help}");
}

#[test]
fn migrate_requires_docs_directory_to_be_configured() {
    let store_dir = TempDir::new().unwrap();
    write_config(store_dir.path(), ""); // gates only — no docs_directory
    let cli = Cli::try_parse_from(["odm", "migrate"]).unwrap();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, store_dir.path(), &mut out, &mut err);
    let message = result.expect_err("no docs_directory configured").to_string();
    assert!(message.contains("docs_directory"), "names the missing key:\n{message}");
}

// ----- s13/s14: `migrate --all` composes every derivation into one pass ----

/// A `docs_root` (`./docs`) combining all three families `--all` composes: a
/// legacy design corpus under `docs/design` (s14 F-2's append target), a
/// plan set, and dev docs — plus a store `config.toml` pointing
/// `docs_directory` at the **parent** `./docs` (s14: the canonical,
/// wider form the operator is moving to) and `dev_directory` at `./docs/dev`.
fn write_all_fixture(store_root: &Path) {
    std::fs::write(
        store_root.join("config.toml"),
        format!("docs_directory = \"./docs\"\ndev_directory = \"./docs/dev\"\n{GATES}"),
    )
    .unwrap();

    let legacy_doc = store_root.join("docs/design/01-draft/0001-early-draft.md");
    std::fs::create_dir_all(legacy_doc.parent().unwrap()).unwrap();
    std::fs::write(
        &legacy_doc,
        "---\nnumber: 1\ntitle: \"An early draft\"\nstate: Draft\nversion: 0.1\n---\n\n\
         # An early draft\n\nBody carried verbatim.\n",
    )
    .unwrap();

    let plan_root = store_root.join("docs/design-v1.0.0");
    std::fs::create_dir_all(&plan_root).unwrap();
    std::fs::write(plan_root.join(".git"), "gitdir: fake\n").unwrap();
    std::fs::write(plan_root.join("project-plan.md"), "# All CLI Test — Plan\n\nNo DoD yet.\n")
        .unwrap();
    let arc_dir = plan_root.join("arc01-alpha");
    std::fs::create_dir_all(&arc_dir).unwrap();
    std::fs::write(arc_dir.join("arc-plan.md"), "# Arc 01 — Alpha\n\nBody.\n").unwrap();

    let dev_doc = store_root.join("docs/dev/notes/0001-scratch.md");
    std::fs::create_dir_all(dev_doc.parent().unwrap()).unwrap();
    std::fs::write(&dev_doc, "# Scratch\n\nBody.\n").unwrap();
}

#[test]
fn migrate_all_composes_every_derivation() {
    let store_dir = TempDir::new().unwrap();
    write_all_fixture(store_dir.path());

    let (ok, _out, err) = run(store_dir.path(), &["migrate", "--all"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(err.contains("migrate --all"), "status names the composite:\n{err}");

    let store = Store::open(store_dir.path());
    let all = store.load_all().unwrap();

    assert!(
        all.iter()
            .any(|d| d.frontmatter().number() == 1
                && d.frontmatter().node_type() == NodeType::Design),
        "the legacy design doc was reconciled/created: {:?}",
        all.iter().map(|d| d.frontmatter().node_type()).collect::<Vec<_>>()
    );
    assert!(
        all.iter().any(|d| d.frontmatter().node_type() == NodeType::Arc),
        "the plan set was self-hosted (arc present)"
    );
    assert!(
        all.iter().any(|d| d.frontmatter().node_type() == NodeType::Project),
        "the plan set was self-hosted (project present)"
    );
    assert!(
        all.iter().any(|d| d.frontmatter().node_type() == NodeType::Note),
        "the dev doc was minted as a note"
    );
    // `--all` no longer has a vision step at all (arc-migration-fidelity
    // s15 F-2, ODD-0025 §2.3 reversal) — the project migrates as a plain
    // 1:1 node, so nothing ever carries `source.synthesis`.
    assert!(
        all.iter().all(|d| d.frontmatter().source().is_none_or(|s| s.synthesis.is_none())),
        "no synthesis was minted (the vision step no longer exists)"
    );
}

#[test]
fn migrate_all_is_idempotent() {
    let store_dir = TempDir::new().unwrap();
    write_all_fixture(store_dir.path());
    run(store_dir.path(), &["migrate", "--all"]);

    let store = Store::open(store_dir.path());
    let before = store.load_all().unwrap().len();

    let (ok, _out, err) = run(store_dir.path(), &["migrate", "--all"]);
    assert!(ok, "second run dispatches cleanly:\n{err}");

    let after = store.load_all().unwrap().len();
    assert_eq!(before, after, "a second run creates nothing new");
}

#[test]
fn migrate_all_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    write_all_fixture(store_dir.path());

    let (ok, _out, err) = run(store_dir.path(), &["migrate", "--all", "--dry-run"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(err.contains("nothing written"), "status names dry-run:\n{err}");
    assert!(!store_dir.path().join("nodes").exists(), "dry-run wrote nothing");
}

#[test]
fn migrate_all_falls_back_to_legacy_config_directories() {
    // Same fixture, but docs_directory/dev_directory only exist under
    // `[legacy]` (what `odm store init` writes when it finds a pre-split
    // odm.toml, arc-migration-fidelity s13) — no top-level keys at all.
    let store_dir = TempDir::new().unwrap();
    write_all_fixture(store_dir.path());
    std::fs::write(
        store_dir.path().join("config.toml"),
        format!("[legacy]\ndocs_directory = \"./docs\"\ndev_directory = \"./docs/dev\"\n{GATES}"),
    )
    .unwrap();

    let (ok, _out, err) = run(store_dir.path(), &["migrate", "--all"]);
    assert!(ok, "dispatches cleanly:\n{err}");

    let store = Store::open(store_dir.path());
    let all = store.load_all().unwrap();
    assert!(
        all.iter()
            .any(|d| d.frontmatter().number() == 1
                && d.frontmatter().node_type() == NodeType::Design),
        "the legacy design doc was found via [legacy].docs_directory + the design append"
    );
    assert!(
        all.iter().any(|d| d.frontmatter().node_type() == NodeType::Note),
        "the dev doc was found via [legacy].dev_directory"
    );
}

#[test]
fn migrate_all_conflicts_with_the_other_action_flags() {
    for flag in ["--plan", "--legacy", "--replan", "--coverage", "--artifacts", "--notes"] {
        assert!(
            Cli::try_parse_from(["odm", "migrate", "--all", flag]).is_err(),
            "--all conflicts with {flag}"
        );
    }
}

// ----- s14 F-2: the docs_directory + "design" append is restored -----------

#[test]
fn migrate_all_applies_design_rules_only_under_docs_directory_design() {
    // docs_directory points at the parent; only `<docs_directory>/design`
    // holds NN-state design docs. A `.md` sitting directly under
    // docs_directory (not inside `design/`) must not receive the
    // design/research derivation — it's swept by --artifacts instead, since
    // it's just some uncovered supporting doc from --all's perspective.
    let store_dir = TempDir::new().unwrap();
    std::fs::write(
        store_dir.path().join("config.toml"),
        format!("docs_directory = \"./docs\"\n{GATES}"),
    )
    .unwrap();

    let design_doc = store_dir.path().join("docs/design/01-draft/0001-a.md");
    std::fs::create_dir_all(design_doc.parent().unwrap()).unwrap();
    std::fs::write(
        &design_doc,
        "---\nnumber: 1\ntitle: \"A\"\nstate: Draft\nversion: 0.1\n---\n\n# A\n\nBody.\n",
    )
    .unwrap();

    // A loose doc directly under docs_directory (not under design/).
    std::fs::write(store_dir.path().join("docs/loose-notes.md"), "# Loose\n\nBody.\n").unwrap();

    let (ok, _out, err) = run(store_dir.path(), &["migrate", "--all"]);
    assert!(ok, "dispatches cleanly:\n{err}");

    let store = Store::open(store_dir.path());
    let all = store.load_all().unwrap();
    assert!(
        all.iter()
            .any(|d| d.frontmatter().node_type() == NodeType::Design
                && d.frontmatter().number() == 1),
        "the design/NN-state doc got the design derivation: {:?}",
        all.iter()
            .map(|d| (d.frontmatter().node_type(), d.frontmatter().number()))
            .collect::<Vec<_>>()
    );
    // The loose doc is real (--artifacts covers it, uncontained), but never
    // as a `Design`-typed, NN-state-derived node.
    assert!(
        all.iter()
            .all(|d| d.frontmatter().node_type() != NodeType::Design
                || d.frontmatter().number() != 0),
        "no spurious design node was minted from the loose file"
    );
}

// ----- s14 F-4/F-5/F-7: additional-paths sweep, persistence, dedup ---------

#[test]
fn migrate_all_sweeps_and_persists_additional_paths() {
    let store_dir = TempDir::new().unwrap();
    write_all_fixture(store_dir.path());
    let research = store_dir.path().join("research");
    std::fs::create_dir_all(&research).unwrap();
    std::fs::write(research.join("0001-idea.md"), "# Idea\n\nBody.\n").unwrap();

    let (ok, out, err) = run(store_dir.path(), &["migrate", "--all", "research"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(err.contains("1 additional dir(s) swept"), "status names the sweep:\n{err}");
    assert!(out.contains("ARTIFACTS"), "the research dir was minted via the generic pass:\n{out}");

    let config = std::fs::read_to_string(store_dir.path().join("config.toml")).unwrap();
    assert!(
        config.contains("[legacy]") && config.contains("additional_paths = [\"research\"]"),
        "persisted for the next run:\n{config}"
    );

    // A forgotten re-pass (no positional) still covers `research`, from config.
    let store = Store::open(store_dir.path());
    let before = store
        .load_all()
        .unwrap()
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == NodeType::Artifact)
        .count();
    let (ok, _out, err) = run(store_dir.path(), &["migrate", "--all"]);
    assert!(ok, "re-pass with no positional dispatches cleanly:\n{err}");
    assert!(err.contains("1 additional dir(s) swept"), "config alone still covers it:\n{err}");
    let after = store
        .load_all()
        .unwrap()
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == NodeType::Artifact)
        .count();
    assert_eq!(before, after, "already covered — 0 new artifacts");
}

#[test]
fn migrate_all_additional_paths_union_sorts_and_dedupes_across_runs() {
    let store_dir = TempDir::new().unwrap();
    write_all_fixture(store_dir.path());
    for name in ["x", "y", "z"] {
        std::fs::create_dir_all(store_dir.path().join(name)).unwrap();
    }

    run(store_dir.path(), &["migrate", "--all", "x,y"]);
    let config = std::fs::read_to_string(store_dir.path().join("config.toml")).unwrap();
    assert!(config.contains("additional_paths = [\"x\", \"y\"]"), "sorted, first pass:\n{config}");

    run(store_dir.path(), &["migrate", "--all", "y,z"]);
    let config = std::fs::read_to_string(store_dir.path().join("config.toml")).unwrap();
    assert!(
        config.contains("additional_paths = [\"x\", \"y\", \"z\"]"),
        "union, sorted, no dup y:\n{config}"
    );
}

#[test]
fn migrate_all_dry_run_writes_no_config() {
    let store_dir = TempDir::new().unwrap();
    write_all_fixture(store_dir.path());
    let before = std::fs::read_to_string(store_dir.path().join("config.toml")).unwrap();

    run(store_dir.path(), &["migrate", "--all", "research", "--dry-run"]);

    let after = std::fs::read_to_string(store_dir.path().join("config.toml")).unwrap();
    assert_eq!(before, after, "dry-run writes neither nodes nor config");
}

#[test]
fn migrate_all_excludes_an_additional_nested_under_design_or_dev_from_the_generic_pass() {
    // An "additional" that is actually inside docs_directory/design (or
    // dev_directory) must not be swept a second time by the generic
    // (artifact) pass — it's already covered by the design or notes
    // derivation, which applies the rules that pass doesn't.
    let store_dir = TempDir::new().unwrap();
    write_all_fixture(store_dir.path());

    let (ok, _out, err) =
        run(store_dir.path(), &["migrate", "--all", "docs/design/01-draft,docs/dev"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(err.contains("0 additional dir(s) swept"), "both excluded as overlapping:\n{err}");

    // Still persisted (F-5's union doesn't filter what F-7 excludes from
    // processing), just not double-processed.
    let config = std::fs::read_to_string(store_dir.path().join("config.toml")).unwrap();
    assert!(
        config.contains("docs/design/01-draft") && config.contains("docs/dev"),
        "remembered anyway:\n{config}"
    );
}

#[test]
fn migrate_all_plan_set_escape_hatch_self_hosts_an_additional_arc_directory() {
    // D-2: an "additional" dir that turns out to be Plan-shaped (an `arcNN-*`
    // child, `detect_corpus`'s own definition) gets self-hosted, not swept as
    // generic supporting docs.
    //
    // **Finding, disclosed in the closing report**: the escape hatch cannot
    // sensibly self-host a *second, independent* `project-plan.md` tree — a
    // store has exactly one project space (`selfhost::PROJECT_NUMBER` is a
    // fixed `1000`, not derived per plan-set), so a second tree's project
    // node collides with the first's and is silently skipped as
    // already-existing, not created as a genuine second project. The escape
    // hatch only makes sense for an additional dir that folds *into* the
    // same project — e.g. a detached arc directory, as tested here.
    let store_dir = TempDir::new().unwrap();
    write_all_fixture(store_dir.path());
    let extra_arc = store_dir.path().join("orphaned-arc").join("arc99-detached");
    std::fs::create_dir_all(&extra_arc).unwrap();
    std::fs::write(extra_arc.join("arc-plan.md"), "# Arc 99 — Detached\n\nBody.\n").unwrap();

    let (ok, _out, err) = run(store_dir.path(), &["migrate", "--all", "orphaned-arc"]);
    assert!(ok, "dispatches cleanly:\n{err}");

    let store = Store::open(store_dir.path());
    let arcs = store
        .load_all()
        .unwrap()
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == NodeType::Arc)
        .count();
    assert_eq!(arcs, 2, "the primary plan's arc, plus the escape-hatch arc, both self-hosted");
}

// ----- s14 F-6: `[legacy]` resolves from the config the code reads ---------

#[test]
fn migrate_all_resolves_legacy_config_from_a_split_store_not_the_locator() {
    // ODD-0022 §4.2's split: a code-branch `odm.toml` with `[store]`
    // redirects to a *separate* store dir with its own `config.toml`. A
    // `[legacy]` block sitting in the *locator* (the code-branch odm.toml)
    // is not what `StoreHome::resolve` reads once a `[store]`-pointed
    // config.toml exists — it must be the store's own file.
    let repo_root = TempDir::new().unwrap();
    let store_root = repo_root.path().join("store-home");
    std::fs::create_dir_all(&store_root).unwrap();

    // The locator: `[store]` redirection *and* a decoy `[legacy]` block that
    // must NOT be the one the code resolves.
    std::fs::write(
        repo_root.path().join("odm.toml"),
        format!(
            "[store]\nworktree_base = \"{}\"\nworktree_name = \"store-home\"\nbranch_name = \"x\"\n\n\
             [legacy]\ndocs_directory = \"./wrong\"\n",
            repo_root.path().display()
        ),
    )
    .unwrap();

    // The store's own config.toml: the real `[legacy]` block.
    let plan_root = store_root.join("docs-v1");
    std::fs::create_dir_all(&plan_root).unwrap();
    std::fs::write(plan_root.join("project-plan.md"), "# Split Store Test\n").unwrap();
    std::fs::write(
        store_root.join("config.toml"),
        format!("[legacy]\ndocs_directory = {}\n{GATES}", quoted(&plan_root)),
    )
    .unwrap();

    let (ok, _out, err) = run(repo_root.path(), &["migrate"]);
    assert!(ok, "dispatches cleanly:\n{err}");

    // `Store::open` takes a store root directly (no internal `[store]`
    // redirection — that's `StoreHome::resolve`'s job, already done for
    // `dispatch()` itself); the split layout means nodes live under
    // `store_root`, not `repo_root`.
    let store = Store::open(&store_root);
    assert!(
        store.load_all().unwrap().iter().any(|d| d.frontmatter().node_type() == NodeType::Project),
        "resolved docs_directory from the store's config.toml, not the locator's decoy [legacy]"
    );
}

// ----- s14 F-2's D-1: no positional anywhere; every mode from config -------

#[test]
fn migrate_coverage_and_artifacts_resolve_docs_root_from_config_not_a_positional() {
    let store_dir = TempDir::new().unwrap();
    let plan_set = fixtures("plan-set");
    set_docs_directory(store_dir.path(), &plan_set);

    let (ok, out, _err) = run(store_dir.path(), &["migrate", "--coverage"]);
    assert!(ok, "--coverage dispatches with no positional");
    assert!(out.contains("Coverage report"), "the report renders:\n{out}");
}

// ----- s14: --replan, and the two other config-driven error paths ----------

#[test]
fn migrate_replan_resolves_plan_roots_from_config() {
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    std::fs::write(plan_root.path().join(".git"), "gitdir: fake\n").unwrap();
    std::fs::write(plan_root.path().join("project-plan.md"), "# Replan CLI Test\n").unwrap();
    set_docs_directory(store_dir.path(), plan_root.path());
    run(store_dir.path(), &["migrate"]);

    // Amend the plan's own name — --replan should pick it up in place.
    std::fs::write(plan_root.path().join("project-plan.md"), "# Renamed CLI Test\n").unwrap();

    let (ok, out, err) = run(store_dir.path(), &["migrate", "--replan"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(
        out.contains("Renamed CLI Test") || out.contains("re-derived"),
        "the re-stamp ran over the config-resolved plan root:\n{out}"
    );
}

#[test]
fn migrate_notes_errors_when_dev_directory_is_not_configured() {
    let store_dir = TempDir::new().unwrap();
    write_config(store_dir.path(), ""); // gates only — no dev_directory
    let cli = Cli::try_parse_from(["odm", "migrate", "--notes"]).unwrap();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, store_dir.path(), &mut out, &mut err);
    let message = result.expect_err("no dev_directory configured").to_string();
    assert!(message.contains("dev_directory"), "names the missing key:\n{message}");
}

// ----- N-2: `odm check` is green on the migrated real ODD corpus -------------

#[test]
fn check_green_on_migrated_odm_docs() {
    // Import odm's real `docs/design` into a fresh store (migrate never mutates
    // the legacy tree — proven in odm-migrate's never-delete test), then assert
    // `odm check` is green (exit 0) on the imported document graph.
    let store_dir = TempDir::new().unwrap();
    let docs_dir = docs_directory_wrapping_design(&real_docs());
    set_docs_directory(store_dir.path(), docs_dir.path());

    let (ok, _out, err) = run(store_dir.path(), &["migrate"]);
    assert!(ok, "migrate real docs dispatches cleanly:\n{err}");
    assert!(err.contains("created"), "some ODDs imported:\n{err}");

    let (code, out) = run_code(store_dir.path(), &["validate"]);
    assert_eq!(code, Some(0), "check is green on the imported odd corpus:\n{out}");
}
