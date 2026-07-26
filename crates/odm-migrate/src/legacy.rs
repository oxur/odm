//! Reading the **legacy** ODD corpus: numbered markdown files in state-directories
//! (`01-draft`…`10-superseded`) with the oxur-era frontmatter
//! `{ number, title, author, component, tags, created, updated, state, supersedes,
//! superseded-by, version }`.
//!
//! This module is read-only — it never writes, moves, or mutates a legacy file
//! (M-5). The YAML backend (`serde_norway`) is confined here, mirroring
//! `odm-core`'s isolation of it behind its `frontmatter` module.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use serde::Deserialize;
use walkdir::WalkDir;

/// The legacy frontmatter fields this importer reads. Unknown keys (e.g.
/// `version`) are ignored by `serde` — the legacy file is preserved intact, so
/// nothing is lost. All fields are optional so a malformed doc parses into a
/// value we can *report on* rather than panicking (M-6); the required-field
/// checks live in the mapping.
#[derive(Debug, Clone, Deserialize)]
pub struct LegacyFrontmatter {
    /// The legacy human number (identity in the legacy model). Required by the
    /// mapping; its absence is a reported skip.
    pub number: Option<u32>,
    /// The document title → the node `name`.
    #[serde(default)]
    pub title: Option<String>,
    /// The document author → carried into the node's `extra` (no typed field).
    #[serde(default)]
    pub author: Option<String>,
    /// The subsystem/component label → the node `component`.
    #[serde(default)]
    pub component: Option<String>,
    /// Free-form filter labels → the node `tags`.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Creation date → the node `created`.
    #[serde(default)]
    pub created: Option<NaiveDate>,
    /// Last-updated date → the node `updated` + gate/retire date.
    #[serde(default)]
    pub updated: Option<NaiveDate>,
    /// The legacy `DocState` scalar → the document gate position or a retirement.
    #[serde(default)]
    pub state: Option<String>,
    /// The legacy number this doc supersedes → a `supersedes` edge on this node.
    /// Accepts `null`, a bare number, or a human ref string (`"ODD-0011"` /
    /// `"11"`) — see [`de_opt_ref`] (slice02: the real corpus uses `null`, but the
    /// human-facing `ODD-NN` convention makes the string form plausible).
    #[serde(default, deserialize_with = "de_opt_ref")]
    pub supersedes: Option<u32>,
    /// The legacy number that supersedes this doc → a `supersedes` edge on *that*
    /// node (reverse-derived, ODD-0013 §9). Same shapes as [`supersedes`].
    ///
    /// [`supersedes`]: LegacyFrontmatter::supersedes
    #[serde(default, rename = "superseded-by", deserialize_with = "de_opt_ref")]
    pub superseded_by: Option<u32>,
}

/// Deserializes a legacy document reference that may be `null`, a bare number,
/// or a human string ref (`"ODD-0011"`, `"odd-11"`, `"0011"`, `"11"`) into the
/// referenced legacy `number`. A non-empty string that carries no number is a
/// hard error (a *loud* malformed skip, never a silent `None` that would drop the
/// supersession intent).
fn de_opt_ref<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    /// The accepted on-disk shapes (an absent/`null` value is `None` before this).
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Num(u32),
        Str(String),
    }
    match Option::<Raw>::deserialize(deserializer)? {
        None => Ok(None),
        Some(Raw::Num(n)) => Ok(Some(n)),
        Some(Raw::Str(s)) => first_number(&s).map(Some).ok_or_else(|| {
            serde::de::Error::custom(format!("reference {s:?} carries no document number"))
        }),
    }
}

/// The first contiguous run of ASCII digits in `s`, parsed as a `u32`
/// (`"ODD-0011"` → 11, `"11"` → 11, `"none"` → `None`).
fn first_number(s: &str) -> Option<u32> {
    let start = s.find(|c: char| c.is_ascii_digit())?;
    let rest = &s[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// One legacy document: its parsed frontmatter, its markdown body (carried
/// verbatim onto the new node), and the source path (for error reporting).
#[derive(Debug, Clone)]
pub struct LegacyDoc {
    /// The parsed legacy frontmatter.
    pub front: LegacyFrontmatter,
    /// The markdown body, verbatim.
    pub body: String,
    /// Where it was read from (for positioned errors / the report).
    pub path: PathBuf,
}

/// An error reading a legacy file (kept distinct from a *mapping* skip: this is
/// I/O or a frontmatter parse failure).
#[derive(Debug, thiserror::Error)]
pub enum LegacyError {
    /// The file could not be read.
    #[error("reading {path}: {source}")]
    Io {
        /// The file that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
    /// The frontmatter block was missing, unterminated, or invalid YAML — the
    /// message carries the position the YAML backend reports.
    #[error("{path}: {message}")]
    Frontmatter {
        /// The offending file.
        path: PathBuf,
        /// A human, position-carrying description.
        message: String,
    },
}

/// The frontmatter delimiter line (same as the node format).
const FENCE: &str = "---";

/// Discovers the legacy `*.md` documents under `root`, in sorted path order.
///
/// Skips non-document files: an `index.md` at any level, and anything under a
/// dotted directory (`.dustbin`, `.odm`, `.oxd`) or a `templates/` directory.
/// A missing `root` yields an empty list (not an error) — the caller reports it.
#[must_use]
pub fn discover(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        // Check exclusions on the path *relative to root* — the root itself may
        // contain `.`/`..` components (a relative fixture path) that must not be
        // mistaken for hidden directories.
        let relative = path.strip_prefix(root).unwrap_or(path);
        if is_excluded(relative) {
            continue;
        }
        paths.push(path.to_path_buf());
    }
    paths.sort();
    paths
}

/// Whether a path is a non-document file the importer skips (index files,
/// templates, dot-directories).
fn is_excluded(path: &Path) -> bool {
    if path.file_name().is_some_and(|n| n == "index.md") {
        return true;
    }
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "templates" || s.starts_with('.')
    })
}

/// Reads and parses one legacy document from `path`.
///
/// # Errors
///
/// [`LegacyError::Io`] if the file cannot be read, [`LegacyError::Frontmatter`]
/// if it has no valid `---`-delimited YAML frontmatter block.
pub fn parse_file(path: &Path) -> Result<LegacyDoc, LegacyError> {
    let text = std::fs::read_to_string(path)
        .map_err(|source| LegacyError::Io { path: path.to_path_buf(), source })?;
    let (yaml, body) = split_frontmatter(&text)
        .map_err(|message| LegacyError::Frontmatter { path: path.to_path_buf(), message })?;
    let front: LegacyFrontmatter = serde_norway::from_str(yaml).map_err(|e| {
        LegacyError::Frontmatter { path: path.to_path_buf(), message: e.to_string() }
    })?;
    Ok(LegacyDoc { front, body: body.to_string(), path: path.to_path_buf() })
}

/// Splits `---`-delimited frontmatter from the body, returning `(yaml, body)`.
/// The body is everything after the first closing fence (so a body may itself
/// contain `---`). Mirrors `odm_core::frontmatter::Document::parse`'s split.
fn split_frontmatter(text: &str) -> Result<(&str, &str), String> {
    let rest = text
        .strip_prefix(FENCE)
        .and_then(|r| r.strip_prefix('\n'))
        .ok_or("missing opening '---' frontmatter delimiter")?;
    // Find the closing fence: a line that is exactly `---`.
    let mut search_from = 0;
    while let Some(rel) = rest[search_from..].find(FENCE) {
        let idx = search_from + rel;
        let at_line_start = idx == 0 || rest.as_bytes()[idx - 1] == b'\n';
        let after = &rest[idx + FENCE.len()..];
        let ends_line = after.is_empty() || after.starts_with('\n');
        if at_line_start && ends_line {
            let yaml = &rest[..idx];
            let body = after.strip_prefix('\n').unwrap_or(after);
            return Ok((yaml, body));
        }
        search_from = idx + FENCE.len();
    }
    Err("unterminated frontmatter: missing closing '---'".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frontmatter_extracts_yaml_and_body() {
        let text = "---\nnumber: 7\nstate: Draft\n---\n# Title\n\nBody --- with fence.\n";
        let (yaml, body) = split_frontmatter(text).unwrap();
        assert!(yaml.contains("number: 7"));
        assert!(body.starts_with("# Title"));
        assert!(body.contains("Body --- with fence."), "body keeps internal fences");
    }

    #[test]
    fn split_frontmatter_rejects_missing_delimiters() {
        assert!(split_frontmatter("no frontmatter here").is_err());
        assert!(split_frontmatter("---\nnumber: 7\nno closing fence").is_err());
    }

    #[test]
    fn first_number_extracts_a_ref_number() {
        assert_eq!(first_number("ODD-0011"), Some(11));
        assert_eq!(first_number("odd-11"), Some(11));
        assert_eq!(first_number("0011"), Some(11));
        assert_eq!(first_number("11"), Some(11));
        assert_eq!(first_number("none"), None);
        assert_eq!(first_number(""), None);
    }

    #[test]
    fn de_opt_ref_accepts_null_number_and_string() {
        #[derive(serde::Deserialize)]
        struct Holder {
            #[serde(default, deserialize_with = "de_opt_ref")]
            r: Option<u32>,
        }
        let null: Holder = serde_norway::from_str("r: null").unwrap();
        assert_eq!(null.r, None);
        let num: Holder = serde_norway::from_str("r: 7").unwrap();
        assert_eq!(num.r, Some(7));
        let s: Holder = serde_norway::from_str("r: \"ODD-0011\"").unwrap();
        assert_eq!(s.r, Some(11));
        // A string with no number is a hard error (loud), never a silent None.
        assert!(serde_norway::from_str::<Holder>("r: \"none\"").is_err());
    }

    #[test]
    fn is_excluded_skips_index_templates_and_dotdirs() {
        assert!(is_excluded(Path::new("docs/design/index.md")));
        assert!(is_excluded(Path::new("docs/design/templates/x.md")));
        assert!(is_excluded(Path::new("docs/design/.dustbin/x.md")));
        assert!(!is_excluded(Path::new("docs/design/01-draft/0013-arch.md")));
    }
}
