//! The `odm reconcile` command (arc05 slice03): run the corpus probe-runner and
//! report drift to stdout, with `check`-consistent severities and exit codes.
//!
//! Reads `desired_facts` via [`Runner::run_corpus`] (the slice02 **store**-read
//! runner) — no index reader is added, so the carried A4 adapter-fidelity
//! invariant stays honored by non-triggering.

use std::collections::HashMap;
use std::io::Write;

use anyhow::Context as _;
use odm_core::Id;
use odm_core::frontmatter::Document;
use odm_reconcile::{NodeReport, OutcomeCounts, ProbeOutcome, Runner};
use odm_store::Store;
use serde::Serialize;

use crate::commands::{EXIT_OK, EXIT_VIOLATIONS};

/// The `reconcile --json` schema marker (versions the wire contract from here
/// forward, like `check/v1` / `rollup/v1` / `orient/v1`).
pub(crate) const RECONCILE_SCHEMA: &str = "reconcile/v1";

/// The verdict for a reconcile run, derived **purely** from outcome counts — no
/// I/O, so it is unit-testable in isolation (R-4).
///
/// Mirrors `check`: a confirmed **drift** is an error (always fails); a probe
/// **error** ("couldn't check") is a warning (surfaced always, fails only under
/// `--strict`); otherwise the run is clean. When both drift and probe-error are
/// present, drift dominates — the run is an error either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Verdict {
    /// Overall severity label: `"clean"`, `"warning"`, or `"error"`.
    severity: &'static str,
    /// Exit code without `--strict`.
    exit: u8,
    /// Exit code with `--strict`.
    strict_exit: u8,
}

impl Verdict {
    /// The exit code for the active `--strict` setting.
    fn exit_code(self, strict: bool) -> u8 {
        if strict { self.strict_exit } else { self.exit }
    }
}

/// Maps outcome counts to a [`Verdict`] — the pure severity/exit policy (R-4).
fn verdict(counts: &OutcomeCounts) -> Verdict {
    if counts.drifted > 0 {
        Verdict { severity: "error", exit: EXIT_VIOLATIONS, strict_exit: EXIT_VIOLATIONS }
    } else if counts.errored > 0 {
        Verdict { severity: "warning", exit: EXIT_OK, strict_exit: EXIT_VIOLATIONS }
    } else {
        Verdict { severity: "clean", exit: EXIT_OK, strict_exit: EXIT_OK }
    }
}

/// Runs the corpus reconcile and renders the report to `out`, returning the
/// exit code ([`EXIT_OK`] when clean / warnings-without-`--strict`,
/// [`EXIT_VIOLATIONS`] on drift or, under `--strict`, a probe error).
///
/// # Errors
///
/// Returns an error (which the caller maps to exit code `2`) if the corpus
/// cannot be loaded from the store.
pub(crate) fn reconcile(
    store: &Store,
    strict: bool,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<u8> {
    let report = Runner::new(store).run_corpus().context("running the corpus reconcile")?;
    let counts = report.counts();
    let code = verdict(&counts).exit_code(strict);

    // The report carries node/fact ids only; join with the store's documents for
    // each node's number/name and each fact's `describe` — the render identity.
    let documents = store.load_all().context("loading nodes to label the report")?;
    let by_id: HashMap<Id, &Document> =
        documents.iter().map(|doc| (doc.frontmatter().id(), doc)).collect();
    let views: Vec<NodeView<'_>> =
        report.nodes.iter().map(|node| NodeView::build(node, &by_id)).collect();

    if json {
        let report_json = ReconcileReport {
            schema: RECONCILE_SCHEMA,
            ok: code == EXIT_OK,
            counts: CountsJson::from(&counts),
            nodes: views.iter().map(NodeJson::from).collect(),
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&report_json)?)?;
        return Ok(code);
    }

    render_human(out, &counts, &views, strict)?;
    Ok(code)
}

/// Renders the human report: a clean run is one plain "no drift" line (no
/// fabricated data); otherwise a header plus one entry per drifted / errored
/// fact (holds are not listed — only findings).
fn render_human(
    out: &mut dyn Write,
    counts: &OutcomeCounts,
    views: &[NodeView<'_>],
    strict: bool,
) -> anyhow::Result<()> {
    if counts.drifted == 0 && counts.errored == 0 {
        if counts.holds == 0 {
            writeln!(out, "reconcile: no drift (no desired_facts declared)")?;
        } else {
            writeln!(out, "reconcile: no drift ({} fact(s) hold)", counts.holds)?;
        }
        return Ok(());
    }

    writeln!(
        out,
        "reconcile: {} drifted, {} couldn't-check ({} held)",
        counts.drifted, counts.errored, counts.holds
    )?;
    for node in views {
        for fact in &node.facts {
            match fact.outcome {
                ProbeOutcome::Holds => {}
                ProbeOutcome::Drifted { expected, observed } => {
                    writeln!(
                        out,
                        "  [drift] #{} {:?} / {}: {}",
                        node.number, node.name, fact.fact_id, fact.describe
                    )?;
                    writeln!(out, "    expected: {expected}")?;
                    writeln!(out, "    observed: {observed}")?;
                }
                ProbeOutcome::Error { reason } => {
                    writeln!(
                        out,
                        "  [error] #{} {:?} / {}: {}",
                        node.number, node.name, fact.fact_id, fact.describe
                    )?;
                    writeln!(out, "    reason: {reason}")?;
                }
            }
        }
    }
    if !strict && counts.errored > 0 && counts.drifted == 0 {
        writeln!(out, "(probe errors do not fail; run with --strict to enforce)")?;
    }
    Ok(())
}

/// A node's results joined with its render identity (number, name) and each
/// fact's `describe`. Borrows the loaded documents and the report; the single
/// source both the human and `--json` renderings derive from (no drift between
/// the two views).
struct NodeView<'a> {
    node_id: Id,
    number: u32,
    name: &'a str,
    facts: Vec<FactView<'a>>,
}

/// One fact's result joined with its declared `describe`.
struct FactView<'a> {
    fact_id: &'a str,
    describe: &'a str,
    outcome: &'a ProbeOutcome,
}

impl<'a> NodeView<'a> {
    /// Joins one [`NodeReport`] with its document for the render identity. A
    /// node absent from the corpus map (a race) renders with placeholder
    /// identity rather than failing the whole report.
    fn build(node: &'a NodeReport, by_id: &HashMap<Id, &'a Document>) -> Self {
        let document = by_id.get(&node.node_id).copied();
        let (number, name) = document
            .map(|doc| (doc.frontmatter().number(), doc.frontmatter().name()))
            .unwrap_or((0, "<unknown>"));
        let declared = document.map(|doc| doc.frontmatter().desired_facts()).unwrap_or(&[]);
        let facts = node
            .results
            .iter()
            .map(|result| FactView {
                fact_id: &result.fact_id,
                describe: declared
                    .iter()
                    .find(|fact| fact.id == result.fact_id)
                    .map_or("", |fact| fact.describe.as_str()),
                outcome: &result.outcome,
            })
            .collect();
        NodeView { node_id: node.node_id, number, name, facts }
    }
}

/// The `reconcile/v1` JSON envelope — a 1:1 projection of the report.
#[derive(Serialize)]
struct ReconcileReport {
    /// The schema-version marker (`"reconcile/v1"`).
    schema: &'static str,
    /// Whether the run passed under the active `--strict` setting.
    ok: bool,
    /// Outcome tally (holds / drifted / errored).
    counts: CountsJson,
    /// One entry per node that declared at least one fact.
    nodes: Vec<NodeJson>,
}

/// JSON shape of the outcome tally.
#[derive(Serialize)]
struct CountsJson {
    holds: usize,
    drifted: usize,
    errored: usize,
}

impl From<&OutcomeCounts> for CountsJson {
    fn from(counts: &OutcomeCounts) -> Self {
        Self { holds: counts.holds, drifted: counts.drifted, errored: counts.errored }
    }
}

/// JSON shape of one node's reconcile results.
#[derive(Serialize)]
struct NodeJson {
    node_id: String,
    number: u32,
    name: String,
    results: Vec<FactJson>,
}

impl From<&NodeView<'_>> for NodeJson {
    fn from(view: &NodeView<'_>) -> Self {
        Self {
            node_id: view.node_id.to_string(),
            number: view.number,
            name: view.name.to_string(),
            results: view.facts.iter().map(FactJson::from).collect(),
        }
    }
}

/// JSON shape of one fact's result; `outcome` is the `ProbeOutcome` tagged on
/// `kind` (`holds` / `drifted` / `error`).
#[derive(Serialize)]
struct FactJson {
    fact_id: String,
    describe: String,
    outcome: ProbeOutcome,
}

impl From<&FactView<'_>> for FactJson {
    fn from(fact: &FactView<'_>) -> Self {
        Self {
            fact_id: fact.fact_id.to_string(),
            describe: fact.describe.to_string(),
            outcome: fact.outcome.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(holds: usize, drifted: usize, errored: usize) -> OutcomeCounts {
        OutcomeCounts { holds, drifted, errored }
    }

    /// R-4: the severity/exit mapping is a pure function of the counts.
    #[test]
    fn reconcile_exit_severity_is_pure_fn_of_counts() {
        // counts(holds, drifted, errored) -> (severity, exit, exit-under-strict)
        let cases = [
            (counts(0, 0, 0), ("clean", EXIT_OK, EXIT_OK)),
            (counts(3, 0, 0), ("clean", EXIT_OK, EXIT_OK)),
            (counts(0, 1, 0), ("error", EXIT_VIOLATIONS, EXIT_VIOLATIONS)),
            (counts(2, 1, 0), ("error", EXIT_VIOLATIONS, EXIT_VIOLATIONS)),
            (counts(0, 0, 1), ("warning", EXIT_OK, EXIT_VIOLATIONS)),
            (counts(1, 0, 2), ("warning", EXIT_OK, EXIT_VIOLATIONS)),
            // Drift dominates a probe-error when both are present.
            (counts(0, 1, 1), ("error", EXIT_VIOLATIONS, EXIT_VIOLATIONS)),
        ];
        for (input, (severity, exit, strict_exit)) in cases {
            let verdict = verdict(&input);
            assert_eq!(verdict.severity, severity, "severity for {input:?}");
            assert_eq!(verdict.exit, exit, "exit for {input:?}");
            assert_eq!(verdict.strict_exit, strict_exit, "strict_exit for {input:?}");
            assert_eq!(verdict.exit_code(false), exit);
            assert_eq!(verdict.exit_code(true), strict_exit);
        }
    }
}
