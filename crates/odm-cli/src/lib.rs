//! `odm-cli` — the clap command surface for odm's node CRUD.
//!
//! Commands are named after the question, not the mechanism. Queries (`list`,
//! `show`, `context`) print data to stdout and accept `--json`; mutators
//! (`new`, `rename`, `retire`, `supersede`, `use`) accept `--dry-run` (write
//! nothing) and `--yes` (run non-interactively), and report to stderr.
//!
//! The store root is the current working directory; `odm.toml` and the node
//! tree are resolved from there.

#![deny(missing_docs)]

mod commands;
mod context;

use anyhow::Context as _;
use clap::{Parser, Subcommand, ValueEnum};
use odm_core::frontmatter::SupersedeKind;
use odm_store::Store;

use crate::commands::UseKind;

/// The `odm` command-line interface.
#[derive(Debug, Parser)]
#[command(name = "odm", version, about = "The Odd Document Manager")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The kind of node `use` selects.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum UseKindArg {
    /// Select the current project.
    Project,
    /// Select the current arc.
    Arc,
}

impl From<UseKindArg> for UseKind {
    fn from(value: UseKindArg) -> Self {
        match value {
            UseKindArg::Project => UseKind::Project,
            UseKindArg::Arc => UseKind::Arc,
        }
    }
}

/// The supersession kind for `supersede --kind`.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum KindArg {
    /// The old node is replaced.
    Obsoletes,
    /// The old node is amended (still relevant).
    Updates,
}

impl From<KindArg> for SupersedeKind {
    fn from(value: KindArg) -> Self {
        match value {
            KindArg::Obsoletes => SupersedeKind::Obsoletes,
            KindArg::Updates => SupersedeKind::Updates,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a node (idempotent: re-running describes rather than duplicating).
    New {
        /// Node type: project|arc|slice|odd|adr|note.
        node_type: String,
        /// Human-readable name.
        name: String,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively (no confirmation prompt).
        #[arg(long)]
        yes: bool,
    },
    /// List nodes, optionally filtered.
    List {
        /// Filter by node type.
        #[arg(long = "type")]
        node_type: Option<String>,
        /// Filter by tag.
        #[arg(long)]
        tag: Option<String>,
        /// Filter by component.
        #[arg(long)]
        component: Option<String>,
        /// Emit JSON instead of a table.
        #[arg(long)]
        json: bool,
    },
    /// Show a node, its edges, and its way-finding (parent + children).
    Show {
        /// A node id, number, or unique name prefix.
        reference: String,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Rename a node (name only — id and path are unchanged).
    Rename {
        /// A node id, number, or unique name prefix.
        reference: String,
        /// The new name.
        name: String,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
    },
    /// Retire a node (withdraw it; the file is preserved, never deleted).
    Retire {
        /// A node id, number, or unique name prefix.
        reference: String,
        /// Why the node is being retired.
        #[arg(long)]
        because: String,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
    },
    /// Record that one node supersedes another.
    Supersede {
        /// The node being superseded (id, number, or name prefix).
        reference: String,
        /// The node that supersedes it (id, number, or name prefix).
        #[arg(long = "with")]
        with: String,
        /// Whether the old node is obsoleted (replaced) or merely updated.
        #[arg(long)]
        kind: KindArg,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
    },
    /// Set the current project or arc context.
    Use {
        /// Which slot to set.
        kind: UseKindArg,
        /// A node id, number, or unique name prefix.
        reference: String,
    },
    /// Show the current project/arc context.
    Context {
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
}

/// Parses arguments and dispatches, rooted at the current working directory,
/// writing to the process's stdout/stderr.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the command fails. `clap` handles
/// argument-parse errors itself, exiting the process directly.
pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let root = std::env::current_dir().context("determining the current directory")?;
    dispatch(cli, &root, &mut std::io::stdout(), &mut std::io::stderr())
}

/// Dispatches a parsed [`Cli`] against a store rooted at `root`, writing query
/// results to `out` and diagnostics to `err`.
///
/// This is the in-process entry point: [`run`] wires `out`/`err` to
/// stdout/stderr, and tests wire them to buffers with an explicit `root` (no
/// global current-directory mutation).
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the command fails (e.g. an unknown
/// reference, a type mismatch, or an I/O/store error).
pub fn dispatch(
    cli: Cli,
    root: &std::path::Path,
    out: &mut dyn std::io::Write,
    err: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    let store = Store::open(root);

    match cli.command {
        Command::New { node_type, name, dry_run, yes: _ } => {
            commands::new(&store, &node_type, &name, dry_run, err)
        }
        Command::List { node_type, tag, component, json } => commands::list(
            &store,
            node_type.as_deref(),
            tag.as_deref(),
            component.as_deref(),
            json,
            out,
        ),
        Command::Show { reference, json } => commands::show(&store, &reference, json, out),
        Command::Rename { reference, name, dry_run, yes: _ } => {
            commands::rename(&store, &reference, &name, dry_run, err)
        }
        Command::Retire { reference, because, dry_run, yes: _ } => {
            commands::retire(&store, &reference, &because, dry_run, err)
        }
        Command::Supersede { reference, with, kind, dry_run, yes: _ } => {
            commands::supersede(&store, &reference, &with, kind.into(), dry_run, err)
        }
        Command::Use { kind, reference } => {
            commands::use_context(&store, root, kind.into(), &reference, err)
        }
        Command::Context { json } => commands::context(&store, root, json, out),
    }
}
