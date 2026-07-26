//! Themed status lines, written to a caller-supplied writer (RH C-1 / `F-1`).
//!
//! The house style for status output is [`oxur_term::common::output`] —
//! `success`/`error`/`info`/`warning`, four thin wrappers over `colored`. Its
//! helpers write with `println!`/`eprintln!` straight to the *process's*
//! streams, but odm's command surface threads `out`/`err` as
//! `&mut dyn Write` so [`crate::dispatch`] can be driven in-process by tests
//! with no global state. A command therefore cannot call them.
//!
//! These four helpers emit the **same glyphs and colours** to a writer the
//! caller supplies. `oxur_term::common::output` is still called directly in
//! [`crate::run`], where the sink really is the process's stderr.
//!
//! *Upstream follow-up:* the duplication disappears if `oxur-term` grows
//! string-returning variants (`success_str(&str) -> String`, …) or
//! writer-taking ones; then these become one-line forwards. Kept deliberately
//! small — four format strings — until it does.
//!
//! Colour is `colored`'s decision, not ours: it suppresses ANSI when the
//! process's stdout is not a terminal, and honours `NO_COLOR`/`CLICOLOR`. So a
//! captured or piped run gets plain text.

use std::io::Write;

use colored::Colorize;

/// A completed action: `✓ <msg>`, the tick in green.
pub(crate) fn success(w: &mut dyn Write, msg: &str) -> std::io::Result<()> {
    writeln!(w, "{} {msg}", "✓".green().bold())
}

/// A failure: `Error: <msg>`, the prefix in red.
pub(crate) fn error(w: &mut dyn Write, msg: &str) -> std::io::Result<()> {
    writeln!(w, "{} {msg}", "Error:".red().bold())
}

/// An informational line — a plan, a no-op, a resolved context: `→ <msg>`, the
/// arrow in cyan.
pub(crate) fn info(w: &mut dyn Write, msg: &str) -> std::io::Result<()> {
    writeln!(w, "{} {msg}", "→".cyan())
}

/// A caveat that did not fail the run: `Warning: <msg>`, the prefix in yellow.
pub(crate) fn warning(w: &mut dyn Write, msg: &str) -> std::io::Result<()> {
    writeln!(w, "{} {msg}", "Warning:".yellow().bold())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Renders `f` to a buffer and returns it as a string.
    fn render(f: impl FnOnce(&mut dyn Write) -> std::io::Result<()>) -> String {
        let mut buf: Vec<u8> = Vec::new();
        f(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn test_success_prefixes_a_tick_and_keeps_the_message() {
        let out = render(|w| success(w, "created slice #5"));
        assert!(out.contains('✓'), "tick glyph present: {out:?}");
        assert!(out.contains("created slice #5"), "message preserved: {out:?}");
        assert!(out.ends_with('\n'), "line-terminated: {out:?}");
    }

    #[test]
    fn test_error_prefixes_the_error_label() {
        let out = render(|w| error(w, "no such node"));
        assert!(out.contains("Error:"), "label present: {out:?}");
        assert!(out.contains("no such node"), "message preserved: {out:?}");
    }

    #[test]
    fn test_info_prefixes_an_arrow() {
        let out = render(|w| info(w, "would create slice #5"));
        assert!(out.contains('→'), "arrow glyph present: {out:?}");
        assert!(out.contains("would create slice #5"), "message preserved: {out:?}");
    }

    #[test]
    fn test_warning_prefixes_the_warning_label() {
        let out = render(|w| warning(w, "nothing to do"));
        assert!(out.contains("Warning:"), "label present: {out:?}");
        assert!(out.contains("nothing to do"), "message preserved: {out:?}");
    }
}
