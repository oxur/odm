//! Integration tests for the `Probe` trait, the three-way `ProbeOutcome`, and
//! the shell probe (arc05 slice01). Public API only, so library `src/` stays
//! panic-free. Test names carry the substrings the ledger Verify commands
//! filter on (`probe_outcome_has_three_variants`, `shell_probe_holds_on_match`,
//! `shell_probe_drifts_on_divergence`, `shell_probe_errors_on_unrunnable`).
//!
//! The shell-probe cases run **real local commands** (`true`, `false`, `echo`,
//! `printf`, a missing binary) — the freeze-harness pattern, exercised directly
//! rather than through a runner (the runner is slice02).

use odm_core::ShellExpect;
use odm_reconcile::{Probe, ProbeOutcome, ShellProbe};

fn expect(exit: i32) -> ShellExpect {
    ShellExpect { exit, stdout_contains: None }
}

fn expect_stdout(exit: i32, needle: &str) -> ShellExpect {
    ShellExpect { exit, stdout_contains: Some(needle.to_string()) }
}

// ----- F-3: the three-way outcome model ------------------------------------

#[test]
fn probe_outcome_has_three_variants() {
    // All three are constructible and distinct — "couldn't check" (Error) is a
    // different value from "checked, drifted" (Drifted) and "checked, holds".
    let holds = ProbeOutcome::Holds;
    let drifted =
        ProbeOutcome::Drifted { expected: "exit 0".to_string(), observed: "exit 1".to_string() };
    let errored = ProbeOutcome::Error { reason: "command not found".to_string() };

    assert_ne!(holds, drifted);
    assert_ne!(drifted, errored);
    assert_ne!(holds, errored);

    // The variants carry the data the reconciler reports on.
    match drifted {
        ProbeOutcome::Drifted { expected, observed } => {
            assert_eq!(expected, "exit 0");
            assert_eq!(observed, "exit 1");
        }
        _ => panic!("expected Drifted"),
    }
    match errored {
        ProbeOutcome::Error { reason } => assert_eq!(reason, "command not found"),
        _ => panic!("expected Error"),
    }
}

// ----- F-4: the shell probe maps a command to the three outcomes -----------

#[test]
fn shell_probe_holds_on_match() {
    // `true` exits 0 — matches the expectation, so the fact holds.
    let probe = ShellProbe::new("true", expect(0));
    assert_eq!(probe.evaluate(), ProbeOutcome::Holds);
}

#[test]
fn shell_probe_holds_on_match_with_stdout() {
    // Exit *and* the optional stdout substring both met.
    let probe = ShellProbe::new("echo ready", expect_stdout(0, "ready"));
    assert_eq!(probe.evaluate(), ProbeOutcome::Holds);
}

#[test]
fn shell_probe_drifts_on_divergence() {
    // `false` exits 1 where 0 was expected — reality observed, and it diverges.
    let probe = ShellProbe::new("false", expect(0));
    match probe.evaluate() {
        ProbeOutcome::Drifted { expected, observed } => {
            assert_eq!(expected, "exit 0");
            assert_eq!(observed, "exit 1");
        }
        other => panic!("expected Drifted, got {other:?}"),
    }
}

#[test]
fn shell_probe_drifts_on_stdout_divergence() {
    // Exit matches but the required stdout substring is absent → drift. The
    // observed message reports both the exit and the (truncated) stdout.
    let probe = ShellProbe::new("echo hello", expect_stdout(0, "goodbye"));
    match probe.evaluate() {
        ProbeOutcome::Drifted { expected, observed } => {
            assert!(expected.contains("stdout contains \"goodbye\""));
            assert!(observed.contains("exit 0"));
            assert!(observed.contains("hello"));
        }
        other => panic!("expected Drifted, got {other:?}"),
    }
}

#[test]
fn shell_probe_drifts_truncates_chatty_stdout() {
    // A long stdout is truncated in the observed report (with an ellipsis), so
    // a chatty command cannot produce an unbounded finding.
    let big = "A".repeat(250);
    let probe = ShellProbe::new(format!("printf {big}"), expect_stdout(0, "ZZZ"));
    match probe.evaluate() {
        ProbeOutcome::Drifted { observed, .. } => assert!(observed.contains('…')),
        other => panic!("expected Drifted, got {other:?}"),
    }
}

#[test]
fn shell_probe_errors_on_unrunnable() {
    // A non-existent binary cannot be spawned → Error, distinct from Drift.
    let probe = ShellProbe::new("odm-no-such-binary-xyzzy --check", expect(0));
    match probe.evaluate() {
        ProbeOutcome::Error { reason } => assert!(reason.contains("odm-no-such-binary-xyzzy")),
        other => panic!("expected Error, got {other:?}"),
    }
}

#[test]
fn shell_probe_errors_on_empty_command() {
    // An all-whitespace command has no program to run → Error.
    let probe = ShellProbe::new("   ", expect(0));
    match probe.evaluate() {
        ProbeOutcome::Error { reason } => assert!(reason.contains("empty command")),
        other => panic!("expected Error, got {other:?}"),
    }
}

// A command terminated by a signal ran but produced no exit code to compare
// against — "couldn't check", so Error, not Drift. Exercised with a tiny
// self-signalling script (Unix-only: signals + the exec bit).
#[cfg(unix)]
#[test]
fn shell_probe_errors_on_signal_termination() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("tempdir");
    let script = dir.path().join("selfkill.sh");
    {
        let mut f = std::fs::File::create(&script).expect("create script");
        writeln!(f, "#!/bin/sh\nkill -TERM $$").expect("write script");
    }
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("chmod +x");

    let probe = ShellProbe::new(script.to_str().expect("utf-8 path"), expect(0));
    match probe.evaluate() {
        ProbeOutcome::Error { reason } => assert!(reason.contains("signal")),
        other => panic!("expected Error from signal termination, got {other:?}"),
    }
}
