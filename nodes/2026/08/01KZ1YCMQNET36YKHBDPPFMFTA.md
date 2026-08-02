---
id: 01KZ1YCMQNET36YKHBDPPFMFTA
number: 572205400
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 16 (Migration Fidelity): orient P-12 readiness (exclude retired nodes)'
created: 2026-08-01
updated: 2026-08-01
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice16-orient-p12-readiness/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK41BNR96RQ537Y9KBZW
---
# CC Prompt — Slice 16 (Migration Fidelity): orient P-12 readiness (exclude retired nodes)

**Small, targeted slice — the last step before the P-12 demonstration.** The arc-close freeze fired: the
project-vision pair collapsed, the corpus is byte-faithful (0 drift), `check` is green. But `odm orient`
prints **"2 projects, none selected — choose one"**, listing the active `#1000` **and the retired `#1001`**.
`orient.rs` enumerates project nodes with **no `retired()` guard**, so the tombstone counts. `orient`
already auto-orients when exactly one project exists — so excluding retired is sufficient for a clean P-12.

> **Start condition:** on `release/1.0.x`, s15 (+ iteration 1) merged. **Fixture only — no `odm`-branch
> commit.**

## Read first

`slice16-orient-p12-readiness/ledger.md` (5 rows) + `slice-doc.md`. `command-surface-uat-checklist.md`
**F-22** (this is its P-12-critical slice; the rest — `next` etc. — stays LLM-arc). `crates/odm-cli/src/orient.rs`
(the project enumeration ~line 63; the READY set render).

## Load skills

- `/rust-guidelines` (anti-patterns first) · `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **Project selection** (F-1). The `node_type() == Project` filter (orient.rs:~63) also requires
   `f.retired().is_none()`. A retired project is never counted, listed, or offered; a single remaining
   active project auto-orients through the existing `len() == 1` path.
2. **READY set** (F-2). The READY / next-actions list `orient` renders excludes retired nodes.
3. **Audit** (F-3). Confirm no other `orient` enumeration (BLOCKED, counts) surfaces a retired node — one
   consistent "orient never surfaces a retired node" rule. A read, not a rewrite.
4. **Fixtures** (F-1/F-2): 1 active + 1 retired project → auto-orients to the active + renders its vision,
   no "N projects" prompt; a retired node in the ready frontier is absent from READY.

## Constraints (flag, don't silently change)

- **orient-only.** `next` + every non-`orient` surface stays untouched (broader F-22 → LLM arc). Don't touch
  retirement semantics or the collapse — `#1001` stays a retired tombstone; s16 only stops `orient`
  *surfacing* it.
- **No live store mutation. No `odm`-branch commit.**
- No `unsafe`; typed; clippy `-D warnings` clean; cover the new predicate.

## Deliverables

The `orient.rs` change + fixtures on `release/1.0.x`; `ledger.md` evidence per row; `closing-report.md` —
the walk + the v2.0 bubble-up (orient demonstrates P-12; the arc-close runs next). Branch: `release/1.0.x`.

## Working agreement

Amend don't work around; flag deviations; five-iteration cap. Your `done` is proposed-done — CDC reproduces
the predicate + fixtures + a re-run of `odm orient` on the frozen store (single project + its vision). On
close, bubble up to `../arc-plan.md`.
