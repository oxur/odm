# CC Prompt — Slice 04 (Migration Fidelity): Scope + repair capability

Build — and prove on **fixtures** — the capability to import **every** arc/slice dir and to
**repair a stub node update-in-place**, plus execute the ODD-0020 schema-minor bump. **No live
mutation** — firing this on the real corpus is s05.

> **Start condition:** on `release/1.0.x`. **Fixture-only — do NOT touch `.worktrees/odm`** (F-9 is
> a hard row). If `1.0.x` isn't green, hold. Heaviest slice of the arc — if the schema bump + repair
> + import-all + tests exceed one comfortable context, land the cap-removal + repair first and split
> the schema bump out; flag CDC, don't grind past the five-iteration cap.

## Read first
1. `slice04-scope-repair-capability/ledger.md` (12 rows) — the spec of "done."
2. `slice-doc.md`; **ODD-0025 §2.8** (update-in-place repair — the authority), §2.1/§2.2 (gate + `source`).
3. `../arc-plan.md` **v1.6** (the three decisions: cap removed, numbers = handles, capability/live-run split); `../design-notes.md` §3 (F8, F11, F12).
4. **The code you change:**
   - `crates/odm-migrate/src/selfhost.rs` — `MAX_MVP_ARC` (line ~35), `arc_in_scope` (~79, delete it), the pass-1 create-or-skip loop (~150), `PlanNode.source`, `arc_number`/`slice_number`.
   - `crates/odm-migrate/src/coverage.rs:512` — the shared `arc_in_scope` call site (update for the removal).
   - `crates/odm-migrate/src/fidelity.rs` — reuse `verify_body_hash` + `build_source` for repair.
   - `crates/odm-core/src/schema.rs` — `SchemaVersion::CURRENT` (bump `v1.0→v1.1`) + `is_newer_than_current`.
   - `crates/odm-store/src/store.rs` — `persist` (overwrites by id; the update-in-place mechanism), `load`/`load_all`.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `03-error-handling.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task
1. **Remove the cap** (`MAX_MVP_ARC`/`arc_in_scope`) — **delete** the predicate (don't raise the
   constant — v1.6, no new lock-in); `self_host` imports every arc dir; update the shared
   `coverage.rs:512` use so the detector treats all arc dirs as in-scope.
2. **Named-arc handles** — assign named arc/slice dirs a **deterministic, collision-free** `number`
   (a band above A1–A8's 1100–1800; state the rule in code + report). Handle only — not identity/order.
3. **Update-in-place repair** (ODD-0025 §2.8): match a stub node (lone-H1 body — reuse s01's
   predicate) → its source by coordinate → write **verbatim body + `source`** into the **same**
   node via `Store::persist`, **preserving `id`/`edges`/`status`/`number`**, through
   `fidelity::verify_body_hash`. **No `delete`.** Populate `author`/`version` for document nodes.
4. **Schema bump** — `SchemaVersion::CURRENT` `v1.0→v1.1`; new/repaired nodes stamp `<type>/v1.1`;
   **existing `v1.0` nodes stay valid** (additive fields, forward-compat); fix `is_newer_than_current`
   + any `v1.0` assertions. Apply the ODD-0020 §4 edit (record v1.1 as current).
5. **Tests (fixtures/`TempDir` only):** import-all (named + `arc07` + A1–A6 → all imported, handled,
   no collision); update-in-place repair (stub → verbatim body, `source` populated,
   id/edges/status/number **preserved**, `v1.1` stamped); gate-on-repair (mutated body errors);
   schema forward-compat (`v1.0` validates, `v1.1` current).

## Constraints (flag, don't silently change)
- **Fixture-only. Do NOT mutate `.worktrees/odm`** (F-9 hard). The live run is s05.
- **Update-in-place, not delete-then-re-migrate** (ODD-0025 §2.8) — preserve id/edges/status; a
  stub's ULID and its real edges/status must survive the body rewrite. If a stub genuinely cannot
  be updated in place, raise it — don't reach for delete.
- **Implement ODD-0025 / the v1.6 decisions as written.** If something is wrong/impossible, raise an
  amendment — don't work around (esp. the record is `source` not `provenance`; no stored body hash;
  cap **removed** not raised).
- No `unsafe`; typed errors (`thiserror`); coverage ≥ 90% (line), target 95%.
- Don't pull s05 (live run), s06 (`artifact` minting + check-wiring), or s07 (synthesis) forward.

## Deliverables
Green `cargo test -p odm-migrate` + `-p odm-core` + clippy + coverage; the `schema.rs` bump + the
ODD-0020 §4 edit; `ledger.md` evidence per row (`attested`); `closing-report.md` — per-row walk
**plus the v2.0 Bubble-up to the arc** (did s04 deliver the MF-2/MF-3/MF-5 capability; what did
implementing it reveal — e.g. a named-arc numbering edge, a schema-contract surprise; the
silent-drop diff). Branch: `release/1.0.x` (per the fast-forward; no per-slice branch this arc).

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap (+ the split-escape above); your
`done` is *proposed-done* (`attested`) → CDC reproduces. On close, bubble up to `../arc-plan.md`.
