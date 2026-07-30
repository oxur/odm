//! Integration test for the project-vision re-cast (arc-migration-fidelity
//! s11 F-6, MF-7): the vision becomes an **editorial-merge synthesis**
//! superseding a **1:1 `project-plan` node**, replacing the bespoke
//! `replan::vision_from_plan` + body-patch mechanism with the modeled
//! synthesis capability (`odm_migrate::synthesis`).
//!
//! **Fixture only — no live store write.** This proves the *mechanism*: a
//! faithful 1:1 project node plus a synthesis node that supersedes it with
//! intact lineage + a recorded attestation. The live numbering/identity-
//! continuity policy (does the synthesis inherit `#1000`, does the 1:1 node
//! move to a new number, …) is s12's live-application decision, out of this
//! slice's scope — this test assigns the two nodes distinct numbers purely
//! to keep the fixture's identities unambiguous.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use odm_core::check;
use odm_core::frontmatter::{Frontmatter, SupersedeKind};
use odm_core::{Id, NodeType};
use odm_migrate::fidelity::verify_body_hash;
use odm_migrate::replan::vision_from_plan;
use odm_migrate::synthesis::{
    Attestation, SynthesisSource, SynthesisType, apply_project_vision, build_synthesis,
};
use odm_migrate::{Mode, selfhost::self_host};
use odm_store::Store;
use tempfile::TempDir;

const PROJECT_PLAN_BODY: &str = "\
# Vision Test — Plan

## 1. Definition of done

odm ships when every migrated node carries a source record and `check` is\n\
green on the whole corpus, with no file left behind.

## 2. Scope

Everything under `docs/design-v1.0.0/`.
";

fn write_plan_set(root: &Path) {
    std::fs::write(root.join(".git"), "gitdir: fake\n").unwrap();
    let plan = root.join("docs/design-v1.0.0");
    std::fs::create_dir_all(&plan).unwrap();
    std::fs::write(plan.join("project-plan.md"), PROJECT_PLAN_BODY).unwrap();
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, 29).unwrap()
}

#[test]
fn vision_becomes_an_editorial_merge_synthesis_over_a_1to1_project_plan_node() {
    let repo = TempDir::new().unwrap();
    write_plan_set(repo.path());
    let plan_root = repo.path().join("docs/design-v1.0.0");

    // Step 1: the ordinary 1:1 migration — `self_host` mints the project
    // node with the **verbatim** project-plan.md body, no vision injection
    // (that bespoke patch lives only in `replan::restamp`, not here).
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_root, Mode::Commit).expect("self-host");

    let all = store.load_all().expect("load_all");
    let plan_node = all
        .iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Project)
        .expect("project node");
    assert_eq!(plan_node.body().trim(), PROJECT_PLAN_BODY.trim(), "1:1, faithful body");
    let plan_node_id = plan_node.frontmatter().id();
    let plan_node_number = plan_node.frontmatter().number();

    // Step 2: extract the vision text — the same distillation
    // `replan::vision_from_plan` already performs; only what happens to it
    // next changes (a modeled synthesis node, not a body patch).
    let vision_body = vision_from_plan(&plan_root).expect("a Definition-of-done section exists");
    assert!(vision_body.contains("no file left behind"));

    // Step 3: build the synthesis node — editorial-merge (the vision is a
    // human distillation, not a byte-for-byte join, so it is not hash-
    // checkable), superseding the 1:1 node, with a recorded attestation.
    let source = SynthesisSource {
        path: PathBuf::from("docs/design-v1.0.0/project-plan.md"),
        body: PROJECT_PLAN_BODY.to_string(),
        node: plan_node_id,
    };
    let attestation = Attestation {
        by: "CC".to_string(),
        statement: "distills project-plan.md §1 (Definition of done) verbatim".to_string(),
        on: day(),
    };
    let vision_fm = build_synthesis(
        Id::new(),
        plan_node_number + 1,
        NodeType::Project,
        "Vision".to_string(),
        day(),
        day(),
        &vision_body,
        std::slice::from_ref(&source),
        SynthesisType::EditorialMerge,
        SupersedeKind::Updates,
        Some(&attestation),
        "vision",
        day(),
    )
    .expect("editorial-merge with lineage + attestation");

    // ----- F-6's exact criteria -----------------------------------------

    // "the vision becomes an editorial-merge synthesis"
    let vision_source = vision_fm.source().expect("source present");
    assert_eq!(vision_source.synthesis.as_deref(), Some("editorial-merge"));
    assert!(vision_source.attestation.is_some(), "recorded, never a silent pass");

    // "superseding a 1:1 project-plan node"
    assert_eq!(vision_fm.edges().supersedes.len(), 1);
    assert_eq!(vision_fm.edges().supersedes[0].node, plan_node_id);
    assert_eq!(vision_fm.edges().supersedes[0].kind, SupersedeKind::Updates);

    // "(faithful body = project-plan.md, hard-gated)" — the *source* the
    // synthesis supersedes is itself still verified faithful; the gate is
    // on the 1:1 node's own migration (step 1), unchanged by synthesizing
    // from it.
    verify_body_hash(PROJECT_PLAN_BODY, plan_node.body(), "1:1 project-plan node")
        .expect("the superseded node is still a faithful 1:1 copy");

    // Bidirectional-lineage integrity (s11 F-2): a `check` run over both
    // nodes together is green — the forward edge resolves, no cycle, no
    // self-supersede.
    let corpus: Vec<Frontmatter> = vec![vision_fm.clone(), plan_node.frontmatter().clone()];
    let findings = check::check(&corpus);
    assert!(findings.is_empty(), "clean lineage: {findings:?}");

    // "no live store write" — this whole synthesis lives only in memory;
    // confirm the store on disk still holds exactly the one node self_host
    // minted (the synthesis was never persisted).
    let after = store.load_all().expect("load_all");
    assert_eq!(after.len(), all.len(), "store untouched by the in-memory synthesis");
}

// ----- s12 F-4: the promoted, reusable apply_project_vision path -----------

#[test]
fn apply_project_vision_yields_the_synthesis_over_the_faithful_1to1_node() {
    let repo = TempDir::new().unwrap();
    write_plan_set(repo.path());
    let plan_root = repo.path().join("docs/design-v1.0.0");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &plan_root, Mode::Commit).expect("self-host");

    let all = store.load_all().expect("load_all");
    let plan_node = all
        .iter()
        .find(|d| d.frontmatter().node_type() == NodeType::Project)
        .expect("project node");
    let plan_source = plan_node.frontmatter().source().expect("1:1 node carries source");
    let plan_source_path = plan_source.paths[0].clone();

    let attestation = Attestation {
        by: "CC".to_string(),
        statement: "distills project-plan.md §1 (Definition of done) verbatim".to_string(),
        on: day(),
    };
    let (vision_fm, vision_body) = apply_project_vision(
        plan_node.frontmatter().id(),
        plan_source_path.clone(),
        plan_node.body(),
        &plan_root,
        Id::new(),
        plan_node.frontmatter().number() + 1,
        day(),
        day(),
        attestation,
    )
    .expect("apply_project_vision");

    // Same criteria as the manual mechanism above, now exercised through the
    // promoted, reusable function every future caller (s13) actually calls.
    assert!(vision_body.contains("no file left behind"));
    assert!(
        vision_body.starts_with("# Vision\n"),
        "carries the literal heading check/orient's L-3a lookup requires: {vision_body:?}"
    );
    let vision_source = vision_fm.source().expect("source present");
    assert_eq!(vision_source.synthesis.as_deref(), Some("editorial-merge"));
    assert!(vision_source.attestation.is_some());
    assert_eq!(vision_source.paths[0], plan_source_path, "records the 1:1 node's own source path");
    assert_eq!(vision_fm.edges().supersedes.len(), 1);
    assert_eq!(vision_fm.edges().supersedes[0].node, plan_node.frontmatter().id());
    assert_eq!(vision_fm.edges().supersedes[0].kind, SupersedeKind::Updates);

    let corpus: Vec<Frontmatter> = vec![vision_fm, plan_node.frontmatter().clone()];
    assert!(check::check(&corpus).is_empty(), "clean lineage");

    // No live write: the store still holds only what self_host minted.
    let after = store.load_all().expect("load_all");
    assert_eq!(after.len(), all.len());
}

#[test]
fn apply_project_vision_errors_when_the_plan_has_no_vision_section() {
    let repo = TempDir::new().unwrap();
    std::fs::write(repo.path().join(".git"), "gitdir: fake\n").unwrap();
    let plan_root = repo.path().join("docs/design-v1.0.0");
    std::fs::create_dir_all(&plan_root).unwrap();
    // No "## Definition of done" section at all.
    std::fs::write(
        plan_root.join("project-plan.md"),
        "# No Vision Here\n\n## Scope\n\nJust scope.\n",
    )
    .unwrap();

    let attestation = Attestation { by: "CC".to_string(), statement: "n/a".to_string(), on: day() };
    let err = apply_project_vision(
        Id::new(),
        PathBuf::from("docs/design-v1.0.0/project-plan.md"),
        "irrelevant",
        &plan_root,
        Id::new(),
        1,
        day(),
        day(),
        attestation,
    )
    .unwrap_err();
    assert!(matches!(err, odm_migrate::synthesis::VisionApplyError::NoVisionText(_)));
}
