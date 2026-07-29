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

use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDate;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::gates::GateSet;
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;

use crate::legacy::{self, LegacyDoc, LegacyFrontmatter};
use crate::selfhost::Repaired;
use crate::{MigrateError, Mode};

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
///
/// `source_path` is the legacy file this node was migrated from, and
/// `migrated_on` the date the migration ran — both go into the node's
/// `source` record (ODD-0025 §2.0/§2.2).
#[must_use]
pub fn build_node(
    id: Id,
    front: &LegacyFrontmatter,
    prep: &Prepared,
    gates: &DocGates,
    anchor: &std::path::Path,
    source_path: &std::path::Path,
    migrated_on: chrono::NaiveDate,
) -> Frontmatter {
    let node_type = classify_type(&front.tags);
    // The legacy frontmatter's own `created`/`updated` are authoritative when
    // present (they are the doc's real authored dates, which git blame on a
    // *migrated* node can't recover — the migrate commit isn't the original).
    // Only when a legacy doc carries **neither** field does this fall back to
    // the file's own git history (RH F-20) before finally falling back to
    // "today" (arc-migration-fidelity s10) — closing the one gap this
    // frontmatter-first design left.
    let created = front
        .created
        .or(front.updated)
        .unwrap_or_else(|| crate::fidelity::git_derived_dates(anchor, source_path, today()).0);
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
    // `author`/`version` are now typed fields (ODD-0025 §2.2) — preserved
    // explicitly rather than git-derived (git blame on a migrated node
    // returns the migrator, not the source author).
    if let Some(author) = &front.author {
        fm = fm.with_author(author.clone());
    }
    if let Some(version) = &front.version {
        fm = fm.with_version(version.clone());
    }
    let relative = crate::fidelity::relativize(anchor, source_path);
    fm = fm.with_source(crate::fidelity::build_source(
        vec![std::path::PathBuf::from(relative)],
        "odd",
        migrated_on,
    ));

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

/// One node `backfill_source` could not reconcile: its legacy source has
/// **drifted** since it was migrated — the body no longer matches, byte-for-
/// byte after normalization, the current legacy file (arc-migration-fidelity
/// s10, surfaced while running `backfill_source` live against `docs/design`:
/// ODD-0013 and ODD-0020, both actively amended throughout this rebuild,
/// hit exactly this).
///
/// `backfill_source` never overwrites a body it cannot verify (the hard
/// body-hash gate, ODD-0025 §2.1) — but rather than aborting the *whole*
/// batch on the first drifted node (as a single-node caller of
/// [`crate::fidelity::verify_body_hash`] would via `?`), a drift is a
/// **per-node skip**, disclosed here, not swallowed and not fatal: the other,
/// undrifted nodes in the same run still get backfilled. The node itself is
/// left completely untouched (no `source`, no body change) — reconciling a
/// living-doc drift is the arc's `s12` reconcile-run's job (the same gap
/// s08's CDC verification surfaced for an actively-edited arc-plan node);
/// this only ever reports it.
#[derive(Debug, Clone)]
pub struct Drifted {
    /// The node's number (unchanged — nothing was written).
    pub number: u32,
    /// The node's identity.
    pub id: Id,
    /// The node's name.
    pub name: String,
    /// The node's type (`design` or `research`).
    pub node_type: NodeType,
}

/// The outcome of a [`backfill_source`] run.
#[derive(Debug, Clone)]
pub struct BackfillReport {
    /// Nodes backfilled (or, under `--dry-run`, that would be backfilled).
    pub repaired: Vec<Repaired>,
    /// Nodes skipped because their legacy source has drifted — see
    /// [`Drifted`]'s doc for why this is a report, not a failure.
    pub drifted: Vec<Drifted>,
    /// Whether this was a dry run (nothing written).
    pub dry_run: bool,
}

impl BackfillReport {
    /// The number of nodes backfilled (or planned, under `--dry-run`).
    #[must_use]
    pub fn repaired_count(&self) -> usize {
        self.repaired.len()
    }

    /// The number of nodes skipped as drifted.
    #[must_use]
    pub fn drifted_count(&self) -> usize {
        self.drifted.len()
    }
}

/// Backfills `source` onto every existing `design`/`research` node in `store`
/// that lacks one — update-in-place (ODD-0025 §2.8), mirroring
/// [`crate::selfhost::repair`]'s shape but keyed on the **legacy corpus's**
/// number-based identity: unlike a plan-set arc/slice, a design/research node
/// has no structural-directory fallback to match by — `number` (the legacy
/// ODD's own frontmatter field, the pre-s05 identity this document family
/// still runs on — MF-3's residual scope) is the only handle available.
///
/// Matches each sourceless document node to its legacy file under
/// `legacy_path` by `number` ([`legacy::discover`]/[`legacy::parse_file`], the
/// same read [`crate::migrate`] uses), then reconciles it exactly as
/// [`crate::selfhost::reconcile_source`] does for a plan-set node: a stub body
/// ([`crate::fidelity::is_stub_body`]) is replaced with the verbatim legacy
/// body; a faithful non-stub body is kept and verified against the source
/// under the hard body-hash gate ([`crate::fidelity::verify_body_hash`]) — a
/// mismatch is never silently swallowed **and never aborts the batch**: it is
/// recorded as [`Drifted`] and the run continues (see that type's doc).
/// Containment is left untouched: design/research containment is optional
/// (ODD-0025 §2.7), so this backfill only ever adds `source`, `updated`, and
/// re-stamps the schema marker.
///
/// A node whose legacy `number` has no matching file under `legacy_path`
/// (moved or removed since) is left untouched, not an error — same
/// never-touches-what-it-can't-resolve discipline as `repair`. Idempotent: a
/// node that already carries `source` is skipped, so re-running is safe.
///
/// # Errors
///
/// [`MigrateError`] if the corpus can't be loaded or a persist fails.
pub fn backfill_source(
    store: &Store,
    legacy_path: &Path,
    mode: Mode,
) -> Result<BackfillReport, MigrateError> {
    let legacy_path_buf = legacy_path.canonicalize().unwrap_or_else(|_| legacy_path.to_path_buf());
    let legacy_path = legacy_path_buf.as_path();
    let anchor = crate::fidelity::anchor_for(legacy_path);

    let mut by_number: HashMap<u32, LegacyDoc> = HashMap::new();
    for path in legacy::discover(legacy_path) {
        if let Ok(doc) = legacy::parse_file(&path) {
            if let Some(number) = doc.front.number {
                by_number.insert(number, doc);
            }
        }
    }

    let corpus = store.load_all().map_err(MigrateError::LoadCorpus)?;
    let today = chrono::Utc::now().date_naive();
    let mut repaired = Vec::new();
    let mut drifted = Vec::new();
    for document in &corpus {
        let fm = document.frontmatter();
        if !matches!(fm.node_type(), NodeType::Design | NodeType::Research) {
            continue;
        }
        if fm.source().is_some() {
            continue;
        }
        let Some(legacy_doc) = by_number.get(&fm.number()) else {
            continue; // no matching legacy source file — left as-is
        };

        let new_body = if crate::fidelity::is_stub_body(document.body()) {
            legacy_doc.body.clone()
        } else {
            document.body().to_string()
        };
        let relative = crate::fidelity::relativize(&anchor, &legacy_doc.path);
        // Real `updated` from the legacy file's own last-touch commit (RH
        // F-20), not "the day backfill ran" — falls back to `today` only
        // when git has no record.
        let (_, updated) = crate::fidelity::git_derived_dates(&anchor, &legacy_doc.path, today);

        let mut new_fm = fm.clone();
        new_fm.set_updated(updated);
        new_fm.stamp_schema();
        let new_fm = new_fm.with_source(crate::fidelity::build_source(
            vec![std::path::PathBuf::from(relative)],
            "odd",
            today,
        ));

        let new_document = Document::new(new_fm, new_body);
        if crate::fidelity::verify_body_hash(
            &legacy_doc.body,
            new_document.body(),
            format!("#{} ({}) source backfill", fm.number(), fm.node_type()),
        )
        .is_err()
        {
            drifted.push(Drifted {
                number: fm.number(),
                id: fm.id(),
                name: fm.name().to_string(),
                node_type: fm.node_type(),
            });
            continue;
        }

        repaired.push(Repaired {
            number: fm.number(),
            id: fm.id(),
            name: new_document.frontmatter().name().to_string(),
            node_type: fm.node_type(),
        });

        if !mode.is_dry_run() {
            store
                .persist(&new_document)
                .map_err(|source| MigrateError::Persist { number: fm.number(), source })?;
        }
    }

    repaired.sort_by_key(|r| r.number);
    drifted.sort_by_key(|d| d.number);
    Ok(BackfillReport { repaired, drifted, dry_run: mode.is_dry_run() })
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
            version: None,
            component: None,
            tags: vec![],
            created: NaiveDate::from_ymd_opt(2026, 1, 1),
            updated: NaiveDate::from_ymd_opt(2026, 2, 1),
            state: Some("accepted".into()),
            supersedes: None,
            superseded_by: None,
        };
        let prep = prepare(&front).unwrap();
        let fm = build_node(
            Id::new(),
            &front,
            &prep,
            &gates,
            std::path::Path::new("."),
            std::path::Path::new("docs/design/04-accepted/0005-x.md"),
            NaiveDate::from_ymd_opt(2026, 7, 27).unwrap(),
        );
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
