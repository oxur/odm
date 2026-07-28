---
id: 01KWXMBBTKTMM6R4ZX4CEXH832
number: 1104
type: slice
schema: slice/v1.1
name: Store layer
created: 2026-06-22
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc01-substrate-node-crud/slice04-store-layer/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KWXMBBTKKTRXE1DF6WYSG5JB
status:
  built:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  planned:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  tested:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
---
# Slice 04 (Arc 01) — Store layer (plan-of-record)

> Refs: ODD-0013 §6 (storage/layout), ODD-0014 (atomic-write/fsync). `depends_on:`
> slice03 (frontmatter parse/emit) + slice01 (the `odm-store` crate). Git via `gix`.

## Goal

Persist nodes as the source of truth: the `nodes/YYYY/MM/<ULID>.md` layout (path
computed from the ULID's creation timestamp), atomic writes, git integration, and a
full-scan load into memory. **Done when a node set persists and reloads identically,
git-tracked, with the path derivable from the id alone.**

## Scope

**In:** path-from-ULID (`nodes/<YYYY>/<MM>/<ULID>.md`, creation-time month from the
ULID — file never moves on retitle/reparent); atomic write (write-temp + fsync +
rename + dir-fsync, per ODD-0014); `gix` stage/commit/status; `odm.toml` via confyg
layered search (reuse legacy config logic); full-scan load (walk `nodes/`, parse via
slice03); locate-by-id in O(1) (path is a pure function of the id); self-heal on a
missing dir. Harvest the surviving legacy `git`/`config`/`filename` utilities.

**Out:** the incremental index/cache (Arc A4 / `odm-index` — this slice full-scans);
CRUD commands + CLI (slice05); `check` (slice06); rollup (Arc A3).

## Verification

`cargo test -p odm-store` green; **persist→reload round-trip** of a node set; path
computed from id matches; atomic-write leaves no partial file on simulated failure;
`gix` commit works (in a temp repo); clippy `-D warnings`; coverage ≥ 90%. Rows in
`ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). Then slice05
(CRUD commands) drives the store from the CLI.
