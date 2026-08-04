//! `odm store set-remote` + `sync`'s first-push path (arc-store-lifecycle,
//! slice 06). Test ids (`F-N`) match
//! `docs/design-v1.0.0/arc-store-lifecycle/slice06-store-set-remote/ledger.md`.
//!
//! Fixture, class-(a) per the slice-doc: real bare-repo remotes (`git init
//! --bare`, real push/fetch, no mock) — the same discipline `store_sync.rs`
//! uses.

use std::path::Path;
use std::process::Command;

use clap::Parser as _;
use odm_cli::Cli;
use odm_store::init::{self, Plan};
use tempfile::TempDir;

struct Run {
    code: Option<u8>,
    out: String,
    err: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = odm_cli::dispatch(cli, root, &mut out, &mut err).ok();
    Run { code, out: String::from_utf8(out).unwrap(), err: String::from_utf8(err).unwrap() }
}

fn git_init(dir: &Path) {
    let git = |args: &[&str]| {
        Command::new("git").args(args).current_dir(dir).status().expect("git command");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    std::fs::write(dir.join(".gitkeep"), "").unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "initial"]);
}

fn bootstrapped_store() -> TempDir {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    let plan = Plan::new(dir.path(), None, None);
    init::bootstrap(&plan).expect("bootstrap");
    dir
}

/// Adds a real bare-repo remote named `name` to `repo`. Returns the bare
/// repo's `TempDir` so it outlives the test.
fn add_remote(repo: &Path, name: &str) -> TempDir {
    let bare = TempDir::new().unwrap();
    Command::new("git")
        .args(["init", "--bare", "-q"])
        .current_dir(bare.path())
        .status()
        .expect("git init --bare");
    Command::new("git")
        .args(["remote", "add", name, &bare.path().display().to_string()])
        .current_dir(repo)
        .status()
        .expect("git remote add");
    bare
}

/// The configured `[store].remote` in `repo`'s `odm.toml`, or `None` if the
/// key is absent.
fn configured_remote(repo: &Path) -> Option<String> {
    let text = std::fs::read_to_string(repo.join("odm.toml")).ok()?;
    let doc: toml::Value = text.parse().ok()?;
    doc.get("store")?.get("remote")?.as_str().map(str::to_string)
}

/// `HEAD`'s commit id for `branch` in `dir` — works against a bare repo too;
/// empty when the branch doesn't exist there yet.
fn branch_oid(dir: &Path, branch: &str) -> String {
    let out = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", branch])
        .current_dir(dir)
        .output()
        .expect("git rev-parse");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

// ----- F-1: explicit set-remote --------------------------------------------------

#[test]
fn set_remote_explicit() {
    let dir = bootstrapped_store();
    let _bare = add_remote(dir.path(), "origin");

    let r = run(dir.path(), &["store", "set-remote", "origin"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert_eq!(configured_remote(dir.path()).as_deref(), Some("origin"));
}

#[test]
fn set_remote_explicit_nonexistent() {
    let dir = bootstrapped_store();

    let r = run(dir.path(), &["store", "set-remote", "nope"]);
    assert_eq!(r.code, None, "a nonexistent remote is an error, not a silent write");
    assert!(configured_remote(dir.path()).is_none());
}

#[test]
fn set_remote_explicit_overwrite() {
    let dir = bootstrapped_store();
    let _origin = add_remote(dir.path(), "origin");
    let _other = add_remote(dir.path(), "other");

    assert_eq!(run(dir.path(), &["store", "set-remote", "origin"]).code, Some(0));
    assert_eq!(configured_remote(dir.path()).as_deref(), Some("origin"));

    assert_eq!(run(dir.path(), &["store", "set-remote", "other"]).code, Some(0));
    assert_eq!(configured_remote(dir.path()).as_deref(), Some("other"), "the second value wins");
}

// ----- F-2: auto-detect ------------------------------------------------------------

#[test]
fn set_remote_auto_one() {
    let dir = bootstrapped_store();
    let _bare = add_remote(dir.path(), "origin");

    let r = run(dir.path(), &["store", "set-remote", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["remote"], "origin", "{v}");
    assert_eq!(v["auto_detected"], true, "{v}");
    assert_eq!(configured_remote(dir.path()).as_deref(), Some("origin"));
}

#[test]
fn set_remote_auto_zero() {
    let dir = bootstrapped_store();

    let r = run(dir.path(), &["store", "set-remote"]);
    assert_eq!(r.code, None, "zero remotes is an error");
}

#[test]
fn set_remote_auto_multi() {
    let dir = bootstrapped_store();
    let _a = add_remote(dir.path(), "origin");
    let _b = add_remote(dir.path(), "upstream");

    let r = run(dir.path(), &["store", "set-remote"]);
    assert_eq!(r.code, None, "multiple remotes is an error");
    assert!(configured_remote(dir.path()).is_none(), "nothing written on ambiguity");
}

// ----- F-3: sync's first-push path --------------------------------------------------

#[test]
fn sync_first_push() {
    let dir = bootstrapped_store();
    let bare = add_remote(dir.path(), "origin");
    assert_eq!(run(dir.path(), &["store", "set-remote", "origin"]).code, Some(0));

    run(dir.path(), &["node", "new", "project", "Root project"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    assert!(branch_oid(bare.path(), "odm").is_empty(), "not yet on the remote");

    let r = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["action"], "pushed", "{v}");

    let store = dir.path().join(".worktrees").join("odm");
    assert_eq!(
        branch_oid(bare.path(), "odm"),
        branch_oid(&store, "HEAD"),
        "the first push landed the branch on the remote"
    );
}

#[test]
fn sync_first_push_dry_run() {
    let dir = bootstrapped_store();
    let bare = add_remote(dir.path(), "origin");
    assert_eq!(run(dir.path(), &["store", "set-remote", "origin"]).code, Some(0));

    run(dir.path(), &["node", "new", "project", "Root project"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "sync", "--dry-run", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["action"], "pushed", "{v}");
    assert_eq!(v["dry_run"], true, "{v}");

    assert!(branch_oid(bare.path(), "odm").is_empty(), "dry-run pushed nothing");

    // A real sync afterward still performs the first push.
    let real = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(real.code, Some(0), "{}", real.err);
    assert!(!branch_oid(bare.path(), "odm").is_empty());
}

// ----- F-4: sync reads the configured remote (or falls back) -----------------------

#[test]
fn sync_reads_configured_remote() {
    let dir = bootstrapped_store();
    // Only a non-"origin" remote exists — if sync ignored the configured
    // remote and fell back to the "origin" default, this would fail outright
    // (no such remote), so a passing push proves it read the config.
    let bare = add_remote(dir.path(), "github");
    assert_eq!(run(dir.path(), &["store", "set-remote", "github"]).code, Some(0));

    run(dir.path(), &["node", "new", "project", "Root project"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["upstream"]["ref"], "github/odm", "{v}");
    assert!(!branch_oid(bare.path(), "odm").is_empty(), "pushed to the configured remote");
}

#[test]
fn sync_falls_back_to_default() {
    let dir = bootstrapped_store();
    // A remote named "origin" exists, but `set-remote` was never run — no
    // `remote` field in odm.toml at all.
    let bare = add_remote(dir.path(), "origin");
    assert!(configured_remote(dir.path()).is_none());

    run(dir.path(), &["node", "new", "project", "Root project"]);
    assert_eq!(run(dir.path(), &["store", "commit"]).code, Some(0));

    let r = run(dir.path(), &["store", "sync", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["upstream"]["ref"], "origin/odm", "{v}");
    assert!(!branch_oid(bare.path(), "odm").is_empty(), "pushed via the default fallback");
}

// ----- F-5: `store init` auto-sets the remote ---------------------------------------

#[test]
fn init_auto_sets_remote() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    let _bare = add_remote(dir.path(), "origin");

    let r = run(dir.path(), &["store", "init"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert_eq!(configured_remote(dir.path()).as_deref(), Some("origin"));
}

#[test]
fn init_no_remote_no_field() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());

    let r = run(dir.path(), &["store", "init"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(configured_remote(dir.path()).is_none(), "nothing to auto-detect, nothing written");
}

#[test]
fn init_multi_remote_no_field() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    let _a = add_remote(dir.path(), "origin");
    let _b = add_remote(dir.path(), "upstream");

    let r = run(dir.path(), &["store", "init"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    assert!(configured_remote(dir.path()).is_none(), "ambiguous — requires explicit set-remote");
}

// ----- F-6: `--json` shape ----------------------------------------------------------

#[test]
fn set_remote_json() {
    let dir = bootstrapped_store();
    let _bare = add_remote(dir.path(), "origin");

    let r = run(dir.path(), &["store", "set-remote", "origin", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["remote"], "origin", "{v}");
    assert!(v["url"].is_string(), "{v}");
    assert_eq!(v["auto_detected"], false, "{v}");
    assert_eq!(v["branch"], "odm", "{v}");
    assert!(v["store_root"].is_string(), "{v}");
}

#[test]
fn set_remote_json_auto() {
    let dir = bootstrapped_store();
    let _bare = add_remote(dir.path(), "origin");

    let r = run(dir.path(), &["store", "set-remote", "--json"]);
    assert_eq!(r.code, Some(0), "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).expect("valid json");
    assert_eq!(v["auto_detected"], true, "{v}");
}

// ----- Guard: no `[store]` section means nothing to configure -----------------------

#[test]
fn without_a_store_section_set_remote_refuses() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("odm.toml"), "author_name = \"Ada\"\n").unwrap();

    let r = run(dir.path(), &["store", "set-remote", "origin"]);
    assert_eq!(r.code, None, "no `[store]` section is an error, not a silent write");
}
