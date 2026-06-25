//! The on-disk node format: a `---`-delimited YAML frontmatter block followed
//! by a markdown body.
//!
//! [`Document::parse`] splits and deserializes a node file; [`Document::emit`]
//! serializes it back in the canonical field order of ODD-0013 §2.3. The
//! headline invariant is **`parse ∘ emit == identity`**: emitting a document
//! and parsing the result yields an equal document, including any keys this
//! slice does not yet model (`status`, `desired_facts`, …), which are
//! preserved verbatim.
//!
//! # YAML library isolation
//!
//! The YAML backend (`serde_norway`) is used **only** inside this module — no
//! YAML-crate type appears in the public API, so the backend can be swapped
//! without touching callers. (Same insurance applied to `ulid` in slice 02.)

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_norway::Mapping;

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

/// The typed frontmatter schema (ODD-0013 §2.3).
///
/// Fields are declared — and therefore emitted — in canonical order: `id`,
/// `number`, `type`, `name`, `created`, `updated`, `tags`, `component`,
/// `origin`, `reserved`, `retired`, `edges`, `status`, `decomposed`. Any keys
/// not modeled here (e.g. `desired_facts`) are captured in a hidden catch-all
/// and re-emitted last, so they survive a round-trip until their owning slices
/// model them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Stable ULID identity.
    id: Id,
    /// Human-facing number (metadata, not identity).
    number: u32,
    #[serde(rename = "type")]
    node_type: NodeType,
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
    /// How the node arose.
    origin: Origin,
    /// Tentative future-work placeholder flag.
    #[serde(default)]
    reserved: bool,
    /// Retirement marker, set by `odm retire`. Absent unless the node has been
    /// withdrawn. (Not in the ODD-0013 §2.3 example yet — see slice05 report.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    retired: Option<Retirement>,
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
    /// Keys not yet modeled, preserved verbatim across a round-trip (forward
    /// compatibility for `desired_facts`, …).
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
            name: name.into(),
            created,
            updated,
            tags: Vec::new(),
            component: None,
            origin,
            reserved: false,
            retired: None,
            edges: Edges::default(),
            status: crate::status::Status::new(),
            decomposed: None,
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

    /// The number of preserved-but-unmodeled top-level keys (e.g.
    /// `desired_facts`).
    #[must_use]
    pub fn unknown_key_count(&self) -> usize {
        self.extra.len()
    }

    /// Changes the human label. Does not touch `id` or the on-disk path.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
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
    /// Supersession lineage, if this node supersedes another.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<Supersedes>,
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
            && self.supersedes.is_none()
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
