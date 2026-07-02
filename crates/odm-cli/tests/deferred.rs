//! In-process tests for deferred surfacing (arc05 slice06). Drives
//! [`odm_cli::dispatch`] against a temp store. Test names carry the substrings
//! the ledger Verify commands filter on (`deferred_ready_when_reenter_fact_holds`,
//! `deferred_waiting_when_reenter_fact_drifts`, `rollup_surfaces_deferred`,
//! `rollup_no_deferred_section_when_none`, `orient_surfaces_deferred`,
//! `orient_no_deferred_when_none`, `rollup_json_includes_deferred`,
//! `orient_json_includes_deferred`).
//!
//! The re-entry predicate is a **shell** fact (`true` → Holds → ready; `false` →
//! Drifted → waiting), so the ready/waiting split is deterministic.

use std::path::Path;

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Deferral, Document, Frontmatter};
use odm_core::{DesiredFact, Id, NodeType, Origin, ProbeSpec, ShellExpect};
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
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, root, &mut out, &mut err);
    Run { ok: result.is_ok(), out: String::from_utf8(out).unwrap() }
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
}

fn write_config(root: &Path) {
    std::fs::write(root.join("odm.toml"), CONFIG).unwrap();
}

fn read_rollup(root: &Path) -> String {
    std::fs::read_to_string(root.join("ROLLUP.md")).expect("ROLLUP.md")
}

fn persist(root: &Path, fm: Frontmatter) {
    Store::open(root).persist(&Document::new(fm, "body\n")).expect("persist");
}

fn node(number: u32, ty: NodeType, name: &str) -> Frontmatter {
    Frontmatter::new(Id::new(), number, ty, name, day(), day(), Origin::Planned)
}

/// A deferred slice: one shell fact (`reenter_when`) + the deferred marker.
fn deferred_slice(number: u32, name: &str, because: &str, run: &str) -> Frontmatter {
    let fact = DesiredFact {
        id: "back".to_string(),
        describe: "the blocker is cleared".to_string(),
        probe: ProbeSpec::Shell {
            run: run.to_string(),
            expect: ShellExpect { exit: 0, stdout_contains: None },
        },
    };
    node(number, NodeType::Slice, name).with_desired_facts(vec![fact]).with_deferred(Some(
        Deferral { because: because.to_string(), reenter_when: "back".to_string() },
    ))
}

// ----- D-2/D-3: rollup surfaces deferred; ready vs waiting -------------------

#[test]
fn deferred_ready_when_reenter_fact_holds() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path());
    // `true` holds → the re-entry condition is met → ready to re-enter.
    persist(dir.path(), deferred_slice(1, "Parked", "waiting on prod", "true"));

    assert!(run(dir.path(), &["rollup"]).ok);
    let md = read_rollup(dir.path());
    let section = md.split("## Deferred").nth(1).expect("a Deferred section");
    assert!(section.contains("#1 Parked"), "identity:\n{md}");
    assert!(section.contains("waiting on prod"), "because:\n{md}");
    assert!(section.contains("ready to re-enter"), "ready status:\n{md}");
}

#[test]
fn deferred_waiting_when_reenter_fact_drifts() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path());
    // `false` drifts → the re-entry condition is not met → still waiting.
    persist(dir.path(), deferred_slice(1, "Parked", "waiting on prod", "false"));

    assert!(run(dir.path(), &["rollup"]).ok);
    let md = read_rollup(dir.path());
    let section = md.split("## Deferred").nth(1).expect("a Deferred section");
    assert!(section.contains("waiting on the blocker is cleared"), "waiting status:\n{md}");
}

#[test]
fn rollup_surfaces_deferred() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path());
    persist(dir.path(), deferred_slice(3, "Blocked work", "external dep", "false"));
    assert!(run(dir.path(), &["rollup"]).ok);
    let md = read_rollup(dir.path());
    assert!(md.contains("## Deferred"), "Deferred section present:\n{md}");
    assert!(md.contains("#3 Blocked work"), "identity:\n{md}");
}

#[test]
fn rollup_no_deferred_section_when_none() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path());
    // A non-deferred node → no Deferred section at all (no fabricated data).
    persist(dir.path(), node(1, NodeType::Slice, "Active"));
    assert!(run(dir.path(), &["rollup"]).ok);
    let md = read_rollup(dir.path());
    assert!(!md.contains("## Deferred"), "no Deferred section when none:\n{md}");
}

// ----- D-4: orient surfaces deferred ----------------------------------------

#[test]
fn orient_surfaces_deferred() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path());
    persist(dir.path(), node(1, NodeType::Project, "Proj")); // single project auto-resolves
    persist(dir.path(), deferred_slice(2, "Parked", "waiting on prod", "false"));

    let r = run(dir.path(), &["orient"]);
    assert!(r.ok);
    let section = r.out.split("DEFERRED").nth(1).expect("a DEFERRED section");
    assert!(section.contains("#2 Parked"), "identity:\n{}", r.out);
    assert!(section.contains("waiting on"), "status:\n{}", r.out);
}

#[test]
fn orient_no_deferred_when_none() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path());
    persist(dir.path(), node(1, NodeType::Project, "Proj"));
    let r = run(dir.path(), &["orient"]);
    assert!(r.ok);
    assert!(!r.out.contains("DEFERRED"), "no DEFERRED section when none:\n{}", r.out);
}

// ----- D-6: --json carries the deferred slot additively ---------------------

#[test]
fn rollup_json_includes_deferred() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path());
    persist(dir.path(), deferred_slice(3, "Blocked work", "external dep", "true"));

    let r = run(dir.path(), &["rollup", "--json"]);
    assert!(r.ok);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");
    assert_eq!(v["schema"], "rollup/v1");
    let d = &v["deferred"][0];
    assert_eq!(d["number"], 3);
    assert_eq!(d["because"], "external dep");
    assert_eq!(d["reentry"]["status"], "ready");
}

#[test]
fn orient_json_includes_deferred() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path());
    persist(dir.path(), node(1, NodeType::Project, "Proj"));
    persist(dir.path(), deferred_slice(2, "Parked", "external dep", "false"));

    let r = run(dir.path(), &["orient", "--json"]);
    assert!(r.ok);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid JSON");
    assert_eq!(v["schema"], "orient/v1");
    let d = &v["deferred"][0];
    assert_eq!(d["number"], 2);
    assert_eq!(d["reentry"]["status"], "waiting");
    assert!(d["reentry"]["waiting_on"].is_string());
}
