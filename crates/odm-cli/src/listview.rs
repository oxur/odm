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
use odm_core::gates::GateSets;
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

/// The `STATUS` value shown for a node (F-7), in override order.
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

    /// Whether this status means the node is not live work — the rows `list`
    /// hides unless asked for them (F-15).
    pub(crate) fn is_withdrawn(&self) -> bool {
        matches!(self, Status::Retired | Status::Superseded)
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
/// The **document** lifecycle keeps `oxur-odm` 0.3.5's palette verbatim — this
/// calls the very helper the original called, so the colours cannot drift by
/// being retyped.
///
/// The **work** sequences postdate that palette, so they are mapped onto the
/// same *slots* rather than given new colours: early is yellow, under way is
/// cyan, done is green, and the strongest evidence is bright green. Keeping
/// `complete` green and `verified` bright green is what preserves ODD-0013
/// §5.1's distinction — "done at its layer" is not "verified live", and
/// collapsing them is the confusion that gate model exists to prevent.
///
/// The two palettes are disjoint (no gate name appears in both), so the
/// document one always wins where it applies.
pub(crate) fn status_color(label: &str) -> Option<TabledColor> {
    helpers::state_to_fg_color(label).or_else(|| work_gate_color(label))
}

/// The work-node half of [`status_color`] — `project`/`arc`/`slice` gates.
fn work_gate_color(label: &str) -> Option<TabledColor> {
    match label.trim().to_ascii_lowercase().as_str() {
        // Recorded, not started — the slot `draft` holds.
        "planned" => Some(TabledColor::FG_YELLOW),
        // Under way — the slot `under-review` holds.
        "in-progress" | "in progress" | "built" => Some(TabledColor::FG_CYAN),
        // Done at its layer, and a slice's terminal gate.
        "complete" | "tested" => Some(TabledColor::FG_GREEN),
        // Verified live — the strongest, as `active` is for a document.
        "verified" => Some(TabledColor::FG_BRIGHT_GREEN),
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
        NodeType::Design => (229, 192, 123),  // yellow
        NodeType::Research => (240, 128, 74), // red-orange
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
    /// The status token.
    pub(crate) status: Status,
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
            let status = status_of(r, gates, &superseded);
            let wanted =
                status_filter.is_none_or(|want| status.label().eq_ignore_ascii_case(want.trim()));
            wanted && (include_withdrawn || !status.is_withdrawn())
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
        status: status_of(record, gates, superseded),
        name: format!("{prefix}{}", denumber(&record.title)),
        id: record.id,
    }
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
        assert_eq!(Status::Retired.label(), "retired");
        assert_eq!(Status::Superseded.label(), "superseded");
        assert_eq!(Status::Gate("tested".into()).label(), "tested");
        assert_eq!(Status::None.label(), "—");
        assert!(Status::Retired.is_withdrawn());
        assert!(Status::Superseded.is_withdrawn());
        assert!(!Status::Gate("tested".into()).is_withdrawn());
        assert!(!Status::None.is_withdrawn());
    }
}
