//! `odm-cli` — the clap command surface for odm's node CRUD.
//!
//! Commands are named after the question, not the mechanism. Queries (`list`,
//! `show`, `context`) print data to stdout and accept `--json`; mutators
//! (`new`, `rename`, `retire`, `supersede`, `use`) accept `--dry-run` (write
//! nothing) and `--yes` (run non-interactively), and report to stderr.
//!
//! The invocation is rooted at the current working directory. From there
//! `odm.toml` is found and its `[store]` locator resolved, which is what says
//! where the node tree actually lives (ODD-0022 §4.2) — the repo root itself
//! when no `[store]` section directs otherwise.

#![deny(missing_docs)]

mod commands;
mod context;
mod json;
mod listview;
mod metadata;
mod migrate;
mod orient;
mod reconcile;
mod rollup;
mod store_cmd;
mod table;
mod term;

use std::process::ExitCode;

use anyhow::Context as _;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use odm_core::frontmatter::SupersedeKind;
use odm_core::status::Evidence;
use odm_store::{Store, StoreHome};

use crate::commands::{EXIT_OK, LinkEdge, UseKind};

/// Exit code for a usage or operational error (clap also uses `2` for argument
/// errors). Distinct from `1`, which `check` reserves for "ran, found
/// violations".
const EXIT_ERROR: u8 = 2;

/// The `odm` command-line interface.
///
/// The subcommand is optional: bare `odm` runs `orient` (it never bare-errors).
#[derive(Debug, Parser)]
#[command(name = "odm", version, about = "The Odd Document Manager")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// Which family of nodes `list` shows.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum GroupArg {
    /// The plan: `project`/`arc`/`slice`, plus any `artifact` `part_of` a
    /// slice.
    Plan,
    /// The reference material: `design`/`research`/`adr`/`note`, and any
    /// `artifact` not `part_of` a slice.
    Reference,
}

impl From<GroupArg> for crate::listview::Group {
    fn from(value: GroupArg) -> Self {
        match value {
            GroupArg::Plan => crate::listview::Group::Plan,
            GroupArg::Reference => crate::listview::Group::Reference,
        }
    }
}

/// Which date `list`'s leading column shows.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum DateArg {
    /// The node's creation date (the default).
    Created,
    /// The node's last-updated date.
    Updated,
}

impl From<DateArg> for crate::listview::DateColumn {
    fn from(value: DateArg) -> Self {
        match value {
            DateArg::Created => crate::listview::DateColumn::Created,
            DateArg::Updated => crate::listview::DateColumn::Updated,
        }
    }
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

/// The edge kind `link`/`unlink` operates on (the source-stored edges; reverse
/// edges are derived, never written, so they are not selectable here).
#[derive(Debug, Clone, Copy, ValueEnum)]
enum LinkEdgeArg {
    /// Ordering dependency (optionally `--satisfied-at <gate>`).
    #[value(name = "depends_on")]
    DependsOn,
    /// Hard external block.
    #[value(name = "blocked_by")]
    BlockedBy,
    /// Consumes a concrete output/artifact.
    #[value(name = "consumes")]
    Consumes,
    /// Verifies the target.
    #[value(name = "verifies")]
    Verifies,
    /// Affects the target's docs.
    #[value(name = "affects")]
    Affects,
    /// Containment parent (single-parent: replaces any existing parent).
    #[value(name = "part_of")]
    PartOf,
}

impl From<LinkEdgeArg> for LinkEdge {
    fn from(value: LinkEdgeArg) -> Self {
        match value {
            LinkEdgeArg::DependsOn => LinkEdge::DependsOn,
            LinkEdgeArg::BlockedBy => LinkEdge::BlockedBy,
            LinkEdgeArg::Consumes => LinkEdge::Consumes,
            LinkEdgeArg::Verifies => LinkEdge::Verifies,
            LinkEdgeArg::Affects => LinkEdge::Affects,
            LinkEdgeArg::PartOf => LinkEdge::PartOf,
        }
    }
}

/// The edge kind `tear` operates on (only `depends_on` is tearable — §4.3).
#[derive(Debug, Clone, Copy, ValueEnum)]
enum TearEdgeArg {
    /// The only tearable edge kind.
    #[value(name = "depends_on")]
    DependsOn,
}

/// The evidence level for `set-gate --evidence`.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum EvidenceArg {
    /// Claimed, no verification (the default).
    Asserted,
    /// Someone else's verification, relayed.
    Attested,
    /// Independently reproduced.
    Reproduced,
    /// Reconciled against observed reality.
    Reconciled,
}

impl From<EvidenceArg> for Evidence {
    fn from(value: EvidenceArg) -> Self {
        match value {
            EvidenceArg::Asserted => Evidence::Asserted,
            EvidenceArg::Attested => Evidence::Attested,
            EvidenceArg::Reproduced => Evidence::Reproduced,
            EvidenceArg::Reconciled => Evidence::Reconciled,
        }
    }
}

/// The output format for `rollup` (RH F-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum FormatArg {
    /// Markdown — the committed, diffable representation.
    Md,
    /// JSON, for a machine consumer.
    Json,
}

impl FormatArg {
    /// The file extension this format writes.
    fn extension(self) -> &'static str {
        match self {
            FormatArg::Md => "md",
            FormatArg::Json => "json",
        }
    }
}

/// The `odm store …` operations.
#[derive(Debug, Subcommand)]
enum StoreCommand {
    /// Create or refresh the store's home: a worktree holding an orphan branch.
    ///
    /// Three arms, chosen from what is already there: **bootstrap** when the
    /// branch exists nowhere, **attach** when a remote has it (check it out —
    /// never re-orphan, never re-scaffold, since that would overwrite a
    /// teammate's store with defaults), and **ff-sync** when it is already
    /// local. Divergence stops rather than merging or rebasing: odm does not
    /// rewrite history other clones already hold.
    Init {
        /// The worktree directory name (default `odm`).
        #[arg(long, value_name = "NAME")]
        worktree: Option<String>,
        /// The orphan branch name (default `odm`).
        #[arg(long, value_name = "NAME")]
        branch: Option<String>,
        /// Report the plan and write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
        /// Emit JSON describing the created home.
        #[arg(long)]
        json: bool,
    },
    /// Rename the store's worktree directory and/or its branch.
    ///
    /// Distinct from `odm rename`, which renames a *node*: this moves where the
    /// store lives, keeping the `[store]` locator in step so the corpus is
    /// never lost.
    Rename {
        /// Rename both the worktree and the branch to this name.
        #[arg(value_name = "NEW")]
        new: Option<String>,
        /// Rename only the worktree directory.
        #[arg(long, value_name = "NEW")]
        worktree: Option<String>,
        /// Rename only the local branch.
        #[arg(long, value_name = "NEW")]
        branch: Option<String>,
        /// Report the plan and change nothing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
        /// Emit JSON describing the rename.
        #[arg(long)]
        json: bool,
    },
    /// Persist the store worktree's pending node changes as a commit on the
    /// orphan branch.
    ///
    /// The verb that closes the `init → mutate → ???` gap: `migrate`/`node`
    /// write files into the worktree but never commit them, so without this
    /// the only way to persist the store was raw `git`. The default message
    /// is a node delta (created/modified/removed, by type); `-m` overrides
    /// it. A clean worktree is a no-op — exit `0`, no empty commit.
    Commit {
        /// Override the auto-generated node-delta summary.
        #[arg(short = 'm', long, value_name = "MSG")]
        message: Option<String>,
        /// Report the delta and message; write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Emit JSON describing the commit (or the no-op / dry-run outcome).
        #[arg(long)]
        json: bool,
    },
    /// Read-only view of the store's git state: the pending node delta (what
    /// `commit` would write) and ahead/behind vs. the configured upstream.
    ///
    /// The read half of the same coin as `commit` — reuses its delta
    /// computation and `init`'s remote-tracking plumbing, but never writes:
    /// no commit, no fetch, no index or worktree mutation. The upstream
    /// comparison reflects whatever the last fetch left behind (`sync` is
    /// what fetches); a store with no upstream reports that, not an error.
    Status {
        /// Emit JSON describing the pending delta and upstream state.
        #[arg(long)]
        json: bool,
    },
}

/// The `odm node …` operations — node entity management (ODD-0023 §4).
#[derive(Debug, Subcommand)]
enum NodeCommand {
    /// Create a node (idempotent: re-running describes rather than duplicating).
    ///
    /// Mints `origin: authored` (arc-store-as-source slice02/03, ODD-0026 §2.1/
    /// §2.5) — odm owns `id`/`number`/placement/`source`; the author supplies
    /// only the body (pure markdown) and/or a metadata partial (author-owned
    /// fields), never the fused `---`-plus-body file directly.
    New {
        /// Node type: project|arc|slice|design|research|adr|note|artifact.
        node_type: String,
        /// Human-readable name.
        name: String,
        /// Set `part_of` to this parent (id, number, or unique name prefix).
        /// Takes precedence over `--metadata`'s `part_of`, if both are given.
        #[arg(long)]
        parent: Option<String>,
        /// The author-owned metadata partial (`part_of`/`tags`/`status`),
        /// loaded from a `.json` or `.toml` file — see [`crate::metadata`].
        #[arg(long)]
        metadata: Option<String>,
        /// The body, read verbatim from a markdown file.
        #[arg(long, conflicts_with_all = ["body", "from_file"])]
        content: Option<String>,
        /// The body, read verbatim from a markdown file — an alias for
        /// `--content` (ODD-0026 §2.5's own naming).
        #[arg(long = "from-file", conflicts_with_all = ["content", "body"])]
        from_file: Option<String>,
        /// The body, as a literal inline string.
        #[arg(long, conflicts_with_all = ["content", "from_file"])]
        body: Option<String>,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Emit JSON (the created/would-be-created node).
        #[arg(long)]
        json: bool,
        /// Proceed non-interactively (no confirmation prompt).
        #[arg(long)]
        yes: bool,
    },
    /// Set one author-owned metadata field (odm owns the `---` seam — this
    /// never opens the fused file).
    ///
    /// Field-addressed, re-validated (schema + the author-vs-odm boundary)
    /// before any write. Valid fields: `name`, `tags` (comma-separated,
    /// replaces the list wholesale), `part_of` (a reference), `status` (a
    /// gate name — recorded at `asserted` evidence via the same mechanism
    /// `odm set-gate` uses; `set-gate` remains the way to record a stronger
    /// evidence level).
    Set {
        /// The node (id, number, or unique name prefix).
        reference: String,
        /// The field to set: `name` | `tags` | `part_of` | `status`.
        field: String,
        /// The new value.
        value: String,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Emit JSON (the updated node).
        #[arg(long)]
        json: bool,
        /// Proceed non-interactively (no confirmation prompt).
        #[arg(long)]
        yes: bool,
    },
    /// Replace a node's whole body as pure markdown (odm re-attaches the
    /// unchanged frontmatter — the seam invariant: only the body changes).
    SetBody {
        /// The node (id, number, or unique name prefix).
        reference: String,
        /// The new body, read verbatim from a markdown file.
        #[arg(long, conflicts_with = "body")]
        from_file: Option<String>,
        /// The new body, as a literal inline string.
        #[arg(long, conflicts_with = "from_file")]
        body: Option<String>,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Emit JSON (the updated node).
        #[arg(long)]
        json: bool,
        /// Proceed non-interactively (no confirmation prompt).
        #[arg(long)]
        yes: bool,
    },
    /// List nodes as a plan: date, type, status, containment tree.
    ///
    /// Retired and superseded nodes are omitted by default — they are not live
    /// work; `--all` brings them back, marked in the STATUS column.
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
        /// Which date the leading column shows.
        #[arg(long, value_name = "WHICH", default_value = "created")]
        date: DateArg,
        /// Max width of the NAME column; longer names are elided with ` ...`.
        /// Defaults to `[display] max_width` in `odm.toml`.
        #[arg(long, value_name = "COLS")]
        width: Option<usize>,
        /// Show only one family: the `plan` (project/arc/slice) or the
        /// `reference` material (design/research/adr/note).
        #[arg(long, value_name = "FAMILY")]
        group: Option<GroupArg>,
        /// Show only nodes whose STATUS is this value (e.g. `retired`,
        /// `tested`). Implies `--all` when the value is a withdrawn one, so
        /// `--status retired` shows exactly what a default listing withholds.
        #[arg(long, value_name = "VALUE")]
        status: Option<String>,
        /// Include retired and superseded nodes.
        #[arg(long, visible_alias = "include-retired")]
        all: bool,
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
    /// Add an edge on the source node (reverse is derived, never written).
    Link {
        /// The source node (id, number, or unique name prefix).
        source: String,
        /// The edge kind.
        edge: LinkEdgeArg,
        /// The target node (id, number, or unique name prefix).
        target: String,
        /// For `depends_on`: the gate at which the dependency is satisfied.
        #[arg(long = "satisfied-at")]
        satisfied_at: Option<String>,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
    },
    /// Remove an edge from the source node (absent edge → a clear no-op).
    Unlink {
        /// The source node (id, number, or unique name prefix).
        source: String,
        /// The edge kind.
        edge: LinkEdgeArg,
        /// The target node (id, number, or unique name prefix).
        target: String,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
    },
    /// Record that a node has reached a gate (validated against its gate-set).
    SetGate {
        /// The node (id, number, or unique name prefix).
        reference: String,
        /// The gate name (must be in the node type's gate-set).
        gate: String,
        /// Who recorded reaching it.
        #[arg(long)]
        by: Option<String>,
        /// The evidence level (defaults to `asserted`).
        #[arg(long, default_value = "asserted")]
        evidence: EvidenceArg,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
    },
    /// Affirm that a parent's children fully account for its scope (§4.5).
    Decomposed {
        /// The parent node (id, number, or unique name prefix).
        reference: String,
        /// The affirmed children; if omitted, the node's current children.
        #[arg(long, num_args = 1.., value_name = "REF")]
        children: Vec<String>,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
    },
    /// Declare a deliberately-assumed dependency edge (breaks a cycle).
    Tear {
        /// The source node (id, number, or unique name prefix).
        source: String,
        /// The edge kind (only `depends_on` is tearable).
        edge: TearEdgeArg,
        /// The target node (id, number, or unique name prefix).
        target: String,
        /// Why the dependency is being assumed (required).
        #[arg(long)]
        because: String,
        /// Show what would happen without writing.
        #[arg(long)]
        dry_run: bool,
        /// Proceed non-interactively.
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Orient: vision → current focus → ready/blocked → integrity → drift.
    ///
    /// The default command — bare `odm` runs this. `brief` is an alias.
    #[command(visible_alias = "brief")]
    Orient {
        /// Emit JSON instead of the human-readable view.
        #[arg(long)]
        json: bool,
    },
    /// Regenerate the plan rollup: the single cheap view of the whole plan.
    ///
    /// Writes `<out>.<ext>` at the invocation root (default `ROLLUP.md`). The
    /// format decides the extension, so `--format json` writes `ROLLUP.json`
    /// (RH F-13 — the filename is a default, not a law).
    Rollup {
        /// Render to stdout without writing the file.
        #[arg(long)]
        dry_run: bool,
        /// Output format.
        #[arg(long, value_name = "FORMAT", default_value = "md")]
        format: FormatArg,
        /// Output file stem, without extension (default `ROLLUP`).
        #[arg(long, value_name = "NAME")]
        out: Option<String>,
        /// Emit JSON to stdout instead of writing a file (equivalent to
        /// `--format json --dry-run`; kept because every other query has it).
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
    /// Show the critical chain from X, or the dependency path X → Y.
    ///
    /// Named `chain` rather than `path` (RH F-12): "path" reads as a filesystem
    /// path, and the thing this answers is "what has to happen, in order".
    Chain {
        /// The start node (id, number, or unique name prefix).
        reference: String,
        /// Optional destination node.
        to: Option<String>,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show where you are in the plan: the current project and arc.
    ///
    /// Was `context` (RH F-11) — the question is "which project am I in", and
    /// `project` names the answer rather than the mechanism. Defaults to the
    /// current selection; `--name` inspects another project instead.
    Project {
        /// Show this project instead of the current one (id, number, or unique
        /// name prefix).
        #[arg(long, value_name = "NAME")]
        name: Option<String>,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Set the current project or arc context.
    Use {
        /// Which slot to set.
        kind: UseKindArg,
        /// A node id, number, or unique name prefix.
        reference: String,
    },
    /// Validate the graph: schema, links, cycles, recomposition, order.
    ///
    /// Pure — reads the corpus and nothing else. This is what `check` used to
    /// do; ODD-0023 §5 gave the cheap operation its own name so nobody has to
    /// remember a flag to avoid paying for probes.
    Validate {
        /// CI mode: promote warnings (staleness, soft-satisfaction) to failures.
        #[arg(long)]
        strict: bool,
        /// Emit JSON (`validate/v1`).
        #[arg(long)]
        json: bool,
    },
    /// Check the plan against reality: `validate`, then `reconcile`.
    ///
    /// The composite (ODD-0023 §5). `validate` runs first and, if it reports
    /// **errors**, `check` stops there — probing a graph with dangling edges or
    /// cycles is noise on top of a known-bad state, and probes cost time and
    /// have side effects. Warnings do not stop it.
    ///
    /// Exit is the worse of the two phases: `0` clean / `1` violations or drift.
    Check {
        /// CI mode: promote warnings and probe errors to failures, in both halves.
        #[arg(long)]
        strict: bool,
        /// Emit JSON (`check/v2` — a `validate` section and a `reconcile` one).
        #[arg(long)]
        json: bool,
    },
    /// Reconcile declared `desired_facts` against reality: report drift.
    ///
    /// Runs every node's probes. A confirmed drift fails (exit 1); a probe that
    /// could not run is a warning, surfaced always, failing only under `--strict`.
    Reconcile {
        /// CI mode: promote probe errors ("couldn't check") to failures.
        #[arg(long)]
        strict: bool,
        /// Emit JSON (`reconcile/v1`).
        #[arg(long)]
        json: bool,
    },
    /// Import a document corpus or a plan set into the node model.
    ///
    /// **Config-driven, not path-driven** (arc-migration-fidelity s14): every
    /// root — the plan-set(s), the design/research corpus
    /// (`docs_directory` + `"design"`), the dev-doc corpus (`dev_directory`)
    /// — resolves from the operational config (`[legacy]`, or the top-level
    /// key). Run from where that config lives; there is no path argument for
    /// "the corpus." Without `--plan`/`--legacy`, both the plan-set and the
    /// design/research derivations run — their roots are separate and
    /// unambiguous, so there is no shape to guess.
    ///
    /// Idempotent (re-running is a no-op, keyed on the preserved number),
    /// `--dry-run`-able, and never deletes or mutates a source file.
    Migrate {
        /// Extra legacy directories to sweep — un-typed dirs beyond
        /// `docs_directory`/`dev_directory` (research notes, brainstorm
        /// sessions, chat logs) that `--all` migrates and remembers between
        /// runs. Comma-separated, e.g. `research,brainstorm,chats`; only
        /// meaningful with `--all` (arc-migration-fidelity s14 F-1/F-4/F-5).
        additional_paths: Option<String>,
        /// Treat the plan-set(s) as the only derivation to run.
        #[arg(long, conflicts_with = "legacy")]
        plan: bool,
        /// Treat the design/research corpus as the only derivation to run.
        #[arg(long)]
        legacy: bool,
        /// Re-derive the **existing** plan nodes in place: normalized names
        /// and git-derived dates.
        ///
        /// Ordinary import skips nodes that already exist, so this is how a
        /// derivation fix reaches a corpus already minted. Ids, numbers, gates
        /// and edges are never touched.
        #[arg(long)]
        replan: bool,
        /// Report the plan and write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Report the doc-coverage/representation/stub-body/provenance gap
        /// inventory over the configured `docs_directory` and exit.
        /// Read-only: mints no node, changes no schema (arc-migration-
        /// fidelity slice01).
        #[arg(long, conflicts_with_all = ["plan", "legacy", "replan", "dry_run"])]
        coverage: bool,
        /// Mint an `artifact` node for every supporting doc under the
        /// configured `docs_directory` not already covered — mint-all, no
        /// exemption (arc-migration-fidelity slice09/slice10, ODD-0025 §2.6).
        #[arg(long, conflicts_with_all = ["plan", "legacy", "replan", "coverage"])]
        artifacts: bool,
        /// Mint a `note` node for every dev doc under the configured
        /// `dev_directory` not already covered — mint-all, uncontained,
        /// tagged by its immediate subdirectory (arc-migration-fidelity
        /// slice10, operator decision).
        #[arg(long, conflicts_with_all = ["plan", "legacy", "replan", "coverage", "artifacts"])]
        notes: bool,
        /// Compose every derivation into one idempotent, dry-run-able pass:
        /// self-host every plan-set directory found under `docs_directory`
        /// (project included — arc-migration-fidelity slice15 F-3: it
        /// reconciles like any other plan node), reconcile design/research
        /// over `docs_directory` + `"design"`, mint-or-reconcile the artifact
        /// and note corpora, and sweep `[ADDITIONAL_PATHS]` — closing the gap
        /// where a forgotten `--artifacts` re-run lets newly-authored docs
        /// sit uncovered (arc-migration-fidelity slice13/slice14).
        #[arg(
            long,
            conflicts_with_all = ["plan", "legacy", "replan", "coverage", "artifacts", "notes"]
        )]
        all: bool,
        /// Re-classify every genuinely-migrated `project`/`arc`/`slice` node
        /// as **authored** (ODD-0026 §2.1 sub-decision (i), arc-store-as-
        /// source slice02): `origin` flips to `authored`, and the former
        /// `source.paths` is preserved in a new `source.migrated_from`
        /// marker. Explicit and one-time — deliberately **not** part of
        /// `--all`, so it never fires as a side effect of the ordinary
        /// migrate workflow; `--dry-run` previews it first. Idempotent — an
        /// already-authored node is a no-op.
        #[arg(
            long,
            conflicts_with_all = ["plan", "legacy", "replan", "coverage", "artifacts", "notes", "all"]
        )]
        to_authored: bool,
    },
    /// Manage nodes: create, inspect, relate, and advance them.
    ///
    /// The node *entity* tier (ODD-0023 §4) — as distinct from the top-level
    /// verbs, whose object is the graph as a whole, and `store`, whose object is
    /// the container.
    Node {
        /// The node operation. Omitted, the group lists what it can do.
        #[command(subcommand)]
        command: Option<NodeCommand>,
    },
    /// Manage the store itself: where it lives and how it is created.
    ///
    /// The store's own lifecycle, as distinct from the nodes inside it
    /// (ODD-0023).
    Store {
        /// The store operation. Omitted, the group lists what it can do.
        #[command(subcommand)]
        command: Option<StoreCommand>,
    },
}

/// Renders a command group's own help to `out` and reports success.
///
/// Used when a group is invoked bare. clap would treat that as a usage error —
/// help text on stderr, exit 2 — but "what can I do to a node?" is a fair
/// question with a real answer, so it is answered on stdout with exit `0`
/// (ODD-0023 §7; odm's data-to-stdout convention).
fn group_help(group: &str, out: &mut dyn std::io::Write) -> anyhow::Result<u8> {
    let mut cli = Cli::command();
    let help = cli
        .find_subcommand_mut(group)
        .with_context(|| format!("the {group} group is missing from the command surface"))?
        .render_help();
    writeln!(out, "{help}")?;
    Ok(EXIT_OK)
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
            // The sink here really is the process's stderr, so this is
            // `oxur-term`'s own helper rather than the writer-scoped `term`.
            oxur_term::common::output::error(&format!("{e:#}"));
            return ExitCode::from(EXIT_ERROR);
        }
    };
    match dispatch(cli, &root, &mut std::io::stdout(), &mut std::io::stderr()) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            // The sink here really is the process's stderr, so this is
            // `oxur-term`'s own helper rather than the writer-scoped `term`.
            oxur_term::common::output::error(&format!("{e:#}"));
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
    // The store is wherever the locator says — the repo root when it says
    // nothing. `root` stays the *invocation* root: it is what the layered
    // config search starts from, and what a relative path argument is resolved
    // against, neither of which moves with the store.
    let home = StoreHome::resolve(root);
    let store = Store::open(&home.store_root);

    // Bare `odm` (no subcommand) orients — it never bare-errors.
    let command = cli.command.unwrap_or(Command::Orient { json: false });

    match command {
        Command::Orient { json } => orient::orient(&store, root, json, out)?,
        // The context names node ids, so it belongs with the nodes: it resolves
        // under the store root, as the `.odm/` index already does (slice-01
        // carried item #1). The path comes off the `Store` handle, so no caller
        // can hand it the invocation root instead — which is what left `orient`
        // blind to a selection `use` had just written (RH C-5).
        Command::Use { kind, reference } => {
            commands::use_context(&store, kind.into(), &reference, err)?;
        }
        Command::Project { name, json } => {
            commands::project(&store, name.as_deref(), json, out)?;
        }
        // `validate` returns its own exit code (0 clean / 1 violations).
        Command::Validate { strict, json } => {
            return commands::validate(&store, root, strict, json, out);
        }
        // `check` is the composite: validate, then reconcile unless validate
        // errored. It returns the worse of the two exit codes.
        Command::Check { strict, json } => {
            return commands::check(&store, root, strict, json, out);
        }
        // `reconcile` likewise returns its own exit code (0 clean / 1 drift).
        Command::Reconcile { strict, json } => {
            return reconcile::reconcile(&store, strict, json, out);
        }
        Command::Rollup { dry_run, format, out: stem, json } => {
            // `--json` is the long-standing spelling of "give me json on
            // stdout"; it stays equivalent to `--format json --dry-run`.
            let format = if json { FormatArg::Json } else { format };
            let options = rollup::Options {
                dry_run: dry_run || json,
                extension: format.extension(),
                stem: stem.as_deref().unwrap_or(rollup::DEFAULT_STEM),
                json: format == FormatArg::Json,
            };
            rollup::rollup(&store, root, options, out, err)?;
        }
        Command::Next { json } => commands::next(&store, root, json, out)?,
        Command::Blocked { reference, json } => {
            commands::blocked(&store, root, &reference, json, out)?;
        }
        Command::Chain { reference, to, json } => {
            commands::chain(&store, root, &reference, to.as_deref(), json, out)?;
        }
        Command::Store { command } => match command {
            None => return group_help("store", out),
            Some(command) => match command {
                StoreCommand::Rename { new, worktree, branch, dry_run, yes: _, json } => {
                    // The bare positional renames both halves; the flags override
                    // it per half, so `rename plan --branch keep` is expressible.
                    let new_worktree = worktree.or_else(|| new.clone());
                    let new_branch = branch.or(new);
                    store_cmd::rename(
                        root,
                        new_worktree.as_deref(),
                        new_branch.as_deref(),
                        dry_run,
                        json,
                        out,
                        err,
                    )?;
                }
                StoreCommand::Init { worktree, branch, dry_run, yes: _, json } => {
                    store_cmd::init(
                        root,
                        worktree.as_deref(),
                        branch.as_deref(),
                        dry_run,
                        json,
                        out,
                        err,
                    )?;
                }
                StoreCommand::Commit { message, dry_run, json } => {
                    store_cmd::commit(root, message.as_deref(), dry_run, json, out, err)?;
                }
                StoreCommand::Status { json } => {
                    store_cmd::status(root, json, out, err)?;
                }
            },
        },
        Command::Migrate {
            additional_paths,
            plan,
            legacy,
            replan,
            dry_run,
            coverage,
            artifacts,
            notes,
            all,
            to_authored,
        } => {
            let forced = if plan {
                Some(odm_migrate::Corpus::Plan)
            } else if legacy {
                Some(odm_migrate::Corpus::Legacy)
            } else {
                None
            };
            // Trimmed, empties-dropped (arc-migration-fidelity s14 F-1): a
            // trailing/doubled comma or stray whitespace must not smuggle an
            // empty string into `[legacy].additional_paths`.
            let additional_paths: Vec<String> = additional_paths
                .as_deref()
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            migrate::migrate(
                &store,
                root,
                &additional_paths,
                migrate::Options {
                    forced,
                    replan,
                    dry_run,
                    coverage,
                    artifacts,
                    notes,
                    all,
                    to_authored,
                },
                out,
                err,
            )?;
        }
        Command::Node { command } => match command {
            // A bare group name is a question — "what can I do to a node?" —
            // and clap's default answer is a usage *error* on stderr, exit 2.
            // Answering it properly (ODD-0023 §7) means help on stdout, exit 0.
            None => return group_help("node", out),
            Some(command) => match command {
                NodeCommand::New {
                    node_type,
                    name,
                    parent,
                    metadata,
                    content,
                    from_file,
                    body,
                    dry_run,
                    json,
                    yes: _,
                } => {
                    // At most one of content/from_file/body is set — clap's
                    // `conflicts_with_all` on each flag already enforces it.
                    let content_path = content.as_deref().or(from_file.as_deref());
                    let body_source = match (content_path, body.as_deref()) {
                        (Some(path), _) => {
                            Some(commands::BodySource::File(std::path::Path::new(path)))
                        }
                        (None, Some(inline)) => Some(commands::BodySource::Inline(inline)),
                        (None, None) => None,
                    };
                    commands::new(
                        &store,
                        root,
                        &node_type,
                        &name,
                        commands::NewOptions {
                            parent: parent.as_deref(),
                            metadata: metadata.as_deref().map(std::path::Path::new),
                            body: body_source,
                            dry_run,
                            json,
                        },
                        out,
                        err,
                    )?;
                }
                NodeCommand::Set { reference, field, value, dry_run, json, yes: _ } => {
                    commands::set(
                        &store,
                        root,
                        commands::SetArgs {
                            reference: &reference,
                            field: &field,
                            value: &value,
                            dry_run,
                            json,
                        },
                        out,
                        err,
                    )?;
                }
                NodeCommand::SetBody { reference, from_file, body, dry_run, json, yes: _ } => {
                    let source = match (from_file.as_deref(), body.as_deref()) {
                        (Some(path), _) => commands::BodySource::File(std::path::Path::new(path)),
                        (None, Some(inline)) => commands::BodySource::Inline(inline),
                        (None, None) => {
                            anyhow::bail!("`node set-body` needs one of --from-file or --body")
                        }
                    };
                    commands::set_body(&store, &reference, source, dry_run, json, out, err)?;
                }
                NodeCommand::List {
                    node_type,
                    tag,
                    component,
                    date,
                    width,
                    group,
                    status,
                    all,
                    json,
                } => {
                    // Asking for a withdrawn status is asking to see withheld rows.
                    let withdrawn_status = status.as_deref().is_some_and(|s| {
                        matches!(s.trim().to_ascii_lowercase().as_str(), "retired" | "superseded")
                    });
                    let view = commands::ListView {
                        type_filter: node_type.as_deref(),
                        tag: tag.as_deref(),
                        component: component.as_deref(),
                        date: date.into(),
                        width,
                        status: status.as_deref(),
                        group: group.map(Into::into),
                        include_withdrawn: all || withdrawn_status,
                        json,
                    };
                    commands::list(&store, root, view, out)?;
                }
                NodeCommand::Show { reference, json } => {
                    commands::show(&store, root, &reference, json, out)?
                }
                NodeCommand::Rename { reference, name, dry_run, yes: _ } => {
                    commands::rename(&store, &reference, &name, dry_run, err)?;
                }
                NodeCommand::Retire { reference, because, dry_run, yes: _ } => {
                    commands::retire(&store, &reference, &because, dry_run, err)?;
                }
                NodeCommand::Supersede { reference, with, kind, dry_run, yes: _ } => {
                    commands::supersede(&store, &reference, &with, kind.into(), dry_run, err)?;
                }
                NodeCommand::Link { source, edge, target, satisfied_at, dry_run, yes: _ } => {
                    commands::link(
                        &store,
                        &source,
                        edge.into(),
                        &target,
                        satisfied_at.as_deref(),
                        dry_run,
                        err,
                    )?;
                }
                NodeCommand::Unlink { source, edge, target, dry_run, yes: _ } => {
                    commands::unlink(&store, &source, edge.into(), &target, dry_run, err)?;
                }
                NodeCommand::SetGate { reference, gate, by, evidence, dry_run, yes: _ } => {
                    let reach = commands::GateReach { gate: &gate, by, evidence: evidence.into() };
                    commands::set_gate(&store, root, &reference, reach, dry_run, err)?;
                }
                NodeCommand::Tear { source, edge: _, target, because, dry_run, yes: _ } => {
                    commands::tear(&store, &source, &target, &because, dry_run, err)?;
                }
                NodeCommand::Decomposed { reference, children, dry_run, yes: _ } => {
                    commands::decomposed(&store, &reference, &children, dry_run, err)?;
                }
            },
        },
    }
    Ok(EXIT_OK)
}
