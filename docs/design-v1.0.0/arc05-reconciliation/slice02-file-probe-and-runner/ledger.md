# Slice 02 (Arc 05): `file` probe + probe-runner

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Builds on slice01's
> model/trait/shell probe; delivers the second probe + the runner the rest of A5 composes.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| G-1 | The **`file`** probe is a second `ProbeSpec` variant + `Probe` impl: it checks a `path` (relative to repo root) against `expect { exists(default true), sha256?, size? }` → `Holds` (all met) / `Drifted{expected, observed}` (missing-when-exists, wrong hash, wrong size) / `Error` (path unevaluable); content hash reuses the workspace `sha2` (no reimpl) | `cargo test -p odm-reconcile file_probe_{holds,drifts_on_missing,drifts_on_hash_mismatch,errors_on_unreadable}` → ok AND `grep -nE "sha2|Sha256" crates/odm-reconcile/src` | serious | arc-plan slice02 | open | | "File gone" = **Drift** (declared-vs-observed); "can't read the dir" = **Error**. The slice01 `Error ≠ Drifted` split carries. |
| G-2 | A malformed `file` spec (missing `path`, bad `sha256`, unknown sub-field) is a **positioned parse error**, consistent with slice01 F-2 | `cargo test -p odm-core file_probe_spec_malformed_errors_with_position` (or `-p odm-reconcile`, wherever the spec parses) → ok | serious | slice01 F-2 / project error convention | open | | The `file` variant must not regress the positioned-error guarantee. |
| G-3 | The **probe-runner** executes one node's `desired_facts` and collects `(fact_id, outcome)`; a node with **no** facts yields an **empty** result (a no-op, never an error) | `cargo test -p odm-reconcile runner_collects_per_node` + `runner_factless_node_is_empty` → ok | serious | arc-plan slice02 | open | | Empty ≠ error: a node simply may declare nothing. |
| G-4 | The runner executes across the **corpus** and collects `(node_id, fact_id, outcome)`, reading **current** frontmatter — a newly written fact is seen with no manual rebuild (read-through freshness) | `cargo test -p odm-reconcile runner_corpus_read_through` → ok (seed nodes, run, add a fact, run again, observe it) | serious | arc-plan slice02 | open | | The corpus-level collect slice03 renders. |
| G-5 | The runner reads `desired_facts` **from the store, not the index** — the index is **not** extended with `desired_facts` this slice (the carried A4 adapter-fidelity invariant is honored **by non-triggering**; no `IndexRecord`/adapter/`FORMAT_VERSION` change) | `! grep -nE "index_frontmatters|IndexRecord|FORMAT_VERSION" crates/odm-reconcile/src` (the runner does not route facts through the index) AND `grep -nE "load|Store|frontmatter" crates/odm-reconcile/src` shows the store read AND G-4's read-through test passes | serious | arc-plan v1.3 / A4 invariant (`arc04 closing-report` #2) | open | | **The invariant-honoring row.** Reading the field where it lives (the store) means the index equivalence guarantee is untouched. Catches a future accidental switch to `index_frontmatters` (which would silently return facts-less frontmatters). Decision + rationale: `slice-doc.md` "central design decision" (ratification point — Duncan/CC may override to index-extend). |
| G-6 | The result/report model keeps **drift vs error distinct** per fact and **aggregates** per node / corpus so slice03 can assign severities + exit codes (no flattening to a single count) | `cargo test -p odm-reconcile report_distinguishes_drift_from_error` → ok (a corpus with one holding, one drifted, one error fact → the aggregate exposes all three) | serious | arc-plan slice02 / 0001-C2 | open | | slice03 maps drift → one severity, error → another; the model must not lose the distinction. |
| G-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the slice's `odm-reconcile` additions | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-reconcile/src` AND `cargo llvm-cov --summary-only -p odm-reconcile` → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(At slice close. Patterns that made the slice close cleanly.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 7. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
