//! The synthesis step (ODD-0025 §2.3, arc-migration-fidelity s11 F1/F2): a
//! node that *supersedes* many sources, verified by regime — never
//! migration (which stays strictly 1:1, hard-gated, unchanged; ODD-0025
//! §2.1/§2.3 draw this line explicitly).
//!
//! Two verification regimes, plus an escape hatch:
//!
//! - **`concatenation`** stays hash-gated against a *defined, deterministic
//!   join*: order = the caller's own `sources` order, separator =
//!   [`JOIN_SEPARATOR`], per-source normalize =
//!   [`crate::fidelity::NORMALIZATION`] — as much hard-fail as this regime
//!   allows.
//! - **`editorial-merge`** is not hash-checkable (the whole point is that a
//!   human edited/distilled, rather than just concatenated) — verified
//!   instead by *intact supersede lineage* (every source gets a forward
//!   `supersedes` edge, which `check`'s bidirectional-lineage rule then
//!   guarantees resolves, s11 F-2) plus an explicit, recorded
//!   [`Attestation`] — never a silent pass.
//! - **`other`**: neither regime applies; recorded as-is (ODD-0025 §2.3,
//!   "as needed").
//!
//! The synthesized node's `source` records `synthesis: <type>` and the
//! multi-element `source.paths` (already list-shaped since s02, ODD-0025
//! §2.2: "a list (1+) so a later synthesis... fits the same shape without a
//! model change").
//!
//! This module builds a synthesis node **in memory**; it is a capability,
//! not a live-store operation — firing a synthesis on `.worktrees/odm` is
//! s13's job (arc-migration-fidelity s11 built the general mechanism; s12
//! adds [`apply_project_vision`], the project-vision-specific application
//! of it).

use std::path::PathBuf;

use chrono::NaiveDate;
use odm_core::frontmatter::{Frontmatter, Source, SupersedeKind, Supersedes};
use odm_core::{Id, NodeType, Origin};

use crate::MigrateError;
use crate::fidelity::{NORMALIZATION, migrated_by, normalize_body, verify_body_hash};

/// The fixed separator [`deterministic_join`] places between two sources'
/// normalized bodies — part of the deterministic-join definition
/// `concatenation` is hash-gated against (ODD-0025 §2.3): order = the
/// caller-supplied `sources` order, separator = this constant, per-source
/// normalize = [`NORMALIZATION`].
pub const JOIN_SEPARATOR: &str = "\n\n";

/// A source a synthesis merges: the file it came from, its verbatim body
/// (joined for `concatenation`'s hash gate; otherwise carried only for the
/// caller's own record-keeping), and the id of the existing node it
/// supersedes.
#[derive(Debug, Clone)]
pub struct SynthesisSource {
    /// The source file path — stored in the synthesized node's
    /// `source.paths`.
    pub path: PathBuf,
    /// The verbatim source body.
    pub body: String,
    /// The existing node this source corresponds to. The synthesized node's
    /// forward `supersedes` edge points here.
    pub node: Id,
}

/// The verification regime a synthesis is checked under (ODD-0025 §2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynthesisType {
    /// Hash-gated against the deterministic join of every source.
    Concatenation,
    /// Not hash-checkable — verified by supersede lineage + an
    /// [`Attestation`].
    EditorialMerge,
    /// Neither regime applies; recorded as-is (ODD-0025 §2.3, "as needed").
    Other,
}

impl SynthesisType {
    /// The string this regime is recorded as in `source.synthesis`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            SynthesisType::Concatenation => "concatenation",
            SynthesisType::EditorialMerge => "editorial-merge",
            SynthesisType::Other => "other",
        }
    }
}

/// The explicit, recorded attestation an `editorial-merge` synthesis carries
/// (ODD-0025 §2.3) — never a silent pass, since the content itself is not
/// hash-checkable.
#[derive(Debug, Clone)]
pub struct Attestation {
    /// Who or what attested (e.g. an operator name, or a tool identity).
    pub by: String,
    /// What was attested (e.g. "faithfully distills project-plan.md §1").
    pub statement: String,
    /// The date of attestation.
    pub on: NaiveDate,
}

impl Attestation {
    /// The single recorded string [`build_synthesis`] stores in
    /// `source.attestation`.
    fn record(&self) -> String {
        format!("{} on {}: {}", self.by, self.on, self.statement)
    }
}

/// Why a synthesis could not be built — a reported error, not a panic.
#[derive(Debug, thiserror::Error)]
pub enum SynthesisError {
    /// A synthesis with no sources is meaningless (nothing to merge).
    #[error("a synthesis must have at least one source")]
    NoSources,
    /// `editorial-merge` was requested without a recorded attestation.
    #[error("an editorial-merge synthesis requires a recorded attestation")]
    MissingAttestation,
    /// A `concatenation` synthesis's body did not hash-match the
    /// deterministic join of its sources.
    #[error(transparent)]
    Fidelity(#[from] MigrateError),
}

/// The deterministic join [`SynthesisType::Concatenation`] is hash-gated
/// against (ODD-0025 §2.3): each source's body normalized
/// ([`NORMALIZATION`]) individually, then joined in the caller-supplied
/// `sources` order with [`JOIN_SEPARATOR`].
#[must_use]
pub fn deterministic_join(sources: &[SynthesisSource]) -> String {
    sources.iter().map(|s| normalize_body(&s.body)).collect::<Vec<_>>().join(JOIN_SEPARATOR)
}

/// Builds a synthesized node's [`Frontmatter`]: the forward `supersedes`
/// edges to every source (ODD-0025 §2.3's bidirectional-lineage requirement
/// — `check`'s rule then guarantees they resolve, s11 F-2), plus a `source`
/// record carrying `synthesis: <type>` and the multi-element `paths`,
/// verified per `synthesis_type`'s regime. The caller pairs the returned
/// frontmatter with `body` to persist (mirroring
/// [`crate::mapping::build_node`]'s shape) — this function performs no I/O.
///
/// - `Concatenation`: `body` must hash-match [`deterministic_join`] of
///   `sources`, byte-for-byte after normalization — the hard gate.
/// - `EditorialMerge`: `attestation` must be `Some` — recorded in
///   `source.attestation`. The lineage half of the invariant is the
///   `supersedes` edges this function always attaches; `check`'s rule (F-2)
///   verifies they resolve, not re-checked here.
/// - `Other`: no automatic verification (ODD-0025 §2.3, "as needed").
///
/// # Errors
///
/// [`SynthesisError::NoSources`] if `sources` is empty;
/// [`SynthesisError::MissingAttestation`] if `synthesis_type` is
/// `EditorialMerge` and `attestation` is `None`;
/// [`SynthesisError::Fidelity`] if `synthesis_type` is `Concatenation` and
/// `body` does not hash-match the deterministic join.
#[allow(clippy::too_many_arguments)]
pub fn build_synthesis(
    id: Id,
    number: u32,
    node_type: NodeType,
    name: String,
    created: NaiveDate,
    updated: NaiveDate,
    body: &str,
    sources: &[SynthesisSource],
    synthesis_type: SynthesisType,
    supersede_kind: SupersedeKind,
    attestation: Option<&Attestation>,
    class: impl Into<String>,
    migrated_on: NaiveDate,
) -> Result<Frontmatter, SynthesisError> {
    if sources.is_empty() {
        return Err(SynthesisError::NoSources);
    }

    match synthesis_type {
        SynthesisType::Concatenation => {
            let joined = deterministic_join(sources);
            verify_body_hash(&joined, body, format!("#{number} concatenation synthesis"))?;
        }
        SynthesisType::EditorialMerge => {
            if attestation.is_none() {
                return Err(SynthesisError::MissingAttestation);
            }
        }
        SynthesisType::Other => {}
    }

    let mut fm = Frontmatter::new(id, number, node_type, name, created, updated, Origin::Planned);
    fm.edges_mut().supersedes =
        sources.iter().map(|s| Supersedes { node: s.node, kind: supersede_kind }).collect();

    fm = fm.with_source(Source {
        paths: sources.iter().map(|s| s.path.clone()).collect(),
        class: class.into(),
        normalization: NORMALIZATION.to_string(),
        migrated_by: migrated_by(),
        migrated_on,
        synthesis: Some(synthesis_type.as_str().to_string()),
        attestation: attestation.map(Attestation::record),
    });

    Ok(fm)
}

/// Why [`apply_project_vision`] could not build the vision synthesis.
#[derive(Debug, thiserror::Error)]
pub enum VisionApplyError {
    /// `plan_root`'s `project-plan.md` has no `# Definition of done` section
    /// for [`crate::replan::vision_from_plan`] to extract.
    #[error("no vision text found in {0}'s project-plan.md (no Definition-of-done section)")]
    NoVisionText(std::path::PathBuf),
    /// The synthesis itself could not be built (see [`SynthesisError`]).
    #[error(transparent)]
    Synthesis(#[from] SynthesisError),
}

/// The project-vision re-cast (MF-7's live half; arc-migration-fidelity s12
/// F-4): builds the **editorial-merge synthesis** node that supersedes the
/// 1:1 `project-plan` node, using [`crate::replan::vision_from_plan`] for the
/// distillation and [`build_synthesis`] for the verified construction —
/// replacing `replan::restamp`'s bespoke body-patch (which injected the
/// vision text directly into the project node's own body, violating the 1:1
/// rule ODD-0025 §2.1 requires of a *migrated* node) with the modeled
/// mechanism.
///
/// Takes the **already-persisted** 1:1 `project_plan` node's identity,
/// canonical `source.paths` entry, and verbatim body — this function does
/// not mint or verify that node; the caller (`self_host`, in the live path)
/// already did. `plan_root` is read only to extract the vision text.
///
/// This performs no I/O beyond that one read, and never persists anything —
/// firing it live (deciding the vision node's id/number and writing both
/// documents) is s13's job.
///
/// # Errors
///
/// [`VisionApplyError::NoVisionText`] if `plan_root`'s `project-plan.md` has
/// no `Definition of done` section; [`VisionApplyError::Synthesis`] if the
/// synthesis itself can't be built (e.g. a missing attestation).
#[allow(clippy::too_many_arguments)]
pub fn apply_project_vision(
    project_plan_id: Id,
    project_plan_source_path: std::path::PathBuf,
    project_plan_body: &str,
    plan_root: &std::path::Path,
    vision_id: Id,
    vision_number: u32,
    created: NaiveDate,
    updated: NaiveDate,
    attestation: Attestation,
) -> Result<(Frontmatter, String), VisionApplyError> {
    let vision_body = crate::replan::vision_from_plan(plan_root)
        .ok_or_else(|| VisionApplyError::NoVisionText(plan_root.to_path_buf()))?;

    let source = SynthesisSource {
        path: project_plan_source_path,
        body: project_plan_body.to_string(),
        node: project_plan_id,
    };
    let fm = build_synthesis(
        vision_id,
        vision_number,
        NodeType::Project,
        "Vision".to_string(),
        created,
        updated,
        &vision_body,
        std::slice::from_ref(&source),
        SynthesisType::EditorialMerge,
        SupersedeKind::Updates,
        Some(&attestation),
        "vision",
        updated,
    )?;
    Ok((fm, vision_body))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 7, 29).unwrap()
    }

    fn a_source(body: &str) -> SynthesisSource {
        SynthesisSource {
            path: PathBuf::from("docs/a.md"),
            body: body.to_string(),
            node: Id::new(),
        }
    }

    // ----- F-3: synthesis records type + multi-path source -----------------

    #[test]
    fn synthesis_records_type_and_multi_path_source() {
        let sources = vec![
            a_source("first\n"),
            SynthesisSource { path: "docs/b.md".into(), ..a_source("second\n") },
        ];
        let joined = deterministic_join(&sources);
        let fm = build_synthesis(
            Id::new(),
            1,
            NodeType::Design,
            "Synth".to_string(),
            day(),
            day(),
            &joined,
            &sources,
            SynthesisType::Concatenation,
            SupersedeKind::Obsoletes,
            None,
            "synthesis",
            day(),
        )
        .expect("valid concatenation");

        let source = fm.source().expect("source present");
        assert_eq!(source.synthesis.as_deref(), Some("concatenation"));
        assert_eq!(source.paths.len(), 2, "multi-element source.paths");
        assert_eq!(source.paths, vec![PathBuf::from("docs/a.md"), PathBuf::from("docs/b.md")]);
    }

    // ----- F-1/F-2: the synthesis writes forward supersedes edges ----------

    #[test]
    fn synthesis_writes_a_forward_supersede_edge_per_source() {
        let sources = vec![a_source("x\n"), a_source("y\n")];
        let joined = deterministic_join(&sources);
        let fm = build_synthesis(
            Id::new(),
            1,
            NodeType::Design,
            "Synth".to_string(),
            day(),
            day(),
            &joined,
            &sources,
            SynthesisType::Concatenation,
            SupersedeKind::Obsoletes,
            None,
            "synthesis",
            day(),
        )
        .unwrap();

        let targets: Vec<Id> = fm.edges().supersedes.iter().map(|s| s.node).collect();
        assert_eq!(targets, vec![sources[0].node, sources[1].node]);
    }

    // ----- F-4: concatenation is hard-gated against the deterministic join -

    #[test]
    fn concatenation_passes_on_a_faithful_join() {
        let sources = vec![a_source("first\n"), a_source("second\n")];
        let joined = deterministic_join(&sources);
        assert!(
            build_synthesis(
                Id::new(),
                1,
                NodeType::Design,
                "Synth".to_string(),
                day(),
                day(),
                &joined,
                &sources,
                SynthesisType::Concatenation,
                SupersedeKind::Obsoletes,
                None,
                "synthesis",
                day(),
            )
            .is_ok()
        );
    }

    #[test]
    fn concatenation_fails_on_a_tampered_body() {
        let sources = vec![a_source("first\n"), a_source("second\n")];
        let tampered = "first\n\nsecond, but someone edited this\n";
        let err = build_synthesis(
            Id::new(),
            1,
            NodeType::Design,
            "Synth".to_string(),
            day(),
            day(),
            tampered,
            &sources,
            SynthesisType::Concatenation,
            SupersedeKind::Obsoletes,
            None,
            "synthesis",
            day(),
        )
        .unwrap_err();
        assert!(matches!(err, SynthesisError::Fidelity(MigrateError::BodyHashMismatch { .. })));
    }

    #[test]
    fn deterministic_join_is_order_sensitive_and_normalizes_each_source() {
        let a =
            SynthesisSource { path: "a.md".into(), body: "  A  \r\n".to_string(), node: Id::new() };
        let b = SynthesisSource { path: "b.md".into(), body: "B\n".to_string(), node: Id::new() };
        assert_eq!(deterministic_join(&[a.clone(), b.clone()]), "A\n\nB");
        assert_eq!(deterministic_join(&[b, a]), "B\n\nA", "order is part of the definition");
    }

    // ----- F-5: editorial-merge requires lineage + attestation -------------

    #[test]
    fn editorial_merge_requires_an_attestation() {
        let sources = vec![a_source("anything — not hash-checked\n")];
        let err = build_synthesis(
            Id::new(),
            1,
            NodeType::Design,
            "Vision".to_string(),
            day(),
            day(),
            "a human-distilled body, unrelated to the source text",
            &sources,
            SynthesisType::EditorialMerge,
            SupersedeKind::Updates,
            None,
            "vision",
            day(),
        )
        .unwrap_err();
        assert!(matches!(err, SynthesisError::MissingAttestation));
    }

    #[test]
    fn editorial_merge_is_accepted_with_lineage_and_attestation() {
        let sources = vec![a_source("the full source text\n")];
        let attestation = Attestation {
            by: "operator".to_string(),
            statement: "faithfully distills the source's definition-of-done".to_string(),
            on: day(),
        };
        let fm = build_synthesis(
            Id::new(),
            1,
            NodeType::Design,
            "Vision".to_string(),
            day(),
            day(),
            "a human-distilled body, unrelated to the source text",
            &sources,
            SynthesisType::EditorialMerge,
            SupersedeKind::Updates,
            Some(&attestation),
            "vision",
            day(),
        )
        .expect("lineage + attestation present");

        assert_eq!(fm.edges().supersedes.len(), 1, "lineage: the forward edge is attached");
        let source = fm.source().unwrap();
        assert_eq!(source.synthesis.as_deref(), Some("editorial-merge"));
        assert_eq!(
            source.attestation.as_deref(),
            Some("operator on 2026-07-29: faithfully distills the source's definition-of-done")
        );
    }

    // ----- edge cases ---------------------------------------------------------

    #[test]
    fn a_synthesis_with_no_sources_is_rejected() {
        let err = build_synthesis(
            Id::new(),
            1,
            NodeType::Design,
            "Empty".to_string(),
            day(),
            day(),
            "",
            &[],
            SynthesisType::Other,
            SupersedeKind::Obsoletes,
            None,
            "synthesis",
            day(),
        )
        .unwrap_err();
        assert!(matches!(err, SynthesisError::NoSources));
    }

    #[test]
    fn other_regime_performs_no_automatic_verification() {
        let sources = vec![a_source("whatever\n")];
        let fm = build_synthesis(
            Id::new(),
            1,
            NodeType::Design,
            "Other".to_string(),
            day(),
            day(),
            "anything at all — not verified",
            &sources,
            SynthesisType::Other,
            SupersedeKind::Obsoletes,
            None,
            "synthesis",
            day(),
        )
        .expect("`other` performs no verification");
        assert_eq!(fm.source().unwrap().synthesis.as_deref(), Some("other"));
    }
}
