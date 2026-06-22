//! How a node arose ([`Origin`]).

use core::fmt;
use core::str::FromStr;

/// How a node came to exist — the planning provenance of the work, distinct
/// from the [`reserved`](crate::Node::reserved) future-placeholder flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Origin {
    /// Arose from deliberate up-front planning.
    Planned,
    /// Surfaced during the work itself (not foreseen at planning time).
    Discovered,
    /// Arose from an amendment to an existing plan.
    Amendment,
}

impl Origin {
    /// Returns the canonical lowercase string form (the inverse of
    /// [`FromStr`](Origin::from_str)).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::Planned => "planned",
            Origin::Discovered => "discovered",
            Origin::Amendment => "amendment",
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Origin {
    type Err = ParseOriginError;

    /// Parses an `Origin` from its string form, case-insensitively.
    ///
    /// # Errors
    ///
    /// Returns [`ParseOriginError`] if `s` is not one of `planned`,
    /// `discovered`, or `amendment`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "planned" => Ok(Origin::Planned),
            "discovered" => Ok(Origin::Discovered),
            "amendment" => Ok(Origin::Amendment),
            _ => Err(ParseOriginError(s.to_owned())),
        }
    }
}

/// The error returned when a string does not name a known [`Origin`].
///
/// Carries the offending input for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown origin: {0:?}")]
pub struct ParseOriginError(String);
