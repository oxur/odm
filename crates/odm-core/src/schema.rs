//! The per-type, versioned file-metadata schema marker (ODD-0020).
//!
//! Each node's frontmatter carries `schema: <type>/vMAJOR.MINOR` — the on-disk
//! contract for that node *type*, versioned independently (so a change to
//! `slice`'s fields bumps `slice/v1.0 → slice/v1.1` without touching `design`). This
//! is the **file-metadata** version axis — a sibling of the output-projection
//! markers (`check/v1` …) and the binary formats (`FORMAT_VERSION`,
//! `SNAPSHOT_VERSION`), not a replacement for any of them (ODD-0020 §3).
//!
//! - The **current** schema is [`SchemaVersion::CURRENT`] (`v1.0`). New
//!   odm-created nodes stamp `<type>/v1.0`.
//! - **Absent** `schema:` ⇒ [`SchemaVersion::LEGACY`] (`v0.1`), a *computed*
//!   default on read — in practice only the legacy `docs/design` ODDs (migrate
//!   never mutates them).
//! - A node stamped with an **unknown newer** schema (e.g. `design/v1.1` read by a
//!   `v1.0` binary) is a **reported** condition ([`SchemaVersion::is_newer_than_current`]),
//!   never a silent misparse.

use core::fmt;
use core::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::NodeType;

/// A `MAJOR.MINOR` schema version. Ordered so "newer" is a plain comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion {
    /// A breaking-change bump.
    pub major: u32,
    /// An additive-change bump.
    pub minor: u32,
}

impl SchemaVersion {
    /// The current file-metadata schema (`v1.0`) — what new nodes stamp.
    pub const CURRENT: SchemaVersion = SchemaVersion { major: 1, minor: 0 };
    /// The legacy default (`v0.1`) — the *computed* version of an unversioned
    /// (pre-v1.0) node with no `schema:` field.
    pub const LEGACY: SchemaVersion = SchemaVersion { major: 0, minor: 1 };

    /// Whether this version is newer than [`CURRENT`](Self::CURRENT) — i.e. a
    /// node this binary cannot fully understand (ODD-0020 §5). A newer *minor*
    /// (additive) as well as a newer *major* (breaking) is reported: a `v1.0`
    /// reader does not know a `v1.1`'s additions, so it must not proceed silently.
    #[must_use]
    pub fn is_newer_than_current(self) -> bool {
        self > Self::CURRENT
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}", self.major, self.minor)
    }
}

impl FromStr for SchemaVersion {
    type Err = SchemaParseError;

    /// Parses `vMAJOR.MINOR` (e.g. `v1.0`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s.strip_prefix('v').ok_or_else(|| SchemaParseError(s.to_string()))?;
        let (major, minor) = rest.split_once('.').ok_or_else(|| SchemaParseError(s.to_string()))?;
        let major = major.parse().map_err(|_| SchemaParseError(s.to_string()))?;
        let minor = minor.parse().map_err(|_| SchemaParseError(s.to_string()))?;
        Ok(SchemaVersion { major, minor })
    }
}

/// The per-type schema marker: a node type plus its schema version, serialized as
/// `<type>/vMAJOR.MINOR` (e.g. `design/v1.0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaMarker {
    /// The node type the schema contract is for.
    pub node_type: NodeType,
    /// The schema version.
    pub version: SchemaVersion,
}

impl SchemaMarker {
    /// The current-schema marker for `node_type` (`<type>/v1.0`) — what a fresh
    /// odm-created node of that type stamps.
    #[must_use]
    pub fn current(node_type: NodeType) -> Self {
        Self { node_type, version: SchemaVersion::CURRENT }
    }
}

impl fmt::Display for SchemaMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.node_type.as_str(), self.version)
    }
}

impl FromStr for SchemaMarker {
    type Err = SchemaParseError;

    /// Parses `<type>/vMAJOR.MINOR` (e.g. `design/v1.0`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ty, ver) = s.split_once('/').ok_or_else(|| SchemaParseError(s.to_string()))?;
        let node_type = ty.parse::<NodeType>().map_err(|_| SchemaParseError(s.to_string()))?;
        let version = ver.parse::<SchemaVersion>()?;
        Ok(SchemaMarker { node_type, version })
    }
}

impl Serialize for SchemaMarker {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SchemaMarker {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

/// The error returned when a schema marker or version string is malformed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid schema marker {0:?}; expected `<type>/vMAJOR.MINOR` (e.g. `design/v1.0`)")]
pub struct SchemaParseError(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_orders_and_flags_newer() {
        assert!(SchemaVersion::LEGACY < SchemaVersion::CURRENT);
        assert!(!SchemaVersion::CURRENT.is_newer_than_current());
        assert!(!SchemaVersion::LEGACY.is_newer_than_current());
        assert!(SchemaVersion { major: 1, minor: 1 }.is_newer_than_current());
        assert!(SchemaVersion { major: 2, minor: 0 }.is_newer_than_current());
    }

    #[test]
    fn marker_round_trips_through_string() {
        let m = SchemaMarker::current(NodeType::Design);
        assert_eq!(m.to_string(), "design/v1.0");
        assert_eq!("design/v1.0".parse::<SchemaMarker>().unwrap(), m);
        assert_eq!(
            "slice/v1.1".parse::<SchemaMarker>().unwrap().version,
            SchemaVersion { major: 1, minor: 1 }
        );
    }

    #[test]
    fn marker_rejects_malformed() {
        assert!("design".parse::<SchemaMarker>().is_err()); // no version
        assert!("design/1.0".parse::<SchemaMarker>().is_err()); // no `v`
        assert!("bogus/v1.0".parse::<SchemaMarker>().is_err()); // unknown type
        assert!("design/v1".parse::<SchemaMarker>().is_err()); // no minor
    }
}
