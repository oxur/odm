//! Slice02 (arc06) — the importer meets reality: `odm migrate` over odm's own
//! `docs/design` corpus. Tests run against a **snapshot copy** of the live tree
//! so they are deterministic and never touch the real docs. Test names carry the
//! substrings the slice02 ledger Verify commands filter on:
//! `migrate_real_docs_all_accounted`, `supersedes_real_shape_resolves`,
//! `migrate_real_docs_idempotent_and_dry_run`.

use std::path::{Path, PathBuf};

use odm_core::NodeType;
use odm_core::frontmatter::SupersedeKind;
use odm_migrate::{Mode, migrate};
use odm_store::Store;
use tempfile::TempDir;

/// The workspace path of the live `docs/design` corpus (odm-migrate manifest dir
/// is `crates/odm-migrate`).
fn real_docs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design")
}

/// A fixture corpus under the workspace `test-data/`.
fn fixtures(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data").join(name)
}

// ----- N-1: every real ODD is accounted for; legacy files intact ------------

#[test]
fn migrate_real_docs_all_accounted() {
    // Snapshot-copy the live corpus so the test is deterministic and read-only
    // w.r.t. the real tree.
    let legacy_dir = TempDir::new().unwrap();
    copy_tree(&real_docs(), legacy_dir.path());
    let before = snapshot_bytes(legacy_dir.path());

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let report = migrate(&store, legacy_dir.path(), Mode::Commit).expect("migrate real docs");

    // Every discovered ODD lands in exactly one of created/skipped — none dropped.
    let discovered = odm_migrate::legacy::discover(legacy_dir.path());
    assert!(
        discovered.len() >= 12,
        "sanity: the real corpus has ≥12 ODDs, got {}",
        discovered.len()
    );
    assert_eq!(
        report.created_count() + report.skipped_count(),
        discovered.len(),
        "every ODD created or reported-skipped (none silently dropped)"
    );
    // The current corpus is all valid progression states → all created, 0 skipped.
    assert_eq!(report.skipped_count(), 0, "no ODD skipped: {:?}", report.skipped);
    assert_eq!(report.created_count(), discovered.len(), "all ODDs imported");

    // Every imported node is a document node with its number preserved, and the
    // C-2 taxonomy is applied: `research` iff the source tags say so, else
    // `design` (never the pre-C-2 `odd`, which no longer exists).
    let nodes = store.load_all().unwrap();
    assert!(
        nodes
            .iter()
            .all(|d| matches!(d.frontmatter().node_type(), NodeType::Design | NodeType::Research)),
        "every imported node is design or research"
    );
    assert!(nodes.iter().any(|d| d.frontmatter().number() == 13), "ODD-0013 imported");
    assert!(nodes.iter().any(|d| d.frontmatter().number() == 19), "ODD-0019 imported");

    // Classification tracks the source tags, not the title: every node tagged
    // `research` is a `research` node, and every other one is `design`.
    for node in &nodes {
        let fm = node.frontmatter();
        let tagged_research = fm.tags().iter().any(|t| t.eq_ignore_ascii_case("research"));
        let expected = if tagged_research { NodeType::Research } else { NodeType::Design };
        assert_eq!(
            fm.node_type(),
            expected,
            "#{} {:?} (tags {:?}) classified wrong",
            fm.number(),
            fm.name(),
            fm.tags()
        );
    }
    assert!(
        nodes.iter().any(|d| d.frontmatter().node_type() == NodeType::Research),
        "the real corpus contains at least one research doc"
    );

    // Never-delete: the legacy corpus is byte-for-byte intact after migration.
    assert_eq!(before, snapshot_bytes(legacy_dir.path()), "no legacy file removed or mutated");
}

// ----- N-3: the real supersedes shape resolves; numbering space distinct -----

#[test]
fn supersedes_real_shape_resolves() {
    // (a) The string ref shape (`"ODD-00NN"`) — the plausible real form — resolves
    // to the target's freshly-minted ULID.
    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let report = migrate(&store, &fixtures("legacy-refstr"), Mode::Commit).expect("migrate refstr");
    assert!(report.warnings.is_empty(), "string refs resolve, no warnings: {:?}", report.warnings);

    let nodes: std::collections::HashMap<u32, _> =
        store.load_all().unwrap().into_iter().map(|d| (d.frontmatter().number(), d)).collect();
    let sup = nodes[&31].frontmatter().edges().supersedes.first().expect("#31 supersedes edge");
    assert_eq!(sup.node, nodes[&30].frontmatter().id(), "\"ODD-0030\" → #30's ULID");
    assert_eq!(sup.kind, SupersedeKind::Obsoletes);

    // (b) The real corpus uses `null` everywhere → no supersession, no warnings.
    let legacy_dir = TempDir::new().unwrap();
    copy_tree(&real_docs(), legacy_dir.path());
    let real_store_dir = TempDir::new().unwrap();
    let real_store = Store::open(real_store_dir.path());
    let real = migrate(&real_store, legacy_dir.path(), Mode::Commit).expect("migrate real");
    assert!(
        real.warnings.is_empty(),
        "real corpus has no supersession warnings: {:?}",
        real.warnings
    );
    assert!(
        real_store.load_all().unwrap().iter().all(|d| d
            .frontmatter()
            .edges()
            .supersedes
            .is_empty()),
        "no supersedes edges in the real corpus"
    );
    // Numbering space: the imported ODD numbers are all distinct (idempotence keys
    // on (type=odd, number)); no duplicate would have created a false skip.
    let mut numbers: Vec<u32> =
        real_store.load_all().unwrap().iter().map(|d| d.frontmatter().number()).collect();
    let count = numbers.len();
    numbers.sort_unstable();
    numbers.dedup();
    assert_eq!(numbers.len(), count, "odd numbers are distinct (no collision)");
}

// ----- N-5: idempotent + --dry-run hold at real-corpus scale ----------------

#[test]
fn migrate_real_docs_idempotent_and_dry_run() {
    let legacy_dir = TempDir::new().unwrap();
    copy_tree(&real_docs(), legacy_dir.path());

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let first = migrate(&store, legacy_dir.path(), Mode::Commit).expect("first");
    let n = first.created_count();
    assert!(n >= 12);

    // Re-run: every number already exists as an odd node → 0 created.
    let second = migrate(&store, legacy_dir.path(), Mode::Commit).expect("second");
    assert_eq!(second.created_count(), 0, "idempotent re-run creates nothing");
    assert_eq!(second.skipped_count(), n, "all skipped as already-existing");
    assert_eq!(store.load_all().unwrap().len(), n, "no duplicates on disk");

    // --dry-run over a fresh store: plans all, writes nothing.
    let dry_dir = TempDir::new().unwrap();
    let dry_store = Store::open(dry_dir.path());
    let dry = migrate(&dry_store, legacy_dir.path(), Mode::DryRun).expect("dry-run");
    assert_eq!(dry.created_count(), n, "dry-run plans the whole corpus");
    assert!(dry_store.load_all().unwrap().is_empty(), "dry-run wrote nothing");
    assert!(!dry_dir.path().join("nodes").exists(), "no nodes/ created");
}

// ----- helpers (deterministic copy + byte snapshot) -------------------------

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

fn snapshot_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out: Vec<(PathBuf, Vec<u8>)> = walk(root)
        .into_iter()
        .filter(|p| p.is_file())
        .map(|p| (p.strip_prefix(root).unwrap().to_path_buf(), std::fs::read(&p).unwrap()))
        .collect();
    out.sort();
    out
}
