//! In-process tests for the self-host cutover at the CLI surface (arc06 slice04).
//! Builds a self-hosted temp store from odm's **real** `docs/design` (odd nodes)
//! plus `docs/design-v1.0.0` (work nodes), then runs `check`/`rollup`/`orient`
//! over the mixed corpus. Test names carry the ledger Verify substrings
//! (`check_green_on_self_hosted_corpus` S-4; `check_no_orphan_work_nodes` S-2;
//! `rollup_reflects_self_hosted_state` and `orient_runs_on_self_hosted_corpus` S-5).

use std::path::{Path, PathBuf};

use clap::Parser;
use odm_cli::Cli;
use serde_json::Value;
use tempfile::TempDir;

/// The workspace root (odm-cli manifest dir is `crates/odm-cli`).
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The `[gates.*]` config the self-hosted corpus needs so `rollup`/`check` can
/// render/validate work-node status (mirrors the repo `odm.toml`).
const GATES: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
[gates.odd]
sequence = [\"draft\", \"under-review\", \"revised\", \"accepted\", \"active\", \"final\"]
";

fn run(root: &Path, args: &[&str]) -> (Option<u8>, String, String) {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = odm_cli::dispatch(cli, root, &mut out, &mut err).ok();
    (code, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

/// Sets up a self-hosted temp store: writes the gate config, imports the real
/// `docs/design` ODDs, and self-hosts the real `design-v1.0.0` plan set. Returns
/// the TempDir (kept alive by the caller).
fn self_hosted_store(with_gates: bool) -> TempDir {
    let dir = TempDir::new().unwrap();
    if with_gates {
        std::fs::write(dir.path().join("odm.toml"), GATES).unwrap();
    }
    let docs = repo().join("docs/design");
    let plan = repo().join("docs/design-v1.0.0");
    let (m, _o, _e) = run(dir.path(), &["migrate", docs.to_str().unwrap()]);
    assert_eq!(m, Some(0), "migrate odd corpus");
    let (s, _o, _e) = run(dir.path(), &["self-host", plan.to_str().unwrap()]);
    assert_eq!(s, Some(0), "self-host plan set");
    dir
}

// ----- S-4: mixed-corpus `check` is green ------------------------------------

#[test]
fn check_green_on_self_hosted_corpus() {
    let dir = self_hosted_store(true);
    let (code, out) = {
        let (c, o, _e) = run(dir.path(), &["check"]);
        (c, o)
    };
    assert_eq!(code, Some(0), "check green on the mixed odd+work corpus:\n{out}");
}

// ----- S-2: no orphan work nodes ---------------------------------------------

#[test]
fn check_no_orphan_work_nodes() {
    let dir = self_hosted_store(true);
    let (code, out, _e) = run(dir.path(), &["check"]);
    assert_eq!(code, Some(0));
    assert!(!out.contains("orphan"), "no orphan work nodes (all parented):\n{out}");
}

// ----- S-5: rollup reproduces the real project state -------------------------

/// Finds a tree node by `number` anywhere in the rollup tree.
fn find_node(v: &Value, number: u64) -> Option<Value> {
    if v["number"].as_u64() == Some(number) {
        return Some(v.clone());
    }
    for child in v["children"].as_array()?.iter() {
        if let Some(found) = find_node(child, number) {
            return Some(found);
        }
    }
    None
}

/// The gate names a node has reached (evidence non-null) in its status vector.
fn reached_gates(node: &Value) -> Vec<String> {
    node["status"]
        .as_array()
        .map(|gs| {
            gs.iter()
                .filter(|g| !g["evidence"].is_null())
                .filter_map(|g| g["gate"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn rollup_reflects_self_hosted_state() {
    let dir = self_hosted_store(true);
    let (code, out, _e) = run(dir.path(), &["rollup", "--json"]);
    assert_eq!(code, Some(0));
    let v: Value = serde_json::from_str(&out).expect("valid rollup JSON");
    assert_eq!(v["schema"], "rollup/v1");

    // The project (#1000) is a forest root with the arcs beneath it.
    let project = v["tree"].as_array().unwrap().iter().find(|n| n["number"] == 1000).cloned();
    let project = project.expect("project #1000 is a tree root");
    assert_eq!(project["type"], "project");

    // A5 (#1500) is closed → reached its terminal gate; A6 (#1600) is active →
    // reached in-progress but NOT complete. (The loop-closing reproduction.)
    let a5 = find_node(&project, 1500).expect("arc A5 in the tree");
    assert!(
        reached_gates(&a5).contains(&"verified".to_string()),
        "A5 done: {:?}",
        reached_gates(&a5)
    );
    let a6 = find_node(&project, 1600).expect("arc A6 in the tree");
    assert!(reached_gates(&a6).contains(&"in-progress".to_string()), "A6 active");
    assert!(!reached_gates(&a6).contains(&"complete".to_string()), "A6 not complete");
}

// ----- S-5: orient runs coherently over the self-hosted corpus ---------------

#[test]
fn orient_runs_on_self_hosted_corpus() {
    let dir = self_hosted_store(true);
    let (code, out, _e) = run(dir.path(), &["orient"]);
    assert_eq!(code, Some(0), "orient runs clean over the self-hosted corpus:\n{out}");
    // A coherent brief: the single self-hosted project auto-resolves and orient
    // renders its sections (vision → … ). It must not be the no-project fallback.
    assert!(!out.contains("odm new project"), "a project is resolved (not the empty fallback)");
}
