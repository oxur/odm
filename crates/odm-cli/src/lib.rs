//! `odm-cli` — the clap command surface for odm.
//!
//! Stub for the v1.0.0 workspace skeleton (slice 01): it defines the top-level
//! `odm` command so the umbrella binary can report `--version` and `--help`.
//! Real subcommands (`new`/`list`/`show`/`rename`/`retire`/`supersede`, plus
//! `use`/`context`) arrive in Arc 01 slices 05-06.

use clap::Parser;

/// The `odm` command-line interface.
#[derive(Debug, Parser)]
#[command(name = "odm", version, about = "The Odd Document Manager")]
pub struct Cli {}

/// Parse arguments and dispatch.
///
/// Currently a stub: `clap` handles `--version` and `--help`; any other
/// invocation parses to an empty command and returns `Ok(())`.
///
/// # Errors
///
/// Returns [`anyhow::Error`] once subcommands add fallible work (slices 05-06).
/// Argument-parse failures are handled by `clap`, which exits the process
/// directly rather than returning here.
pub fn run() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn smoke() {
        // clap verifies the command tree is well-formed (no duplicate args,
        // valid names, etc.). Fails loudly if the derive is misconfigured.
        Cli::command().debug_assert();
    }
}
