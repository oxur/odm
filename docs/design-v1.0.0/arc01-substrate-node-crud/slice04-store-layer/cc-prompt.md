# CC Prompt — Slice 04 (Arc 01): Store layer

Persist nodes as the source of truth: `nodes/YYYY/MM/<ULID>.md`, atomic writes,
`gix`, `odm.toml`, full-scan load.

> **Start condition:** slice03 (frontmatter parse/emit) CDC-closed. Else hold.

## Read first
1. `slice04-store-layer/ledger.md` (10 rows).
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §6 (+ §6.1); ODD-0014
   (atomic write / fsync, racy-stat lessons for later).

## Load skills
- **rust-guidelines**: `11-anti-patterns.md`, `03-error-handling.md`,
  `04-ownership-borrowing.md` (RAII for files), `02-api-design.md`.
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
- Path-from-ULID: `nodes/<YYYY>/<MM>/<ULID>.md`, month read from the ULID's
  **creation** timestamp; locate-by-id is O(1) (no scan).
- Atomic write: temp + fsync + rename + dir-fsync (no partial files on failure).
- `gix` stage/commit/status; `odm.toml` via confyg layered search; full-scan load
  (walk `nodes/`, parse via slice03). Self-heal a missing `nodes/` dir.
- Harvest the surviving legacy `git`/`config`/`filename` helpers where they fit.

## Constraints (flag, don't silently change)
- Files **never move** on retitle/reparent — the path is a pure function of the
  immutable id. Update-time is metadata, not the path.
- No incremental index here — full scan is fine (Arc A4 adds `odm-index`).
- No `unsafe`; typed errors; coverage ≥ 90%.

## Deliverables
Green test/clippy/coverage (persist→reload round-trip; atomic-write-no-partial;
gix commit in a temp repo); `ledger.md` evidence per row; `closing-report.md`.
Feature branch (`slice04-store-layer`); not `main`.

## Working agreement
Amend don't work around; five-iteration cap; proposed-done → CDC via CI/local 1.85+
before slice05 opens.
