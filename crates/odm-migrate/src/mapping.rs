//! The legacy → new **mapping** (ODD-0013 §9): how a legacy `DocState` scalar,
//! `number`, `supersedes` pair, and metadata become a node in the new model.
//!
//! The crux is `state → document gate-set position`. The `design` gate-set is
//! canonical (ODD-0013 §5.1): `draft → under-review → revised → accepted →
//! active → final`, and `research` mirrors it (C-2 operator decision), so a
//! source doc in any state directory maps with no special-casing.
//! A **progression** state maps to a *cumulative* reach up to and including its
//! gate (a `Final` doc has passed every earlier gate); a **retired** state
//! (`deferred`/`rejected`/`withdrawn`/`superseded`) has no document gate — it
//! maps to a retirement marker (supersede-not-delete, git preserves history). An
//! unrecognized state is neither: the caller reports it and skips (M-6).
//!
//! **Which document type** a legacy doc becomes is decided by
//! [`classify_type`]: `research` iff its `tags` include `research`, else
//! `design` (ODD-0013 §2.2 v2.0).

use chrono::NaiveDate;
use odm_core::frontmatter::Frontmatter;
use odm_core::gates::GateSet;
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};

use crate::legacy::LegacyFrontmatter;

/// The canonical `design` gate sequence (ODD-0013 §5.1). The migration mapping
/// is pinned against this; a repo that customizes `[gates.design]` can pass its
/// own [`GateSet`] to [`crate::migrate`], which overrides this default.
pub const DESIGN_GATES: [&str; 6] =
    ["draft", "under-review", "revised", "accepted", "active", "final"];

/// The canonical `research` gate sequence. It **mirrors [`DESIGN_GATES`]**
/// (ODD-0013 §5.1 v2.0, operator decision 2026-07-26): the importer maps a
/// source doc's state directory onto a gate reach, so a shared sequence lets a
/// research doc sit in any state directory with no bespoke mapping. Named
/// separately so a tighter research lifecycle stays a one-line change.
pub const RESEARCH_GATES: [&str; 6] = DESIGN_GATES;

/// The canonical `design` [`GateSet`] (ODD-0013 §5.1) — the mapping default when
/// a repo does not configure `[gates.design]`.
#[must_use]
pub fn canonical_design_gates() -> GateSet {
    GateSet::new(DESIGN_GATES.iter().map(|s| (*s).to_string()).collect())
}

/// The canonical `research` [`GateSet`] — the mapping default when a repo does
/// not configure `[gates.research]`.
#[must_use]
pub fn canonical_research_gates() -> GateSet {
    GateSet::new(RESEARCH_GATES.iter().map(|s| (*s).to_string()).collect())
}

/// The gate-sets the importer validates a document node's reach against — one
/// per document type it can produce.
///
/// Carrying both (rather than the single set the pre-C-2 importer took) is what
/// lets `research` diverge from `design` later without touching call sites: the
/// two sequences are identical today, and [`Self::for_type`] is the one place
/// that would notice if they stopped being.
#[derive(Debug, Clone)]
pub struct DocGates {
    /// The `design` gate-set.
    pub design: GateSet,
    /// The `research` gate-set.
    pub research: GateSet,
}

impl DocGates {
    /// The canonical pair (ODD-0013 §5.1) — the default when a repo configures
    /// neither `[gates.design]` nor `[gates.research]`.
    #[must_use]
    pub fn canonical() -> Self {
        Self { design: canonical_design_gates(), research: canonical_research_gates() }
    }

    /// The gate-set governing `node_type`.
    ///
    /// Any non-document type falls back to `design`; the importer only ever
    /// builds `design`/`research` nodes ([`classify_type`]), so this is a
    /// defensive default rather than a reachable branch.
    #[must_use]
    pub fn for_type(&self, node_type: NodeType) -> &GateSet {
        match node_type {
            NodeType::Research => &self.research,
            _ => &self.design,
        }
    }
}

/// Classifies a legacy doc's **document type** from its `tags`: `research` iff
/// the tags include `research` (case-insensitive), else `design`.
///
/// Tag-based rather than title- or filename-based (ODD-0013 §2.2 v2.0): the tag
/// is a deliberate authoring act that survives renames and re-titling, where a
/// `Research —` title prefix is a convention a doc can silently drift out of.
#[must_use]
pub fn classify_type(tags: &[String]) -> NodeType {
    if tags.iter().any(|t| t.trim().eq_ignore_ascii_case("research")) {
        NodeType::Research
    } else {
        NodeType::Design
    }
}

/// How a legacy `state` scalar maps into the new model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateClass {
    /// A progression state → reach document gates cumulatively up to `gate`.
    Progression {
        /// The terminal gate reached (all earlier gates are reached too).
        gate: &'static str,
    },
    /// A non-progression state → retire the node with this reason (the document
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
/// The node's type is [`classify_type`]'s verdict over the source `tags`
/// (`research` or `design`), and its gate reach is validated against the
/// matching set from `gates`.
///
/// The `id` is reserved by the caller (pass 1) so the resolved supersession
/// edges point at the right node; a mis-set gate is a bug, not a user error —
/// the reach only ever uses [`DESIGN_GATES`]/[`RESEARCH_GATES`] names.
#[must_use]
pub fn build_node(
    id: Id,
    front: &LegacyFrontmatter,
    prep: &Prepared,
    gates: &DocGates,
) -> Frontmatter {
    let node_type = classify_type(&front.tags);
    let created = front.created.or(front.updated).unwrap_or_else(today);
    let name = front.title.clone().unwrap_or_else(|| format!("ODD-{:04}", prep.number));

    let mut fm = Frontmatter::new(
        id,
        prep.number,
        node_type,
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
            reach_cumulative(&mut fm, gates.for_type(node_type), gate, prep.updated);
        }
        StateClass::Retired { reason } => {
            fm.retire(*reason, prep.updated);
        }
    }
    // Every imported node is stamped the current schema (`design/v1.0` or
    // `research/v1.0`, ODD-0020 V-4): the legacy source (no `schema:`) is
    // understood as v0.1; the new node is v1.0.
    fm.stamp_schema();
    fm
}

/// Records the document gates from the start of the sequence up to and including
/// `terminal`, each at [`Evidence::Asserted`] (the honest level for a historical
/// migration — claimed from the legacy record, not independently reproduced) on
/// `on`. A `Final` doc thus shows the full passed sequence, not a lone gate.
fn reach_cumulative(fm: &mut Frontmatter, doc_gates: &GateSet, terminal: &str, on: NaiveDate) {
    for gate in doc_gates.sequence() {
        // set_gate validates against the set; every canonical gate name is in it.
        let _ = fm.status_mut().set_gate(doc_gates, gate, None, Evidence::Asserted, on);
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
        let gates = DocGates::canonical();
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

/// The document-role suffixes a derived name may carry, and which are stripped
/// (ODD-0013 §2.1 v2.3).
///
/// A closed list on purpose. The rule prohibits *metadata*, not *qualifiers*,
/// and only a fixed set of labels is mechanically known to be the former —
/// `"(v-major rebuild)"` is part of a name's meaning, `"(plan-of-record)"` is
/// the role of the document the name was copied from.
const ROLE_SUFFIXES: [&str; 2] = ["(plan-of-record)", "(build plan)"];

/// The type words a derived name may be prefixed with.
const TYPE_PREFIXES: [&str; 4] = ["slice", "arc", "phase", "step"];

/// Strips positional and role metadata from a derived name (ODD-0013 §2.1
/// v2.3, F-18).
///
/// Enforcement lives here, at mint time, rather than in the display layer: a
/// name copied verbatim from a plan document's heading carries that document's
/// coordinates (`"Slice 01 (Arc 02) — …"`) and role (`"… (plan-of-record)"`),
/// both of which `number`, the `part_of` tree and `type` already record, and
/// both of which go stale on any renumber or re-role.
///
/// Deliberately conservative — it removes **one** leading positional prefix and
/// the known role suffixes, and nothing else. A name that would be emptied is
/// left alone, because an unhelpful name beats no name.
#[must_use]
pub fn normalize_name(name: &str) -> String {
    let stripped = strip_role_suffix(strip_positional_prefix(name.trim()));
    let stripped = stripped.trim();
    if stripped.is_empty() { name.trim().to_string() } else { stripped.to_string() }
}

/// Removes a leading `"<Type> NN[.M] [(Arc NN)] —|:"`, if present.
///
/// Only the *first* separator is consumed: `"Slice 05 (Arc 06): UAT — CLI
/// feedback"` must become `"UAT — CLI feedback"`, not `"CLI feedback"` — the
/// second dash belongs to the name.
fn strip_positional_prefix(name: &str) -> &str {
    let lower = name.to_ascii_lowercase();
    let Some(kind) = TYPE_PREFIXES.iter().find(|p| lower.starts_with(&format!("{p} "))) else {
        return name;
    };
    let rest = name[kind.len() + 1..].trim_start();

    // The number, possibly dotted (`05.1`).
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    if digits.is_empty() || !digits.starts_with(|c: char| c.is_ascii_digit()) {
        return name;
    }
    let rest = rest[digits.len()..].trim_start();

    // An optional parenthesised coordinate: `(Arc 06)`.
    let rest = match rest.strip_prefix('(') {
        Some(after) => match after.split_once(')') {
            Some((inside, tail)) if is_coordinate(inside) => tail.trim_start(),
            _ => return name,
        },
        None => rest,
    };

    // The separator that ends the prefix — and only that one.
    for sep in ["—", "–", ":", "-"] {
        if let Some(tail) = rest.strip_prefix(sep) {
            return tail.trim_start();
        }
    }
    name
}

/// Whether a parenthesised fragment is a positional coordinate (`Arc 06`)
/// rather than a descriptive qualifier (`v-major rebuild`).
fn is_coordinate(inside: &str) -> bool {
    let lower = inside.trim().to_ascii_lowercase();
    TYPE_PREFIXES.iter().any(|p| {
        lower
            .strip_prefix(&format!("{p} "))
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit() || c == '.'))
    })
}

/// Removes a trailing role suffix from [`ROLE_SUFFIXES`], if present.
fn strip_role_suffix(name: &str) -> &str {
    let trimmed = name.trim_end();
    for suffix in ROLE_SUFFIXES {
        if let Some(head) = trimmed.strip_suffix(suffix) {
            return head.trim_end();
        }
    }
    trimmed
}

#[cfg(test)]
mod name_tests {
    use super::normalize_name;

    /// Every shape the real corpus carries (F-18), verified case by case.
    #[test]
    fn test_positional_prefixes_and_role_suffixes_are_stripped() {
        for (raw, want) in [
            ("Arc 01 — Substrate & node CRUD (plan-of-record)", "Substrate & node CRUD"),
            ("Slice 01 — Workspace scaffolding (plan-of-record)", "Workspace scaffolding"),
            (
                "Slice 01 (Arc 02) — Graph construction + reverse edges (plan-of-record)",
                "Graph construction + reverse edges",
            ),
            (
                "Slice 01 (Arc 05): `desired_facts` schema + `Probe` trait + shell probe",
                "`desired_facts` schema + `Probe` trait + shell probe",
            ),
            (
                "Slice 05.1 (Arc 02) — Evidence-transition dates (plan-of-record)",
                "Evidence-transition dates",
            ),
            ("Slice 03 (Arc 06): schema versioning (ODD-0020)", "schema versioning (ODD-0020)"),
        ] {
            assert_eq!(normalize_name(raw), want, "normalizing {raw:?}");
        }
    }

    #[test]
    fn test_only_the_prefix_separator_is_consumed() {
        // The second dash belongs to the name, not to the coordinate.
        assert_eq!(normalize_name("Slice 05 (Arc 06): UAT — CLI feedback"), "UAT — CLI feedback");
    }

    #[test]
    fn test_descriptive_parentheticals_survive() {
        // The rule prohibits metadata, not qualifiers.
        for name in [
            "odm — Architecture & Design (v-major rebuild)",
            "odm — Project Definition (v-major rebuild)",
            "Research — odm-index: incremental indexing & caching (no DB, no FTS)",
        ] {
            assert_eq!(normalize_name(name), name, "{name:?} must be left alone");
        }
    }

    #[test]
    fn test_names_without_metadata_are_untouched() {
        for name in ["Workspace scaffolding", "Rollup & orient", "odm v1.0.0 — Project Plan"] {
            assert_eq!(normalize_name(name), name);
        }
    }

    #[test]
    fn test_a_coordinate_is_distinguished_from_a_qualifier() {
        // `(Arc 06)` is a coordinate and goes; `(draft)` is not a coordinate,
        // so the prefix is not recognised and the name is left whole.
        assert_eq!(
            normalize_name("Slice 02 (Arc 06): migrate odm's own docs"),
            "migrate odm's own docs"
        );
        assert_eq!(normalize_name("Slice 02 (draft) — something"), "Slice 02 (draft) — something");
    }

    #[test]
    fn test_normalizing_never_empties_a_name() {
        for name in ["Slice 05 —", "Arc 01 (plan-of-record)", "(plan-of-record)"] {
            assert!(!normalize_name(name).is_empty(), "{name:?} kept something");
        }
    }

    #[test]
    fn test_normalization_is_idempotent() {
        for raw in [
            "Arc 01 — Substrate & node CRUD (plan-of-record)",
            "Slice 05 (Arc 06): UAT — CLI feedback",
        ] {
            let once = normalize_name(raw);
            assert_eq!(normalize_name(&once), once, "re-running changes nothing");
        }
    }
}
