//! The `odm` binary — thin umbrella entry point.
//!
//! Delegates to `odm-cli`. Domain behavior is implemented in the library crates
//! (`odm-core`/`odm-store`/`odm-graph`) and surfaced through `odm-cli`. This is
//! the v1.0.0 workspace skeleton (slice 01): the binary reports `--version` and
//! otherwise does nothing yet.

use std::process::ExitCode;

fn main() -> ExitCode {
    match odm_cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(env!("CARGO_PKG_NAME"), "oxur-odm");
    }
}
