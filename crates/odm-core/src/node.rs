//! The minimal node skeleton ([`Node`]).

use crate::{Id, NodeType, Origin};

/// A node in the odm graph, reduced to its identity-bearing core.
///
/// This is the slice-02 skeleton: it carries identity ([`id`](Node::id)), the
/// human handle ([`number`](Node::number)), [`node_type`](Node::node_type),
/// [`name`](Node::name), [`origin`](Node::origin), and the
/// [`reserved`](Node::reserved) flag. Dates, edges, status/gates, and tags are
/// added in later slices and are intentionally absent here.
///
/// # Identity stability
///
/// The [`id`](Node::id) is assigned once by [`Node::new`] and is **immutable**:
/// renaming the node or changing its `number` never touches it. There is no
/// setter for `id` and no way to derive one from a `number`.
///
/// # Examples
///
/// ```
/// use odm_core::{Node, NodeType, Origin};
///
/// let mut node = Node::new(7, NodeType::Slice, "Identity core", Origin::Planned, false);
/// let id = node.id();
///
/// node.set_name("Stable identity core");
/// node.set_number(8);
/// assert_eq!(node.id(), id); // identity is unchanged by edits
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    id: Id,
    number: u32,
    node_type: NodeType,
    name: String,
    origin: Origin,
    reserved: bool,
}

impl Node {
    /// Creates a node with a freshly minted [`Id`].
    ///
    /// The `number` is a human handle (display and CLI), independent of
    /// identity. `name` is the human label; it never affects identity. Set
    /// `reserved` to `true` for a tentative future-work placeholder that is not
    /// yet real work.
    pub fn new(
        number: u32,
        node_type: NodeType,
        name: impl Into<String>,
        origin: Origin,
        reserved: bool,
    ) -> Self {
        Self { id: Id::new(), number, node_type, name: name.into(), origin, reserved }
    }

    /// The node's stable, immutable identity.
    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }

    /// The node's human-facing number (metadata, not identity).
    #[must_use]
    pub fn number(&self) -> u32 {
        self.number
    }

    /// The node's type, fixed at creation.
    #[must_use]
    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    /// The node's human label.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// How the node arose.
    #[must_use]
    pub fn origin(&self) -> Origin {
        self.origin
    }

    /// Whether the node is a tentative future-work placeholder.
    #[must_use]
    pub fn reserved(&self) -> bool {
        self.reserved
    }

    /// Renames the node. Does not affect [`id`](Node::id).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Changes the node's human number. Does not affect [`id`](Node::id).
    pub fn set_number(&mut self, number: u32) {
        self.number = number;
    }
}
