//! Desired-state facts a node declares about the world (ODD-0013 §5.2).
//!
//! A node's frontmatter may carry `desired_facts`: a list of checkable claims
//! about reality. Each [`DesiredFact`] pairs a node-local `id` and a human
//! `describe` with a [`ProbeSpec`] — a `kind`-tagged description of *how* to
//! check the claim.
//!
//! This module is the **declaration** layer only: it models the facts and
//! round-trips them through the frontmatter serde layer. *Evaluating* a fact
//! against reality is the job of the `Probe` trait in `odm-reconcile` (the
//! shell probe is the first impl, arc05 slice01); the probe-runner that
//! executes a whole node's facts is slice02.
//!
//! # Serde evolution
//!
//! [`ProbeSpec`] is an **internally tagged** enum (`kind: …`), so a future probe
//! kind — `file`, slice02 — joins the set as `kind: file` without disturbing
//! the wire shape of `kind: shell`. This is the additive-evolution discipline
//! carried in from arc04: the discriminant is an explicit, always-serialized
//! field, never an `untagged` guess.

use serde::{Deserialize, Deserializer, Serialize};

/// A single checkable claim a node makes about the world (ODD-0013 §5.2).
///
/// A node's `desired_facts` is a `Vec<DesiredFact>`; absent/empty is the default
/// (a node that claims nothing). The model is uniform across node types,
/// including the **project node** — program-level acceptance facts are just
/// `desired_facts` declared there, with no separate layer (arc-plan v1.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesiredFact {
    /// Node-local identifier for the fact (e.g. `db-reachable`). Must be
    /// non-empty — an empty `id` is a positioned parse error, never a silent
    /// drop (it would name an unaddressable fact).
    #[serde(deserialize_with = "deserialize_non_empty_id")]
    pub id: String,
    /// Human-readable statement of what should be true.
    pub describe: String,
    /// How to check the claim against reality.
    pub probe: ProbeSpec,
}

/// A `kind`-tagged description of how to check a [`DesiredFact`].
///
/// Internally tagged on `kind`. slice01 defined the **shell** variant; slice02
/// adds **file**. A new kind joins as `kind: <name>` without changing how the
/// existing variants parse or emit.
///
/// Deliberately **not** `#[non_exhaustive]`: consumers (the probes, the runner)
/// match it exhaustively, so adding a kind is a *compile error* at every
/// dispatch site until it is handled — the completeness guarantee we want,
/// stronger than a silently-tolerated wildcard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ProbeSpec {
    /// Run an author-declared command and compare its result to `expect`.
    Shell {
        /// The command to run, as `program arg1 arg2 …` (whitespace-tokenized).
        /// See the shell probe's docs in `odm-reconcile` for the trust model
        /// and the no-shell-metacharacters boundary.
        run: String,
        /// The result the command must produce for the fact to *hold*.
        expect: ShellExpect,
    },
    /// Check a file (relative to the repo root) against `expect`.
    File {
        /// Path to the file, **relative to the repo root** (the odm working
        /// directory). Required.
        path: String,
        /// The expectation the file must meet. Absent in the source means "the
        /// file should exist" — see [`FileExpect`]'s defaults.
        #[serde(default)]
        expect: FileExpect,
    },
}

/// The expectation a `shell` probe's command must meet to hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellExpect {
    /// The exit code the command must return (required).
    pub exit: i32,
    /// An optional substring the command's stdout must contain. Absent means
    /// stdout is not inspected — only the exit code is compared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout_contains: Option<String>,
}

/// The expectation a `file` probe's path must meet to hold.
///
/// `exists` defaults to `true` (the common case: "this file should be there").
/// `sha256` and `size` are optional refinements — absent means that aspect is
/// not checked. Unknown sub-fields are a parse error (`deny_unknown_fields`), so
/// a typo like `sh256:` is caught rather than silently ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileExpect {
    /// Whether the file is expected to exist. Defaults to `true`; set `false` to
    /// assert a file is *absent*.
    #[serde(default = "default_true")]
    pub exists: bool,
    /// Optional expected SHA-256 of the file's contents, as 64 lowercase hex
    /// characters. Absent means the content hash is not checked.
    #[serde(
        default,
        deserialize_with = "deserialize_sha256",
        skip_serializing_if = "Option::is_none"
    )]
    pub sha256: Option<String>,
    /// Optional expected size in bytes. Absent means the size is not checked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

impl Default for FileExpect {
    fn default() -> Self {
        Self { exists: true, sha256: None, size: None }
    }
}

/// The serde default for [`FileExpect::exists`].
fn default_true() -> bool {
    true
}

/// Deserializes a fact `id`, rejecting an empty (or whitespace-only) value with
/// a positioned parse error.
///
/// The error is raised through serde during deserialization, so the YAML
/// backend attaches the offending line/column — the workspace's
/// build/parse-error-carries-position convention, applied to a semantic check
/// serde cannot express structurally.
fn deserialize_non_empty_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let id = String::deserialize(deserializer)?;
    if id.trim().is_empty() {
        return Err(serde::de::Error::custom("desired_fact `id` must not be empty"));
    }
    Ok(id)
}

/// Deserializes an optional `sha256`, rejecting a malformed digest (not 64 hex
/// characters) with a positioned parse error and normalizing to lowercase. Same
/// positioned-error discipline as [`deserialize_non_empty_id`].
fn deserialize_sha256<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(raw) = Option::<String>::deserialize(deserializer)? else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.len() == 64 && trimmed.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(Some(trimmed.to_ascii_lowercase()))
    } else {
        Err(serde::de::Error::custom("`sha256` must be 64 hexadecimal characters"))
    }
}
