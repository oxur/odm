---
id: 01KYP5GV0DPP3DSP9AKXTF0G5B
number: 570534200
type: artifact
schema: artifact/v1.1
name: 'Slice 02 (Arc 05): `file` probe + probe-runner'
created: 2026-06-30
updated: 2026-06-30
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice02-file-probe-and-runner/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKA8F1BR039MK16BCY
---
# Slice 02 (Arc 05): `file` probe + probe-runner

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Builds on slice01's
> model/trait/shell probe; delivers the second probe + the runner the rest of A5 composes.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| G-1 | The **`file`** probe is a second `ProbeSpec` variant + `Probe` impl: it checks a `path` (relative to repo root) against `expect { exists(default true), sha256?, size? }` → `Holds` (all met) / `Drifted{expected, observed}` (missing-when-exists, wrong hash, wrong size) / `Error` (path unevaluable); content hash reuses the workspace `sha2` (no reimpl) | `cargo test -p odm-reconcile file_probe_{holds,drifts_on_missing,drifts_on_hash_mismatch,errors_on_unreadable}` → ok AND `grep -nE "sha2|Sha256" crates/odm-reconcile/src` | serious | arc-plan slice02 | done | `cargo test -p odm-reconcile file_probe` → 6 passed: `file_probe_holds` (exists+size+sha256), `file_probe_drifts_on_missing` (incl. present-when-`exists:false`), `file_probe_drifts_on_hash_mismatch`, `file_probe_drifts_on_wrong_size`, `file_probe_errors_on_unreadable` (sha256 of a directory → read fails → Error), `file_probe_errors_on_invalid_path` (stat fails → Error). `grep -nE "sha2\|Sha256" crates/odm-reconcile/src` → `file.rs` uses `sha2::{Digest, Sha256}` (no reimpl). **attested** | "File gone" = **Drift** (declared-vs-observed); "can't read the dir" = **Error**. The slice01 `Error ≠ Drifted` split carries. |
| G-2 | A malformed `file` spec (missing `path`, bad `sha256`, unknown sub-field) is a **positioned parse error**, consistent with slice01 F-2 | `cargo test -p odm-core file_probe_spec_malformed_errors_with_position` (or `-p odm-reconcile`, wherever the spec parses) → ok | serious | slice01 F-2 / project error convention | done | `cargo test -p odm-core file_probe_spec_malformed_errors_with_position` → ok (1 passed); missing `path` (serde missing-field), non-hex `sha256` (`deserialize_with` semantic check), and unknown `expect` sub-field (`deny_unknown_fields`) each return `FrontmatterError::Yaml` carrying `line`/`column`; the sha256 error names the field. **attested** | The `file` variant must not regress the positioned-error guarantee. The spec parses in `odm-core` (frontmatter), so the test lives there. |
| G-3 | The **probe-runner** executes one node's `desired_facts` and collects `(fact_id, outcome)`; a node with **no** facts yields an **empty** result (a no-op, never an error) | `cargo test -p odm-reconcile runner_collects_per_node` + `runner_factless_node_is_empty` → ok | serious | arc-plan slice02 | done | `cargo test -p odm-reconcile runner_collects_per_node` → ok (1 passed): a 2-fact node yields a `NodeReport` of 2 `(fact_id, outcome)` in declaration order. `cargo test -p odm-reconcile runner_factless_node_is_empty` → ok (1 passed): a factless node yields an empty report (`is_empty()`), never an error. **attested** | Empty ≠ error: a node simply may declare nothing. |
| G-4 | The runner executes across the **corpus** and collects `(node_id, fact_id, outcome)`, reading **current** frontmatter — a newly written fact is seen with no manual rebuild (read-through freshness) | `cargo test -p odm-reconcile runner_corpus_read_through` → ok (seed nodes, run, add a fact, run again, observe it) | serious | arc-plan slice02 | done | `cargo test -p odm-reconcile runner_corpus_read_through` → ok (1 passed): seed a fact-bearing + a factless node, `run_corpus` → 1 node report; persist a new fact-bearing node, `run_corpus` again → 2 reports, the new node observed — `run_corpus` reloads via `Store::load_all` each call, so no manual rebuild. **attested** | The corpus-level collect slice03 renders. |
| G-5 | The runner reads `desired_facts` **from the store, not the index** — the index is **not** extended with `desired_facts` this slice (the carried A4 adapter-fidelity invariant is honored **by non-triggering**; no `IndexRecord`/adapter/`FORMAT_VERSION` change) | `! grep -nE "index_frontmatters|IndexRecord|FORMAT_VERSION" crates/odm-reconcile/src` (the runner does not route facts through the index) AND `grep -nE "load|Store|frontmatter" crates/odm-reconcile/src` shows the store read AND G-4's read-through test passes | serious | arc-plan v1.3 / A4 invariant (`arc04 closing-report` #2) | done | `grep -rnE "index_frontmatters\|IndexRecord\|FORMAT_VERSION" crates/odm-reconcile/src` → **no matches** (the guarded identifiers are kept out of the crate entirely, prose included, so the grep stays a real tripwire). `grep -rnE "load_all\|Store\|frontmatter" crates/odm-reconcile/src/runner.rs` → the runner uses `odm_store::Store` + `Store::load_all`. G-4 read-through passes. No `IndexRecord`/adapter/`FORMAT_VERSION` change in this slice. **attested** | **The invariant-honoring row.** Reading the field where it lives (the store) means the index equivalence guarantee is untouched. Decision + rationale: `slice-doc.md` "central design decision". Implemented as recommended (store read); not overridden to index-extend. |
| G-6 | The result/report model keeps **drift vs error distinct** per fact and **aggregates** per node / corpus so slice03 can assign severities + exit codes (no flattening to a single count) | `cargo test -p odm-reconcile report_distinguishes_drift_from_error` → ok (a corpus with one holding, one drifted, one error fact → the aggregate exposes all three) | serious | arc-plan slice02 / 0001-C2 | done | `cargo test -p odm-reconcile report_distinguishes_drift_from_error` → ok (1 passed): a corpus with one holding, one drifted, one errored fact → `CorpusReport::counts()` = `{ holds: 1, drifted: 1, errored: 1 }`. `OutcomeCounts` keeps the three kinds separate (no flattening); per-fact `ProbeOutcome` is preserved and `CorpusReport::iter()` exposes every `(node_id, fact_result)`. **attested** | slice03 maps drift → one severity, error → another; the model must not lose the distinction. |
| G-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for the slice's `odm-reconcile` additions | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-reconcile/src` AND `cargo llvm-cov --summary-only -p odm-reconcile` → **line** ≥ 90% | serious | CLAUDE.md | done | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0; `grep -RnE '\bunsafe\b' crates/odm-reconcile/src` → none (also denied workspace-wide); `cargo llvm-cov --summary-only -p odm-reconcile` → line: `file.rs` **97.01%**, `runner.rs` **96.92%**, `shell.rs` **100%** (all ≥ 90). Full workspace `cargo test` green (40 suites, no regression). **attested** | |

## What Worked

- **The not-`#[non_exhaustive]` safety net fired exactly as designed.** Adding the
  `File` variant produced one compile error (the odm-core round-trip test's match)
  *plus* the new runner dispatch — each forced to handle `File` explicitly. No
  silent wildcard swallowed the new kind.
- **Reliable, root-insensitive Error tests.** "Unreadable" is awkward to test (CI
  often runs as root, so `chmod 000` doesn't deny). Two deterministic triggers
  cover both Error branches on any platform/user: sha256 of a **directory**
  (read fails) and a path with an embedded **NUL** (stat fails). Both are genuine
  "couldn't check" cases, keeping Error ≠ Drift honest without flaky permissions.
- **Store-not-index kept the slice tiny.** Honoring the carried invariant by
  *non-triggering* meant zero index/adapter/fidelity-test work — the runner is
  `Store::load_all` + a dispatch match. The G-5 grep is the cheap guard that keeps
  it that way.
- **`deny_unknown_fields` on `FileExpect` (a plain struct, not the tagged enum)**
  gave the "unknown sub-field → positioned error" half of G-2 for free, without
  fighting serde's tagged-enum limitation.

## Closure

Closed at commit `e4ca702` on 2026-06-30. Verified by: CC (self-attested); CDC to
reproduce. Rows: 7. Done: 7. Deferred: 0. No-op: 0.

> Per LEDGER-DISCIPLINE v2.0 §A, all evidence is **attested** (CC built and ran
> every cargo/grep row on a local 1.95 toolchain). Independent **reproduction**
> (CI / local 1.85+) elevates to `reproduced` in `cdc-verification.md`.
> **Branching note:** slice01 had not merged to `main`, so this slice branched off
> `arc05-slice01-desired-facts-probe` (per the prompt's fallback) — rebase onto
> `main` once slice01 merges.
