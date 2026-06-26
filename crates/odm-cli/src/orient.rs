//! `odm orient` (alias `brief`) — the cheap, actionable read a fresh session
//! leads with (ODD-0013 §4.1/§7; arc03 slice03).
//!
//! Where `odm rollup` writes the full structural view, `orient` composes a
//! *terse* situational-awareness view over the **same** slice02 `Rollup` model
//! (D-3 — it does not re-derive the graph) plus the existing `check` aggregation
//! for integrity (slice02 ruling 3 — it does not re-walk integrity). The order
//! is: **vision → current focus → ready/blocked → integrity → drift** (ODD-0015
//! §2: full situational awareness from one call).
//!
//! Bare `odm` runs this, and every no-current-project branch is an
//! affordance, not an error — `orient` never bare-errors (it returns `Ok` and
//! the binary exits `0`).

use std::fmt::Write as _;
use std::path::Path;

use anyhow::Context as _;
use odm_core::frontmatter::{Document, Frontmatter};
use odm_core::rollup::{NodeRef, Rollup, TreeNode};
use odm_core::{Id, NodeType};
use odm_store::Store;

use crate::commands::{self, IntegrityFinding};
use crate::context::Context;
use crate::json::{FocusJson, OrientJson};
use crate::rollup::{dep_label, label, status_inline};

/// The fix affordance shown when the corpus has no project.
const NO_PROJECT_HINT: &str = "no project yet — create one with `odm new project \"<name>\"`";

/// The fix affordance shown when several projects exist but none is selected.
fn pick_project_hint(n: usize) -> String {
    format!("{n} projects, none selected — choose one with `odm use project <ref>`")
}

/// The vision excerpt budget: at most this many non-empty body lines before the
/// excerpt is cut with a `odm show` continuation marker (D-1a).
const VISION_LINE_BUDGET: usize = 15;

/// `orient` / `brief` — compose the situational-awareness view. Always returns
/// `Ok` for the normal and no-project paths (the binary then exits `0`); only a
/// genuine I/O / config failure propagates as an error.
///
/// # Errors
///
/// Returns an error if the corpus or gate config cannot be loaded.
pub fn orient(
    store: &Store,
    root: &Path,
    json: bool,
    out: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    let docs = store.load_all().context("loading the corpus to orient")?;

    let ctx = Context::load(root)?;
    let projects: Vec<&Document> =
        docs.iter().filter(|d| d.frontmatter().node_type() == NodeType::Project).collect();

    // Resolve the current project, or emit a never-bare-error fallback (valid
    // JSON or the human affordance, per `json`).
    let project: &Document = if let Some(p) =
        ctx.project.and_then(|id| docs.iter().find(|d| d.frontmatter().id() == id))
    {
        p
    } else if projects.is_empty() {
        return emit_fallback(json, out, NO_PROJECT_HINT.to_string(), &|o| render_no_project(o));
    } else if projects.len() == 1 {
        projects[0]
    } else {
        let hint = pick_project_hint(projects.len());
        return emit_fallback(json, out, hint, &|o| render_pick_project(o, &projects));
    };

    // Reuse the slice02 model (single full scan) + the check aggregation.
    let frontmatters: Vec<Frontmatter> = docs.iter().map(|d| d.frontmatter().clone()).collect();
    let (gates, threshold) = commands::load_gate_config(root)?;
    let model = Rollup::assemble(&frontmatters, &gates, threshold);
    let findings = commands::integrity_findings(store, root, &docs)?;

    if json {
        let view = orient_json(project, &ctx, &docs, &model, &findings);
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        return Ok(());
    }

    let view = render_orient(project, &ctx, &docs, &model, &findings);
    write!(out, "{view}")?;
    Ok(())
}

/// Emits a no-project fallback: a valid JSON envelope (with `hint`) when `json`,
/// else the human affordance via `render`. Both exit 0 — never bare-errors.
fn emit_fallback(
    json: bool,
    out: &mut dyn std::io::Write,
    hint: String,
    render: &dyn Fn(&mut dyn std::io::Write) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    if json {
        let view = OrientJson::fallback(hint);
        writeln!(out, "{}", serde_json::to_string_pretty(&view)?)?;
        Ok(())
    } else {
        render(out)
    }
}

/// Builds the resolved-project `orient --json` envelope over the same model the
/// human view renders (D-3).
fn orient_json(
    project: &Document,
    ctx: &Context,
    docs: &[Document],
    model: &Rollup,
    findings: &[IntegrityFinding],
) -> OrientJson {
    let fm = project.frontmatter();
    let project_ref = NodeRef {
        id: fm.id(),
        number: fm.number(),
        name: fm.name().to_string(),
        node_type: fm.node_type(),
    };

    let vision_text = extract_vision(project.body(), fm.number());
    let vision = (!vision_text.is_empty()).then_some(vision_text);

    let focus =
        ctx.arc.and_then(|id| docs.iter().find(|d| d.frontmatter().id() == id)).map(|arc| {
            let afm = arc.frontmatter();
            let arc_ref = NodeRef {
                id: afm.id(),
                number: afm.number(),
                name: afm.name().to_string(),
                node_type: afm.node_type(),
            };
            let status = find_in_tree(&model.tree, afm.id())
                .map(|n| n.status.as_slice())
                .unwrap_or(&[])
                .iter()
                .map(Into::into)
                .collect();
            FocusJson { arc: (&arc_ref).into(), status }
        });

    OrientJson::resolved((&project_ref).into(), vision, focus, model, findings)
}

/// Fallback when the corpus has no project: an affordance to create one.
fn render_no_project(out: &mut dyn std::io::Write) -> anyhow::Result<()> {
    writeln!(out, "odm — orient\n")?;
    writeln!(out, "No project yet.")?;
    writeln!(out, "  → create one: `odm new project \"<name>\"`")?;
    Ok(())
}

/// Fallback when several projects exist but none is the current context: list
/// them and prompt a selection.
fn render_pick_project(out: &mut dyn std::io::Write, projects: &[&Document]) -> anyhow::Result<()> {
    writeln!(out, "odm — orient\n")?;
    writeln!(out, "{} projects, none selected:", projects.len())?;
    for p in projects {
        let fm = p.frontmatter();
        writeln!(out, "  - #{} {}", fm.number(), fm.name())?;
    }
    writeln!(out, "  → choose one: `odm use project <ref>`")?;
    Ok(())
}

/// Renders the full orient view for a resolved current `project`.
fn render_orient(
    project: &Document,
    ctx: &Context,
    docs: &[Document],
    model: &Rollup,
    findings: &[IntegrityFinding],
) -> String {
    let mut s = String::new();
    let fm = project.frontmatter();

    let _ = writeln!(s, "odm — orient\n");

    // 1. Vision: the project name + the extracted vision excerpt.
    let _ = writeln!(s, "VISION  #{} {}", fm.number(), fm.name());
    let vision = extract_vision(project.body(), fm.number());
    if vision.is_empty() {
        let _ =
            writeln!(s, "  _(no vision text yet — add a `# Vision` section to the project body)_");
    } else {
        for line in vision.lines() {
            let _ = writeln!(s, "  {line}");
        }
    }

    // 2. Current focus: the current arc + its status vector.
    let _ = writeln!(s, "\nCURRENT FOCUS");
    match ctx.arc.and_then(|id| docs.iter().find(|d| d.frontmatter().id() == id)) {
        Some(arc) => {
            let arc_fm = arc.frontmatter();
            let status = find_in_tree(&model.tree, arc_fm.id())
                .map(|n| status_inline(&n.status))
                .unwrap_or_default();
            if status.is_empty() {
                let _ = writeln!(s, "  arc #{} {}", arc_fm.number(), arc_fm.name());
            } else {
                let _ = writeln!(s, "  arc #{} {} — {status}", arc_fm.number(), arc_fm.name());
            }
        }
        None => {
            let _ = writeln!(s, "  (no current arc — `odm use arc <ref>`)");
        }
    }

    // 3. Ready / blocked (with the soft-sat ⚠ on the ready frontier).
    let _ = writeln!(s, "\nREADY");
    if model.ready.is_empty() {
        let _ = writeln!(s, "  (nothing ready)");
    } else {
        for r in &model.ready {
            let _ = writeln!(s, "  - {}", label(&r.node));
            for soft in &r.soft {
                let _ = writeln!(
                    s,
                    "    ⚠ soft dep {} at evidence={}",
                    dep_label(&soft.dep),
                    soft.evidence.as_str()
                );
            }
        }
    }

    let _ = writeln!(s, "\nBLOCKED");
    if model.blocked.is_empty() {
        let _ = writeln!(s, "  (nothing blocked)");
    } else {
        for b in &model.blocked {
            let _ = writeln!(s, "  - {}", label(&b.node));
            for reason in &b.reasons {
                let _ = writeln!(s, "    - {}", block_reason_line(reason));
            }
        }
    }

    // 4. Integrity: every hard error, inline (slice02 ruling 3).
    let _ = writeln!(s, "\nINTEGRITY");
    let errors: Vec<&IntegrityFinding> = findings.iter().filter(|f| f.is_error).collect();
    if errors.is_empty() {
        let _ = writeln!(s, "  ok — no structural errors");
    } else {
        for f in errors {
            let _ = writeln!(s, "  ✗ [{}] {}: {}", f.code, f.who, f.detail);
        }
    }

    // 5. Drift: the A5 placeholder (Q-A3-2).
    let _ = writeln!(s, "\nDRIFT");
    let _ = writeln!(s, "  not yet tracked (A5)");

    s
}

/// Formats one block reason for the terse orient view.
fn block_reason_line(reason: &odm_core::rollup::BlockReason) -> String {
    use odm_core::rollup::BlockReason::{ExternallyBlocked, SoftSatisfied, Unsatisfied};
    match reason {
        Unsatisfied { dep } => format!("unsatisfied: {}", dep_label(dep)),
        SoftSatisfied { dep, evidence, threshold } => format!(
            "low-evidence: {} at evidence={} (needs {})",
            dep_label(dep),
            evidence.as_str(),
            threshold.as_str()
        ),
        ExternallyBlocked { by } => format!("blocked-by: {}", dep_label(by)),
    }
}

/// Finds a node by id anywhere in the way-finding forest.
fn find_in_tree(nodes: &[TreeNode], id: Id) -> Option<&TreeNode> {
    for n in nodes {
        if n.node.id == id {
            return Some(n);
        }
        if let Some(found) = find_in_tree(&n.children, id) {
            return Some(found);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Vision extraction (D-1a) — a pure helper over the project body.
// ---------------------------------------------------------------------------

/// Extracts the vision excerpt from a project `body` (D-1a).
///
/// If the body has an ATX heading whose text is `Vision` (case-insensitive), the
/// section under it (up to the next same-or-higher-level heading) is taken;
/// otherwise the **lead section** (body text before the first ATX heading) is
/// taken. The excerpt is trimmed of blank edges and truncated to
/// [`VISION_LINE_BUDGET`] non-empty lines, appending a
/// `… (full vision: odm show <number>)` marker when cut. Returns an empty string
/// when there is no vision text (e.g. a body that is only a title heading).
fn extract_vision(body: &str, show_number: u32) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let section = vision_section(&lines).unwrap_or_else(|| lead_section(&lines));
    truncate_excerpt(&section, VISION_LINE_BUDGET, show_number)
}

/// The ATX heading level (1–6) and trimmed text of `line`, or `None` if it is
/// not an ATX heading. A heading is `#`×(1–6) followed by a space or EOL.
fn atx_heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&hashes) {
        let rest = &line[hashes..];
        if rest.is_empty() || rest.starts_with(' ') {
            return Some((hashes, rest.trim()));
        }
    }
    None
}

/// The lines of the `# Vision` section (excluding its heading), up to the next
/// heading of the same or higher level, or `None` if there is no vision heading.
fn vision_section<'a>(lines: &[&'a str]) -> Option<Vec<&'a str>> {
    let (start, level) = lines.iter().enumerate().find_map(|(i, line)| {
        atx_heading(line)
            .and_then(|(lvl, text)| text.eq_ignore_ascii_case("vision").then_some((i, lvl)))
    })?;
    let mut out = Vec::new();
    for line in &lines[start + 1..] {
        if let Some((lvl, _)) = atx_heading(line) {
            if lvl <= level {
                break;
            }
        }
        out.push(*line);
    }
    Some(out)
}

/// The lead section: body lines before the first ATX heading.
fn lead_section<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    lines.iter().take_while(|line| atx_heading(line).is_none()).copied().collect()
}

/// Trims blank edges, then keeps at most `budget` non-empty lines, appending a
/// continuation marker when content is cut.
fn truncate_excerpt(section: &[&str], budget: usize, show_number: u32) -> String {
    let start = section.iter().position(|l| !l.trim().is_empty());
    let Some(start) = start else {
        return String::new();
    };
    let end = section.iter().rposition(|l| !l.trim().is_empty()).unwrap_or(start);
    let trimmed = &section[start..=end];

    let mut kept: Vec<&str> = Vec::new();
    let mut nonempty = 0usize;
    let mut cut = false;
    for line in trimmed {
        if line.trim().is_empty() {
            kept.push(line);
            continue;
        }
        if nonempty == budget {
            cut = true;
            break;
        }
        nonempty += 1;
        kept.push(line);
    }

    let mut text = kept.join("\n").trim_end().to_string();
    if cut {
        text.push('\n');
        let _ = write!(text, "… (full vision: odm show {show_number})");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- O-2: vision extraction rule --------------------------------------

    #[test]
    fn vision_extraction_rule_prefers_vision_heading() {
        let body = "\
# Some Project

Intro paragraph that is NOT the vision.

# Vision

The north star: make the plan legible from one call.
It spans two lines.

# Details

Not part of the vision.
";
        let v = extract_vision(body, 1);
        assert!(v.contains("north star"), "took the Vision section: {v:?}");
        assert!(v.contains("two lines"));
        assert!(!v.contains("Intro paragraph"), "did not take the intro");
        assert!(!v.contains("Not part of the vision"), "stopped at the next same/higher heading");
    }

    #[test]
    fn vision_extraction_rule_case_insensitive_and_nested_subheadings() {
        // A lower-level heading inside the vision section is kept; the section
        // ends only at a same-or-higher-level heading.
        let body = "\
## vISIoN

Lead vision text.

### Sub-point

Still vision.

## Other

Out.
";
        let v = extract_vision(body, 7);
        assert!(v.contains("Lead vision text"));
        assert!(
            v.contains("Sub-point") && v.contains("Still vision"),
            "nested subheading kept: {v:?}"
        );
        assert!(!v.contains("Out"));
    }

    #[test]
    fn vision_extraction_rule_falls_back_to_lead_section() {
        let body = "\
Lead paragraph before any heading.

# Heading

After.
";
        let v = extract_vision(body, 2);
        assert_eq!(v, "Lead paragraph before any heading.");
    }

    #[test]
    fn vision_extraction_rule_truncates_with_marker() {
        // 20 non-empty lines under # Vision; budget is 15 ⇒ cut with a marker.
        let mut body = String::from("# Vision\n\n");
        for i in 1..=20 {
            body.push_str(&format!("line {i}\n"));
        }
        let v = extract_vision(&body, 3);
        assert!(v.contains("line 15"), "keeps up to the budget");
        assert!(!v.contains("line 16"), "drops past the budget");
        assert!(
            v.contains("… (full vision: odm show 3)"),
            "appends the continuation marker: {v:?}"
        );
    }

    #[test]
    fn vision_extraction_rule_empty_for_title_only_body() {
        // The default `odm new` body is just a title heading: no vision text.
        assert_eq!(extract_vision("# Proj\n", 1), "");
    }

    #[test]
    fn vision_extraction_rule_no_marker_when_under_budget() {
        let body = "# Vision\n\nShort and sweet.\n";
        let v = extract_vision(body, 1);
        assert_eq!(v, "Short and sweet.");
    }
}
