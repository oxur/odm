//! The canonical author-owned **metadata partial** (arc-store-as-source
//! slice03, ODD-0026 §2.5, ODD-0013 v2.6's author-vs-odm field boundary).
//!
//! **odm owns the seam.** Every field an author can supply through this type
//! is author-owned (per ODD-0013 v2.6's boundary); every odm-owned field
//! (`id`, `number`, placement, the `source`/provenance record) has **no
//! field here at all** — there is nothing for an author to set, correctly or
//! incorrectly. Combined with `#[serde(deny_unknown_fields)]`, an attempt to
//! set one (e.g. a stray `id = "..."` in a hand-written metadata file) is a
//! **hard parse error**, not a value this type could silently accept and
//! `node new`/`node set` then have to separately reject — the boundary is
//! enforced by the type's shape, not by a second validation pass.
//!
//! **One canonical form, two wire formats.** `node new`/`node set --metadata`
//! accepts the partial as JSON *or* TOML, dispatched on file extension; both
//! deserialize into this identical type, so an equivalent partial produces an
//! identical node regardless of which format authored it (JSON favors LLM
//! emission; TOML matches odm's own `odm.toml`/`config.toml` idiom for a
//! human author).
//!
//! `name`/`type` are **not** fields here — `node new <type> <name>` already
//! takes them as positional arguments (unchanged CLI shape), so the partial
//! covers exactly the fields that shape doesn't: `part_of`, `tags`, and
//! `status` (intent).

use std::path::Path;

use anyhow::{Context as _, anyhow};
use serde::Deserialize;

/// The author-owned fields a metadata partial may carry. See the module doc
/// for why odm-owned fields have no field here at all.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataPartial {
    /// The containment parent, as a reference (id, number, or a unique name
    /// prefix) — resolved to an [`odm_core::Id`] by the caller, the same way
    /// `node new --parent`/`node link` already do.
    #[serde(default)]
    pub part_of: Option<String>,
    /// Filter tags — replaces the node's tag list wholesale when applied via
    /// `node set` (no add/remove semantics; the whole list is author-owned).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Status **intent**: a gate name to record as reached, at
    /// [`odm_core::status::Evidence::Asserted`] — the lightest evidence
    /// tier, matching "intent" rather than a verified claim. Reuses
    /// [`odm_core::status::Status::set_gate`] (the same mechanism
    /// `odm set-gate` already validates through) rather than inventing a
    /// second status representation; `odm set-gate` remains the way to
    /// record a *stronger* evidence level.
    #[serde(default)]
    pub status: Option<String>,
}

/// Loads a metadata partial from `path`, dispatched on extension (`.json` →
/// `serde_json`, `.toml` → `toml`) into the one canonical [`MetadataPartial`]
/// form.
///
/// # Errors
///
/// Returns an error if the file cannot be read, its extension is neither
/// `.json` nor `.toml`, or its content fails to parse — including a
/// **rejected odm-owned field** (an unknown top-level key such as `id` or
/// `source`), which `#[serde(deny_unknown_fields)]` turns into a parse
/// error naming the offending key.
pub fn load(path: &Path) -> anyhow::Result<MetadataPartial> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading metadata partial {}", path.display()))?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => serde_json::from_str(&text)
            .with_context(|| format!("parsing {} as JSON metadata", path.display())),
        Some("toml") => toml::from_str(&text)
            .with_context(|| format!("parsing {} as TOML metadata", path.display())),
        _ => {
            Err(anyhow!("{} has no recognized extension — use `.json` or `.toml`", path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_and_toml_partials_deserialize_identically() {
        let dir = tempfile::TempDir::new().unwrap();
        let json_path = dir.path().join("meta.json");
        let toml_path = dir.path().join("meta.toml");
        std::fs::write(&json_path, r#"{"part_of": "5", "tags": ["a", "b"], "status": "planned"}"#)
            .unwrap();
        std::fs::write(
            &toml_path,
            "part_of = \"5\"\ntags = [\"a\", \"b\"]\nstatus = \"planned\"\n",
        )
        .unwrap();

        let from_json = load(&json_path).unwrap();
        let from_toml = load(&toml_path).unwrap();
        assert_eq!(from_json, from_toml);
        assert_eq!(from_json.part_of.as_deref(), Some("5"));
        assert_eq!(from_json.tags, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(from_json.status.as_deref(), Some("planned"));
    }

    #[test]
    fn an_odm_owned_field_is_a_hard_parse_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("meta.json");
        std::fs::write(&path, r#"{"id": "01ARZ3NDEKTSV4RRFFQ69G5FAV"}"#).unwrap();
        let err = load(&path).unwrap_err();
        assert!(format!("{err:#}").contains("id"), "{err:#}");
    }

    #[test]
    fn an_unrecognized_extension_is_rejected() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("meta.yaml");
        std::fs::write(&path, "part_of: 5\n").unwrap();
        let err = load(&path).unwrap_err();
        assert!(err.to_string().contains("recognized extension"), "{err}");
    }

    #[test]
    fn an_empty_partial_defaults_every_field() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("meta.json");
        std::fs::write(&path, "{}").unwrap();
        assert_eq!(load(&path).unwrap(), MetadataPartial::default());
    }
}
