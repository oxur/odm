//! Node types ([`NodeType`]).

use core::fmt;
use core::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The type of a node — fixed at creation, never changed (a type change is
/// modeled as supersession by a new node).
///
/// Two families share one substrate:
///
/// - **Work nodes** — [`Project`](NodeType::Project), [`Arc`](NodeType::Arc),
///   [`Slice`](NodeType::Slice): scope decomposition via the containment tree.
/// - **Document nodes** — [`Odd`](NodeType::Odd) (design doc),
///   [`Adr`](NodeType::Adr) (decision record), [`Note`](NodeType::Note).
///
/// There is deliberately **no node smaller than a slice**: a single operation
/// is always too small to deserve its own node, so the urge to drill deeper is
/// funnelled into breadth (more slices/arcs), not depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeType {
    /// A top-level body of work; decomposes into arcs.
    Project,
    /// A mid-level body of work within a project; decomposes into slices.
    Arc,
    /// The smallest unit of independently shippable work.
    Slice,
    /// A design document (ODD).
    Odd,
    /// A decision record (ADR/RFC).
    Adr,
    /// A free-form note.
    Note,
}

impl NodeType {
    /// Returns the canonical lowercase string form (the inverse of
    /// [`FromStr`](NodeType::from_str)).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            NodeType::Project => "project",
            NodeType::Arc => "arc",
            NodeType::Slice => "slice",
            NodeType::Odd => "odd",
            NodeType::Adr => "adr",
            NodeType::Note => "note",
        }
    }

    /// Returns `true` for work nodes (`project`/`arc`/`slice`).
    #[must_use]
    pub fn is_work(self) -> bool {
        matches!(self, NodeType::Project | NodeType::Arc | NodeType::Slice)
    }

    /// Returns `true` for document nodes (`odd`/`adr`/`note`).
    #[must_use]
    pub fn is_document(self) -> bool {
        matches!(self, NodeType::Odd | NodeType::Adr | NodeType::Note)
    }

    /// Returns the node types allowed as containment children of `self` in the
    /// work-decomposition tree.
    ///
    /// This encodes the `project → arc → slice` rule used later by `check`:
    /// a project's children are arcs, an arc's children are slices, and a slice
    /// has no work children. Document nodes have no work children either; how
    /// documents attach (via the `part_of` edge) is handled in a later slice.
    #[must_use]
    pub fn valid_child_types(self) -> &'static [NodeType] {
        match self {
            NodeType::Project => &[NodeType::Arc],
            NodeType::Arc => &[NodeType::Slice],
            NodeType::Slice | NodeType::Odd | NodeType::Adr | NodeType::Note => &[],
        }
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NodeType {
    type Err = ParseNodeTypeError;

    /// Parses a `NodeType` from its string form, case-insensitively.
    ///
    /// # Errors
    ///
    /// Returns [`ParseNodeTypeError`] if `s` is not one of the six known type
    /// names.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "project" => Ok(NodeType::Project),
            "arc" => Ok(NodeType::Arc),
            "slice" => Ok(NodeType::Slice),
            "odd" => Ok(NodeType::Odd),
            "adr" => Ok(NodeType::Adr),
            "note" => Ok(NodeType::Note),
            _ => Err(ParseNodeTypeError(s.to_owned())),
        }
    }
}

// Serializes as the canonical lowercase name (`"slice"`, `"odd"`, …).
impl Serialize for NodeType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NodeType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

/// The error returned when a string does not name a known [`NodeType`].
///
/// Carries the offending input for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown node type: {0:?}")]
pub struct ParseNodeTypeError(String);
