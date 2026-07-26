//! The **taxonomy re-stamp** (RH C-2): rewriting already-imported document
//! nodes from the pre-C-2 `odd` type to `design`/`research`.
//!
//! ## Why this cannot be an ordinary typed pass
//!
//! ODD-0020 §4 chose a **hard re-stamp with no legacy read-alias**: `"odd"` is
//! gone from [`NodeType`], so it no longer parses. That is the right end state,
//! but it means an on-disk `type: odd` node cannot be *loaded* — and a migration
//! that cannot read its input cannot run. Every other schema upgrade
//! (`backfill_schema`) reads typed nodes and rewrites them; this one has to get
//! the corpus parseable first.
//!
//! So the re-stamp does the minimum raw-text work to unblock parsing — it
//! rewrites the two frontmatter lines that carry the dead type name, `type:` and
//! `schema:` — and then hands off to the ordinary typed path for everything
//! else. It is a line rewrite, not a YAML round-trip: nothing else in the file
//! is touched by it, and the subsequent `persist` re-emits the whole node in
//! canonical order.
//!
//! ## Which type a node becomes
//!
//! The same rule the importer uses for a fresh import
//! ([`crate::mapping::classify_type`]): `research` iff the tags include
//! `research`, else `design`. The **source document's** tags win over the node's
//! own, because a node's tags were copied at import time and may predate a
//! correction to the source — which is exactly the case for the corpus this
//! runs on (two docs carried a `change-me` placeholder). When no source doc
//! matches the node's number, the node's own tags are used.
//!
//! This runs once per corpus; on a re-stamped corpus every node is skipped.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use odm_core::NodeType;
use odm_core::frontmatter::Document;
use odm_store::Store;

use crate::mapping::classify_type;
use crate::{MigrateError, Mode, Upgraded};

/// The dead type name this pass exists to remove.
const LEGACY_TYPE: &str = "odd";

/// What a re-stamp pass did, or (under [`Mode::DryRun`]) would do.
#[derive(Debug, Default)]
pub struct Restamp {
    /// One entry per node re-stamped, for the migration report.
    pub upgraded: Vec<Upgraded>,
    /// The rewritten documents, by node path.
    ///
    /// Under [`Mode::Commit`] these are already on disk and this is only a
    /// record. Under [`Mode::DryRun`] nothing was written, so it is also the
    /// **only** readable form of those nodes: the files still carry the dead
    /// `odd` type and will not parse. [`crate::migrate_with_gates`] overlays
    /// them so the rest of the preview sees the corpus as it *would* be.
    pub rewritten: HashMap<PathBuf, Document>,
}

/// Re-stamps every `type: odd` node under `store` to `design`/`research`,
/// refreshing its tags from the matching legacy source document when one is
/// available, and returns what was (or, under [`Mode::DryRun`], would be)
/// re-stamped.
///
/// Idempotent: a corpus with no `odd` node left is a no-op. `nodes/` is
/// odm-owned, so this rewrites node files in place — the never-mutate rule
/// applies to the legacy `docs/design` tree, **not** to `nodes/`.
///
/// # Errors
///
/// [`MigrateError`] on a store read/persist failure. A node file that is
/// unreadable, or that still does not parse after the type rewrite, is skipped
/// rather than raised: this pass must not be able to wedge a corpus.
pub fn restamp_taxonomy(
    store: &Store,
    sources: &[SourceTags],
    mode: Mode,
) -> Result<Restamp, MigrateError> {
    let mut restamp = Restamp::default();
    for path in store.node_paths().map_err(MigrateError::LoadCorpus)? {
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        if !carries_legacy_type(&text) {
            continue; // already on the current taxonomy — idempotent skip
        }

        // The source doc's tags decide the type; fall back to the node's own.
        let number = frontmatter_number(&text);
        let source = number.and_then(|n| sources.iter().find(|s| s.number == n));
        let node_type = match source {
            Some(source) => classify_type(&source.tags),
            None => classify_type(&node_tags(&text)),
        };

        let rewritten = rewrite_type_lines(&text, node_type);
        let Ok(mut document) = Document::parse(&rewritten) else { continue };

        // Everything from here is typed and canonical.
        document.frontmatter_mut().stamp_schema();
        if let Some(source) = source
            && !source.tags.is_empty()
        {
            let refreshed = document.frontmatter().clone().with_tags(source.tags.clone());
            *document.frontmatter_mut() = refreshed;
        }

        let (number, id, name, schema) = {
            let fm = document.frontmatter();
            let marker = fm.schema().expect("just stamped");
            (fm.number(), fm.id(), fm.name().to_string(), marker.to_string())
        };
        if !mode.is_dry_run() {
            store.persist(&document).map_err(|source| MigrateError::Persist { number, source })?;
        }
        restamp.upgraded.push(Upgraded { number, id, name, schema });
        restamp.rewritten.insert(path, document);
    }
    restamp.upgraded.sort_by_key(|u| u.number);
    Ok(restamp)
}

/// A legacy source document's identity and tags — the input the re-stamp needs
/// in order to classify a node it can only partially read.
#[derive(Debug, Clone)]
pub struct SourceTags {
    /// The legacy number, which is also the node's `number`.
    pub number: u32,
    /// The source document's `tags`.
    pub tags: Vec<String>,
}

impl SourceTags {
    /// Collects `(number, tags)` from every legacy document under
    /// `legacy_path` that carries a number. Unreadable or unnumbered files are
    /// skipped — they are reported by the import pass, not here.
    #[must_use]
    pub fn collect(legacy_path: &Path) -> Vec<Self> {
        crate::legacy::discover(legacy_path)
            .into_iter()
            .filter_map(|path: PathBuf| crate::legacy::parse_file(&path).ok())
            .filter_map(|doc| {
                doc.front.number.map(|number| Self { number, tags: doc.front.tags.clone() })
            })
            .collect()
    }
}

/// Whether the frontmatter declares the dead `odd` type.
fn carries_legacy_type(text: &str) -> bool {
    frontmatter_lines(text).any(|line| line.trim_end() == format!("type: {LEGACY_TYPE}"))
}

/// The node's `number`, read straight from the frontmatter text.
fn frontmatter_number(text: &str) -> Option<u32> {
    frontmatter_lines(text)
        .find_map(|line| line.strip_prefix("number:"))
        .and_then(|v| v.trim().parse().ok())
}

/// The node's own `tags`, read from the frontmatter's block sequence (the form
/// odm emits). Used only when no source document matches.
fn node_tags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_tags = false;
    for line in frontmatter_lines(text) {
        if line.trim_end() == "tags:" {
            in_tags = true;
            continue;
        }
        if in_tags {
            match line.strip_prefix("- ") {
                Some(tag) => tags.push(tag.trim().to_string()),
                None => break,
            }
        }
    }
    tags
}

/// The lines of the frontmatter block (between the opening and closing fences).
fn frontmatter_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split('\n').skip(1).take_while(|line| *line != "---")
}

/// Rewrites the `type:` and `schema:` lines that name the dead type, leaving
/// every other line — and the body — byte-identical.
fn rewrite_type_lines(text: &str, node_type: NodeType) -> String {
    let new_type = node_type.as_str();
    let mut out: Vec<String> = Vec::new();
    let mut in_frontmatter = false;
    for (i, line) in text.split('\n').enumerate() {
        if i == 0 || line == "---" {
            in_frontmatter = i == 0;
            out.push(line.to_string());
            continue;
        }
        let rewritten = match (in_frontmatter, line.trim_end()) {
            (true, l) if l == format!("type: {LEGACY_TYPE}") => format!("type: {new_type}"),
            // The version is re-stamped typed afterwards; only the type name
            // has to change here, and only so the marker parses.
            (true, l) if l.starts_with(&format!("schema: {LEGACY_TYPE}/")) => {
                l.replacen(LEGACY_TYPE, new_type, 1)
            }
            _ => line.to_string(),
        };
        out.push(rewritten);
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const NODE: &str = "---\nid: 01KWWGS8HD25CQE3BX5FQEW84Q\nnumber: 11\ntype: odd\nschema: odd/v1.0\nname: 'Research: a thing'\ncreated: 2026-06-20\nupdated: 2026-06-20\ntags:\n- change-me\ncomponent: All\norigin: planned\nreserved: false\n---\n# Research: a thing\n\ntype: odd in the body must survive.\n";

    #[test]
    fn test_carries_legacy_type_detects_the_dead_type() {
        assert!(carries_legacy_type(NODE));
        assert!(!carries_legacy_type(&NODE.replace("type: odd\n", "type: design\n")));
    }

    #[test]
    fn test_frontmatter_number_reads_the_number() {
        assert_eq!(frontmatter_number(NODE), Some(11));
    }

    #[test]
    fn test_node_tags_reads_the_block_sequence() {
        assert_eq!(node_tags(NODE), vec!["change-me".to_string()]);
    }

    #[test]
    fn test_rewrite_type_lines_rewrites_only_the_frontmatter() {
        let out = rewrite_type_lines(NODE, NodeType::Research);
        assert!(out.contains("type: research\n"), "type rewritten: {out}");
        assert!(out.contains("schema: research/v1.0\n"), "marker rewritten: {out}");
        assert!(
            out.contains("type: odd in the body must survive."),
            "the body is untouched: {out}"
        );
        assert!(out.contains("number: 11\n"), "other fields untouched: {out}");
    }

    #[test]
    fn test_rewrite_type_lines_output_parses_and_keeps_identity() {
        let out = rewrite_type_lines(NODE, NodeType::Design);
        let doc = Document::parse(&out).expect("rewritten node parses");
        assert_eq!(doc.frontmatter().node_type(), NodeType::Design);
        assert_eq!(doc.frontmatter().number(), 11);
        assert_eq!(doc.frontmatter().id().to_string(), "01KWWGS8HD25CQE3BX5FQEW84Q");
    }

    #[test]
    fn test_rewrite_type_lines_is_a_no_op_on_a_current_node() {
        let current = NODE.replace("type: odd", "type: design").replace("odd/v1.0", "design/v1.0");
        assert_eq!(rewrite_type_lines(&current, NodeType::Design), current);
    }
}
