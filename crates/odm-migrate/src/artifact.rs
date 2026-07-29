//! Artifact-family discovery + mint-all (arc-migration-fidelity slice09,
//! ODD-0025 §2.5/§2.6): the supporting-doc cohort (`ledger`, `cc-prompt`,
//! `cdc-verification`, `closing-report`, ADR, amendment, UAT, and every
//! generated report) minted as `artifact` nodes — faithful 1:1 body +
//! `source`, `part_of` its nearest **modeled** scale (slice, else arc, else
//! left top-level — §2.7's optional containment for document-family nodes).
//!
//! Deliberately its own module, not folded into `selfhost.rs`: it walks the
//! *whole* docs tree ([`crate::coverage::enumerate_docs`]), not the
//! plan-set's arc/slice-directory walk `discover()` performs, and mints a
//! different node family with no `part_of` requirement.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;

use crate::coverage::{self, DocClass};
use crate::{MigrateError, Mode};

/// The `number` band artifact nodes mint into — clear of every other band in
/// use: the numbered arc/slice range (`arc_number`/`slice_number`, ≤ ~1800),
/// the named-arc range ([`crate::selfhost::named_arc_number`],
/// `NAMED_ARC_BASE` ..= ~100,001,800), and the legacy ODD document-number
/// space (small, hand-authored integers). `number` carries no identity
/// meaning here — s05 retired `number` as the identity key in favor of
/// `source.paths` — it exists only because [`Frontmatter::new`] requires one,
/// and a human reading a report wants *a* stable handle rather than a
/// repeated `0`.
const ARTIFACT_NUMBER_BASE: u32 = 500_000_000;
/// How many hash slots the artifact band spans before repeating — see
/// [`crate::selfhost::named_arc_number`]'s identical rationale.
const ARTIFACT_NUMBER_SLOTS: u32 = 1_000_000;
/// The spacing between artifact number slots, leaving room for collision
/// bumps without ever reaching the next hash slot.
const ARTIFACT_NUMBER_STEP: u32 = 100;

/// A deterministic (FNV-1a) `number` handle for an artifact's own anchor-
/// relative path, collision-bumped against `taken` — the same technique
/// [`crate::selfhost::named_arc_number`] uses for a named arc's handle,
/// applied here to a doc path instead of a directory slug. Recomputable from
/// the path alone, so a re-run assigns the identical handle to an
/// already-minted artifact (moot for matching — that's `source.paths` — but
/// keeps the human-facing number stable across runs too).
fn artifact_number(relative_path: &str, taken: &BTreeSet<u32>) -> u32 {
    let mut candidate = ARTIFACT_NUMBER_BASE
        + (crate::selfhost::slug_hash(relative_path) % ARTIFACT_NUMBER_SLOTS)
            * ARTIFACT_NUMBER_STEP;
    while taken.contains(&candidate) {
        candidate += ARTIFACT_NUMBER_STEP;
    }
    candidate
}

/// The artifact-family [`DocClass`]es (ODD-0025 §2.5) — the supporting-doc
/// cohort discovery reaches. `Other` is included deliberately: it is the
/// ADR/amendment/UAT/generated-report catch-all, and **mint-all** (§2.6) means
/// no exemption — `coverage-report.md` classifies `Other` and is minted like
/// everything else.
fn is_artifact_class(class: DocClass) -> bool {
    matches!(
        class,
        DocClass::Ledger
            | DocClass::CcPrompt
            | DocClass::CdcVerification
            | DocClass::ClosingReport
            | DocClass::Other
    )
}

/// One artifact minted (or, under `--dry-run`, that would be minted).
#[derive(Debug, Clone)]
pub struct MintedArtifact {
    /// The path, relative to `docs_root`.
    pub path: PathBuf,
    /// The freshly-minted identity.
    pub id: Id,
    /// The assigned (non-identity) number handle.
    pub number: u32,
    /// The name (its first H1, or the doc's own relative path).
    pub name: String,
    /// The id of the nearest modeled scale it's `part_of`, if any (§2.7).
    pub contained_by: Option<Id>,
}

/// The outcome of an artifact mint-all run.
#[derive(Debug, Clone)]
pub struct ArtifactReport {
    /// Artifacts minted (or, under `--dry-run`, that would be minted).
    pub minted: Vec<MintedArtifact>,
    /// Whether this was a dry run (nothing written).
    pub dry_run: bool,
}

impl ArtifactReport {
    /// The number of artifacts minted (or planned, under `--dry-run`).
    #[must_use]
    pub fn minted_count(&self) -> usize {
        self.minted.len()
    }
}

/// A directory → containing-node index built from every persisted `arc`/
/// `slice` node's own `source.paths` entry (its `arc-plan.md`/`slice-doc.md`
/// file) — resolves an artifact doc's nearest **modeled** containing scale
/// (ODD-0025 §2.5) without re-deriving arc/slice numbering: a node's own
/// directory is looked up directly, in the same canonical, anchor-relative
/// form `source.paths` is already stored in (s08) — so this resolves
/// identically for a numbered *or* a **named** arc/slice, with no special
/// case for either, and needs no `discover()` re-walk.
fn scale_index(corpus: &[Document], anchor: &Path) -> HashMap<String, Id> {
    let mut index = HashMap::new();
    for document in corpus {
        let fm = document.frontmatter();
        if !matches!(fm.node_type(), NodeType::Arc | NodeType::Slice) {
            continue;
        }
        let Some(source) = fm.source() else { continue };
        for path in &source.paths {
            let relative = crate::fidelity::relativize(anchor, path);
            if let Some((dir, _file)) = relative.rsplit_once('/') {
                index.insert(dir.to_string(), fm.id());
            }
        }
    }
    index
}

/// Resolves `doc_relative`'s (anchor-relative) nearest modeled containing
/// scale: walks its ancestor directories, nearest first, returning the id of
/// the first slice/arc node whose own directory matches — `None` if no
/// ancestor directory is modeled (a legitimately top-level artifact, §2.7).
///
/// A doc living directly inside a slice directory resolves at distance zero
/// (its slice); one living in an arc directory but no slice subdirectory —
/// a chunk-level doc, e.g. `c1-cdc-verification.md` — resolves to the arc,
/// exactly as §2.5 specifies ("chunk-level → its arc", no `chunk` scale).
fn nearest_scale(doc_relative: &str, index: &HashMap<String, Id>) -> Option<Id> {
    let mut dir = Path::new(doc_relative).parent();
    while let Some(d) = dir {
        let key = d.to_string_lossy();
        if key.is_empty() {
            break;
        }
        if let Some(&id) = index.get(key.as_ref()) {
            return Some(id);
        }
        dir = d.parent();
    }
    None
}

/// Mints an `artifact` node (ODD-0025 §2.5) for every supporting doc under
/// `docs_root` not already covered by an existing node's `source.paths` —
/// **mint-all** (§2.6: no exemption, `coverage-report.md` included),
/// idempotent (re-running mints nothing new, since a minted doc becomes
/// `source.paths`-covered), 1:1 verbatim body under the §2.1 hard body-hash
/// gate, `part_of` its nearest **modeled** scale (slice, else arc, else left
/// top-level — §2.7's optional containment).
///
/// Reads a file, mints a node: no transformation, no synthesized heading
/// (mirrors [`crate::selfhost::self_host`]'s own body-import discipline).
///
/// # Errors
///
/// [`MigrateError`] if the corpus can't be loaded, a doc body can't be read,
/// the body-hash gate fails, or a persist fails.
pub fn mint_artifacts(
    store: &Store,
    docs_root: &Path,
    mode: Mode,
) -> Result<ArtifactReport, MigrateError> {
    let docs_root_buf = docs_root.canonicalize().unwrap_or_else(|_| docs_root.to_path_buf());
    let docs_root = docs_root_buf.as_path();
    let anchor = crate::fidelity::anchor_for(docs_root);

    let corpus = store.load_all().map_err(MigrateError::LoadCorpus)?;
    let already_covered: HashSet<String> = corpus
        .iter()
        .filter_map(|d| d.frontmatter().source())
        .flat_map(|s| s.paths.iter().map(|p| crate::fidelity::relativize(&anchor, p)))
        .collect();
    let scale_index = scale_index(&corpus, &anchor);

    let today = chrono::Utc::now().date_naive();
    let mut taken: BTreeSet<u32> = BTreeSet::new();
    let mut minted = Vec::new();

    for doc in coverage::enumerate_docs(docs_root) {
        if !is_artifact_class(doc.class) {
            continue;
        }
        let absolute = docs_root.join(&doc.path);
        let relative = crate::fidelity::relativize(&anchor, &absolute);
        if already_covered.contains(&relative) {
            continue; // already minted (or otherwise covered) — idempotent
        }

        let body = std::fs::read_to_string(&absolute)
            .map_err(|source| MigrateError::SourceRead { path: absolute.clone(), source })?;

        let number = artifact_number(&relative, &taken);
        taken.insert(number);
        let id = Id::new();
        let name =
            crate::selfhost::first_h1(&absolute).unwrap_or_else(|| doc.path.display().to_string());
        let contained_by = nearest_scale(&relative, &scale_index);
        // Real dates from the doc's own git history (RH F-20), not "the day
        // the mint ran" — falls back to `today` only when git has no record
        // (an untracked fixture).
        let (created, updated) = crate::fidelity::git_derived_dates(&anchor, &absolute, today);

        let mut fm = Frontmatter::new(
            id,
            number,
            NodeType::Artifact,
            name.clone(),
            created,
            updated,
            Origin::Planned,
        );
        fm.stamp_schema();
        fm.edges_mut().part_of = contained_by;
        let fm = fm.with_source(crate::fidelity::build_source(
            vec![PathBuf::from(&relative)],
            doc.class.as_str(),
            today,
        ));

        let document = Document::new(fm, body.clone());
        crate::fidelity::verify_body_hash(&body, document.body(), format!("artifact {relative}"))?;

        minted.push(MintedArtifact { path: doc.path.clone(), id, number, name, contained_by });

        if !mode.is_dry_run() {
            store.persist(&document).map_err(|source| MigrateError::Persist { number, source })?;
        }
    }

    minted.sort_by_key(|m| m.path.clone());
    Ok(ArtifactReport { minted, dry_run: mode.is_dry_run() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_number_is_recomputable_and_collision_free() {
        let taken = BTreeSet::new();
        let a = artifact_number("docs/x/ledger.md", &taken);
        let b = artifact_number("docs/x/ledger.md", &taken);
        assert_eq!(a, b, "recomputable from the path alone");
        assert!(a >= ARTIFACT_NUMBER_BASE);

        let mut bumped = BTreeSet::new();
        bumped.insert(a);
        let c = artifact_number("docs/x/ledger.md", &bumped);
        assert_ne!(a, c, "a taken candidate is bumped, never silently reused");
    }

    #[test]
    fn is_artifact_class_covers_the_supporting_doc_cohort_and_other() {
        for class in [
            DocClass::Ledger,
            DocClass::CcPrompt,
            DocClass::CdcVerification,
            DocClass::ClosingReport,
            DocClass::Other,
        ] {
            assert!(is_artifact_class(class), "{class:?} is artifact-family");
        }
        for class in [DocClass::ProjectPlan, DocClass::ArcPlan, DocClass::SliceDoc, DocClass::Odd] {
            assert!(!is_artifact_class(class), "{class:?} is not artifact-family");
        }
    }

    #[test]
    fn nearest_scale_resolves_a_chunk_level_doc_to_its_arc() {
        let mut index = HashMap::new();
        let arc_id = Id::new();
        index.insert("docs/design-v1.0.0/arc01-alpha".to_string(), arc_id);
        let resolved =
            nearest_scale("docs/design-v1.0.0/arc01-alpha/c1-cdc-verification.md", &index);
        assert_eq!(resolved, Some(arc_id));
    }

    #[test]
    fn nearest_scale_is_none_for_a_top_level_doc() {
        let index = HashMap::new();
        assert_eq!(nearest_scale("docs/design-v1.0.0/benchmark-results.md", &index), None);
    }
}
