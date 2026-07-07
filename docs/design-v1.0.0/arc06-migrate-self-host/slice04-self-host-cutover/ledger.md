# Slice 04 (Arc 06): self-host cutover

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). Five-iteration cap. **The loop-closer** — brings the `design-v1.0.0` plan set
> into `nodes/` as work nodes; delivers arc-plan **A-4**, mechanism for compose **A-8**
> (self-host), and makes project-plan **P-12** reproducible-at-arc-close. Scope: project +
> **A1–A6** (A7/A8 are the post-MVP horizon, owned separately).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| S-1 | **Plan-set → work nodes** — the `design-v1.0.0` plan (project + A1–A6 arcs + their slices) becomes `project`/`arc`/`slice` nodes under `nodes/`, each schema-stamped `<type>/v1.0`; `number`/`name`/`created`/`updated` carried; legacy plan-set MD **intact** (never-delete) | `cargo test -p odm-migrate selfhost_creates_work_nodes` → ok (project/arc/slice nodes of the right type, each `<type>/v1.0`; a byte-snapshot of the plan-set copy is unchanged after) | serious | arc-plan slice04 / 0013 §9 | done | attested — `selfhost::self_host` (new `odm-migrate` module) reusing the slice01 rails; `selfhost_creates_work_nodes` → ok. **Real cutover committed**: `odm self-host docs/design-v1.0.0` → **45 work nodes** (1 project + 6 arcs + 38 slices) under `nodes/`, each `<type>/v1.0`; plan-set MD byte-intact. Mechanism = **directory-structure adapter** (the dir tree *is* the structure; no prose parse) — see the closing-report deviation. First mint of `project/v1.0`·`arc/v1.0`·`slice/v1.0`. | Mechanism flagged in closing-report. |
| S-2 | **Containment tree** — `arc part_of project`, `slice part_of arc`, matching the plan-set hierarchy; the work-decomposition children rules (project→arc→slice) hold; **no orphan work nodes** (project is root; every arc/slice parented) | `cargo test -p odm-migrate selfhost_tree_matches_hierarchy` → ok (imported edges reproduce the dir tree) AND `cargo test -p odm-cli check_no_orphan_work_nodes` → ok | serious | arc-plan slice04 / node_type children rules | done | attested — `selfhost_tree_matches_hierarchy` + `check_no_orphan_work_nodes` → ok: project is the root (no `part_of`), every arc `part_of` project, every slice `part_of` its arc; no orphan finding on the real corpus. Containment only (work↔odd reference edges out). | The directory hierarchy **is** the tree — no prose parse for structure. |
| S-3 | **Gate status reflects reality (Asserted)** — each arc/slice gate position derives from the plan (closed arc ⇒ gates reached; active ⇒ partial; planned ⇒ none) at `Evidence::Asserted`; status source = ledgers / Project-Ledger P-rows | `cargo test -p odm-migrate selfhost_status_from_plan` → ok (a closed arc [A5] imports with gates reached; an active arc [A6] partial; a planned slice none) | serious | arc-plan slice04 / slice01 (state→cumulative-gate-at-Asserted) | done | attested — `selfhost_status_from_plan` → ok (closed arc → terminal `verified`; active arc → `in-progress` not `complete`; planned slice → none) at `Evidence::Asserted`. Arc-close signal = **P-row `done` OR arc-level close file** (A1/A2 have no close file → the P-row is authoritative). Slice-complete = `closing-report.md` presence. Real rollup: A1–A5 verified, A6 planned+in-progress. | A1–A3 `closing-report.md` gap → P-row. Fidelity bar = plausible asserted status. |
| S-4 | **Mixed-corpus `check` green** — the 13 `odd` nodes (slice02/03) + the new work tree coexist; `odm check` exits 0 (or every finding is a **Warning** — e.g. an undeveloped stub — never an **Error**): work nodes parented, docs orphan-exempt, no dangling edges, **no wrong-type-field** on work nodes, schema markers valid | `cargo test -p odm-cli check_green_on_self_hosted_corpus` → ok (exit 0 on odd + work) | serious | arc-plan slice04 (real acceptance) / slice03 bubble-up | done | attested — `check_green_on_self_hosted_corpus` → ok. **Real corpus**: `odm check` → "ok (58 node(s), no problems)" exit 0 (13 odd + 45 work). Green-by-construction: work nodes parented + carry only `{part_of, gates, decomposed}` (never `{supersedes, affects}` → no wrong-type-field); closed arcs affirm `decomposed` (no advanced-without-decomposition warning); schema `<type>/v1.0` valid. | slice03's per-type validity meets work nodes for the first time — the bucket rule keeps it green. |
| S-5 | **`orient`/`rollup` reproduce the real state** — `odm rollup` over the self-hosted plan shows A1–A5 done, A6 active, the rest planned; `odm orient` gives a coherent brief (vision → focus → ready/blocked → integrity). **The loop closes.** | `cargo test -p odm-cli rollup_reflects_self_hosted_state` + `orient_runs_on_self_hosted_corpus` → ok | serious | arc-plan A-8 / project-plan P-12 | done | attested — `rollup_reflects_self_hosted_state` (A5 `verified` reached; A6 `in-progress` reached, `complete` not) + `orient_runs_on_self_hosted_corpus` (resolves the self-hosted project, not the empty fallback) → ok. Real `odm rollup` reproduces A1–A5 done / A6 active; `odm orient` runs clean. **The loop closes.** Needs `[gates.*]` in `odm.toml` (added this slice). | `odm rollup --json` regenerating the dashboard is the **named follow** (out of bar). |
| S-6 | **Reflexive-import safety** — `--dry-run` previews the whole cutover writing nothing; the **project (root) node is created last**; a git checkpoint is documented; a re-run is **idempotent** (creates 0) and **never-deletes**; the reflexive case (slice04/arc06 nodes describe their own cutover) is handled by idempotence + last-ordering | `cargo test -p odm-migrate selfhost_dry_run_writes_nothing` + `selfhost_idempotent` → ok (dry-run → 0 on disk; re-run → 0 created; byte-snapshot of the plan-set unchanged) | serious | arc-plan open Q (cutover safety) / slice01 M-3/M-4 | done | attested — `selfhost_dry_run_writes_nothing` + `selfhost_idempotent` → ok. `self_host` persists children-up (slices → arcs → **project last**, via `persist_rank`). Real cutover on a committed checkpoint; re-run → "0 created, 45 skipped"; plan-set MD intact. Reflexive slice04 node handled by idempotence + last-ordering. | Both representations (MD + nodes) coexist post-cutover — retirement is slice06. |
| S-7 | **Gates + no regression + no index change** — clippy `-D warnings`; no `unsafe`; ≥ 90% new-path line cov; **full workspace green**; **no `odm-index` change** (self-host does not require the index) | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-migrate/src` AND `cargo llvm-cov --summary-only -p odm-migrate` → **line** ≥ 90% AND `cargo test --workspace` green AND `git diff --stat release/1.0.x...HEAD -- crates/odm-index` is empty | serious | CLAUDE.md / arc-plan dependencies (A6 ⟂ A4) | done | attested — clippy `-D warnings` exit 0; no `unsafe` in `crates/odm-migrate/src`; line cov: selfhost.rs 96.25%, lib.rs 99.21%, mapping.rs 99.06%, legacy.rs 93.86% (all ≥ 90); `cargo test --workspace` green (52 suites); **`git diff crates/odm-index/` empty**. | The self-host layer rides the store + command surface (A1/A3), not the index. |

## What Worked

- **The directory hierarchy *is* the tree — zero prose parsing for structure.**
  `arcNN-*/` are arcs, `arcNN-*/sliceMM*/` are slices, the root is the project.
  Structure came straight from the filesystem; only names (doc H1) and status
  (close signals) needed reading. The most fragile part the slice-doc feared
  (parsing structure from prose) simply didn't exist.
- **Green-by-construction on the mixed corpus, as slice02/03 predicted.** 58 nodes
  (13 odd + 45 work), `odm check` exit 0 with **no findings at all**. Work nodes
  carry only `{part_of, gates, decomposed}` — never the document-only
  `{supersedes, affects}` — so slice03's per-type validity rule keeps them clean;
  affirming `decomposed` on closed arcs suppressed the advanced-without-decomposition
  warning; document nodes stay orphan-exempt. The bucket rule earned its keep.
- **A stable, disjoint numbering scheme made idempotence trivial.** project=1000,
  arc N = 1000+100N, slice = arc + position — each number a pure function of the
  node's own place, so a re-run re-derives the same `(type, number)` and skips.
  No global counter that shifts when a slice is added.
- **Children-up persistence gave reflexive-import safety for free.** Slices, then
  arcs, then the project root **last** (`persist_rank`) — a partial failure never
  leaves a dangling root, and the reflexive slice04 node is just another slice,
  handled by idempotence + last-ordering.
- **`odm rollup` reproduced the real project state.** A1–A5 verified, A6
  planned+in-progress — the hand-maintained truth is now derivable from odm
  querying its own nodes. The loop closed.

## Deviations / decisions flagged

1. **Mechanism = directory-structure adapter (not a manifest).** The recommended
   dir+status adapter, reusing the slice01 rails. Structure is the filesystem;
   names are each doc's H1; status is file-presence + one P-row status-column scan.
   The manifest fallback was **not** needed — no fragile per-slice ledger-table
   parsing (the concern the slice-doc raised) was required.
2. **Arc-close status = P-row `done` OR arc-level close file.** A1/A2 predate the
   arc-level `closing-report.md` (the P-1/P-2 disclosed gap), so file presence
   alone is insufficient; the Project-Ledger P-row is the authoritative signal
   (`| P-N | … | done |` → arc N closed). A robust column scan, not full table
   parsing. Slice-complete = `closing-report.md` presence.
3. **Scope = A1–A6 only (arc07/arc08 excluded).** The post-MVP horizon (0 slices,
   owned separately) is not imported — importing it would create undeveloped-stub
   arcs and collide with in-flight work. `arc_in_scope` caps at 6. **Flag:** if the
   full roadmap should be represented, A7/A8 import as `planned` stubs — a
   coordination call, not this slice's default.
4. **`[gates.*]` added to `odm.toml`.** Self-hosting work nodes requires the
   work-node gate-sets (project/arc/slice) so `rollup`/`orient`/`check` render and
   validate status — the index adapter drops gates for a type with no configured
   set. The sequences are the canonical ODD-0013 §5.1 sets; the importer stamps
   status against the same sequences. A genuine self-host prerequisite, flagged.
5. **`created`/`updated` = cutover date (today).** The plan docs' own authorship
   history lives in git; the *node* representing each is created at cutover. Fresh
   ULIDs (like slice01/02), so idempotence keys on `(type, number)`, not identity.
6. **A closed arc affirms `decomposed` with its slice set** — honest (a closed arc
   *has* fully decomposed) and it suppresses the spurious
   advanced-without-decomposition warning. Affirmed with the exact children → no
   decomposition-drift.
7. **No model amendment.** The work-node model, `check`, and gates hosted the plan
   faithfully as-is — no node was hand-fudged to force green.

## Closure

Closed at commit `4ac36f6` on 2026-07-07. Verified by: CC (proposed-done, attested);
CDC to reproduce. Rows: 7. Done: 7. Deferred: 0. No-op: 0.
On close → bubble up to `arc-plan.md` (A-4 + the A-8 mechanism) per
LEDGER-DISCIPLINE v2.0 §A. **odm self-hosts** — project-plan **P-12** becomes
reproducible-at-arc-close; only PM-skill (slice05) + prose-retirement (slice06)
remain before the A6 arc-close.
