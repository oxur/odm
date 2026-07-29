//! Integration tests for the coverage/gap detector (arc-migration-fidelity
//! slice01). Test names carry the substrings the slice01 ledger Verify commands
//! filter on: `coverage_classify`, `coverage_doc_coverage`,
//! `coverage_representation`, `coverage_stubs`, `coverage_provenance_absence`,
//! plus `coverage_is_read_only` for F-7.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter, Source};
use odm_core::{Id, NodeType, Origin};
use odm_migrate::Mode;
use odm_migrate::coverage::{self, DocClass};
use odm_migrate::selfhost::{arc_number, named_arc_number, self_host, slice_number};
use odm_store::Store;
use tempfile::TempDir;

/// The synthetic plan-set fixture root (shared with the selfhost tests).
fn plan_set() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/plan-set")
}

/// The synthetic legacy ODD fixture root (shared with the migrate tests).
fn legacy_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/legacy")
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
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

/// A minimal, directly-persisted node — bypassing `self_host`/`migrate` so
/// these tests can construct the exact store shape a detector needs without
/// depending on the derivations under test elsewhere.
#[allow(clippy::too_many_arguments)]
fn persist_node(
    store: &Store,
    number: u32,
    node_type: NodeType,
    name: &str,
    body: &str,
    retired: bool,
    source: Option<Source>,
) {
    let today = NaiveDate::from_ymd_opt(2026, 7, 27).unwrap();
    let mut fm =
        Frontmatter::new(Id::new(), number, node_type, name, today, today, Origin::Planned);
    if retired {
        fm.retire("superseded", today);
    }
    if let Some(source) = source {
        fm = fm.with_source(source);
    }
    let document = Document::new(fm, body.to_string());
    store.persist(&document).expect("persist");
}

// ----- F-2: classification covers every doc under the root, unfiltered ------

#[test]
fn coverage_classify() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();

    write(root, "design-v1.0.0/project-plan.md", "# Plan\n");
    write(root, "design-v1.0.0/arc01-alpha/arc-plan.md", "# Arc\n");
    write(root, "design-v1.0.0/arc01-alpha/slice01-aa/slice-doc.md", "# Slice\n");
    write(root, "design-v1.0.0/arc01-alpha/slice01-aa/ledger.md", "# Ledger\n");
    write(root, "design-v1.0.0/arc01-alpha/slice01-aa/cc-prompt.md", "# Prompt\n");
    write(root, "design-v1.0.0/arc01-alpha/slice01-aa/cc-prompt-amendment.md", "# Amendment\n");
    write(root, "design-v1.0.0/arc01-alpha/slice01-aa/cdc-verification.md", "# CDC\n");
    write(root, "design-v1.0.0/arc01-alpha/slice01-aa/closing-report.md", "# Closed\n");
    write(root, "design-v1.0.0/arc01-alpha/c1-cdc-verification.md", "# Chunk CDC\n");
    write(root, "design-v1.0.0/arc01-alpha/c1-closing-report.md", "# Chunk closed\n");
    write(root, "design-v1.0.0/benchmark-results.md", "# Bench\n");
    write(root, "design/01-draft/0001-x.md", "---\nnumber: 1\n---\nbody\n");
    write(root, "design/index.md", "# Design Index\n");
    write(root, "dev/0001-note.md", "# Dev note\n");
    write(root, "dev/research/0001-survey.md", "# Survey\n");

    let docs_list = coverage::enumerate_docs(root);

    // Total matches an unfiltered walk — the F-2 Verify's `find … | wc -l`.
    let expected_total = walk(root).into_iter().filter(|p| p.is_file()).count();
    assert_eq!(docs_list.len(), expected_total, "every .md under root counted, none excluded");

    let class_of = |rel: &str| {
        docs_list
            .iter()
            .find(|d| d.path == Path::new(rel))
            .unwrap_or_else(|| panic!("{rel} not enumerated"))
            .class
    };
    assert_eq!(class_of("design-v1.0.0/project-plan.md"), DocClass::ProjectPlan);
    assert_eq!(class_of("design-v1.0.0/arc01-alpha/arc-plan.md"), DocClass::ArcPlan);
    assert_eq!(class_of("design-v1.0.0/arc01-alpha/slice01-aa/slice-doc.md"), DocClass::SliceDoc);
    assert_eq!(class_of("design-v1.0.0/arc01-alpha/slice01-aa/ledger.md"), DocClass::Ledger);
    assert_eq!(class_of("design-v1.0.0/arc01-alpha/slice01-aa/cc-prompt.md"), DocClass::CcPrompt);
    assert_eq!(
        class_of("design-v1.0.0/arc01-alpha/slice01-aa/cc-prompt-amendment.md"),
        DocClass::CcPrompt,
        "cc-prompt-* variants classify as CcPrompt too"
    );
    assert_eq!(
        class_of("design-v1.0.0/arc01-alpha/slice01-aa/cdc-verification.md"),
        DocClass::CdcVerification
    );
    assert_eq!(
        class_of("design-v1.0.0/arc01-alpha/slice01-aa/closing-report.md"),
        DocClass::ClosingReport
    );
    assert_eq!(
        class_of("design-v1.0.0/arc01-alpha/c1-cdc-verification.md"),
        DocClass::CdcVerification,
        "chunk-scale cdc-verification variants still classify"
    );
    assert_eq!(
        class_of("design-v1.0.0/arc01-alpha/c1-closing-report.md"),
        DocClass::ClosingReport,
        "chunk-scale closing-report variants still classify"
    );
    assert_eq!(
        class_of("design-v1.0.0/benchmark-results.md"),
        DocClass::Other,
        "ad-hoc docs under design-v1.0.0 fall to Other"
    );
    assert_eq!(class_of("design/01-draft/0001-x.md"), DocClass::Odd);
    assert_eq!(
        class_of("design/index.md"),
        DocClass::Odd,
        "the design tree root is odd, even for non-ODD files"
    );
    assert_eq!(class_of("dev/0001-note.md"), DocClass::Dev);
    assert_eq!(class_of("dev/research/0001-survey.md"), DocClass::Research);
}

// ----- F-3: doc-coverage — matched vs. uncovered, basis always stated -------

#[test]
fn coverage_doc_coverage() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    copy_tree(&plan_set(), &root.join("design-v1.0.0"));
    // Supporting docs the plan-set fixture doesn't carry — added for full class
    // coverage (mirrors the real corpus's per-slice cohort).
    write(root, "design-v1.0.0/arc01-alpha/slice01-aa/ledger.md", "# Ledger\n");
    write(root, "design-v1.0.0/arc01-alpha/slice01-aa/cc-prompt.md", "# Prompt\n");
    write(root, "design-v1.0.0/arc01-alpha/slice01-aa/cdc-verification.md", "# CDC\n");

    // A matched ODD (#1) and an unmatched one (#99, frontmatter present but no
    // node) reusing the shared legacy fixture's real frontmatter shape.
    copy_tree(&legacy_fixture().join("01-draft"), &root.join("design/01-draft"));
    write(root, "design/04-accepted/0099-orphan.md", "---\nnumber: 99\ntitle: Orphan\n---\nbody\n");
    // A file with no parsable frontmatter under the ODD tree (an index page).
    write(root, "design/index.md", "# Design Index\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    self_host(&store, &root.join("design-v1.0.0"), Mode::Commit).expect("self-host");
    // #1 is the legacy fixture's real number (test-data/legacy/01-draft/0001-early-draft.md).
    persist_node(&store, 1, NodeType::Design, "Early draft", "body", false, None);

    let report = coverage::run(&store, root).expect("coverage run");
    let dc = &report.doc_coverage;

    // Total matches the unfiltered enumeration (F-2 threaded through F-3).
    assert_eq!(dc.total(), coverage::enumerate_docs(root).len());

    let entry = |rel: &str| {
        dc.entries
            .iter()
            .find(|e| e.path == Path::new(rel))
            .unwrap_or_else(|| panic!("{rel} not in the doc-coverage report"))
    };

    assert!(
        entry("design-v1.0.0/project-plan.md").covered,
        "project-plan matches the project root"
    );
    assert!(entry("design-v1.0.0/arc01-alpha/arc-plan.md").covered, "arc01 matches");
    assert!(
        entry("design-v1.0.0/arc07-horizon/arc-plan.md").covered,
        "arc-migration-fidelity slice04: the scope cap is removed — self_host now \
         imports arc07 too, so its arc-plan.md now matches a real node"
    );
    assert!(
        entry("design-v1.0.0/arc01-alpha/slice01-aa/slice-doc.md").covered,
        "arc01's slice matches"
    );
    for supporting in [
        "design-v1.0.0/arc01-alpha/slice01-aa/ledger.md",
        "design-v1.0.0/arc01-alpha/slice01-aa/cc-prompt.md",
        "design-v1.0.0/arc01-alpha/slice01-aa/cdc-verification.md",
        "design-v1.0.0/arc01-alpha/slice01-aa/closing-report.md",
    ] {
        assert!(!entry(supporting).covered, "{supporting}: no node class exists yet");
    }
    assert!(entry("design/01-draft/0001-early-draft.md").covered, "ODD #1 matches by number");
    assert!(
        !entry("design/04-accepted/0099-orphan.md").covered,
        "ODD #99 has a number but no matching node"
    );
    assert!(
        entry("design/index.md").covered,
        "an index page is excluded infrastructure, not a coverage gap (s10)"
    );

    // Every entry carries a non-empty basis — never presented without one.
    assert!(dc.entries.iter().all(|e| !e.basis.is_empty()));
}

// ----- s10: infrastructure files (index/templates) are excluded, not gaps --

#[test]
fn coverage_excludes_index_and_template_files() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    write(root, "design/index.md", "# Design Index\n");
    write(root, "design/templates/design-doc-template.md", "# Template\n");
    write(root, "design/01-draft/0001-real.md", "---\nnumber: 1\n---\nbody\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let report = coverage::run(&store, root).expect("coverage run");
    let dc = &report.doc_coverage;

    let entry =
        |rel: &str| dc.entries.iter().find(|e| e.path == Path::new(rel)).expect("entry present");
    assert!(entry("design/index.md").covered, "index.md is excluded infrastructure");
    assert!(
        entry("design/templates/design-doc-template.md").covered,
        "a templates/ file is excluded infrastructure"
    );
    assert!(
        !entry("design/01-draft/0001-real.md").covered,
        "a real ODD with no matching node is still a genuine gap"
    );
    assert!(
        entry("design/index.md").basis.contains("excluded infrastructure"),
        "basis: {}",
        entry("design/index.md").basis
    );
}

// ----- F-4: representation — arc/slice dirs vs. nodes, missing units named --

#[test]
fn coverage_representation() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    let design_root = root.join("design-v1.0.0");

    write(&design_root, "arc01-alpha/arc-plan.md", "# Arc\n");
    write(&design_root, "arc01-alpha/slice01-aa/slice-doc.md", "# S1\n");
    write(&design_root, "arc01-alpha/slice02-bb/slice-doc.md", "# S2\n");
    // A named arc — no numbered coordinate is derivable at all.
    write(&design_root, "arc-store-home/arc-plan.md", "# Named arc\n");
    write(&design_root, "arc-store-home/slice01-cc/slice-doc.md", "# S3\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // arc01 is represented, and only its first slice is — slice02-bb is the
    // deliberate gap this test exists to catch.
    persist_node(&store, arc_number(1), NodeType::Arc, "Arc 01", "# Arc 01\nbody\n", false, None);
    persist_node(
        &store,
        slice_number(1, 1, None),
        NodeType::Slice,
        "Slice 01",
        "# Slice 01\nbody\n",
        false,
        None,
    );

    let report = coverage::run(&store, root).expect("coverage run");
    let rep = &report.representation;

    assert_eq!(rep.total_arc_dirs, 2);
    assert_eq!(
        rep.missing_arcs,
        vec!["arc-store-home".to_string()],
        "the named arc has no coordinate"
    );
    assert_eq!(rep.represented_arc_dirs(), 1);

    assert_eq!(rep.total_slice_dirs, 3);
    assert_eq!(rep.represented_slice_dirs(), 1);
    // Arc directories are walked in sorted order ("arc-store-home" < "arc01-alpha",
    // since '-' < '0'), so the named arc's slice is reported first.
    assert_eq!(
        rep.missing_slices,
        vec!["arc-store-home/slice01-cc".to_string(), "arc01-alpha/slice02-bb".to_string()],
        "the unrepresented arc's slice, and the represented arc's unmatched slice, are both named"
    );
}

// ----- F-6/CDC Finding 2: a named arc *with* a node is now represented -----

#[test]
fn coverage_representation_resolves_a_named_arc_with_a_node() {
    let docs = TempDir::new().unwrap();
    let root = docs.path();
    let design_root = root.join("design-v1.0.0");

    write(&design_root, "arc-store-home/arc-plan.md", "# Named arc\n");
    write(&design_root, "arc-store-home/slice01-cc/slice-doc.md", "# S1\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    // The same name-derived handle `discover()`/`self_host()` would mint for
    // this slug (a lone named arc, so `taken` is empty).
    let arc_num = named_arc_number("arc-store-home", &std::collections::BTreeSet::new());
    persist_node(&store, arc_num, NodeType::Arc, "Store home", "# Store home\nbody\n", false, None);
    persist_node(
        &store,
        arc_num + 1,
        NodeType::Slice,
        "Slice 01",
        "# Slice 01\nbody\n",
        false,
        None,
    );

    let report = coverage::run(&store, root).expect("coverage run");
    let rep = &report.representation;

    assert!(rep.missing_arcs.is_empty(), "the named arc now resolves via its name-derived key");
    assert!(rep.missing_slices.is_empty(), "its slice resolves via the same key + offset");
}

// ----- F-5: stub bodies — ≤ 1 non-blank line, tombstones excluded -----------

#[test]
fn coverage_stubs() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    persist_node(&store, 100, NodeType::Arc, "Stub arc", "# Stub arc\n", false, None);
    persist_node(
        &store,
        101,
        NodeType::Slice,
        "Real slice",
        "# Real slice\n\nActual migrated content here.\n",
        false,
        None,
    );
    persist_node(&store, 102, NodeType::Slice, "Tombstone", "# Tombstone\n", true, None);
    persist_node(
        &store,
        103,
        NodeType::Design,
        "Stub-shaped doc",
        "# Stub-shaped doc\n",
        false,
        None,
    );

    let docs = TempDir::new().unwrap();
    let report = coverage::run(&store, docs.path()).expect("coverage run");

    let numbers: Vec<u32> = report.stubs.iter().map(|s| s.number).collect();
    assert_eq!(numbers, vec![100], "only the non-retired work-node stub is reported");
}

// ----- F-6/CDC Finding 3: provenance-absence — retargeted to `source:` -----

#[test]
fn coverage_provenance_absence() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());

    let today = NaiveDate::from_ymd_opt(2026, 7, 27).unwrap();
    let source = Source {
        paths: vec![PathBuf::from("design-v1.0.0/arc01-alpha/slice01-aa/slice-doc.md")],
        class: "slice-doc".to_string(),
        normalization: "trim+lf".to_string(),
        migrated_by: "odm-migrate/1.0.0".to_string(),
        migrated_on: today,
        synthesis: None,
        attestation: None,
    };

    persist_node(&store, 200, NodeType::Slice, "No source", "# X\nbody\n", false, None);
    persist_node(&store, 201, NodeType::Slice, "Has source", "# Y\nbody\n", false, Some(source));

    let docs = TempDir::new().unwrap();
    let report = coverage::run(&store, docs.path()).expect("coverage run");

    let numbers: Vec<u32> = report.provenance_missing.iter().map(|p| p.number).collect();
    assert!(numbers.contains(&200), "the node with no `source` sub-map is reported");
    assert!(!numbers.contains(&201), "the node carrying a `source` sub-map is not reported");
}

// ----- F-7: --coverage (here: `run`) is read-only ---------------------------

#[test]
fn coverage_is_read_only() {
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    persist_node(&store, 300, NodeType::Slice, "Existing", "# X\nbody\n", false, None);

    let docs = TempDir::new().unwrap();
    copy_tree(&plan_set(), &docs.path().join("design-v1.0.0"));

    let before = snapshot_bytes(store_dir.path());
    let _report = coverage::run(&store, docs.path()).expect("coverage run");
    let after = snapshot_bytes(store_dir.path());

    assert_eq!(before, after, "coverage::run wrote nothing to the store");
}

fn snapshot_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out: Vec<(PathBuf, Vec<u8>)> = walk(root)
        .into_iter()
        .filter(|p| p.is_file())
        .map(|p| (p.strip_prefix(root).unwrap().to_path_buf(), std::fs::read(&p).unwrap()))
        .collect();
    out.sort();
    out
}
