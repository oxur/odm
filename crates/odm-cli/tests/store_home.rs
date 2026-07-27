//! arc-store-home slice 01 — commands operate on the **resolved** store home.
//!
//! The store is hand-placed here: this slice resolves and reads, it creates
//! nothing (no worktree, no branch — that is slice 02). What is under test is
//! that `[store]` redirects the node tree and that `config.toml` supplies the
//! operational settings, with both absences behaving exactly as before.

use std::fs;
use std::path::Path;

use clap::Parser as _;
use odm_cli::Cli;
use tempfile::TempDir;

/// The operational settings a work corpus needs to render and validate.
const OPERATIONAL: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

struct Run {
    code: Option<u8>,
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = odm_cli::dispatch(cli, root, &mut out, &mut err).ok();
    Run { code, out: String::from_utf8(out).unwrap() }
}

/// A repo whose `odm.toml` carries `locator`, with a store hand-placed at
/// `.worktrees/odm` holding `config.toml` and a seeded corpus.
fn repo_with_worktree_store(locator: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join("odm.toml"), locator).unwrap();

    let store = dir.path().join(".worktrees").join("odm");
    fs::create_dir_all(&store).unwrap();
    fs::write(store.join("config.toml"), OPERATIONAL).unwrap();
    dir
}

/// Seeds a project → arc → slice into the store rooted at `root`.
fn seed(root: &Path) {
    run(root, &["node", "new", "project", "Root project"]);
    run(root, &["node", "new", "arc", "An arc", "--parent", "1"]);
    run(root, &["node", "new", "slice", "A slice", "--parent", "2"]);
}

// ----- L-6: node reads/writes use the resolved store root --------------------

#[test]
fn nodes_are_written_into_the_resolved_store_not_the_repo_root() {
    let dir = repo_with_worktree_store("[store]\n");
    let store = dir.path().join(".worktrees").join("odm");

    // `new` is driven from the repo root, as a user would.
    seed(dir.path());

    assert!(store.join("nodes").is_dir(), "nodes landed in the worktree store");
    assert!(
        !dir.path().join("nodes").exists(),
        "and nothing was written at the repo root:\n{:?}",
        fs::read_dir(dir.path()).unwrap().flatten().map(|e| e.path()).collect::<Vec<_>>()
    );

    let r = run(dir.path(), &["node", "list"]);
    assert_eq!(r.code, Some(0));
    for name in ["Root project", "An arc", "A slice"] {
        assert!(r.out.contains(name), "{name} listed from the resolved store:\n{}", r.out);
    }
}

#[test]
fn a_custom_worktree_name_is_honoured() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(
        dir.path().join("odm.toml"),
        "[store]\nworktree_base = \"wt\"\nworktree_name = \"planning\"\n",
    )
    .unwrap();
    let store = dir.path().join("wt").join("planning");
    fs::create_dir_all(&store).unwrap();
    fs::write(store.join("config.toml"), OPERATIONAL).unwrap();

    seed(dir.path());
    assert!(store.join("nodes").is_dir(), "nodes follow the configured names");
}

// ----- L-9: `check` is green in both modes ----------------------------------

#[test]
fn check_is_green_against_the_resolved_store() {
    let dir = repo_with_worktree_store("[store]\n");
    seed(dir.path());

    let r = run(dir.path(), &["validate"]);
    assert_eq!(r.code, Some(0), "check green on the worktree store:\n{}", r.out);
    assert!(r.out.contains("3 node(s)"), "it validated the seeded corpus:\n{}", r.out);
}

// ----- L-3 / L-9: no `[store]` ⇒ unchanged behaviour -------------------------

#[test]
fn without_a_store_section_everything_stays_at_the_repo_root() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join("odm.toml"), OPERATIONAL).unwrap();

    seed(dir.path());

    assert!(dir.path().join("nodes").is_dir(), "nodes stay at the repo root");
    assert!(!dir.path().join(".worktrees").exists(), "no worktree is invented");

    let r = run(dir.path(), &["validate"]);
    assert_eq!(r.code, Some(0), "check green in the unredirected mode:\n{}", r.out);
    assert!(r.out.contains("3 node(s)"));
}

// ----- L-4: operational config comes from the store's `config.toml` ----------

#[test]
fn gate_sets_are_read_from_the_stores_config_toml() {
    // The locator carries *no* gate-sets; only the store's `config.toml` does.
    // A gate that validates therefore proves which file was read.
    let dir = repo_with_worktree_store("[store]\n");
    seed(dir.path());

    let r = run(dir.path(), &["node", "set-gate", "3", "built"]);
    assert_eq!(r.code, Some(0), "the slice gate-set came from config.toml");

    let listed = run(dir.path(), &["node", "list"]);
    assert!(listed.out.contains("built"), "the gate took effect:\n{}", listed.out);
}

#[test]
fn the_stores_config_wins_over_the_locator() {
    // Both files define `[gates.slice]`, with different sequences: only the
    // store's names `polished`, so accepting it proves precedence.
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(
        dir.path().join("odm.toml"),
        "[store]\n[gates.slice]\nsequence = [\"planned\", \"built\"]\n",
    )
    .unwrap();
    let store = dir.path().join(".worktrees").join("odm");
    fs::create_dir_all(&store).unwrap();
    fs::write(
        store.join("config.toml"),
        "[gates.slice]\nsequence = [\"planned\", \"built\", \"polished\"]\n",
    )
    .unwrap();

    run(dir.path(), &["node", "new", "slice", "A slice"]);
    let r = run(dir.path(), &["node", "set-gate", "1", "polished"]);
    assert_eq!(r.code, Some(0), "the store's config.toml won");
}

// ----- L-5: falling back to the locator before migration --------------------

#[test]
fn operational_config_falls_back_to_the_locator_when_the_store_has_none() {
    // A redirected store that has not yet been given its own `config.toml` —
    // the state every repo is in between slice 01 and the C-5 cutover.
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join("odm.toml"), format!("[store]\n{OPERATIONAL}")).unwrap();
    fs::create_dir_all(dir.path().join(".worktrees").join("odm")).unwrap();

    seed(dir.path());
    let r = run(dir.path(), &["node", "set-gate", "3", "built"]);
    assert_eq!(r.code, Some(0), "gate-sets still came from odm.toml");
    assert_eq!(run(dir.path(), &["validate"]).code, Some(0), "and check is green");
}
