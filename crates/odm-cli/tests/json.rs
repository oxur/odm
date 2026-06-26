//! `--json` schema tests for `rollup` / `orient` / `check` (arc03 slice04).
//!
//! Drives [`odm_cli::dispatch`] against a seeded temp store, parses the emitted
//! JSON with `serde_json`, and shape-locks each envelope (keys + types) so
//! accidental drift fails CI. Test names carry the substrings the slice04 ledger
//! Verify commands filter on (`rollup_json_serializes_model`,
//! `orient_json_serializes_view`, `check_json_envelope_shape_locked`,
//! `rollup_json_shape_locked`, `orient_json_shape_locked`,
//! `json_schema_version_marker`, `json_valid_on_empty_and_no_project`,
//! `orient_rollup_affordances_name_fixes`).

use std::path::Path;
use std::str::FromStr;

use chrono::NaiveDate;
use clap::Parser;
use odm_cli::Cli;
use odm_core::frontmatter::{Dependency, Document, Frontmatter, TornEdge};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use serde_json::Value;
use tempfile::TempDir;

const CONFIG: &str = "\
[gates.project]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]

[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]

[gates.slice]
sequence = [\"planned\", \"built\", \"tested\"]
";

const VISION_BODY: &str = "# Vision\n\nMake the plan legible from one cheap call.\n";

struct Run {
    ok: bool,
    code: Option<u8>,
    out: String,
}

fn run(root: &Path, args: &[&str]) -> Run {
    let argv: Vec<&str> = std::iter::once("odm").chain(args.iter().copied()).collect();
    let cli = Cli::try_parse_from(&argv).expect("args should be structurally valid");
    let mut out = Vec::new();
    let mut err = Vec::new();
    let result = odm_cli::dispatch(cli, root, &mut out, &mut err);
    Run {
        ok: result.is_ok(),
        code: result.as_ref().ok().copied(),
        out: String::from_utf8(out).unwrap(),
    }
}

/// Parses the stdout of `odm <args>` as JSON, asserting the command succeeded.
fn run_json(root: &Path, args: &[&str]) -> Value {
    let r = run(root, args);
    assert!(r.ok, "command {args:?} should not bare-error");
    serde_json::from_str(&r.out)
        .unwrap_or_else(|e| panic!("invalid JSON from {args:?}: {e}\n{}", r.out))
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 26).unwrap()
}

fn write_config(root: &Path) {
    std::fs::write(root.join("odm.toml"), CONFIG).unwrap();
}

fn id(tag: char) -> Id {
    Id::from_str(&format!("01ARZ3NDEKTSV4RRFFQ69G5F{tag}0")).unwrap()
}

fn persist(root: &Path, doc: Document) {
    Store::open(root).persist(&doc).expect("seed persist");
}

fn fm(tag: char, number: u32, ty: NodeType, name: &str, origin: Origin) -> Frontmatter {
    Frontmatter::new(id(tag), number, ty, name, day(), day(), origin)
}

/// Sorted top-level keys of a JSON object.
fn keys(v: &Value) -> Vec<String> {
    let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
    k.sort();
    k
}

/// A corpus rich enough that every envelope array is non-empty:
/// - tree: project P(1) ← arc A(2) ← slices Early(3), Late(4), X(5), Y(6), W(7), V(8)
/// - ready: P, A, Early, V (soft on complete-at-attested W)
/// - blocked: Late (unsatisfied Early), X & Y (unsatisfied each other)
/// - tears: Y depends_on X, torn (because …)
/// - provenance: X discovered, Y amendment, rest planned
/// - integrity: a parentless Orphan(9) → orphan error
fn seed_rich(root: &Path) {
    persist(
        root,
        Document::new(fm('P', 1, NodeType::Project, "Proj", Origin::Planned), VISION_BODY),
    );

    let mut a = fm('A', 2, NodeType::Arc, "Arc one", Origin::Planned);
    a.edges_mut().part_of = Some(id('P'));
    persist(root, Document::new(a, "# Arc one\n"));

    let mut early = fm('E', 3, NodeType::Slice, "Early", Origin::Planned);
    early.edges_mut().part_of = Some(id('A'));
    persist(root, Document::new(early, "# Early\n"));

    let mut late = fm('T', 4, NodeType::Slice, "Late", Origin::Planned);
    late.edges_mut().part_of = Some(id('A'));
    late.edges_mut().depends_on.push(Dependency::Bare(id('E')));
    persist(root, Document::new(late, "# Late\n"));

    // X ↔ Y cycle, broken by tearing Y depends_on X (with a rationale).
    let mut x = fm('X', 5, NodeType::Slice, "Exes", Origin::Discovered);
    x.edges_mut().part_of = Some(id('A'));
    x.edges_mut().depends_on.push(Dependency::Bare(id('Y')));
    persist(root, Document::new(x, "# Exes\n"));

    let mut y = fm('Y', 6, NodeType::Slice, "Whys", Origin::Amendment);
    y.edges_mut().part_of = Some(id('A'));
    y.edges_mut().depends_on.push(Dependency::Bare(id('X')));
    y.edges_mut().tears.push(TornEdge {
        edge: Dependency::Bare(id('X')),
        because: "assume X to break X-Y".to_string(),
    });
    persist(root, Document::new(y, "# Whys\n"));

    // W is complete only at attested; V depends_on W ⇒ ready with a soft flag.
    let mut w = fm('W', 7, NodeType::Slice, "Dubya", Origin::Planned);
    w.edges_mut().part_of = Some(id('A'));
    persist(root, Document::new(w, "# Dubya\n"));

    let mut v = fm('V', 8, NodeType::Slice, "Vee", Origin::Planned);
    v.edges_mut().part_of = Some(id('A'));
    v.edges_mut().depends_on.push(Dependency::Bare(id('W')));
    persist(root, Document::new(v, "# Vee\n"));

    persist(
        root,
        Document::new(fm('R', 9, NodeType::Slice, "Orphan", Origin::Planned), "# Orphan\n"),
    );
}

/// Seeds the rich corpus and records W's terminal gate at `attested`.
fn setup_rich(root: &Path) {
    write_config(root);
    seed_rich(root);
    run(root, &["set-gate", "Dubya", "tested", "--evidence", "attested"]);
    run(root, &["use", "project", "Proj"]);
    run(root, &["use", "arc", "Arc one"]);
}

// ----- J-1: rollup --json serializes the model ------------------------------

#[test]
fn rollup_json_serializes_model() {
    let dir = TempDir::new().unwrap();
    setup_rich(dir.path());
    let v = run_json(dir.path(), &["rollup", "--json"]);

    // Tree carries nested children + a gate-sequence status vector.
    let root_node = &v["tree"][0];
    assert_eq!(root_node["type"], "project");
    let arc = &root_node["children"][0];
    assert_eq!(arc["type"], "arc");
    let gates: Vec<&str> =
        arc["status"].as_array().unwrap().iter().map(|g| g["gate"].as_str().unwrap()).collect();
    assert_eq!(gates, ["planned", "in-progress", "complete", "verified"], "gate-sequence order");

    // Ready carries the soft-sat flag; blocked names reasons; tears carry rationale.
    let ready_soft = v["ready"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["soft"].as_array().is_some_and(|s| !s.is_empty()));
    assert!(ready_soft, "a ready node carries a soft dep:\n{v:#}");
    assert!(!v["blocked"].as_array().unwrap().is_empty(), "blocked non-empty");
    assert_eq!(v["tears"][0]["because"], "assume X to break X-Y");
    assert_eq!(v["provenance"]["discovered"][0]["name"], "Exes");
    assert_eq!(v["provenance"]["amendment"][0]["name"], "Whys");
}

// ----- J-1 (cont.): every block-reason kind serializes ----------------------

#[test]
fn rollup_json_block_reason_variants() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path());
    persist(
        dir.path(),
        Document::new(fm('P', 1, NodeType::Project, "Proj", Origin::Planned), VISION_BODY),
    );
    // Soft (terminal at attested), Hard (none), Ext (incomplete); Reader is
    // blocked with all three reason kinds.
    let mut soft = fm('S', 2, NodeType::Slice, "Softdep", Origin::Planned);
    soft.edges_mut().part_of = Some(id('P'));
    persist(dir.path(), Document::new(soft, "# Softdep\n"));
    let mut hard = fm('H', 3, NodeType::Slice, "Harddep", Origin::Planned);
    hard.edges_mut().part_of = Some(id('P'));
    persist(dir.path(), Document::new(hard, "# Harddep\n"));
    let mut ext = fm('X', 4, NodeType::Slice, "Extdep", Origin::Planned);
    ext.edges_mut().part_of = Some(id('P'));
    persist(dir.path(), Document::new(ext, "# Extdep\n"));
    let mut reader = fm('E', 5, NodeType::Slice, "Reader", Origin::Planned);
    reader.edges_mut().part_of = Some(id('P'));
    reader.edges_mut().depends_on.push(Dependency::Bare(id('S')));
    reader.edges_mut().depends_on.push(Dependency::Bare(id('H')));
    reader.edges_mut().blocked_by.push(id('X'));
    persist(dir.path(), Document::new(reader, "# Reader\n"));
    run(dir.path(), &["set-gate", "Softdep", "tested", "--evidence", "attested"]);

    let v = run_json(dir.path(), &["rollup", "--json"]);
    let reader_block = v["blocked"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["node"]["name"] == "Reader")
        .expect("Reader is blocked");
    let kinds: Vec<&str> = reader_block["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"unsatisfied"), "unsatisfied kind:\n{v:#}");
    assert!(kinds.contains(&"soft-satisfied"), "soft-satisfied kind:\n{v:#}");
    assert!(kinds.contains(&"externally-blocked"), "externally-blocked kind:\n{v:#}");
    // The soft-satisfied reason carries evidence + threshold.
    let soft = reader_block["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "soft-satisfied")
        .unwrap();
    assert_eq!(soft["evidence"], "attested");
    assert_eq!(soft["threshold"], "reproduced");
}

// ----- J-2: orient --json serializes the view -------------------------------

#[test]
fn orient_json_serializes_view() {
    let dir = TempDir::new().unwrap();
    setup_rich(dir.path());
    let v = run_json(dir.path(), &["orient", "--json"]);

    assert_eq!(v["project"]["name"], "Proj");
    assert_eq!(v["vision"], "Make the plan legible from one cheap call.");
    assert_eq!(v["focus"]["arc"]["name"], "Arc one");
    assert!(!v["ready"].as_array().unwrap().is_empty());
    assert!(!v["blocked"].as_array().unwrap().is_empty());
    // Integrity surfaces the orphan as an error.
    let codes: Vec<&str> =
        v["integrity"].as_array().unwrap().iter().map(|f| f["code"].as_str().unwrap()).collect();
    assert!(codes.contains(&"orphan"), "orphan surfaced in integrity:\n{v:#}");
    assert_eq!(v["integrity"][0]["severity"], "error");

    // `brief --json` is identical.
    let b = run_json(dir.path(), &["brief", "--json"]);
    assert_eq!(v, b, "brief --json == orient --json");
}

// ----- J-3: the check v2 envelope is shape-locked ---------------------------

#[test]
fn check_json_envelope_shape_locked() {
    let dir = TempDir::new().unwrap();
    setup_rich(dir.path()); // includes an orphan (a finding) + a tear

    let v = run_json(dir.path(), &["check", "--json"]);
    assert_eq!(keys(&v), ["errors", "findings", "ok", "schema", "tears", "warnings"]);
    assert!(v["ok"].is_boolean() && v["errors"].is_u64() && v["warnings"].is_u64());

    // A finding's field set (EntryJson).
    let finding = &v["findings"][0];
    assert_eq!(keys(finding), ["code", "detail", "fix", "name", "node", "number", "severity"]);
    // A tear's field set (TearJson).
    let tear = &v["tears"][0];
    assert_eq!(keys(tear), ["because", "from", "to"]);
}

// ----- J-4: rollup / orient envelopes are shape-locked ----------------------

#[test]
fn rollup_json_shape_locked() {
    let dir = TempDir::new().unwrap();
    setup_rich(dir.path());
    let v = run_json(dir.path(), &["rollup", "--json"]);

    assert_eq!(
        keys(&v),
        ["blocked", "deferred", "drift", "provenance", "ready", "schema", "tears", "tree"]
    );
    assert!(v["tree"].is_array() && v["ready"].is_array() && v["blocked"].is_array());
    assert!(v["tears"].is_array() && v["deferred"].is_array());
    assert_eq!(keys(&v["provenance"]), ["amendment", "discovered", "planned"]);
    assert_eq!(keys(&v["drift"]), ["tracked"]);

    // A tree node's field set.
    assert_eq!(
        keys(&v["tree"][0]),
        ["children", "id", "name", "number", "origin", "status", "type"]
    );
    // A blocked reason is internally tagged by `kind`.
    let reason = &v["blocked"][0]["reasons"][0];
    assert!(reason["kind"].is_string(), "block reason tagged by kind:\n{v:#}");
}

#[test]
fn orient_json_shape_locked() {
    let dir = TempDir::new().unwrap();
    setup_rich(dir.path());
    let v = run_json(dir.path(), &["orient", "--json"]);

    assert_eq!(
        keys(&v),
        ["blocked", "drift", "focus", "hint", "integrity", "project", "ready", "schema", "vision"]
    );
    assert_eq!(keys(&v["focus"]), ["arc", "status"]);
    assert_eq!(keys(&v["integrity"][0]), ["code", "detail", "severity", "who"]);
}

// ----- J-5: additive schema_version marker ----------------------------------

#[test]
fn json_schema_version_marker() {
    let dir = TempDir::new().unwrap();
    setup_rich(dir.path());

    assert_eq!(run_json(dir.path(), &["check", "--json"])["schema"], "check/v1");
    assert_eq!(run_json(dir.path(), &["rollup", "--json"])["schema"], "rollup/v1");
    assert_eq!(run_json(dir.path(), &["orient", "--json"])["schema"], "orient/v1");
}

// ----- J-7: valid JSON on the empty corpus + the no-project paths -----------

#[test]
fn json_valid_on_empty_and_no_project() {
    // Empty corpus: rollup --json parses; orient --json is the no-project fallback.
    let empty = TempDir::new().unwrap();
    write_config(empty.path());
    let r = run_json(empty.path(), &["rollup", "--json"]);
    assert!(r["tree"].as_array().unwrap().is_empty(), "empty tree:\n{r:#}");
    let o = run(empty.path(), &["orient", "--json"]);
    assert!(o.ok && o.code == Some(0), "orient --json on empty exits 0");
    let ov: Value = serde_json::from_str(&o.out).expect("valid JSON");
    assert!(ov["project"].is_null() && ov["hint"].is_string(), "no-project fallback:\n{ov:#}");

    // Exactly one project: orient --json resolves it (project non-null).
    let one = TempDir::new().unwrap();
    write_config(one.path());
    persist(
        one.path(),
        Document::new(fm('P', 1, NodeType::Project, "Solo", Origin::Planned), VISION_BODY),
    );
    let ov = run_json(one.path(), &["orient", "--json"]);
    assert_eq!(ov["project"]["name"], "Solo");
    assert!(ov["hint"].is_null());

    // Multiple projects, no context: fallback with a hint, still valid + exit 0.
    let many = TempDir::new().unwrap();
    write_config(many.path());
    persist(
        many.path(),
        Document::new(fm('A', 1, NodeType::Project, "Alpha", Origin::Planned), "# a\n"),
    );
    persist(
        many.path(),
        Document::new(fm('B', 2, NodeType::Project, "Beta", Origin::Planned), "# b\n"),
    );
    let ov = run_json(many.path(), &["orient", "--json"]);
    assert!(ov["project"].is_null() && ov["hint"].is_string());
}

// ----- J-8: errors-as-affordances name an exact fix command -----------------

#[test]
fn orient_rollup_affordances_name_fixes() {
    // No project: both the human affordance and the JSON hint name `odm new project`.
    let none = TempDir::new().unwrap();
    write_config(none.path());
    let human = run(none.path(), &["orient"]);
    assert!(
        human.out.contains("odm new project"),
        "human affordance names the fix:\n{}",
        human.out
    );
    let json = run_json(none.path(), &["orient", "--json"]);
    assert!(
        json["hint"].as_str().unwrap().contains("odm new project"),
        "json hint names the fix:\n{json:#}"
    );

    // Multiple projects: both name `odm use project`.
    let many = TempDir::new().unwrap();
    write_config(many.path());
    persist(
        many.path(),
        Document::new(fm('A', 1, NodeType::Project, "Alpha", Origin::Planned), "# a\n"),
    );
    persist(
        many.path(),
        Document::new(fm('B', 2, NodeType::Project, "Beta", Origin::Planned), "# b\n"),
    );
    let human = run(many.path(), &["orient"]);
    assert!(human.out.contains("odm use project"), "human prompt names the fix:\n{}", human.out);
    let json = run_json(many.path(), &["orient", "--json"]);
    assert!(json["hint"].as_str().unwrap().contains("odm use project"), "json hint names the fix");
}
