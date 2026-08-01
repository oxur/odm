//! Dev-doc discovery + mint (arc-migration-fidelity slice10, operator
//! decision 2026-07-28): every markdown file under a project's dev-docs
//! directory becomes a `note` node — faithful 1:1 body + `source`,
//! deliberately **uncontained** (a dev doc predates any particular slice; it
//! cannot be reliably attributed to one — the operator's call: "top-level
//! note docs will give us the most consistency").
//!
//! **General `odm-migrate` capability, not project-specific.** The pattern —
//! a dev-docs directory full of dated, occasionally-subdirectory-grouped
//! development history that is the precursor material to later slice docs
//! and cc-prompts — recurs across every pre-1.0 odm project, via the legacy
//! `Config::dev_directory` field (`legacy/oxur-odm/src/config.rs`, default
//! `"./docs/dev"`, carried forward — currently unread — into the v1.0 store's
//! own `config.toml` at the C-5 cutover). This module reads no config itself;
//! the caller resolves `dev_directory` and passes it as `dev_root`, exactly
//! how [`crate::artifact::mint_artifacts`] takes its scan root.
//!
//! The immediate subdirectory a doc lives in (if any) becomes a **tag** on
//! its note — `docs/dev/research/0001-x.md` tags `research`; a file directly
//! under the dev root gets no tag. This preserves the one piece of
//! structural metadata a flat mint would otherwise discard.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::{Id, NodeType, Origin};
use odm_store::Store;
use walkdir::WalkDir;

use crate::legacy;
use crate::{MigrateError, Mode};

/// The `number` band note nodes mint into — clear of the work-node bands
/// (≤ ~100,001,800) and the artifact band
/// ([`crate::artifact`]'s `ARTIFACT_NUMBER_BASE`, 500,000,000..=~599,999,900).
/// `number` carries no identity meaning here (`source.paths` does) — see
/// [`crate::artifact`]'s identical rationale.
const NOTE_NUMBER_BASE: u32 = 700_000_000;
/// See [`crate::artifact`]'s `ARTIFACT_NUMBER_SLOTS` for the identical
/// rationale.
const NOTE_NUMBER_SLOTS: u32 = 1_000_000;
/// See [`crate::artifact`]'s `ARTIFACT_NUMBER_STEP` for the identical
/// rationale.
const NOTE_NUMBER_STEP: u32 = 100;

/// A deterministic (FNV-1a) `number` handle for a note's own anchor-relative
/// path, collision-bumped against `taken` — the same technique
/// [`crate::artifact::mint_artifacts`] uses for an artifact's handle.
fn note_number(relative_path: &str, taken: &BTreeSet<u32>) -> u32 {
    let mut candidate = NOTE_NUMBER_BASE
        + (crate::selfhost::slug_hash(relative_path) % NOTE_NUMBER_SLOTS) * NOTE_NUMBER_STEP;
    while taken.contains(&candidate) {
        candidate += NOTE_NUMBER_STEP;
    }
    candidate
}

/// One note minted (or, under `--dry-run`, that would be minted).
#[derive(Debug, Clone)]
pub struct MintedNote {
    /// The path, relative to `dev_root`.
    pub path: PathBuf,
    /// The freshly-minted identity.
    pub id: Id,
    /// The assigned (non-identity) number handle.
    pub number: u32,
    /// The name (its first H1, or the doc's own relative path).
    pub name: String,
    /// The tag derived from the doc's immediate subdirectory, if any.
    pub tag: Option<String>,
}

/// One already-covered note whose body had drifted from its current source
/// and was re-snapshotted in place (arc-migration-fidelity s15 F-4) —
/// `id`/`number` preserved (a note is deliberately uncontained, so there is
/// no `part_of` to preserve), only `body`/`source`/`updated` change.
#[derive(Debug, Clone)]
pub struct ReconciledNote {
    /// The path, relative to `dev_root`.
    pub path: PathBuf,
    /// The node's identity (unchanged — a reconcile never re-mints).
    pub id: Id,
    /// The node's `number` handle (unchanged).
    pub number: u32,
    /// The node's name.
    pub name: String,
}

/// The outcome of a note mint-or-reconcile run.
#[derive(Debug, Clone)]
pub struct NoteReport {
    /// Notes minted (or, under `--dry-run`, that would be minted).
    pub minted: Vec<MintedNote>,
    /// Already-covered notes re-snapshotted because their body had drifted
    /// from their current source (or, under `--dry-run`, that would be) —
    /// s15 F-4.
    pub reconciled: Vec<ReconciledNote>,
    /// Whether this was a dry run (nothing written).
    pub dry_run: bool,
}

impl NoteReport {
    /// The number of notes minted (or planned, under `--dry-run`).
    #[must_use]
    pub fn minted_count(&self) -> usize {
        self.minted.len()
    }

    /// The number of notes reconciled (or planned, under `--dry-run`).
    #[must_use]
    pub fn reconciled_count(&self) -> usize {
        self.reconciled.len()
    }
}

/// Enumerates every `.md` under `dev_root` (recursively), excluding the same
/// non-document files [`legacy::discover`] does (index pages, templates,
/// dot-directories), each paired with its immediate-subdirectory name if it
/// lives one or more levels below `dev_root`.
fn enumerate_dev_docs(dev_root: &Path) -> Vec<(PathBuf, Option<String>)> {
    let mut docs = Vec::new();
    for entry in WalkDir::new(dev_root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let relative = path.strip_prefix(dev_root).unwrap_or(path).to_path_buf();
        if legacy::is_excluded(&relative) {
            continue;
        }
        let tag = relative
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .and_then(|parent| parent.components().next())
            .map(|component| component.as_os_str().to_string_lossy().into_owned());
        docs.push((relative, tag));
    }
    docs.sort();
    docs
}

/// Mints a `note` node for every dev doc under `dev_root` not already covered
/// by an existing node's `source.paths` — mint-all (mirrors
/// [`crate::artifact::mint_artifacts`]'s policy), idempotent, 1:1 verbatim
/// body under the hard body-hash gate, deliberately **uncontained**
/// (`part_of` is never set — see the module doc for why). An already-covered
/// doc whose body has since drifted from its current source is
/// **re-snapshotted in place** instead of silently skipped — mint-or-
/// reconcile, arc-migration-fidelity s15 F-4, mirroring
/// [`crate::artifact::mint_artifacts`]'s identical extension.
///
/// Reads a file, mints or reconciles a node: no transformation, no
/// synthesized heading. A retired note is a historical record and is never
/// reconciled.
///
/// # Errors
///
/// [`MigrateError`] if the corpus can't be loaded, a doc body can't be read,
/// the body-hash gate fails, or a persist fails.
pub fn mint_notes(store: &Store, dev_root: &Path, mode: Mode) -> Result<NoteReport, MigrateError> {
    let dev_root_buf = dev_root.canonicalize().unwrap_or_else(|_| dev_root.to_path_buf());
    let dev_root = dev_root_buf.as_path();
    let anchor = crate::fidelity::anchor_for(dev_root);

    let corpus = store.load_all().map_err(MigrateError::LoadCorpus)?;
    let mut covered: HashMap<String, Document> = HashMap::new();
    for document in &corpus {
        let Some(source) = document.frontmatter().source() else { continue };
        for path in &source.paths {
            covered.insert(crate::fidelity::relativize(&anchor, path), document.clone());
        }
    }

    let today = chrono::Utc::now().date_naive();
    let mut taken: BTreeSet<u32> = BTreeSet::new();
    let mut minted = Vec::new();
    let mut reconciled = Vec::new();

    for (relative_to_root, tag) in enumerate_dev_docs(dev_root) {
        let absolute = dev_root.join(&relative_to_root);
        let relative = crate::fidelity::relativize(&anchor, &absolute);

        if let Some(existing) = covered.get(&relative) {
            if existing.frontmatter().retired().is_some() {
                continue; // a historical record — never reconciled
            }
            let body = std::fs::read_to_string(&absolute)
                .map_err(|source| MigrateError::SourceRead { path: absolute.clone(), source })?;
            if existing.body() == body {
                continue; // already faithful — no-op (idempotent)
            }

            let fm = existing.frontmatter();
            let (_, updated) = crate::fidelity::git_derived_dates(&anchor, &absolute, today);
            let mut new_fm = fm.clone();
            new_fm.set_updated(updated);
            new_fm.stamp_schema();
            let new_fm = new_fm.with_source(crate::fidelity::build_source(
                vec![PathBuf::from(&relative)],
                "dev-doc",
                today,
            ));
            let new_document = Document::new(new_fm, body.clone());
            crate::fidelity::verify_body_hash(
                &body,
                new_document.body(),
                format!("note {relative} reconcile"),
            )?;

            reconciled.push(ReconciledNote {
                path: relative_to_root.clone(),
                id: fm.id(),
                number: fm.number(),
                name: fm.name().to_string(),
            });
            if !mode.is_dry_run() {
                store
                    .persist(&new_document)
                    .map_err(|source| MigrateError::Persist { number: fm.number(), source })?;
            }
            continue;
        }

        let body = std::fs::read_to_string(&absolute)
            .map_err(|source| MigrateError::SourceRead { path: absolute.clone(), source })?;

        let number = note_number(&relative, &taken);
        taken.insert(number);
        let id = Id::new();
        let name = crate::selfhost::first_h1(&absolute)
            .unwrap_or_else(|| relative_to_root.display().to_string());
        // Real dates from the doc's own git history (RH F-20), not "the day
        // the mint ran" — falls back to `today` only when git has no record
        // (an untracked fixture).
        let (created, updated) = crate::fidelity::git_derived_dates(&anchor, &absolute, today);

        let mut fm = Frontmatter::new(
            id,
            number,
            NodeType::Note,
            name.clone(),
            created,
            updated,
            Origin::Planned,
        );
        fm.stamp_schema();
        if let Some(tag) = &tag {
            fm = fm.with_tags(vec![tag.clone()]);
        }
        let fm = fm.with_source(crate::fidelity::build_source(
            vec![PathBuf::from(&relative)],
            "dev-doc",
            today,
        ));

        let document = Document::new(fm, body.clone());
        crate::fidelity::verify_body_hash(&body, document.body(), format!("note {relative}"))?;

        minted.push(MintedNote { path: relative_to_root, id, number, name, tag });

        if !mode.is_dry_run() {
            store.persist(&document).map_err(|source| MigrateError::Persist { number, source })?;
        }
    }

    minted.sort_by_key(|m| m.path.clone());
    reconciled.sort_by_key(|r| r.path.clone());
    Ok(NoteReport { minted, reconciled, dry_run: mode.is_dry_run() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_number_is_recomputable_and_collision_free() {
        let taken = BTreeSet::new();
        let a = note_number("docs/dev/x.md", &taken);
        let b = note_number("docs/dev/x.md", &taken);
        assert_eq!(a, b, "recomputable from the path alone");
        assert!(a >= NOTE_NUMBER_BASE);

        let mut bumped = BTreeSet::new();
        bumped.insert(a);
        let c = note_number("docs/dev/x.md", &bumped);
        assert_ne!(a, c, "a taken candidate is bumped, never silently reused");
    }

    #[test]
    fn enumerate_dev_docs_derives_the_immediate_subdirectory_tag() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::write(root.join("direct.md"), "# Direct\n").unwrap();
        std::fs::create_dir_all(root.join("research")).unwrap();
        std::fs::write(root.join("research/nested.md"), "# Nested\n").unwrap();
        std::fs::write(root.join("index.md"), "# Index\n").unwrap();

        let docs = enumerate_dev_docs(root);
        let tag_of = |rel: &str| {
            docs.iter()
                .find(|(p, _)| p == Path::new(rel))
                .unwrap_or_else(|| panic!("{rel} not enumerated"))
                .1
                .clone()
        };
        assert_eq!(tag_of("direct.md"), None, "a direct child gets no tag");
        assert_eq!(tag_of("research/nested.md"), Some("research".to_string()));
        assert!(
            !docs.iter().any(|(p, _)| p == Path::new("index.md")),
            "index.md is excluded, same as the legacy importer"
        );
    }
}
