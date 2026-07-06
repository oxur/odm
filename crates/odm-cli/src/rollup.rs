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
use odm_core::rollup::{
    ActiveTear, BlockReason, BlockedNode, GateStatus, NodeRef, Provenance, ReadyNode, Rollup,
    TreeNode,
};
use odm_store::Store;
use sha2::{Digest as _, Sha256};

use crate::commands;
use crate::json::RollupJson;

/// The generated rollup file, written at the store root.
const ROLLUP_FILE: &str = "ROLLUP.md";

/// The token in the generated header that carries the corpus meta-fingerprint
/// (slice07). `odm rollup` compares the current corpus's fingerprint against the
/// one stamped here to decide whether a regenerate is needed.
const FINGERPRINT_TAG: &str = "fingerprint=";

/// The generated-file header line, stamping the corpus meta-fingerprint so a
/// later run can detect a semantically-unchanged corpus and skip the rewrite.
fn header_line(fingerprint: &str) -> String {
    format!(
        "<!-- GENERATED — do not edit by hand. Regenerate with `odm rollup`. \
         {FINGERPRINT_TAG}{fingerprint} -->"
    )
}

/// Extracts the `fingerprint=<hex>` stamped in a generated `ROLLUP.md`'s header
/// (its first line), or `None` if absent (e.g. a hand-written or pre-slice07
/// file — which then always regenerates, the safe default).
fn stamped_fingerprint(markdown: &str) -> Option<String> {
    let line = markdown.lines().next()?;
    let start = line.find(FINGERPRINT_TAG)? + FINGERPRINT_TAG.len();
    let rest = &line[start..];
    let end = rest.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(rest.len());
    (end > 0).then(|| rest[..end].to_string())
}

/// Lowercase hex of a 32-byte digest (for the header fingerprint).
fn to_hex(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// The drift-aware early-cutoff fingerprint (slice08 L-5): SHA-256 over the whole
/// rendered model (the `rollup/v1` JSON projection — corpus tree + drift +
/// deferred). Because it hashes the *semantic* projection (drift outcomes +
/// absolute `last_checked`, never the relative "Xm ago"), it is stable across
/// wall-clock ticks and changes only when the corpus **or** the drift/deferred
/// state actually changes. Replaces the corpus-only meta-fingerprint so a drift
/// change with no corpus change no longer hides behind the cutoff.
fn content_fingerprint(model: &Rollup) -> String {
    let json = serde_json::to_vec(&RollupJson::from(model)).unwrap_or_default();
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&Sha256::digest(&json));
    to_hex(&digest)
}

/// `rollup` — regenerate `ROLLUP.md` at the repo root (with slice07 early cutoff).
///
/// Reads the corpus through the `.odm/` index (`reconcile`-then-read, slice06 —
/// no full parse), assembles the [`Rollup`] model, renders it to Markdown, and
/// writes it atomically (write-temp-rename) via odm-store. The render is
/// deterministic, so re-running on an unchanged corpus produces identical bytes.
///
/// **Early cutoff (slice07, ODD-0014 §2.4):** the generated header stamps a
/// **meta-fingerprint** (a hash over the reconciled records' `(id, meta_hash)`).
/// A run recomputes that fingerprint and, if it matches the one already stamped
/// in `ROLLUP.md`, **leaves the file untouched** — a body-only edit refreshes the
/// index record but regenerates nothing downstream. Any `meta_hash` change, new,
/// or deleted node moves the fingerprint and forces a regenerate.
///
/// `--dry-run` writes no file: it previews the rendered Markdown to `out` and
/// reports to `err`. `--json` serializes the **same** model (D-3) to `out` and
/// writes no file.
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
    let (gates, threshold) = commands::load_gate_config(root)?;
    let snapshot = odm_index::reconcile(store, &odm_index::default_index_path(store.root()))
        .context("reconciling the index for rollup")?
        .snapshot;
    let frontmatters = odm_index::frontmatters_from_records(&snapshot.records, &gates);
    // The model is index-backed, but the index carries no `desired_facts` (slice02):
    // drift + deferred come from a single on-demand, store-read incremental
    // reconcile (S-5 / D-5), never the index. `orient` uses the same shared
    // projector so the two views can't diverge (and no volatile probes run).
    let (drift, deferred) = crate::reconcile::reconcile_views(store)?;
    let model = Rollup::assemble(&frontmatters, &gates, threshold)
        .with_drift(drift)
        .with_deferred(deferred);

    // The early-cutoff fingerprint is a hash of the whole rendered model (slice08
    // L-5), not just the corpus meta-fingerprint — so it is **drift-aware**: a
    // drift/deferred change with no corpus change regenerates `ROLLUP.md`, while
    // an unchanged model still skips. Staleness (`last checked Xm ago`) is
    // rendered from *absolute* timestamps carried in the JSON, so the fingerprint
    // is stable across wall-clock ticks (only a re-check or outcome change moves it).
    let fingerprint = content_fingerprint(&model);

    // `--json` is a non-writing output mode: serialize the same model to stdout.
    if json {
        let view = RollupJson::from(&model);
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    let markdown = render(&model, &fingerprint);

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

    // Early cutoff: the existing file already reflects this exact semantic state.
    if let Ok(existing) = std::fs::read_to_string(&path) {
        if stamped_fingerprint(&existing).as_deref() == Some(fingerprint.as_str()) {
            writeln!(
                err,
                "{} unchanged (corpus semantically unchanged); skipped regeneration",
                path.display()
            )?;
            return Ok(());
        }
    }

    odm_store::atomic::write(&path, markdown.as_bytes())
        .with_context(|| format!("writing {}", path.display()))?;
    writeln!(err, "wrote {} ({} node(s))", path.display(), snapshot.records.len())?;
    Ok(())
}

/// Renders the [`Rollup`] model to Markdown in the canonical section order
/// (ODD-0013 §6): way-finding tree (status inline) → ready → blocked → active
/// tears → provenance → drift → deferred. The deferred section is emitted only
/// when a node is deferred (slice06); none deferred → no section, no fabricated
/// data. The header stamps `fingerprint` (slice07) so a later run can detect a
/// semantically-unchanged corpus.
#[must_use]
pub fn render(model: &Rollup, fingerprint: &str) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "{}\n", header_line(fingerprint));
    let _ = writeln!(s, "# Rollup\n");

    render_tree(&mut s, model);
    render_ready(&mut s, &model.ready);
    render_blocked(&mut s, &model.blocked);
    render_tears(&mut s, &model.tears);
    render_provenance(&mut s, &model.provenance);
    render_drift(&mut s, model);
    render_deferred(&mut s, model);

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

/// Renders the drift section from the injected [`Drift`](odm_core::rollup::Drift)
/// projection (A5 slice04 — Q-A3-2): per-fact drift/couldn't-check with identity
/// and expected/observed. A clean corpus renders an honest "no drift" — never
/// fabricated rows.
fn render_drift(s: &mut String, model: &Rollup) {
    use crate::reconcile::staleness_suffix;
    let _ = writeln!(s, "## Drift\n");
    let drift = &model.drift;
    if drift.is_clean() && drift.unchecked.is_empty() {
        let _ = writeln!(s, "_No drift._\n");
        return;
    }
    if !drift.is_clean() {
        let _ = writeln!(
            s,
            "{} drifted, {} couldn't-check ({} holding).\n",
            drift.drifted.len(),
            drift.errored.len(),
            drift.holds
        );
        for d in &drift.drifted {
            let _ = writeln!(
                s,
                "- #{} {} / {} — {}{}",
                d.number,
                d.name,
                d.fact_id,
                d.describe,
                staleness_suffix(d.freshness)
            );
            let _ = writeln!(s, "  - expected: {}", d.expected);
            let _ = writeln!(s, "  - observed: {}", d.observed);
        }
        for e in &drift.errored {
            let _ = writeln!(
                s,
                "- #{} {} / {} (couldn't check) — {}{}",
                e.number,
                e.name,
                e.fact_id,
                e.describe,
                staleness_suffix(e.freshness)
            );
            let _ = writeln!(s, "  - reason: {}", e.reason);
        }
    }
    // Never-checked volatile facts — surfaced honestly, never a fabricated "fresh".
    if !drift.unchecked.is_empty() {
        let _ = writeln!(s, "Not yet checked ({}) — run `odm reconcile`:", drift.unchecked.len());
        for u in &drift.unchecked {
            let _ = writeln!(s, "- #{} {} / {} — {}", u.number, u.name, u.fact_id, u.describe);
        }
    }
    let _ = writeln!(s);
}

/// Renders the deferred section from the injected
/// [`Deferred`](odm_core::rollup::Deferred) projection (A5 slice06 — Q-A3-1):
/// each parked node with *why* and its re-entry status. **Emitted only when a
/// node is deferred** — none deferred → no section, no fabricated data.
fn render_deferred(s: &mut String, model: &Rollup) {
    use odm_core::rollup::Reentry;
    if model.deferred.is_empty() {
        return;
    }
    let _ = writeln!(s, "## Deferred\n");
    for node in &model.deferred.nodes {
        let status = match &node.reentry {
            Reentry::Ready => "ready to re-enter".to_string(),
            Reentry::Waiting { describe } => format!("waiting on {describe}"),
        };
        let _ = writeln!(s, "- #{} {} — {} ({status})", node.number, node.name, node.because);
    }
    let _ = writeln!(s);
}
