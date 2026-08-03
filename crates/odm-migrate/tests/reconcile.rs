//! Integration tests for [`reconcile`] (arc-migration-fidelity s12 F-1/F-2/
//! F-3/F-6): `repair`'s complement over the work-tree (`arc`/`slice`) family
//! — reconciling a node that **already carries** a `source`, whether because
//! its plan directory moved or because its plan doc's content legitimately
//! changed (the concrete live case: the *active* arc-plan.md, edited
//! throughout this very arc — s08's CDC verification).

use std::path::Path;

use odm_core::NodeType;
use odm_migrate::Mode;
use odm_migrate::selfhost::{reconcile, self_host};
use odm_store::Store;
use tempfile::TempDir;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn write_plan_set_with_git(root: &Path, arc_dir: &str, arc_body: &str) {
    std::fs::write(root.join(".git"), "gitdir: fake\n").unwrap();
    write(root, "docs/design-v1.0.0/project-plan.md", "# Test Project\n");
    write(root, &format!("docs/design-v1.0.0/{arc_dir}/arc-plan.md"), arc_body);
}

fn arc_node(store: &Store) -> odm_core::frontmatter::Document {
    store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Arc)
        .expect("arc node")
}

// ----- F-1: a living arc-plan (the s08 CDC concrete case) reconciles -------

#[test]
fn reconcile_resnapshots_a_drifted_arc_plan_body() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path(), "arc01-alpha", "# Arc 01 — Alpha\n\nOriginal content.\n");
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_root, Mode::Commit).expect("self-host");
    let original_id = arc_node(&store).frontmatter().id();

    // The arc-plan.md keeps evolving, as every live arc-plan in this arc
    // actually does (bubble-ups, status-header updates, …).
    write(
        repo.path(),
        "docs/design-v1.0.0/arc01-alpha/arc-plan.md",
        "# Arc 01 — Alpha\n\nAmended content — the plan evolved.\n",
    );

    let report = reconcile(&store, &plan_root, Mode::Commit).expect("reconcile");
    assert_eq!(report.reconciled_count(), 1);
    assert!(!report.reconciled[0].path_moved);
    assert!(report.reconciled[0].body_drifted);

    let node = arc_node(&store);
    assert_eq!(node.frontmatter().id(), original_id, "same node, not re-minted");
    assert!(node.body().contains("Amended content"), "body re-snapshotted: {}", node.body());
}

// ----- F-2: an arc directory that moved is re-discovered by number --------

#[test]
fn reconcile_rediscovers_an_arc_whose_directory_moved() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path(), "arc01-alpha", "# Arc 01 — Alpha\n\nOriginal content.\n");
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_root, Mode::Commit).expect("self-host");
    let original_id = arc_node(&store).frontmatter().id();

    // The arc directory is renamed — same `arcNN` coordinate (still arc01),
    // different slug — the mechanism a moved design/ODD file (s11 L-8b) also
    // hits, for the work-tree family.
    std::fs::rename(
        repo.path().join("docs/design-v1.0.0/arc01-alpha"),
        repo.path().join("docs/design-v1.0.0/arc01-renamed"),
    )
    .unwrap();

    let report = reconcile(&store, &plan_root, Mode::Commit).expect("reconcile");
    assert_eq!(report.reconciled_count(), 1);
    assert!(report.reconciled[0].path_moved, "the directory rename is detected");

    let node = arc_node(&store);
    assert_eq!(node.frontmatter().id(), original_id);
    let stored = node.frontmatter().source().unwrap().paths[0].to_string_lossy().into_owned();
    assert_eq!(stored, "docs/design-v1.0.0/arc01-renamed/arc-plan.md");
}

// ----- arc-store-as-source slice02: an authored node is never re-fidelity-
// ----- checked, even when its plan-tree counterpart keeps drifting --------

#[test]
fn reconcile_never_touches_an_authored_node_even_as_its_plan_tree_file_drifts() {
    use odm_core::Origin;
    use odm_core::frontmatter::{Document, Frontmatter, Source};

    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path(), "arc01-alpha", "# Arc 01 — Alpha (docs, stale)\n");
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    // An authored arc at the coordinate self_host's discovery would compute —
    // ODD-0026 §2.1's shape: `origin: authored`, no external `source.paths`.
    let arc_id = odm_core::Id::new();
    let created = chrono::NaiveDate::from_ymd_opt(2026, 7, 20).unwrap();
    let mut fm = Frontmatter::new(
        arc_id,
        1100,
        NodeType::Arc,
        "Arc 01 — Alpha",
        created,
        created,
        Origin::Authored,
    );
    fm.stamp_schema();
    let fm = fm.with_source(Source::authored(Vec::new()));
    store
        .persist(&Document::new(fm, "# Arc 01 — Alpha\n\nAuthored, store-owned.\n".to_string()))
        .unwrap();

    self_host(&store, &plan_root, Mode::Commit)
        .expect("self-host recognizes it, mints only the project");

    // The plan-tree file keeps drifting, as every legacy arc-plan.md does
    // until the slice04 cutover — this must never leak back into the
    // authored node.
    write(
        repo.path(),
        "docs/design-v1.0.0/arc01-alpha/arc-plan.md",
        "# Arc 01 — Alpha\n\nAmended in ./docs — must never land in the authored node.\n",
    );

    let report = reconcile(&store, &plan_root, Mode::Commit).expect("reconcile");
    assert_eq!(report.reconciled_count(), 0, "an authored node is never reconciled against ./docs");

    let node = arc_node(&store);
    assert_eq!(node.frontmatter().id(), arc_id, "same node");
    assert_eq!(node.frontmatter().origin(), Origin::Authored, "origin untouched");
    assert_eq!(
        node.body(),
        "# Arc 01 — Alpha\n\nAuthored, store-owned.\n",
        "body never overwritten from the drifting ./docs file"
    );
    assert!(
        node.frontmatter().source().is_some_and(|s| s.is_authored()),
        "source stays the authored shape, never rebuilt as a migration record"
    );
}

// ----- F-3: the living-plan-node policy — reconcile twice, no rejection ---

#[test]
fn reconcile_reconciles_cleanly_across_repeated_arc_plan_edits() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path(), "arc01-alpha", "# Arc 01 — Alpha\n\nV1.\n");
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_root, Mode::Commit).expect("self-host");

    write(repo.path(), "docs/design-v1.0.0/arc01-alpha/arc-plan.md", "# Arc 01 — Alpha\n\nV2.\n");
    let first = reconcile(&store, &plan_root, Mode::Commit).expect("first reconcile");
    assert_eq!(first.reconciled_count(), 1);
    assert!(arc_node(&store).body().contains("V2"));

    write(repo.path(), "docs/design-v1.0.0/arc01-alpha/arc-plan.md", "# Arc 01 — Alpha\n\nV3.\n");
    let second = reconcile(&store, &plan_root, Mode::Commit).expect("second reconcile");
    assert_eq!(
        second.reconciled_count(),
        1,
        "the second edit reconciles cleanly too, no rejection"
    );
    assert!(arc_node(&store).body().contains("V3"));
}

// ----- F-6: idempotent + dry-run-safe ---------------------------------------

#[test]
fn reconcile_is_idempotent_and_dry_run_writes_nothing() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path(), "arc01-alpha", "# Arc 01 — Alpha\n\nOriginal.\n");
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_root, Mode::Commit).expect("self-host");
    write(
        repo.path(),
        "docs/design-v1.0.0/arc01-alpha/arc-plan.md",
        "# Arc 01 — Alpha\n\nAmended.\n",
    );

    let dry = reconcile(&store, &plan_root, Mode::DryRun).expect("dry-run");
    assert!(dry.dry_run);
    assert_eq!(dry.reconciled_count(), 1);
    assert!(arc_node(&store).body().contains("Original"), "dry-run wrote nothing");

    reconcile(&store, &plan_root, Mode::Commit).expect("commit");
    let second = reconcile(&store, &plan_root, Mode::Commit).expect("second run");
    assert_eq!(second.reconciled_count(), 0, "already reconciled — nothing left to do");
}

// ----- s15 F-3: the project reconciles like any other plan node ------------

#[test]
fn reconcile_reconciles_a_drifted_project_body() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path(), "arc01-alpha", "# Arc 01 — Alpha\n\nOriginal.\n");
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_root, Mode::Commit).expect("self-host");
    let original_id = project_node(&store).frontmatter().id();
    write(repo.path(), "docs/design-v1.0.0/project-plan.md", "# Test Project\n\nAmended.\n");

    let report = reconcile(&store, &plan_root, Mode::Commit).expect("reconcile");
    assert!(
        report.reconciled.iter().any(|r| r.node_type == NodeType::Project),
        "the faithful 1:1 project reconciles like any other plan node: {:?}",
        report.reconciled
    );

    let project = project_node(&store);
    assert_eq!(project.frontmatter().id(), original_id, "same node, not re-minted");
    assert!(project.body().contains("Amended"), "project body re-snapshotted: {}", project.body());
}

// ----- a `source.synthesis`-bearing project stays excluded (ODD-0025 §2.3) -

#[test]
fn reconcile_excludes_a_synthesis_bearing_project() {
    let repo = TempDir::new().unwrap();
    write_plan_set_with_git(repo.path(), "arc01-alpha", "# Arc 01 — Alpha\n\nOriginal.\n");
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_root, Mode::Commit).expect("self-host");

    // Re-cast the project as a synthesis in place (mirrors what the
    // vision-apply mechanism used to do), then drift the real source.
    let mut project = project_node(&store);
    let fm = project.frontmatter_mut();
    *fm = fm.clone().with_source(odm_core::frontmatter::Source {
        paths: vec![Path::new("docs/design-v1.0.0/project-plan.md").to_path_buf()],
        class: "vision".to_string(),
        normalization: Some("trim+lf".to_string()),
        migrated_by: Some("odm-migrate/test".to_string()),
        migrated_on: Some(fm.updated()),
        synthesis: Some("editorial-merge".to_string()),
        attestation: Some("operator: distills the source".to_string()),
        migrated_from: Vec::new(),
    });
    store.persist(&project).unwrap();
    write(repo.path(), "docs/design-v1.0.0/project-plan.md", "# Test Project\n\nAmended.\n");

    let report = reconcile(&store, &plan_root, Mode::Commit).expect("reconcile");
    assert!(
        report.reconciled.iter().all(|r| r.node_type != NodeType::Project),
        "a source.synthesis-bearing project is never reconciled: {:?}",
        report.reconciled
    );
}

fn project_node(store: &Store) -> odm_core::frontmatter::Document {
    store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Project)
        .expect("project node")
}
