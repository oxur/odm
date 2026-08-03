//! Command implementations over the [`Store`].
//!
//! Output is dependency-injected: query **results (data) are written to `out`**
//! while mutation confirmations and dry-run notices (**diagnostics**) are
//! written to `err`. `run` wires these to stdout/stderr; tests wire them to
//! buffers and drive commands in-process.
//!
//! Rendering is Oxur's: tables go through [`oxur_term::table::OxurTable`] (the
//! warm-orange theme) and status lines through [`crate::term`]. Both degrade to
//! plain text off a terminal, so assertions stay stable — `colored` suppresses
//! ANSI when stdout is not a TTY.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, anyhow, bail};
use chrono::NaiveDate;
use odm_core::check::{Finding, Violation};
use odm_core::frontmatter::{
    Dependency, Document, Frontmatter, Source, SupersedeKind, Supersedes, TornEdge,
};
use odm_core::gates::{GateSet, GateSets};
use odm_core::graph::{Block, NodeGraph, Tear};
use odm_core::recompose::{self, Issue};
use odm_core::satisfaction::{Satisfaction, staleness_on_advance, threshold_from_toml};
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use odm_store::{Store, StoreHome};
use serde::Serialize;

use crate::context::Context;
use crate::listview;
use crate::table::Themed;
use crate::term;

/// Exit code: the command succeeded (and, for `check`, the corpus is clean).
pub const EXIT_OK: u8 = 0;
/// Exit code: `check` ran and found violations.
pub const EXIT_VIOLATIONS: u8 = 1;

/// Which context slot `use` sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseKind {
    /// The current project.
    Project,
    /// The current arc.
    Arc,
}

impl UseKind {
    fn node_type(self) -> NodeType {
        match self {
            UseKind::Project => NodeType::Project,
            UseKind::Arc => NodeType::Arc,
        }
    }

    fn label(self) -> &'static str {
        match self {
            UseKind::Project => "project",
            UseKind::Arc => "arc",
        }
    }
}

/// Today's date (UTC) — used to stamp `updated` and retirement.
fn today() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

// ---------------------------------------------------------------------------
// Reference resolution: id | number | unique name prefix
// ---------------------------------------------------------------------------

/// Resolves a user-supplied reference to a node, accepting a full ULID, a human
/// number, or a unique (case-insensitive) name prefix.
///
/// # Errors
///
/// Errors if no node matches, or if a name prefix is ambiguous.
fn resolve(store: &Store, reference: &str) -> anyhow::Result<Document> {
    // 1. A full ULID id.
    if let Ok(id) = reference.parse::<Id>() {
        return store
            .load(id)
            .with_context(|| format!("no node with id {reference} (try `odm list`)"));
    }

    let all = store.load_all()?;

    // 2. A human number.
    if let Ok(number) = reference.parse::<u32>() {
        let mut hits = all.into_iter().filter(|d| d.frontmatter().number() == number);
        return match hits.next() {
            Some(doc) => Ok(doc),
            None => Err(anyhow!("no node with number {number} (try `odm list`)")),
        };
    }

    // 3. A unique name prefix (case-insensitive).
    let needle = reference.to_lowercase();
    let mut matches: Vec<Document> = all
        .into_iter()
        .filter(|d| d.frontmatter().name().to_lowercase().starts_with(&needle))
        .collect();
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(anyhow!("no node matching {reference:?} (try `odm list`)")),
        n => {
            let names: Vec<String> = matches
                .iter()
                .map(|d| format!("#{} {}", d.frontmatter().number(), d.frontmatter().name()))
                .collect();
            Err(anyhow!("{reference:?} is ambiguous ({n} matches): {}", names.join(", ")))
        }
    }
}

// ---------------------------------------------------------------------------
// JSON views — stable, documented schemas for `--json`
// ---------------------------------------------------------------------------

/// JSON shape of a node (stable schema for `list`/`show --json`).
#[derive(Serialize)]
struct NodeJson {
    id: String,
    number: u32,
    #[serde(rename = "type")]
    node_type: String,
    name: String,
    origin: String,
    reserved: bool,
    tags: Vec<String>,
    component: Option<String>,
    retired: Option<RetiredJson>,
    part_of: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    supersedes: Vec<SupersedesJson>,
    /// The normalized, cross-type-comparable state (F-19) — the same token
    /// `node list` shows. `null` when the caller had no gate config to derive
    /// it from, since a guess would be worse than an absence.
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    /// The **raw** gate ladder: every reached gate with its evidence, in the
    /// order the node's own gate-set defines. The derived `status` above is a
    /// projection of this; both are emitted so a machine consumer can compare
    /// across types *and* still see which rung.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    gates: Vec<GateJson>,
    /// Deliberately-assumed dependencies, each with the reason it was assumed
    /// (G-2). Omitted when the node has torn nothing.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tears: Vec<TearEdgeJson>,
}

/// One assumed (torn) dependency, as `--json` reports it.
#[derive(Serialize)]
struct TearEdgeJson {
    depends_on: String,
    because: String,
}

/// One reached gate, as `--json` reports it.
#[derive(Serialize)]
struct GateJson {
    gate: String,
    reached: String,
    evidence: String,
}

#[derive(Serialize)]
struct RetiredJson {
    reason: String,
    on: String,
}

#[derive(Serialize)]
struct SupersedesJson {
    node: String,
    kind: String,
}

impl NodeJson {
    /// The node without a derived state — for callers with no gate config in
    /// hand. The raw ladder is still emitted; only the projection is omitted.
    fn from(doc: &Document) -> Self {
        Self::with_gates(doc, None)
    }

    /// The node including its normalized state, derived against `gates`.
    fn with_gates(doc: &Document, gates: Option<&GateSets>) -> Self {
        let mut json = Self::build(doc);
        let fm = doc.frontmatter();
        let reached: Vec<&str> = fm.status().reached().map(|(name, _)| name).collect();
        if let Some(gates) = gates {
            let sequence = gates.for_type(fm.node_type()).map(GateSet::sequence);
            json.status = crate::listview::derive_display_status(
                &reached,
                sequence,
                fm.retired().is_some(),
                false,
            )
            .name()
            .map(str::to_string);
            // Ladder order, not map order, so the rungs read in sequence.
            if let Some(sequence) = sequence {
                json.gates = sequence
                    .iter()
                    .filter_map(|gate| fm.status().gate(gate).map(|r| (gate, r)))
                    .map(|(gate, record)| GateJson {
                        gate: gate.clone(),
                        reached: record.reached.to_string(),
                        evidence: record.evidence.as_str().to_string(),
                    })
                    .collect();
            }
        }
        json
    }

    fn build(doc: &Document) -> Self {
        let fm = doc.frontmatter();
        let edges = fm.edges();
        Self {
            id: fm.id().to_string(),
            number: fm.number(),
            node_type: fm.node_type().as_str().to_string(),
            name: fm.name().to_string(),
            origin: fm.origin().as_str().to_string(),
            reserved: fm.reserved(),
            tags: fm.tags().to_vec(),
            component: fm.component().map(str::to_string),
            retired: fm
                .retired()
                .map(|r| RetiredJson { reason: r.reason.clone(), on: r.on.to_string() }),
            part_of: edges.part_of.map(|id| id.to_string()),
            supersedes: edges
                .supersedes
                .iter()
                .map(|s| SupersedesJson {
                    node: s.node.to_string(),
                    kind: supersede_kind_str(s.kind).to_string(),
                })
                .collect(),
            status: None,
            gates: Vec::new(),
            tears: edges
                .tears
                .iter()
                .map(|tear| TearEdgeJson {
                    depends_on: dependency_label(&tear.edge),
                    because: tear.because.clone(),
                })
                .collect(),
        }
    }
}

fn supersede_kind_str(kind: SupersedeKind) -> &'static str {
    match kind {
        SupersedeKind::Obsoletes => "obsoletes",
        SupersedeKind::Updates => "updates",
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Where a `node new`/`node set-body` body comes from — resolved by the
/// caller (the CLI dispatch, which of `--content`/`--from-file`/`--body` was
/// given) so this layer only ever reads one already-disambiguated source
/// (arc-store-as-source slice03).
pub enum BodySource<'a> {
    /// `--content`/`--from-file` — read verbatim from a markdown file.
    File(&'a Path),
    /// `--body` — the literal string, verbatim.
    Inline(&'a str),
}

impl BodySource<'_> {
    fn read(&self) -> anyhow::Result<String> {
        match self {
            BodySource::File(path) => std::fs::read_to_string(path)
                .with_context(|| format!("reading body content {}", path.display())),
            BodySource::Inline(s) => Ok((*s).to_string()),
        }
    }
}

/// Options for [`new`] beyond `type`/`name` (arc-store-as-source slice03) —
/// grouped since the flag set outgrew a readable argument list.
pub struct NewOptions<'a> {
    /// `--parent` — takes precedence over the metadata partial's `part_of`.
    pub parent: Option<&'a str>,
    /// `--metadata=<file>` — the author-owned partial ([`crate::metadata`]).
    pub metadata: Option<&'a Path>,
    /// `--content`/`--from-file`/`--body`, already disambiguated. `None` for
    /// a bare `node new` — mints an authored stub.
    pub body: Option<BodySource<'a>>,
    /// Show what would happen without writing.
    pub dry_run: bool,
    /// Emit JSON instead of a human confirmation line.
    pub json: bool,
}

/// `new <type> <name> [--parent <ref>] [--metadata=<file>] [--content=<md>]`
/// — idempotent describe-or-create. Confirmations go to `err` (diagnostics);
/// `--json` writes the created (or, under `--dry-run`, would-be-created) node
/// to `out` instead.
///
/// Mints `origin: authored` + `source: { class: authored }` (arc-store-as-
/// source slice02's model — every node this command creates is store-owned
/// from birth, never pulled from an external file) with odm owning
/// `id`/`number`/placement/`source`; the metadata partial and body are the
/// only author-owned inputs (ODD-0026 §2.5, ODD-0013 v2.6). A bare
/// `node new` (no `--metadata`/body flags) mints an authored stub — the
/// skeleton-then-grow path, filled in later via `node set`/`node set-body`.
///
/// # Errors
///
/// Returns an error (before any write) if the type is unknown, the metadata
/// partial fails to load/validate (including an odm-owned field — rejected
/// by [`crate::metadata::MetadataPartial`]'s shape), `--parent`/`part_of`
/// doesn't resolve, a status intent names a gate not in the type's gate-set,
/// or the body content can't be read.
pub fn new(
    store: &Store,
    root: &Path,
    node_type: &str,
    name: &str,
    options: NewOptions<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let NewOptions { parent, metadata, body, dry_run, json } = options;

    let node_type: NodeType = node_type.parse().map_err(|_| {
        anyhow!(
            "unknown type {node_type:?}; expected one of \
             project|arc|slice|design|research|adr|note|artifact"
        )
    })?;

    // Load + validate the metadata partial up front — before the
    // idempotency check below, so a malformed call errors the same way
    // whether or not the node already exists (F-2: rejected before any
    // write). An odm-owned field (`id`/`source`/…) is rejected by the
    // partial's own shape (`deny_unknown_fields`) inside `metadata::load`.
    let partial = metadata.map(crate::metadata::load).transpose()?;

    // `--parent` wins over the partial's `part_of` when both are given.
    let part_of_ref: Option<&str> =
        parent.or_else(|| partial.as_ref().and_then(|p| p.part_of.as_deref()));
    let parent_id =
        part_of_ref.map(|p| resolve(store, p)).transpose()?.map(|d| d.frontmatter().id());

    // A status intent needs a gate-set to validate against — only required
    // when one is actually given, so a bare/tags-only/part_of-only create
    // never needs gate config in hand at all.
    let status_gate_set = if let Some(status) = partial.as_ref().and_then(|p| p.status.as_deref()) {
        let (gates, _) = load_gate_config(root)?;
        let gate_set = gates
            .for_type(node_type)
            .ok_or_else(|| {
                anyhow!(
                    "no gate-set for type `{}`; add a `[gates.{}]` sequence to odm.toml",
                    node_type.as_str(),
                    node_type.as_str()
                )
            })?
            .clone();
        if !gate_set.contains(status) {
            bail!(
                "unknown gate {status:?} for type `{}`; allowed: {}",
                node_type.as_str(),
                gate_set.sequence().join(", ")
            );
        }
        Some(gate_set)
    } else {
        None
    };

    // Read the body up front too — fail before any write, same discipline.
    let body_text = body.as_ref().map(BodySource::read).transpose()?;

    let all = store.load_all()?;

    // Idempotent: a node of the same type and exact name already exists.
    if let Some(existing) = all
        .iter()
        .find(|d| d.frontmatter().node_type() == node_type && d.frontmatter().name() == name)
    {
        // A **warning**, not info (RH F-10): the caller asked to create
        // something and nothing was created, which is a mismatch worth
        // noticing even though it is not an error. And it stays a one-liner —
        // dumping the node's details here answers a question nobody asked, so
        // it points at the command that does answer it instead.
        let fm = existing.frontmatter();
        term::warning(
            err,
            &format!(
                "{} exists: #{} {:?} — for details run `odm project --name={:?}`",
                node_type.as_str(),
                fm.number(),
                name,
                name
            ),
        )?;
        return Ok(());
    }

    let next_number = all.iter().map(|d| d.frontmatter().number()).max().map_or(1, |m| m + 1);
    let id = Id::new();
    let created = id.created_at().date_naive();
    let mut fm =
        Frontmatter::new(id, next_number, node_type, name, created, created, Origin::Authored);
    // Every odm-created node is stamped the current schema (`<type>/v1.0`, ODD-0020
    // V-2) — no new node is ever written unversioned.
    fm.stamp_schema();
    if let Some(parent_id) = parent_id {
        fm.edges_mut().part_of = Some(parent_id);
    }
    if let Some(partial) = &partial {
        if !partial.tags.is_empty() {
            fm = fm.with_tags(partial.tags.clone());
        }
    }
    // Authored, not migrated (ODD-0026 §2.1) — no external paths, ever.
    fm = fm.with_source(Source::authored(Vec::new()));
    if let Some(status) = partial.as_ref().and_then(|p| p.status.as_deref()) {
        let gate_set = status_gate_set.as_ref().expect("gate-set loaded above when status is set");
        fm.status_mut()
            .set_gate(gate_set, status, None, Evidence::Asserted, created)
            .expect("gate validated up front");
    }

    let body_content = body_text.unwrap_or_else(|| format!("# {name}\n"));
    let doc = Document::new(fm, body_content);

    let parent_note = parent_id.map(|p| format!(" (part_of {p})")).unwrap_or_default();
    if dry_run {
        if json {
            writeln!(out, "{}", serde_json::to_string_pretty(&NodeJson::from(&doc))?)?;
            return Ok(());
        }
        term::info(
            err,
            &format!(
                "would create {} #{next_number} {name:?} ({id}){parent_note}",
                node_type.as_str()
            ),
        )?;
        return Ok(());
    }

    store.persist(&doc)?;
    if json {
        let gates = load_gate_config(root).ok().map(|(g, _)| g);
        writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(&NodeJson::with_gates(&doc, gates.as_ref()))?
        )?;
        return Ok(());
    }
    term::success(
        err,
        &format!("created {} #{next_number} {name:?} ({id}){parent_note}", node_type.as_str()),
    )?;
    Ok(())
}

/// The author-owned fields `node set` may address — the same set the
/// metadata partial carries (`part_of`/`tags`/`status`), plus `name`, which
/// `node set` addresses directly rather than through `node rename` (kept as
/// its own dedicated command for the common case; both end up setting the
/// same field). Any other field name — including every odm-owned one
/// (`id`/`number`/`path`/`source`) — is rejected before any write.
const SETTABLE_FIELDS: [&str; 4] = ["name", "tags", "part_of", "status"];

/// Arguments for [`set`] beyond `store`/`root` — grouped to stay under
/// clippy's argument-count lint (arc-store-as-source slice03).
pub struct SetArgs<'a> {
    /// The node (id, number, or unique name prefix).
    pub reference: &'a str,
    /// The field to set — must be one of [`SETTABLE_FIELDS`].
    pub field: &'a str,
    /// The new value.
    pub value: &'a str,
    /// Show what would happen without writing.
    pub dry_run: bool,
    /// Emit JSON instead of a human confirmation line.
    pub json: bool,
}

/// `set <ref> <field> <value>` — field-addressed author-owned metadata edit
/// (arc-store-as-source slice03, ODD-0026 §2.5). The `---` block is never in
/// the editable surface: `field` must be one of [`SETTABLE_FIELDS`], and the
/// **body is left byte-identical** (the seam invariant — only the named
/// field changes, verified by the fixtures at `odm-cli/tests/cli.rs`).
///
/// `field`:
/// - `name` — the node's label (mirrors `node rename`).
/// - `tags` — comma-separated; replaces the tag list wholesale.
/// - `part_of` — a reference (id/number/name prefix); re-resolved and
///   re-validated exactly like `node link … part_of` / `node new --parent`.
/// - `status` — a gate name, recorded at `asserted` evidence via the same
///   mechanism `node set-gate` uses (which remains the way to record a
///   stronger evidence level, or to pass `--by`).
///
/// # Errors
///
/// Returns an error (before any write) if `reference` doesn't resolve,
/// `field` is not one of [`SETTABLE_FIELDS`] (this is the author-vs-odm
/// boundary enforcement — F-2), `part_of`'s value doesn't resolve, or
/// `status`'s value isn't a gate in the node type's gate-set.
pub fn set(
    store: &Store,
    root: &Path,
    args: SetArgs<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let SetArgs { reference, field, value, dry_run, json } = args;
    if !SETTABLE_FIELDS.contains(&field) {
        bail!(
            "field {field:?} is not author-settable; allowed: {} (odm owns id/number/path/source)",
            SETTABLE_FIELDS.join(", ")
        );
    }

    let mut doc = resolve(store, reference)?;
    let (number, name) = (doc.frontmatter().number(), doc.frontmatter().name().to_string());

    // Resolve/validate up front (before any write), same discipline as `new`.
    let resolved_part_of =
        if field == "part_of" { Some(resolve(store, value)?.frontmatter().id()) } else { None };
    let status_gate_set = if field == "status" {
        let (gates, _) = load_gate_config(root)?;
        let node_type = doc.frontmatter().node_type();
        let gate_set = gates
            .for_type(node_type)
            .ok_or_else(|| {
                anyhow!(
                    "no gate-set for type `{}`; add a `[gates.{}]` sequence to odm.toml",
                    node_type.as_str(),
                    node_type.as_str()
                )
            })?
            .clone();
        if !gate_set.contains(value) {
            bail!(
                "unknown gate {value:?} for type `{}`; allowed: {}",
                node_type.as_str(),
                gate_set.sequence().join(", ")
            );
        }
        Some(gate_set)
    } else {
        None
    };

    if dry_run {
        if json {
            writeln!(out, "{}", serde_json::to_string_pretty(&NodeJson::from(&doc))?)?;
            return Ok(());
        }
        term::info(err, &format!("would set {field}={value:?} on #{number} {name:?}"))?;
        return Ok(());
    }

    match field {
        "name" => doc.frontmatter_mut().set_name(value),
        "tags" => {
            let tags: Vec<String> = value
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            doc = Document::new(doc.frontmatter().clone().with_tags(tags), doc.body().to_string());
        }
        "part_of" => {
            doc.frontmatter_mut().edges_mut().part_of =
                Some(resolved_part_of.expect("resolved above"));
        }
        "status" => {
            let gate_set = status_gate_set.as_ref().expect("loaded above");
            doc.frontmatter_mut()
                .status_mut()
                .set_gate(gate_set, value, None, Evidence::Asserted, today())
                .expect("gate validated up front");
        }
        _ => unreachable!("field validated against SETTABLE_FIELDS above"),
    }
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?;

    if json {
        let gates = load_gate_config(root).ok().map(|(g, _)| g);
        writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(&NodeJson::with_gates(&doc, gates.as_ref()))?
        )?;
        return Ok(());
    }
    term::success(err, &format!("set {field}={value:?} on #{number} {name:?}"))?;
    Ok(())
}

/// `set-body <ref> --from-file=<md>`/`--body=<str>` — replaces the whole body
/// as pure markdown; odm re-attaches the **unchanged** frontmatter on write
/// (arc-store-as-source slice03, ODD-0026 §2.5's seam invariant: only the
/// body changes, verified by the fixtures at `odm-cli/tests/cli.rs`). The
/// body is prose — never parsed, never a rejection source.
///
/// # Errors
///
/// Returns an error if `reference` doesn't resolve, or the body content
/// can't be read (a file source).
pub fn set_body(
    store: &Store,
    reference: &str,
    body: BodySource<'_>,
    dry_run: bool,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let new_body = body.read()?;
    let doc = resolve(store, reference)?;
    let (number, name) = (doc.frontmatter().number(), doc.frontmatter().name().to_string());

    if dry_run {
        let preview = Document::new(doc.frontmatter().clone(), new_body.clone());
        if json {
            writeln!(out, "{}", serde_json::to_string_pretty(&NodeJson::from(&preview))?)?;
            return Ok(());
        }
        term::info(err, &format!("would set-body on #{number} {name:?}"))?;
        return Ok(());
    }

    let mut new_fm = doc.frontmatter().clone();
    new_fm.set_updated(today());
    let new_doc = Document::new(new_fm, new_body);
    store.persist(&new_doc)?;

    if json {
        writeln!(out, "{}", serde_json::to_string_pretty(&NodeJson::from(&new_doc))?)?;
        return Ok(());
    }
    term::success(err, &format!("set-body on #{number} {name:?}"))?;
    Ok(())
}

/// The `list` table's columns (RH C-3 / F-4: no NUMBER).
const LIST_COLUMNS: [&str; 5] = ["DATE", "TYPE", "STATUS", "NAME", "ID"];

/// The `DATE` column's index.
const DATE_COLUMN: usize = 0;
/// The `TYPE` column's index, for the per-type colouring.
const TYPE_COLUMN: usize = 1;
/// The `STATUS` column's index, for the per-state colouring.
const STATUS_COLUMN: usize = 2;
/// The `ID` column's index.
const ID_COLUMN: usize = 4;

/// The NAME column's default width bound when neither `--width` nor
/// `[display] max_width` says otherwise (F-9).
const DEFAULT_NAME_WIDTH: usize = 64;

/// How `list` should render (RH C-3). Grouped into one struct because the flag
/// set outgrew a readable argument list.
pub struct ListView<'a> {
    /// Filter by node type.
    pub type_filter: Option<&'a str>,
    /// Filter by tag.
    pub tag: Option<&'a str>,
    /// Filter by component.
    pub component: Option<&'a str>,
    /// Which date the leading column shows.
    pub(crate) date: listview::DateColumn,
    /// The NAME column's width bound; `None` defers to config, then the default.
    pub width: Option<usize>,
    /// Show only nodes whose STATUS is this value.
    pub status: Option<&'a str>,
    /// Show only one family of nodes.
    pub(crate) group: Option<listview::Group>,
    /// Whether to include retired/superseded nodes (F-15).
    pub include_withdrawn: bool,
    /// Emit JSON instead of the table.
    pub json: bool,
}

/// `list` — the plan view: date, type, status, containment tree. Data → `out`.
///
/// The human table is **index-backed** (slice04): it `reconcile`s the `.odm/`
/// index (freshening it against any edit) and renders from the index records —
/// no full corpus parse. The C-3 columns are all served from the index
/// (`created` and `retired` were added to the record for exactly this).
///
/// `--json` stays a full-node dump over `load_all`: it emits fields the index
/// deliberately does not carry, since the index is the filter/sort accelerator,
/// not a full-node store (ODD-0014 §3.5). **JSON is unfiltered by `--all`** —
/// a machine consumer gets every node and its `retired` field, and decides for
/// itself; the default-hiding is a *human-view* affordance (F-15).
///
/// # Errors
///
/// Returns an error if the type filter names an unknown type, or if the store /
/// index cannot be read.
pub fn list(
    store: &Store,
    root: &Path,
    view: ListView<'_>,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let type_filter = view
        .type_filter
        .map(|t| t.parse::<NodeType>().map_err(|_| anyhow!("unknown type {t:?}")))
        .transpose()?;

    if view.json {
        // The derived state needs the ladder, so `--json` loads the gate config
        // the human path already loads below.
        let (gates, _threshold) = load_gate_config(root)?;
        // Full-node serialization stays load_all-backed (see the doc comment).
        let mut nodes = store.load_all()?;
        nodes.retain(|d| {
            let fm = d.frontmatter();
            type_filter.is_none_or(|t| fm.node_type() == t)
                && view.tag.is_none_or(|t| fm.tags().iter().any(|x| x == t))
                && view.component.is_none_or(|c| fm.component() == Some(c))
        });
        nodes.sort_by_key(|d| d.frontmatter().number());
        let json_view: Vec<NodeJson> =
            nodes.iter().map(|d| NodeJson::with_gates(d, Some(&gates))).collect();
        writeln!(out, "{}", serde_json::to_string_pretty(&json_view)?)?;
        return Ok(());
    }

    // Human table: reconcile-then-read the index (slice03 finding #2 / I-9).
    let index = odm_index::default_index_path(store.root());
    let snapshot = odm_index::reconcile(store, &index)?.snapshot;
    let records: Vec<&odm_index::IndexRecord> = snapshot
        .records
        .iter()
        .filter(|r| {
            type_filter.is_none_or(|t| r.node_type == t)
                && view.tag.is_none_or(|t| r.tags.iter().any(|x| x == t))
                && view.component.is_none_or(|c| r.component.as_deref() == Some(c))
        })
        .collect();

    let (gates, _) = load_gate_config(root)?;
    let rows = listview::build_rows(
        &records,
        &snapshot.records,
        &gates,
        view.date,
        view.include_withdrawn,
        view.status,
        view.group,
    );

    if rows.is_empty() {
        match view.status {
            Some(status) => writeln!(out, "(no nodes with status {status:?})")?,
            None => writeln!(out, "(no nodes)")?,
        }
        return Ok(());
    }

    let width = view.width.or_else(|| display_max_width(root)).unwrap_or(DEFAULT_NAME_WIDTH);
    let mut table = Themed::new("NODES", &LIST_COLUMNS);
    // The date and the id are context, not the answer — muted so the type,
    // status and name between them carry the eye.
    table.mute_columns(&[DATE_COLUMN, ID_COLUMN]);
    let mut shown = 0usize;
    for row in &rows {
        match row {
            listview::Row::Node(node) => {
                shown += 1;
                table.row([
                    node.date.to_string(),
                    node.node_type.as_str().to_string(),
                    node.status.label().to_string(),
                    listview::elide(&node.name, width),
                    node.id.to_string(),
                ]);
                if node.status.is_withdrawn() {
                    // Present, but not live work (F-15). Dimming the whole row
                    // is the stronger signal, so it wins over the status colour
                    // below rather than fighting it cell by cell.
                    table.dim_last();
                } else {
                    // Colour only, no weight — exactly as 0.3.5 rendered the
                    // STATUS column. See `listview::status_color` for which
                    // palette a given gate comes from, and `type_color` for the
                    // per-type hues.
                    if let Some(fg) = listview::type_color(node.node_type, node.promoted) {
                        table.color_last(TYPE_COLUMN, fg);
                    }
                    if let Some(fg) = listview::status_color(node.status.label()) {
                        table.color_last(STATUS_COLUMN, fg);
                    }
                }
            }
            listview::Row::Divider => table.divider(),
        }
    }
    // The count is of what is *shown*; say so when rows were withheld, so a
    // reader is never left to infer that the corpus is smaller than it is.
    let hidden = snapshot.records.len() - shown;
    table.summary(if view.include_withdrawn || hidden == 0 {
        format!("Total: {shown} node(s)")
    } else {
        format!(
            "Total: {shown} node(s) shown — {hidden} filtered or withdrawn (--all shows every node)"
        )
    });
    writeln!(out, "{}", table.render())?;
    Ok(())
}

/// The `[display] max_width` setting from the operational config, if
/// configured (F-9).
///
/// Read from the raw text like the gate-sets are, rather than through
/// `StoreConfig`: that struct is the author identity, and a display preference
/// does not belong in it.
fn display_max_width(root: &Path) -> Option<usize> {
    let text = StoreHome::resolve(root).operational_text();
    let value: toml::Value = text.parse().ok()?;
    value.get("display")?.get("max_width")?.as_integer().and_then(|n| usize::try_from(n).ok())
}

/// The `[coverage] scan_root` setting from the operational config
/// (arc-migration-fidelity s09 F-5): the docs tree `check`'s doc-coverage
/// rule scans, resolved relative to `root` if given as a relative path.
///
/// Read from the raw text like [`display_max_width`], not through
/// `StoreConfig` — same rationale, a scan-root setting is not author
/// identity. **Absent by default, and that absence is deliberate**: the key
/// does not exist in the live store's config today, so this rule stays a
/// no-op there until a follow-on live-run slice adds it — turning the rule on
/// against the real corpus is gated behind the snapshot → dry-run → fire
/// protocol (pre-mint, the live corpus has ~211 uncovered docs by
/// construction, which would make `check` red the moment this key appears).
fn coverage_scan_root(root: &Path) -> Option<PathBuf> {
    let text = StoreHome::resolve(root).operational_text();
    let value: toml::Value = text.parse().ok()?;
    let configured = value.get("coverage")?.get("scan_root")?.as_str()?;
    let path = Path::new(configured);
    Some(if path.is_absolute() { path.to_path_buf() } else { root.join(path) })
}

/// Resolves a directory setting from the operational config text: the
/// top-level key first, falling back to the same key under `[legacy]` (the
/// pre-split odm-0.3.x settings `odm store init` ports forward when it finds
/// them, arc-migration-fidelity s13) — so a store that has only ever seen the
/// legacy value still resolves correctly, without requiring the modern key
/// to be set explicitly.
fn configured_directory(root: &Path, key: &str) -> Option<PathBuf> {
    let text = StoreHome::resolve(root).operational_text();
    let value: toml::Value = text.parse().ok()?;
    let configured = value
        .get(key)
        .and_then(|v| v.as_str())
        .or_else(|| value.get("legacy")?.get(key)?.as_str())?;
    let path = Path::new(configured);
    Some(if path.is_absolute() { path.to_path_buf() } else { root.join(path) })
}

/// The `docs_directory` setting from the operational config
/// (arc-migration-fidelity s13, `migrate --all`): the design/research root
/// the default (non-plan) `migrate` derivation reconciles, resolved so
/// `--all` doesn't require the operator to type it out.
///
/// Read from the raw text like [`display_max_width`]/[`coverage_scan_root`]
/// — a source-tree location is not author identity, so it does not belong
/// in `StoreConfig`.
pub(crate) fn configured_docs_directory(root: &Path) -> Option<PathBuf> {
    configured_directory(root, "docs_directory")
}

/// The `dev_directory` setting from the operational config — `migrate --all`'s
/// default root for the `--notes` mint-all pass.
pub(crate) fn configured_dev_directory(root: &Path) -> Option<PathBuf> {
    configured_directory(root, "dev_directory")
}

/// The design/research corpus root: `docs_directory` **joined with
/// `"design"`** (arc-migration-fidelity s14 F-2) — the legacy (v0.3.5−)
/// semantic `migrate --all` dropped by using `docs_directory` as-is. It
/// happened to still work while the live config set `docs_directory =
/// "./docs/design"` directly; once the config carries the canonical,
/// wider `docs_directory = "./docs"` (the umbrella `--artifacts`/self-host
/// sweep root, F-3), an as-is read would reconcile design/research over the
/// *whole* docs tree — applying the NN-state + §2.4 frontmatter rules to
/// files (a plan tree, dev docs) that must never get them. The append keeps
/// those rules scoped to the one subtree that carries them.
pub(crate) fn configured_design_directory(root: &Path) -> Option<PathBuf> {
    configured_docs_directory(root).map(|d| d.join("design"))
}

/// The persisted extra legacy directories (arc-migration-fidelity s14 F-5):
/// `[legacy].additional_paths`, each resolved relative to `root` if not
/// already absolute. `additional_paths` has no legacy top-level form (it is
/// a new concept `migrate --all` introduces), so — unlike
/// [`configured_directory`] — it is read **only** from `[legacy]`.
///
/// Absent, unparsable, or non-array is the same as "none configured": an
/// empty `Vec`, never an error — a malformed or missing config is a reason
/// to fall back to "nothing extra," not to fail every other derivation
/// `migrate --all` would otherwise still run cleanly.
pub(crate) fn configured_additional_paths(root: &Path) -> Vec<PathBuf> {
    let text = StoreHome::resolve(root).operational_text();
    let Ok(value) = text.parse::<toml::Value>() else {
        return Vec::new();
    };
    let Some(array) =
        value.get("legacy").and_then(|l| l.get("additional_paths")).and_then(|v| v.as_array())
    else {
        return Vec::new();
    };
    array
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| {
            let path = Path::new(s);
            if path.is_absolute() { path.to_path_buf() } else { root.join(path) }
        })
        .collect()
}

/// The raw `[legacy].additional_paths` strings, as stored — the identity
/// [`write_additional_paths`] unions new positional paths against (string
/// equality, not resolved-path equality: `additional_paths` is stored in
/// whatever relative form the operator typed, and re-typing the same string
/// must not appear as "new").
fn configured_additional_path_strings(root: &Path) -> Vec<String> {
    let text = StoreHome::resolve(root).operational_text();
    let Ok(value) = text.parse::<toml::Value>() else {
        return Vec::new();
    };
    value
        .get("legacy")
        .and_then(|l| l.get("additional_paths"))
        .and_then(|v| v.as_array())
        .map(|array| array.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// Unions `new_paths` (the just-passed positional strings, already trimmed
/// and empties-dropped) into `[legacy].additional_paths`, sorted +
/// deduplicated, and writes the result back to the operational config file
/// (arc-migration-fidelity s14 F-5) — so a forgotten re-pass of the same
/// positional still covers everything a prior run was told about.
///
/// A no-op (no write, `Ok(false)`) when the union already equals what is
/// stored: idempotency (F-8) falls out of comparing *before* writing, not
/// from hoping a format-preserving edit happens to reproduce the same bytes.
/// `dry_run` short-circuits the same way, before any comparison — the
/// caller decides whether to call this at all, but a defensive check here
/// means a future call site can't forget to gate it.
///
/// Edits with `toml_edit` (`DocumentMut`), not a `toml::Value` round-trip:
/// the operational config is a long-lived, heavily-commented file a human
/// reads and edits directly, and reserializing from `toml::Value` would
/// discard every comment. Only the one key this function owns is touched;
/// everything else in the document is preserved byte-for-byte.
///
/// # Errors
///
/// Returns an error if the operational file cannot be read (when it exists)
/// or the write fails. A missing file is not an error — the config is
/// created fresh, matching a flat store's existing "no config yet" bootstrap.
pub(crate) fn write_additional_paths(
    root: &Path,
    new_paths: &[String],
    dry_run: bool,
) -> anyhow::Result<bool> {
    if dry_run {
        return Ok(false);
    }
    let mut union: Vec<String> = configured_additional_path_strings(root)
        .into_iter()
        .chain(new_paths.iter().cloned())
        .collect();
    union.sort();
    union.dedup();

    if union == configured_additional_path_strings(root) {
        return Ok(false);
    }

    let home = StoreHome::resolve(root);
    let path =
        home.operational_path.unwrap_or_else(|| root.join(odm_store::home::OPERATIONAL_FILE));
    let text = if path.is_file() {
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?
    } else {
        String::new()
    };
    let mut doc = text.parse::<toml_edit::DocumentMut>().with_context(|| {
        format!("parsing {} as TOML for the additional_paths write-back", path.display())
    })?;

    // A bare `doc["legacy"][...]` auto-vivifies an *inline* table
    // (`legacy = { additional_paths = [...] }`), inconsistent with every
    // other section in this file (`[gates.project]`, `[display]`, …) — build
    // an explicit, non-implicit `[legacy]` table so a first-time write reads
    // the same way a hand-authored one would.
    if !doc.contains_key("legacy") {
        let mut table = toml_edit::Table::new();
        table.set_implicit(false);
        doc["legacy"] = toml_edit::Item::Table(table);
    }

    let mut array = toml_edit::Array::new();
    for p in &union {
        array.push(p.as_str());
    }
    array.set_trailing_comma(false);
    doc["legacy"]["additional_paths"] = toml_edit::value(array);

    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(true)
}

/// `show X` — node + edges + way-finding (parent and children). Data → `out`.
/// Writes the node's gate ladder: the normalized state, then every rung with
/// whether it is reached and at what evidence.
///
/// **This is where the raw ladder lives.** RH C-8 normalized `node list`'s
/// STATUS column so types compare, which is only safe if the rung a node is
/// actually on remains reachable — and it was not: before C-8, neither `show`
/// nor `--json` reported gates at all, so `list` was the only place any of it
/// surfaced. Normalizing without this would have made the ladder invisible from
/// the CLI entirely.
fn write_gate_ladder(
    out: &mut dyn Write,
    fm: &Frontmatter,
    gates: Option<&GateSets>,
) -> anyhow::Result<()> {
    let Some(sequence) = gates.and_then(|g| g.for_type(fm.node_type())).map(GateSet::sequence)
    else {
        // No ladder for this type (an `adr` has none) — nothing to report, and
        // an empty "gates:" heading would imply otherwise.
        return Ok(());
    };
    let reached: Vec<&str> = fm.status().reached().map(|(name, _)| name).collect();
    let state = crate::listview::derive_display_status(
        &reached,
        Some(sequence),
        fm.retired().is_some(),
        false,
    );
    writeln!(out, "  status:    {}", state.label())?;
    writeln!(out, "  gates:")?;
    for gate in sequence {
        match fm.status().gate(gate) {
            Some(record) => {
                writeln!(out, "    [x] {gate} — {} ({})", record.reached, record.evidence.as_str())?
            }
            None => writeln!(out, "    [ ] {gate}")?,
        }
    }
    Ok(())
}

pub fn show(
    store: &Store,
    root: &Path,
    reference: &str,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let doc = resolve(store, reference)?;
    let id = doc.frontmatter().id();
    let all = store.load_all()?;
    let children: Vec<&Document> =
        all.iter().filter(|d| d.frontmatter().edges().part_of == Some(id)).collect();
    // A missing/invalid gate config must not make `show` fail — it is a read,
    // and the node is still worth printing. Without it the ladder and the
    // derived state are simply absent rather than guessed.
    let gates = load_gate_config(root).ok().map(|(gates, _)| gates);

    if json {
        writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(&NodeJson::with_gates(&doc, gates.as_ref()))?
        )?;
        return Ok(());
    }

    let fm = doc.frontmatter();
    writeln!(out, "{} #{} {}", fm.node_type().as_str(), fm.number(), fm.name())?;
    writeln!(out, "  id:        {id}")?;
    writeln!(out, "  origin:    {}", fm.origin().as_str())?;
    writeln!(out, "  created:   {}", fm.created())?;
    writeln!(out, "  updated:   {}", fm.updated())?;
    write_gate_ladder(out, fm, gates.as_ref())?;
    if !fm.tags().is_empty() {
        writeln!(out, "  tags:      {}", fm.tags().join(", "))?;
    }
    if let Some(component) = fm.component() {
        writeln!(out, "  component: {component}")?;
    }
    if let Some(retired) = fm.retired() {
        writeln!(out, "  retired:   {} ({})", retired.reason, retired.on)?;
    }
    let edges = fm.edges();
    if let Some(parent) = edges.part_of {
        writeln!(out, "  part_of:   {parent}")?;
    }
    for s in &edges.supersedes {
        writeln!(out, "  supersedes: {} ({})", s.node, supersede_kind_str(s.kind))?;
    }
    // Assumed dependencies, with the reason each was assumed (G-2). A tear is a
    // deliberate, reviewable decision; reading the node should show both that
    // one was made and why, not just that an edge is missing from the ordering.
    if !edges.tears.is_empty() {
        writeln!(out, "  assumed (torn) dependencies:")?;
        for tear in &edges.tears {
            writeln!(out, "    depends_on {} — {}", dependency_label(&tear.edge), tear.because)?;
        }
    }
    // Way-finding: children in the containment tree.
    if children.is_empty() {
        writeln!(out, "  children:  (none)")?;
    } else {
        writeln!(out, "  children:")?;
        for child in children {
            let c = child.frontmatter();
            writeln!(out, "    - {} #{} {}", c.node_type().as_str(), c.number(), c.name())?;
        }
    }
    Ok(())
}

/// `rename X <new-name>` — changes the name only; id and on-disk path are
/// unchanged (the path is a pure function of the immutable id).
pub fn rename(
    store: &Store,
    reference: &str,
    new_name: &str,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut doc = resolve(store, reference)?;
    let fm = doc.frontmatter();
    let (id, number, old_name) = (fm.id(), fm.number(), fm.name().to_string());

    if dry_run {
        term::info(err, &format!("would rename #{number} {old_name:?} -> {new_name:?} ({id})"))?;
        return Ok(());
    }

    doc.frontmatter_mut().set_name(new_name);
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?; // same id => same path, file is rewritten in place
    term::success(err, &format!("renamed #{number} {old_name:?} -> {new_name:?} ({id})"))?;
    Ok(())
}

/// `retire X --because <reason>` — marks the node withdrawn. The file is
/// preserved (git keeps history); this is never a destructive delete.
pub fn retire(
    store: &Store,
    reference: &str,
    reason: &str,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut doc = resolve(store, reference)?;
    let fm = doc.frontmatter();
    let (id, number, name) = (fm.id(), fm.number(), fm.name().to_string());

    if dry_run {
        term::info(err, &format!("would retire #{number} {name:?} ({id}): {reason}"))?;
        return Ok(());
    }

    doc.frontmatter_mut().retire(reason, today());
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?; // overwrites in place — file kept, not deleted
    term::success(err, &format!("retired #{number} {name:?} ({id}): {reason}"))?;
    Ok(())
}

/// `supersede X --with Y --kind <kind>` — records that Y supersedes X. The
/// lineage edge is stored on Y (the newer node), pointing at X. `Y` may
/// supersede more than one node (ODD-0025 §2.3, a synthesis merging many
/// sources): a target not already recorded is appended; a target already
/// recorded has its `kind` updated in place rather than duplicated, so the
/// command is idempotent under repetition.
pub fn supersede(
    store: &Store,
    old_ref: &str,
    with_ref: &str,
    kind: SupersedeKind,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let old = resolve(store, old_ref)?;
    let mut new_doc = resolve(store, with_ref)?;
    let old_id = old.frontmatter().id();
    let (old_number, new_number) = (old.frontmatter().number(), new_doc.frontmatter().number());

    if old_id == new_doc.frontmatter().id() {
        bail!("a node cannot supersede itself");
    }

    if dry_run {
        term::info(
            err,
            &format!(
                "would record #{new_number} supersedes #{old_number} ({})",
                supersede_kind_str(kind)
            ),
        )?;
        return Ok(());
    }

    let supersedes = &mut new_doc.frontmatter_mut().edges_mut().supersedes;
    match supersedes.iter_mut().find(|s| s.node == old_id) {
        Some(existing) => existing.kind = kind,
        None => supersedes.push(Supersedes { node: old_id, kind }),
    }
    new_doc.frontmatter_mut().set_updated(today());
    store.persist(&new_doc)?;
    term::success(
        err,
        &format!("recorded: #{new_number} supersedes #{old_number} ({})", supersede_kind_str(kind)),
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// graph mutators: link / unlink / set-gate / tear (ODD-0013 §3, §4.3, §5.1)
//
// These wire the existing odm-core ops (`edges_mut`, `Status::set_gate`,
// `Tear::new`) to the CLI and persist atomically via odm-store. Edges are
// stored on the **source**; reverse edges stay derived (never written).
// ---------------------------------------------------------------------------

/// The edge kind `link`/`unlink` operates on (source-stored edges only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkEdge {
    /// Ordering dependency.
    DependsOn,
    /// Hard external block.
    BlockedBy,
    /// Consumes a concrete output.
    Consumes,
    /// Verifies the target.
    Verifies,
    /// Affects the target's docs.
    Affects,
    /// Containment parent (single-parent).
    PartOf,
}

impl LinkEdge {
    fn as_str(self) -> &'static str {
        match self {
            LinkEdge::DependsOn => "depends_on",
            LinkEdge::BlockedBy => "blocked_by",
            LinkEdge::Consumes => "consumes",
            LinkEdge::Verifies => "verifies",
            LinkEdge::Affects => "affects",
            LinkEdge::PartOf => "part_of",
        }
    }
}

/// `link X <edge> Y` — adds the edge on the source X. `depends_on` may carry a
/// `--satisfied-at <gate>`. `part_of` enforces a single parent (it replaces any
/// existing parent rather than appending). Re-linking is idempotent.
pub fn link(
    store: &Store,
    source_ref: &str,
    edge: LinkEdge,
    target_ref: &str,
    satisfied_at: Option<&str>,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut src = resolve(store, source_ref)?;
    let target_id = resolve(store, target_ref)?.frontmatter().id();
    let src_id = src.frontmatter().id();
    let (number, name) = (src.frontmatter().number(), src.frontmatter().name().to_string());

    if src_id == target_id {
        bail!(
            "a node cannot {} itself; pick a different target",
            match edge {
                LinkEdge::PartOf => "be `part_of`",
                _ => "link to",
            }
        );
    }
    if satisfied_at.is_some() && edge != LinkEdge::DependsOn {
        bail!("`--satisfied-at` applies only to `depends_on` (got `{}`)", edge.as_str());
    }

    if dry_run {
        term::info(err, &format!("would link #{number} {name:?} {} {target_id}", edge.as_str()))?;
        return Ok(());
    }

    let edges = src.frontmatter_mut().edges_mut();
    match edge {
        LinkEdge::DependsOn => {
            // Replace any existing dependency on the same target (so re-linking
            // can update `satisfied_at`), then add the new one.
            edges.depends_on.retain(|d| dependency_target(d) != target_id);
            let dep = match satisfied_at {
                Some(gate) => {
                    Dependency::Qualified { node: target_id, satisfied_at: gate.to_string() }
                }
                None => Dependency::Bare(target_id),
            };
            edges.depends_on.push(dep);
        }
        LinkEdge::BlockedBy => push_unique(&mut edges.blocked_by, target_id),
        LinkEdge::Consumes => push_unique(&mut edges.consumes, target_id),
        LinkEdge::Verifies => push_unique(&mut edges.verifies, target_id),
        LinkEdge::Affects => push_unique(&mut edges.affects, target_id),
        LinkEdge::PartOf => edges.part_of = Some(target_id), // single-parent: replace
    }
    src.frontmatter_mut().set_updated(today());
    store.persist(&src)?;
    term::success(err, &format!("linked #{number} {name:?} {} {target_id}", edge.as_str()))?;
    Ok(())
}

/// Pushes `id` onto `v` only if absent (idempotent edge add).
fn push_unique(v: &mut Vec<Id>, id: Id) {
    if !v.contains(&id) {
        v.push(id);
    }
}

/// `unlink X <edge> Y` — removes the edge from X. Removing an absent edge is a
/// clear no-op (reported, not an error).
pub fn unlink(
    store: &Store,
    source_ref: &str,
    edge: LinkEdge,
    target_ref: &str,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut src = resolve(store, source_ref)?;
    let target_id = resolve(store, target_ref)?.frontmatter().id();
    let (number, name) = (src.frontmatter().number(), src.frontmatter().name().to_string());

    let present = {
        let edges = src.frontmatter().edges();
        match edge {
            LinkEdge::DependsOn => {
                edges.depends_on.iter().any(|d| dependency_target(d) == target_id)
            }
            LinkEdge::BlockedBy => edges.blocked_by.contains(&target_id),
            LinkEdge::Consumes => edges.consumes.contains(&target_id),
            LinkEdge::Verifies => edges.verifies.contains(&target_id),
            LinkEdge::Affects => edges.affects.contains(&target_id),
            LinkEdge::PartOf => edges.part_of == Some(target_id),
        }
    };
    if !present {
        term::warning(
            err,
            &format!("no-op: #{number} {name:?} has no `{}` edge to {target_id}", edge.as_str()),
        )?;
        return Ok(());
    }

    if dry_run {
        term::info(err, &format!("would unlink #{number} {name:?} {} {target_id}", edge.as_str()))?;
        return Ok(());
    }

    let edges = src.frontmatter_mut().edges_mut();
    match edge {
        LinkEdge::DependsOn => edges.depends_on.retain(|d| dependency_target(d) != target_id),
        LinkEdge::BlockedBy => edges.blocked_by.retain(|&id| id != target_id),
        LinkEdge::Consumes => edges.consumes.retain(|&id| id != target_id),
        LinkEdge::Verifies => edges.verifies.retain(|&id| id != target_id),
        LinkEdge::Affects => edges.affects.retain(|&id| id != target_id),
        LinkEdge::PartOf => edges.part_of = None,
    }
    src.frontmatter_mut().set_updated(today());
    store.persist(&src)?;
    term::success(err, &format!("unlinked #{number} {name:?} {} {target_id}", edge.as_str()))?;
    Ok(())
}

/// The details recorded by [`set_gate`]: the gate name, who recorded it, and at
/// what evidence level.
pub struct GateReach<'a> {
    /// The gate name (must be in the node type's gate-set).
    pub gate: &'a str,
    /// Who recorded reaching it, if known.
    pub by: Option<String>,
    /// The evidence level.
    pub evidence: Evidence,
}

/// `set-gate X <gate> [--by] [--evidence]` — records a reached gate via
/// [`odm_core::status::Status::set_gate`], validating it against the node type's
/// gate-set. Records the slice05.1 per-level first-reach automatically.
pub fn set_gate(
    store: &Store,
    root: &Path,
    reference: &str,
    reach: GateReach<'_>,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let GateReach { gate, by, evidence } = reach;
    let mut doc = resolve(store, reference)?;
    let node_type = doc.frontmatter().node_type();
    let (number, name) = (doc.frontmatter().number(), doc.frontmatter().name().to_string());

    let (gates, _threshold) = load_gate_config(root)?;
    let gate_set = gates.for_type(node_type).ok_or_else(|| {
        anyhow!(
            "no gate-set for type `{}`; add a `[gates.{}]` sequence to odm.toml",
            node_type.as_str(),
            node_type.as_str()
        )
    })?;

    if dry_run {
        term::info(
            err,
            &format!("would set gate {gate:?}={} on #{number} {name:?}", evidence.as_str()),
        )?;
        return Ok(());
    }

    doc.frontmatter_mut()
        .status_mut()
        .set_gate(gate_set, gate, by, evidence, today())
        .map_err(|e| {
            anyhow!(
                "unknown gate {:?} for type `{}`; allowed: {}. Run `odm set-gate {reference} <one-of-those>`",
                e.gate,
                node_type.as_str(),
                e.allowed.join(", ")
            )
        })?;
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?;
    term::success(err, &format!("set gate {gate:?}={} on #{number} {name:?}", evidence.as_str()))?;
    Ok(())
}

/// `tear X depends_on Y --because <r>` — declares a deliberately-assumed
/// dependency edge. The rationale is validated via [`Tear::new`] (empty →
/// rejected) and **persisted** as a [`TornEdge`] (`{ edge, because }`) in
/// `edges.tears` on X, so it survives to `check`'s active-tears listing
/// (ODD-0013 §4.3). A re-tear of the same target refreshes the rationale.
pub fn tear(
    store: &Store,
    source_ref: &str,
    target_ref: &str,
    because: &str,
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut src = resolve(store, source_ref)?;
    let target_id = resolve(store, target_ref)?.frontmatter().id();
    let src_id = src.frontmatter().id();
    let (number, name) = (src.frontmatter().number(), src.frontmatter().name().to_string());

    // Validate the rationale through the model (empty/whitespace → rejected).
    Tear::new(src_id, target_id, because).map_err(|_| {
        anyhow!(
            "a tear needs a rationale; run `odm tear {source_ref} depends_on {target_ref} --because \"<why>\"`"
        )
    })?;

    if dry_run {
        term::info(err, &format!("would tear #{number} {name:?} depends_on {target_id}"))?;
        return Ok(());
    }

    let edges = src.frontmatter_mut().edges_mut();
    let entry = TornEdge { edge: Dependency::Bare(target_id), because: because.to_string() };
    match edges.tears.iter_mut().find(|t| dependency_target(&t.edge) == target_id) {
        // Re-tearing the same target refreshes the rationale (the latest `--because`).
        Some(existing) => existing.because = because.to_string(),
        None => edges.tears.push(entry),
    }
    src.frontmatter_mut().set_updated(today());
    store.persist(&src)?;
    term::success(
        err,
        &format!("tore #{number} {name:?} depends_on {target_id} (because: {because})"),
    )?;
    Ok(())
}

/// `decomposed X [--children <ref…>]` — affirms that X's children fully account
/// for its scope (ODD-0013 §4.5), via [`Frontmatter::affirm_decomposed`]. With
/// `--children`, affirms against those explicit nodes; without, against X's
/// current containment children (reverse `part_of`) — the form that clears a
/// `check` drift / advanced-without-decomposition finding.
///
/// Only parent-capable nodes (`project`/`arc`) can be decomposed.
pub fn decomposed(
    store: &Store,
    reference: &str,
    children: &[String],
    dry_run: bool,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut doc = resolve(store, reference)?;
    let node_type = doc.frontmatter().node_type();
    let id = doc.frontmatter().id();
    let (number, name) = (doc.frontmatter().number(), doc.frontmatter().name().to_string());

    if node_type.valid_child_types().is_empty() {
        bail!(
            "only a project or arc can be `decomposed`; #{number} {name:?} is {}",
            with_article(node_type)
        );
    }

    // Resolve the child set: the explicit `--children`, or X's current
    // **work-typed** containment children (reverse `part_of`) when none are
    // given — the same `decomposition_children` definition `check`'s drift
    // detection uses (s05), so affirming here and checking there can never
    // permanently disagree over a non-work (artifact/note) child.
    let child_ids: Vec<Id> = if children.is_empty() {
        let docs = store.load_all()?;
        let all: Vec<Frontmatter> = docs.iter().map(|d| d.frontmatter().clone()).collect();
        let recomp = recompose::Recomposition::build(&all);
        let types: HashMap<Id, NodeType> = all.iter().map(|f| (f.id(), f.node_type())).collect();
        recompose::decomposition_children(&recomp, &types, id)
    } else {
        let mut ids = Vec::new();
        for c in children {
            ids.push(resolve(store, c)?.frontmatter().id());
        }
        ids
    };

    if dry_run {
        term::info(
            err,
            &format!(
                "would affirm decomposition of #{number} {name:?} ({} child(ren))",
                child_ids.len()
            ),
        )?;
        return Ok(());
    }

    let count = child_ids.len();
    doc.frontmatter_mut().affirm_decomposed(child_ids, today());
    doc.frontmatter_mut().set_updated(today());
    store.persist(&doc)?;
    term::success(
        err,
        &format!("affirmed decomposition of #{number} {name:?} ({count} child(ren))"),
    )?;
    Ok(())
}

/// `use [project|arc] X` — sets the current context slot to node X.
pub fn use_context(
    store: &Store,
    kind: UseKind,
    reference: &str,
    err: &mut dyn Write,
) -> anyhow::Result<()> {
    let doc = resolve(store, reference)?;
    let fm = doc.frontmatter();
    if fm.node_type() != kind.node_type() {
        bail!(
            "{reference:?} is {}, not a {} (use `odm use {} <a {}>`)",
            with_article(fm.node_type()),
            kind.label(),
            kind.label(),
            kind.label()
        );
    }
    let mut ctx = Context::load(store)?;
    match kind {
        UseKind::Project => ctx.project = Some(fm.id()),
        UseKind::Arc => ctx.arc = Some(fm.id()),
    }
    ctx.save(store)?;
    term::success(err, &format!("context: {} = {} ({})", kind.label(), fm.name(), fm.id()))?;
    Ok(())
}

/// A node type with its indefinite article — "an arc", "a slice".
///
/// Only `arc` and `adr` take "an" today, but the rule is spelled as a
/// vowel test rather than a two-item match so a future type is right on arrival.
fn with_article(node_type: NodeType) -> String {
    let word = node_type.as_str();
    let article = if word.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
    format!("{article} {word}")
}

/// `project` — where you are in the plan: the current project and arc.
///
/// Was `context` (RH F-11): the command answers "which project am I in", and
/// the name should be the answer, not the mechanism. `name` inspects a named
/// project instead of the selected one.
pub fn project(
    store: &Store,
    name: Option<&str>,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let ctx = Context::load(store)?;
    // `--name` asks about a *different* project than the selected one. Its arc
    // is deliberately not shown: the current arc belongs to the current
    // selection, and pairing it with another project would read as a claim
    // about that project which nothing established.
    let (project, arc) = match name {
        Some(reference) => {
            let doc = resolve(store, reference)?;
            if doc.frontmatter().node_type() != NodeType::Project {
                bail!(
                    "{reference:?} is {}, not a project (use `odm project` for the current one)",
                    with_article(doc.frontmatter().node_type())
                );
            }
            (Some(doc), None)
        }
        None => (
            ctx.project.and_then(|id| store.load(id).ok()),
            ctx.arc.and_then(|id| store.load(id).ok()),
        ),
    };

    if json {
        let view = serde_json::json!({
            "project": project.as_ref().map(NodeJson::from),
            "arc": arc.as_ref().map(NodeJson::from),
        });
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    match &project {
        Some(d) => writeln!(
            out,
            "project: #{} {} ({})",
            d.frontmatter().number(),
            d.frontmatter().name(),
            d.frontmatter().id()
        )?,
        None => writeln!(out, "project: (none)")?,
    }
    match &arc {
        Some(d) => writeln!(
            out,
            "arc:     #{} {} ({})",
            d.frontmatter().number(),
            d.frontmatter().name(),
            d.frontmatter().id()
        )?,
        None => writeln!(out, "arc:     (none)")?,
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// check (v2) — the single mechanical gate: aggregates every graph-level
// invariant (ODD-0013 §7, §4.3/§4.4/§4.5). It consumes the predicates built in
// arc01 slice06 (schema + link-integrity), arc02 slice02 (cycles), slice04
// (satisfaction/staleness), and slice05 (recomposition) — it does not
// reimplement them.
// ---------------------------------------------------------------------------

/// The severity of a [`CheckEntry`]. **Errors** always fail the run (exit `1`);
/// **warnings** fail only under `--strict` (ODD-0013 §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    /// A hard violation: always fails the run.
    Error,
    /// A soft signal (staleness, soft-satisfaction): fails only under `--strict`.
    Warning,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// One aggregated check finding (internal; rendered to text or [`EntryJson`]).
struct CheckEntry {
    severity: Severity,
    /// A stable, one-word code (e.g. `missing-field`, `cycle`, `orphan`).
    code: &'static str,
    /// The node the finding attaches to (a graph-spanning cycle attaches to its
    /// first member); `None` only if no single node applies.
    node: Option<Id>,
    number: Option<u32>,
    name: Option<String>,
    detail: String,
    /// The exact command or edit that resolves it (errors-as-affordances).
    fix: String,
}

/// JSON shape of one `check` finding (stable schema for `check --json`).
#[derive(Serialize)]
struct EntryJson {
    severity: String,
    code: String,
    node: Option<String>,
    number: Option<u32>,
    name: Option<String>,
    detail: String,
    fix: String,
}

/// One active tear surfaced in `check`'s active-tears listing (ODD-0013 §4.3):
/// an assumed dependency edge in effect, with the rationale that justifies it.
struct ActiveTear {
    from_label: String,
    to_label: String,
    from: Id,
    to: Id,
    because: String,
}

/// JSON shape of one active tear in `check --json` (additive to the v2 schema —
/// existing `ok`/`errors`/`warnings`/`findings` keys are unchanged).
#[derive(Serialize)]
struct TearJson {
    from: String,
    to: String,
    because: String,
}

/// The `check --json` schema marker (arc03 slice04). Additive; versions the
/// contract from its introduction forward.
/// The pure graph-validation payload's schema id.
///
/// This is the payload `check/v1` used to carry. ODD-0023 §5 moved the pure
/// operation to `validate`, and §6a bumped the ids rather than reusing them: a
/// consumer pinned to `check/v1` should fail to recognize what `check` emits
/// now, because `check` no longer does what it did.
pub(crate) const VALIDATE_SCHEMA: &str = "validate/v1";

/// The composite `check` payload's schema id — `validate` + `reconcile`.
pub(crate) const CHECK_SCHEMA: &str = "check/v2";

/// JSON shape of the `validate` report (stable, documented schema).
#[derive(Serialize)]
struct ValidateReport {
    /// The schema-version marker (`"validate/v1"`).
    schema: &'static str,
    /// Whether the run passed (no failing findings for the active mode).
    ok: bool,
    errors: usize,
    warnings: usize,
    findings: Vec<EntryJson>,
    /// The assumed dependencies (tears) in effect, each with its rationale.
    tears: Vec<TearJson>,
}

/// A one-word violation label (stable across the JSON schema and human output).
fn violation_label(v: &Violation) -> &'static str {
    match v {
        Violation::MissingField { .. } => "missing-field",
        Violation::DanglingPartOf { .. } => "dangling-part_of",
        Violation::DanglingEdge { .. } => "dangling-edge",
        Violation::SelfSupersede => "self-supersede",
        Violation::SupersessionCycle { .. } => "supersession-cycle",
        Violation::StaleDoc { .. } => "stale-doc",
        Violation::DanglingReenterWhen { .. } => "dangling-reenter_when",
        Violation::FieldNotValidForType { .. } => "wrong-type-field",
        Violation::UnsupportedSchema { .. } => "unsupported-schema",
        Violation::AbsoluteSourcePath { .. } => "absolute-source-path",
        Violation::InconsistentAuthoredSource { .. } => "inconsistent-authored-source",
        // `Violation` is #[non_exhaustive] (v2 adds kinds); render unknowns
        // generically rather than failing the build when they appear.
        _ => "violation",
    }
}

/// A human-readable detail line for a violation.
fn violation_detail(v: &Violation) -> String {
    match v {
        Violation::MissingField { field } => format!("required field {field:?} is empty"),
        Violation::DanglingPartOf { target } => {
            format!("`part_of` references {target}, which is not in the corpus")
        }
        Violation::DanglingEdge { edge, target } => {
            format!("`{edge}` references {target}, which is not in the corpus")
        }
        Violation::SelfSupersede => "`supersedes` points at the node itself".to_string(),
        Violation::StaleDoc {
            decision_number,
            decision_name,
            decision_updated,
            doc_updated,
            ..
        } => format!(
            "may be stale: governing decision #{decision_number} {decision_name:?} was updated \
             {decision_updated}, after this doc (updated {doc_updated})"
        ),
        Violation::DanglingReenterWhen { reenter_when } => format!(
            "`deferred.reenter_when` references {reenter_when:?}, which is not one of this node's \
             `desired_facts` — its re-entry predicate can never resolve"
        ),
        Violation::SupersessionCycle { cycle } => {
            let ids: Vec<String> = cycle.iter().map(ToString::to_string).collect();
            format!("`supersedes` forms a cycle: {}", ids.join(" -> "))
        }
        Violation::FieldNotValidForType { field, node_type } => {
            format!("`{field}` is not a valid field on a `{}` node", node_type.as_str())
        }
        Violation::UnsupportedSchema { schema } => {
            format!("schema {schema:?} is newer than this binary supports")
        }
        Violation::AbsoluteSourcePath { path } => {
            format!("`source.paths` entry {} is not repo-content-root-relative", path.display())
        }
        Violation::InconsistentAuthoredSource { reason } => reason.to_string(),
        _ => "structural violation".to_string(),
    }
}

/// The exact fix affordance for a finding (errors-as-affordances). Where an
/// Arc-01 command can fix it, the command is named; otherwise the precise file
/// edit is named (the `link`/`unlink` commands that would set edges arrive in
/// Arc 02).
fn violation_fix(store: &Store, finding: &Finding) -> String {
    let file = store.path_of(finding.node);
    let file = file.display();
    match &finding.violation {
        Violation::MissingField { field: "name" } => {
            format!("run `odm rename {} \"<a name>\"`", finding.node)
        }
        Violation::MissingField { field } => {
            format!("set `{field}` in {file}")
        }
        Violation::DanglingPartOf { .. } => {
            format!(
                "edit {file}: repoint `edges.part_of` at an existing node (or `odm new` its parent)"
            )
        }
        Violation::DanglingEdge { edge, .. } => {
            format!("edit {file}: repoint `edges.{edge}` at an existing node")
        }
        Violation::SelfSupersede => {
            format!("edit {file}: remove the self-referential `edges.supersedes`")
        }
        Violation::SupersessionCycle { .. } => {
            format!("edit {file}: break the `supersedes` cycle by removing one link")
        }
        Violation::StaleDoc { decision_number, decision_name, .. } => format!(
            "review {file} against #{decision_number} {decision_name:?}; bump its `updated` \
             once reconciled (or `odm rename`-touch it)"
        ),
        Violation::DanglingReenterWhen { reenter_when } => format!(
            "edit {file}: point `deferred.reenter_when` at one of this node's `desired_facts` \
             ids (declared: {reenter_when:?} is not among them)"
        ),
        Violation::FieldNotValidForType { field, .. } => {
            format!("edit {file}: remove `{field}` (not valid for this node's type)")
        }
        Violation::UnsupportedSchema { .. } => {
            format!("upgrade odm, or edit {file}'s `schema:` to a version this binary supports")
        }
        Violation::AbsoluteSourcePath { .. } => {
            format!("edit {file}: rewrite `source.paths` relative to the repo content root")
        }
        Violation::InconsistentAuthoredSource { .. } => {
            format!(
                "edit {file}: make `origin` and `source.class` agree on whether this node is \
                 authored (ODD-0026 §2.1)"
            )
        }
        _ => format!("inspect {file}"),
    }
}

/// The severity for a structural finding. All v1/v2 structural violations are
/// hard errors *except* the advisory stale-doc warning (slice05): possible
/// staleness is a human-judgment signal, not a proven defect, so it fails only
/// under `--strict` (like the recomposition/staleness warnings).
fn violation_severity(v: &Violation) -> Severity {
    match v {
        Violation::StaleDoc { .. } | Violation::DanglingReenterWhen { .. } => Severity::Warning,
        _ => Severity::Error,
    }
}

/// A short `#<number> <name>` label for a node id, falling back to the id.
fn label_of(by_id: &HashMap<Id, &Frontmatter>, id: Id) -> String {
    by_id.get(&id).map_or_else(|| id.to_string(), |f| format!("#{} {}", f.number(), f.name()))
}

/// The target id of a dependency edge (bare or gate-qualified).
fn dependency_target(dep: &Dependency) -> Id {
    match dep {
        Dependency::Bare(id) => *id,
        Dependency::Qualified { node, .. } => *node,
    }
}

/// Whether a node has *advanced* past its initial (planning) gate — the trigger
/// for the staleness check. Unjudgeable (so `false`) when its type has no
/// configured gate-set.
fn has_advanced(fm: &Frontmatter, gates: &GateSets) -> bool {
    gates.for_type(fm.node_type()).is_some_and(|gset| {
        gset.sequence().iter().enumerate().any(|(i, gate)| i > 0 && fm.status().has_reached(gate))
    })
}

/// The severity of a recomposition finding. A structural break (**orphan**) or
/// a now-false assertion (**decomposition-drift**) is a hard `Error`; the
/// advisory "develop this further" findings (**undeveloped-stub**,
/// **advanced-without-decomposition**) are `Warning`s that fail only under
/// `--strict` — matching the staleness / soft-satisfaction treatment
/// (ODD-0013 §4.4; slice06 CDC rec #2). Everyday `check` no longer exits 1
/// merely because an arc was advanced before its decomposition was affirmed.
fn recompose_severity(issue: &Issue) -> Severity {
    match issue {
        Issue::UndevelopedStub { .. }
        | Issue::AdvancedWithoutDecomposition
        | Issue::UndecomposedParent { .. } => Severity::Warning,
        // Orphan, decomposition-drift, and any future structural issue: error.
        _ => Severity::Error,
    }
}

/// Renders a recomposition finding to `(code, detail, fix)`.
fn recompose_render(store: &Store, f: &recompose::Finding) -> (&'static str, String, String) {
    let file = store.path_of(f.node);
    let file = file.display();
    match &f.issue {
        Issue::Orphan => (
            "orphan",
            "no resolvable containment parent (recomposition is not total)".to_string(),
            format!("edit {file}: set `edges.part_of` to its container (or `odm new` the parent)"),
        ),
        Issue::UndevelopedStub { gate } => (
            "undeveloped-stub",
            format!("advanced to gate {gate:?} with zero children"),
            format!("decompose it (`odm new` its children) or hold its gate at planning in {file}"),
        ),
        Issue::DecompositionDrift { added, removed } => (
            "decomposition-drift",
            format!(
                "children changed since `decomposed` was affirmed (added {}, removed {})",
                added.len(),
                removed.len()
            ),
            format!("re-affirm `decomposed` in {file} after the child-set change"),
        ),
        Issue::AdvancedWithoutDecomposition => (
            "advanced-without-decomposition",
            "reached its terminal gate without affirming `decomposed: complete`".to_string(),
            format!("affirm `decomposed` in {file} before completing it"),
        ),
        Issue::UndecomposedParent { children } => (
            "undecomposed-parent",
            format!(
                "has {children} child(ren) but has never affirmed that they account for its scope"
            ),
            format!("affirm it with `odm node decomposed {}` (or edit {file})", f.number),
        ),
        _ => (
            "recomposition",
            "structural decomposition issue".to_string(),
            format!("inspect {file}"),
        ),
    }
}

/// A dependency's target as display text: the bare id, or `id@gate` when the
/// dependency is gate-qualified.
///
/// Distinct from [`dependency_target`], which answers *which node* — this
/// answers *how to show the edge*, and the qualifying gate is part of that.
fn dependency_label(dep: &Dependency) -> String {
    match dep {
        Dependency::Bare(id) => id.to_string(),
        Dependency::Qualified { node, satisfied_at } => format!("{node}@{satisfied_at}"),
    }
}

/// Whether a project body states a vision (L-3b).
///
/// Matches the heading `orient` looks for, at any heading level and regardless
/// of case, so a body that renders a vision to a reader is not reported as
/// lacking one on a technicality. A heading with nothing under it does **not**
/// count: an empty section is the same absence with extra steps.
fn has_vision(body: &str) -> bool {
    let mut lines = body.lines().skip_while(|l| !is_vision_heading(l));
    if lines.next().is_none() {
        return false;
    }
    // Some prose before the next heading.
    lines.take_while(|l| !l.trim_start().starts_with('#')).any(|l| !l.trim().is_empty())
}

/// Whether `line` is a `# Vision` heading at any level.
fn is_vision_heading(line: &str) -> bool {
    let line = line.trim_start();
    let Some(rest) = line.strip_prefix('#') else { return false };
    rest.trim_start_matches('#').trim().eq_ignore_ascii_case("vision")
}

/// Aggregates every graph-level invariant over the given frontmatters (the
/// index-backed projection, slice06) into a single ordered list of findings:
/// schema + link-integrity (v1), cycles-without-tears, recomposition
/// (orphan/stub/drift/advance-without), out-of-order/staleness, and
/// below-threshold (soft-satisfied) dependencies. It owns no I/O beyond the gate
/// config — the caller supplies the (reconciled) frontmatters.
fn aggregate(
    store: &Store,
    root: &Path,
    frontmatters: &[Frontmatter],
) -> anyhow::Result<(Vec<CheckEntry>, Vec<ActiveTear>)> {
    let by_id: HashMap<Id, &Frontmatter> = frontmatters.iter().map(|f| (f.id(), f)).collect();
    let (gates, threshold) = load_gate_config(root)?;
    let graph = NodeGraph::build(frontmatters);
    let satisfaction = Satisfaction::compute(frontmatters, &gates, threshold);

    let mut entries = Vec::new();

    // (a) schema + link-integrity + supersession (v1) — hard errors; the
    // stale-doc finding (slice05) is a Warning (advisory, --strict-gated).
    for f in odm_core::check::check(frontmatters) {
        entries.push(CheckEntry {
            severity: violation_severity(&f.violation),
            code: violation_label(&f.violation),
            node: Some(f.node),
            number: Some(f.number),
            name: Some(f.name.clone()),
            detail: violation_detail(&f.violation),
            fix: violation_fix(store, &f),
        });
    }

    // (a2) content-level validity (arc06 slice03, ODD-0020) — per-type field
    // validity + schema-version. These need the actual frontmatter content
    // (`desired_facts`/`deferred`/`schema`), which the derived index omits, so
    // read the store for this cheap per-node pass (the A5 store-read pattern).
    let documents = store.load_all().context("loading the corpus for content validity")?;
    let full: Vec<Frontmatter> = documents.iter().map(|d| d.frontmatter().clone()).collect();
    for f in odm_core::check::content_validity(&full) {
        entries.push(CheckEntry {
            severity: violation_severity(&f.violation),
            code: violation_label(&f.violation),
            node: Some(f.node),
            number: Some(f.number),
            name: Some(f.name.clone()),
            detail: violation_detail(&f.violation),
            fix: violation_fix(store, &f),
        });
    }

    // (b) cycle-without-tear (slice02) — a hard error; passes once torn.
    let tears = odm_core::graph::frontmatter_tears(frontmatters);
    if let Err(cycle) = graph.topological_order(&tears) {
        let members = cycle.members();
        let chain: Vec<String> = members.iter().map(|&id| label_of(&by_id, id)).collect();
        let head = members.first().copied();
        let fix = match (members.first(), members.get(1)) {
            (Some(a), Some(b)) => {
                format!("`odm tear {a} depends_on {b} --because \"<reason>\"` to break the cycle")
            }
            _ => "break the dependency cycle by tearing one edge".to_string(),
        };
        entries.push(CheckEntry {
            severity: Severity::Error,
            code: "cycle",
            node: head,
            number: head.and_then(|id| by_id.get(&id).map(|f| f.number())),
            name: head.and_then(|id| by_id.get(&id).map(|f| f.name().to_string())),
            detail: format!("ordering cycle (depends_on/consumes): {}", chain.join(" -> ")),
            fix,
        });
    }

    // (c) recomposition (slice05) — orphan/drift are errors; stub/advance-without
    // are warnings (fail only under --strict). See `recompose_severity`.
    for f in recompose::integrity(frontmatters, &gates) {
        let (code, detail, fix) = recompose_render(store, &f);
        entries.push(CheckEntry {
            severity: recompose_severity(&f.issue),
            code,
            node: Some(f.node),
            number: Some(f.number),
            name: Some(f.name.clone()),
            detail,
            fix,
        });
    }

    // (c2) L-3b: a project must state a vision. The definition of done depends
    // on one, and L-3a had to backfill it into odm's own project after
    // `self-host` carried the plan's structure but not its substance — so this
    // is the rule that stops that regressing silently.
    //
    // Bodies are deliberately out of the index (ODD-0014 §3.5), so this is a
    // targeted load of the project nodes alone — the same shape `orient` uses
    // for the one body it needs. A project that cannot be loaded is skipped
    // rather than reported: that is a store problem, and the schema pass above
    // already owns it.
    //
    // A **superseded** project (arc-migration-fidelity s13: the vision mint's
    // faithful 1:1 record, ODD-0025 §2.3) is exempt — its body is a verbatim
    // migrated copy and must stay that way, not gain an injected heading; the
    // synthesis that supersedes it is the vision-bearing node this rule is
    // actually about. A **retired** project is exempt for the same reason
    // (arc-migration-fidelity s15 F-1 — the collapse retires the superseded
    // 1:1 base *without* a supersedes edge, since the surviving node drops
    // its own `supersedes` too; retirement alone is the historical-record
    // signal here, matching `replan.rs`'s established principle). `retired`
    // is not part of the index projection (only id/type/edges/status —
    // `Derived::load`'s doc), so it's read off the freshly-loaded `doc`
    // below, not the index-reconstructed `fm`.
    let superseded: std::collections::HashSet<Id> =
        full.iter().flat_map(|f| f.edges().supersedes.iter().map(|s| s.node)).collect();
    for fm in frontmatters
        .iter()
        .filter(|f| f.node_type() == NodeType::Project && !superseded.contains(&f.id()))
    {
        let Ok(doc) = store.load(fm.id()) else { continue };
        if doc.frontmatter().retired().is_some() {
            continue;
        }
        if !has_vision(doc.body()) {
            let file = store.path_of(fm.id());
            entries.push(CheckEntry {
                severity: Severity::Warning,
                code: "no-vision",
                node: Some(fm.id()),
                number: Some(fm.number()),
                name: Some(fm.name().to_string()),
                detail: "the project states no vision (no `# Vision` section in its body)"
                    .to_string(),
                fix: format!("add a `# Vision` section to {}", file.display()),
            });
        }
    }

    // (c3) doc-coverage (MF-1/MF-6, arc-migration-fidelity s09 F-5): any `.md`
    // under the configured scan root with no covering node is an Error — "no
    // file left behind" as a mechanically enforced property, built on s08's
    // portable relative `source.paths` key (`odm_migrate::coverage::run`'s
    // primary, exact match). `None` (no `[coverage] scan_root` configured) is
    // the ordinary case today — see `coverage_scan_root`'s doc for why that's
    // deliberate, not a gap.
    if let Some(scan_root) = coverage_scan_root(root) {
        let coverage_report = odm_migrate::coverage::run(store, &scan_root)
            .context("running doc-coverage for `check`")?;
        for uncovered in coverage_report.doc_coverage.uncovered() {
            entries.push(CheckEntry {
                severity: Severity::Error,
                code: "uncovered-doc",
                node: None,
                number: None,
                name: None,
                detail: format!(
                    "`{}` ({}) has no covering node — {}",
                    uncovered.path.display(),
                    uncovered.class.as_str(),
                    uncovered.basis
                ),
                fix: format!(
                    "mint a node for `{}` (`odm migrate --coverage` / the artifact minter), \
                     or remove the file",
                    uncovered.path.display()
                ),
            });
        }
    }

    // (d) soft-satisfaction + (e) out-of-order/staleness (slice04) — warnings.
    // Walk nodes in id order for deterministic output.
    let mut ordered: Vec<&Frontmatter> = frontmatters.iter().collect();
    ordered.sort_by_key(|f| f.id());
    for fm in ordered {
        let reasons = graph.blocked(fm.id(), &satisfaction);

        for reason in &reasons {
            if let Block::SoftSatisfied { dep, evidence, threshold } = reason {
                entries.push(CheckEntry {
                    severity: Severity::Warning,
                    code: "soft-satisfied",
                    node: Some(fm.id()),
                    number: Some(fm.number()),
                    name: Some(fm.name().to_string()),
                    detail: format!(
                        "dependency {} satisfied only at evidence={} (threshold {})",
                        label_of(&by_id, *dep),
                        evidence.as_str(),
                        threshold.as_str()
                    ),
                    fix: format!(
                        "raise {}'s evidence to {} (re-run its verification)",
                        label_of(&by_id, *dep),
                        threshold.as_str()
                    ),
                });
            }
        }

        // Staleness: a node advanced past planning while a dependency is
        // unsatisfied — out of order.
        if has_advanced(fm, &gates) {
            let unsatisfied: Vec<Id> = reasons
                .iter()
                .filter_map(|r| match r {
                    Block::Unsatisfied { dep } => Some(*dep),
                    _ => None,
                })
                .collect();
            if let Some(stale) = staleness_on_advance(fm.id(), unsatisfied) {
                let deps: Vec<String> =
                    stale.unsatisfied.iter().map(|&id| label_of(&by_id, id)).collect();
                entries.push(CheckEntry {
                    severity: Severity::Warning,
                    code: "staleness",
                    node: Some(fm.id()),
                    number: Some(fm.number()),
                    name: Some(fm.name().to_string()),
                    detail: format!(
                        "advanced while dependencies are unsatisfied: {}",
                        deps.join(", ")
                    ),
                    fix: format!(
                        "satisfy {} before advancing, or `odm tear` the edge if intentional",
                        deps.join(", ")
                    ),
                });
            }
        }
    }

    // Active tears: assumed dependencies actually in effect (naming a real
    // ordering edge), each with its persisted rationale. Listed by `check` so
    // they stay visible (ODD-0013 §4.3); not findings — informational.
    let mut active: Vec<ActiveTear> = graph
        .active_tears(&tears)
        .into_iter()
        .map(|t| ActiveTear {
            from_label: label_of(&by_id, *t.from()),
            to_label: label_of(&by_id, *t.to()),
            from: *t.from(),
            to: *t.to(),
            because: t.rationale().to_string(),
        })
        .collect();
    active.sort_by_key(|t| (t.from, t.to));

    Ok((entries, active))
}

/// The outcome of one pure validation pass over the corpus.
///
/// Split out from the command so the composite `check` can look at the result —
/// specifically at `errors` — before deciding whether reconciling is worth
/// doing (ODD-0023 §5).
pub(crate) struct Validation {
    /// The exit code for the active `strict` setting.
    pub code: u8,
    /// Hard findings. Non-zero means `check` stops here.
    pub errors: usize,
    /// Soft findings; they fail only under `strict`.
    pub warnings: usize,
    /// Every finding, in report order.
    entries: Vec<CheckEntry>,
    /// Assumed dependencies currently in effect.
    tears: Vec<ActiveTear>,
    /// How many nodes were validated.
    nodes: usize,
}

/// Runs the graph-level invariants over the full corpus. **Pure**: it reads the
/// corpus and touches nothing else — no probes, no writes.
fn run_validation(store: &Store, root: &Path, strict: bool) -> anyhow::Result<Validation> {
    let (gates, _threshold) = load_gate_config(root)?;
    let frontmatters =
        index_frontmatters(store, &gates).context("reconciling the index to validate")?;
    let (entries, tears) = aggregate(store, root, &frontmatters)?;

    let errors = entries.iter().filter(|e| e.severity == Severity::Error).count();
    let warnings = entries.iter().filter(|e| e.severity == Severity::Warning).count();
    let failed = errors > 0 || (strict && warnings > 0);
    Ok(Validation {
        code: if failed { EXIT_VIOLATIONS } else { EXIT_OK },
        errors,
        warnings,
        entries,
        tears,
        nodes: frontmatters.len(),
    })
}

/// The JSON body for a validation, without the surrounding writeln.
fn validation_report(v: &Validation) -> ValidateReport {
    ValidateReport {
        schema: VALIDATE_SCHEMA,
        ok: v.code == EXIT_OK,
        errors: v.errors,
        warnings: v.warnings,
        findings: v
            .entries
            .iter()
            .map(|e| EntryJson {
                severity: e.severity.as_str().to_string(),
                code: e.code.to_string(),
                node: e.node.map(|id| id.to_string()),
                number: e.number,
                name: e.name.clone(),
                detail: e.detail.clone(),
                fix: e.fix.clone(),
            })
            .collect(),
        tears: v
            .tears
            .iter()
            .map(|t| TearJson {
                from: t.from.to_string(),
                to: t.to.to_string(),
                because: t.because.clone(),
            })
            .collect(),
    }
}

/// Renders a validation as the human report. `label` names the command in the
/// verdict line, so the composite can say `validate:` for its first phase.
fn render_validation(
    out: &mut dyn Write,
    v: &Validation,
    strict: bool,
    label: &str,
) -> anyhow::Result<()> {
    if v.entries.is_empty() {
        term::success(out, &format!("{label}: ok ({} node(s), no problems)", v.nodes))?;
        write_active_tears(out, &v.tears)?;
        return Ok(());
    }

    // The verdict carries the severity: a hard failure reads as an error, a
    // warnings-only run (which does not fail without `--strict`) as a warning.
    let verdict = format!("{label}: {} error(s), {} warning(s)", v.errors, v.warnings);
    if v.errors > 0 {
        term::error(out, &verdict)?;
    } else {
        term::warning(out, &verdict)?;
    }
    for e in &v.entries {
        let who = match (e.number, &e.name, e.node) {
            (Some(n), Some(name), Some(id)) => format!("#{n} {name:?} ({id})"),
            _ => "(corpus)".to_string(),
        };
        writeln!(out, "  [{}] {who}: [{}] {}", e.severity.as_str(), e.code, e.detail)?;
        writeln!(out, "    fix: {}", e.fix)?;
    }
    write_active_tears(out, &v.tears)?;
    if !strict && v.warnings > 0 && v.errors == 0 {
        writeln!(out, "(warnings do not fail; run with --strict to enforce)")?;
    }
    Ok(())
}

/// `validate` — the single mechanical gate: aggregates every graph-level
/// invariant over the full corpus. Returns the exit code ([`EXIT_OK`] when the
/// run passes, [`EXIT_VIOLATIONS`] when it fails). The report is data → `out`.
///
/// **Pure.** No probes, no writes — that is the whole point of it having its own
/// name (ODD-0023 §5). For "is the plan actually true", see [`check`].
///
/// Errors always fail. Warnings (staleness, soft-satisfaction) fail only under
/// `strict` (the CI mode). A clean corpus prints `validate: ok`.
///
/// # Errors
///
/// Returns an error (which the caller maps to exit code `2`) if the corpus
/// cannot be loaded or the gate config is invalid.
pub fn validate(
    store: &Store,
    root: &Path,
    strict: bool,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<u8> {
    let v = run_validation(store, root, strict)?;
    if json {
        writeln!(out, "{}", serde_json::to_string_pretty(&validation_report(&v))?)?;
        return Ok(v.code);
    }
    render_validation(out, &v, strict, "validate")?;
    Ok(v.code)
}

/// `check` — the composite: [`validate`], then [`crate::reconcile::reconcile`].
///
/// "Is my plan actually true?" — the structure holds *and* the world still
/// matches what the nodes claim about it.
///
/// **Validation errors stop the run.** Probing a graph with dangling edges or
/// cycles reports on a state already known to be broken, and probes cost time
/// and have side effects; there is nothing to learn from them until the graph
/// itself is sound. Warnings do not stop it — they do not fail the run either,
/// without `strict`.
///
/// The exit code is the worse of the two phases.
///
/// # Errors
///
/// Returns an error (mapped to exit code `2`) if either phase cannot run.
pub fn check(
    store: &Store,
    root: &Path,
    strict: bool,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<u8> {
    let v = run_validation(store, root, strict)?;
    let stopped = v.errors > 0;

    if json {
        // The reconcile half is `null` when it did not run, which is honest:
        // absent is not the same as clean, and a consumer must be able to tell
        // "no drift" from "we never looked".
        let reconcile =
            if stopped { None } else { Some(crate::reconcile::reconcile_value(store, strict)?) };
        let code = reconcile.as_ref().map_or(v.code, |(rc, _)| v.code.max(*rc));
        let report = serde_json::json!({
            "schema": CHECK_SCHEMA,
            "ok": code == EXIT_OK,
            "validate": validation_report(&v),
            "reconcile": reconcile.map(|(_, value)| value),
            "reconcile_skipped": stopped,
        });
        writeln!(out, "{}", serde_json::to_string_pretty(&report)?)?;
        return Ok(code);
    }

    render_validation(out, &v, strict, "validate")?;
    if stopped {
        writeln!(out)?;
        term::warning(
            out,
            "skipped reconcile — fix the graph first, then `odm check` will probe it",
        )?;
        return Ok(v.code);
    }
    writeln!(out)?;
    let reconcile_code = crate::reconcile::reconcile(store, strict, false, out)?;
    Ok(v.code.max(reconcile_code))
}

/// Writes the active-tears listing (assumed dependencies in effect, each with
/// its rationale) to `out`, or nothing when there are no active tears.
fn write_active_tears(out: &mut dyn Write, tears: &[ActiveTear]) -> anyhow::Result<()> {
    if tears.is_empty() {
        return Ok(());
    }
    writeln!(out, "active tears ({}):", tears.len())?;
    for t in tears {
        writeln!(out, "  {} depends_on {} (because: {})", t.from_label, t.to_label, t.because)?;
    }
    Ok(())
}

/// One `check` finding surfaced for another command (e.g. `orient`) to render,
/// already reduced to its display parts. The caller filters by `is_error`.
pub(crate) struct IntegrityFinding {
    /// Whether this is a hard error (vs. a warning).
    pub(crate) is_error: bool,
    /// The stable one-word code (`orphan`, `cycle`, …).
    pub(crate) code: &'static str,
    /// `#<number> <name>` for the node, or `(corpus)` when none applies.
    pub(crate) who: String,
    /// The human-readable detail line.
    pub(crate) detail: String,
}

/// Runs the full `check` aggregation over the index-backed frontmatters and
/// returns its findings (schema, links, cycles, recomposition, staleness,
/// soft-satisfaction) for another command to surface. `orient` filters these to
/// errors so a structural break is unmissable (slice02 ruling 3). Reuses
/// [`aggregate`] — integrity is never re-walked.
///
/// # Errors
///
/// Returns an error if the gate config is invalid.
pub(crate) fn integrity_findings(
    store: &Store,
    root: &Path,
    frontmatters: &[Frontmatter],
) -> anyhow::Result<Vec<IntegrityFinding>> {
    let (entries, _tears) = aggregate(store, root, frontmatters)?;
    Ok(entries
        .into_iter()
        .map(|e| IntegrityFinding {
            is_error: e.severity == Severity::Error,
            code: e.code,
            who: match (e.number, e.name) {
                (Some(n), Some(name)) => format!("#{n} {name}"),
                _ => "(corpus)".to_string(),
            },
            detail: e.detail,
        })
        .collect())
}

// ---------------------------------------------------------------------------
// derived order: next / blocked / path (ODD-0013 §4.1/§4.4)
// ---------------------------------------------------------------------------

/// Loads the gate-sets and satisfaction threshold from the operational config
/// (absent file ⇒ empty gate-sets and the default threshold).
pub(crate) fn load_gate_config(root: &Path) -> anyhow::Result<(GateSets, Evidence)> {
    // The store's `config.toml` when it has one, else the `odm.toml` the
    // layered search finds — see `StoreHome` (ODD-0022 §4.2).
    let text = StoreHome::resolve(root).operational_text();
    let gates = GateSets::from_toml_str(&text).map_err(|e| anyhow!("gate config: {e}"))?;
    let threshold = threshold_from_toml(&text).map_err(|e| anyhow!("satisfaction config: {e}"))?;
    Ok((gates, threshold))
}

/// The index-backed corpus frontmatters: `reconcile` the `.odm/` index against
/// the corpus (the warm path — freshens any edit) and reconstruct one
/// `Frontmatter` per record via the index→graph adapter — **no full corpus
/// parse**. The reconstructed frontmatters carry exactly what the graph,
/// satisfaction, recomposition, and provenance read — id, type, number, edges,
/// status with evidence, and (since slice06) `origin` and `decomposed` — so
/// every consumer built on them is identical to its `load_all` baseline. Bodies
/// stay out of the index (ODD-0014 §3.5); a consumer that needs one (only
/// `orient`'s vision) does its own targeted [`Store::load`].
///
/// # Errors
///
/// Returns an error if the index cannot be reconciled.
pub(crate) fn index_frontmatters(
    store: &Store,
    gates: &GateSets,
) -> anyhow::Result<Vec<Frontmatter>> {
    let snapshot =
        odm_index::reconcile(store, &odm_index::default_index_path(store.root()))?.snapshot;
    Ok(odm_index::frontmatters_from_records(&snapshot.records, gates))
}

/// The corpus, graph, and satisfaction needed by every derived-order query.
///
/// **Index-backed (slice05):** `load` freshens the `.odm/` index (`reconcile`)
/// and reconstructs the `Frontmatter`s from the index records via the index→graph
/// adapter — no corpus parse — then feeds the *existing* `NodeGraph::build` /
/// `Satisfaction::compute`. The reconstructed frontmatters carry exactly what the
/// graph + satisfaction read (id/type/edges/status+evidence), so the derived
/// order is identical to the `load_all` baseline.
struct Derived {
    frontmatters: Vec<Frontmatter>,
    graph: NodeGraph,
    satisfaction: Satisfaction,
}

impl Derived {
    fn load(store: &Store, root: &Path) -> anyhow::Result<Self> {
        let (gates, threshold) = load_gate_config(root)?;
        let frontmatters = index_frontmatters(store, &gates)?;
        let graph = NodeGraph::build(&frontmatters);
        let satisfaction = Satisfaction::compute(&frontmatters, &gates, threshold);
        Ok(Self { frontmatters, graph, satisfaction })
    }

    /// A short `#<number> <name>` label for a node id (falls back to the id).
    fn label(&self, id: Id) -> String {
        self.frontmatters
            .iter()
            .find(|f| f.id() == id)
            .map_or_else(|| id.to_string(), |f| format!("#{} {}", f.number(), f.name()))
    }

    fn number(&self, id: Id) -> Option<u32> {
        self.frontmatters.iter().find(|f| f.id() == id).map(Frontmatter::number)
    }
}

#[derive(Serialize)]
struct SoftDepJson {
    dep: String,
    number: Option<u32>,
    evidence: String,
}

#[derive(Serialize)]
struct ReadyJson {
    node: String,
    number: Option<u32>,
    effective_evidence: Option<String>,
    soft: Vec<SoftDepJson>,
}

/// `next` — the ready frontier, soft-satisfied deps flagged. Data → `out`.
///
/// **Graph-pure by design (arc05 slice08, L-6).** `next` answers one question —
/// which nodes are graph-ready (deps satisfied, gates met) — and is deliberately
/// *unaffected* by a node's `deferred` marker. Deferral is a reconcile-time
/// concern (its re-entry predicate is a probe), so it is surfaced only in the
/// reconcile-aware views (`rollup`/`orient`), never here. Making `next` withhold
/// deferred nodes would force it either to index the marker (a rejected schema
/// invariant) or to run a reconcile (which would re-introduce the slice04
/// volatile-probe regression on a hot path). Graph-ready ≠ parked; both surfaces
/// are correct at their own layer.
pub fn next(store: &Store, root: &Path, json: bool, out: &mut dyn Write) -> anyhow::Result<()> {
    let derived = Derived::load(store, root)?;
    let ready = derived.graph.next(&derived.satisfaction);

    if json {
        let view: Vec<ReadyJson> = ready
            .iter()
            .map(|r| ReadyJson {
                node: r.node.to_string(),
                number: derived.number(r.node),
                effective_evidence: derived
                    .graph
                    .min_evidence(r.node, &derived.satisfaction)
                    .map(|e| e.as_str().to_string()),
                soft: r
                    .soft
                    .iter()
                    .map(|s| SoftDepJson {
                        dep: s.dep.to_string(),
                        number: derived.number(s.dep),
                        evidence: s.evidence.as_str().to_string(),
                    })
                    .collect(),
            })
            .collect();
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    if ready.is_empty() {
        writeln!(out, "next: (nothing ready)")?;
        return Ok(());
    }
    for r in &ready {
        writeln!(out, "{}", derived.label(r.node))?;
        for soft in &r.soft {
            writeln!(
                out,
                "  ⚠ dep {} satisfied at evidence={}",
                derived.label(soft.dep),
                soft.evidence.as_str()
            )?;
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct BlockJson {
    kind: String,
    node: String,
    number: Option<u32>,
    evidence: Option<String>,
    threshold: Option<String>,
}

/// `blocked X` — the unsatisfied / soft-satisfied / externally-blocked reasons.
pub fn blocked(
    store: &Store,
    root: &Path,
    reference: &str,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let derived = Derived::load(store, root)?;
    let target = resolve(store, reference)?;
    let reasons = derived.graph.blocked(target.frontmatter().id(), &derived.satisfaction);

    if json {
        let view: Vec<BlockJson> = reasons.iter().map(block_json).collect();
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    if reasons.is_empty() {
        writeln!(out, "blocked: nothing holding {}", derived.label(target.frontmatter().id()))?;
        return Ok(());
    }
    for reason in &reasons {
        match reason {
            Block::Unsatisfied { dep } => {
                writeln!(out, "  unsatisfied dependency {}", derived.label(*dep))?;
            }
            Block::SoftSatisfied { dep, evidence, threshold } => {
                writeln!(
                    out,
                    "  low-evidence dependency {} at evidence={} (raise to {})",
                    derived.label(*dep),
                    evidence.as_str(),
                    threshold.as_str()
                )?;
            }
            Block::ExternallyBlocked { by } => {
                writeln!(out, "  blocked by {}", derived.label(*by))?;
            }
        }
    }
    Ok(())
}

fn block_json(reason: &Block<Id, Evidence>) -> BlockJson {
    match reason {
        Block::Unsatisfied { dep } => BlockJson {
            kind: "unsatisfied".to_string(),
            node: dep.to_string(),
            number: None,
            evidence: None,
            threshold: None,
        },
        Block::SoftSatisfied { dep, evidence, threshold } => BlockJson {
            kind: "soft-satisfied".to_string(),
            node: dep.to_string(),
            number: None,
            evidence: Some(evidence.as_str().to_string()),
            threshold: Some(threshold.as_str().to_string()),
        },
        Block::ExternallyBlocked { by } => BlockJson {
            kind: "blocked-by".to_string(),
            node: by.to_string(),
            number: None,
            evidence: None,
            threshold: None,
        },
    }
}

#[derive(Serialize)]
struct PathJson {
    path: Option<Vec<String>>,
}

/// `path X [Y]` — the critical dependency chain from X, or a path from X to Y.
pub fn chain(
    store: &Store,
    root: &Path,
    reference: &str,
    to: Option<&str>,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let derived = Derived::load(store, root)?;
    let from = resolve(store, reference)?.frontmatter().id();
    let target = to.map(|t| resolve(store, t)).transpose()?.map(|d| d.frontmatter().id());
    let chain = derived.graph.path(from, target, &[]);

    if json {
        let view =
            PathJson { path: chain.as_ref().map(|c| c.iter().map(ToString::to_string).collect()) };
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    match chain {
        Some(chain) => {
            let labels: Vec<String> = chain.iter().map(|&id| derived.label(id)).collect();
            writeln!(out, "{}", labels.join(" -> "))?;
        }
        None => writeln!(out, "path: no dependency path")?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ----- s14 F-5/F-6: the additional_paths write-back ---------------------

    #[test]
    fn test_write_additional_paths_creates_a_fresh_config_when_none_exists() {
        let dir = TempDir::new().unwrap();
        let wrote = write_additional_paths(dir.path(), &["research".to_string()], false).unwrap();
        assert!(wrote, "a fresh config is created and reports a change");
        let config = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
        assert!(
            config.contains("[legacy]") && config.contains("additional_paths = [\"research\"]"),
            "written from nothing:\n{config}"
        );
    }

    #[test]
    fn test_write_additional_paths_errors_on_unparsable_existing_config() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("config.toml"), "not = [valid : toml").unwrap();
        let result = write_additional_paths(dir.path(), &["x".to_string()], false);
        assert!(result.is_err(), "a malformed config is a real error, not a silent no-op");
    }

    #[test]
    fn test_configured_additional_paths_is_empty_for_unparsable_config() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("config.toml"), "not = [valid : toml").unwrap();
        assert!(
            configured_additional_paths(dir.path()).is_empty(),
            "a malformed config resolves to nothing extra, not an error \
             (--all's other steps must still run)"
        );
    }

    // ----- L-3b: the no-vision predicate ------------------------------------

    #[test]
    fn test_has_vision_accepts_a_stated_vision() {
        assert!(has_vision("# odm\n\n# Vision\n\nA planning substrate.\n"));
    }

    #[test]
    fn test_has_vision_is_indifferent_to_heading_level_and_case() {
        // A body that renders a vision to a reader must not be reported as
        // lacking one over a `##` or a lowercase `v`.
        assert!(has_vision("# P\n\n## Vision\n\nText.\n"));
        assert!(has_vision("# P\n\n### vision\n\nText.\n"));
        assert!(has_vision("# P\n\n#   VISION   \n\nText.\n"));
    }

    #[test]
    fn test_has_vision_rejects_an_absent_section() {
        assert!(!has_vision("# odm\n\nSome prose but no vision heading.\n"));
        assert!(!has_vision(""));
    }

    #[test]
    fn test_has_vision_rejects_an_empty_section() {
        // A heading with nothing under it is the same absence with extra steps.
        assert!(!has_vision("# P\n\n# Vision\n\n# Next section\n\nText.\n"));
        assert!(!has_vision("# P\n\n# Vision\n"));
    }

    #[test]
    fn test_has_vision_ignores_the_word_elsewhere() {
        // Prose about a vision is not a stated vision, and a heading that merely
        // begins with the word is a different section.
        assert!(!has_vision("# P\n\nOur vision is great.\n"));
        assert!(!has_vision("# P\n\n# Vision statement process\n\nText.\n"));
    }

    // ----- G-2: how a tear's target renders ---------------------------------

    #[test]
    fn test_dependency_label_keeps_the_qualifying_gate() {
        let id = odm_core::Id::new();
        assert_eq!(dependency_label(&Dependency::Bare(id)), id.to_string());
        assert_eq!(
            dependency_label(&Dependency::Qualified { node: id, satisfied_at: "tested".into() }),
            format!("{id}@tested")
        );
    }
}
