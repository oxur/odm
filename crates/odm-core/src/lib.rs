//! `odm-core` — the odm domain model.
//!
//! This crate holds the pure value types every other crate builds on. Slice 02
//! (this slice) implements the **identity primitives** only:
//!
//! - [`Id`] — a node's stable, never-reused [ULID](https://github.com/ulid/spec)
//!   identity.
//! - [`NodeType`] — the closed set of node types (`project`/`arc`/`slice` work
//!   nodes; `odd`/`adr`/`note` document nodes) — no node smaller than a slice.
//! - [`Origin`] — how a node arose (`planned`/`discovered`/`amendment`).
//! - [`Node`] — a minimal skeleton tying the above together with a human
//!   `number`, `name`, and `reserved` flag.
//!
//! Serialization, the frontmatter schema, edges, status/gates, and persistence
//! are explicitly **out of scope** here; they arrive in later slices. These
//! types carry no `serde` derives — the on-disk format is slice 03's decision.

#![deny(missing_docs)]

mod id;
mod node;
mod node_type;
mod origin;

pub use id::{Id, IdParseError};
pub use node::Node;
pub use node_type::{NodeType, ParseNodeTypeError};
pub use origin::{Origin, ParseOriginError};
