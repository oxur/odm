//! The legacy → new **mapping** (ODD-0013 §9): how a legacy `DocState` scalar,
//! `number`, `supersedes` pair, and metadata become a node in the new model.
//!
//! The crux is `state → odd gate-set position`. The `odd` gate-set is canonical
//! (ODD-0013 §5.1): `draft → under-review → revised → accepted → active → final`.
//! A **progression** state maps to a *cumulative* reach up to and including its
//! gate (a `Final` doc has passed every earlier gate); a **retired** state
//! (`deferred`/`rejected`/`withdrawn`/`superseded`) has no `odd` gate — it maps
//! to a retirement marker (supersede-not-delete, git preserves history). An
//! unrecognized state is neither: the caller reports it and skips (M-6).

use chrono::NaiveDate;
use odm_core::frontmatter::Frontmatter;
use odm_core::gates::GateSet;
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};

use crate::legacy::LegacyFrontmatter;

/// The canonical `odd` gate sequence (ODD-0013 §5.1). The migration mapping is
/// pinned against this; a repo that customizes `[gates.odd]` can pass its own
/// [`GateSet`] to [`crate::migrate`], which overrides this default.
pub const ODD_GATES: [&str; 6] =
    ["draft", "under-review", "revised", "accepted", "active", "final"];

/// The canonical `odd` [`GateSet`] (ODD-0013 §5.1) — the mapping default when a
/// repo does not configure `[gates.odd]`.
#[must_use]
pub fn canonical_odd_gates() -> GateSet {
    GateSet::new(ODD_GATES.iter().map(|s| (*s).to_string()).collect())
}

/// How a legacy `state` scalar maps into the new model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateClass {
    /// A progression state → reach `odd` gates cumulatively up to `gate`.
    Progression {
        /// The terminal gate reached (all earlier gates are reached too).
        gate: &'static str,
    },
    /// A non-progression state → retire the node with this reason (the `odd`
    /// gate-set has no post-`final`/off-path gate; the reason preserves *which*
    /// dustbin/parked state it was).
    Retired {
        /// The retirement reason (the lowercased legacy state).
        reason: &'static str,
    },
}

/// Classifies a legacy `state` scalar (case-insensitive), or `None` if it is not
/// a recognized legacy state (→ the caller reports a skip, never guesses a gate).
#[must_use]
pub fn classify_state(state: &str) -> Option<StateClass> {
    match state.trim().to_ascii_lowercase().as_str() {
        "draft" => Some(StateClass::Progression { gate: "draft" }),
        "under-review" | "under review" => Some(StateClass::Progression { gate: "under-review" }),
        "revised" => Some(StateClass::Progression { gate: "revised" }),
        "accepted" => Some(StateClass::Progression { gate: "accepted" }),
        "active" => Some(StateClass::Progression { gate: "active" }),
        "final" => Some(StateClass::Progression { gate: "final" }),
        "deferred" => Some(StateClass::Retired { reason: "deferred" }),
        "rejected" => Some(StateClass::Retired { reason: "rejected" }),
        "withdrawn" => Some(StateClass::Retired { reason: "withdrawn" }),
        "superseded" => Some(StateClass::Retired { reason: "superseded" }),
        _ => None,
    }
}

/// A validated legacy doc ready to become a node: the required fields resolved,
/// the state classified. Built by [`prepare`]; consumed by [`build_node`].
#[derive(Debug, Clone)]
pub struct Prepared {
    /// The preserved legacy number (also the idempotence key).
    pub number: u32,
    /// The classified state disposition.
    pub class: StateClass,
    /// The last-updated date (gate/retire date; falls back to `created`/today).
    pub updated: NaiveDate,
}

/// Why a legacy doc cannot be mapped (a reported skip, never a panic — M-6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapError {
    /// No `number` field — the legacy identity (and idempotence key) is missing.
    MissingNumber,
    /// The `state` scalar is not a recognized legacy state.
    UnknownState(String),
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapError::MissingNumber => write!(f, "missing `number` (legacy identity)"),
            MapError::UnknownState(s) => write!(f, "unknown legacy state {s:?}"),
        }
    }
}

/// Validates the required fields and classifies the state, or reports why it
/// cannot be mapped.
///
/// # Errors
///
/// [`MapError::MissingNumber`] if `number` is absent; [`MapError::UnknownState`]
/// if `state` is missing or not a recognized legacy state.
pub fn prepare(front: &LegacyFrontmatter) -> Result<Prepared, MapError> {
    let number = front.number.ok_or(MapError::MissingNumber)?;
    let state = front.state.as_deref().unwrap_or_default();
    let class = classify_state(state).ok_or_else(|| MapError::UnknownState(state.to_string()))?;
    let updated = front.updated.or(front.created).unwrap_or_else(today);
    Ok(Prepared { number, class, updated })
}

/// Builds the new-model [`Frontmatter`] for a prepared legacy doc under the
/// reserved identity `id`, carrying every mapped field. The `supersedes` edge (if
/// any) is attached by the caller after ids are resolved; a retirement or the
/// cumulative gate reach is applied here from the classified state.
///
/// The `id` is reserved by the caller (pass 1) so the resolved supersession
/// edges point at the right node; `odd_gates` is the gate-set the reach is
/// validated against (a mis-set gate is a bug, not a user error — the reach only
/// ever uses [`ODD_GATES`] names).
#[must_use]
pub fn build_node(
    id: Id,
    front: &LegacyFrontmatter,
    prep: &Prepared,
    odd_gates: &GateSet,
) -> Frontmatter {
    let created = front.created.or(front.updated).unwrap_or_else(today);
    let name = front.title.clone().unwrap_or_else(|| format!("ODD-{:04}", prep.number));

    let mut fm = Frontmatter::new(
        id,
        prep.number,
        NodeType::Odd,
        name,
        created,
        prep.updated,
        // Migrated design docs were deliberate, planned artifacts; the model has
        // no "migrated" origin, so `planned` is the faithful choice.
        Origin::Planned,
    );

    if !front.tags.is_empty() {
        fm = fm.with_tags(front.tags.clone());
    }
    if let Some(component) = &front.component {
        fm = fm.with_component(component.clone());
    }
    // The node schema has no typed `author` field (ODD-0013 §2.3 dropped it);
    // carry it into the forward-compat catch-all so it is not lost.
    if let Some(author) = &front.author {
        fm.insert_extra("author", author.clone());
    }

    match &prep.class {
        StateClass::Progression { gate } => {
            reach_cumulative(&mut fm, odd_gates, gate, prep.updated);
        }
        StateClass::Retired { reason } => {
            fm.retire(*reason, prep.updated);
        }
    }
    fm
}

/// Records the `odd` gates from the start of the sequence up to and including
/// `terminal`, each at [`Evidence::Asserted`] (the honest level for a historical
/// migration — claimed from the legacy record, not independently reproduced) on
/// `on`. A `Final` doc thus shows the full passed sequence, not a lone gate.
fn reach_cumulative(fm: &mut Frontmatter, odd_gates: &GateSet, terminal: &str, on: NaiveDate) {
    for gate in odd_gates.sequence() {
        // set_gate validates against the set; every ODD_GATES name is in it.
        let _ = fm.status_mut().set_gate(odd_gates, gate, None, Evidence::Asserted, on);
        if gate == terminal {
            break;
        }
    }
}

/// The migration's fallback date when a legacy doc omits both `created` and
/// `updated` (rare). Uses UTC "today", matching the CLI convention.
fn today() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_state_maps_progression_and_retired() {
        assert_eq!(classify_state("Draft"), Some(StateClass::Progression { gate: "draft" }));
        assert_eq!(classify_state("FINAL"), Some(StateClass::Progression { gate: "final" }));
        assert_eq!(classify_state("Rejected"), Some(StateClass::Retired { reason: "rejected" }));
        assert_eq!(classify_state("deferred"), Some(StateClass::Retired { reason: "deferred" }));
        assert_eq!(classify_state("bogus"), None);
    }

    #[test]
    fn reach_cumulative_reaches_all_gates_up_to_terminal() {
        let gates = canonical_odd_gates();
        let front = LegacyFrontmatter {
            number: Some(5),
            title: Some("X".into()),
            author: None,
            component: None,
            tags: vec![],
            created: NaiveDate::from_ymd_opt(2026, 1, 1),
            updated: NaiveDate::from_ymd_opt(2026, 2, 1),
            state: Some("accepted".into()),
            supersedes: None,
            superseded_by: None,
        };
        let prep = prepare(&front).unwrap();
        let fm = build_node(Id::new(), &front, &prep, &gates);
        // Cumulative: draft..=accepted reached; active/final not.
        for g in ["draft", "under-review", "revised", "accepted"] {
            assert!(fm.status().has_reached(g), "{g} should be reached");
        }
        assert!(!fm.status().has_reached("active"));
        assert!(!fm.status().has_reached("final"));
        assert!(fm.retired().is_none());
    }
}
