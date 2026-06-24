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
//! Slice 03 adds the [`frontmatter`] module: the on-disk node format (a
//! `---`-delimited YAML frontmatter block + markdown body) and its
//! round-trip-stable parse/emit. The identity types gain `serde` impls (as
//! their canonical string forms) so they can appear in that schema; the YAML
//! backend itself is confined to the [`frontmatter`] module.
//!
//! Edge *semantics*, status/gates, persistence, and link-integrity remain out
//! of scope (later slices) — though edge and status data round-trips through
//! the schema already.

#![deny(missing_docs)]

pub mod check;
pub mod frontmatter;
pub mod graph;
mod id;
mod node;
mod node_type;
mod origin;

pub use id::{Id, IdParseError};
pub use node::Node;
pub use node_type::{NodeType, ParseNodeTypeError};
pub use origin::{Origin, ParseOriginError};
