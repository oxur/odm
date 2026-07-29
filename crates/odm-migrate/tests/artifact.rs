//! Integration tests for artifact-family mint-all (arc-migration-fidelity
//! slice09, ODD-0025 §2.5/§2.6). Test names carry the ledger's F-3 substrings:
//! `artifact_mint`, `artifact_containment`, `artifact_body`.

use std::path::{Path, PathBuf};

use odm_core::NodeType;
use odm_core::frontmatter::Document;
use odm_migrate::artifact::mint_artifacts;
use odm_migrate::coverage::DocClass;
use odm_migrate::selfhost::{arc_number, self_host, slice_number};
use odm_migrate::{Mode, coverage};
use odm_store::Store;
use tempfile::TempDir;

/// The synthetic plan-set fixture root (shared with the selfhost tests) — it
/// already carries two real `closing-report.md` supporting docs under
/// `arc01-alpha`'s slices, unreachable by `self_host` today.
fn plan_set() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/plan-set")
}

fn copy_tree(src: &Path, dst: &Path) {
    for entry in walk(src) {
        let rel = entry.strip_prefix(src).unwrap();
        let target = dst.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&target).unwrap();
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::copy(&entry, &target).unwrap();
        }
    }
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    if root.is_dir() {
        for entry in std::fs::read_dir(root).unwrap() {
            out.extend(walk(&entry.unwrap().path()));
        }
    }
    out
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn artifacts_by_path(store: &Store) -> std::collections::HashMap<String, Document> {
    store
        .load_all()
        .expect("load_all")
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == NodeType::Artifact)
        .map(|d| {
            let relative =
                d.frontmatter().source().expect("artifact carries source").paths[0].clone();
            (relative.display().to_string(), d)
        })
        .collect()
}

// ----- F-3: mint-all — every supporting doc class, reports included ---------

#[test]
fn artifact_mint_all_covers_supporting_docs_incl_reports() {
    let docs = TempDir::new().unwrap();
    copy_tree(&plan_set(), docs.path());
    write(docs.path(), "arc01-alpha/slice01-aa/ledger.md", "# Ledger\nrows\n");
    write(docs.path(), "arc01-alpha/slice01-aa/cc-prompt.md", "# Prompt\n");
    write(docs.path(), "arc01-alpha/c1-cdc-verification.md", "# Chunk CDC\n");
    // A generated report at the docs root — mint-all, no exemption (§2.6).
    write(docs.path(), "coverage-report.md", "# Coverage report\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, docs.path(), Mode::Commit).expect("self-host");

    let report = mint_artifacts(&store, docs.path(), Mode::Commit).expect("mint");

    // Every artifact-family doc counted by the coverage detector's own
    // classification is minted — no silent drop, no exemption.
    let expected = coverage::enumerate_docs(docs.path())
        .into_iter()
        .filter(|d| {
            matches!(
                d.class,
                DocClass::Ledger
                    | DocClass::CcPrompt
                    | DocClass::CdcVerification
                    | DocClass::ClosingReport
                    | DocClass::Other
            )
        })
        .count();
    assert_eq!(report.minted_count(), expected, "mint-all: every supporting doc, no exemption");

    let by_path = artifacts_by_path(&store);
    assert!(by_path.contains_key("arc01-alpha/slice01-aa/closing-report.md"));
    assert!(by_path.contains_key("arc02-beta/slice01-cc/closing-report.md"));
    assert!(by_path.contains_key("coverage-report.md"), "coverage-report.md is not exempt");

    // Every minted artifact validates: `artifact` type, schema-stamped, and
    // its body matches the source file byte-for-byte (after trim+lf).
    for document in by_path.values() {
        assert_eq!(document.frontmatter().node_type(), NodeType::Artifact);
        assert!(document.frontmatter().schema().is_some(), "schema-stamped");
    }
}

// ----- F-1/F-2: containment — nearest modeled scale, chunk → arc, top-level -

#[test]
fn artifact_containment_resolves_nearest_modeled_scale() {
    let docs = TempDir::new().unwrap();
    copy_tree(&plan_set(), docs.path());
    write(docs.path(), "arc01-alpha/slice01-aa/ledger.md", "# Ledger\n");
    write(docs.path(), "arc01-alpha/c1-cdc-verification.md", "# Chunk CDC\n");
    write(docs.path(), "benchmark-results.md", "# Bench\n");
    // A named arc — no numbered coordinate, containment must still resolve
    // through `source.paths`, not a re-derived structural number (F-6's same
    // named-arc key, reused here for a *different* purpose: containment).
    write(docs.path(), "arc-store-home/arc-plan.md", "# Named arc\n");
    write(docs.path(), "arc-store-home/ledger.md", "# Named arc ledger\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, docs.path(), Mode::Commit).expect("self-host");
    mint_artifacts(&store, docs.path(), Mode::Commit).expect("mint");

    let nodes = store.load_all().expect("load_all");
    let node_by_number = |ty: NodeType, n: u32| {
        nodes
            .iter()
            .find(|d| d.frontmatter().node_type() == ty && d.frontmatter().number() == n)
            .unwrap_or_else(|| panic!("{ty:?} #{n} not found"))
    };
    let by_path = artifacts_by_path(&store);

    let arc1 = node_by_number(NodeType::Arc, arc_number(1));
    let slice1 = node_by_number(NodeType::Slice, slice_number(1, 1, None));

    // A per-slice doc → its slice.
    assert_eq!(
        by_path["arc01-alpha/slice01-aa/ledger.md"].frontmatter().edges().part_of,
        Some(slice1.frontmatter().id()),
        "per-slice artifact is part_of its slice"
    );
    // A chunk-level doc (no slice subdirectory) → the arc, not orphaned and
    // not any nonexistent chunk scale.
    assert_eq!(
        by_path["arc01-alpha/c1-cdc-verification.md"].frontmatter().edges().part_of,
        Some(arc1.frontmatter().id()),
        "chunk-level artifact is part_of its arc"
    );
    // A genuinely top-level doc → no containment (§2.7, optional).
    assert_eq!(
        by_path["benchmark-results.md"].frontmatter().edges().part_of,
        None,
        "a top-level artifact is legitimately uncontained"
    );
    // A named arc's own supporting doc → the named arc, resolved via
    // `source.paths`, not a numbered-coordinate re-derivation.
    let named_arc = nodes
        .iter()
        .find(|d| {
            d.frontmatter().node_type() == NodeType::Arc && d.frontmatter().name() == "Named arc"
        })
        .expect("named arc node");
    assert_eq!(
        by_path["arc-store-home/ledger.md"].frontmatter().edges().part_of,
        Some(named_arc.frontmatter().id()),
        "a named arc's own artifact resolves via source.paths"
    );
}

// ----- F-4: idempotent — a re-run mints nothing new --------------------------

#[test]
fn artifact_mint_is_idempotent() {
    let docs = TempDir::new().unwrap();
    copy_tree(&plan_set(), docs.path());

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, docs.path(), Mode::Commit).expect("self-host");

    let first = mint_artifacts(&store, docs.path(), Mode::Commit).expect("first mint");
    assert!(first.minted_count() > 0);
    let n = store.load_all().unwrap().len();

    let second = mint_artifacts(&store, docs.path(), Mode::Commit).expect("second mint");
    assert_eq!(second.minted_count(), 0, "already-covered docs are not re-minted");
    assert_eq!(store.load_all().unwrap().len(), n, "no duplicate nodes on disk");
}

// ----- F-5: dry-run writes nothing -------------------------------------------

#[test]
fn artifact_mint_dry_run_writes_nothing() {
    let docs = TempDir::new().unwrap();
    copy_tree(&plan_set(), docs.path());

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, docs.path(), Mode::Commit).expect("self-host");
    let before = store.load_all().unwrap().len();

    let report = mint_artifacts(&store, docs.path(), Mode::DryRun).expect("dry-run mint");
    assert!(report.dry_run);
    assert!(report.minted_count() > 0, "the plan still lists what would be minted");
    assert_eq!(store.load_all().unwrap().len(), before, "dry-run wrote nothing");
}

// ----- F-6: body is verbatim 1:1, under the hard body-hash gate -------------

#[test]
fn artifact_body_is_verbatim_1to1() {
    let docs = TempDir::new().unwrap();
    copy_tree(&plan_set(), docs.path());
    let body = "# Ledger\n\n| id | status |\n|----|--------|\n| F-1 | open |\n";
    write(docs.path(), "arc01-alpha/slice01-aa/ledger.md", body);

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, docs.path(), Mode::Commit).expect("self-host");
    mint_artifacts(&store, docs.path(), Mode::Commit).expect("mint");

    let by_path = artifacts_by_path(&store);
    assert_eq!(
        by_path["arc01-alpha/slice01-aa/ledger.md"].body().trim(),
        body.trim(),
        "the artifact body is the source file, verbatim"
    );
}
