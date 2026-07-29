# Slice 11 (Migration Fidelity): Synthesis capability + L-8b reconciliation

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. **Fixture +
> doc only — no live store mutation** (the live application is s12). Capability rows are fixture-proven
> (`attested` → CI); the L-8b rows are doc-tree moves CDC reviews directly. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **`supersedes` is a list** (F1): `edges.supersedes: Vec<Supersedes>`; index/adapter/CLI read the list; `superseded_by` stays derived, never stored | `cargo test` → a node supersedes ≥2 targets; round-trips parse/emit; no stored `superseded_by` anywhere | serious | ODD-0025 §2.3/§4 | open | | `SupersedesKind` (Obsoletes/Updates) preserved — orthogonal axis. |
| F-2 | **Bidirectional-lineage invariant + `check` rule** (F1, hard req): every synthesis writes forward edges; reverse derived; `check` flags a supersede whose target is missing / any lineage inconsistency | `cargo test` → seed a broken supersede lineage → `check` **Error**; valid → green; the reverse is computed, never read from a stored field | serious | ODD-0025 §2.3 | open | | The back-edge can never be hand-maintained → never drifts silently. |
| F-3 | **Synthesis records type + multi-path source** (F2): `source.synthesis: concatenation\|editorial-merge\|other` + multi-element `source.paths` | `cargo test` → a synthesized node carries `synthesis:` + ≥2 `source.paths` | serious | ODD-0025 §2.3 | open | | Synthesis-type = how merged (distinct from SupersedesKind). |
| F-4 | **`concatenation` hard-gated against a deterministic join** (F2): defined + documented order/separator/per-source `normalize`; body-hash-gated | `cargo test` → a concatenation synthesis passes the gate on a faithful join, **fails** on a tampered one | serious | ODD-0025 §2.3 | open | | Keep as much hard-fail as the regime allows. |
| F-5 | **`editorial-merge` verified by lineage + attestation** (F2): not hash-checkable → accepted with intact supersede lineage + a recorded attestation, never a silent pass | `cargo test` → an editorial-merge synthesis is accepted only with lineage + attestation present | serious | ODD-0025 §2.3 | open | | |
| F-6 | **Project-vision re-cast fixture-proven** (MF-7): the vision becomes an **editorial-merge synthesis** superseding a **1:1 `project-plan` node** (faithful body, hard-gated) — replacing the bespoke `vision_from_plan` synthesis | `cargo test` (`TempDir`) → vision node is a synthesis superseding a faithful `project-plan` node; **no live store write** | serious | MF-7 / design-notes | open | | Fixture only — s12 fires it live. |
| F-7 | **L-8b: correct the four ODDs' authoritative `state:`** (MF-8): update `state:` frontmatter to true authority for ODD-0013, 0017, 0018 (+0013's 0019/0020 amendments), **confirmed per §L-8**; then `git mv` each into the matching state dir | direct review: each doc's `state:` matches its true authority per `uat-coverage-audit.md` §L-8 (e.g. 0013 `draft`→`accepted`), file relocated to the matching `NN-state/` dir, `git mv` history preserved | serious (ship-gate) | uat-coverage-audit §L-8 | open | | The `state:` update is the real fix (it becomes the node's gate vector via s12); the folder move is cosmetic display. Doc-tree only, **no store mutation** — coverage stays green (`match_odd` covers by number, not path); the node authority reads stale until s12 reconciles. |
| F-8 | **No live store mutation; downstream not pulled forward**: `.worktrees/odm` untouched; the live vision mint + all reconcile stay s12; L-8a not started | `git -C .worktrees/odm status` clean after the slice; no synthesis fired live; no design-corpus migration (L-8a) | serious | LEDGER-DISCIPLINE / operator | open | | The store reconcile of the L-8b-moved nodes is explicitly s12. |
| F-9 | **No model drift; ODD amend-not-work-around**: 1:1 migration rule + body-hash gate unchanged; if the deterministic-join or lineage model needs a line, ODD-0025/0013 **amended** (cited), not worked around | cross-read: migration still strictly 1:1; gate unchanged; any model line added is an ODD edit with a dated version-history entry | correctness | ODD-0025 | open | | Synthesis is the *separate* superseding step, never migration. |
| F-10 | **Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) changed modules, target 95%** | clippy exit 0; `! grep unsafe`; `llvm-cov` ≥ 90% | polish/correctness | CLAUDE.md | open | | Same bar as s08–s10. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` (`release/1.0.x`) on `<date>`. Verified by: `<CC then CDC>`.
Rows: 10. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`. On close, bubble up to `../arc-plan.md`: MF-7/MF-8
done (capability + gate); **s12 (reconcile run) next, now carrying the full live close** (vision mint +
all drift re-snapshot + composition/P-12).
