//! Thin `odm-cli` binary: drives [`odm_cli::run`] so the command surface can be
//! exercised end-to-end (e.g. by `assert_cmd`) under `cargo test -p odm-cli`.
//! The published binary is `odm`, built by the `oxur-odm` umbrella, which calls
//! the same entry point.

use std::process::ExitCode;

fn main() -> ExitCode {
    match odm_cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
