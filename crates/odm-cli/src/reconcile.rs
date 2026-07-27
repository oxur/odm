//! The `odm reconcile` command (arc05 slice03) and the shared drift/deferred
//! projectors (slice04/06), wired onto the **incremental** freshness mechanism
//! (slice08, ODD-0019).
//!
//! - Bare commands (`orient`/`rollup`) call [`reconcile_views`], which runs the
//!   **incremental** reconcile (slice07): input-derived facts are re-probed only
//!   when an input changed; **volatile facts are never run** — they are carried
//!   from the persisted `.odm/drift` snapshot with a "last checked" stamp. This
//!   dissolves the slice04 regression (bare `odm` runs zero volatile probes).
//! - `odm reconcile` calls the **full** reconcile: it re-probes everything
//!   (input-derived *and* volatile) and re-stamps the snapshot — the sanctioned
//!   "refresh the volatile facts now" verb.
//!
//! The projections join the snapshot's outcomes with the corpus's declared facts
//! (for render identity + never-checked detection); freshness is `Fresh` for
//! input-derived facts and `LastChecked` for volatile ones (honest staleness).
//! No index reader is added — the freshness home is the drift snapshot.

use std::io::Write;

use anyhow::Context as _;
use odm_core::frontmatter::Document;
use odm_core::rollup::{
    Deferred, DeferredNode, Drift, DriftedFact, ErroredFact, Freshness, Reentry, UncheckedFact,
};
use odm_reconcile::{
    DriftSnapshot, FactState, ProbeOutcome, default_drift_path, reconcile_full,
    reconcile_incremental,
};
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

/// A tally of checked outcomes (mirrors `odm_reconcile::OutcomeCounts` for the
/// pure verdict; recomputed here from the snapshot/corpus join).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Counts {
    holds: usize,
    drifted: usize,
    errored: usize,
}

/// Maps outcome counts to a [`Verdict`] — the pure severity/exit policy (R-4).
fn verdict(counts: &Counts) -> Verdict {
    if counts.drifted > 0 {
        Verdict { severity: "error", exit: EXIT_VIOLATIONS, strict_exit: EXIT_VIOLATIONS }
    } else if counts.errored > 0 {
        Verdict { severity: "warning", exit: EXIT_OK, strict_exit: EXIT_VIOLATIONS }
    } else {
        Verdict { severity: "clean", exit: EXIT_OK, strict_exit: EXIT_OK }
    }
}

/// The freshness of a snapshot entry: `Fresh` for input-derived facts,
/// `LastChecked` for volatile ones (ODD-0019 §3.4).
fn entry_freshness(state: &FactState) -> Freshness {
    match state {
        FactState::InputDerived { .. } => Freshness::Fresh,
        FactState::Volatile { last_checked } => Freshness::LastChecked { at: *last_checked },
    }
}

/// Runs the **incremental** reconcile and projects it into the shared [`Drift`]
/// and [`Deferred`] models — the single compute both `rollup` and `orient` call
/// (so the two views cannot diverge). Input-derived drift is fresh at near-zero
/// cost; volatile facts are carried from the snapshot with staleness; **zero
/// volatile probes run** (the slice04 regression dissolved).
///
/// # Errors
///
/// Returns an error if the reconcile or corpus load fails.
pub(crate) fn reconcile_views(store: &Store) -> anyhow::Result<(Drift, Deferred)> {
    let snapshot = reconcile_incremental(store, &default_drift_path(store.root()))
        .context("running the incremental drift reconcile")?;
    // The snapshot carries outcomes + freshness by (node, fact) id; join with the
    // corpus for render identity (number/name/describe) and never-checked facts.
    let documents = store.load_all().context("loading the corpus to project drift")?;
    Ok((project_drift(&snapshot, &documents), project_deferred(&snapshot, &documents)))
}

/// Projects the drift snapshot + corpus into the [`Drift`] model (pure). Iterates
/// the corpus's *declared* facts (the source of what exists) and reads each
/// outcome from the snapshot: a volatile fact with no entry is **never-checked**;
/// every other fact has an entry with its outcome and freshness.
fn project_drift(snapshot: &DriftSnapshot, documents: &[Document]) -> Drift {
    let mut holds = 0;
    let mut drifted = Vec::new();
    let mut errored = Vec::new();
    let mut unchecked = Vec::new();
    for document in documents {
        let fm = document.frontmatter();
        for fact in fm.desired_facts() {
            match snapshot.entry(fm.id(), &fact.id) {
                None => unchecked.push(UncheckedFact {
                    node_id: fm.id(),
                    number: fm.number(),
                    name: fm.name().to_string(),
                    fact_id: fact.id.clone(),
                    describe: fact.describe.clone(),
                }),
                Some(entry) => {
                    let freshness = entry_freshness(&entry.state);
                    match &entry.outcome {
                        ProbeOutcome::Holds => holds += 1,
                        ProbeOutcome::Drifted { expected, observed } => drifted.push(DriftedFact {
                            node_id: fm.id(),
                            number: fm.number(),
                            name: fm.name().to_string(),
                            fact_id: fact.id.clone(),
                            describe: fact.describe.clone(),
                            expected: expected.clone(),
                            observed: observed.clone(),
                            freshness,
                        }),
                        ProbeOutcome::Error { reason } => errored.push(ErroredFact {
                            node_id: fm.id(),
                            number: fm.number(),
                            name: fm.name().to_string(),
                            fact_id: fact.id.clone(),
                            describe: fact.describe.clone(),
                            reason: reason.clone(),
                            freshness,
                        }),
                    }
                }
            }
        }
    }
    Drift::new(holds, drifted, errored, unchecked)
}

/// Projects the drift snapshot + corpus into the [`Deferred`] model (pure): each
/// node carrying a `deferred` marker, with its re-entry status resolved from the
/// referenced fact's snapshot outcome (`Holds` → ready; drift/error/missing/
/// not-yet-checked → still waiting).
fn project_deferred(snapshot: &DriftSnapshot, documents: &[Document]) -> Deferred {
    let mut nodes = Vec::new();
    for document in documents {
        let fm = document.frontmatter();
        let Some(marker) = fm.deferred() else {
            continue;
        };
        let declared = fm.desired_facts().iter().find(|fact| fact.id == marker.reenter_when);
        let reentry = match (declared, snapshot.entry(fm.id(), &marker.reenter_when)) {
            (_, Some(entry)) => match &entry.outcome {
                ProbeOutcome::Holds => Reentry::Ready,
                ProbeOutcome::Drifted { .. } | ProbeOutcome::Error { .. } => Reentry::Waiting {
                    describe: declared
                        .map_or_else(|| marker.reenter_when.clone(), |fact| fact.describe.clone()),
                },
            },
            // Declared but never checked (volatile, no explicit reconcile yet).
            (Some(fact), None) => {
                Reentry::Waiting { describe: format!("{} (not yet checked)", fact.describe) }
            }
            // Dangling `reenter_when` — surfaced honestly; `check` flags it too.
            (None, None) => Reentry::Waiting {
                describe: format!("(re-entry fact `{}` not declared)", marker.reenter_when),
            },
        };
        nodes.push(DeferredNode {
            node_id: fm.id(),
            number: fm.number(),
            name: fm.name().to_string(),
            because: marker.because.clone(),
            reentry,
        });
    }
    Deferred::new(nodes)
}

/// Runs the **full** reconcile (`odm reconcile`) — re-probes every fact and
/// re-stamps the snapshot — then renders the report, returning the exit code
/// ([`EXIT_OK`] when clean / warnings-without-`--strict`, [`EXIT_VIOLATIONS`] on
/// drift or, under `--strict`, a probe error).
///
/// # Errors
///
/// Returns an error (mapped to exit code `2`) if the reconcile or corpus load
/// fails.
pub(crate) fn reconcile(
    store: &Store,
    strict: bool,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<u8> {
    let snapshot = reconcile_full(store, &default_drift_path(store.root()))
        .context("running the full drift reconcile")?;
    let documents = store.load_all().context("loading the corpus to report drift")?;
    let drift = project_drift(&snapshot, &documents);
    let counts =
        Counts { holds: drift.holds, drifted: drift.drifted.len(), errored: drift.errored.len() };
    let code = verdict(&counts).exit_code(strict);

    if json {
        let report_json = ReconcileReport {
            schema: RECONCILE_SCHEMA,
            ok: code == EXIT_OK,
            counts: CountsJson::from(&counts),
            nodes: nodes_json(&snapshot, &documents),
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&report_json)?)?;
        return Ok(code);
    }

    render_human(out, &counts, &drift, strict)?;
    Ok(code)
}

/// The reconcile report as a JSON value, for the composite `check` to embed
/// (ODD-0023 §6a) rather than re-deriving or re-parsing printed output.
///
/// # Errors
///
/// As [`reconcile`].
pub(crate) fn reconcile_value(
    store: &Store,
    strict: bool,
) -> anyhow::Result<(u8, serde_json::Value)> {
    let snapshot = reconcile_full(store, &default_drift_path(store.root()))
        .context("running the full drift reconcile")?;
    let documents = store.load_all().context("loading the corpus to report drift")?;
    let drift = project_drift(&snapshot, &documents);
    let counts =
        Counts { holds: drift.holds, drifted: drift.drifted.len(), errored: drift.errored.len() };
    let code = verdict(&counts).exit_code(strict);
    let report = ReconcileReport {
        schema: RECONCILE_SCHEMA,
        ok: code == EXIT_OK,
        counts: CountsJson::from(&counts),
        nodes: nodes_json(&snapshot, &documents),
    };
    Ok((code, serde_json::to_value(report)?))
}

/// Renders the reconcile human report: a clean run is one plain "no drift" line
/// (no fabricated data); otherwise a header plus one entry per drifted / errored
/// fact (holds are not listed — only findings), with staleness where volatile.
fn render_human(
    out: &mut dyn Write,
    counts: &Counts,
    drift: &Drift,
    strict: bool,
) -> anyhow::Result<()> {
    if counts.drifted == 0 && counts.errored == 0 {
        if counts.holds == 0 {
            crate::term::success(out, "reconcile: no drift (no desired_facts declared)")?;
        } else {
            crate::term::success(
                out,
                &format!("reconcile: no drift ({} fact(s) hold)", counts.holds),
            )?;
        }
        return Ok(());
    }

    // Confirmed drift fails the run; a couldn't-check-only run is a warning
    // (it fails only under `--strict`).
    let verdict = format!(
        "reconcile: {} drifted, {} couldn't-check ({} held)",
        counts.drifted, counts.errored, counts.holds
    );
    if counts.drifted > 0 {
        crate::term::error(out, &verdict)?;
    } else {
        crate::term::warning(out, &verdict)?;
    }
    for d in &drift.drifted {
        writeln!(
            out,
            "  [drift] #{} {:?} / {}: {}{}",
            d.number,
            d.name,
            d.fact_id,
            d.describe,
            staleness_suffix(d.freshness)
        )?;
        writeln!(out, "    expected: {}", d.expected)?;
        writeln!(out, "    observed: {}", d.observed)?;
    }
    for e in &drift.errored {
        writeln!(
            out,
            "  [error] #{} {:?} / {}: {}{}",
            e.number,
            e.name,
            e.fact_id,
            e.describe,
            staleness_suffix(e.freshness)
        )?;
        writeln!(out, "    reason: {}", e.reason)?;
    }
    if !strict && counts.errored > 0 && counts.drifted == 0 {
        writeln!(out, "(probe errors do not fail; run with --strict to enforce)")?;
    }
    Ok(())
}

/// The human staleness suffix for a finding: empty for a fresh (input-derived)
/// fact, " (last checked Xm ago)" for a volatile one.
pub(crate) fn staleness_suffix(freshness: Freshness) -> String {
    match freshness {
        Freshness::Fresh => String::new(),
        Freshness::LastChecked { at } => format!(" (last checked {} ago)", humanize_age(at)),
    }
}

/// Formats the age of a Unix-seconds timestamp as a coarse "Xs/Xm/Xh/Xd"
/// human string, relative to now.
pub(crate) fn humanize_age(at: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(at);
    let secs = (now - at).max(0);
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86_400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86_400)
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

impl From<&Counts> for CountsJson {
    fn from(counts: &Counts) -> Self {
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

/// JSON shape of one fact's result: `outcome` is the `ProbeOutcome` tagged on
/// `kind`; `last_checked` (additive, slice08) is the volatile last-checked stamp
/// (`null` for a fresh input-derived fact).
#[derive(Serialize)]
struct FactJson {
    fact_id: String,
    describe: String,
    outcome: ProbeOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_checked: Option<i64>,
}

/// Builds the `reconcile/v1` node list from the snapshot + corpus (a full run has
/// an entry for every declared fact).
fn nodes_json(snapshot: &DriftSnapshot, documents: &[Document]) -> Vec<NodeJson> {
    let mut nodes = Vec::new();
    for document in documents {
        let fm = document.frontmatter();
        let mut results = Vec::new();
        for fact in fm.desired_facts() {
            if let Some(entry) = snapshot.entry(fm.id(), &fact.id) {
                let last_checked = match &entry.state {
                    FactState::Volatile { last_checked } => Some(*last_checked),
                    FactState::InputDerived { .. } => None,
                };
                results.push(FactJson {
                    fact_id: fact.id.clone(),
                    describe: fact.describe.clone(),
                    outcome: entry.outcome.clone(),
                    last_checked,
                });
            }
        }
        if !results.is_empty() {
            nodes.push(NodeJson {
                node_id: fm.id().to_string(),
                number: fm.number(),
                name: fm.name().to_string(),
                results,
            });
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(holds: usize, drifted: usize, errored: usize) -> Counts {
        Counts { holds, drifted, errored }
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

    /// Staleness suffix: fresh is silent; volatile reads "last checked Xm ago".
    #[test]
    fn staleness_suffix_is_honest() {
        assert_eq!(staleness_suffix(Freshness::Fresh), "");
        let now =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                as i64;
        let s = staleness_suffix(Freshness::LastChecked { at: now - 120 });
        assert!(s.contains("last checked") && s.contains("2m ago"), "got {s:?}");
    }
}
