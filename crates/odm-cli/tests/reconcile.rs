//! In-process tests for `odm reconcile` (arc05 slice03). Drives
//! [`odm_cli::dispatch`] against a temp store with captured buffers (the
//! established `odm-cli` pattern). Test names carry the substrings the ledger
//! Verify commands filter on (`reconcile_clean_reports_no_drift_exit_0`,
//! `reconcile_drift_reported_nonzero`, `reconcile_probe_error_surfaced_warning`,
//! `reconcile_strict_fails_on_probe_error`, `reconcile_json_schema`).
//!
//! Facts use **shell** probes against real commands (`true` → holds, `false` →
//! drift, a missing binary → error) so the three outcomes are deterministic.

use std::path::Path;

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{DesiredFact, Id, NodeType, Origin, ProbeSpec, ShellExpect};
use odm_store::Store;
use tempfile::TempDir;

struct Run {
    code: Option<u8>,
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, root, &mut out, &mut err);
    Run { code: result.ok(), out: String::from_utf8(out).unwrap() }
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()
}

/// A shell fact: `run` the command and expect exit `0`.
fn shell_fact(id: &str, describe: &str, run: &str) -> DesiredFact {
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

/// Seeds a node with a number, name, and declared facts.
fn seed(root: &Path, number: u32, name: &str, facts: Vec<DesiredFact>) {
    let fm =
        Frontmatter::new(Id::new(), number, NodeType::Slice, name, day(), day(), Origin::Planned)
            .with_desired_facts(facts);
    Store::open(root).persist(&Document::new(fm, "body\n")).expect("seed persist");
}

// ----- R-1: clean corpus → no drift, exit 0 ---------------------------------

#[test]
fn reconcile_clean_reports_no_drift_exit_0() {
    let dir = TempDir::new().unwrap();
    // A node whose fact holds (`true` exits 0).
    seed(dir.path(), 1, "Healthy", vec![shell_fact("up", "service is up", "true")]);

    let r = run(dir.path(), &["reconcile"]);
    assert_eq!(r.code, Some(0));
    assert!(r.out.contains("no drift"), "out: {}", r.out);

    // A corpus with no declared facts is also clean (no fabricated data).
    let empty = TempDir::new().unwrap();
    seed(empty.path(), 1, "Factless", vec![]);
    let r2 = run(empty.path(), &["reconcile"]);
    assert_eq!(r2.code, Some(0));
    assert!(r2.out.contains("no drift"), "out: {}", r2.out);
}

// ----- R-2: drift reported with identity, non-zero exit ---------------------

#[test]
fn reconcile_drift_reported_nonzero() {
    let dir = TempDir::new().unwrap();
    // `false` exits 1 where 0 was expected → drift.
    seed(dir.path(), 7, "DB layer", vec![shell_fact("db-up", "the prod DB answers", "false")]);

    let r = run(dir.path(), &["reconcile"]);
    assert_eq!(r.code, Some(1), "drift must exit non-zero");
    // Identity: node number + name, fact id, describe.
    assert!(r.out.contains("#7"), "out: {}", r.out);
    assert!(r.out.contains("DB layer"), "out: {}", r.out);
    assert!(r.out.contains("db-up"), "out: {}", r.out);
    assert!(r.out.contains("the prod DB answers"), "out: {}", r.out);
    // Declared-vs-observed.
    assert!(r.out.contains("expected:"), "out: {}", r.out);
    assert!(r.out.contains("observed:"), "out: {}", r.out);
}

// ----- R-3: probe error surfaced as a warning; --strict gates the exit ------

#[test]
fn reconcile_probe_error_surfaced_warning() {
    let dir = TempDir::new().unwrap();
    // A non-existent binary cannot run → probe Error ("couldn't check").
    seed(
        dir.path(),
        3,
        "Flaky check",
        vec![shell_fact("reachable", "host reachable", "odm-no-such-binary-xyzzy")],
    );

    let r = run(dir.path(), &["reconcile"]);
    // Surfaced distinctly from drift, but does not fail without --strict.
    assert_eq!(r.code, Some(0), "a probe error alone must not fail without --strict");
    assert!(r.out.contains("[error]"), "error must be surfaced: {}", r.out);
    assert!(r.out.contains("couldn't-check"), "out: {}", r.out);
    assert!(r.out.contains("reason:"), "out: {}", r.out);
}

#[test]
fn reconcile_strict_fails_on_probe_error() {
    let dir = TempDir::new().unwrap();
    seed(
        dir.path(),
        3,
        "Flaky check",
        vec![shell_fact("reachable", "host reachable", "odm-no-such-binary-xyzzy")],
    );

    let r = run(dir.path(), &["reconcile", "--strict"]);
    assert_eq!(r.code, Some(1), "--strict promotes a probe error to a failure");
}

// ----- R-5: --json emits reconcile/v1, 1:1 with the report ------------------

#[test]
fn reconcile_json_schema() {
    let dir = TempDir::new().unwrap();
    seed(dir.path(), 7, "DB layer", vec![shell_fact("db-up", "the prod DB answers", "false")]);

    let r = run(dir.path(), &["reconcile", "--json"]);
    assert_eq!(r.code, Some(1));
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");
    assert_eq!(v["schema"], "reconcile/v1");
    assert_eq!(v["ok"], false);
    assert_eq!(v["counts"]["drifted"], 1);
    assert_eq!(v["counts"]["holds"], 0);
    assert_eq!(v["counts"]["errored"], 0);
    let node = &v["nodes"][0];
    assert_eq!(node["number"], 7);
    assert_eq!(node["name"], "DB layer");
    let fact = &node["results"][0];
    assert_eq!(fact["fact_id"], "db-up");
    assert_eq!(fact["describe"], "the prod DB answers");
    assert_eq!(fact["outcome"]["kind"], "drifted");
    assert!(fact["outcome"]["expected"].is_string());
    assert!(fact["outcome"]["observed"].is_string());
}

// A clean run's JSON is well-formed and reports ok with zeroed drift/error.
#[test]
fn reconcile_json_clean_is_ok() {
    let dir = TempDir::new().unwrap();
    seed(dir.path(), 1, "Healthy", vec![shell_fact("up", "service is up", "true")]);
    let r = run(dir.path(), &["reconcile", "--json"]);
    assert_eq!(r.code, Some(0));
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");
    assert_eq!(v["schema"], "reconcile/v1");
    assert_eq!(v["ok"], true);
    assert_eq!(v["counts"]["holds"], 1);
    assert_eq!(v["nodes"][0]["results"][0]["outcome"]["kind"], "holds");
}
