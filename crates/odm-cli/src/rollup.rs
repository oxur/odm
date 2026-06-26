//! `odm rollup` — regenerate `ROLLUP.md`, the single cheap view of the whole
//! plan (ODD-0013 §6; arc03 slice02).
//!
//! The whole-plan **model** is assembled in `odm-core`
//! ([`Rollup::assemble`](odm_core::rollup::Rollup::assemble)) as a pure function
//! of a full-scan corpus (arc03 D-2/D-3). This module owns only the **Markdown
//! rendering** of that model and the command that writes it atomically to the
//! repo root. Keeping rendering here (and the model in `odm-core`) lets slice03
//! (`orient`) and slice04 (`--json`) reuse the one model instead of re-deriving
//! the view.
//!
//! The render is a pure function of the model and carries no timestamp, so the
//! same corpus regenerates byte-identical output (idempotent).

use std::fmt::Write as _;
use std::path::Path;

use anyhow::Context as _;
use odm_core::frontmatter::Frontmatter;
use odm_core::rollup::{
    ActiveTear, BlockReason, BlockedNode, GateStatus, NodeRef, Provenance, ReadyNode, Rollup,
    TreeNode,
};
use odm_store::Store;

use crate::commands;
use crate::json::RollupJson;

/// The generated rollup file, written at the store root.
const ROLLUP_FILE: &str = "ROLLUP.md";

/// The header marking the file as generated (do not hand-edit).
const HEADER: &str = "<!-- GENERATED — do not edit by hand. Regenerate with `odm rollup`. -->";

/// `rollup` — full-scan regenerate of `ROLLUP.md` at the repo root.
///
/// Loads the whole corpus, assembles the [`Rollup`] model, renders it to
/// Markdown, and writes it atomically (write-temp-rename) via odm-store. The
/// render is deterministic, so re-running on an unchanged corpus produces
/// identical bytes. `--dry-run` writes no file: it previews the rendered
/// Markdown to `out` and reports to `err`. `--json` serializes the **same**
/// model (D-3) to `out` and writes no file.
///
/// # Errors
///
/// Returns an error (mapped to exit code `2`) if the corpus or gate config
/// cannot be loaded, or the file cannot be written.
pub fn rollup(
    store: &Store,
    root: &Path,
    dry_run: bool,
    json: bool,
    out: &mut dyn std::io::Write,
    err: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    let docs = store.load_all().context("loading the corpus for rollup")?;
    let frontmatters: Vec<Frontmatter> = docs.iter().map(|d| d.frontmatter().clone()).collect();
    let (gates, threshold) = commands::load_gate_config(root)?;
    let model = Rollup::assemble(&frontmatters, &gates, threshold);

    // `--json` is a non-writing output mode: serialize the same model to stdout.
    if json {
        let view = RollupJson::from(&model);
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    let markdown = render(&model);

    let path = root.join(ROLLUP_FILE);
    if dry_run {
        write!(out, "{markdown}")?;
        writeln!(
            err,
            "dry-run: would write {} ({} bytes); nothing written",
            path.display(),
            markdown.len()
        )?;
        return Ok(());
    }

    odm_store::atomic::write(&path, markdown.as_bytes())
        .with_context(|| format!("writing {}", path.display()))?;
    writeln!(err, "wrote {} ({} node(s))", path.display(), docs.len())?;
    Ok(())
}

/// Renders the [`Rollup`] model to Markdown in the canonical section order
/// (ODD-0013 §6): way-finding tree (status inline) → ready → blocked → active
/// tears → provenance → drift. The deferred slot is empty in A3 (Q-A3-1), so no
/// deferred section is emitted.
#[must_use]
pub fn render(model: &Rollup) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "{HEADER}\n");
    let _ = writeln!(s, "# Rollup\n");

    render_tree(&mut s, model);
    render_ready(&mut s, &model.ready);
    render_blocked(&mut s, &model.blocked);
    render_tears(&mut s, &model.tears);
    render_provenance(&mut s, &model.provenance);
    render_drift(&mut s, model);

    s
}

/// A node's full label: `<type> #<number> <name>`.
pub(crate) fn label(node: &NodeRef) -> String {
    format!("{} #{} {}", node.node_type.as_str(), node.number, node.name)
}

/// A short reference to a dependency node: `#<number> <name>`.
pub(crate) fn dep_label(node: &NodeRef) -> String {
    format!("#{} {}", node.number, node.name)
}

/// The inline status vector: `gate=evidence` for reached gates, `gate=–` for
/// not-reached, in gate-sequence order. Empty string when the type has no gates.
pub(crate) fn status_inline(status: &[GateStatus]) -> String {
    status
        .iter()
        .map(|g| match g.evidence {
            Some(ev) => format!("{}={}", g.gate, ev.as_str()),
            None => format!("{}=–", g.gate),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Renders the way-finding tree section (status inline per node).
fn render_tree(s: &mut String, model: &Rollup) {
    let _ = writeln!(s, "## Way-finding tree\n");
    if model.tree.is_empty() {
        let _ = writeln!(s, "_(no nodes)_\n");
        return;
    }
    for root in &model.tree {
        render_tree_node(s, root, 0);
    }
    let _ = writeln!(s);
}

/// Renders one tree node and its subtree at the given indentation `depth`.
fn render_tree_node(s: &mut String, node: &TreeNode, depth: usize) {
    let indent = "  ".repeat(depth);
    let status = status_inline(&node.status);
    if status.is_empty() {
        let _ = writeln!(s, "{indent}- {}", label(&node.node));
    } else {
        let _ = writeln!(s, "{indent}- {} — {status}", label(&node.node));
    }
    for child in &node.children {
        render_tree_node(s, child, depth + 1);
    }
}

/// Renders the ready frontier section.
fn render_ready(s: &mut String, ready: &[ReadyNode]) {
    let _ = writeln!(s, "## Ready\n");
    if ready.is_empty() {
        let _ = writeln!(s, "_(nothing ready)_\n");
        return;
    }
    for r in ready {
        let _ = writeln!(s, "- {}", label(&r.node));
        for soft in &r.soft {
            let _ = writeln!(
                s,
                "  - soft: {} at evidence={}",
                dep_label(&soft.dep),
                soft.evidence.as_str()
            );
        }
    }
    let _ = writeln!(s);
}

/// Renders the blocked section, naming each blocked node's unsatisfied edges.
fn render_blocked(s: &mut String, blocked: &[BlockedNode]) {
    let _ = writeln!(s, "## Blocked\n");
    if blocked.is_empty() {
        let _ = writeln!(s, "_(nothing blocked)_\n");
        return;
    }
    for b in blocked {
        let _ = writeln!(s, "- {}", label(&b.node));
        for reason in &b.reasons {
            match reason {
                BlockReason::Unsatisfied { dep } => {
                    let _ = writeln!(s, "  - unsatisfied: {}", dep_label(dep));
                }
                BlockReason::SoftSatisfied { dep, evidence, threshold } => {
                    let _ = writeln!(
                        s,
                        "  - low-evidence: {} at evidence={} (needs {})",
                        dep_label(dep),
                        evidence.as_str(),
                        threshold.as_str()
                    );
                }
                BlockReason::ExternallyBlocked { by } => {
                    let _ = writeln!(s, "  - blocked-by: {}", dep_label(by));
                }
            }
        }
    }
    let _ = writeln!(s);
}

/// Renders the active-tears section, each with its rationale.
fn render_tears(s: &mut String, tears: &[ActiveTear]) {
    let _ = writeln!(s, "## Active tears\n");
    if tears.is_empty() {
        let _ = writeln!(s, "_(none)_\n");
        return;
    }
    for t in tears {
        let _ = writeln!(
            s,
            "- {} depends_on {} — because: {}",
            dep_label(&t.from),
            dep_label(&t.to),
            t.because
        );
    }
    let _ = writeln!(s);
}

/// Renders the provenance (origin) view: planned / discovered / amendment.
fn render_provenance(s: &mut String, prov: &Provenance) {
    let _ = writeln!(s, "## Provenance\n");
    render_origin_group(s, "Planned", &prov.planned);
    render_origin_group(s, "Discovered", &prov.discovered);
    render_origin_group(s, "Amendment", &prov.amendment);
}

/// Renders one origin group as a labelled list (or `(none)` when empty).
fn render_origin_group(s: &mut String, title: &str, nodes: &[NodeRef]) {
    let _ = writeln!(s, "### {title}\n");
    if nodes.is_empty() {
        let _ = writeln!(s, "_(none)_\n");
        return;
    }
    for node in nodes {
        let _ = writeln!(s, "- {}", label(node));
    }
    let _ = writeln!(s);
}

/// Renders the drift section. Drift/`reconcile` is A5 (Q-A3-2): the section is
/// structurally present but reads "not yet tracked (A5)" with no fabricated
/// data. A5 wires real drift output here once `reconcile` lands.
fn render_drift(s: &mut String, _model: &Rollup) {
    let _ = writeln!(s, "## Drift\n");
    let _ = writeln!(s, "_Not yet tracked (A5)._");
}
