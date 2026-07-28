//! The migration-fidelity primitives both importers share (ODD-0025 §2.1/§2.2,
//! arc-migration-fidelity slice03): body normalization, the hard body-hash
//! gate, and the `source` record both importers populate at build time.
//!
//! Neither importer transforms a body — `selfhost.rs` now imports the verbatim
//! `arc-plan.md`/`slice-doc.md`/`project-plan.md` content, and `mapping.rs`
//! already imported the ODD body verbatim. The gate is therefore, by
//! construction, an **invariant check**: it verifies the body about to be
//! persisted is exactly the body that was read, so a future change that
//! reintroduces a transformation (e.g. a synthesized heading) is caught
//! immediately rather than silently reproducing the 44-stub regression.

use chrono::NaiveDate;
use odm_core::frontmatter::Source;
use sha2::{Digest as _, Sha256};

use crate::MigrateError;

/// The `source.normalization` value both importers record (ODD-0025 §2.1):
/// leading/trailing whitespace trimmed, then CRLF → LF line endings
/// normalized — nothing else. Any *internal* content change still fails the
/// hash gate; line endings are treated as a cross-platform checkout artifact,
/// not content.
pub const NORMALIZATION: &str = "trim+lf";

/// Normalizes a body per [`NORMALIZATION`]: CRLF → LF, then trim.
fn normalize_body(body: &str) -> String {
    body.replace("\r\n", "\n").trim().to_string()
}

/// The hex SHA-256 of `text`, for comparison and error messages (no hash is
/// ever stored on the node — ODD-0025 §2.1).
fn sha256_hex(text: &str) -> String {
    Sha256::digest(text.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Verifies the hard body-hash gate (ODD-0025 §2.1):
/// `sha256(normalize(source)) == sha256(normalize(node))`. `context` labels
/// the node in the error (e.g. `"#1602 (slice)"`).
///
/// # Errors
///
/// [`MigrateError::BodyHashMismatch`] if the normalized bodies hash differently.
pub fn verify_body_hash(
    source_body: &str,
    node_body: &str,
    context: impl Into<String>,
) -> Result<(), MigrateError> {
    let source_hash = sha256_hex(&normalize_body(source_body));
    let node_hash = sha256_hex(&normalize_body(node_body));
    if source_hash != node_hash {
        return Err(MigrateError::BodyHashMismatch { context: context.into() });
    }
    Ok(())
}

/// The `source.migrated_by` value both importers record: this crate's own
/// name + version, so a pre-fix import stays queryable by tool version.
#[must_use]
pub fn migrated_by() -> String {
    concat!("odm-migrate/", env!("CARGO_PKG_VERSION")).to_string()
}

/// Builds the `source` record (ODD-0025 §2.0/§2.2) a migrated node carries:
/// the source path(s), the source doc's class (odm-migrate's `DocClass`
/// vocabulary, e.g. `"arc-plan"`, `"slice-doc"`, `"odd"`), this module's
/// [`NORMALIZATION`], [`migrated_by`], and the date the migration ran.
#[must_use]
pub fn build_source(
    paths: Vec<std::path::PathBuf>,
    class: impl Into<String>,
    migrated_on: NaiveDate,
) -> Source {
    Source {
        paths,
        class: class.into(),
        normalization: NORMALIZATION.to_string(),
        migrated_by: migrated_by(),
        migrated_on,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_body_hash_passes_on_identical_bodies() {
        assert!(verify_body_hash("# Title\n\nbody\n", "# Title\n\nbody\n", "#1").is_ok());
    }

    #[test]
    fn verify_body_hash_passes_across_crlf_and_trim_differences() {
        // Only line-ending + surrounding-whitespace differences — still a pass.
        assert!(verify_body_hash("# Title\r\n\r\nbody\r\n", "  # Title\n\nbody\n  ", "#1").is_ok());
    }

    #[test]
    fn verify_body_hash_fails_on_an_internal_change() {
        let err =
            verify_body_hash("# Title\n\noriginal\n", "# Title\n\nmutated\n", "#1").unwrap_err();
        assert!(matches!(err, MigrateError::BodyHashMismatch { context } if context == "#1"));
    }

    #[test]
    fn build_source_records_the_normalization_and_tool_version() {
        let source = build_source(
            vec!["docs/design-v1.0.0/arc01-alpha/arc-plan.md".into()],
            "arc-plan",
            NaiveDate::from_ymd_opt(2026, 7, 27).unwrap(),
        );
        assert_eq!(source.normalization, "trim+lf");
        assert!(source.migrated_by.starts_with("odm-migrate/"));
        assert_eq!(source.class, "arc-plan");
    }
}
