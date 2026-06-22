//! `odm-store` — persistence for odm: the node store is the source of truth.
//!
//! Slice 04 implements:
//!
//! - [`layout`] — the `nodes/YYYY/MM/<ULID>.md` path, a pure function of the id
//!   (so files never move on retitle/reparent and locate-by-id is O(1)).
//! - [`atomic`] — crash-safe atomic writes (temp + fsync + rename + dir-fsync).
//! - [`Store`] — persist a [`Document`](odm_core::frontmatter::Document) to its
//!   id-derived path and load nodes back (single or full scan); self-heals a
//!   missing `nodes/` directory.
//! - [`git`] — `gix`-based commit/status (pure Rust, no shelling out).
//! - [`StoreConfig`] — `odm.toml` via a layered search.
//!
//! The incremental index/cache (`odm-index`) is deliberately out of scope here;
//! this slice full-scans. Edge semantics, CRUD commands, and `check` arrive in
//! later slices.

#![deny(missing_docs)]

pub mod atomic;
pub mod git;
pub mod layout;

mod config;
mod error;
mod store;

pub use config::StoreConfig;
pub use error::{Result, StoreError};
pub use git::Repo;
pub use store::Store;
