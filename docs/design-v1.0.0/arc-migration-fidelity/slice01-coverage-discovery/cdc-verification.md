# Slice 01 CDC verification — Coverage discovery

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 01 · **Feeds:** MF-1, MF-5
> **Ledger:** `ledger.md` (F-1…F-9) · **Implemented by:** CC · **Closing report:** `closing-report.md`
> **Verified by:** CDC (Opus session) · **Date:** 2026-07-27 · **Branch:** `arc-migfidelity-slice01-coverage`
> @ `50b91c2` (off `release/1.0.x` @ `14f098b`)
> **Verdict: PASS — CDC-verified. All 9 rows carried to `reproduced` or a justified `attested`→CI.
> Slice may close. No iterations required. Two model-questions routed to s02 (one is CC's, one is added here).**

## Method & evidence access

Per LEDGER-DISCIPLINE v2.0, the closer (CC) is not the verifier. This CDC pass read the actual
artifacts — `coverage.rs`, `tests/coverage.rs`, the CLI wiring in `odm-cli/src/migrate.rs`, the
committed `coverage-report.md`, the filled `ledger.md`, and the `closing-report.md` — and
**independently re-derived the count-based rows** against the live corpus, rather than reading
CC's summary. The CDC sandbox has no 1.85+ cargo (edition 2024), so cargo/executable rows are
**attested-by-CC → reproduced-on-CI**; structural and count rows are **reproduced here** via
`device_bash` over the real `1.0.x/docs` corpus and the `.worktrees/odm` store (60 nodes).

**Independent reproductions (CDC, on the live corpus — not CC's tool):**

| Quantity | Row | Expected | CDC re-derived | Basis |
|----------|-----|----------|----------------|-------|
| Store composition | — | 60 (6 arc · 39 slice · 9 design · 5 research · 1 project) | **exact match** | `find`/parse over `.worktrees/odm/nodes` |
| Stub bodies | F-5 | 44 (6 arc + 38 slice), tombstone excluded | **44 = 6 arc + 38 slice** | independent Python re-implementation of the ≤1-non-blank-line rule |
| Provenance-absent | F-6 | 60 | **60** | grep `^provenance:` over emitted-equivalent frontmatter |
| `unsafe` in module | F-9 | 0 | **0** | grep `\bunsafe\b` over `coverage.rs` |
| Store-write callsites in module | F-7 | none | **none** (matches are `///` doc-comment prose only) | grep `persist\|fs::write\|File::create\|\.write(\|save\|mint` |

## Per-row disposition

| ID | Verdict | Strength | How CDC verified it |
|----|---------|----------|---------------------|
| **F-1** | ✅ done | reproduced (structural) | `--coverage` bool flag confirmed in `odm-cli/src/lib.rs:614`, `conflicts_with_all` the sibling derivation flags, dispatched at `migrate.rs:54` to a `coverage()` arm calling `odm_migrate::coverage::run` and rendering to **stdout**. Reachable and read-only-by-construction (writes no file; the human redirects `> coverage-report.md`). `--help` visibility is clap-derive-guaranteed; build/run attested→CI. |
| **F-2** | ✅ done | attested→CI (test) + invariant confirmed | `enumerate_docs` walks unfiltered (`coverage.rs:301`), so `len() == find … -name '*.md' \| wc -l` **by construction**; the `coverage_classify` test asserts exactly this against an unfiltered walk. The report's run-time 326 is now 328 on disk — **explained, not drift**: the run wrote `coverage-report.md` + `closing-report.md` into `design-v1.0.0/`, +2. Exact-count re-derivation at the original instant is not reproducible post-hoc for this reason; the invariant is. |
| **F-3** | ✅ done | attested→CI (test) + read | `coverage_doc_coverage` asserts covered/uncovered across project, in-scope arc, **out-of-scope arc07**, slice, all four supporting-doc classes, matched ODD #1, unmatched ODD #99, and an unparsable index — **and that every entry carries a non-empty `basis`**. The heuristic caveat is carried per-entry in code (`CoverageEntry.basis`), exactly as the slice-doc required. 266 uncovered reconciled (see F-8). |
| **F-4** | ✅ done | attested→CI (test) + read | The row's phrasing centred on arcs; I checked the slice gap specifically. `coverage_representation` constructs a **represented arc with an unmatched slice** (`arc01-alpha/slice02-bb`) and a named arc, and asserts both `missing_slices` entries — my pre-flagged concern is tested. Live: 6/12 arc dirs, 39/44 slice dirs; the 6 missing arcs = audit's 5 + `arc-migration-fidelity`; the 5 missing slices = 4 `arc-store-home` + this slice. Arithmetic closes (39 represented dirs = 39 slice nodes; no dir-less node). |
| **F-5** | ✅ done | **reproduced** | CDC re-derived 44 (6 arc + 38 slice) independently; `coverage_stubs` additionally proves the tombstone- and non-work-node exclusions. |
| **F-6** | ✅ done | **reproduced** | CDC re-derived 60 independently; `coverage_provenance_absence` proves both the absent and present (via untyped `extra`) cases, validating the durable-across-s02-typing claim. |
| **F-7** | ✅ done | reproduced (structural) + attested (live git-check) | No write callsites in the module (CDC grep); the tool renders to stdout, never to the store; `coverage_is_read_only` byte-snapshots the store before/after and asserts equality. CC's live `git status --porcelain` empty + identical node md5-of-md5 (`40aecffaa88be3aa8764d9423594c8b0`) attested. CDC could **not** re-run the git check — the store worktree's `.git` gitdir points at the macOS absolute path, absent in the CDC VM — but the structural guarantee + snapshot test cover the criterion. |
| **F-8** | ✅ done | reproduced (read) | `coverage-report.md` committed; §1 lists 266 uncovered by class, each with a basis; the summary and per-class lists are internally consistent (per-class sums = grand total). Divergence from the audit ballpark (266 vs ≈211; 6 vs 5 arcs) is explained by name: audit-excluded scope + chunk-variant folding + corpus growth since the audit. No unexplained residual. |
| **F-9** | ✅ done | reproduced (no-unsafe) + attested→CI (clippy/cov) | 0 `unsafe` re-derived by CDC; clippy `-D warnings` clean and `coverage.rs` at **93.92%** line (296/18) attested — above the 90% floor, short of the 95% stretch (acceptable; 95% was target not gate). |

**Rows: 9. Done: 9. Deferred: 0. No-op: 0. Silent drops: none** (scope-as-specified vs
scope-as-delivered diffed against the slice-doc "In"/"Out" — the four "Out" items are confirmed
absent: no minting, no schema change, no `check`/`validate` wiring, no body-hash/provenance work).

## Code-quality note (non-blocking)

The module is clean: typed `thiserror` error with a per-doc-vs-fatal split mirroring
`MigrateError`; `#[must_use]` on the count accessors; representation derived from the classified
doc list rather than a second blind filesystem walk (a genuinely better design — it makes no
assumption about where `design-v1.0.0/` sits under `docs/`, and the "What Worked" note records
that this caught a real 0-arc bug on first run). One observation, not a defect: the spec named
`mapping.rs` as the ODD-matcher reuse target; CC reused `legacy::parse_file` (the shared
frontmatter reader `mapping` itself calls) plus a fresh `DocClass` classifier. That is consistent
with "don't re-derive coordinate/number rules" (those come from `selfhost`) and is arguably more
direct; noted only so the reuse lineage is on the record.

## Flags — ruling

1. **Chunk-scale artifact classification (CC's flag) — CONCUR, routed to s02.** `cN-closing-report.md`
   / `cN-cdc-verification.md` / `cc-prompt-cN-*.md` currently fold into their slice-scale sibling
   class. Correct for a read-only *measure* (the grand total is unaffected, and the tests pin the
   folding behaviour explicitly), but s02's supporting-doc node-class model must decide for real
   whether a chunk artifact is `part_of` its slice or needs its own granularity between slice and arc.
2. **Three meanings of "research" (CC's flag) — CONCUR, s02 doc-note.** `docs/dev/research/` (5),
   `docs/dev/` (26), and the tag-based `research` node type (5, inside the `odd` class) are three
   distinct things sharing the word. Scope already covers all three; the name overlap wants one line
   in s02's model doc so a future reader doesn't conflate them.
3. **⭐ Report-artifact self-coverage (CDC-added) — NEW, must be resolved by s02/s05.** Because the
   detector renders to stdout and the operator redirects into `coverage-report.md` under
   `design-v1.0.0/`, **every regeneration adds ≥1 permanently-uncovered `.md`** — the report itself,
   plus this arc's own `closing-report.md` / `cdc-verification.md` / future slice artifacts (the
   326→328 drift is the first instance). CC named the self-reference as "correct, not a bug," which is
   right for s01. The consequence for the arc: when s05 wires doc-coverage into `odm check` as an
   enforced gate (MF-6), the arc's own coverage/report/verification artifacts will make `check`
   **red forever** unless s02's model decides their disposition — a report/verification node class, an
   explicit coverage exemption, or an ignore rule. This is a design fork not currently in the s02
   scope bullet; it should be added there.

## Arc bubble-up ratification (MF-1, MF-5)

**CONCUR** with CC's recommendation: MF-1 and MF-5 **remain `planned`**, not `done`. s01 built the
read-only *detector* and produced the exact inventory; it did not wire the enforced doc-coverage
*check* (MF-1 → s05) nor perform the representation *fix* (MF-5 → s04/s05). Recording MF-1/MF-5 as a
pointer to this closing report as baseline evidence is the correct LEDGER-DISCIPLINE v2.0 §B
disposition (composition rows reproduce at arc scale, at arc close).

**Recommended arc-plan touch (for operator confirmation, not applied here):** a dated version-history
line on `arc-plan.md` — *"v1.1 — 2026-07-27 — s01 (coverage-discovery) closed & CDC-verified; exact
inventory in slice01 `coverage-report.md` supersedes the audit estimates as the work-list; s02 scope
gains the report-artifact-coverage fork (flag 3) alongside the chunk-class and research-name notes."*
Flagged rather than silently edited, per the plan-change discipline.

## What s02 inherits

The authoritative, re-runnable numbers to plan against (not the audit's estimates):
**326 source docs · 266 uncovered across 10 classes · 6/12 arc dirs + 39/44 slice dirs represented ·
44 stub bodies (6 arc + 38 slice) · 60/60 nodes provenance-absent** — plus the three model-forks above.
