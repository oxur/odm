# CC Prompt — Slice 03 (Migration Fidelity): Fidelity core

Make migration **faithful**: a migrated node's body **is** its source body (no transform), proven by
a **hard body-hash gate**, with every migrated node carrying the ODD-0025 `source` record plus
preserved `author`/`version`. Build it in `odm-core` (new typed fields) + `odm-migrate` (both
importers), **verified on fixtures**. This is the first *implementation* slice of the arc.

> **Start condition:** on `release/1.0.x`. **Fixture-only — do NOT run migration on the live
> `.worktrees/odm` corpus** (that repair is s04; F-10 is a hard row). If `1.0.x` isn't green, hold.
> **This is the arc's heaviest slice** — see the split-escape in `slice-doc.md`: if the odm-core
> typing + both-importer fidelity + tests exceed a comfortable one context, land core-typing first,
> then import-fidelity as a follow — flag CDC, don't grind past the five-iteration cap.

## Read first
1. `slice03-fidelity-core/ledger.md` (12 rows) — the spec of "done."
2. `slice-doc.md` (same dir); **ODD-0025** (`docs/design/04-accepted/0025-migration-fidelity-model.md`)
   — **the model you implement**, esp. §2.0 (origin/source/provenance), §2.1 (no-transform + gate +
   `normalize = trim+lf`), §2.2 (`source`/`author`/`version`), §2.4 (mapping), §4 (amendments).
3. `../arc-plan.md` (Capability 1–2, MF-2/MF-3); `../design-notes.md` §1 (root-cause code sites).
4. **The code you change:**
   - `crates/odm-core/src/frontmatter.rs` — the `Frontmatter` struct + canonical emit order + the
     round-trip proptest; `extra` flatten (`author` currently rides here).
   - `crates/odm-migrate/src/selfhost.rs` — the stub synthesis is **line ~201**
     (`Document::new(fm, format!("# {}\n", fm.name()))`); `PlanNode.source` (line ~130) holds the
     source path; `build_node` (~226) sets it.
   - `crates/odm-migrate/src/mapping.rs` — the ODD importer; `author` is read at ~line 229.
   - `crates/odm-core/src/schema.rs` / the per-type validity path (ODD-0020).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `03-error-handling.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE (fill evidence at `attested` per commit).

## Task
1. **odm-core — type three fields** (ODD-0025 §2.2): a `source` sub-struct
   (`paths: Vec<PathBuf>`, `class`, `normalization`, `migrated_by`, `migrated_on`), and
   `author: Option<String>` / `version: Option<String>` (document-node fields). Put them in a
   **documented canonical slot**; extend the **round-trip proptest**; add **per-type validity**
   (valid on document nodes; a `check` finding on a work node).
2. **odm-migrate — no-transform + gate + populate** (both importers):
   - `selfhost.rs`: replace the stub synthesis with the **verbatim source body** (resolve
     `arc-plan.md`/`slice-doc.md` under `PlanNode.source`; body per ODD-0025 §2.1).
   - `mapping.rs`: import the ODD body verbatim; type `author`/`version` onto the new fields.
   - **Hard body-hash gate** (both): `sha256(normalize(src)) == sha256(normalize(node))`,
     `normalize = trim + CRLF→LF`; mismatch → a **hard `MigrateError`** (not a per-doc skip).
   - **Populate `source`** (+ `author`/`version`) at build from `PlanNode.source`.
3. **Apply the ODD-0025 §4 doc amendments** to ODD-0013 (§2.2 name `artifact` for s05; §2.3 add the
   three fields; §9 the new migration semantics) and ODD-0020 (§2/§4 schema-minor note). *(Docs
   only — the `artifact` NodeType enum + minting are s05.)*
4. **Tests (fixtures only):** round-trip; per-type validity; no-transform (imported body == fixture
   source body after normalize); hash-gate **pass and fail**; `source` populated; `author`/`version`
   preserved.

## Constraints (flag, don't silently change)
- **Fixture-only. Do NOT migrate the live corpus or mutate `.worktrees/odm`** (F-10 is hard).
- **Implement ODD-0025 as written.** If a decision is wrong/impossible, **raise an amendment to
  ODD-0025** — do not work around it or re-decide (esp. `normalize = trim+lf`; the record is
  `source` not `provenance`; **no stored body hash**).
- **Reuse**, don't re-derive: `PlanNode.source`, the existing `MigrateError` per-doc-vs-fatal split,
  the existing round-trip proptest.
- No `unsafe`; typed errors (`thiserror`); coverage ≥ 90% (line), target 95%.
- Don't pull s04 (live re-migration / `arc_in_scope` widening), s05 (`artifact` type + minting),
  or s06 (synthesis) forward — this slice only builds + fixture-tests the capability.

## Deliverables
Green `cargo test -p odm-core` + `cargo test -p odm-migrate` + clippy + coverage; the ODD-0013/0020
edits; `ledger.md` evidence per row (at `attested`); `closing-report.md` — the per-row walk **plus
the v2.0 Bubble-up to the arc** (did s03 deliver the MF-2/MF-3 capability; what did implementing it
reveal the plan didn't anticipate — e.g. a canonical-slot or normalization edge case; the
slice-scale silent-drop diff). Feature branch (`arc-migfidelity-slice03-fidelity-core`), not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap (and the split-escape above); your
`done` is *proposed-done* (`attested`) → CDC reproduces. On close, bubble up to `../arc-plan.md` per
LEDGER-DISCIPLINE v2.0 §A / PM Part IV.
