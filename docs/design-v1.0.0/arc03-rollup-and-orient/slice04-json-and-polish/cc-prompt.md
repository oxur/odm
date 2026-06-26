# CC Prompt — Slice 04 (Arc 03): `--json` + polish

Give `rollup` and `orient`/`brief` a stable, documented `--json` — serialized from the
slice02 model (D-3, no second derivation) — pin every query's `--json` envelope as a
canonical schema, and finish the errors-as-affordances pass. **This closes Arc 03 →
the MVP (A1–A3) is complete.**

> **Start condition:** slice02 (rollup model) + slice03 (orient view) CDC-verified /
> CI-green. If either isn't in, hold.

## Read first
1. `slice04-json-and-polish/ledger.md` (9 rows).
2. `slice-doc.md` (same dir) and the **arc-plan** (`../arc-plan.md`).
3. `../slice01-arc02-cleanup/cdc-verification.md` (forward note: pin the `check --json`
   v2 envelope) and `../slice02-rollup-generation/cdc-verification.md` (ruling 4).
4. The established `--json` pattern: `crates/odm-cli/src/commands.rs` — the `#[derive(
   Serialize)]` view structs (`NodeJson`, `EntryJson`, `TearJson`, the `check`
   envelope), rendered via `serde_json::to_string_pretty`; and `Rollup` in
   `crates/odm-core/src/rollup.rs`.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `02-api-design.md`, `05-type-design.md`
  (serde view structs).
- `/collaboration-framework` → LEDGER_DISCIPLINE.

## Task

1. **`odm rollup --json`**: serialize the `Rollup` model (tree, status vectors,
   ready/blocked with soft-sat, active tears with rationale, provenance, drift +
   deferred slots) as `#[derive(Serialize)]` views → `to_string_pretty`. **Serialize
   the model, don't reshape it** — mirror the Markdown render's source (D-3).
2. **`odm orient --json` + `brief --json`**: serialize the orient view (vision, focus,
   ready/blocked, integrity, drift) over the same model.
3. **Pin the schemas:** shape-lock tests for the `check` v2 envelope (`ok`, `errors`,
   `warnings`, `findings[]`, `tears[]` + field sets), and for the new `rollup`/`orient`
   envelopes (keys + types) — so accidental drift fails CI.
4. **`schema_version` marker:** add an additive top-level marker (e.g. `"schema":
   "rollup/v1"`) to the `check`/`rollup`/`orient` envelopes. *(New convention — if you
   doubt it, flag rather than skip; it's the 0017 forward-compat hook.)*
5. **Document** the canonical `--json` schemas for `check`/`rollup`/`orient` in 0013 §7
   (a "JSON output schemas" note) — the contract ODD-0017 export will target.
6. **Polish:** `--json` stays valid + never bare-errors on the empty corpus and the
   three no-project paths; every `orient`/`rollup` message names an exact fix command.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Reuse the model + the established `--json` pattern** — no parallel JSON-only
  structure that could drift from the Markdown render.
- Additive only on the existing `check` envelope (don't break v2 consumers).
- No `unsafe`; typed errors; valid JSON on every path; coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test -p odm-core -p odm-cli` + clippy + coverage; `ledger.md` evidence per
row; `closing-report.md` (per-row walk for all 9, What Worked, uncertainties named);
the 0013 §7 diff. Feature branch (`arc03-slice04-json-and-polish`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration
cap; your `done` is *proposed done* → CDC verifies (cargo rows via CI / local 1.85+).
**On close, Arc 03 is done and the MVP is complete** — CDC then runs the arc-level
recomposition / silent-drop check before the arc is declared closed.
