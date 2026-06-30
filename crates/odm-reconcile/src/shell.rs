//! The shell probe (slice01): run an author-declared command, compare its
//! result to the declared expectation.

use std::process::Command;

use odm_core::ShellExpect;

use crate::probe::{Probe, ProbeOutcome};

/// The **shell** probe: run an author-declared command and map its result to a
/// [`ProbeOutcome`] — `Holds` when the exit code (and optional stdout match)
/// meet the expectation, `Drifted` when they diverge, `Error` when the command
/// cannot run.
///
/// # Trust model
///
/// The shell probe runs **author-declared commands locally with the invoking
/// user's own privileges** — the same trust boundary as a git hook, a `make`
/// target, or a `build.rs` in your own repository. The command text comes from
/// a node file you (or your collaborators) wrote and committed. There is
/// **no sandbox** in the MVP: a probe can do anything the user running `odm`
/// can do. The boundary is therefore "you ran a command written in your own
/// node files," stated here so it is an explicit decision and not an
/// assumption.
///
/// Guardrails for an *untrusted* or multi-author corpus — a `--no-exec` /
/// dry-run mode, or an allowlist of permitted commands — were considered and
/// **deferred**: they belong to a threat model (running probes from a corpus
/// you do not trust) that is out of MVP scope. They are a no-op in this slice
/// by design, not by omission.
///
/// # Command form
///
/// `run` is whitespace-tokenized into a program and its arguments and executed
/// **directly** — it is *not* passed to a shell interpreter. So shell
/// metacharacters (pipes, redirects, `&&`, globbing, `$VAR` expansion) are
/// **not** interpreted in the MVP. Executing directly is a deliberate choice:
/// it makes "the command cannot run" (a spawn failure → [`ProbeOutcome::Error`])
/// genuinely distinct from "the command ran and diverged" (a non-zero exit →
/// [`ProbeOutcome::Drifted`]). Routing through `sh -c` would turn a missing
/// binary into exit code 127 — a *divergence*, collapsing the load-bearing
/// Error/Drift distinction. Richer shell semantics, if needed, are a later
/// extension to the spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellProbe {
    run: String,
    expect: ShellExpect,
}

impl ShellProbe {
    /// Builds a shell probe from a command line and its expectation. `run` is
    /// the `program arg1 arg2 …` form described on the type.
    #[must_use]
    pub fn new(run: impl Into<String>, expect: ShellExpect) -> Self {
        Self { run: run.into(), expect }
    }

    /// The expectation rendered for a `Drifted.expected` message.
    fn expectation(&self) -> String {
        match &self.expect.stdout_contains {
            Some(needle) => format!("exit {}, stdout contains {needle:?}", self.expect.exit),
            None => format!("exit {}", self.expect.exit),
        }
    }

    /// What was observed, rendered for a `Drifted.observed` message. Stdout is
    /// truncated so a chatty command cannot produce an unbounded report.
    fn observation(&self, code: i32, stdout: &str) -> String {
        match &self.expect.stdout_contains {
            Some(_) => format!("exit {code}, stdout {:?}", crate::file::truncate(stdout, 200)),
            None => format!("exit {code}"),
        }
    }
}

impl Probe for ShellProbe {
    fn evaluate(&self) -> ProbeOutcome {
        let mut parts = self.run.split_whitespace();
        let Some(program) = parts.next() else {
            return ProbeOutcome::Error { reason: "empty command".to_string() };
        };
        let args: Vec<&str> = parts.collect();

        let output = match Command::new(program).args(&args).output() {
            Ok(output) => output,
            // Spawn failed: the command cannot run (not found, not executable,
            // I/O error). This is "couldn't check", not drift.
            Err(err) => {
                return ProbeOutcome::Error { reason: format!("could not run `{program}`: {err}") };
            }
        };

        // No exit code means the process was terminated by a signal — it ran
        // but produced no code to compare against, so the expectation could not
        // be evaluated.
        let Some(code) = output.status.code() else {
            return ProbeOutcome::Error {
                reason: format!("`{program}` terminated by a signal without an exit code"),
            };
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let exit_ok = code == self.expect.exit;
        let stdout_ok = match &self.expect.stdout_contains {
            Some(needle) => stdout.contains(needle.as_str()),
            None => true,
        };

        if exit_ok && stdout_ok {
            ProbeOutcome::Holds
        } else {
            ProbeOutcome::Drifted {
                expected: self.expectation(),
                observed: self.observation(code, &stdout),
            }
        }
    }
}
