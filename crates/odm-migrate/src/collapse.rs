//! The project-vision-pair collapse (arc-migration-fidelity slice15 F-1/F-2,
//! ODD-0025 §2.3 reversal — operator decision 2026-08-01).
//!
//! [`crate::synthesis::apply_project_vision`] used to split the project into
//! **two** nodes: an editorial-merge synthesis (a curated `# Vision` body)
//! that `supersedes` a separately-minted 1:1 `project-plan` node. The
//! operator judged that split documentation over-engineering — it is also
//! the source of the reconcile gap and the fragile "already a synthesis"
//! idempotency (slice15's *why*). This module reverses it: the synthesis
//! node's body is **re-snapshotted from `project-plan.md`**, its
//! `source.synthesis` and `supersedes` edge are **dropped** — becoming the
//! one faithful 1:1 project node — and the node it superseded is **retired**
//! (supersede-don't-delete: a historical record, not deleted).
//!
//! **The synthesis node's `id`/`number` are never re-minted** — they are
//! re-cast in place, so every `part_of` child (every arc) keeps resolving to
//! the identical id without being touched at all. This is a capability, not
//! a live-store operation: firing it against `.worktrees/odm` is the
//! arc-close freeze's job (slice15 is fixture-only).

use std::path::{Path, PathBuf};

use odm_core::frontmatter::Document;
use odm_core::{Id, NodeType};
use odm_store::Store;

use crate::{MigrateError, Mode};

/// One project-vision pair collapsed (or, under `--dry-run`, that would be).
#[derive(Debug, Clone)]
pub struct Collapsed {
    /// The surviving project node's identity — unchanged: the synthesis
    /// node's own id, re-cast in place, so every `part_of` child (each of
    /// the 12 arcs) still resolves without being touched.
    pub project_id: Id,
    /// The surviving project node's `number` (unchanged).
    pub project_number: u32,
    /// The surviving project node's new name (the base's real project name,
    /// not the synthesis's `"Vision"` label) — for a caller that renders a
    /// preview under `--dry-run`, before anything is persisted to load back.
    pub project_name: String,
    /// The retired 1:1 base node's identity (unchanged by retirement).
    pub retired_id: Id,
    /// The retired 1:1 base node's `number` (unchanged).
    pub retired_number: u32,
    /// The retired 1:1 base node's name (unchanged by retirement).
    pub retired_name: String,
}

/// The outcome of a [`collapse_project_vision`] run.
#[derive(Debug, Clone)]
pub struct CollapseReport {
    /// What was collapsed — `None` if the store carries no project node with
    /// `source.synthesis` (already collapsed, or never split): a 0-change,
    /// idempotent no-op ("already a plain project node").
    pub collapsed: Option<Collapsed>,
    /// Whether this was a dry run (nothing written).
    pub dry_run: bool,
}

/// Why the collapse could not proceed — a reported shape mismatch, never a
/// silent skip: the store carries a project node with `source.synthesis`,
/// but not in the shape [`crate::synthesis::apply_project_vision`] is
/// documented to produce (exactly one forward `supersedes` edge, to another
/// project node).
#[derive(Debug, thiserror::Error)]
pub enum CollapseError {
    /// The synthesis-shaped project node supersedes zero, or more than one,
    /// node — `apply_project_vision` always attaches exactly one.
    #[error(
        "the synthesis-shaped project #{number} supersedes {count} node(s); the collapse \
         expects exactly one (the 1:1 project-plan base)"
    )]
    UnexpectedSupersedeShape {
        /// The synthesis node's `number`.
        number: u32,
        /// How many nodes it supersedes.
        count: usize,
    },
    /// The synthesis node's supersede target is not itself a project node.
    #[error("the synthesis-shaped project #{0}'s supersede target is not a project node")]
    TargetNotAProject(u32),
    /// The supersede target could not be found in the store.
    #[error("the synthesis-shaped project #{0}'s supersede target could not be loaded")]
    TargetMissing(u32),
    /// A fidelity/store failure.
    #[error(transparent)]
    Migrate(#[from] MigrateError),
}

/// Collapses the project-vision pair in `store` against the real
/// `project-plan.md` under `plan_root`, if one exists — idempotent: a store
/// that already carries a plain 1:1 project node (no `source.synthesis`) is
/// a 0-change no-op, so re-running after a successful collapse is always
/// safe.
///
/// Finds the **project** node carrying `source.synthesis` (there is at most
/// one — the model does not support two live project syntheses at once),
/// resolves its single `supersedes` target (the 1:1 base), then:
///
/// - re-snapshots the synthesis node's body from `project-plan.md` verbatim,
///   under the same hard body-hash gate every migrated node is verified
///   against (ODD-0025 §2.1), restores its name to the base's (the real
///   project name, not the synthesis's `"Vision"` label), and rebuilds its
///   `source` record with no `synthesis` key — `id`/`number`/`part_of` are
///   never touched, so this is the **same node**, just no longer a merge;
/// - retires the 1:1 base node (supersede-don't-delete) with a recorded
///   reason — its own `id`/`number`/body are otherwise untouched.
///
/// # Errors
///
/// [`CollapseError`] if the synthesis node's `supersedes` shape does not
/// match what [`crate::synthesis::apply_project_vision`] produces, or on a
/// store load/persist/fidelity failure.
pub fn collapse_project_vision(
    store: &Store,
    plan_root: &Path,
    mode: Mode,
) -> Result<CollapseReport, CollapseError> {
    let plan_root_buf = plan_root.canonicalize().unwrap_or_else(|_| plan_root.to_path_buf());
    let plan_root = plan_root_buf.as_path();
    let anchor = crate::fidelity::anchor_for(plan_root);

    let corpus = store.load_all().map_err(MigrateError::LoadCorpus)?;
    let Some(synthesis) = corpus.iter().find(|d| {
        d.frontmatter().node_type() == NodeType::Project
            && d.frontmatter().source().is_some_and(|s| s.synthesis.is_some())
    }) else {
        return Ok(CollapseReport { collapsed: None, dry_run: mode.is_dry_run() });
    };
    let synth_fm = synthesis.frontmatter();

    let supersedes = &synth_fm.edges().supersedes;
    if supersedes.len() != 1 {
        return Err(CollapseError::UnexpectedSupersedeShape {
            number: synth_fm.number(),
            count: supersedes.len(),
        });
    }
    let base_id = supersedes[0].node;
    let base = corpus
        .iter()
        .find(|d| d.frontmatter().id() == base_id)
        .ok_or(CollapseError::TargetMissing(synth_fm.number()))?;
    if base.frontmatter().node_type() != NodeType::Project {
        return Err(CollapseError::TargetNotAProject(synth_fm.number()));
    }

    let today = chrono::Utc::now().date_naive();
    let source_path = plan_root.join("project-plan.md");
    let body = std::fs::read_to_string(&source_path)
        .map_err(|source| MigrateError::SourceRead { path: source_path.clone(), source })?;
    let relative = crate::fidelity::relativize(&anchor, &source_path);
    let (_, updated) = crate::fidelity::git_derived_dates(&anchor, &source_path, today);

    let project_name = base.frontmatter().name().to_string();
    let mut new_fm = synth_fm.clone();
    new_fm.set_name(project_name.clone());
    new_fm.set_updated(updated);
    new_fm.stamp_schema();
    new_fm.edges_mut().supersedes.clear();
    let new_fm = new_fm.with_source(crate::fidelity::build_source(
        vec![PathBuf::from(&relative)],
        "project-plan",
        today,
    ));
    let new_document = Document::new(new_fm, body.clone());
    crate::fidelity::verify_body_hash(
        &body,
        new_document.body(),
        format!("#{} (project) collapse", synth_fm.number()),
    )?;

    let mut retired_fm = base.frontmatter().clone();
    retired_fm.retire(
        "collapsed into the faithful 1:1 project node (arc-migration-fidelity s15, \
         ODD-0025 §2.3 reversal — operator decision 2026-08-01)",
        today,
    );
    let retired_document = Document::new(retired_fm, base.body().to_string());

    let collapsed = Collapsed {
        project_id: synth_fm.id(),
        project_number: synth_fm.number(),
        project_name,
        retired_id: base.frontmatter().id(),
        retired_number: base.frontmatter().number(),
        retired_name: base.frontmatter().name().to_string(),
    };

    if !mode.is_dry_run() {
        store
            .persist(&new_document)
            .map_err(|source| MigrateError::Persist { number: collapsed.project_number, source })?;
        store
            .persist(&retired_document)
            .map_err(|source| MigrateError::Persist { number: collapsed.retired_number, source })?;
    }

    Ok(CollapseReport { collapsed: Some(collapsed), dry_run: mode.is_dry_run() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use odm_core::Origin;
    use odm_core::frontmatter::{Frontmatter, Source, SupersedeKind, Supersedes};
    use odm_store::Store;
    use tempfile::TempDir;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()
    }

    /// Seeds a synthesis-shaped project pair: `#1000` (editorial-merge
    /// synthesis, "Vision") supersedes `#1001` (the 1:1 `project-plan`
    /// clone). Returns `(synthesis_id, base_id)`.
    fn seed_pair(store: &Store, plan_body: &str) -> (Id, Id) {
        let base_id = Id::new();
        let mut base_fm = Frontmatter::new(
            base_id,
            1001,
            NodeType::Project,
            "odm",
            day(),
            day(),
            Origin::Planned,
        );
        base_fm.stamp_schema();
        let base_fm = base_fm.with_source(Source {
            paths: vec![PathBuf::from("project-plan.md")],
            class: "project-plan".to_string(),
            normalization: "trim+lf".to_string(),
            migrated_by: "odm-migrate/test".to_string(),
            migrated_on: day(),
            synthesis: None,
            attestation: None,
        });
        store.persist(&Document::new(base_fm, plan_body.to_string())).unwrap();

        let synth_id = Id::new();
        let mut synth_fm = Frontmatter::new(
            synth_id,
            1000,
            NodeType::Project,
            "Vision",
            day(),
            day(),
            Origin::Planned,
        );
        synth_fm.stamp_schema();
        synth_fm.edges_mut().supersedes =
            vec![Supersedes { node: base_id, kind: SupersedeKind::Updates }];
        let synth_fm = synth_fm.with_source(Source {
            paths: vec![PathBuf::from("project-plan.md")],
            class: "vision".to_string(),
            normalization: "trim+lf".to_string(),
            migrated_by: "odm-migrate/test".to_string(),
            migrated_on: day(),
            synthesis: Some("editorial-merge".to_string()),
            attestation: Some("operator: distills the source".to_string()),
        });
        store
            .persist(&Document::new(
                synth_fm,
                "# Vision\n\nSynthesized, not the source.\n".to_string(),
            ))
            .unwrap();

        (synth_id, base_id)
    }

    #[test]
    fn collapse_makes_the_synthesis_a_plain_1to1_node_and_retires_the_base() {
        let tmp = TempDir::new().unwrap();
        let plan_body = "# Test Project\n\nThe real plan-of-record content.\n";
        std::fs::write(tmp.path().join("project-plan.md"), plan_body).unwrap();

        let store_dir = TempDir::new().unwrap();
        let store = Store::open(store_dir.path());
        let (synth_id, base_id) = seed_pair(&store, plan_body);

        let report = collapse_project_vision(&store, tmp.path(), Mode::Commit).expect("collapse");
        let collapsed = report.collapsed.expect("a pair was collapsed");
        assert_eq!(collapsed.project_id, synth_id, "the synthesis id survives, re-cast in place");
        assert_eq!(collapsed.retired_id, base_id);
        assert_eq!(collapsed.project_name, "odm", "report carries the restored name for preview");
        assert_eq!(collapsed.retired_name, "odm", "report carries the retired node's own name");

        let nodes = store.load_all().unwrap();
        let project = nodes.iter().find(|d| d.frontmatter().id() == synth_id).unwrap();
        assert_eq!(project.body(), plan_body, "body re-snapshotted from project-plan.md");
        assert_eq!(project.frontmatter().number(), 1000, "number unchanged");
        assert_eq!(project.frontmatter().name(), "odm", "name restored from the base");
        assert!(
            project.frontmatter().source().unwrap().synthesis.is_none(),
            "source.synthesis dropped"
        );
        assert!(project.frontmatter().edges().supersedes.is_empty(), "supersedes edge dropped");

        let retired = nodes.iter().find(|d| d.frontmatter().id() == base_id).unwrap();
        assert!(retired.frontmatter().retired().is_some(), "the base is retired, not deleted");
        assert_eq!(retired.frontmatter().number(), 1001, "the retired node's number is untouched");
    }

    #[test]
    fn collapse_preserves_part_of_children() {
        let tmp = TempDir::new().unwrap();
        let plan_body = "# Test Project\n\nContent.\n";
        std::fs::write(tmp.path().join("project-plan.md"), plan_body).unwrap();

        let store_dir = TempDir::new().unwrap();
        let store = Store::open(store_dir.path());
        let (synth_id, _base_id) = seed_pair(&store, plan_body);

        // 12 arcs, each `part_of` the synthesis node — the exact shape the
        // real corpus carries (F-1: "#1000 must survive; it carries all 12
        // arcs").
        let mut arc_ids = Vec::new();
        for n in 0..12u32 {
            let arc_id = Id::new();
            let mut fm = Frontmatter::new(
                arc_id,
                1100 + n * 100,
                NodeType::Arc,
                format!("Arc {n}"),
                day(),
                day(),
                Origin::Planned,
            );
            fm.stamp_schema();
            fm.edges_mut().part_of = Some(synth_id);
            store.persist(&Document::new(fm, format!("# Arc {n}\n"))).unwrap();
            arc_ids.push(arc_id);
        }

        collapse_project_vision(&store, tmp.path(), Mode::Commit).expect("collapse");

        let nodes = store.load_all().unwrap();
        for arc_id in &arc_ids {
            let arc = nodes.iter().find(|d| d.frontmatter().id() == *arc_id).unwrap();
            assert_eq!(
                arc.frontmatter().edges().part_of,
                Some(synth_id),
                "every arc still resolves to the (untouched) surviving project id"
            );
        }
    }

    #[test]
    fn collapse_leaves_a_genuine_non_project_synthesis_untouched() {
        // F-7: "genuine synthesis nodes untouched" — a design/research
        // synthesis (not the project pair) must never be touched by the
        // collapse, which only ever targets a `NodeType::Project` carrying
        // `source.synthesis`.
        let tmp = TempDir::new().unwrap();
        let plan_body = "# Test Project\n\nContent.\n";
        std::fs::write(tmp.path().join("project-plan.md"), plan_body).unwrap();

        let store_dir = TempDir::new().unwrap();
        let store = Store::open(store_dir.path());
        seed_pair(&store, plan_body);

        let other_synth_id = Id::new();
        let mut other_fm = Frontmatter::new(
            other_synth_id,
            42,
            NodeType::Design,
            "Some other synthesis",
            day(),
            day(),
            Origin::Planned,
        );
        other_fm.stamp_schema();
        let other_fm = other_fm.with_source(Source {
            paths: vec![PathBuf::from("a.md"), PathBuf::from("b.md")],
            class: "synthesis".to_string(),
            normalization: "trim+lf".to_string(),
            migrated_by: "odm-migrate/test".to_string(),
            migrated_on: day(),
            synthesis: Some("concatenation".to_string()),
            attestation: None,
        });
        let other_body = "merged body\n";
        store.persist(&Document::new(other_fm, other_body.to_string())).unwrap();

        collapse_project_vision(&store, tmp.path(), Mode::Commit).expect("collapse");

        let nodes = store.load_all().unwrap();
        let other = nodes.iter().find(|d| d.frontmatter().id() == other_synth_id).unwrap();
        assert_eq!(other.body(), other_body, "a non-project synthesis body is untouched");
        assert_eq!(
            other.frontmatter().source().unwrap().synthesis.as_deref(),
            Some("concatenation"),
            "a non-project synthesis keeps its synthesis key"
        );
    }

    #[test]
    fn collapse_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let plan_body = "# Test Project\n\nContent.\n";
        std::fs::write(tmp.path().join("project-plan.md"), plan_body).unwrap();

        let store_dir = TempDir::new().unwrap();
        let store = Store::open(store_dir.path());
        seed_pair(&store, plan_body);

        let first = collapse_project_vision(&store, tmp.path(), Mode::Commit).expect("first");
        assert!(first.collapsed.is_some());

        let second = collapse_project_vision(&store, tmp.path(), Mode::Commit).expect("second");
        assert!(second.collapsed.is_none(), "already a plain project node — 0-change no-op");
    }

    #[test]
    fn collapse_dry_run_writes_nothing() {
        let tmp = TempDir::new().unwrap();
        let plan_body = "# Test Project\n\nContent.\n";
        std::fs::write(tmp.path().join("project-plan.md"), plan_body).unwrap();

        let store_dir = TempDir::new().unwrap();
        let store = Store::open(store_dir.path());
        seed_pair(&store, plan_body);

        let before: Vec<_> = store.load_all().unwrap();
        let report = collapse_project_vision(&store, tmp.path(), Mode::DryRun).expect("dry-run");
        assert!(report.dry_run);
        assert!(report.collapsed.is_some(), "the plan still names what would be collapsed");

        let after: Vec<_> = store.load_all().unwrap();
        assert_eq!(before.len(), after.len());
        for (b, a) in before.iter().zip(after.iter()) {
            assert_eq!(b.body(), a.body(), "dry-run wrote nothing");
            assert_eq!(b.frontmatter().retired().is_some(), a.frontmatter().retired().is_some());
        }
    }

    #[test]
    fn collapse_is_a_no_op_when_no_synthesis_project_exists() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("project-plan.md"), "# Test Project\n\nContent.\n").unwrap();

        let store_dir = TempDir::new().unwrap();
        let store = Store::open(store_dir.path());
        // A plain, already-1:1 project — never split.
        let mut fm = Frontmatter::new(
            Id::new(),
            1000,
            NodeType::Project,
            "odm",
            day(),
            day(),
            Origin::Planned,
        );
        fm.stamp_schema();
        store.persist(&Document::new(fm, "# Test Project\n\nContent.\n".to_string())).unwrap();

        let report = collapse_project_vision(&store, tmp.path(), Mode::Commit).expect("no-op");
        assert!(report.collapsed.is_none());
    }

    #[test]
    fn collapse_rejects_a_synthesis_with_zero_supersede_targets() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("project-plan.md"), "# Test Project\n\nContent.\n").unwrap();

        let store_dir = TempDir::new().unwrap();
        let store = Store::open(store_dir.path());
        let mut synth_fm = Frontmatter::new(
            Id::new(),
            1000,
            NodeType::Project,
            "Vision",
            day(),
            day(),
            Origin::Planned,
        );
        synth_fm.stamp_schema();
        let synth_fm = synth_fm.with_source(Source {
            paths: vec![PathBuf::from("project-plan.md")],
            class: "vision".to_string(),
            normalization: "trim+lf".to_string(),
            migrated_by: "odm-migrate/test".to_string(),
            migrated_on: day(),
            synthesis: Some("editorial-merge".to_string()),
            attestation: Some("operator: distills the source".to_string()),
        });
        store.persist(&Document::new(synth_fm, "# Vision\n\nText.\n".to_string())).unwrap();

        let err = collapse_project_vision(&store, tmp.path(), Mode::Commit).unwrap_err();
        assert!(
            matches!(err, CollapseError::UnexpectedSupersedeShape { number: 1000, count: 0 }),
            "{err:?}"
        );
    }

    #[test]
    fn collapse_rejects_a_supersede_target_that_is_not_a_project() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("project-plan.md"), "# Test Project\n\nContent.\n").unwrap();

        let store_dir = TempDir::new().unwrap();
        let store = Store::open(store_dir.path());

        let arc_id = Id::new();
        let mut arc_fm =
            Frontmatter::new(arc_id, 1100, NodeType::Arc, "Arc 01", day(), day(), Origin::Planned);
        arc_fm.stamp_schema();
        store.persist(&Document::new(arc_fm, "# Arc 01\n".to_string())).unwrap();

        let mut synth_fm = Frontmatter::new(
            Id::new(),
            1000,
            NodeType::Project,
            "Vision",
            day(),
            day(),
            Origin::Planned,
        );
        synth_fm.stamp_schema();
        synth_fm.edges_mut().supersedes =
            vec![Supersedes { node: arc_id, kind: SupersedeKind::Updates }];
        let synth_fm = synth_fm.with_source(Source {
            paths: vec![PathBuf::from("project-plan.md")],
            class: "vision".to_string(),
            normalization: "trim+lf".to_string(),
            migrated_by: "odm-migrate/test".to_string(),
            migrated_on: day(),
            synthesis: Some("editorial-merge".to_string()),
            attestation: Some("operator: distills the source".to_string()),
        });
        store.persist(&Document::new(synth_fm, "# Vision\n\nText.\n".to_string())).unwrap();

        let err = collapse_project_vision(&store, tmp.path(), Mode::Commit).unwrap_err();
        assert!(matches!(err, CollapseError::TargetNotAProject(1000)), "{err:?}");
    }

    #[test]
    fn collapse_error_messages_name_the_synthesis_number() {
        let shape = CollapseError::UnexpectedSupersedeShape { number: 7, count: 2 };
        assert!(shape.to_string().contains('7'));
        assert!(CollapseError::TargetNotAProject(9).to_string().contains('9'));
        assert!(CollapseError::TargetMissing(11).to_string().contains("11"));
    }
}
