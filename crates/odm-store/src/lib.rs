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
//! - [`StoreHome`] — where the store is (`odm.toml`'s `[store]` locator) and
//!   which file configures it (the store's `config.toml`, else the locator).
//! - [`StoreConfig`] — the operational config, parsed from whichever of those
//!   two files applies.
//! - [`rename`] — moving the home (`odm store rename`), keeping the locator
//!   and the git side in lockstep so the corpus is never lost.
//! - [`init`] — standing the home up (`odm store init`, bootstrap arm), and
//!   [`worktree`] — the one place odm shells out to the `git` binary, because
//!   `gix` cannot create a worktree (ODD-0022 §5).
//!
//! The incremental index/cache (`odm-index`) is deliberately out of scope here;
//! this slice full-scans. Edge semantics, CRUD commands, and `check` arrive in
//! later slices.

#![deny(missing_docs)]

pub mod atomic;
pub mod delta;
pub mod git;
pub mod layout;

pub mod home;
pub mod init;
pub mod rename;
pub mod worktree;

mod config;
mod error;
mod store;

pub use config::StoreConfig;
pub use delta::NodeDelta;
pub use error::{Result, StoreError};
pub use git::Repo;
pub use home::{StoreHome, StoreLocation};
pub use init::{Bootstrapped, Mode as InitMode, Plan as InitPlan};
pub use rename::{RenamePlan, Renamed};
pub use store::Store;
