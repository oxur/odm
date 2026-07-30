//! In-process tests for the `odm migrate` command surface. Drives
//! [`odm_cli::dispatch`] against a temp store. Test names carry the substrings
//! the ledger Verify commands filter on: `migrate_command_exists` (slice01 M-1),
//! `check_green_on_migrated_odm_docs` (slice02 N-2).

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

#[test]
fn migrate_command_exists() {
    let store_dir = TempDir::new().unwrap();
    let legacy = fixtures("legacy");

    // Dry-run first: the command parses, runs, and writes nothing.
    let (ok, out, err) = run(store_dir.path(), &["migrate", legacy.to_str().unwrap(), "--dry-run"]);
    assert!(ok, "migrate --dry-run dispatches cleanly");
    assert!(out.contains("would create"), "dry-run plans creations:\n{out}");
    assert!(err.contains("dry-run"), "status names dry-run:\n{err}");
    assert!(!store_dir.path().join("nodes").exists(), "dry-run wrote nothing");

    // Commit: the six fixture docs become nodes.
    let (ok, _out, err) = run(store_dir.path(), &["migrate", legacy.to_str().unwrap()]);
    assert!(ok, "migrate dispatches cleanly");
    assert!(err.contains("6 created"), "reports six created:\n{err}");
    let store = Store::open(store_dir.path());
    assert_eq!(store.load_all().unwrap().len(), 6, "six nodes persisted");
}

#[test]
fn migrate_command_renders_warnings_and_skips() {
    // The edge corpus surfaces a skip table and a dangling-supersedes warning.
    let store_dir = TempDir::new().unwrap();
    let edge = fixtures("legacy-edge");
    let (ok, out, _err) = run(store_dir.path(), &["migrate", edge.to_str().unwrap()]);
    assert!(ok, "migrate over the edge corpus dispatches cleanly");
    assert!(out.contains("skip"), "skip rows rendered:\n{out}");
    assert!(out.contains("warnings:"), "warnings section rendered:\n{out}");
    assert!(out.contains("dangling"), "the dangling supersession is shown:\n{out}");
}

#[test]
fn migrate_command_reports_empty_corpus() {
    // An empty directory → the "no legacy documents" path (not an error).
    let store_dir = TempDir::new().unwrap();
    let empty = TempDir::new().unwrap();
    let (ok, out, _err) = run(store_dir.path(), &["migrate", empty.path().to_str().unwrap()]);
    assert!(ok);
    assert!(out.contains("no legacy documents found"), "empty-corpus message:\n{out}");
}

// ----- s10: sourceless nodes are backfilled in the same `migrate` pass ------

#[test]
fn migrate_backfills_a_sourceless_node_and_still_imports_the_rest() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // A pre-source design node matching the legacy fixture's real #1
    // (test-data/legacy/01-draft/0001-early-draft.md) -- the pre-slice03
    // shape `backfill_source` exists to repair.
    let today = NaiveDate::from_ymd_opt(2026, 7, 28).unwrap();
    let fm =
        Frontmatter::new(Id::new(), 1, NodeType::Design, "Stub #1", today, today, Origin::Planned);
    store.persist(&Document::new(fm, "# Stub #1\n".to_string())).unwrap();

    let legacy = fixtures("legacy");
    let (ok, _out, err) = run(store_dir.path(), &["migrate", legacy.to_str().unwrap()]);
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

// ----- s13: plain `migrate` on a plan set also reconciles a drifted node ----

#[test]
fn migrate_reconciles_a_drifted_already_sourced_arc_node() {
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    std::fs::write(plan_root.path().join(".git"), "gitdir: fake\n").unwrap();
    std::fs::write(plan_root.path().join("project-plan.md"), "# Test Project\n").unwrap();
    let arc_dir = plan_root.path().join("arc01-alpha");
    std::fs::create_dir_all(&arc_dir).unwrap();
    std::fs::write(arc_dir.join("arc-plan.md"), "# Arc 01 — Alpha\n\nOriginal.\n").unwrap();

    let (ok, _out, err) = run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap()]);
    assert!(ok, "self-host dispatches cleanly:\n{err}");

    // The arc-plan.md keeps evolving, as a real, actively-worked arc's does.
    std::fs::write(arc_dir.join("arc-plan.md"), "# Arc 01 — Alpha\n\nAmended, live content.\n")
        .unwrap();

    let (ok, out, err) = run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap()]);
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

    // Self-host first so containment has something to resolve against.
    let (ok, _out, err) = run(store_dir.path(), &["migrate", plan_set.to_str().unwrap()]);
    assert!(ok, "self-host dispatches cleanly:\n{err}");

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", plan_set.to_str().unwrap(), "--artifacts"]);
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
    run(store_dir.path(), &["migrate", plan_set.to_str().unwrap()]);

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", plan_set.to_str().unwrap(), "--artifacts", "--dry-run"]);
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

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", dev.path().to_str().unwrap(), "--notes"]);
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

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", dev.path().to_str().unwrap(), "--notes", "--dry-run"]);
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
    run(store_dir.path(), &["migrate", plan_set.to_str().unwrap()]);
    run(store_dir.path(), &["migrate", plan_set.to_str().unwrap(), "--artifacts"]);

    let (ok, out, _err) =
        run(store_dir.path(), &["migrate", plan_set.to_str().unwrap(), "--artifacts"]);
    assert!(ok);
    assert!(out.contains("nothing to mint"), "a second run reports nothing left:\n{out}");

    let dev = TempDir::new().unwrap();
    std::fs::write(dev.path().join("0001-a.md"), "# A\nbody\n").unwrap();
    run(store_dir.path(), &["migrate", dev.path().to_str().unwrap(), "--notes"]);
    let (ok, out, _err) =
        run(store_dir.path(), &["migrate", dev.path().to_str().unwrap(), "--notes"]);
    assert!(ok);
    assert!(out.contains("nothing to mint"), "a second notes run reports nothing left:\n{out}");
}

#[test]
fn migrate_artifacts_and_notes_conflict() {
    assert!(
        Cli::try_parse_from(["odm", "migrate", "x", "--artifacts", "--notes"]).is_err(),
        "--artifacts and --notes are mutually exclusive"
    );
    assert!(
        Cli::try_parse_from(["odm", "migrate", "x", "--coverage", "--artifacts"]).is_err(),
        "--coverage and --artifacts are mutually exclusive"
    );
}

// ----- s13: `migrate --vision` re-casts the project as the vision synthesis -

const VISION_PLAN_BODY: &str = "\
# Vision CLI Test — Plan

## 1. Definition of done

odm ships when the corpus is faithful and the vision is live.

## 2. Scope

Everything.
";

fn write_vision_plan_set(root: &Path) {
    std::fs::write(root.join(".git"), "gitdir: fake\n").unwrap();
    std::fs::write(root.join("project-plan.md"), VISION_PLAN_BODY).unwrap();
}

#[test]
fn migrate_vision_recasts_the_project_as_the_synthesis() {
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    write_vision_plan_set(plan_root.path());

    let (ok, _out, err) = run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap()]);
    assert!(ok, "self-host dispatches cleanly:\n{err}");

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap(), "--vision"]);
    assert!(ok, "migrate --vision dispatches cleanly:\n{err}");
    assert!(err.contains("project re-cast"), "status names what happened:\n{err}");
    assert!(out.contains("vision synthesis"), "the report names the re-cast:\n{out}");

    let store = Store::open(store_dir.path());
    let all = store.load_all().unwrap();
    let plan_1to1 = all
        .iter()
        .find(|d| d.frontmatter().number() == 1001)
        .expect("the 1:1 project-plan node was minted");
    assert_eq!(plan_1to1.body(), VISION_PLAN_BODY, "faithful 1:1 body");
    assert!(
        plan_1to1.frontmatter().source().unwrap().synthesis.is_none(),
        "not itself a synthesis"
    );

    let project = all
        .iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Project)
        .expect("project node present");
    let source = project.frontmatter().source().expect("source present");
    assert_eq!(source.synthesis.as_deref(), Some("editorial-merge"));
    assert!(source.attestation.is_some());
    assert_eq!(project.frontmatter().edges().supersedes.len(), 1);
    assert_eq!(project.frontmatter().edges().supersedes[0].node, plan_1to1.frontmatter().id());
    assert!(project.body().contains("faithful and the vision is live"));
    assert!(
        project.body().starts_with("# Vision\n"),
        "carries the literal heading check/orient's L-3a lookup requires: {:?}",
        project.body()
    );
}

#[test]
fn migrate_vision_recast_body_satisfies_the_no_vision_check() {
    // The re-cast body must not just contain the distilled text (asserted
    // above) but must do so under a literal `# Vision` heading — otherwise
    // `odm check`'s `no-vision` warning fires on the very node built to
    // carry the vision (a real regression this test guards against).
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    write_vision_plan_set(plan_root.path());
    run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap()]);
    run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap(), "--vision"]);

    let (_code, out) = run_code(store_dir.path(), &["check"]);
    assert!(
        !out.contains("no-vision"),
        "no-vision warning must not fire on the re-cast project:\n{out}"
    );
}

#[test]
fn migrate_vision_dry_run_writes_nothing() {
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    write_vision_plan_set(plan_root.path());
    run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap()]);

    let (ok, _out, err) = run(
        store_dir.path(),
        &["migrate", plan_root.path().to_str().unwrap(), "--vision", "--dry-run"],
    );
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(err.contains("nothing written"), "status names dry-run:\n{err}");

    let store = Store::open(store_dir.path());
    let all = store.load_all().unwrap();
    assert!(all.iter().all(|d| d.frontmatter().number() != 1001), "no 1:1 node minted");
    let project = all.iter().find(|d| d.frontmatter().node_type() == NodeType::Project).unwrap();
    assert!(
        project.frontmatter().source().unwrap().synthesis.is_none(),
        "the project is not yet re-cast"
    );
}

#[test]
fn migrate_vision_is_idempotent() {
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    write_vision_plan_set(plan_root.path());
    run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap()]);
    run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap(), "--vision"]);

    let (ok, _out, err) =
        run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap(), "--vision"]);
    assert!(ok);
    assert!(err.contains("nothing to do"), "a second run is a no-op:\n{err}");

    let store = Store::open(store_dir.path());
    let plan_1to1_count =
        store.load_all().unwrap().into_iter().filter(|d| d.frontmatter().number() == 1001).count();
    assert_eq!(plan_1to1_count, 1, "the second run did not mint a duplicate 1:1 node");
}

#[test]
fn migrate_vision_refreshes_a_synthesis_body_that_drifted_from_the_derivation() {
    // Simulates exactly what happened on the live `.worktrees/odm` corpus: a
    // synthesis minted before the `# Vision` heading fix landed, whose body
    // no longer matches what `vision_body` derives today. A re-run must
    // notice and repair it in place, not treat "already a synthesis" as
    // license to ignore drift forever.
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    write_vision_plan_set(plan_root.path());
    run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap()]);
    run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap(), "--vision"]);

    let store = Store::open(store_dir.path());
    let project = store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Project)
        .unwrap();
    let stale_body = project.body().trim_start_matches("# Vision\n\n").to_string();
    let original_id = project.frontmatter().id();
    let original_supersedes = project.frontmatter().edges().supersedes.clone();
    store.persist(&Document::new(project.frontmatter().clone(), stale_body)).unwrap();

    let (ok, out, err) =
        run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap(), "--vision"]);
    assert!(ok, "dispatches cleanly:\n{err}");
    assert!(err.contains("refreshed"), "status names the refresh, not a no-op:\n{err}");
    assert!(out.contains("refreshed"), "the report names it too:\n{out}");

    let refreshed = store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Project)
        .unwrap();
    assert!(
        refreshed.body().starts_with("# Vision\n"),
        "the heading is restored: {:?}",
        refreshed.body()
    );
    assert_eq!(refreshed.frontmatter().id(), original_id, "identity preserved");
    assert_eq!(
        refreshed.frontmatter().edges().supersedes,
        original_supersedes,
        "lineage preserved"
    );

    // Idempotent from here: a third run over the now-correct body is a no-op.
    let (_ok, _out, err) =
        run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap(), "--vision"]);
    assert!(err.contains("nothing to do"), "settles once the body matches: {err}");
}

#[test]
fn migrate_vision_errors_when_no_project_node_exists() {
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    write_vision_plan_set(plan_root.path());

    let cli =
        Cli::try_parse_from(["odm", "migrate", plan_root.path().to_str().unwrap(), "--vision"])
            .unwrap();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, store_dir.path(), &mut out, &mut err);
    let message =
        result.expect_err("no project node exists yet — --vision must refuse").to_string();
    assert!(message.contains("no project node found"), "names the real problem:\n{message}");
}

#[test]
fn migrate_vision_preserves_the_projects_tags_and_component() {
    let store_dir = TempDir::new().unwrap();
    let plan_root = TempDir::new().unwrap();
    write_vision_plan_set(plan_root.path());
    run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap()]);

    let store = Store::open(store_dir.path());
    let project = store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Project)
        .expect("self-host minted the project");
    let tagged_fm = project
        .frontmatter()
        .clone()
        .with_tags(vec!["v1.0.0".to_string()])
        .with_component("odm".to_string());
    store.persist(&Document::new(tagged_fm, project.body().to_string())).unwrap();

    let (ok, _out, err) =
        run(store_dir.path(), &["migrate", plan_root.path().to_str().unwrap(), "--vision"]);
    assert!(ok, "migrate --vision dispatches cleanly:\n{err}");

    let all = store.load_all().unwrap();
    let plan_1to1 = all
        .iter()
        .find(|d| d.frontmatter().number() == 1001)
        .expect("the 1:1 project-plan node was minted");
    assert_eq!(plan_1to1.frontmatter().tags(), ["v1.0.0"], "tags carried onto the 1:1 node");
    assert_eq!(
        plan_1to1.frontmatter().component(),
        Some("odm"),
        "component carried onto the 1:1 node"
    );
}

// ----- N-2: `odm check` is green on the migrated real ODD corpus -------------

#[test]
fn check_green_on_migrated_odm_docs() {
    // Import odm's real `docs/design` into a fresh store (migrate never mutates
    // the legacy tree — proven in odm-migrate's never-delete test), then assert
    // `odm check` is green (exit 0) on the imported document graph.
    let store_dir = TempDir::new().unwrap();
    let (ok, _out, err) = run(store_dir.path(), &["migrate", real_docs().to_str().unwrap()]);
    assert!(ok, "migrate real docs dispatches cleanly:\n{err}");
    assert!(err.contains("created"), "some ODDs imported:\n{err}");

    let (code, out) = run_code(store_dir.path(), &["validate"]);
    assert_eq!(code, Some(0), "check is green on the imported odd corpus:\n{out}");
}
