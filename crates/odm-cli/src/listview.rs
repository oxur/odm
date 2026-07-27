//! The `odm list` view (RH C-3): turning index records into a table that reads
//! as a **plan** rather than a dump.
//!
//! The shape, one column per finding:
//!
//! ```text
//! DATE | TYPE | STATUS | NAME (tree, de-numbered) | ID
//! ```
//!
//! - **F-4** no `NUMBER` column — `number` stays frontmatter metadata and a CLI
//!   handle; the ULID is identity.
//! - **F-5** `DATE` leads, showing `created` (`--date=updated` switches it).
//! - **F-7** `STATUS` is the **furthest-reached gate** in the node's own
//!   gate-set (`—` when none), overridden by `retired`/`superseded`.
//! - **F-8** `NAME` renders work nodes as a containment tree (`project → arc →
//!   slice` via `part_of`); document nodes have no containment parent, so they
//!   follow as their own flat group rather than being forced into the tree.
//! - **F-6** displayed names are **de-numbered** — display-only; the stored
//!   `name` is untouched.
//! - **F-9** the name column is width-bounded with ` ...` elision.
//! - **F-15** retired/superseded nodes are **excluded by default**; `--all`
//!   brings them back, marked in `STATUS`.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use odm_core::gates::{GateSet, GateSets};
use odm_core::{Id, NodeType};
use odm_index::{EdgeKind, IndexRecord};
use oxur_term::table::{TabledColor, helpers};

/// Which family of nodes to list.
///
/// The two families are the model's own (ODD-0013 §2.2 — *work* nodes and
/// *document* nodes); these are their **display names**, chosen so the flag
/// reads as a question about the corpus rather than about the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Group {
    /// Work nodes: `project`/`arc`/`slice` — the plan itself.
    Plan,
    /// Document nodes: `design`/`research`/`adr`/`note` — what the plan is
    /// grounded in and decided by; consulted, not executed.
    Reference,
}

impl Group {
    /// Whether `node_type` belongs to this group.
    fn holds(self, node_type: NodeType) -> bool {
        match self {
            Group::Plan => node_type.is_work(),
            Group::Reference => !node_type.is_work(),
        }
    }
}

/// Which date the leading column shows (F-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DateColumn {
    /// The node's creation date (the default).
    #[default]
    Created,
    /// The node's last-updated date.
    Updated,
}

/// A node's **raw** status: the furthest gate it reached, with the withdrawal
/// overlays (F-7).
///
/// No longer what the column shows — [`DisplayStatus`] is (F-19). This survives
/// because `--status` still accepts raw gate spellings (`--status tested`), so
/// the raw vocabulary has to remain derivable even though it is not rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Status {
    /// The node carries a `retired:` marker.
    Retired,
    /// Another node supersedes this one.
    Superseded,
    /// The furthest gate the node has reached in its own gate-set.
    Gate(String),
    /// No gate reached yet.
    None,
}

impl Status {
    /// The single token shown in the column.
    pub(crate) fn label(&self) -> &str {
        match self {
            Status::Retired => "retired",
            Status::Superseded => "superseded",
            Status::Gate(gate) => gate,
            Status::None => "—",
        }
    }
}

/// The **normalized** state shown in the STATUS column (F-19).
///
/// The raw furthest-reached gate is not comparable across types: a slice's
/// `tested` and an arc's `verified` both mean *done* but read differently, and
/// an arc's `complete` reads like an endpoint though `verified` is still ahead.
/// This is each node's *position in its own ladder*, so a listing can be scanned
/// down the column and mean one thing.
///
/// A **view** concept only — nothing is stored, no gate is added, and the
/// underlying gate vector is untouched (see [`Status`], which still carries it
/// for the raw `--status` filter and for `show`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DisplayStatus {
    /// At the first rung, or nothing reached yet.
    Planned,
    /// Past the first rung, not yet terminal.
    Active,
    /// The terminal gate of its own gate-set is reached.
    Done,
    /// Withdrawn: the node carries a `retired:` marker.
    Retired,
    /// Withdrawn: another node supersedes it.
    Superseded,
    /// The node's type has **no gate ladder**, so it has no position to report.
    ///
    /// Distinct from [`DisplayStatus::Planned`] on purpose: an `adr` is not
    /// "planned but not started", it simply has no lifecycle to be at the start
    /// of. Rendering it as `planned` would assert something about it that no one
    /// recorded.
    None,
}

impl DisplayStatus {
    /// The machine-facing name, or `None` when there is no ladder to have a
    /// position in.
    ///
    /// Distinct from [`Self::label`] because that returns an em-dash for the
    /// no-ladder case — a *display* glyph, meaningless to a JSON consumer, which
    /// should see `null` and know the field does not apply.
    pub(crate) fn name(&self) -> Option<&'static str> {
        match self {
            DisplayStatus::None => None,
            other => Some(other.label()),
        }
    }

    /// The single token shown in the column.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            DisplayStatus::Planned => "planned",
            DisplayStatus::Active => "active",
            DisplayStatus::Done => "done",
            DisplayStatus::Retired => "retired",
            DisplayStatus::Superseded => "superseded",
            DisplayStatus::None => "—",
        }
    }

    /// Whether this state means the node is not live work — the rows `list`
    /// hides unless asked for them (F-15).
    pub(crate) fn is_withdrawn(&self) -> bool {
        matches!(self, DisplayStatus::Retired | DisplayStatus::Superseded)
    }
}

/// Derives the normalized state from a node's position in its own gate ladder.
///
/// Pure, and deliberately takes the ladder rather than a `GateSets` — every
/// branch is then testable with two string slices and two flags, with no store,
/// no config and no corpus.
///
/// Withdrawal overlays win: a retired or superseded node reports that whatever
/// its gates say, because "how far did this get" stops being the interesting
/// question once it has been withdrawn (preserves C-3/F-15 behaviour exactly).
pub(crate) fn derive_display_status(
    reached: &[&str],
    sequence: Option<&[String]>,
    retired: bool,
    superseded: bool,
) -> DisplayStatus {
    if retired {
        return DisplayStatus::Retired;
    }
    if superseded {
        return DisplayStatus::Superseded;
    }
    let Some(sequence) = sequence.filter(|s| !s.is_empty()) else {
        return DisplayStatus::None;
    };
    // The furthest rung reached, as an index into this node's own ladder.
    let furthest =
        sequence.iter().rposition(|gate| reached.iter().any(|r| r.eq_ignore_ascii_case(gate)));
    match furthest {
        Some(i) if i == sequence.len() - 1 => DisplayStatus::Done,
        Some(0) | None => DisplayStatus::Planned,
        Some(_) => DisplayStatus::Active,
    }
}

/// A line of the rendered list: either a node, or the rule separating the work
/// tree from the document group (F-8 — the two are different kinds of thing, so
/// the eye should not have to infer the boundary).
#[derive(Debug, Clone)]
pub(crate) enum Row {
    /// A node row.
    Node(Box<NodeRow>),
    /// A horizontal rule between the two groups.
    Divider,
}

/// The colour a STATUS token renders in, or `None` to leave it uncoloured.
///
/// Two vocabularies meet here. The **normalized states** (F-19) are what the
/// column shows and are matched first — `planned` and `active` are *also* raw
/// gate names, and the derived meaning is the one on screen. The **withdrawal
/// overlays** (`retired`, `superseded`) fall through to `oxur-odm` 0.3.5's
/// palette via the very helper the original called, so those colours cannot
/// drift by being retyped.
///
/// The per-gate work palette that used to live here retired with the raw-gate
/// column: once STATUS shows a normalized state, no gate name can reach this
/// function, and a palette nothing can select is worse than no palette. Its
/// *slots* survive in [`display_status_color`] — early yellow, under way cyan,
/// done green — so the tuning outlived the mapping.
///
/// One distinction is genuinely gone, and deliberately: `complete` was green
/// and `verified` bright green, keeping ODD-0013 §5.1's "done at its layer" and
/// "verified live" apart. Normalizing folds both into `done`. The rung is still
/// exact in `odm node show` and `--json`; it is the *column* that trades that
/// detail for cross-type comparability, which is the whole of F-19.
pub(crate) fn status_color(label: &str) -> Option<TabledColor> {
    display_status_color(label).or_else(|| helpers::state_to_fg_color(label))
}

/// The colour for a **normalized** state token (F-19).
///
/// The slots are the ones already tuned for the gate vocabulary rather than new
/// hues: early is yellow, under way is cyan, done is green.
fn display_status_color(label: &str) -> Option<TabledColor> {
    match label.trim().to_ascii_lowercase().as_str() {
        "planned" => Some(TabledColor::FG_YELLOW),
        "active" => Some(TabledColor::FG_CYAN),
        "done" => Some(TabledColor::FG_GREEN),
        _ => None,
    }
}

/// The colour a TYPE token renders in, or `None` to leave it uncoloured.
///
/// One hue per node type, so the eye can sort a long listing by kind without
/// reading a word of it. Chosen as **truecolor**, not the basic ANSI slots, for
/// two reasons: `violet` and `orange` have no basic slot at all, and the values
/// can then be tuned to stay legible on the theme's dark band (`#451A03`)
/// rather than depending on whatever the terminal maps its sixteen colours to.
///
/// The work types run cool-to-warm down the containment tree (project → arc →
/// slice), and the document types take warm hues that echo the Oxur theme —
/// which also keeps a type distinguishable from the STATUS beside it, since the
/// status palette is basic ANSI.
///
/// `adr` and `note` are deliberately uncoloured: no colour has been chosen for
/// them, and inventing one here would be a decision made by omission.
pub(crate) fn type_color(node_type: NodeType) -> Option<TabledColor> {
    let (r, g, b) = match node_type {
        NodeType::Project => (212, 110, 197), // magenta
        NodeType::Arc => (167, 139, 250),     // violet
        NodeType::Slice => (122, 162, 247),   // blue
        NodeType::Design => (240, 128, 74),   // orange
        // Red at the *same* saturation and luminosity as `design`'s orange
        // (HSL 84.7% / 61.6%, hue rotated 19.5° → 0°), so the two read as one
        // family at one weight rather than either shouting over the other.
        NodeType::Research => (240, 74, 74), // red
        NodeType::Adr | NodeType::Note => return None,
    };
    Some(TabledColor::rgb_fg(r, g, b))
}

/// One rendered node row.
#[derive(Debug, Clone)]
pub(crate) struct NodeRow {
    /// The date column.
    pub(crate) date: NaiveDate,
    /// The node type.
    pub(crate) node_type: NodeType,
    /// The normalized state token — what the STATUS column shows (F-19).
    pub(crate) status: DisplayStatus,
    /// The name cell, already tree-prefixed and de-numbered.
    pub(crate) name: String,
    /// The node id.
    pub(crate) id: Id,
}

/// Builds the rows for a `list` render: the work tree first (depth-ordered by
/// containment), then the document nodes as a flat group.
///
/// `records` is the already-filtered record set; `all_records` is the whole
/// corpus, needed because supersession and containment reference nodes a filter
/// may have excluded. `include_withdrawn` keeps retired/superseded rows (F-15),
/// `status_filter` narrows to one STATUS value, and `group` to one family.
pub(crate) fn build_rows(
    records: &[&IndexRecord],
    all_records: &[IndexRecord],
    gates: &GateSets,
    date: DateColumn,
    include_withdrawn: bool,
    status_filter: Option<&str>,
    group: Option<Group>,
) -> Vec<Row> {
    let superseded = superseded_ids(all_records);

    // Every row filter is applied **before** the tree is derived, never after.
    // The branch glyphs encode "last among siblings", and root-ness is "my
    // parent is not on screen" — both are properties of the *visible* set, so
    // filtering afterwards would leave a `├─` pointing at a row that is not
    // there, and would anchor children to an invisible parent.
    let visible: Vec<&IndexRecord> = records
        .iter()
        .copied()
        .filter(|r| group.is_none_or(|g| g.holds(r.node_type)))
        .filter(|r| {
            let display = display_status_of(r, gates, &superseded);
            // The filter accepts both vocabularies: the normalized state the
            // column now shows (`--status done`) and the raw gate underneath
            // it (`--status tested`). Matching only the derived label would
            // silently break the raw spellings F-15 shipped; matching only the
            // raw one would let `--status` and the STATUS cell disagree.
            let wanted = status_filter.is_none_or(|want| {
                let want = want.trim();
                display.label().eq_ignore_ascii_case(want)
                    || status_of(r, gates, &superseded).label().eq_ignore_ascii_case(want)
            });
            wanted && (include_withdrawn || !display.is_withdrawn())
        })
        .collect();
    let shown: HashSet<Id> = visible.iter().map(|r| r.id).collect();

    // Work nodes: walk the containment tree from its roots so children follow
    // their parent, and the depth is the indent.
    let work: Vec<NodeRow> = work_tree(&visible, all_records, &shown)
        .into_iter()
        .map(|(record, depth, last_at)| {
            row_for(record, gates, &superseded, date, tree_prefix(depth, &last_at))
        })
        .collect();

    // Document nodes: no containment parent, so no tree — a flat group.
    let mut docs: Vec<&&IndexRecord> = visible.iter().filter(|r| !r.node_type.is_work()).collect();
    docs.sort_by_key(|r| (r.created, r.number));
    let docs: Vec<NodeRow> = docs
        .into_iter()
        .map(|record| row_for(record, gates, &superseded, date, String::new()))
        .collect();

    let mut rows: Vec<Row> = Vec::new();
    // The rule earns its line only when there is something on both sides of it.
    let both = !work.is_empty() && !docs.is_empty();
    rows.extend(work.into_iter().map(|r| Row::Node(Box::new(r))));
    if both {
        rows.push(Row::Divider);
    }
    rows.extend(docs.into_iter().map(|r| Row::Node(Box::new(r))));
    rows
}

/// Assembles one row.
fn row_for(
    record: &IndexRecord,
    gates: &GateSets,
    superseded: &HashSet<Id>,
    date: DateColumn,
    prefix: String,
) -> NodeRow {
    NodeRow {
        date: match date {
            DateColumn::Created => record.created,
            DateColumn::Updated => record.updated,
        },
        node_type: record.node_type,
        status: display_status_of(record, gates, superseded),
        name: format!("{prefix}{}", denumber(&record.title)),
        id: record.id,
    }
}

/// The **normalized** state for a node — what the STATUS column shows (F-19).
fn display_status_of(
    record: &IndexRecord,
    gates: &GateSets,
    superseded: &HashSet<Id>,
) -> DisplayStatus {
    let reached: Vec<&str> = record.gates.iter().map(|g| g.gate.as_str()).collect();
    let sequence = gates.for_type(record.node_type).map(GateSet::sequence);
    derive_display_status(&reached, sequence, record.retired, superseded.contains(&record.id))
}

/// The status token for a node, in override order: retired, then superseded,
/// then its furthest-reached gate (F-7).
fn status_of(record: &IndexRecord, gates: &GateSets, superseded: &HashSet<Id>) -> Status {
    if record.retired {
        return Status::Retired;
    }
    if superseded.contains(&record.id) {
        return Status::Superseded;
    }
    match furthest_gate(record, gates) {
        Some(gate) => Status::Gate(gate),
        None => Status::None,
    }
}

/// The last gate of the node's gate-set that it has reached.
///
/// Ordered by the **configured sequence**, not by the record's own gate list:
/// the index stores reached gates gate-name sorted, which is alphabetical, not
/// chronological. A node whose type has no configured gate-set, or which has
/// reached nothing, has no furthest gate.
fn furthest_gate(record: &IndexRecord, gates: &GateSets) -> Option<String> {
    let reached: HashSet<&str> = record.gates.iter().map(|g| g.gate.as_str()).collect();
    let set = gates.for_type(record.node_type)?;
    set.sequence().iter().rfind(|gate| reached.contains(gate.as_str())).cloned()
}

/// The ids of nodes that some other node supersedes.
fn superseded_ids(all_records: &[IndexRecord]) -> HashSet<Id> {
    all_records
        .iter()
        .flat_map(|r| r.edges.iter())
        .filter(|e| e.kind == EdgeKind::Supersedes)
        .map(|e| e.target)
        .collect()
}

/// The work nodes in containment order, each with its depth and the
/// "is-last-child" flag per ancestor level (which decides the branch glyphs).
///
/// Roots are work nodes with no `part_of` parent *among the shown set* — so a
/// filtered view (`--type slice`) still renders, flat, rather than vanishing
/// because its parents were filtered out.
fn work_tree<'a>(
    records: &[&'a IndexRecord],
    all_records: &[IndexRecord],
    shown: &HashSet<Id>,
) -> Vec<(&'a IndexRecord, usize, Vec<bool>)> {
    let work: Vec<&&IndexRecord> = records.iter().filter(|r| r.node_type.is_work()).collect();
    let parent_of: HashMap<Id, Option<Id>> =
        all_records.iter().map(|r| (r.id, parent_id(r))).collect();

    // children[parent] = the shown work children, ordered.
    let mut children: HashMap<Option<Id>, Vec<&IndexRecord>> = HashMap::new();
    for record in &work {
        // A parent that is not itself shown does not anchor the child: the row
        // becomes a root of this view.
        let parent = parent_of.get(&record.id).copied().flatten().filter(|p| shown.contains(p));
        children.entry(parent).or_default().push(record);
    }
    for group in children.values_mut() {
        group.sort_by_key(|r| (r.number, r.created));
    }

    let mut out = Vec::new();
    let roots = children.remove(&None).unwrap_or_default();
    for (i, root) in roots.iter().enumerate() {
        let last = i + 1 == roots.len();
        walk(root, 0, &mut vec![last], &children, &mut out);
    }
    out
}

/// Depth-first walk emitting `(record, depth, last-at-each-level)`.
fn walk<'a>(
    record: &'a IndexRecord,
    depth: usize,
    last_at: &mut Vec<bool>,
    children: &HashMap<Option<Id>, Vec<&'a IndexRecord>>,
    out: &mut Vec<(&'a IndexRecord, usize, Vec<bool>)>,
) {
    out.push((record, depth, last_at.clone()));
    let Some(kids) = children.get(&Some(record.id)) else { return };
    for (i, kid) in kids.iter().enumerate() {
        last_at.push(i + 1 == kids.len());
        walk(kid, depth + 1, last_at, children, out);
        last_at.pop();
    }
}

/// A node's `part_of` target, if any.
fn parent_id(record: &IndexRecord) -> Option<Id> {
    record.edges.iter().find(|e| e.kind == EdgeKind::PartOf).map(|e| e.target)
}

/// The branch glyphs for a row at `depth`, given whether each ancestor level was
/// its parent's last child (F-8).
///
/// Depth 0 has no prefix; deeper rows get `│  ` for an ancestor with siblings
/// still to come, three spaces for one without, and `├─ `/`└─ ` for the row.
fn tree_prefix(depth: usize, last_at: &[bool]) -> String {
    if depth == 0 {
        return String::new();
    }
    let mut prefix = String::new();
    for level in 1..depth {
        prefix.push_str(if last_at.get(level).copied().unwrap_or(true) { "   " } else { "│  " });
    }
    prefix.push_str(if last_at.get(depth).copied().unwrap_or(true) {
        "└─ "
    } else {
        "├─ "
    });
    prefix
}

/// Strips a leading number-reference from a displayed name (F-6).
///
/// Handles the shapes the corpus actually carries — `"Slice 05 (Arc 06): X"`,
/// `"Slice 01 — X"`, `"Arc 03 — X"` — reducing each to `X`. Display-only: the
/// stored `name` is never rewritten by `list`. A name with no number-reference
/// passes through untouched, and a strip that would leave nothing is refused
/// (better a numbered name than an empty cell).
pub(crate) fn denumber(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    let is_prefixed =
        ["slice ", "arc ", "phase ", "step "].iter().any(|kind| lower.starts_with(kind))
            && name.chars().any(|c| c.is_ascii_digit());
    if !is_prefixed {
        return name.to_string();
    }
    // Cut at the first separator that follows the number-reference.
    let cut = name
        .find(": ")
        .map(|i| i + 2)
        .or_else(|| name.find(" — ").map(|i| i + " — ".len()))
        .or_else(|| name.find(" - ").map(|i| i + 3));
    match cut {
        Some(i) if i < name.len() => name[i..].trim().to_string(),
        _ => name.to_string(),
    }
}

/// Truncates `text` to `width` display columns, marking the cut with ` ...`
/// (F-9). A `width` too small to hold the marker leaves the text alone rather
/// than emitting an unreadable stub.
pub(crate) fn elide(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if len <= width || width < 8 {
        return text.to_string();
    }
    let keep: String = text.chars().take(width.saturating_sub(4)).collect();
    format!("{} ...", keep.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denumber_strips_slice_and_arc_references() {
        assert_eq!(denumber("Slice 05 (Arc 06): UAT — CLI feedback"), "UAT — CLI feedback");
        assert_eq!(denumber("Slice 01 — Workspace scaffolding"), "Workspace scaffolding");
        assert_eq!(denumber("Arc 03 — Rollup & orient"), "Rollup & orient");
    }

    #[test]
    fn test_denumber_leaves_unnumbered_names_alone() {
        assert_eq!(denumber("Rollup & orient"), "Rollup & orient");
        assert_eq!(denumber("odm — Architecture & Design"), "odm — Architecture & Design");
        // "Research" is not a numbering prefix.
        assert_eq!(denumber("Research — Forecasting"), "Research — Forecasting");
    }

    #[test]
    fn test_denumber_refuses_to_empty_a_name() {
        assert_eq!(denumber("Slice 05:"), "Slice 05:");
        assert_eq!(denumber("Arc 06"), "Arc 06");
    }

    #[test]
    fn test_elide_cuts_and_marks_only_when_over_width() {
        assert_eq!(elide("short", 20), "short");
        assert_eq!(elide("exactly-ten", 11), "exactly-ten");
        let out = elide("a name far longer than the limit", 16);
        assert_eq!(out, "a name far l ...");
        assert_eq!(out.chars().count(), 16, "the elided cell fits the width");
    }

    #[test]
    fn test_elide_leaves_text_alone_when_width_cannot_hold_the_marker() {
        assert_eq!(elide("abcdefgh", 4), "abcdefgh");
    }

    #[test]
    fn test_tree_prefix_draws_branches_by_depth_and_last_flags() {
        assert_eq!(tree_prefix(0, &[true]), "");
        assert_eq!(tree_prefix(1, &[true, false]), "├─ ");
        assert_eq!(tree_prefix(1, &[true, true]), "└─ ");
        // depth 2 under a parent that still has siblings coming
        assert_eq!(tree_prefix(2, &[true, false, false]), "│  ├─ ");
        // depth 2 under a parent that was its own parent's last child
        assert_eq!(tree_prefix(2, &[true, true, true]), "   └─ ");
    }

    #[test]
    fn test_status_labels_and_withdrawn_predicate() {
        // The raw vocabulary, still reachable through `--status tested`.
        assert_eq!(Status::Retired.label(), "retired");
        assert_eq!(Status::Superseded.label(), "superseded");
        assert_eq!(Status::Gate("tested".into()).label(), "tested");
        assert_eq!(Status::None.label(), "—");
    }

    // ----- F-19: the normalized state -------------------------------------

    /// The real ODD-0013 §5.1 ladders, so the cases below are the ones the
    /// corpus actually produces rather than invented sequences.
    fn ladder(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }
    fn project_arc() -> Vec<String> {
        ladder(&["planned", "in-progress", "complete", "verified"])
    }
    fn slice() -> Vec<String> {
        ladder(&["planned", "built", "tested"])
    }
    fn document() -> Vec<String> {
        ladder(&["draft", "under-review", "revised", "accepted", "active", "final"])
    }

    fn derive(reached: &[&str], seq: &[String]) -> DisplayStatus {
        derive_display_status(reached, Some(seq), false, false)
    }

    #[test]
    fn test_derive_display_status_terminal_gate_is_done() {
        assert_eq!(derive(&["planned", "built", "tested"], &slice()), DisplayStatus::Done);
        assert_eq!(derive(&["planned", "verified"], &project_arc()), DisplayStatus::Done);
        assert_eq!(derive(&["draft", "final"], &document()), DisplayStatus::Done);
    }

    #[test]
    fn test_derive_display_status_a_done_slice_and_a_done_arc_agree() {
        // The whole point of F-19: `tested` and `verified` are different words
        // for the same position, and the column must not make them look
        // different.
        assert_eq!(derive(&["tested"], &slice()), derive(&["verified"], &project_arc()));
    }

    #[test]
    fn test_derive_display_status_mid_ladder_is_active() {
        assert_eq!(derive(&["planned", "built"], &slice()), DisplayStatus::Active);
        assert_eq!(derive(&["planned", "in-progress"], &project_arc()), DisplayStatus::Active);
        assert_eq!(derive(&["draft", "accepted"], &document()), DisplayStatus::Active);
    }

    #[test]
    fn test_derive_display_status_complete_is_not_the_endpoint() {
        // `complete` reads like an endpoint but `verified` is still ahead of
        // it — the specific misreading this chunk exists to fix.
        assert_eq!(derive(&["planned", "complete"], &project_arc()), DisplayStatus::Active);
    }

    #[test]
    fn test_derive_display_status_first_rung_or_nothing_is_planned() {
        assert_eq!(derive(&["planned"], &slice()), DisplayStatus::Planned);
        assert_eq!(derive(&["planned"], &project_arc()), DisplayStatus::Planned);
        assert_eq!(derive(&["draft"], &document()), DisplayStatus::Planned);
        assert_eq!(derive(&[], &slice()), DisplayStatus::Planned, "nothing reached");
    }

    #[test]
    fn test_derive_display_status_position_is_what_counts_not_how_many() {
        // Reaching the terminal gate is `done` even if rungs were skipped, and
        // three reached rungs are still `active` if the last one is not
        // terminal. The state is a position, not a count.
        assert_eq!(derive(&["tested"], &slice()), DisplayStatus::Done);
        assert_eq!(
            derive(&["draft", "under-review", "revised"], &document()),
            DisplayStatus::Active
        );
    }

    #[test]
    fn test_derive_display_status_withdrawal_overlays_win() {
        let seq = slice();
        // Whatever the ladder says — including a fully done node.
        assert_eq!(
            derive_display_status(&["tested"], Some(&seq), true, false),
            DisplayStatus::Retired
        );
        assert_eq!(
            derive_display_status(&["tested"], Some(&seq), false, true),
            DisplayStatus::Superseded
        );
        // Retired wins over superseded when a node is somehow both.
        assert_eq!(
            derive_display_status(&["built"], Some(&seq), true, true),
            DisplayStatus::Retired
        );
    }

    #[test]
    fn test_derive_display_status_no_ladder_reports_nothing() {
        // An `adr` has no gate-set. It is not "planned but not started" — it
        // has no lifecycle to be at the start of, so it must not borrow one.
        assert_eq!(derive_display_status(&[], None, false, false), DisplayStatus::None);
        assert_eq!(derive_display_status(&[], Some(&[]), false, false), DisplayStatus::None);
        assert_ne!(derive_display_status(&[], None, false, false), DisplayStatus::Planned);
        // A withdrawal overlay still applies without a ladder.
        assert_eq!(derive_display_status(&[], None, true, false), DisplayStatus::Retired);
    }

    #[test]
    fn test_derive_display_status_ignores_gates_outside_the_ladder() {
        // A gate a node carries that its type's set does not define says
        // nothing about position in *this* ladder.
        assert_eq!(derive(&["verified"], &slice()), DisplayStatus::Planned);
    }

    #[test]
    fn test_display_status_withdrawn_matches_the_raw_status() {
        assert!(DisplayStatus::Retired.is_withdrawn());
        assert!(DisplayStatus::Superseded.is_withdrawn());
        assert!(!DisplayStatus::Done.is_withdrawn());
        assert!(!DisplayStatus::Active.is_withdrawn());
        assert!(!DisplayStatus::Planned.is_withdrawn());
        assert!(!DisplayStatus::None.is_withdrawn());
    }

    #[test]
    fn test_normalized_labels_have_distinct_colours() {
        // The column relies on colour as much as text; two states rendering
        // identically would defeat it.
        let done = status_color("done");
        let active = status_color("active");
        let planned = status_color("planned");
        assert!(done.is_some() && active.is_some() && planned.is_some());
        assert_ne!(done, active);
        assert_ne!(active, planned);
        assert_ne!(done, planned);
    }
}
