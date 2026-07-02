//! `odm-reconcile` — desired-state facts probed against reality (ODD-0013 §5.2).
//!
//! A node declares [`DesiredFact`](odm_core::DesiredFact)s in its frontmatter
//! (modeled in `odm-core`). This crate checks those facts against the world:
//!
//! - [`Probe`] / [`ProbeOutcome`] — the trait an individual check implements and
//!   the **three-way** result (`Holds` / `Drifted` / `Error`); "couldn't check"
//!   is never collapsed into "checked, drifted".
//! - [`ShellProbe`] (slice01) — run an author-declared command, compare exit (and
//!   optional stdout) to the expectation.
//! - [`FileProbe`] (slice02) — check a file against `exists` / `sha256` / `size`.
//! - [`Runner`] (slice02) — execute a node's (or the whole corpus's)
//!   `desired_facts` and collect per-fact outcomes, preserving drift-vs-error in
//!   the aggregate so slice03 can assign severities and exit codes.
//!
//! Scope held to slice02: this is **library** API only. No `odm reconcile`
//! command, no `--json`, no exit codes, no rollup/orient wiring (slices 03–04).
//!
//! # Reading `desired_facts` — from the store, not the index
//!
//! [`Runner`] reads frontmatter directly from the [`Store`](odm_store::Store), it
//! does **not** route facts through the A4 index. Reconcile is an infrequent,
//! I/O-bound action whose cost is the probes themselves; a frontmatter read is
//! noise beside a subprocess spawn or a file hash. The heavy nested
//! `desired_facts` structure is therefore not added to every index snapshot
//! record (the same call A4 made for the `list --json` full-node dump).
//! Consequently the carried adapter-fidelity invariant is honored **by
//! non-triggering**: the field is read where it lives, so the index equivalence
//! guarantee is untouched. See the slice02 slice-doc "central design decision"
//! for the full rationale.
//!
//! (The guarded index identifiers — the snapshot record type, the index-read
//! seam, the format-version constant — are deliberately kept *out* of this
//! crate's source so the ledger G-5 grep stays a meaningful tripwire against a
//! future accidental switch to the index path.)

#![deny(missing_docs)]

mod file;
mod incremental;
mod probe;
mod runner;
mod shell;
mod snapshot;

pub use file::FileProbe;
pub use incremental::{IncrementalError, Mode, reconcile, reconcile_full, reconcile_incremental};
pub use probe::{Probe, ProbeOutcome};
pub use runner::{CorpusReport, FactResult, NodeReport, OutcomeCounts, Runner};
pub use shell::ShellProbe;
pub use snapshot::{
    DriftError, DriftSnapshot, FactEntry, FactState, InputFingerprint, Load, RebuildReason,
};
