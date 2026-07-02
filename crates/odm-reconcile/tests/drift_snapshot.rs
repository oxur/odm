//! Integration tests for the `.odm/` drift snapshot + incremental reconcile
//! (arc05 slice07). Public API only. Test names carry the substrings the ledger
//! Verify commands filter on (`drift_snapshot_round_trip`,
//! `drift_snapshot_corrupt_rebuilds`, `incremental_skips_unchanged_reprobes_changed`,
//! `volatile_not_run_incrementally_stamps_last_checked`).
//!
//! "Did the probe run?" is proven with a **counting script**: a `#!/bin/sh`
//! probe that appends a line to a counter file each time it runs (Unix-only —
//! the same tempfile-script pattern as slice02's signal test). Line count =
//! run count, so a cutoff that *skips* re-probing is observable, not just
//! "same result".

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{DesiredFact, Id, NodeType, Origin, ProbeSpec, ShellExpect};
use odm_reconcile::{
    DriftSnapshot, FactEntry, FactState, InputFingerprint, Load, ProbeOutcome, reconcile_full,
    reconcile_incremental,
};
use odm_store::Store;

const N1: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
}

/// Writes an executable `#!/bin/sh` script at `path` that appends one line to
/// `counter` (absolute path) each time it runs, and exits 0.
fn counting_script(path: &Path, counter: &Path) {
    let body = format!("#!/bin/sh\necho run >> {}\n", counter.display());
    std::fs::write(path, body).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Line count of the counter file (0 if absent) — the number of probe runs.
fn run_count(counter: &Path) -> usize {
    std::fs::read_to_string(counter).map(|s| s.lines().count()).unwrap_or(0)
}

/// Seeds a node with one shell fact `run`ning `script`, with the given `inputs`
/// (empty ⇒ volatile).
fn seed_shell_fact(store: &Store, script: &Path, inputs: Vec<String>) {
    let fact = DesiredFact {
        id: "f".to_string(),
        describe: "d".to_string(),
        probe: ProbeSpec::Shell {
            run: script.to_str().unwrap().to_string(),
            inputs,
            expect: ShellExpect { exit: 0, stdout_contains: None },
        },
    };
    let fm = Frontmatter::new(
        Id::from_str(N1).unwrap(),
        1,
        NodeType::Slice,
        "n",
        day(),
        day(),
        Origin::Planned,
    )
    .with_desired_facts(vec![fact]);
    store.persist(&Document::new(fm, "body\n")).unwrap();
}

// ----- K-3: the drift snapshot round-trips + self-heals ----------------------

#[test]
fn drift_snapshot_round_trip() {
    let snapshot = DriftSnapshot::new(
        100,
        vec![
            FactEntry {
                node_id: Id::from_str(N1).unwrap(),
                fact_id: "input".to_string(),
                outcome: ProbeOutcome::Holds,
                state: FactState::InputDerived {
                    inputs: vec![InputFingerprint {
                        rel_path: "x".to_string(),
                        exists: true,
                        size: 4,
                        mtime_secs: 100,
                        content_hash: Some("a".repeat(64)),
                        captured_at: 100,
                    }],
                },
            },
            FactEntry {
                node_id: Id::from_str(N1).unwrap(),
                fact_id: "vol".to_string(),
                outcome: ProbeOutcome::Drifted {
                    expected: "exit 0".to_string(),
                    observed: "exit 1".to_string(),
                },
                state: FactState::Volatile { last_checked: 99 },
            },
        ],
    );

    let bytes = snapshot.encode().unwrap();
    assert_eq!(&bytes[0..8], b"ODMDRIFT", "own MAGIC");
    let decoded = DriftSnapshot::decode(&bytes).expect("round-trips");
    assert_eq!(decoded, snapshot);

    // Persist + load round-trips through the filesystem too.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".odm/drift.bin");
    snapshot.persist(&path).unwrap();
    match DriftSnapshot::load(&path).unwrap() {
        Load::Loaded(loaded) => assert_eq!(loaded, snapshot),
        Load::RebuildNeeded(r) => panic!("expected a loaded snapshot, got {r:?}"),
    }
    // The entry lookup finds a fact by (node, id).
    assert!(loaded_has(&path, "input"));
}

fn loaded_has(path: &Path, fact_id: &str) -> bool {
    match DriftSnapshot::load(path).unwrap() {
        Load::Loaded(s) => s.entry(Id::from_str(N1).unwrap(), fact_id).is_some(),
        Load::RebuildNeeded(_) => false,
    }
}

#[test]
fn drift_snapshot_corrupt_rebuilds() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".odm/drift.bin");

    // Missing → rebuild.
    assert!(matches!(DriftSnapshot::load(&path).unwrap(), Load::RebuildNeeded(_)));

    // Corrupt (garbage) → rebuild, never a bad parse.
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"not a drift snapshot at all").unwrap();
    assert!(matches!(DriftSnapshot::load(&path).unwrap(), Load::RebuildNeeded(_)));

    // A flipped byte in a valid snapshot → checksum catches it.
    let good = DriftSnapshot::new(1, Vec::new()).encode().unwrap();
    let mut bad = good.clone();
    let last = bad.len() - 1;
    bad[last] ^= 0xff;
    assert!(DriftSnapshot::decode(&bad).is_err());
    assert!(DriftSnapshot::decode(&good).is_ok());
}

// ----- K-4: incremental skips unchanged inputs, re-probes changed ------------

#[test]
fn incremental_skips_unchanged_reprobes_changed() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path());
    let snap = dir.path().join(".odm/drift.bin");

    // Aux files (script + counter + the probe's declared input) live outside the
    // corpus `nodes/` tree so they aren't walked as node files.
    let aux = tempfile::tempdir().unwrap();
    let script = aux.path().join("probe.sh");
    let counter = aux.path().join("count.txt");
    counting_script(&script, &counter);
    let input = aux.path().join("input.txt");
    std::fs::write(&input, b"v1").unwrap();

    seed_shell_fact(&store, &script, vec![input.to_str().unwrap().to_string()]);

    // First incremental run: no prior snapshot → probes (count 1).
    reconcile_incremental(&store, &snap).unwrap();
    assert_eq!(run_count(&counter), 1, "first run probes");

    // Second run, input unchanged → cached outcome, NO re-probe (count stays 1).
    reconcile_incremental(&store, &snap).unwrap();
    assert_eq!(run_count(&counter), 1, "unchanged input → cached, no re-probe");

    // Change the input → re-probe (count 2).
    std::fs::write(&input, b"v2-different").unwrap();
    reconcile_incremental(&store, &snap).unwrap();
    assert_eq!(run_count(&counter), 2, "changed input → re-probe");

    // Unchanged again → still cached (count stays 2).
    reconcile_incremental(&store, &snap).unwrap();
    assert_eq!(run_count(&counter), 2, "unchanged again → cached");
}

// ----- K-5: volatile is not run incrementally; carries + stamps last-checked -

#[test]
fn volatile_not_run_incrementally_stamps_last_checked() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path());
    let snap = dir.path().join(".odm/drift.bin");

    let aux = tempfile::tempdir().unwrap();
    let script = aux.path().join("probe.sh");
    let counter = aux.path().join("count.txt");
    counting_script(&script, &counter);

    // Volatile: a shell fact with NO inputs.
    seed_shell_fact(&store, &script, Vec::new());

    // A full run checks the volatile fact (count 1) and stamps last-checked.
    let full = reconcile_full(&store, &snap).unwrap();
    assert_eq!(run_count(&counter), 1, "full run checks volatile");
    let stamped = match &full.entry(Id::from_str(N1).unwrap(), "f").unwrap().state {
        FactState::Volatile { last_checked } => *last_checked,
        FactState::InputDerived { .. } => panic!("expected a volatile entry"),
    };
    assert!(stamped > 0, "last-checked is stamped");

    // An incremental run does NOT re-run the volatile probe (count stays 1) and
    // carries the last outcome + last-checked verbatim.
    let inc = reconcile_incremental(&store, &snap).unwrap();
    assert_eq!(run_count(&counter), 1, "volatile not run incrementally");
    let carried = inc.entry(Id::from_str(N1).unwrap(), "f").unwrap();
    assert_eq!(carried.outcome, ProbeOutcome::Holds);
    match &carried.state {
        FactState::Volatile { last_checked } => assert_eq!(*last_checked, stamped, "carried"),
        FactState::InputDerived { .. } => panic!("expected a volatile entry"),
    }
}

// Full mode always re-probes input-derived facts (ignores the cache); a missing
// declared input is captured without panic (exists: false).
#[test]
fn full_reprobes_input_derived_including_missing_input() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path());
    let snap = dir.path().join(".odm/drift.bin");

    let aux = tempfile::tempdir().unwrap();
    let script = aux.path().join("probe.sh");
    let counter = aux.path().join("count.txt");
    counting_script(&script, &counter);
    // Declared input that does NOT exist — capture must record exists: false.
    let missing = aux.path().join("nope.txt");

    seed_shell_fact(&store, &script, vec![missing.to_str().unwrap().to_string()]);

    // Full always re-probes input-derived, even with an unchanged (missing) input.
    reconcile_full(&store, &snap).unwrap();
    assert_eq!(run_count(&counter), 1);
    reconcile_full(&store, &snap).unwrap();
    assert_eq!(run_count(&counter), 2, "Full re-probes every input-derived fact");

    // The snapshot recorded the missing input as a captured (absent) fingerprint.
    let entry = reconcile_full(&store, &snap).unwrap();
    let fact = entry.entry(Id::from_str(N1).unwrap(), "f").unwrap();
    match &fact.state {
        FactState::InputDerived { inputs } => {
            assert_eq!(inputs.len(), 1);
            assert!(!inputs[0].exists, "missing input captured as absent");
        }
        FactState::Volatile { .. } => panic!("expected input-derived"),
    }

    // Incremental after Full: the (still-missing) input is unchanged → cached.
    let before = run_count(&counter);
    reconcile_incremental(&store, &snap).unwrap();
    assert_eq!(run_count(&counter), before, "unchanged missing input → cached");
}
