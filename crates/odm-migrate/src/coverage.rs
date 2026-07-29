//! **Coverage discovery** (arc-migration-fidelity slice01): a read-only doc-coverage
//! and gap detector over a docs tree, producing the exact, re-runnable inventory the
//! rest of the arc plans against.
//!
//! Four detectors, each a count + an offending list:
//!
//! 1. **doc-coverage** — every source `.md` matched to a node, or reported uncovered.
//! 2. **representation** — arc/slice directories vs. arc/slice nodes.
//! 3. **stub-body** — work nodes whose body is a lone synthesized H1 (tombstones excluded).
//! 4. **provenance-absence** — nodes carrying no `provenance` key.
//!
//! **Doc-coverage matching is primarily exact** (arc-migration-fidelity s05, F-7,
//! ODD-0025 §5): a source doc's path is matched against every node's
//! `source.paths` — an exact set-difference, no derivation involved. A doc
//! whose node predates this arc's identity model (no `source` yet) falls back
//! to the original **heuristic** match: structurally (arc/slice directory
//! coordinates, reusing [`crate::selfhost::parse_prefix`]) or by frontmatter
//! `number` (ODDs, reusing [`crate::legacy::parse_file`]). A doc this run
//! reports uncovered *may* be a fallback-matcher miss rather than a true hole —
//! every [`CoverageEntry`] carries its matching `basis` so the report can say
//! so.
//!
//! **Read-only**: [`run`] only reads `docs_root` and `store` — it persists no
//! document and mutates no file (F-7).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use odm_core::NodeType;
use odm_core::frontmatter::Document;
use odm_store::{Store, StoreError};
use walkdir::WalkDir;

use std::collections::HashMap;

use crate::legacy;
use crate::selfhost::{arc_number, named_arc_number, parse_prefix, slice_number, slice_position};

/// A source document's coarse classification (F-2). A closed set matching the
/// canonical shapes the `1.0.x` corpus (and the general planning-corpus
/// convention) is known to carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocClass {
    /// `project-plan.md` — the project's plan-of-record.
    ProjectPlan,
    /// `arc-plan.md` — an arc's plan-of-record.
    ArcPlan,
    /// `slice-doc.md` — a slice's plan-of-record.
    SliceDoc,
    /// `ledger.md` — a slice's acceptance ledger.
    Ledger,
    /// `cc-prompt*.md` — an implementation prompt (a slice's or a chunk's).
    CcPrompt,
    /// `*cdc-verification.md` — an independent-verification report.
    CdcVerification,
    /// `*closing-report.md` — a slice's or chunk's closing report.
    ClosingReport,
    /// A frontmatter design doc under `docs/design/` (an ODD, in the pre-1.0
    /// vocabulary) — `research`-tagged or not; both live in the same tree.
    Odd,
    /// A `docs/dev/` note, excluding the `research` subtree.
    Dev,
    /// A `docs/dev/research/` investigation doc.
    Research,
    /// Anything else: ADRs, amendments, UAT artifacts, session handoffs, and
    /// other ad-hoc/cross-scale docs (the audit's "ad-hoc / other" cohort).
    Other,
}

impl DocClass {
    /// The canonical lowercase label (for rendering).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            DocClass::ProjectPlan => "project-plan",
            DocClass::ArcPlan => "arc-plan",
            DocClass::SliceDoc => "slice-doc",
            DocClass::Ledger => "ledger",
            DocClass::CcPrompt => "cc-prompt",
            DocClass::CdcVerification => "cdc-verification",
            DocClass::ClosingReport => "closing-report",
            DocClass::Odd => "odd",
            DocClass::Dev => "dev",
            DocClass::Research => "research",
            DocClass::Other => "other",
        }
    }

    /// Every class, in a stable display order (used to group the report).
    #[must_use]
    pub fn all() -> [DocClass; 11] {
        [
            DocClass::ProjectPlan,
            DocClass::ArcPlan,
            DocClass::SliceDoc,
            DocClass::Ledger,
            DocClass::CcPrompt,
            DocClass::CdcVerification,
            DocClass::ClosingReport,
            DocClass::Odd,
            DocClass::Dev,
            DocClass::Research,
            DocClass::Other,
        ]
    }
}

/// A source `.md` file under a docs root, classified.
#[derive(Debug, Clone)]
pub struct SourceDoc {
    /// The path, relative to the docs root.
    pub path: PathBuf,
    /// Its classification.
    pub class: DocClass,
}

/// One doc-coverage row: a source doc, whether it matched a node, and the basis
/// for that verdict (F-3's heuristic caveat, carried per-row rather than only in
/// prose).
#[derive(Debug, Clone)]
pub struct CoverageEntry {
    /// The path, relative to the docs root.
    pub path: PathBuf,
    /// Its classification.
    pub class: DocClass,
    /// Whether it matched an existing node.
    pub covered: bool,
    /// Why (or why not) — the matching basis, always stated (never presented as
    /// certainty when it is a heuristic).
    pub basis: &'static str,
}

/// The doc-coverage detector's result (F-3): every source doc, matched or not.
#[derive(Debug, Clone)]
pub struct DocCoverageReport {
    /// Every classified source doc, in path order.
    pub entries: Vec<CoverageEntry>,
}

impl DocCoverageReport {
    /// The total number of source docs considered.
    #[must_use]
    pub fn total(&self) -> usize {
        self.entries.len()
    }

    /// The number matched to an existing node.
    #[must_use]
    pub fn covered_count(&self) -> usize {
        self.entries.iter().filter(|e| e.covered).count()
    }

    /// The number with no match — the doc-coverage gap.
    #[must_use]
    pub fn uncovered_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.covered).count()
    }

    /// The unmatched entries, in path order.
    pub fn uncovered(&self) -> impl Iterator<Item = &CoverageEntry> {
        self.entries.iter().filter(|e| !e.covered)
    }
}

/// The representation detector's result (F-4): arc/slice **directories** on disk
/// vs. arc/slice **nodes** in the store.
#[derive(Debug, Clone, Default)]
pub struct RepresentationGap {
    /// Every top-level `arc*` directory found.
    pub total_arc_dirs: usize,
    /// Arc directory names with no matching arc node, sorted.
    pub missing_arcs: Vec<String>,
    /// Every `slice*` directory found under any arc directory.
    pub total_slice_dirs: usize,
    /// `"<arc-dir>/<slice-dir>"` for slice directories with no matching slice
    /// node, sorted.
    pub missing_slices: Vec<String>,
}

impl RepresentationGap {
    /// Arc directories that resolved to an existing node.
    #[must_use]
    pub fn represented_arc_dirs(&self) -> usize {
        self.total_arc_dirs - self.missing_arcs.len()
    }

    /// Slice directories that resolved to an existing node.
    #[must_use]
    pub fn represented_slice_dirs(&self) -> usize {
        self.total_slice_dirs - self.missing_slices.len()
    }
}

/// One stub-body finding (F-5): a work node whose body is effectively empty.
#[derive(Debug, Clone)]
pub struct StubEntry {
    /// The node's number.
    pub number: u32,
    /// The node's type (`arc` or `slice`).
    pub node_type: NodeType,
    /// The node's name.
    pub name: String,
}

/// One provenance-absence finding (F-6): a node carrying no `source` sub-map
/// (the stored lineage record — see [`provenance_absence`]'s doc for why
/// `source`, not a `provenance` key, is the right thing to check).
#[derive(Debug, Clone)]
pub struct ProvenanceEntry {
    /// The node's number.
    pub number: u32,
    /// The node's type.
    pub node_type: NodeType,
    /// The node's name.
    pub name: String,
}

/// The full coverage/gap inventory (F-1…F-6): the arc's work-list.
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// The doc-coverage detector's result.
    pub doc_coverage: DocCoverageReport,
    /// The representation detector's result.
    pub representation: RepresentationGap,
    /// The stub-body detector's result.
    pub stubs: Vec<StubEntry>,
    /// The provenance-absence detector's result.
    pub provenance_missing: Vec<ProvenanceEntry>,
}

/// A fatal error running the coverage detectors — a store load failure. Per-doc
/// problems (a malformed ODD, an unparsed template) are heuristic misses
/// reflected in the report, never raised (mirrors [`crate::MigrateError`]'s
/// per-doc-vs-fatal split).
#[derive(Debug, thiserror::Error)]
pub enum CoverageError {
    /// The store's corpus could not be loaded.
    #[error("loading the store corpus")]
    LoadCorpus(#[source] StoreError),
}

/// Runs the four coverage/gap detectors over `docs_root`, matching against the
/// nodes in `store`. Read-only: no node is persisted, no source file is written.
///
/// # Errors
///
/// [`CoverageError::LoadCorpus`] if the store's corpus cannot be loaded.
pub fn run(store: &Store, docs_root: &Path) -> Result<CoverageReport, CoverageError> {
    let corpus = store.load_all().map_err(CoverageError::LoadCorpus)?;
    // The same anchor `source.paths` is stored relative to (arc-migration-fidelity
    // s08 F-1/F-6) — canonicalized so the doc-coverage comparison key lines up
    // with the stored form regardless of how `docs_root` itself was spelled or
    // reached, the same robustness `self_host`/`repair` need (s08's whole point).
    let docs_root_buf = docs_root.canonicalize().unwrap_or_else(|_| docs_root.to_path_buf());
    let docs_root = docs_root_buf.as_path();
    let anchor = crate::fidelity::anchor_for(docs_root);
    let index = NodeIndex::build(&corpus, &anchor);

    let docs = enumerate_docs(docs_root);
    let doc_coverage = doc_coverage(&docs, &index, docs_root, &anchor);
    let representation = representation(&docs, &index);
    let stubs = stub_bodies(&corpus);
    let provenance_missing = provenance_absence(&corpus);

    Ok(CoverageReport { doc_coverage, representation, stubs, provenance_missing })
}

/// An in-memory index of the store's existing nodes, keyed the way each
/// detector needs to look them up — built once per [`run`].
struct NodeIndex {
    project_exists: bool,
    arc_numbers: BTreeSet<u32>,
    slice_numbers: BTreeSet<u32>,
    doc_numbers: BTreeSet<u32>,
    /// Every `source.paths` entry across the whole corpus, in **canonical,
    /// anchor-relative form** (arc-migration-fidelity s05 F-7 / s08 F-1/F-6,
    /// ODD-0025 §5) — the **primary**, exact doc-coverage match. Built via
    /// [`crate::fidelity::relativize`], which tolerates a stored entry that is
    /// itself still absolute (pre-s08), so this stays correct against a
    /// not-yet-rewritten corpus too. The `*_numbers` sets above are the
    /// **pre-`source` fallback** (a legacy node that predates this arc's
    /// identity model), kept only for that transition.
    source_paths: BTreeSet<String>,
}

impl NodeIndex {
    fn build(corpus: &[Document], anchor: &Path) -> Self {
        let mut index = Self {
            project_exists: false,
            arc_numbers: BTreeSet::new(),
            slice_numbers: BTreeSet::new(),
            doc_numbers: BTreeSet::new(),
            source_paths: BTreeSet::new(),
        };
        for document in corpus {
            let fm = document.frontmatter();
            match fm.node_type() {
                NodeType::Project => index.project_exists = true,
                NodeType::Arc => {
                    index.arc_numbers.insert(fm.number());
                }
                NodeType::Slice => {
                    index.slice_numbers.insert(fm.number());
                }
                NodeType::Design | NodeType::Research => {
                    index.doc_numbers.insert(fm.number());
                }
                // `Artifact` nodes are minted with `source` from the start
                // (s09) — there is no pre-`source` legacy state to fall back
                // from, so no number-based fallback set is needed for them
                // (unlike `Design`/`Research`, whose live corpus predates
                // `source` entirely). They match via `source_paths` below,
                // same as every other type.
                NodeType::Adr | NodeType::Note | NodeType::Artifact => {}
            }
            if let Some(source) = fm.source() {
                index
                    .source_paths
                    .extend(source.paths.iter().map(|p| crate::fidelity::relativize(anchor, p)));
            }
        }
        index
    }
}

// ----- enumeration + classification (F-2) -----------------------------------

/// Enumerates every `.md` file under `docs_root`, classified, in path order.
///
/// Unfiltered — every `.md` file counts, including index pages and templates,
/// so `enumerate_docs(root).len() == find <root> -name '*.md' | wc -l` (F-2).
/// Non-document artifacts are simply classified [`DocClass::Other`] (or `Odd`,
/// for an unparsable file under `docs/design/`) rather than excluded — the
/// doc-coverage detector then reports them uncovered rather than silently
/// vanishing them from the count.
#[must_use]
pub fn enumerate_docs(docs_root: &Path) -> Vec<SourceDoc> {
    let mut docs = Vec::new();
    for entry in WalkDir::new(docs_root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let relative = path.strip_prefix(docs_root).unwrap_or(path).to_path_buf();
        let class = classify(&relative);
        docs.push(SourceDoc { path: relative, class });
    }
    docs.sort_by(|a, b| a.path.cmp(&b.path));
    docs
}

/// Classifies one source doc path (relative to a docs root) into a
/// [`DocClass`]. Canonical basenames are checked first (they identify a doc
/// regardless of which root it sits under); the doc's top-level root directory
/// decides the rest.
fn classify(relative: &Path) -> DocClass {
    let basename = relative.file_name().and_then(|s| s.to_str()).unwrap_or_default();
    if basename == "project-plan.md" {
        return DocClass::ProjectPlan;
    }
    if basename == "arc-plan.md" {
        return DocClass::ArcPlan;
    }
    if basename == "slice-doc.md" {
        return DocClass::SliceDoc;
    }
    if basename == "ledger.md" {
        return DocClass::Ledger;
    }
    if basename.starts_with("cc-prompt") {
        return DocClass::CcPrompt;
    }
    if basename.ends_with("cdc-verification.md") {
        return DocClass::CdcVerification;
    }
    if basename.ends_with("closing-report.md") {
        return DocClass::ClosingReport;
    }

    let mut components =
        relative.components().map(|c| c.as_os_str().to_string_lossy().into_owned());
    match components.next().as_deref() {
        Some("design") => DocClass::Odd,
        Some("dev") => {
            if components.next().as_deref() == Some("research") {
                DocClass::Research
            } else {
                DocClass::Dev
            }
        }
        _ => DocClass::Other,
    }
}

// ----- doc-coverage detector (F-3) -------------------------------------------

fn doc_coverage(
    docs: &[SourceDoc],
    index: &NodeIndex,
    docs_root: &Path,
    anchor: &Path,
) -> DocCoverageReport {
    let entries = docs
        .iter()
        .map(|doc| {
            let absolute = docs_root.join(&doc.path);
            // Primary: an exact `source.paths` match (ODD-0025 §5, F-7),
            // compared in the same canonical, anchor-relative form `source`
            // is stored in (s08 F-6) — this alone resolves a named arc, which
            // the structural fallbacks below can never do (a named arc
            // directory has no numbered coordinate to re-derive; see
            // `arc_coordinate`'s doc).
            let key = crate::fidelity::relativize(anchor, &absolute);
            let (covered, basis) = if index.source_paths.contains(&key) {
                (true, "source.paths (exact match)")
            } else {
                match doc.class {
                    DocClass::ProjectPlan => (
                        index.project_exists,
                        "structural (project root exists) — pre-source fallback",
                    ),
                    DocClass::ArcPlan => match_arc(&doc.path, index),
                    DocClass::SliceDoc => match_slice(&doc.path, index),
                    DocClass::Odd => match_odd(&absolute, index),
                    DocClass::Ledger
                    | DocClass::CcPrompt
                    | DocClass::CdcVerification
                    | DocClass::ClosingReport => {
                        (false, "no node class yet for supporting docs (lands in arc slice 07)")
                    }
                    DocClass::Dev | DocClass::Research | DocClass::Other => {
                        (false, "no node class yet for this doc class")
                    }
                }
            };
            CoverageEntry { path: doc.path.clone(), class: doc.class, covered, basis }
        })
        .collect();
    DocCoverageReport { entries }
}

/// **Pre-`source` fallback** (reached only when the primary `source.paths`
/// check in [`doc_coverage`] misses): matches an `arc-plan.md` to its arc node
/// via its containing directory's structural coordinate (reusing
/// [`parse_prefix`]/[`arc_number`] — the same derivation
/// [`crate::selfhost::self_host`] uses to mint it). A **named** arc directory
/// has no such coordinate at all — see [`arc_coordinate`]'s doc — so this
/// fallback can never resolve one; the primary check is what actually closes
/// that gap (F-7) for a source-bearing named-arc node.
fn match_arc(relative: &Path, index: &NodeIndex) -> (bool, &'static str) {
    let Some(arc_dir) = relative.parent().and_then(Path::file_name).and_then(|s| s.to_str()) else {
        return (false, "unrecognized path shape (no containing directory)");
    };
    match arc_coordinate(arc_dir) {
        Some(major) if index.arc_numbers.contains(&arc_number(major)) => {
            (true, "structural coordinate (arc directory number)")
        }
        Some(_) => (false, "structural coordinate resolved; no matching arc node"),
        None => (
            false,
            "named arc directory — no numbered coordinate to re-derive \
             (heuristic limitation, not a scope exclusion; see the module docs)",
        ),
    }
}

/// Matches a `slice-doc.md` to its slice node via its arc+slice directory
/// coordinates (reusing [`parse_prefix`]/[`slice_number`]).
fn match_slice(relative: &Path, index: &NodeIndex) -> (bool, &'static str) {
    let mut ancestors = relative.components().rev();
    ancestors.next(); // the file name itself
    let Some(slice_dir) = ancestors.next().and_then(|c| c.as_os_str().to_str()) else {
        return (false, "unrecognized path shape (no slice directory)");
    };
    let Some(arc_dir) = ancestors.next().and_then(|c| c.as_os_str().to_str()) else {
        return (false, "unrecognized path shape (no arc directory)");
    };
    let Some(arc_major) = arc_coordinate(arc_dir) else {
        return (false, "parent arc directory has no numbered coordinate (a named arc)");
    };
    match parse_prefix(slice_dir, "slice") {
        Some((slice_major, slice_minor)) => {
            let number = slice_number(arc_major, slice_major, slice_minor);
            if index.slice_numbers.contains(&number) {
                (true, "structural coordinate (arc+slice directory number)")
            } else {
                (false, "structural coordinate resolved; no matching slice node")
            }
        }
        None => (false, "slice directory has no numbered coordinate"),
    }
}

/// Matches an ODD-tree doc to a `design`/`research` node by its frontmatter
/// `number` (reusing [`legacy::parse_file`] — the same read [`crate::mapping`]
/// uses). Document numbers are a single space shared by both node types (see
/// the crate-level numbering-space note), so presence in either counts.
fn match_odd(path: &Path, index: &NodeIndex) -> (bool, &'static str) {
    match legacy::parse_file(path) {
        Ok(doc) => match doc.front.number {
            Some(number) if index.doc_numbers.contains(&number) => {
                (true, "number match (frontmatter `number`)")
            }
            Some(_) => (false, "frontmatter `number` present; no matching document node"),
            None => (false, "no frontmatter `number` (not an importable ODD)"),
        },
        Err(_) => (false, "unparsable frontmatter (an index page or template, not an ODD)"),
    }
}

// ----- representation detector (F-4) -----------------------------------------

/// Derives the set of arc/slice **directories** from the already-classified
/// [`DocClass::ArcPlan`]/[`DocClass::SliceDoc`] docs (one directory per
/// `arc-plan.md`/`slice-doc.md`, by the corpus convention), and reports which
/// have no matching node.
///
/// Deliberately driven off the classified doc list rather than a second,
/// independent filesystem walk: `docs_root` is a general docs tree (`docs/`)
/// whose plan-set root (`design-v1.0.0/`, in odm's case) has no fixed name or
/// depth this crate should assume — the `arc-plan.md`/`slice-doc.md` locations
/// *are* the arc/slice directories, wherever they sit.
///
/// Every directory counts whether or not it parses as a numbered coordinate —
/// a named arc (`arc-store-home`) is exactly as real a directory as
/// `arc01-substrate-node-crud`, and excluding it from the count would hide the
/// representation gap this detector exists to surface.
fn representation(docs: &[SourceDoc], index: &NodeIndex) -> RepresentationGap {
    let mut gap = RepresentationGap::default();

    let arc_dirs: BTreeSet<&Path> = docs
        .iter()
        .filter(|d| d.class == DocClass::ArcPlan)
        .filter_map(|d| d.path.parent())
        .collect();
    gap.total_arc_dirs = arc_dirs.len();
    let arc_dir_numbers = resolve_arc_dir_numbers(&arc_dirs);
    for arc_dir in &arc_dirs {
        let represented =
            arc_dir_numbers.get(arc_dir).is_some_and(|n| index.arc_numbers.contains(n));
        if !represented {
            gap.missing_arcs.push(dir_label(arc_dir));
        }
    }

    let slice_dirs: BTreeSet<&Path> = docs
        .iter()
        .filter(|d| d.class == DocClass::SliceDoc)
        .filter_map(|d| d.path.parent())
        .collect();
    gap.total_slice_dirs = slice_dirs.len();
    for slice_dir in &slice_dirs {
        let slice_name = dir_label(slice_dir);
        let arc_name = slice_dir.parent().map(dir_label).unwrap_or_default();
        // `arc_num + slice_position(...)`, not `slice_number(arc_num, ...)`:
        // `slice_number` re-derives `arc_number(arc_major)` from a **raw**
        // arc major, which `arc_num` (already the final handle — either
        // `arc_number(major)` or a named arc's own name-derived handle) is
        // not; adding the position offset directly is what `discover()`
        // itself does for both numbered and named arcs alike.
        let represented = slice_dir
            .parent()
            .and_then(|arc_dir| arc_dir_numbers.get(arc_dir))
            .is_some_and(|&arc_num| {
                parse_prefix(&slice_name, "slice").is_some_and(|(smaj, smin)| {
                    index.slice_numbers.contains(&(arc_num + slice_position(smaj, smin)))
                })
            });
        if !represented {
            gap.missing_slices.push(format!("{arc_name}/{slice_name}"));
        }
    }

    gap
}

/// Resolves every arc directory in `arc_dirs` to the `number` it would be (or
/// was) minted with — mirroring [`crate::selfhost::discover`]'s own two-phase
/// derivation exactly: a numbered directory (`arcNN-*`) via [`arc_number`]; a
/// **named** directory (`arc-<slug>`) via [`named_arc_number`], collision-
/// handled against the handles already assigned to *other named dirs* earlier
/// in the same sorted pass (`arc_dirs` is a `BTreeSet<&Path>`, the same `Ord`
/// [`crate::selfhost::discover`]'s own `named.sort()` uses, so the two passes
/// agree on ordering and therefore on collision handling too).
///
/// **CDC v2.8 Finding 2 (s09 F-6):** [`representation`] used to only resolve
/// the numbered case ([`arc_coordinate`]) — a named arc directory has no
/// `arcNN` coordinate to re-derive from the name alone, so it always read
/// unrepresented even once its node existed (the "8/12" undercount: 4 named
/// arcs, permanently missing). Recomputing the name-derived key here — the
/// same key [`crate::selfhost::self_host`] minted it with — closes that gap:
/// a named arc with a matching node is now counted represented.
fn resolve_arc_dir_numbers<'a>(arc_dirs: &BTreeSet<&'a Path>) -> HashMap<&'a Path, u32> {
    let mut numbers = HashMap::new();
    for &dir in arc_dirs {
        if let Some(major) = arc_coordinate(&dir_label(dir)) {
            numbers.insert(dir, arc_number(major));
        }
    }
    let mut named_taken: BTreeSet<u32> = BTreeSet::new();
    for &dir in arc_dirs {
        if arc_coordinate(&dir_label(dir)).is_some() {
            continue;
        }
        let slug = dir_label(dir);
        let number = named_arc_number(&slug, &named_taken);
        named_taken.insert(number);
        numbers.insert(dir, number);
    }
    numbers
}

/// A directory's own name (the last path component), as a label for the report.
fn dir_label(dir: &Path) -> String {
    dir.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

/// The numbered-arc major number a directory name resolves to, or `None` if
/// it carries no `arcNN` coordinate at all (a fractional arc, or a **named**
/// arc directory such as `arc-store-home`).
///
/// **No scope filter (v1.6 F11/F3):** every numbered arc counts, including
/// the former post-MVP `arc07`/`arc08` — the hardcoded A1–A6 cap this used to
/// apply is removed, not widened.
///
/// **Structural-fallback limitation, by design:** a named arc's actual
/// `number` is a *name-derived handle* ([`named_arc_number`], v1.9) computed
/// from a hash of the slug plus collision-handling against the arcs already
/// assigned earlier in the same pass — information a single directory name
/// can't recover **in isolation** (there is no "just re-derive it" from one
/// name alone once collisions are possible). This function therefore still
/// cannot resolve a coordinate for a named arc on its own. The **doc-coverage**
/// detector doesn't need it (arc-migration-fidelity s05, F-7):
/// [`doc_coverage`]'s primary `source.paths` check resolves a source-bearing
/// named-arc node directly, exactly as ODD-0025 §5 anticipated. The
/// **representation** detector, which has no `source` to key on, instead
/// recomputes the handle from the full set of named-arc slugs together —
/// see [`resolve_arc_dir_numbers`] (s09, F-6) — which *can* replay the same
/// collision-handling `discover()` used to mint it.
fn arc_coordinate(dir_name: &str) -> Option<u32> {
    parse_prefix(dir_name, "arc").filter(|(_, minor)| minor.is_none()).map(|(major, _)| major)
}

// ----- stub-body detector (F-5) ----------------------------------------------

/// Work nodes (`arc`/`slice`) whose body is effectively empty (≤ 1 non-blank
/// line — the lone synthesized `# {name}` H1 the old self-host importer wrote
/// before arc-migration-fidelity slice03 removed that transform). Retired
/// (tombstone) nodes are excluded — a retired node's body is meant to be a
/// terse marker, not migrated content.
///
/// The predicate itself ([`crate::fidelity::is_stub_body`]) is shared with
/// the s04 update-in-place repair op, so both agree on exactly the same set
/// (s04 ledger F-8 — "don't re-derive the stub predicate").
fn stub_bodies(corpus: &[Document]) -> Vec<StubEntry> {
    corpus
        .iter()
        .filter(|d| matches!(d.frontmatter().node_type(), NodeType::Arc | NodeType::Slice))
        .filter(|d| d.frontmatter().retired().is_none())
        .filter(|d| crate::fidelity::is_stub_body(d.body()))
        .map(|d| StubEntry {
            number: d.frontmatter().number(),
            node_type: d.frontmatter().node_type(),
            name: d.frontmatter().name().to_string(),
        })
        .collect()
}

// ----- provenance-absence detector (F-6) -------------------------------------

/// Nodes carrying no lineage record.
///
/// **CDC v2.8 Finding 3 (s09 F-7):** this used to scan the node's emitted
/// frontmatter YAML for a literal `provenance:` line — but ODD-0025 §2.0
/// renamed that key to `source:` back at s02, and `provenance` itself is
/// **derived-only, never stored** (0013 reserves the name for computed
/// lineage; the stored record is the distinct `source` sub-map). So the old
/// scan was checking for a key the corpus can never carry, and silently
/// flagged every node. **Decision:** retarget to the typed
/// [`odm_core::frontmatter::Frontmatter::source`] accessor — `source` absent
/// is the real "no lineage record" signal, and it's exact (no YAML text scan,
/// no risk of the field moving out of the untyped `extra` catch-all later).
fn provenance_absence(corpus: &[Document]) -> Vec<ProvenanceEntry> {
    corpus
        .iter()
        .filter(|d| d.frontmatter().source().is_none())
        .map(|d| ProvenanceEntry {
            number: d.frontmatter().number(),
            node_type: d.frontmatter().node_type(),
            name: d.frontmatter().name().to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_class_all_covers_every_variant_once() {
        let all = DocClass::all();
        let unique: BTreeSet<_> = all.iter().map(|c| c.as_str()).collect();
        assert_eq!(unique.len(), all.len(), "DocClass::all() lists no duplicates");
    }
}
