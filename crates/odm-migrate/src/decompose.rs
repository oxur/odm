//! Deterministic decomposition auto-recompose after a `migrate` run
//! (arc-store-as-source s05).
//!
//! `migrate` (mint/reconcile) can churn a parent-capable node's children.
//! When it can *prove* the resulting work-child set is identical to what the
//! parent already affirmed via `decomposed: complete` — every "removed"
//! affirmed child maps, through an id remap the run itself performed this
//! run, to an "added" current child (same logical node, new id) — this
//! re-affirms automatically, closing the `init → mutate → manual
//! re-decompose` chore. When it cannot prove that (a genuine membership
//! change, or a removal/addition `id_remap` does not explain), the
//! affirmation is left untouched: `check`'s [`odm_core::recompose::Issue::DecompositionDrift`]
//! is the correct, honest signal, and this module must never silence it —
//! the seam (F-5) is deliberate. `decomposed: complete` is a human
//! completeness judgment; only the mechanical child-set half auto-heals.

use std::collections::{BTreeSet, HashMap};

use chrono::NaiveDate;
use odm_core::frontmatter::Frontmatter;
use odm_core::recompose::{Recomposition, decomposition_children};
use odm_core::{Id, NodeType};
use odm_store::Store;

use crate::{MigrateError, Mode};

/// What happened to one affirmed parent during an auto-recompose pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Every removed affirmed child mapped, via `id_remap`, to a distinct
    /// added child — the affirmation was rewritten with the new child set
    /// (or, under [`Mode::DryRun`], would be).
    ReAffirmed {
        /// The old ids removed from the affirmation.
        removed: Vec<Id>,
        /// The new ids added in their place.
        added: Vec<Id>,
    },
    /// A membership change `id_remap` cannot explain (a genuinely new or
    /// removed child) — left untouched. `check` reports `DecompositionDrift`.
    LeftAsDrift {
        /// Children present now but not in the affirmed set.
        added: Vec<Id>,
        /// Children in the affirmed set but absent now.
        removed: Vec<Id>,
    },
}

/// One affirmed parent's auto-recompose result, named for reporting.
#[derive(Debug, Clone)]
pub struct Recomposed {
    /// The parent's identity.
    pub id: Id,
    /// The parent's human number.
    pub number: u32,
    /// The parent's name.
    pub name: String,
    /// What happened.
    pub outcome: Outcome,
}

/// The outcome of an [`auto_recompose`] run.
#[derive(Debug, Clone)]
pub struct RecomposeReport {
    /// Every affirmed parent whose current work-children no longer match its
    /// affirmation — re-affirmed or left as drift. A parent that is already
    /// up to date, or has no affirmed decomposition at all, is not listed:
    /// there is nothing to report.
    pub changed: Vec<Recomposed>,
    /// Whether this was a dry run (nothing written).
    pub dry_run: bool,
}

impl RecomposeReport {
    /// How many parents were auto-re-affirmed.
    #[must_use]
    pub fn reaffirmed_count(&self) -> usize {
        self.changed.iter().filter(|r| matches!(r.outcome, Outcome::ReAffirmed { .. })).count()
    }

    /// How many parents were left as drift (a human `node decomposed` is
    /// still needed).
    #[must_use]
    pub fn left_as_drift_count(&self) -> usize {
        self.changed.iter().filter(|r| matches!(r.outcome, Outcome::LeftAsDrift { .. })).count()
    }
}

/// Recomputes every affirmed parent-capable node's work-children against
/// `id_remap` (the old→new id correlations `migrate` performed this run —
/// empty if the run performed none), re-affirming a provably-identical churn
/// and leaving every genuine membership change as drift for a human
/// `odm node decomposed`.
///
/// Deterministic and idempotent: run twice with nothing changed in between,
/// the second run reports nothing (every parent is already up to date).
///
/// # Errors
///
/// [`MigrateError::LoadCorpus`] if the corpus cannot be loaded;
/// [`MigrateError::Persist`] if a re-affirmed node cannot be written.
pub fn auto_recompose(
    store: &Store,
    id_remap: &HashMap<Id, Id>,
    today: NaiveDate,
    mode: Mode,
) -> Result<RecomposeReport, MigrateError> {
    let docs = store.load_all().map_err(MigrateError::LoadCorpus)?;
    let all: Vec<Frontmatter> = docs.iter().map(|d| d.frontmatter().clone()).collect();
    let recomp = Recomposition::build(&all);
    let types: HashMap<Id, NodeType> = all.iter().map(|f| (f.id(), f.node_type())).collect();

    let mut changed = Vec::new();
    for fm in &all {
        let Some(decomp) = fm.decomposed() else { continue };
        let affirmed: BTreeSet<Id> = decomp.children.iter().copied().collect();
        let current: BTreeSet<Id> =
            decomposition_children(&recomp, &types, fm.id()).into_iter().collect();

        if affirmed == current {
            continue; // up to date: nothing to report
        }

        let removed: Vec<Id> = affirmed.difference(&current).copied().collect();
        let added: Vec<Id> = current.difference(&affirmed).copied().collect();

        if let Some(outcome) = provably_same_set(&removed, &added, id_remap) {
            if mode.is_dry_run() {
                changed.push(Recomposed {
                    id: fm.id(),
                    number: fm.number(),
                    name: fm.name().to_string(),
                    outcome,
                });
                continue;
            }
            let mut doc = store.load(fm.id()).map_err(MigrateError::LoadCorpus)?;
            doc.frontmatter_mut().affirm_decomposed(current.into_iter().collect(), today);
            doc.frontmatter_mut().set_updated(today);
            store
                .persist(&doc)
                .map_err(|source| MigrateError::Persist { number: fm.number(), source })?;
            changed.push(Recomposed {
                id: fm.id(),
                number: fm.number(),
                name: fm.name().to_string(),
                outcome,
            });
        } else {
            changed.push(Recomposed {
                id: fm.id(),
                number: fm.number(),
                name: fm.name().to_string(),
                outcome: Outcome::LeftAsDrift { added, removed },
            });
        }
    }
    changed.sort_by_key(|r| r.id);
    Ok(RecomposeReport { changed, dry_run: mode.is_dry_run() })
}

/// Whether `removed`/`added` describe **only** an identity re-mint: every
/// removed id maps, via `id_remap`, to a distinct added id, and every added
/// id is accounted for that way. `None` on any uncertainty — a removal with
/// no mapping, an addition no removal maps to, or two removals colliding on
/// the same mapped target (a genuine merge, not a re-mint) — so the caller's
/// safe default is always "leave it as drift".
fn provably_same_set(removed: &[Id], added: &[Id], id_remap: &HashMap<Id, Id>) -> Option<Outcome> {
    let mapped: BTreeSet<Id> =
        removed.iter().map(|old| id_remap.get(old).copied()).collect::<Option<_>>()?;
    let added_set: BTreeSet<Id> = added.iter().copied().collect();
    if mapped.len() == removed.len() && mapped == added_set {
        Some(Outcome::ReAffirmed { removed: removed.to_vec(), added: added.to_vec() })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odm_core::Origin;
    use odm_core::frontmatter::Document;
    use odm_store::Store;
    use tempfile::TempDir;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 2).unwrap()
    }

    fn node(id: Id, number: u32, ty: NodeType, name: &str, parent: Option<Id>) -> Frontmatter {
        let mut fm = Frontmatter::new(id, number, ty, name, day(), day(), Origin::Planned);
        fm.edges_mut().part_of = parent;
        fm
    }

    // ----- F-4: identity re-mint auto-heals -------------------------------

    #[test]
    fn identity_remint_auto_reaffirms_with_the_new_ids() {
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path());

        let parent_id = Id::new();
        let x_old = Id::new();
        let x_new = Id::new();
        let y = Id::new();

        let mut parent = node(parent_id, 2, NodeType::Arc, "Q", None);
        parent.affirm_decomposed(vec![x_old, y], day());
        store.persist(&Document::new(parent, "# Q\n")).unwrap();
        // X was re-minted under a new id this run (a synthetic id_remap
        // stands in for whatever migrate produced); Y is unchanged.
        store
            .persist(&Document::new(node(x_new, 3, NodeType::Slice, "X", Some(parent_id)), "# X\n"))
            .unwrap();
        store
            .persist(&Document::new(node(y, 4, NodeType::Slice, "Y", Some(parent_id)), "# Y\n"))
            .unwrap();

        let id_remap = HashMap::from([(x_old, x_new)]);
        let report = auto_recompose(&store, &id_remap, day(), Mode::Commit).unwrap();

        assert_eq!(report.reaffirmed_count(), 1, "{:?}", report.changed);
        assert_eq!(report.left_as_drift_count(), 0, "{:?}", report.changed);
        assert!(!report.dry_run);

        let reloaded = store.load(parent_id).unwrap();
        let affirmed: BTreeSet<Id> =
            reloaded.frontmatter().decomposed().unwrap().children.iter().copied().collect();
        assert_eq!(
            affirmed,
            BTreeSet::from([x_new, y]),
            "re-affirmed with the new id, no manual step"
        );
    }

    #[test]
    fn dry_run_reports_the_reaffirmation_but_writes_nothing() {
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path());

        let parent_id = Id::new();
        let x_old = Id::new();
        let x_new = Id::new();

        let mut parent = node(parent_id, 2, NodeType::Arc, "Q", None);
        parent.affirm_decomposed(vec![x_old], day());
        store.persist(&Document::new(parent, "# Q\n")).unwrap();
        store
            .persist(&Document::new(node(x_new, 3, NodeType::Slice, "X", Some(parent_id)), "# X\n"))
            .unwrap();

        let id_remap = HashMap::from([(x_old, x_new)]);
        let report = auto_recompose(&store, &id_remap, day(), Mode::DryRun).unwrap();
        assert_eq!(report.reaffirmed_count(), 1);
        assert!(report.dry_run);

        // Nothing written: the affirmation still names the old id.
        let reloaded = store.load(parent_id).unwrap();
        let affirmed = &reloaded.frontmatter().decomposed().unwrap().children;
        assert_eq!(affirmed, &vec![x_old], "dry-run wrote nothing");
    }

    // ----- F-5: the seam — a genuine new child is never auto-blessed -------

    #[test]
    fn genuinely_new_child_is_left_as_drift_not_auto_affirmed() {
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path());

        let parent_id = Id::new();
        let x = Id::new();
        let z = Id::new();

        let mut parent = node(parent_id, 2, NodeType::Arc, "Q", None);
        parent.affirm_decomposed(vec![x], day());
        store.persist(&Document::new(parent, "# Q\n")).unwrap();
        store
            .persist(&Document::new(node(x, 3, NodeType::Slice, "X", Some(parent_id)), "# X\n"))
            .unwrap();
        // A genuinely new slice — no id_remap entry maps anything to it.
        store
            .persist(&Document::new(node(z, 4, NodeType::Slice, "Z", Some(parent_id)), "# Z\n"))
            .unwrap();

        let report = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert_eq!(report.reaffirmed_count(), 0, "{:?}", report.changed);
        assert_eq!(report.left_as_drift_count(), 1, "{:?}", report.changed);
        assert!(
            matches!(&report.changed[0].outcome, Outcome::LeftAsDrift { added, removed } if added == &[z] && removed.is_empty())
        );

        // The affirmation was never touched — `check` still sees the drift.
        let reloaded = store.load(parent_id).unwrap();
        assert_eq!(reloaded.frontmatter().decomposed().unwrap().children, vec![x]);
    }

    #[test]
    fn a_removal_with_no_remap_entry_is_left_as_drift() {
        // X disappears (no re-mint mapping for it) and nothing replaces it —
        // must not be silently dropped from the affirmation.
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path());

        let parent_id = Id::new();
        let x = Id::new();
        let mut parent = node(parent_id, 2, NodeType::Arc, "Q", None);
        parent.affirm_decomposed(vec![x], day());
        store.persist(&Document::new(parent, "# Q\n")).unwrap();
        // X's file is never persisted — it is simply gone from the corpus.

        let report = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert_eq!(report.left_as_drift_count(), 1, "{:?}", report.changed);
        let reloaded = store.load(parent_id).unwrap();
        assert_eq!(reloaded.frontmatter().decomposed().unwrap().children, vec![x], "untouched");
    }

    // ----- up to date / non-work children ----------------------------------

    #[test]
    fn an_up_to_date_parent_is_not_reported() {
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path());

        let parent_id = Id::new();
        let x = Id::new();
        let mut parent = node(parent_id, 2, NodeType::Arc, "Q", None);
        parent.affirm_decomposed(vec![x], day());
        store.persist(&Document::new(parent, "# Q\n")).unwrap();
        store
            .persist(&Document::new(node(x, 3, NodeType::Slice, "X", Some(parent_id)), "# X\n"))
            .unwrap();

        let report = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert!(report.changed.is_empty(), "{:?}", report.changed);
    }

    #[test]
    fn a_non_work_child_added_alongside_never_triggers_a_report() {
        // An artifact attached to an already-affirmed arc (ODD-0025 §2.5
        // containment) is not decomposition scope at all — F-3's guarantee,
        // reproduced through the auto-recompose path.
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path());

        let parent_id = Id::new();
        let x = Id::new();
        let artifact = Id::new();
        let mut parent = node(parent_id, 2, NodeType::Arc, "Q", None);
        parent.affirm_decomposed(vec![x], day());
        store.persist(&Document::new(parent, "# Q\n")).unwrap();
        store
            .persist(&Document::new(node(x, 3, NodeType::Slice, "X", Some(parent_id)), "# X\n"))
            .unwrap();
        store
            .persist(&Document::new(
                node(artifact, 4, NodeType::Artifact, "A", Some(parent_id)),
                "# A\n",
            ))
            .unwrap();

        let report = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert!(report.changed.is_empty(), "{:?}", report.changed);
    }
}
