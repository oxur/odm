# CC Prompt — Slice 05 (Migration Fidelity): Source-based identity

Retire `number` as a **correctness key**: make **`source.paths`** the identity/idempotence key across
`self_host`, the repair/backfill path, and coverage matching, and give **every** node a `source`
record. Prove it on **fixtures** — no live mutation (the live run is s06).

> **Start condition:** on `release/1.0.x`. **Fixture-only — do NOT touch `.worktrees/odm`** (F-10
> hard). If `1.0.x` isn't green, hold. **Why this slice:** `number` is *derived* and has been doing
> an *identity* job — the root of the s04 named-arc re-run-duplicate hazard and the coverage gap.
> Once `source` is the key, `number` is a pure display label and both dissolve. If the four pieces
> exceed one comfortable context, land the **idempotence key + backfill** first (the correctness
> core) and split **coverage-by-source** out toward s07 — flag CDC, don't grind past five iterations.

## Read first
1. `slice05-source-identity/ledger.md` (12 rows) — the spec of "done."
2. `slice-doc.md`; `../design-notes.md` §3 **F12** (DECIDED-done-here); `../arc-plan.md` **v1.9**.
3. **ODD-0025** §2.0/§2.2 (`source`), **§2.8** (update-in-place — extend it), **§5** (coverage →
   exact set-difference on `source`), §2.3 (synthesis — why the project node is excluded).
4. **The code you change:**
   - `crates/odm-migrate/src/selfhost.rs` — the `(type, number)` idempotence key (the
     `existing_work_keys`/pass-1 loop), `repair` (generalize to non-stubs), `named_arc_number`
     (→ name-derived), `PlanNode.source`.
   - `crates/odm-migrate/src/coverage.rs` — the matcher (→ `source.paths`-based).
   - `crates/odm-migrate/src/fidelity.rs` — reuse `verify_body_hash` / `build_source`.
   - `crates/odm-core/src/frontmatter.rs` — `Source` / `source` accessor (read-only here).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `03-error-handling.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task
1. **Source-keyed idempotence** (`self_host`): key on **`source.paths`**, not `(type, number)`. Build
   `by_source` from nodes that have `source`; a node **lacking** `source` matches by structural
   coordinate for its **one-time** population, then by `source` thereafter. A re-run must find an
   already-imported node by its stable source path.
2. **Backfill / reconcile-to-source**: generalize `repair` so it also covers **non-stub** nodes that
   lack `source` — match → source by coordinate, run the existing body through the **hard gate**
   against the source body (faithful = no-op pass; drifted = a `BodyHashMismatch` surfaced, not
   swallowed), add `source`. **Exclude the project node** (its body is a synthesis of
   `project-plan.md` §1 — ODD-0025 §2.3, s08); flag it, don't force the 1:1 gate.
3. **Source-based coverage matching** (`coverage.rs`, ODD-0025 §5): match source-doc → node by
   `source.paths` (exact) instead of the coordinate/number heuristic — which resolves named arcs too.
4. **Name-derived stable handle**: replace `named_arc_number(index)` (position-based) with a handle
   **derived from the slug** (into the ≥ 1900 band, collision-handled) — stable under adding arcs,
   recomputable from the name. Display-only; nothing keys on it.
5. **Tests (fixtures/`TempDir`):** source-keyed idempotence + the coordinate→source transition;
   **the s04 hazard now impossible** (add a named arc + re-run → no duplicate); backfill (faithful
   pass, drifted fail); project-node exclusion; coverage-by-source resolves a named arc; name-derived
   handle stable + recomputable.

## Constraints (flag, don't silently change)
- **Fixture-only. Do NOT mutate `.worktrees/odm`** (F-10 hard). Live run is s06.
- **`source` is the identity key** — do not leave `(type, number)` doing correctness work anywhere
  (idempotence, coverage). `number` is display/CLI-lookup only after this slice.
- **Extend `repair`, don't fork it** — the backfill is the same op relaxed past the stub filter, with
  the project-node/synthesis exclusion. Reuse `fidelity`.
- If a "faithful" node fails the gate on backfill, that is a **finding to surface**, not to suppress.
- No `unsafe`; typed errors (`thiserror`); coverage ≥ 90% (line), target 95%.
- Don't pull s06 (live run), s07 (`artifact` minting + check-wiring), or s08 (synthesis, incl. the
  project node) forward.

## Deliverables
Green `cargo test -p odm-migrate` + `-p odm-core` + clippy + coverage; `ledger.md` evidence per row
(`attested`); `closing-report.md` — per-row walk **plus the v2.0 Bubble-up to the arc** (did s05
retire number-as-key; what did it reveal — e.g. a node that fails the fidelity gate on backfill, a
coverage edge; the silent-drop diff). Branch: `release/1.0.x` (per the arc's fast-forward pattern).

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap (+ the split-escape); your `done`
is *proposed-done* (`attested`) → CDC reproduces. On close, bubble up to `../arc-plan.md`.
