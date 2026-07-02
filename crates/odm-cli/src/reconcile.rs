//! The `odm reconcile` command (arc05 slice03) and the shared drift projector
//! (arc05 slice04).
//!
//! Reads `desired_facts` via [`Runner::run_corpus`] (the slice02 **store**-read
//! runner) — no index reader is added, so the carried A4 adapter-fidelity
//! invariant stays honored by non-triggering. Since slice04 the runner's report
//! carries render-identity (node number/name + fact `describe`), so rendering
//! needs **no second corpus load** (slice03's double-load is resolved).

use std::io::Write;

use anyhow::Context as _;
use odm_core::rollup::{Deferred, DeferredNode, Drift, DriftedFact, ErroredFact, Reentry};
use odm_reconcile::{CorpusReport, NodeReport, OutcomeCounts, ProbeOutcome, Runner};
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

/// Runs **one** on-demand corpus reconcile and projects it into the shared
/// [`Drift`] and [`Deferred`] models — the single compute both `rollup` and
/// `orient` call (S-6 / D-3), so the two views cannot diverge *and* the probes
/// run once per command, not once per view.
///
/// Store-read (`run_corpus`); drift and deferred are derived + reconcile-bound,
/// never cached in the index (S-5 / D-5).
///
/// # Errors
///
/// Returns an error if the corpus cannot be loaded from the store.
pub(crate) fn reconcile_views(store: &Store) -> anyhow::Result<(Drift, Deferred)> {
    let report = Runner::new(store).run_corpus().context("running the corpus reconcile")?;
    Ok((project_drift(&report), project_deferred(&report)))
}

/// Projects a corpus report into the [`Drift`] model (pure — no I/O).
fn project_drift(report: &CorpusReport) -> Drift {
    let mut holds = 0;
    let mut drifted = Vec::new();
    let mut errored = Vec::new();
    for node in &report.nodes {
        for fact in &node.results {
            match &fact.outcome {
                ProbeOutcome::Holds => holds += 1,
                ProbeOutcome::Drifted { expected, observed } => drifted.push(DriftedFact {
                    node_id: node.node_id,
                    number: node.number,
                    name: node.name.clone(),
                    fact_id: fact.fact_id.clone(),
                    describe: fact.describe.clone(),
                    expected: expected.clone(),
                    observed: observed.clone(),
                }),
                ProbeOutcome::Error { reason } => errored.push(ErroredFact {
                    node_id: node.node_id,
                    number: node.number,
                    name: node.name.clone(),
                    fact_id: fact.fact_id.clone(),
                    describe: fact.describe.clone(),
                    reason: reason.clone(),
                }),
            }
        }
    }
    Drift::new(holds, drifted, errored)
}

/// Projects a corpus report into the [`Deferred`] model (pure — no I/O): each
/// node carrying a `deferred` marker, with its re-entry status resolved from the
/// referenced `desired_fact`'s outcome (`Holds` → ready; drift/error/missing →
/// still waiting).
fn project_deferred(report: &CorpusReport) -> Deferred {
    let mut nodes = Vec::new();
    for node in &report.nodes {
        let Some(marker) = &node.deferred else {
            continue;
        };
        let reentry = match node.results.iter().find(|fact| fact.fact_id == marker.reenter_when) {
            Some(fact) => match &fact.outcome {
                ProbeOutcome::Holds => Reentry::Ready,
                ProbeOutcome::Drifted { .. } | ProbeOutcome::Error { .. } => {
                    Reentry::Waiting { describe: fact.describe.clone() }
                }
            },
            // Dangling `reenter_when` — surfaced honestly here and flagged by
            // `check` (never a panic).
            None => Reentry::Waiting {
                describe: format!("(re-entry fact `{}` not declared)", marker.reenter_when),
            },
        };
        nodes.push(DeferredNode {
            node_id: node.node_id,
            number: node.number,
            name: node.name.clone(),
            because: marker.because.clone(),
            reentry,
        });
    }
    Deferred::new(nodes)
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

    if json {
        let report_json = ReconcileReport {
            schema: RECONCILE_SCHEMA,
            ok: code == EXIT_OK,
            counts: CountsJson::from(&counts),
            nodes: report.nodes.iter().map(NodeJson::from).collect(),
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&report_json)?)?;
        return Ok(code);
    }

    render_human(out, &counts, &report.nodes, strict)?;
    Ok(code)
}

/// Renders the human report: a clean run is one plain "no drift" line (no
/// fabricated data); otherwise a header plus one entry per drifted / errored
/// fact (holds are not listed — only findings).
fn render_human(
    out: &mut dyn Write,
    counts: &OutcomeCounts,
    nodes: &[NodeReport],
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
    for node in nodes {
        for fact in &node.results {
            match &fact.outcome {
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

impl From<&NodeReport> for NodeJson {
    fn from(node: &NodeReport) -> Self {
        Self {
            node_id: node.node_id.to_string(),
            number: node.number,
            name: node.name.clone(),
            results: node.results.iter().map(FactJson::from).collect(),
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

impl From<&odm_reconcile::FactResult> for FactJson {
    fn from(fact: &odm_reconcile::FactResult) -> Self {
        Self {
            fact_id: fact.fact_id.clone(),
            describe: fact.describe.clone(),
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
