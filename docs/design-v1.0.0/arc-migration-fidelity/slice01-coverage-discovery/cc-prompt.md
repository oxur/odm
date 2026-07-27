# CC Prompt — Slice 01 (Migration Fidelity): Coverage discovery

Build a **read-only** coverage/gap detector in `odm-migrate` and run it on odm's own
`1.0.x/docs` corpus to produce **`coverage-report.md`** — the exact, re-runnable inventory of
what the migration missed. This is the arc's foundation: every later slice is scoped against
these numbers. **This slice mints nothing and changes no schema** — it only *measures*.

> **Start condition:** on `release/1.0.x`. The corpus lives on the orphan `odm` branch at
> `.worktrees/odm/` (nodes under `nodes/`); read it there. **G-1 is closed** (ODD-0024, ULID
> retained) so minting is permitted arc-wide — but **not in this slice**. If `1.0.x` isn't
> green, hold.

## Read first
1. `slice01-coverage-discovery/ledger.md` (9 rows) — the spec of "done."
2. `slice-doc.md` (same dir), `../arc-plan.md` (the MF arc ledger + capability), and
   `../design-notes.md` (the four dimensions + the root causes).
3. `../reconciliation-audit-2026-07-27.md` — the discovery and its ballpark numbers.
4. **`crates/odm-migrate/src/selfhost.rs`** — reuse its arc/slice → coordinate/number logic
   and `arc_in_scope` (the A1–A6 cap is the representation root cause) for the matcher.
5. **`crates/odm-migrate/src/mapping.rs`** — reuse `classify_*` / number+title logic for the
   ODD matcher. And `crates/odm-core/src/frontmatter.rs` (the `Frontmatter` fields — there is
   no `provenance` field yet; that's what F-6 detects).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `02-api-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE (v2.0 — fill evidence at `attested` per commit; CDC reproduces).

## Task
1. **`coverage` module** in `odm-migrate`: enumerate every `.md` under the docs roots
   (`docs/design-v1.0.0/`, `docs/design/`, `docs/dev/`, research), classify each.
2. **Matcher:** map each source doc to a store node — heuristically, coordinates for the
   corpus (reuse `selfhost.rs`), number/title for ODDs (reuse `mapping.rs`). Provenance does
   not exist yet, so matching is structural; **mark the matching basis in the report.**
3. **Four detectors**, each → count + list: (a) doc-coverage (uncovered docs by class);
   (b) representation (arc/slice dirs vs nodes; name the 5 missing arcs); (c) stub bodies
   (≤ 1 non-blank body line, tombstones excluded); (d) provenance-absence.
4. **`odm migrate --coverage`** — a read-only reporting mode (sibling to `--dry-run`) that
   runs the detectors and emits the report. Confirm the flag name against
   `odm-command-inventory.md` before committing to it.
5. **Run it** on odm's corpus; write **`coverage-report.md`** into the slice dir. Reconcile
   the headline counts against the audit (≈44 stubs, ≈211 uncovered, 5 arcs); **explain any
   divergence in the report** — a divergence is a finding, not a number to round off.

## Constraints (flag, don't silently change)
- **Read-only. Mint nothing, write no node, leave the store unchanged** (F-7 is a hard row).
- **Amend, don't work around.** If a criterion is wrong/impossible, raise an amendment.
- **Reuse** `selfhost.rs` / `mapping.rs` logic — do not re-derive coordinate/number rules.
- Heuristic matching is expected and fine — **but say so in the report**; do not present a
  heuristic match as certainty.
- No `unsafe`; typed errors (`thiserror`); coverage ≥ 90% (line), target 95%.
- Resist pulling s02's model or s03's provenance/hash work forward — this slice only measures.

## Deliverables
Green `cargo test -p odm-migrate coverage*` + clippy + coverage; **`coverage-report.md`** in
the slice dir; `ledger.md` evidence per row (at `attested`); `closing-report.md` — the per-row
walk **plus the v2.0 Bubble-up to the arc** (did s01 deliver its assigned piece; what did it
reveal the arc-plan didn't anticipate — e.g. matcher-miss classes, a doc root the plan didn't
list; the slice-scale silent-drop diff). Feature branch (`arc-migfidelity-slice01-coverage`),
not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces. On close, bubble up to `../arc-plan.md` per
LEDGER-DISCIPLINE v2.0 §A / PM Part IV.
