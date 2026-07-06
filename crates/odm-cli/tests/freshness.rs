//! In-process tests for the slice08 freshness wiring (arc05 capstone, ODD-0019).
//! Drives [`odm_cli::dispatch`] against a temp store. Test names carry the
//! substrings the slice08 ledger Verify commands filter on:
//! `orient_runs_zero_volatile_probes`, `orient_reprobes_changed_input`,
//! `reconcile_command_runs_full_refreshes_volatile`,
//! `rollup_render_volatile_staleness`, `orient_render_volatile_staleness`,
//! `orient_never_checked_volatile`, `rollup_json_staleness_additive`,
//! `orient_json_staleness_additive`, `drift_snapshot_default_path`,
//! `rollup_regenerates_on_drift_change`,
//! `rollup_skips_when_drift_and_corpus_unchanged`,
//! `next_unaffected_by_deferred_marker`.
//!
//! **Probe classes (ODD-0019 §3.1):** a `shell` fact with **no inputs** is
//! *volatile* — never auto-run by a bare command, refreshed only on an explicit
//! `odm reconcile`. A `shell` fact **with inputs** (or a `file` fact) is
//! *input-derived* — re-probed by any command when an input changed. A **counting
//! probe** (a script that appends one line per run) makes "did it run?" directly
//! observable, so the zero-volatile-probes invariant is *proven*, not asserted.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Deferral, Document, Frontmatter};
use odm_core::{DesiredFact, FileExpect, Id, NodeType, Origin, ProbeSpec, ShellExpect};
use odm_store::Store;
use tempfile::TempDir;

const CONFIG: &str = "\
[gates.project]
sequence = [\"planned\", \"complete\"]

[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

struct Run {
    ok: bool,
    code: Option<u8>,
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, root, &mut out, &mut err);
    Run { ok: result.is_ok(), code: result.ok(), out: String::from_utf8(out).unwrap() }
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, 6).unwrap()
}

fn write_config(root: &Path) {
    std::fs::write(root.join("odm.toml"), CONFIG).unwrap();
}

fn read_rollup(root: &Path) -> String {
    std::fs::read_to_string(root.join("ROLLUP.md")).expect("ROLLUP.md")
}

fn node(number: u32, ty: NodeType, name: &str) -> Frontmatter {
    Frontmatter::new(Id::new(), number, ty, name, day(), day(), Origin::Planned)
}

fn persist(root: &Path, fm: Frontmatter) {
    Store::open(root).persist(&Document::new(fm, "body\n")).expect("persist");
}

/// A **volatile** shell fact: `run` a command, expect exit `0`, no inputs.
fn volatile_fact(id: &str, describe: &str, run: &str) -> DesiredFact {
    DesiredFact {
        id: id.to_string(),
        describe: describe.to_string(),
        probe: ProbeSpec::Shell {
            run: run.to_string(),
            inputs: Vec::new(),
            expect: ShellExpect { exit: 0, stdout_contains: None },
        },
    }
}

/// An **input-derived** shell fact: `run` a command that is a pure function of
/// the declared `inputs` (relative to the store root).
fn input_derived_fact(id: &str, describe: &str, run: &str, inputs: &[&str]) -> DesiredFact {
    DesiredFact {
        id: id.to_string(),
        describe: describe.to_string(),
        probe: ProbeSpec::Shell {
            run: run.to_string(),
            inputs: inputs.iter().map(|s| s.to_string()).collect(),
            expect: ShellExpect { exit: 0, stdout_contains: None },
        },
    }
}

/// An **input-derived** file fact: the fact holds iff `path` (relative to root)
/// exists (the [`FileExpect`] default).
fn file_fact(id: &str, describe: &str, path: &str) -> DesiredFact {
    DesiredFact {
        id: id.to_string(),
        describe: describe.to_string(),
        probe: ProbeSpec::File { path: path.to_string(), expect: FileExpect::default() },
    }
}

/// Writes an executable **counting probe** into `scripts_dir`: each run appends
/// one line to `counter` (an absolute path baked in), then exits 0. Returns the
/// absolute script path to use as a fact's `run` (a single whitespace token).
fn counting_probe(scripts_dir: &Path, name: &str, counter: &Path) -> String {
    let script = scripts_dir.join(name);
    let body = format!("#!/bin/sh\nprintf 'x\\n' >> '{}'\nexit 0\n", counter.display());
    std::fs::write(&script, body).unwrap();
    let mut perms = std::fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).unwrap();
    script.to_str().expect("script path is valid UTF-8 with no spaces").to_string()
}

/// How many times the counting probe has run (lines in its counter file).
fn probe_runs(counter: &Path) -> usize {
    std::fs::read_to_string(counter).map(|s| s.lines().count()).unwrap_or(0)
}

// ----- L-1: bare orient runs ZERO volatile probes ---------------------------

#[test]
fn orient_runs_zero_volatile_probes() {
    let dir = TempDir::new().unwrap();
    let aux = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    let counter = aux.path().join("count");
    let script = counting_probe(aux.path(), "probe.sh", &counter);
    persist(root, node(1, NodeType::Project, "Proj"));
    persist(
        root,
        node(2, NodeType::Slice, "DB layer").with_desired_facts(vec![volatile_fact(
            "db-up",
            "the prod DB answers",
            &script,
        )]),
    );

    // Bare orient must not run the volatile probe at all — the slice04 regression.
    assert!(run(root, &["orient"]).ok);
    assert_eq!(probe_runs(&counter), 0, "bare orient ran a volatile probe");

    // Even repeated orients stay at zero.
    assert!(run(root, &["orient"]).ok);
    assert_eq!(probe_runs(&counter), 0, "repeated orient ran a volatile probe");
}

// ----- L-1: an input-derived fact whose input changed IS re-probed ----------

#[test]
fn orient_reprobes_changed_input() {
    let dir = TempDir::new().unwrap();
    let aux = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    let counter = aux.path().join("count");
    let script = counting_probe(aux.path(), "probe.sh", &counter);
    std::fs::write(root.join("trigger.txt"), b"one").unwrap();
    persist(root, node(1, NodeType::Project, "Proj"));
    persist(
        root,
        node(2, NodeType::Slice, "Build").with_desired_facts(vec![input_derived_fact(
            "built",
            "artifact fresh",
            &script,
            &["trigger.txt"],
        )]),
    );

    // First orient: no prior fingerprint → probe once.
    assert!(run(root, &["orient"]).ok);
    assert_eq!(probe_runs(&counter), 1, "first orient must probe the input-derived fact");

    // No input change → carried from the snapshot, not re-probed.
    assert!(run(root, &["orient"]).ok);
    assert_eq!(probe_runs(&counter), 1, "unchanged input must not re-probe");

    // Input changed → re-probed by the bare command (no explicit reconcile).
    std::fs::write(root.join("trigger.txt"), b"two-different").unwrap();
    assert!(run(root, &["orient"]).ok);
    assert_eq!(probe_runs(&counter), 2, "changed input must re-probe");
}

// ----- L-2: `odm reconcile` runs the full path — refreshes volatile ---------

#[test]
fn reconcile_command_runs_full_refreshes_volatile() {
    let dir = TempDir::new().unwrap();
    let aux = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);

    let counter = aux.path().join("count");
    let script = counting_probe(aux.path(), "probe.sh", &counter);
    persist(
        root,
        node(1, NodeType::Slice, "DB layer").with_desired_facts(vec![volatile_fact(
            "db-up",
            "the prod DB answers",
            &script,
        )]),
    );

    // Each explicit reconcile re-runs the volatile probe (full mode).
    assert!(run(root, &["reconcile"]).ok);
    assert_eq!(probe_runs(&counter), 1, "reconcile must run the volatile probe");
    assert!(run(root, &["reconcile"]).ok);
    assert_eq!(probe_runs(&counter), 2, "a second reconcile must re-run it (re-stamp)");

    // The refresh re-stamps a `last_checked` on the volatile fact (--json, additive).
    let r = run(root, &["reconcile", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");
    assert!(
        v["nodes"][0]["results"][0]["last_checked"].is_i64(),
        "volatile fact carries a last_checked stamp:\n{}",
        r.out
    );
}

// ----- L-3: honest-staleness rendering — "last checked" on volatile ---------

#[test]
fn rollup_render_volatile_staleness() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    // A drifting volatile fact (`false` exits 1 where 0 expected) so it renders.
    persist(
        root,
        node(7, NodeType::Slice, "DB layer").with_desired_facts(vec![volatile_fact(
            "db-up",
            "the prod DB answers",
            "false",
        )]),
    );

    // Explicit reconcile stamps the volatile outcome; rollup renders its staleness.
    assert!(run(root, &["reconcile"]).ok);
    assert!(run(root, &["rollup"]).ok);
    let md = read_rollup(root);
    let drift = md.split("## Drift").nth(1).expect("a Drift section");
    assert!(drift.contains("#7 DB layer"), "identity:\n{md}");
    assert!(drift.contains("last checked"), "volatile staleness rendered:\n{md}");
}

#[test]
fn orient_render_volatile_staleness() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    persist(root, node(1, NodeType::Project, "Proj"));
    persist(
        root,
        node(7, NodeType::Slice, "DB layer").with_desired_facts(vec![volatile_fact(
            "db-up",
            "the prod DB answers",
            "false",
        )]),
    );

    assert!(run(root, &["reconcile"]).ok);
    let r = run(root, &["orient"]);
    assert!(r.ok);
    let drift = r.out.split("DRIFT").nth(1).expect("a DRIFT section");
    assert!(drift.contains("#7 DB layer"), "identity:\n{}", r.out);
    assert!(drift.contains("last checked"), "volatile staleness rendered:\n{}", r.out);
}

// ----- L-3: a never-checked volatile fact is honest, never "fresh" ----------

#[test]
fn orient_never_checked_volatile() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    persist(root, node(1, NodeType::Project, "Proj"));
    persist(
        root,
        node(7, NodeType::Slice, "DB layer").with_desired_facts(vec![volatile_fact(
            "db-up",
            "the prod DB answers",
            "false",
        )]),
    );

    // No reconcile: the volatile fact was never checked. Bare orient must say so
    // honestly — never fabricate "fresh".
    let r = run(root, &["orient"]);
    assert!(r.ok);
    let drift = r.out.split("DRIFT").nth(1).expect("a DRIFT section");
    assert!(drift.contains("#7 DB layer"), "identity:\n{}", r.out);
    assert!(drift.contains("not yet checked"), "honest never-checked marker:\n{}", r.out);
    assert!(drift.contains("odm reconcile"), "points at the refresh verb:\n{}", r.out);
    assert!(!drift.contains("last checked"), "must not claim a check happened:\n{}", r.out);
}

// ----- L-3: --json carries staleness additively (no schema bump) ------------

#[test]
fn rollup_json_staleness_additive() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    persist(
        root,
        node(7, NodeType::Slice, "DB layer").with_desired_facts(vec![volatile_fact(
            "db-up",
            "the prod DB answers",
            "false",
        )]),
    );

    assert!(run(root, &["reconcile"]).ok);
    let r = run(root, &["rollup", "--json"]);
    assert!(r.ok);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");
    // Schema is unchanged — the staleness is purely additive.
    assert_eq!(v["schema"], "rollup/v1");
    let d = &v["drift"]["drifted"][0];
    assert_eq!(d["fact_id"], "db-up");
    // A volatile drifted fact carries `{kind:"last-checked", at:<secs>}`.
    assert_eq!(d["freshness"]["kind"], "last-checked");
    assert!(d["freshness"]["at"].is_i64(), "absolute last-checked stamp:\n{v:#}");
    // The unchecked array exists (additive), empty here.
    assert!(v["drift"]["unchecked"].is_array());
}

#[test]
fn orient_json_staleness_additive() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    persist(root, node(1, NodeType::Project, "Proj"));
    persist(
        root,
        node(7, NodeType::Slice, "DB layer").with_desired_facts(vec![volatile_fact(
            "db-up",
            "the prod DB answers",
            "false",
        )]),
    );

    // Never checked → the drifted array is empty but `unchecked` names the fact,
    // additively, with no schema bump.
    let r = run(root, &["orient", "--json"]);
    assert!(r.ok);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");
    assert_eq!(v["schema"], "orient/v1");
    let u = &v["drift"]["unchecked"][0];
    assert_eq!(u["fact_id"], "db-up");
    assert_eq!(u["number"], 7);

    // After a reconcile, it moves to `drifted` with a last-checked stamp.
    assert!(run(root, &["reconcile"]).ok);
    let r2 = run(root, &["orient", "--json"]);
    let v2: serde_json::Value = serde_json::from_str(&r2.out).expect("valid JSON");
    assert_eq!(v2["drift"]["drifted"][0]["freshness"]["kind"], "last-checked");
}

// ----- L-4: the drift snapshot lives at the `.odm/drift` default path --------

#[test]
fn drift_snapshot_default_path() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    persist(
        root,
        node(1, NodeType::Slice, "Healthy").with_desired_facts(vec![volatile_fact(
            "up",
            "service is up",
            "true",
        )]),
    );

    assert!(run(root, &["reconcile"]).ok);
    assert!(root.join(".odm").join("drift").exists(), "snapshot at .odm/drift");
}

// ----- L-5: the ROLLUP.md early-cutoff is drift-aware ------------------------

#[test]
fn rollup_regenerates_on_drift_change() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    // An input-derived file fact: holds iff `flag.txt` exists. Toggling the flag
    // changes drift **without** changing the corpus (flag.txt is not a node).
    persist(
        root,
        node(1, NodeType::Slice, "Gated").with_desired_facts(vec![file_fact(
            "flag",
            "the flag is set",
            "flag.txt",
        )]),
    );

    // flag absent → drifted. First rollup captures the drifted state.
    assert!(run(root, &["rollup"]).ok);
    let before = read_rollup(root);
    assert!(before.split("## Drift").nth(1).unwrap().contains("flag"), "drifted:\n{before}");

    // Set the flag → drift clears. The corpus is unchanged, but the drift-aware
    // cutoff must still regenerate ROLLUP.md.
    std::fs::write(root.join("flag.txt"), b"set").unwrap();
    assert!(run(root, &["rollup"]).ok);
    let after = read_rollup(root);
    assert_ne!(before, after, "a drift change with no corpus change must regenerate");
    assert!(after.split("## Drift").nth(1).unwrap().contains("No drift"), "clean now:\n{after}");
}

#[test]
fn rollup_skips_when_drift_and_corpus_unchanged() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    // An input-derived fact that holds (flag present) → fresh, no volatile
    // timestamp, so the content-fingerprint is fully stable across runs.
    std::fs::write(root.join("flag.txt"), b"set").unwrap();
    persist(
        root,
        node(1, NodeType::Slice, "Gated").with_desired_facts(vec![file_fact(
            "flag",
            "the flag is set",
            "flag.txt",
        )]),
    );

    assert!(run(root, &["rollup"]).ok);
    let first = read_rollup(root);
    // Nothing changed (corpus + drift identical) → the cutoff skips the rewrite.
    assert!(run(root, &["rollup"]).ok);
    let second = read_rollup(root);
    assert_eq!(first, second, "unchanged corpus + drift must be byte-identical (skipped)");
}

// ----- L-6: `next` stays graph-pure — unaffected by a deferred marker --------

#[test]
fn next_unaffected_by_deferred_marker() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_config(root);
    // A deferred slice with no dependencies is still graph-ready: `next` answers
    // graph-readiness only and ignores the deferral (surfaced in rollup/orient).
    let deferred = node(1, NodeType::Slice, "Parked").with_deferred(Some(Deferral {
        because: "waiting on prod".to_string(),
        reenter_when: "back".to_string(),
    }));
    persist(root, deferred);

    let r = run(root, &["next"]);
    assert!(r.ok);
    assert!(r.out.contains("#1 Parked"), "deferred node is still graph-ready in next:\n{}", r.out);
    assert!(!r.out.contains("nothing ready"), "the deferred node must not be withheld:\n{}", r.out);

    // Sanity: the exit is clean and the deferral did not error the reader.
    assert_eq!(r.code, Some(0));
}
