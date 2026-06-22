//! Stable node identity ([`Id`]).

use core::fmt;
use core::str::FromStr;

use chrono::{DateTime, Utc};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ulid::Ulid;

/// A node's stable identity: a [ULID](https://github.com/ulid/spec) assigned
/// once at creation and **never reused or renumbered**.
///
/// `Id` is the *only* identity in the model — all edges reference ids. It is
/// deliberately **not** interconvertible with a node's human-facing
/// [`number`](crate::Node::number): there is no `From<u32>` or any other numeric
/// constructor, so an `Id` can never be confused with a number.
///
/// ULIDs embed their creation time in their most-significant bits, so the
/// derived [`Ord`] agrees with creation order (see [`Id::new`]). Planning
/// order, however, is never *derived* from the id — that is the dependency
/// graph's job.
///
/// # Examples
///
/// ```
/// use odm_core::Id;
///
/// let id = Id::new();
/// let round_tripped: Id = id.to_string().parse()?;
/// assert_eq!(id, round_tripped);
/// # Ok::<(), odm_core::IdParseError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(Ulid);

impl Id {
    /// Mints a fresh, unique `Id` from the current time plus randomness.
    ///
    /// Every call returns a new value; ids are never recycled. Two ids minted
    /// in different milliseconds compare in creation order; ordering within the
    /// same millisecond is unspecified.
    #[must_use]
    pub fn new() -> Self {
        Self(Ulid::new())
    }

    /// The creation time encoded in the ULID, as a UTC timestamp.
    ///
    /// A ULID embeds the millisecond at which it was minted, so this needs no
    /// stored field. The store uses it to derive a node's `nodes/YYYY/MM/` path
    /// from the id alone (the path is a pure function of the id).
    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        // A ULID's 48-bit millisecond timestamp is always within chrono's
        // representable range; the epoch fallback keeps this total (no panic).
        DateTime::from_timestamp_millis(self.0.timestamp_ms() as i64).unwrap_or_default()
    }
}

impl Default for Id {
    /// Equivalent to [`Id::new`] — the default `Id` is a freshly minted one.
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Id {
    /// Renders the id as its 26-character Crockford base32 ULID form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Id {
    type Err = IdParseError;

    /// Parses an `Id` from its 26-character Crockford base32 ULID form.
    ///
    /// # Errors
    ///
    /// Returns [`IdParseError::InvalidLength`] if `s` is not 26 characters, or
    /// [`IdParseError::InvalidChar`] if it contains a non-base32 character.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Ulid::from_string(s) {
            Ok(ulid) => Ok(Self(ulid)),
            Err(ulid::DecodeError::InvalidLength) => Err(IdParseError::InvalidLength),
            Err(ulid::DecodeError::InvalidChar) => Err(IdParseError::InvalidChar),
        }
    }
}

// An `Id` serializes as its canonical ULID string — the form that appears in
// frontmatter and git diffs. This keeps the on-disk representation stable and
// human-readable, and means no serde format ever sees the raw `u128`.
impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

/// The error returned when an [`Id`] cannot be parsed from a string.
///
/// This is a self-contained type: it does not expose the underlying ULID
/// library, so that dependency stays an implementation detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IdParseError {
    /// The string was not the expected 26 characters long.
    #[error("invalid id: expected a 26-character ULID")]
    InvalidLength,
    /// The string contained a character outside Crockford base32.
    #[error("invalid id: contains an invalid character")]
    InvalidChar,
}
