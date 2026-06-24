//! The `odm` binary — thin umbrella entry point.
//!
//! Delegates to `odm-cli`. Domain behavior is implemented in the library crates
//! (`odm-core`/`odm-store`/`odm-graph`) and surfaced through `odm-cli`. The
//! exit code is chosen by [`odm_cli::run`] (`0` ok, `1` `check` violations,
//! `2` error).

use std::process::ExitCode;

fn main() -> ExitCode {
    odm_cli::run()
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(env!("CARGO_PKG_NAME"), "oxur-odm");
    }
}
