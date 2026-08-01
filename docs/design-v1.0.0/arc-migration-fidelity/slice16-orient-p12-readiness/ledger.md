# Slice 16 (Migration Fidelity): orient P-12 readiness — exclude retired nodes

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. **Fixture slice —
> `.worktrees/odm` untouched.** Code/fixtures class-(a): CDC reproduces by direct read; runtime attested→CI.
> Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **orient project selection excludes retired**: the `node_type() == Project` filter (orient.rs:~63) also requires `retired().is_none()`; a retired project is never counted/listed/offered | fixture: 1 active + 1 retired project → `orient` auto-orients to the active one, renders its vision, no "N projects, none selected" | serious (P-12 gate) | freeze P-12 run | open | | orient already auto-orients on `len()==1`; this is the one predicate that gets it there. |
| F-2 | **orient READY excludes retired**: the READY/next-actions set orient renders drops retired nodes | fixture: a retired node in the ready frontier is absent from READY | correctness (F-22 slice) | UAT F-22 | open | | Same "orient never surfaces a retired node" rule; the retired `#1605` UAT tombstone was the original symptom. |
| F-3 | **Nothing else surfaces retired via orient**: any other orient enumeration (BLOCKED, counts) is retired-consistent | read orient's enumerations; none counts a retired node | correctness | slice-doc | open | | A quick audit, not a rewrite. |
| F-4 | **No scope bleed**: `next` + non-orient surfaces unchanged (broader F-22 stays LLM-arc); retirement semantics + the collapse untouched | diff limited to `orient.rs`; `#1001` still a retired tombstone | serious | scope | open | | |
| F-5 | **Clippy clean; no `unsafe`; the new predicate covered** | clippy `-D warnings` exit 0; `! grep unsafe`; fixtures cover the retired-exclusion branches | polish | CLAUDE.md | open | | |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Fixture-only — no store commit. Verified by: `<CC then CDC>`. Rows: 5. On close, bubble
up to `../arc-plan.md`: s16 done — `orient` demonstrates P-12 cleanly; **the arc-close runs** (MF-9
composition + P-12 demo + bubble-up → Migration Fidelity closes).
