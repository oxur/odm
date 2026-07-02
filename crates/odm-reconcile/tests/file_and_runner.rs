//! Integration tests for the `file` probe (G-1) and the probe-runner
//! (G-3/G-4/G-6). Public API only. Test names carry the substrings the ledger
//! Verify commands filter on (`file_probe_holds`, `file_probe_drifts_on_missing`,
//! `file_probe_drifts_on_hash_mismatch`, `file_probe_errors_on_unreadable`,
//! `runner_collects_per_node`, `runner_factless_node_is_empty`,
//! `runner_corpus_read_through`, `report_distinguishes_drift_from_error`).
//!
//! The file-probe cases use **real files** in a temp dir; the runner cases use a
//! real [`Store`] in a temp dir, exercising the store read path (G-5).

use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{DesiredFact, FileExpect, Id, NodeType, Origin, ProbeSpec, ShellExpect};
use odm_reconcile::{Probe, ProbeOutcome, Runner};
use sha2::{Digest as _, Sha256};

const N1: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const N2: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAW";
const N3: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAX";

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 20).expect("valid date")
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn file_expect(exists: bool, sha256: Option<String>, size: Option<u64>) -> FileExpect {
    FileExpect { exists, sha256, size }
}

fn shell_fact(id: &str, run: &str, exit: i32) -> DesiredFact {
    DesiredFact {
        id: id.to_string(),
        describe: "shell fact".to_string(),
        probe: ProbeSpec::Shell {
            run: run.to_string(),
            inputs: Vec::new(),
            expect: ShellExpect { exit, stdout_contains: None },
        },
    }
}

fn node(id: &str, facts: Vec<DesiredFact>) -> Document {
    let fm = Frontmatter::new(
        Id::from_str(id).expect("valid ulid"),
        1,
        NodeType::Slice,
        "n",
        day(),
        day(),
        Origin::Planned,
    )
    .with_desired_facts(facts);
    Document::new(fm, "body\n")
}

// ----- G-1: the file probe ---------------------------------------------------

#[test]
fn file_probe_holds() {
    let dir = tempfile::tempdir().unwrap();
    let content = b"hello world\n";
    std::fs::write(dir.path().join("data.txt"), content).unwrap();

    // exists + matching size + matching hash all hold.
    let probe = odm_reconcile::FileProbe::new(
        dir.path(),
        "data.txt",
        file_expect(true, Some(hex_sha256(content)), Some(content.len() as u64)),
    );
    assert_eq!(probe.evaluate(), ProbeOutcome::Holds);

    // A bare existence check on a present file also holds.
    let bare = odm_reconcile::FileProbe::new(dir.path(), "data.txt", file_expect(true, None, None));
    assert_eq!(bare.evaluate(), ProbeOutcome::Holds);

    // `exists: false` holds when the file is genuinely absent.
    let absent =
        odm_reconcile::FileProbe::new(dir.path(), "gone.txt", file_expect(false, None, None));
    assert_eq!(absent.evaluate(), ProbeOutcome::Holds);
}

#[test]
fn file_probe_drifts_on_missing() {
    let dir = tempfile::tempdir().unwrap();
    // Missing when exists:true → drift (a real declared-vs-observed finding).
    let probe =
        odm_reconcile::FileProbe::new(dir.path(), "absent.txt", file_expect(true, None, None));
    match probe.evaluate() {
        ProbeOutcome::Drifted { observed, .. } => assert!(observed.contains("missing")),
        other => panic!("expected Drifted, got {other:?}"),
    }

    // Present when exists:false → also drift.
    std::fs::write(dir.path().join("here.txt"), b"x").unwrap();
    let present =
        odm_reconcile::FileProbe::new(dir.path(), "here.txt", file_expect(false, None, None));
    match present.evaluate() {
        ProbeOutcome::Drifted { observed, .. } => assert!(observed.contains("present")),
        other => panic!("expected Drifted, got {other:?}"),
    }
}

#[test]
fn file_probe_drifts_on_hash_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("data.txt"), b"actual contents").unwrap();
    let wrong_hash = "0".repeat(64);
    let probe = odm_reconcile::FileProbe::new(
        dir.path(),
        "data.txt",
        file_expect(true, Some(wrong_hash.clone()), None),
    );
    match probe.evaluate() {
        ProbeOutcome::Drifted { expected, observed } => {
            assert!(expected.contains(&wrong_hash));
            assert!(observed.contains("sha256"));
        }
        other => panic!("expected Drifted, got {other:?}"),
    }
}

#[test]
fn file_probe_drifts_on_wrong_size() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("data.txt"), b"12345").unwrap();
    let probe =
        odm_reconcile::FileProbe::new(dir.path(), "data.txt", file_expect(true, None, Some(999)));
    match probe.evaluate() {
        ProbeOutcome::Drifted { expected, observed } => {
            assert!(expected.contains("999"));
            assert!(observed.contains('5'));
        }
        other => panic!("expected Drifted, got {other:?}"),
    }
}

#[test]
fn file_probe_errors_on_unreadable() {
    // A directory cannot be read as file content: requesting a hash on a path
    // that is a directory is "couldn't check" → Error (not drift).
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("subdir")).unwrap();
    let probe = odm_reconcile::FileProbe::new(
        dir.path(),
        "subdir",
        file_expect(true, Some("a".repeat(64)), None),
    );
    match probe.evaluate() {
        ProbeOutcome::Error { reason } => assert!(reason.contains("could not read")),
        other => panic!("expected Error, got {other:?}"),
    }
}

#[test]
fn file_probe_errors_on_invalid_path() {
    // A path that cannot even be stat-ed (an embedded NUL) is unevaluable →
    // Error, distinct from "the file is absent" (which would be drift).
    let dir = tempfile::tempdir().unwrap();
    let probe =
        odm_reconcile::FileProbe::new(dir.path(), "bad\0name", file_expect(true, None, None));
    match probe.evaluate() {
        ProbeOutcome::Error { reason } => assert!(reason.contains("could not stat")),
        other => panic!("expected Error, got {other:?}"),
    }
}

// ----- G-3: per-node collection ----------------------------------------------

#[test]
fn runner_collects_per_node() {
    let dir = tempfile::tempdir().unwrap();
    let store = odm_store::Store::open(dir.path());
    let runner = Runner::new(&store);

    let doc = node(N1, vec![shell_fact("a", "true", 0), shell_fact("b", "false", 0)]);
    let report = runner.run_node(&doc);

    assert_eq!(report.node_id, Id::from_str(N1).unwrap());
    assert_eq!(report.results.len(), 2);
    assert_eq!(report.results[0].fact_id, "a");
    assert_eq!(report.results[0].outcome, ProbeOutcome::Holds);
    assert_eq!(report.results[1].fact_id, "b");
    assert!(matches!(report.results[1].outcome, ProbeOutcome::Drifted { .. }));
}

#[test]
fn runner_factless_node_is_empty() {
    let dir = tempfile::tempdir().unwrap();
    let store = odm_store::Store::open(dir.path());
    let runner = Runner::new(&store);

    let report = runner.run_node(&node(N1, vec![]));
    assert!(report.is_empty());
    assert!(report.results.is_empty());
    assert_eq!(report.counts().total(), 0);

    // A corpus of only factless nodes is empty (no `(node, fact)` tuples).
    store.persist(&node(N1, vec![])).unwrap();
    store.persist(&node(N2, vec![])).unwrap();
    let corpus = runner.run_corpus().unwrap();
    assert!(corpus.is_empty());
    assert_eq!(corpus.counts().total(), 0);
}

// ----- G-4 / G-5: corpus collection with read-through freshness --------------

#[test]
fn runner_corpus_read_through() {
    let dir = tempfile::tempdir().unwrap();
    let store = odm_store::Store::open(dir.path());
    let runner = Runner::new(&store);

    // Seed one node with a fact and one factless node.
    store.persist(&node(N1, vec![shell_fact("a", "true", 0)])).unwrap();
    store.persist(&node(N2, vec![])).unwrap();

    let first = runner.run_corpus().unwrap();
    // Only the node with facts appears.
    assert_eq!(first.nodes.len(), 1);
    assert_eq!(first.nodes[0].node_id, Id::from_str(N1).unwrap());

    // Write a brand-new fact-bearing node; a fresh run sees it with no rebuild.
    store.persist(&node(N3, vec![shell_fact("c", "true", 0)])).unwrap();
    let second = runner.run_corpus().unwrap();
    assert_eq!(second.nodes.len(), 2);
    let ids: Vec<Id> = second.iter().map(|(node_id, _)| node_id).collect();
    assert!(ids.contains(&Id::from_str(N3).unwrap()));
}

// ----- G-6: the aggregate keeps drift and error distinct ---------------------

#[test]
fn report_distinguishes_drift_from_error() {
    let dir = tempfile::tempdir().unwrap();
    let store = odm_store::Store::open(dir.path());
    let runner = Runner::new(&store);

    // One holding, one drifted, one errored fact across the corpus.
    store
        .persist(&node(
            N1,
            vec![
                shell_fact("holds", "true", 0),
                shell_fact("drifts", "false", 0),
                shell_fact("errors", "odm-no-such-binary-xyzzy", 0),
            ],
        ))
        .unwrap();

    let report = runner.run_corpus().unwrap();
    let counts = report.counts();
    assert_eq!(counts.holds, 1, "exactly one holds");
    assert_eq!(counts.drifted, 1, "exactly one drift — distinct from error");
    assert_eq!(counts.errored, 1, "exactly one error — distinct from drift");
    assert_eq!(counts.total(), 3);
}

// ----- S-2 (arc05 slice04): the report carries render-identity ---------------

#[test]
fn report_carries_identity() {
    let dir = tempfile::tempdir().unwrap();
    let store = odm_store::Store::open(dir.path());
    let runner = Runner::new(&store);

    // A node with a describing fact: the report must carry number/name/describe
    // so renderers need no second load.
    let mut fact = shell_fact("built", "true", 0);
    fact.describe = "the crate builds cleanly".to_string();
    let doc = node(N1, vec![fact]);
    let report = runner.run_node(&doc);

    assert_eq!(report.node_id, Id::from_str(N1).unwrap());
    assert_eq!(report.number, 1);
    assert_eq!(report.name, "n");
    assert_eq!(report.results[0].fact_id, "built");
    assert_eq!(report.results[0].describe, "the crate builds cleanly");
}
