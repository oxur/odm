//! The probe contract: the [`Probe`] trait and its three-way [`ProbeOutcome`].

/// The result of evaluating one [`DesiredFact`](odm_core::DesiredFact) against
/// reality.
///
/// The split is **three-way and load-bearing**: "I checked and reality has
/// drifted" ([`Drifted`](ProbeOutcome::Drifted)) is a distinct, actionable
/// finding from "I could not check at all" ([`Error`](ProbeOutcome::Error)). A
/// `bool` would collapse the two and silently report an unrunnable probe as
/// "no drift" — exactly the dishonesty reconciliation exists to prevent. The
/// reconciler (slice03) maps `Drifted` and `Error` to different severities and
/// exit codes; do not merge them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// Observed reality matches the declared expectation.
    Holds,
    /// Reality was observed and **diverges** from the expectation — the finding
    /// that matters. Both sides are carried, human-readable, for reporting.
    Drifted {
        /// What the fact declared should be true.
        expected: String,
        /// What was actually observed.
        observed: String,
    },
    /// The probe **could not be evaluated** (command not found, I/O error,
    /// abnormal termination). This is "couldn't check", never "checked, holds"
    /// and never "checked, drifted".
    Error {
        /// Why the probe could not be evaluated.
        reason: String,
    },
}

/// A single checkable probe: evaluate the declared expectation against reality
/// and report a [`ProbeOutcome`].
///
/// Object-safe and intentionally minimal — the [`Runner`](crate::Runner) holds
/// probes behind this trait and treats every kind uniformly. New probe kinds
/// implement the same one method.
pub trait Probe {
    /// Evaluate this probe against the live local environment.
    ///
    /// Returns [`ProbeOutcome::Holds`] when reality matches, `Drifted` when it
    /// diverges, and `Error` when the check itself could not be carried out.
    fn evaluate(&self) -> ProbeOutcome;
}
