//! The themed table odm renders every tabular answer through (RH C-1 / `F-1`).
//!
//! The shape is the Oxur one — the same one `oxur-odm` rendered before the
//! rebuild, and what `oxur_term::table::TableStyleConfig` is written to paint:
//!
//! ```text
//! row 0      title bar     (bright orange)
//! row 1      column names  (mid orange)
//! rows 2..n  data          (alternating dark bands, each cell space-indented)
//! last row   summary       (`Total: …`, dark orange bar)
//! ```
//!
//! The title and summary rows are **not decoration**: the default theme sets
//! `title.enabled` and `footer.enabled`, so it always paints row 0 as the title
//! bar and the last row as the footer bar. A table handed to it without those
//! rows comes back with its header painted as a title and its first data row
//! painted as the header — so supplying them is what makes the theme correct,
//! not just fuller.
//!
//! [`oxur_term::table::OxurTable`] renders this shape all but exactly: it can
//! add a title, but its footer row is necessarily blank (no summary text), and
//! its theme is private, so cell padding cannot be set. Both are wanted here, so
//! this module drives the same theme through the crate's public lower level
//! ([`Builder`] + [`TableStyleConfig::apply_to_table`]) — the path
//! `oxur-odm` itself used. *Upstream follow-up:* `OxurTable::with_theme(…)` and
//! a text-carrying footer would let this collapse back onto `OxurTable`.

use oxur_term::table::{Builder, TableStyleConfig};
use tabled::settings::Span;
use tabled::settings::object::Cell;

/// The one-space indent every data cell carries. The Oxur theme sets
/// `padding_left = 0` (its bands run edge to edge), so the breathing room lives
/// in the cell text — as it did in `oxur-odm`.
const INDENT: &str = " ";

/// The trailing space closing each row's **last** cell, so the text does not run
/// flush into the right edge of the band. Interior cells need none — the column
/// separator already spaces them.
const TRAILER: &str = " ";

/// A table in the Oxur house shape: title bar, column names, indented data
/// rows, summary bar.
pub(crate) struct Themed {
    /// The title-bar text (row 0).
    title: String,
    /// The column names (row 1).
    headers: Vec<String>,
    /// The data rows, each already stringified, one entry per column.
    rows: Vec<Vec<String>>,
    /// The summary-bar text (last row), e.g. `Total: 59 nodes`.
    summary: String,
}

impl Themed {
    /// Starts a table with `title` on the bar and `headers` as its columns.
    pub(crate) fn new<S: AsRef<str>>(title: &str, headers: &[S]) -> Self {
        Self {
            title: title.to_string(),
            headers: headers.iter().map(|h| h.as_ref().to_string()).collect(),
            rows: Vec::new(),
            summary: String::new(),
        }
    }

    /// Appends one data row. Cells beyond the header count are ignored and
    /// missing ones are blank, so a row can never widen the table.
    pub(crate) fn row<S: Into<String>>(&mut self, cells: impl IntoIterator<Item = S>) {
        let mut row: Vec<String> =
            cells.into_iter().take(self.headers.len()).map(Into::into).collect();
        row.resize(self.headers.len(), String::new());
        self.rows.push(row);
    }

    /// Sets the summary-bar text (the last line of the table).
    pub(crate) fn summary(&mut self, summary: impl Into<String>) {
        self.summary = summary.into();
    }

    /// How many data rows have been pushed — for phrasing the summary.
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    /// Renders the table to a themed string (ANSI included).
    pub(crate) fn render(self) -> String {
        let cols = self.headers.len();
        let mut builder = Builder::default();

        // Row 0: the title bar. Row 1: the column names, flush left — the data
        // below them is what carries the indent.
        builder.push_record(bar_row(&self.title, cols));
        builder.push_record(close_last(self.headers.clone()));
        for row in &self.rows {
            let cells = row.iter().map(|c| format!("{INDENT}{c}")).collect();
            builder.push_record(close_last(cells));
        }
        // Last row: the summary bar.
        builder.push_record(bar_row(&self.summary, cols));

        let mut table = builder.build();
        // The bars carry one long string in their first cell; without a span it
        // would widen column 0 and drag every row out with it.
        let last = self.rows.len() + 2;
        table.modify(Cell::new(0, 0), Span::column(cols));
        table.modify(Cell::new(last, 0), Span::column(cols));

        // The theme is taken unmodified — the indent lives in the cell text
        // (see `INDENT`) so that the column names stay flush against the bars
        // while the data sits one space in, which is the `oxur-odm` look.
        // `apply_to_table`'s type parameter is unused (it reads only the config
        // and the table it is handed); `String` satisfies the bound.
        TableStyleConfig::default().apply_to_table::<String>(&mut table);
        table.to_string()
    }
}

/// A full-width bar row: the text in the first cell, the rest blank (the span
/// applied in [`Themed::render`] stretches it across the table).
fn bar_row(text: &str, cols: usize) -> Vec<String> {
    let mut row = vec![format!("{INDENT}{text}")];
    row.resize(cols.max(1), String::new());
    row
}

/// Closes a row with [`TRAILER`] on its last cell.
fn close_last(mut row: Vec<String>) -> Vec<String> {
    if let Some(last) = row.last_mut() {
        last.push_str(TRAILER);
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Strips ANSI escapes so assertions read the visible text.
    fn plain(s: &str) -> String {
        let mut out = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\u{1b}' {
                for c in chars.by_ref() {
                    if c == 'm' {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    fn sample() -> Themed {
        let mut t = Themed::new("NODES", &["NUMBER", "TYPE"]);
        t.row(["1100", "arc"]);
        t.row(["1101", "slice"]);
        t.summary(format!("Total: {} nodes", t.len()));
        t
    }

    #[test]
    fn test_render_puts_the_title_first_and_the_columns_second() {
        let out = plain(&sample().render());
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].contains("NODES"), "line 1 is the title: {:?}", lines[0]);
        assert!(!lines[0].contains("NUMBER"), "the title bar is its own line: {:?}", lines[0]);
        assert!(lines[1].contains("NUMBER"), "line 2 is the columns: {:?}", lines[1]);
        assert!(lines[1].contains("TYPE"), "line 2 is the columns: {:?}", lines[1]);
    }

    #[test]
    fn test_render_ends_with_the_summary_line() {
        let out = plain(&sample().render());
        let last = out.lines().next_back().unwrap();
        assert!(last.contains("Total: 2 nodes"), "last line is the summary: {last:?}");
    }

    #[test]
    fn test_render_indents_every_data_cell_by_one_space() {
        let out = plain(&sample().render());
        let data: Vec<&str> = out.lines().skip(2).take(2).collect();
        assert!(data[0].starts_with(" 1100"), "first cell indented: {:?}", data[0]);
        assert!(data[0].contains(" arc"), "later cells indented: {:?}", data[0]);
        assert!(data[1].starts_with(" 1101"), "first cell indented: {:?}", data[1]);
    }

    #[test]
    fn test_render_closes_every_row_with_a_trailing_space() {
        let out = plain(&sample().render());
        // Header + both data rows: the last cell never runs flush to the edge.
        for line in out.lines().take(4).skip(1) {
            assert!(line.ends_with(' '), "row ends with a space: {line:?}");
        }
        // The widest last-column cell is `slice` (5) + indent + trailer = 7.
        let header = out.lines().nth(1).unwrap();
        assert!(header.ends_with("TYPE   "), "the trailer widens the column: {header:?}");
    }

    #[test]
    fn test_render_bars_do_not_widen_the_first_column() {
        // The summary is far longer than any NUMBER cell; the span must absorb
        // it rather than stretching column 0.
        let mut t = Themed::new("N", &["NUMBER", "TYPE"]);
        t.row(["1100", "arc"]);
        t.summary("Total: 1 node — a summary far wider than the number column");
        let out = plain(&t.render());
        let data = out.lines().nth(2).unwrap();
        assert!(data.starts_with(" 1100 "), "column 0 stayed narrow: {data:?}");
    }

    #[test]
    fn test_row_is_clamped_to_the_header_count() {
        let mut t = Themed::new("T", &["A", "B"]);
        t.row(["1", "2", "3"]); // extra cell dropped
        t.row(["1"]); // missing cell blank
        assert_eq!(t.len(), 2);
        t.summary("Total: 2");
        let out = plain(&t.render());
        assert!(!out.contains('3'), "the extra cell is dropped: {out:?}");
    }
}
