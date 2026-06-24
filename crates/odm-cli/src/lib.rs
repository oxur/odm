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

use std::process::ExitCode;

use anyhow::Context as _;
use clap::{Parser, Subcommand, ValueEnum};
use odm_core::frontmatter::SupersedeKind;
use odm_store::Store;

use crate::commands::{EXIT_OK, UseKind};

/// Exit code for a usage or operational error (clap also uses `2` for argument
/// errors). Distinct from `1`, which `check` reserves for "ran, found
/// violations".
const EXIT_ERROR: u8 = 2;

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
    /// Validate the whole graph: schema, links, cycles, recomposition, order.
    Check {
        /// CI mode: promote warnings (staleness, soft-satisfaction) to failures.
        #[arg(long)]
        strict: bool,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show the ready frontier (nodes whose dependencies are satisfied).
    Next {
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Explain why a node is blocked or low-confidence.
    Blocked {
        /// A node id, number, or unique name prefix.
        reference: String,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show a dependency path: the critical chain from X, or a path X → Y.
    Path {
        /// The start node (id, number, or unique name prefix).
        reference: String,
        /// Optional destination node.
        to: Option<String>,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
}

/// Parses arguments and dispatches, rooted at the current working directory,
/// writing to the process's stdout/stderr, and returns the process exit code.
///
/// Exit codes: `0` success (and, for `check`, a clean corpus); `1` `check`
/// found violations; `2` an operational error (or, via `clap`, an argument
/// error).
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let root = match std::env::current_dir().context("determining the current directory") {
        Ok(root) => root,
        Err(e) => {
            eprintln!("error: {e:#}");
            return ExitCode::from(EXIT_ERROR);
        }
    };
    match dispatch(cli, &root, &mut std::io::stdout(), &mut std::io::stderr()) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(EXIT_ERROR)
        }
    }
}

/// Dispatches a parsed [`Cli`] against a store rooted at `root`, writing query
/// results to `out` and diagnostics to `err`, and returns the intended exit
/// code (`0` ok / clean, `1` `check` violations).
///
/// This is the in-process entry point: [`run`] wires `out`/`err` to
/// stdout/stderr, and tests wire them to buffers with an explicit `root` (no
/// global current-directory mutation).
///
/// # Errors
///
/// Returns an [`anyhow::Error`] (which [`run`] maps to exit code `2`) if the
/// command fails — e.g. an unknown reference, a type mismatch, or an I/O/store
/// error.
pub fn dispatch(
    cli: Cli,
    root: &std::path::Path,
    out: &mut dyn std::io::Write,
    err: &mut dyn std::io::Write,
) -> anyhow::Result<u8> {
    let store = Store::open(root);

    match cli.command {
        Command::New { node_type, name, dry_run, yes: _ } => {
            commands::new(&store, &node_type, &name, dry_run, err)?;
        }
        Command::List { node_type, tag, component, json } => commands::list(
            &store,
            node_type.as_deref(),
            tag.as_deref(),
            component.as_deref(),
            json,
            out,
        )?,
        Command::Show { reference, json } => commands::show(&store, &reference, json, out)?,
        Command::Rename { reference, name, dry_run, yes: _ } => {
            commands::rename(&store, &reference, &name, dry_run, err)?;
        }
        Command::Retire { reference, because, dry_run, yes: _ } => {
            commands::retire(&store, &reference, &because, dry_run, err)?;
        }
        Command::Supersede { reference, with, kind, dry_run, yes: _ } => {
            commands::supersede(&store, &reference, &with, kind.into(), dry_run, err)?;
        }
        Command::Use { kind, reference } => {
            commands::use_context(&store, root, kind.into(), &reference, err)?;
        }
        Command::Context { json } => commands::context(&store, root, json, out)?,
        // `check` returns its own exit code (0 clean / 1 violations).
        Command::Check { strict, json } => return commands::check(&store, root, strict, json, out),
        Command::Next { json } => commands::next(&store, root, json, out)?,
        Command::Blocked { reference, json } => {
            commands::blocked(&store, root, &reference, json, out)?;
        }
        Command::Path { reference, to, json } => {
            commands::path(&store, root, &reference, to.as_deref(), json, out)?;
        }
    }
    Ok(EXIT_OK)
}
