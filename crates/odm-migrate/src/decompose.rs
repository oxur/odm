//! Deterministic decomposition auto-recompose after a `migrate` run
//! (arc-store-as-source s05, extended by s06).
//!
//! `migrate` (mint/reconcile) can churn a parent-capable node's children.
//! Two mechanical cases auto-heal an **already-affirmed** parent without a
//! human `node decomposed`:
//!
//! - **Identity re-mint** (s05): every "removed" affirmed child maps, through
//!   an id remap the run itself performed this run, to an "added" current
//!   child (same logical node, new id) → [`Outcome::ReAffirmed`].
//! - **Authored addition** (s06): the current work-children are a superset of
//!   the affirmed work-children — no affirmed work-child is missing — because
//!   in odm's model the plan tree declares scope, and this pass runs *inside*
//!   `migrate --all`, after the store has just been reconciled to the plan
//!   tree. This also drops any stale non-work ids a pre-s05 affirmation still
//!   carries → [`Outcome::AutoExtended`].
//!
//! When neither explains the churn — a genuine work-child **removal** the
//! addition-only and re-mint cases don't cover — the affirmation is left
//! untouched: `check`'s
//! [`odm_core::recompose::Issue::DecompositionDrift`] is the correct, honest
//! signal, and this module must never silence it — the seam (F-5) is
//! deliberate. A **never-affirmed** parent is never auto-affirmed either: the
//! *first* affirmation is a human act; this pass only maintains an existing
//! one. `decomposed: complete` is a human completeness judgment; only the
//! mechanical child-set half auto-heals.

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
    /// The current work-children are a superset of the affirmed work-children
    /// (additions only, no affirmed work-child missing) — the affirmation was
    /// rewritten to the current work-child set (or, under [`Mode::DryRun`],
    /// would be).
    AutoExtended {
        /// The new work-children the affirmation now includes.
        added: Vec<Id>,
        /// Stale non-work ids (e.g. artifacts) a pre-s05 affirmation carried,
        /// dropped by the rewrite.
        dropped_stale: Vec<Id>,
    },
    /// A membership change neither the addition-only nor the identity-remint
    /// case explains (a work-child was genuinely removed) — left untouched.
    /// `check` reports `DecompositionDrift`.
    LeftAsDrift {
        /// Children present now but not in the affirmed work-child set.
        added: Vec<Id>,
        /// Affirmed work-children absent now.
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

    /// How many parents were auto-extended (an authored addition folded into
    /// an already-affirmed decomposition, no manual `node decomposed`).
    #[must_use]
    pub fn auto_extended_count(&self) -> usize {
        self.changed.iter().filter(|r| matches!(r.outcome, Outcome::AutoExtended { .. })).count()
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
        let affirmed_raw: BTreeSet<Id> = decomp.children.iter().copied().collect();
        let current: BTreeSet<Id> =
            decomposition_children(&recomp, &types, fm.id()).into_iter().collect();

        if affirmed_raw == current {
            continue; // up to date: nothing to report
        }

        // s06: filter the affirmation down to work-children before comparing.
        // An affirmed id still present in the corpus but not work-typed (an
        // artifact/note a pre-s05 affirmation carried) is stale and safe to
        // drop. An affirmed id **absent from the corpus entirely** is kept —
        // we cannot know it was non-work, and treating "vanished" as "was
        // never work" would silently swallow a genuine removal (F-3).
        let affirmed_work: BTreeSet<Id> = affirmed_raw
            .iter()
            .copied()
            .filter(|id| types.get(id).is_none_or(|ty| ty.is_work()))
            .collect();

        let removed: Vec<Id> = affirmed_work.difference(&current).copied().collect();
        let added: Vec<Id> = current.difference(&affirmed_work).copied().collect();

        let outcome = if removed.is_empty() {
            let dropped_stale: Vec<Id> = affirmed_raw.difference(&affirmed_work).copied().collect();
            Some(Outcome::AutoExtended { added: added.clone(), dropped_stale })
        } else {
            provably_same_set(&removed, &added, id_remap)
        };

        if let Some(outcome) = outcome {
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

    // ----- s06 F-1: an authored addition auto-extends -----------------------

    #[test]
    fn authored_addition_auto_extends_an_affirmed_parent() {
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
        // A genuinely new slice, authored under the arc this same run — the
        // plan tree just declared it as scope (s06's premise).
        store
            .persist(&Document::new(node(z, 4, NodeType::Slice, "Z", Some(parent_id)), "# Z\n"))
            .unwrap();

        let report = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert_eq!(report.reaffirmed_count(), 0, "{:?}", report.changed);
        assert_eq!(report.auto_extended_count(), 1, "{:?}", report.changed);
        assert_eq!(report.left_as_drift_count(), 0, "{:?}", report.changed);
        assert!(
            matches!(&report.changed[0].outcome, Outcome::AutoExtended { added, dropped_stale } if added == &[z] && dropped_stale.is_empty())
        );

        // Auto-extended with no manual `node decomposed` — check clears too.
        let reloaded = store.load(parent_id).unwrap();
        let affirmed: BTreeSet<Id> =
            reloaded.frontmatter().decomposed().unwrap().children.iter().copied().collect();
        assert_eq!(affirmed, BTreeSet::from([x, z]));

        let all: Vec<Frontmatter> =
            store.load_all().unwrap().iter().map(|d| d.frontmatter().clone()).collect();
        let findings = odm_core::recompose::integrity(&all, &odm_core::gates::GateSets::default());
        assert!(
            !findings
                .iter()
                .any(|f| matches!(f.issue, odm_core::recompose::Issue::DecompositionDrift { .. })),
            "0 decomposition drift after auto-extend: {findings:?}"
        );
    }

    // ----- s06 F-2: the MF transitional shape auto-heals --------------------

    #[test]
    fn mf_transitional_shape_drops_stale_non_work_and_extends() {
        // A pre-s05 affirmation carrying a stale artifact id, plus a
        // genuinely new slice added this run — both anomalies clear in one
        // automatic step (MF's real shape).
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path());

        let parent_id = Id::new();
        let x = Id::new();
        let artifact = Id::new();
        let z = Id::new();

        let mut parent = node(parent_id, 2, NodeType::Arc, "Q", None);
        parent.affirm_decomposed(vec![x, artifact], day());
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
        store
            .persist(&Document::new(node(z, 5, NodeType::Slice, "Z", Some(parent_id)), "# Z\n"))
            .unwrap();

        let report = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert_eq!(report.auto_extended_count(), 1, "{:?}", report.changed);
        assert_eq!(report.left_as_drift_count(), 0, "{:?}", report.changed);
        assert!(
            matches!(&report.changed[0].outcome, Outcome::AutoExtended { added, dropped_stale } if added == &[z] && dropped_stale == &[artifact])
        );

        let reloaded = store.load(parent_id).unwrap();
        let affirmed: BTreeSet<Id> =
            reloaded.frontmatter().decomposed().unwrap().children.iter().copied().collect();
        assert_eq!(affirmed, BTreeSet::from([x, z]), "stale artifact dropped, new slice included");
    }

    // ----- s06 F-3: a genuine work-child removal is still drift ------------

    #[test]
    fn a_removal_with_no_remap_entry_is_left_as_drift() {
        // X disappears (no re-mint mapping for it) and nothing replaces it —
        // must not be silently dropped from the affirmation, and must not be
        // conflated with a stale non-work id (F-3, distinct from the
        // additions-only case F-1/F-2 auto-heal).
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path());

        let parent_id = Id::new();
        let x = Id::new();
        let mut parent = node(parent_id, 2, NodeType::Arc, "Q", None);
        parent.affirm_decomposed(vec![x], day());
        store.persist(&Document::new(parent, "# Q\n")).unwrap();
        // X's file is never persisted — it is simply gone from the corpus.

        let report = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert_eq!(report.auto_extended_count(), 0, "{:?}", report.changed);
        assert_eq!(report.left_as_drift_count(), 1, "{:?}", report.changed);
        let reloaded = store.load(parent_id).unwrap();
        assert_eq!(reloaded.frontmatter().decomposed().unwrap().children, vec![x], "untouched");
    }

    // ----- s06 F-4: a never-affirmed parent is never auto-affirmed ---------

    #[test]
    fn a_never_affirmed_parent_is_untouched() {
        let dir = TempDir::new().unwrap();
        let store = Store::open(dir.path());

        let parent_id = Id::new();
        let x = Id::new();
        let parent = node(parent_id, 2, NodeType::Arc, "Q", None); // no affirm_decomposed
        store.persist(&Document::new(parent, "# Q\n")).unwrap();
        store
            .persist(&Document::new(node(x, 3, NodeType::Slice, "X", Some(parent_id)), "# X\n"))
            .unwrap();

        let report = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert!(report.changed.is_empty(), "{:?}", report.changed);

        let reloaded = store.load(parent_id).unwrap();
        assert!(reloaded.frontmatter().decomposed().is_none(), "still undecomposed");
    }

    // ----- s06 F-5: idempotent -----------------------------------------------

    #[test]
    fn a_second_run_after_auto_extend_reports_nothing_new() {
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
        store
            .persist(&Document::new(node(z, 4, NodeType::Slice, "Z", Some(parent_id)), "# Z\n"))
            .unwrap();

        let first = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert_eq!(first.auto_extended_count(), 1, "{:?}", first.changed);

        let second = auto_recompose(&store, &HashMap::new(), day(), Mode::Commit).unwrap();
        assert!(second.changed.is_empty(), "second run is a no-op: {:?}", second.changed);
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
