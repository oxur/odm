//! The on-disk node format: a `---`-delimited YAML frontmatter block followed
//! by a markdown body.
//!
//! [`Document::parse`] splits and deserializes a node file; [`Document::emit`]
//! serializes it back in the canonical field order of ODD-0013 §2.3. The
//! headline invariant is **`parse ∘ emit == identity`**: emitting a document
//! and parsing the result yields an equal document, including any keys not yet
//! modeled by the schema, which are preserved verbatim.
//!
//! # YAML library isolation
//!
//! The YAML backend (`serde_norway`) is used **only** inside this module — no
//! YAML-crate type appears in the public API, so the backend can be swapped
//! without touching callers. (Same insurance applied to `ulid` in slice 02.)

use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_norway::Mapping;

use crate::desired::DesiredFact;
use crate::{Id, NodeType, Origin};

/// The frontmatter delimiter line.
const FENCE: &str = "---";

/// A parsed node file: typed [`Frontmatter`] plus the markdown `body`.
///
/// # Examples
///
/// ```
/// use odm_core::frontmatter::Document;
///
/// let text = "\
/// ---
/// id: 01ARZ3NDEKTSV4RRFFQ69G5FAV
/// number: 7
/// type: slice
/// name: Store layer
/// created: 2026-06-20
/// updated: 2026-06-20
/// origin: planned
/// reserved: false
/// ---
/// # Store layer
///
/// Body text.
/// ";
/// let doc = Document::parse(text)?;
/// assert_eq!(doc.frontmatter().number(), 7);
/// // Emitting and re-parsing yields an equal document.
/// assert_eq!(Document::parse(&doc.emit()?)?, doc);
/// # Ok::<(), odm_core::frontmatter::FrontmatterError>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    frontmatter: Frontmatter,
    body: String,
}

impl Document {
    /// Assembles a document from typed frontmatter and a markdown body.
    pub fn new(frontmatter: Frontmatter, body: impl Into<String>) -> Self {
        Self { frontmatter, body: body.into() }
    }

    /// The typed frontmatter.
    #[must_use]
    pub fn frontmatter(&self) -> &Frontmatter {
        &self.frontmatter
    }

    /// The markdown body (everything after the closing `---`).
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Mutable access to the frontmatter, for in-place edits (rename, retire,
    /// supersede) that must preserve everything else — including unknown keys.
    pub fn frontmatter_mut(&mut self) -> &mut Frontmatter {
        &mut self.frontmatter
    }

    /// Replaces the markdown body.
    ///
    /// The body is the node's human content, normally edited by a person in the
    /// file itself. The one programmatic writer is the plan re-derivation, which
    /// writes the project's `# Vision` section from the plan document it was
    /// derived from (RH L-3a).
    pub fn set_body(&mut self, body: impl Into<String>) {
        self.body = body.into();
    }

    /// Parses a node file into typed frontmatter plus its body.
    ///
    /// The text must begin with a `---` line, contain a closing `---` line, and
    /// carry valid YAML in between. Everything after the first closing `---` is
    /// taken as the body verbatim (so the body may itself contain `---` lines).
    ///
    /// # Errors
    ///
    /// - [`FrontmatterError::MissingOpen`] if the text does not start with `---`.
    /// - [`FrontmatterError::Unterminated`] if there is no closing `---`.
    /// - [`FrontmatterError::Yaml`] if the frontmatter block is not valid YAML
    ///   or does not match the schema (the message carries the position where
    ///   the YAML backend reports one).
    pub fn parse(text: &str) -> Result<Self, FrontmatterError> {
        let mut lines = text.split('\n');
        if lines.next() != Some(FENCE) {
            return Err(FrontmatterError::MissingOpen);
        }

        let mut yaml_lines: Vec<&str> = Vec::new();
        let mut body_lines: Vec<&str> = Vec::new();
        let mut closed = false;
        for line in lines {
            if !closed && line == FENCE {
                closed = true;
                continue;
            }
            if closed {
                body_lines.push(line);
            } else {
                yaml_lines.push(line);
            }
        }
        if !closed {
            return Err(FrontmatterError::Unterminated);
        }

        let yaml = yaml_lines.join("\n");
        let frontmatter = serde_norway::from_str(&yaml).map_err(FrontmatterError::from_yaml)?;
        Ok(Self { frontmatter, body: body_lines.join("\n") })
    }

    /// Serializes the document back to its on-disk form, with the frontmatter in
    /// canonical field order.
    ///
    /// `Document::parse(&doc.emit()?)? == doc` for every well-formed document.
    ///
    /// # Errors
    ///
    /// Returns [`FrontmatterError::Yaml`] if the frontmatter cannot be
    /// serialized (not reachable for values produced by this crate).
    pub fn emit(&self) -> Result<String, FrontmatterError> {
        let mut yaml =
            serde_norway::to_string(&self.frontmatter).map_err(FrontmatterError::from_yaml)?;
        if !yaml.ends_with('\n') {
            yaml.push('\n');
        }
        let mut out = String::with_capacity(yaml.len() + self.body.len() + 8);
        out.push_str(FENCE);
        out.push('\n');
        out.push_str(&yaml);
        out.push_str(FENCE);
        out.push('\n');
        out.push_str(&self.body);
        Ok(out)
    }
}

/// The typed frontmatter schema (ODD-0013 §2.3, amended by ODD-0025 §4 for
/// `author`/`version`/`source`).
///
/// Fields are declared — and therefore emitted — in canonical order: `id`,
/// `number`, `type`, `schema`, `name`, `created`, `updated`, `tags`,
/// `component`, `author`, `version`, `origin`, `reserved`, `retired`,
/// `source`, `edges`, `status`, `decomposed`, `desired_facts`, `deferred`.
/// Any keys not modeled here are captured in a hidden catch-all and
/// re-emitted last, so they survive a round-trip until their owning slices
/// model them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Stable ULID identity.
    id: Id,
    /// Human-facing number (metadata, not identity).
    number: u32,
    #[serde(rename = "type")]
    node_type: NodeType,
    /// The per-type versioned schema marker (`<type>/vMAJOR.MINOR`, ODD-0020).
    /// Absent ⇒ `v0.1` on read (a *computed* legacy default — see
    /// [`schema_version`](Frontmatter::schema_version)); every odm-created node is
    /// stamped `<type>/`[`SchemaVersion::CURRENT`](crate::schema::SchemaVersion::CURRENT)
    /// at creation. Additive + skipped-when-absent, so an unversioned legacy
    /// node round-trips byte-identically.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    schema: Option<crate::schema::SchemaMarker>,
    /// Human label.
    name: String,
    /// Creation date (the human copy; also encoded in the ULID).
    created: NaiveDate,
    /// Last-updated date.
    updated: NaiveDate,
    /// Free-form filter labels.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    /// Optional subsystem/component filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    component: Option<String>,
    /// The document's original author (ODD-0025 §2.2), preserved explicitly on
    /// migration rather than git-derived — git blame on a migrated node
    /// returns the migrator, not the source author (ODD-0002 §2.2's original
    /// git-derivation intent is wrong for migrated docs). Absent on a
    /// hand-created node. Document-node only; [`crate::check::content_validity`]
    /// flags it on a work node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    /// The document's own content-version marker (ODD-0025 §2.2) — the
    /// source-of-truth quick-access answer to "what version is this doc,"
    /// distinct from [`schema`](Self::schema) (which versions the frontmatter
    /// *shape*, ODD-0020 §3, not the document's content). Absent on a
    /// hand-created node. Document-node only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    /// How the node arose.
    origin: Origin,
    /// Tentative future-work placeholder flag.
    #[serde(default)]
    reserved: bool,
    /// Retirement marker, set by `odm retire`. Absent unless the node has been
    /// withdrawn. (Not in the ODD-0013 §2.3 example yet — see slice05 report.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    retired: Option<Retirement>,
    /// The node's **source record** (ODD-0025 §2.0/§2.2) — set only on a
    /// migrated node; absent on a hand-created one. See [`Source`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<Source>,
    /// The node's outgoing edges.
    #[serde(default, skip_serializing_if = "Edges::is_empty")]
    edges: Edges,
    /// The multi-gate, evidence-tagged status vector (ODD-0013 §2.3/§5.1).
    /// Typed since arc02 slice04 (previously preserved as an unknown key).
    #[serde(default, skip_serializing_if = "crate::status::Status::is_empty")]
    status: crate::status::Status,
    /// The guarded "decomposition complete" assertion (ODD-0013 §4.5), set when
    /// a parent affirms its child set fully accounts for its scope. Absent until
    /// affirmed. Typed since arc02 slice05.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    decomposed: Option<Decomposition>,
    /// The node's declared desired-state facts (ODD-0013 §5.2): checkable
    /// claims about the world that `reconcile` probes for drift. Absent/empty is
    /// the default. Typed since arc05 slice01 (previously preserved as an
    /// unknown key). Skipped on emit when empty, so a fact-free node round-trips
    /// to byte-identical frontmatter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    desired_facts: Vec<DesiredFact>,
    /// The node's deferred marker (ODD-0013 Q-A3-1): parked work with a
    /// checkable re-entry condition. Absent ⇒ not deferred (the default). Typed
    /// since arc05 slice06; skipped on emit when absent (YAML-additive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    deferred: Option<Deferral>,
    /// Keys not yet modeled, preserved verbatim across a round-trip (forward
    /// compatibility for schema additions not yet typed).
    #[serde(flatten)]
    extra: Mapping,
}

impl Frontmatter {
    /// Creates frontmatter with the required fields; optional fields start
    /// empty and can be set with the `with_*` methods.
    pub fn new(
        id: Id,
        number: u32,
        node_type: NodeType,
        name: impl Into<String>,
        created: NaiveDate,
        updated: NaiveDate,
        origin: Origin,
    ) -> Self {
        Self {
            id,
            number,
            node_type,
            schema: None,
            name: name.into(),
            created,
            updated,
            tags: Vec::new(),
            component: None,
            author: None,
            version: None,
            origin,
            reserved: false,
            retired: None,
            source: None,
            edges: Edges::default(),
            status: crate::status::Status::new(),
            decomposed: None,
            desired_facts: Vec::new(),
            deferred: None,
            extra: Mapping::new(),
        }
    }

    /// Sets the filter tags.
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Sets the component/subsystem label.
    #[must_use]
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }

    /// Sets the document's original author (ODD-0025 §2.2).
    #[must_use]
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Sets the document's own content-version marker (ODD-0025 §2.2).
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the node's source record (ODD-0025 §2.0/§2.2).
    #[must_use]
    pub fn with_source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the `reserved` placeholder flag.
    #[must_use]
    pub fn with_reserved(mut self, reserved: bool) -> Self {
        self.reserved = reserved;
        self
    }

    /// Sets the outgoing edges.
    #[must_use]
    pub fn with_edges(mut self, edges: Edges) -> Self {
        self.edges = edges;
        self
    }

    /// Sets the declared desired-state facts (ODD-0013 §5.2).
    #[must_use]
    pub fn with_desired_facts(mut self, desired_facts: Vec<DesiredFact>) -> Self {
        self.desired_facts = desired_facts;
        self
    }

    /// Sets (or clears) the deferred marker (Q-A3-1).
    #[must_use]
    pub fn with_deferred(mut self, deferred: Option<Deferral>) -> Self {
        self.deferred = deferred;
        self
    }

    /// The stable identity.
    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }

    /// The human number.
    #[must_use]
    pub fn number(&self) -> u32 {
        self.number
    }

    /// The node type.
    #[must_use]
    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    /// The human label.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The creation date.
    #[must_use]
    pub fn created(&self) -> NaiveDate {
        self.created
    }

    /// The last-updated date.
    #[must_use]
    pub fn updated(&self) -> NaiveDate {
        self.updated
    }

    /// The filter tags.
    #[must_use]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// The component/subsystem label, if set.
    #[must_use]
    pub fn component(&self) -> Option<&str> {
        self.component.as_deref()
    }

    /// The document's original author, if migrated with one (ODD-0025 §2.2).
    #[must_use]
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    /// The document's own content-version marker, if migrated with one
    /// (ODD-0025 §2.2).
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// The node's source record, if migrated (ODD-0025 §2.0/§2.2).
    #[must_use]
    pub fn source(&self) -> Option<&Source> {
        self.source.as_ref()
    }

    /// How the node arose.
    #[must_use]
    pub fn origin(&self) -> Origin {
        self.origin
    }

    /// The `reserved` placeholder flag.
    #[must_use]
    pub fn reserved(&self) -> bool {
        self.reserved
    }

    /// The outgoing edges.
    #[must_use]
    pub fn edges(&self) -> &Edges {
        &self.edges
    }

    /// The node's declared desired-state facts (empty if none are declared).
    #[must_use]
    pub fn desired_facts(&self) -> &[DesiredFact] {
        &self.desired_facts
    }

    /// The node's deferred marker, if it has parked itself (Q-A3-1).
    #[must_use]
    pub fn deferred(&self) -> Option<&Deferral> {
        self.deferred.as_ref()
    }

    /// The retirement marker, if the node has been retired.
    #[must_use]
    pub fn retired(&self) -> Option<&Retirement> {
        self.retired.as_ref()
    }

    /// The node's status vector (the gates it has reached).
    #[must_use]
    pub fn status(&self) -> &crate::status::Status {
        &self.status
    }

    /// Mutable access to the status vector (e.g. to record a reached gate).
    pub fn status_mut(&mut self) -> &mut crate::status::Status {
        &mut self.status
    }

    /// The guarded "decomposition complete" assertion, if the node has affirmed
    /// it (ODD-0013 §4.5).
    #[must_use]
    pub fn decomposed(&self) -> Option<&Decomposition> {
        self.decomposed.as_ref()
    }

    /// Affirms (or re-affirms) that `children` fully account for this node's
    /// scope as of `on` — "no missing, no extra" (ODD-0013 §4.5). The child set
    /// is sorted and de-duplicated so a later add/remove is detectable as drift
    /// (see [`crate::recompose`]).
    pub fn affirm_decomposed(&mut self, children: Vec<Id>, on: NaiveDate) {
        let mut children = children;
        children.sort_unstable();
        children.dedup();
        self.decomposed = Some(Decomposition { on, children });
    }

    /// The number of preserved-but-unmodeled top-level keys (forward
    /// compatibility for schema additions not yet typed).
    #[must_use]
    pub fn unknown_key_count(&self) -> usize {
        self.extra.len()
    }

    /// Changes the human label. Does not touch `id` or the on-disk path.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Corrects the creation date.
    ///
    /// Ordinarily `created` is written once, at mint time, and never touched —
    /// it is a fact about the node's origin, not a mutable field. The exception
    /// is a **derivation fix**: when the importer recorded the date it *ran*
    /// rather than the date the work began, correcting it is restoring the
    /// fact, not changing it (RH F-20). Note that `created` decides the node's
    /// month shard, so a caller that changes it must relocate the file.
    pub fn set_created(&mut self, created: NaiveDate) {
        self.created = created;
    }

    /// Sets the last-updated date (bumped by edits).
    pub fn set_updated(&mut self, updated: NaiveDate) {
        self.updated = updated;
    }

    /// Mutable access to the outgoing edges (e.g. to record a `supersedes`).
    pub fn edges_mut(&mut self) -> &mut Edges {
        &mut self.edges
    }

    /// Marks the node retired with a reason and date. The node's file is kept;
    /// retirement is recorded in frontmatter, never by deleting the file.
    pub fn retire(&mut self, reason: impl Into<String>, on: NaiveDate) {
        self.retired = Some(Retirement { reason: reason.into(), on });
    }

    /// The stored schema marker, if the node carries one (ODD-0020).
    #[must_use]
    pub fn schema(&self) -> Option<crate::schema::SchemaMarker> {
        self.schema
    }

    /// The node's *effective* schema version: its stored version, or
    /// [`SchemaVersion::LEGACY`](crate::schema::SchemaVersion::LEGACY) (`v0.1`)
    /// when unversioned (the computed legacy default — ODD-0020 §2).
    #[must_use]
    pub fn schema_version(&self) -> crate::schema::SchemaVersion {
        self.schema.map_or(crate::schema::SchemaVersion::LEGACY, |m| m.version)
    }

    /// Stamps the current schema marker for this node's type
    /// (`<type>/`[`SchemaVersion::CURRENT`](crate::schema::SchemaVersion::CURRENT))
    /// — called by every odm create/write path so no node is written
    /// unversioned (ODD-0020 §2). Idempotent.
    pub fn stamp_schema(&mut self) {
        self.schema = Some(crate::schema::SchemaMarker::current(self.node_type));
    }

    /// Sets the schema marker (builder form of [`stamp_schema`](Self::stamp_schema)
    /// for a specific marker, e.g. when reconstructing or testing).
    #[must_use]
    pub fn with_schema(mut self, marker: crate::schema::SchemaMarker) -> Self {
        self.schema = Some(marker);
        self
    }

    /// Records a string-valued key not modeled by the typed schema into the
    /// forward-compat catch-all, so it survives a round-trip and is emitted with
    /// the frontmatter. This is the sanctioned way to carry a value the schema
    /// does not (yet) type — e.g. `odm migrate` carrying a legacy `author`, which
    /// ODD-0013 §2.3 has no typed field for. A later slice that types the key
    /// takes it over from `extra` transparently.
    pub fn insert_extra(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.extra.insert(
            serde_norway::Value::String(key.into()),
            serde_norway::Value::String(value.into()),
        );
    }
}

/// A node's **deferred** marker (ODD-0013 Q-A3-1): parked work with a checkable
/// re-entry condition.
///
/// `reenter_when` references one of the node's own [`desired_facts`] by its
/// node-local `id` — the fact whose *holding* means "the reason to defer is
/// gone; this can resume." Reusing a declared fact (rather than embedding a
/// second probe) keeps one probe model (slice01); reconcile evaluates that fact
/// to decide ready-to-re-enter vs still-deferred. A `reenter_when` that resolves
/// to no such fact is a `check` finding (dangling), never a panic.
///
/// [`desired_facts`]: Frontmatter::desired_facts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deferral {
    /// Why the node was parked (human text).
    pub because: String,
    /// The node-local `id` of the `desired_fact` whose holding means the node is
    /// ready to re-enter.
    pub reenter_when: String,
}

/// A parent's guarded "decomposition complete" assertion (ODD-0013 §4.5):
/// "these children fully account for my scope — no missing, no extra".
///
/// The affirmed child set is recorded so a later add/remove is detectable as
/// drift ([`crate::recompose`]). This is a deliberate enrichment of the bare
/// `decomposed: complete` scalar shown in §2.3, which cannot support the
/// drift-guard (it carries no record of *what* was affirmed). See the slice05
/// report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decomposition {
    /// The date the decomposition was affirmed complete.
    pub on: NaiveDate,
    /// The child ids affirmed against, sorted and de-duplicated. A difference
    /// from the node's current children is drift (re-affirmation needed).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Id>,
}

/// A migrated node's **source record** (ODD-0025 §2.0/§2.2): *where* the
/// node's content came from. A third, deliberately distinct axis alongside
/// [`Origin`] (*why* the node exists) and 0013's `provenance` (*derived*
/// git/supersede/gate lineage, never stored) — `source` is stored because git
/// cannot derive it: after migration, git blame returns the migrate commit,
/// not the original author or path.
///
/// Present only on a migrated node; absent on a hand-created one. Fields are
/// computed once, at migration time, and never re-verified later — no hash is
/// stored (ODD-0025 §2.1: content is allowed to change post-migration, e.g. a
/// Version-History section, so a stored hash would be a false drift signal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    /// The source file path(s) this node's content was migrated from — a
    /// list (1+) so a later synthesis (many sources → one node, ODD-0025
    /// §2.3) fits the same shape without a model change.
    pub paths: Vec<PathBuf>,
    /// The source document's class (the `DocClass` odm-migrate's coverage
    /// detector assigns, e.g. `"arc-plan"`, `"slice-doc"`, `"odd"`) — carried
    /// as a plain string since the enum lives in `odm-migrate`, which depends
    /// on this crate, not the reverse.
    pub class: String,
    /// What the body-hash gate's normalization stripped before comparing
    /// (ODD-0025 §2.1, e.g. `"trim+lf"`), so the comparison stays
    /// interpretable without re-deriving the rule from code.
    pub normalization: String,
    /// The migrating tool + version (e.g. `"odm-migrate/1.0.0"`) — so a
    /// pre-fix import is queryable by tool version.
    pub migrated_by: String,
    /// The date the migration ran.
    pub migrated_on: NaiveDate,
    /// The synthesis regime, if this node's content was **synthesized** —
    /// many sources merged into one (ODD-0025 §2.3) — rather than migrated
    /// 1:1: `"concatenation"`, `"editorial-merge"`, or `"other"`. `None` for
    /// an ordinary migrated node. Carried as a plain string for the same
    /// cross-crate reason as `class` (the enum lives in `odm-migrate`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synthesis: Option<String>,
    /// The recorded attestation for an `editorial-merge` synthesis (ODD-0025
    /// §2.3): not hash-checkable, so verified by supersede lineage *plus*
    /// this explicit, human-authored statement instead — never a silent
    /// pass. `None` for a `concatenation` synthesis (hash-gated) or an
    /// ordinary migrated node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation: Option<String>,
}

/// A node's retirement marker (set by `odm retire`).
///
/// A retired node is withdrawn but its file is preserved — git keeps the
/// history; retirement is never a destructive delete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retirement {
    /// Why the node was retired.
    pub reason: String,
    /// The date the node was retired.
    pub on: NaiveDate,
}

/// A node's outgoing edges (ODD-0013 §3). Reverse edges are derived, never
/// stored, so they do not appear here.
///
/// Fields are emitted in canonical order: `part_of`, `depends_on`,
/// `blocked_by`, `verifies`, `consumes`, `affects`, `supersedes`, `tears`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Edges {
    /// Containment parent (single parent — the hierarchy tree).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_of: Option<Id>,
    /// Ordering dependencies; each is a bare id or an id qualified with the
    /// gate at which it counts as satisfied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<Dependency>,
    /// Hard external blocks (withhold the node from `next`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<Id>,
    /// Nodes this node verifies (traceability).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verifies: Vec<Id>,
    /// Concrete outputs/artifacts this node consumes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumes: Vec<Id>,
    /// Nodes whose docs this node affects (stale-doc-vs-decision check).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affects: Vec<Id>,
    /// Supersession lineage — the nodes this node supersedes (ODD-0025 §2.3: a
    /// synthesis may supersede many sources, so this is a list, not a single
    /// edge). The reverse (`superseded_by`) stays derived, never stored.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supersedes: Vec<Supersedes>,
    /// Dependency edges deliberately assumed/broken to cut a cycle, each with
    /// its required rationale (ODD-0013 §4.3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tears: Vec<TornEdge>,
}

impl Edges {
    /// Returns `true` if there are no edges of any kind (used to omit an empty
    /// `edges:` block on emit).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.part_of.is_none()
            && self.depends_on.is_empty()
            && self.blocked_by.is_empty()
            && self.verifies.is_empty()
            && self.consumes.is_empty()
            && self.affects.is_empty()
            && self.supersedes.is_empty()
            && self.tears.is_empty()
    }
}

/// A dependency edge target: either a bare id (satisfied at the target's
/// terminal gate) or an id qualified with the gate at which it is satisfied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// A bare target id.
    Bare(Id),
    /// A target id qualified with a satisfaction gate.
    Qualified {
        /// The target node.
        node: Id,
        /// The gate at which the dependency counts as satisfied (e.g.
        /// `"tested"`). Gate semantics arrive in Arc 02; the name is kept as a
        /// string here so it round-trips before then.
        satisfied_at: String,
    },
}

/// A deliberately-assumed ("torn") dependency edge recorded in `tears:` on the
/// source node, with the rationale that justifies assuming it (ODD-0013 §4.3).
///
/// This is the *persisted* frontmatter entry, deliberately named distinctly
/// from [`odm_graph::Tear`] (the engine's pure cycle-breaking primitive over
/// abstract ids). Graph-build maps each `TornEdge` → a `Tear<Id>` carrying this
/// `because` text, so the rationale flows from disk into the cycle detector and
/// `check`'s active-tears listing — it is no longer dropped after validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TornEdge {
    /// The torn dependency edge (bare id or gate-qualified), mirroring the
    /// `depends_on` entry it assumes.
    pub edge: Dependency,
    /// Why this dependency was deliberately assumed — the tear's audit
    /// rationale. Required (the `tear` command rejects an empty one via
    /// [`odm_graph::Tear::new`]).
    pub because: String,
}

/// A supersession edge: this node supersedes `node` with a given `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Supersedes {
    /// The superseded node.
    pub node: Id,
    /// Whether the old node is replaced or merely amended.
    pub kind: SupersedeKind,
}

/// The kind of a [`Supersedes`] edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupersedeKind {
    /// The old node is replaced.
    Obsoletes,
    /// The old node is amended (still relevant).
    Updates,
}

/// An error parsing or emitting frontmatter.
///
/// Self-contained: it does not expose the underlying YAML library type.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FrontmatterError {
    /// The text did not begin with a `---` delimiter line.
    #[error("missing opening '---' frontmatter delimiter")]
    MissingOpen,
    /// No closing `---` delimiter line was found.
    #[error("unterminated frontmatter: missing closing '---'")]
    Unterminated,
    /// The frontmatter block was not valid YAML, or did not match the schema.
    /// The message includes the position the YAML backend reported, when it
    /// reports one.
    #[error("invalid frontmatter YAML: {0}")]
    Yaml(String),
}

impl FrontmatterError {
    /// Converts a YAML-backend error into a self-contained message, keeping its
    /// position text (the backend includes line/column in its `Display`).
    fn from_yaml(error: serde_norway::Error) -> Self {
        FrontmatterError::Yaml(error.to_string())
    }
}
