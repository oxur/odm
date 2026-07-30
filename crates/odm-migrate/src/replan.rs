//! Re-deriving an already-imported plan corpus in place (RH C-5).
//!
//! The plan nodes were minted once, by `self-host`, and three things about that
//! derivation were wrong or missing:
//!
//! - **names** carried the plan document's coordinates and role — `"Slice 01
//!   (Arc 02) — … (plan-of-record)"` (F-18, ODD-0013 §2.1 v2.3);
//! - **dates** were the cutover date, identical for 45 of 46 nodes, so `created`
//!   said when the *importer ran* rather than when the work began (F-20);
//! - the project node had **no vision body**, so `odm orient` printed "no vision
//!   text yet" on odm's own repo (L-3a).
//!
//! ## Why a re-stamp and not a re-derivation
//!
//! Re-running the derivation cannot fix any of it: import is idempotent on
//! `(type, number)`, so existing nodes are skipped. Deriving into an empty store
//! *would* apply the new logic — and would **mint new ULIDs**, breaking every
//! edge that references the old ones, and tripping the G-1 freeze on new ids.
//!
//! So this pass rewrites the existing files **in place, id-preserving**. What it
//! touches: `name`, `created`, `updated`, and the project's body. What it must
//! never touch: `id`, `number`, `type`, `schema`, gates, edges, and any other
//! body. A diff showing anything else is a defect.
//!
//! ## Why nothing relocates
//!
//! The chunk's brief expected a corrected `created` to move each file into its
//! real `nodes/YYYY/MM/` shard. It does not, and should not: `layout` derives
//! the path from the **ULID's** timestamp, not from the frontmatter date —
//! deliberately, so that a node file never moves on retitle, reparent or gate
//! change, and locate-by-id stays O(1) with no lookup index.
//!
//! Since ids are preserved (G-1), the shard is preserved with them. Forcing a
//! move would in fact *break* `Store::load`, which recomputes the path from the
//! id. So the two requirements are in tension only apparently: id-preservation
//! wins, and there is nothing to relocate. The `moved` flag stays as a guard —
//! it should always report zero, and a non-zero would mean the layout rule had
//! changed underneath this pass.
//!
//! ## Retired nodes are left alone
//!
//! A retired node is a **record**, not live work. Node #1605 exists precisely
//! to document a mistake — a slice minted against a stale directory listing —
//! and its source directory is itself a tombstone, so re-deriving its name and
//! date from that directory would quietly rewrite the record of the error into
//! something else. Tombstones are skipped.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::NaiveDate;
use odm_core::NodeType;
use odm_store::{Store, worktree};

use crate::mapping::normalize_name;
use crate::{MigrateError, Mode};

/// The heading the project body carries its vision under — what `orient` looks
/// for (L-3a), and what `check`'s `no-vision` warning requires
/// ([`crate::synthesis::apply_project_vision`] also stamps it, arc-migration-
/// fidelity s13 iteration).
pub(crate) const VISION_HEADING: &str = "# Vision";

/// What the plan says a node should look like, for one `(type, number)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derived {
    /// The normalized name.
    pub name: String,
    /// The real creation date, when git knows it.
    pub created: Option<NaiveDate>,
    /// The real last-touched date, when git knows it.
    pub updated: Option<NaiveDate>,
}

/// What a re-stamp changed on one node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Restamped {
    /// The node's number.
    pub number: u32,
    /// Its name before and after, when it changed.
    pub name: Option<(String, String)>,
    /// Its `created` before and after, when it changed.
    pub created: Option<(NaiveDate, NaiveDate)>,
    /// Whether a vision body was written (the project node only).
    pub vision: bool,
    /// Whether correcting `created` moved the file to another month shard.
    pub moved: bool,
}

impl Restamped {
    /// Whether this node changed at all.
    #[must_use]
    pub fn is_change(&self) -> bool {
        self.name.is_some() || self.created.is_some() || self.vision
    }
}

/// The derived facts for every plan node under `plan_root`, keyed by
/// `(type, number)` — the join key the corpus was minted with.
///
/// Dates come from `repo_root`'s git history for the node's own plan directory,
/// so a slice dated from its own directory gets its own date rather than its
/// arc's. A path git does not track yields `None`, which the caller reports
/// rather than papering over.
///
/// # Errors
///
/// [`MigrateError`] if git cannot be run.
pub fn derive_plan(
    repo_root: &Path,
    plan_root: &Path,
) -> Result<BTreeMap<(NodeType, u32), Derived>, MigrateError> {
    let mut out = BTreeMap::new();
    for node in crate::selfhost::plan_nodes(plan_root) {
        let created = git_date(repo_root, &node.source, true)?;
        let updated = git_date(repo_root, &node.source, false)?;
        out.insert(
            (node.node_type, node.number),
            Derived { name: normalize_name(&node.name), created, updated },
        );
    }
    Ok(out)
}

/// A git-derived date for `path`, as a `NaiveDate`.
fn git_date(repo_root: &Path, path: &Path, first: bool) -> Result<Option<NaiveDate>, MigrateError> {
    // Paths are given to git relative to the repository root, so a store living
    // in a worktree still resolves them against the code branch's history.
    let relative = path.strip_prefix(repo_root).unwrap_or(path);
    let text = if first {
        worktree::first_commit_date(repo_root, relative)
    } else {
        worktree::last_commit_date(repo_root, relative)
    }
    .map_err(MigrateError::LoadCorpus)?;
    Ok(text.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()))
}

/// The vision text for the project body, read from the plan's §1.
///
/// `orient` looks for a `# Vision` section and, finding none, tells the reader
/// to add one — which is exactly what it did on odm's own repo, because
/// `self-host` carried the plan's *structure* but not its *substance* (L-3a).
/// The definition-of-done paragraph is that substance.
#[must_use]
pub fn vision_from_plan(plan_root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(plan_root.join("project-plan.md")).ok()?;
    let mut body = String::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("## ") {
            // §1 is the definition of done; the next `##` ends it.
            if inside {
                break;
            }
            inside = line.contains("Definition of done");
            continue;
        }
        if inside {
            body.push_str(line);
            body.push('\n');
        }
    }
    let body = body.trim();
    (!body.is_empty()).then(|| body.to_string())
}

/// Rewrites every plan node in `store` to match `derived`, in place.
///
/// # Errors
///
/// [`MigrateError`] on a store read/write failure.
pub fn restamp(
    store: &Store,
    derived: &BTreeMap<(NodeType, u32), Derived>,
    vision: Option<&str>,
    mode: Mode,
) -> Result<Vec<Restamped>, MigrateError> {
    let documents = store.load_all().map_err(MigrateError::LoadCorpus)?;
    let mut changes = Vec::new();

    for mut document in documents {
        let (node_type, number) = {
            let fm = document.frontmatter();
            (fm.node_type(), fm.number())
        };
        // Document nodes are `migrate`'s business and keep their legacy dates.
        if !node_type.is_work() {
            continue;
        }
        // A retired node is a historical record; re-deriving it would rewrite
        // the record (see the module header).
        if document.frontmatter().retired().is_some() {
            continue;
        }
        let Some(want) = derived.get(&(node_type, number)) else {
            continue;
        };

        let old_path = store.path_of(document.frontmatter().id());
        let mut change =
            Restamped { number, name: None, created: None, vision: false, moved: false };

        if document.frontmatter().name() != want.name {
            change.name = Some((document.frontmatter().name().to_string(), want.name.clone()));
            document.frontmatter_mut().set_name(want.name.clone());
        }
        if let Some(created) = want.created
            && document.frontmatter().created() != created
        {
            change.created = Some((document.frontmatter().created(), created));
            document.frontmatter_mut().set_created(created);
        }
        if let Some(updated) = want.updated {
            document.frontmatter_mut().set_updated(updated);
        }
        // The project carries the vision (L-3a).
        if node_type == NodeType::Project
            && let Some(vision) = vision
            && !document.body().contains(VISION_HEADING)
        {
            let body = with_vision(document.body(), vision);
            document.set_body(body);
            change.vision = true;
        }

        if !change.is_change() {
            continue;
        }
        // `created` decides the month shard, so a corrected date relocates the
        // file. The id — and therefore the node's identity — is untouched.
        let new_path = store.path_of(document.frontmatter().id());
        change.moved = new_path != old_path;

        if !mode.is_dry_run() {
            store.persist(&document).map_err(|source| MigrateError::Persist { number, source })?;
            // The node now lives in a different month shard; drop the file it
            // vacated, or the corpus would carry the node twice.
            if change.moved && old_path.exists() {
                std::fs::remove_file(&old_path).map_err(|source| MigrateError::Persist {
                    number,
                    source: odm_store::StoreError::Io { path: old_path.clone(), source },
                })?;
            }
        }
        changes.push(change);
    }
    changes.sort_by_key(|c| c.number);
    Ok(changes)
}

/// The body with a `# Vision` section inserted after the title line.
fn with_vision(body: &str, vision: &str) -> String {
    let mut lines = body.lines();
    let title = lines.next().unwrap_or_default();
    let rest: Vec<&str> = lines.collect();
    let mut out = String::new();
    out.push_str(title);
    out.push_str("\n\n");
    out.push_str(VISION_HEADING);
    out.push_str("\n\n");
    out.push_str(vision.trim());
    out.push('\n');
    let rest = rest.join("\n");
    if !rest.trim().is_empty() {
        out.push('\n');
        out.push_str(rest.trim_start_matches('\n'));
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// The re-stamp report as a summary line.
#[must_use]
pub fn summarize(changes: &[Restamped]) -> String {
    let names = changes.iter().filter(|c| c.name.is_some()).count();
    let dates = changes.iter().filter(|c| c.created.is_some()).count();
    let moved = changes.iter().filter(|c| c.moved).count();
    let vision = changes.iter().filter(|c| c.vision).count();
    format!("{names} name(s), {dates} date(s), {moved} relocated, {vision} vision")
}

/// The paths a re-stamp would leave behind, for a caller that needs to clean up.
#[must_use]
pub fn is_empty(changes: &[Restamped]) -> bool {
    changes.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_is_inserted_after_the_title() {
        let out = with_vision("# odm\n\nSome existing prose.\n", "The vision text.");
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "# odm", "the title stays first");
        assert_eq!(lines[2], VISION_HEADING, "the vision heading follows it");
        assert!(out.contains("The vision text."));
        assert!(out.contains("Some existing prose."), "existing body survives");
    }

    #[test]
    fn test_vision_insertion_handles_a_title_only_body() {
        let out = with_vision("# odm\n", "V.");
        assert!(out.starts_with("# odm\n"));
        assert!(out.contains("# Vision"));
        assert!(out.trim_end().ends_with("V."));
    }

    #[test]
    fn test_restamped_reports_whether_anything_changed() {
        let none = Restamped { number: 1, name: None, created: None, vision: false, moved: false };
        assert!(!none.is_change());
        let some = Restamped {
            number: 1,
            name: Some(("a".into(), "b".into())),
            created: None,
            vision: false,
            moved: false,
        };
        assert!(some.is_change());
    }

    #[test]
    fn test_summary_counts_each_kind_of_change() {
        let changes = vec![
            Restamped {
                number: 1,
                name: Some(("a".into(), "b".into())),
                created: None,
                vision: true,
                moved: false,
            },
            Restamped {
                number: 2,
                name: None,
                created: Some((
                    NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
                    NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
                )),
                vision: false,
                moved: true,
            },
        ];
        let s = summarize(&changes);
        assert!(s.contains("1 name(s)"), "{s}");
        assert!(s.contains("1 date(s)"), "{s}");
        assert!(s.contains("1 relocated"), "{s}");
        assert!(s.contains("1 vision"), "{s}");
    }
}
