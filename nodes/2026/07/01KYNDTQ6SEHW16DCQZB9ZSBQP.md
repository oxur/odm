---
id: 01KYNDTQ6SEHW16DCQZB9ZSBQP
number: 58837403
type: slice
schema: slice/v1.1
name: Slice 03 (Migration Fidelity) — Fidelity core (plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice03-fidelity-core/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
status:
  built:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  planned:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  tested:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
---
# Slice 03 (Migration Fidelity) — Fidelity core (plan-of-record)

> Refs: **ODD-0025** (the Accepted model — this slice implements it); `../arc-plan.md` (s03 row +
> Capability properties 1–2 + MF-2/MF-3); `../design-notes.md` §1 (the root-cause code sites).
> `depends_on:` s02 (ODD-0025). **Amends (applies the ODD-0025 §4 spec):** ODD-0013 §2.2/§2.3/§3/§9,
> ODD-0020 §2/§4.
>
> **Sizing (activation call).** This is the arc's heaviest slice. It is scoped to **build and
> fixture-test the faithful-import capability** — it does **not** run migration on the live corpus
> (that repair is s04), which bounds it to one context. **Split-escape:** if, at execution, the
> odm-core field typing + both-importer fidelity + tests exceed a comfortable one-context budget,
> land **(A) core-model typing** first (verified), then **(B) import-fidelity** as an immediate
> follow slice — flag CDC, do **not** grind past the five-iteration cap.

## Goal

Make migration **faithful**: a migrated node's body **is** its source body (no transform), proven
by a **hard body-hash gate**, and every migrated node carries the ODD-0025 `source` record plus the
preserved `author`/`version` fields. Built in `odm-core` (the new typed fields) + `odm-migrate`
(both importers), **verified on fixtures** — the live corpus re-migration is s04. **Done when** the
no-transform verbatim import + hard hash gate + `source`/`author`/`version` population are
implemented in both importers, the three fields are typed in `odm-core` with round-trip + per-type
validity, the ODD-0013/0020 doc amendments are applied, and the fixture tests are green.

## Scope

**In:**

- **odm-core — type three new frontmatter fields** (ODD-0025 §2.2, Q5 = typed at creation):
  - **`source`** sub-map: `paths: Vec<PathBuf>`, `class: String`, `normalization: String`,
    `migrated_by: String`, `migrated_on: NaiveDate` (a small typed struct), emitted in canonical
    order (after `retired`, before/around the migration-metadata region — pick a canonical slot and
    document it).
  - **`author`: Option<String>** and **`version`: Option<String>** — document-node fields; move
    `author` off the `extra` catch-all (`mapping.rs:229` currently routes it there via
    `insert_extra`) onto the typed field.
  - **Round-trip:** `parse ∘ emit = identity` holds with the new fields (extend the proptest).
  - **Per-type validity (0020):** `author`/`version`/`source` valid on document-family nodes; a
    `check` finding if present on a work node (shared-core + per-type-validity model, ODD-0020 §2).
- **odm-migrate — no-transform verbatim import + hard body-hash gate + population**, both importers:
  - **`selfhost.rs`:** replace the stub synthesis at **line ~201**
    (`Document::new(fm, format!("# {}\n", fm.name()))`) with the **verbatim source body** read from
    the node's source (`PlanNode.source`; arc/slice bodies come from `arc-plan.md`/`slice-doc.md`
    within the source dir — resolve the file, read post-frontmatter/whole-file per ODD-0025 §2.1).
  - **`mapping.rs`:** confirm/align the ODD importer to import the body verbatim; type `author`
    (already read at `:229`) and `version` onto the new fields.
  - **Hard body-hash gate** (both): `sha256(normalize(source_body)) == sha256(normalize(node_body))`,
    `normalize` = **trim + CRLF→LF** (ODD-0025 §2.1); a mismatch is a **hard `MigrateError`** that
    fails the migration (not a per-doc skip).
  - **Populate `source`** (+ `author`/`version` where present) at build time from the source
    (`PlanNode.source` is already carried; ODD-0025 §2.2).
- **Apply the ODD-0025 §4 doc amendments** to ODD-0013 (§2.2 `artifact` type *named* for s05; §2.3
  add `source`/`author`/`version`; §3 `supersedes`→list note; §9 the new migration semantics) and
  ODD-0020 (§2/§4 schema-minor note). *(Documentation edits; the `artifact` NodeType enum + minting
  are s05, not here.)*
- **Tests** (fixtures, not the live store): round-trip with the new fields; per-type validity;
  no-transform (imported body == fixture source body, byte-for-byte after normalize); hash-gate
  **pass and fail** cases (a deliberately-mutated body fails hard); `author`/`version` preserved.

**Out:** running migration on the **live corpus** / any minting or store mutation (s04 —
delete-bodyless-then-re-migrate); the **`artifact` NodeType** enum + minting supporting docs (s05);
wiring doc-coverage into `check` (s05); the **synthesis** implementation (s06); widening
`arc_in_scope` past A1–A6 (s04). `supersedes`→`Vec` *may* land here as a small model change if it
falls out of the §2.3 edit, but its synthesis *use* is s06.

## Verification

`cargo test -p odm-core` (round-trip, per-type validity) and `cargo test -p odm-migrate`
(no-transform, hash-gate pass/fail, population) green; `cargo clippy --workspace --all-targets --
-D warnings`; no `unsafe`; coverage ≥ 90% (line) on the changed modules; the no-transform assertion
demonstrates a fixture whose imported body equals its source body, and the hash-gate fail-case
demonstrates a mutated body errors. Rows in `ledger.md`. Cargo rows are `attested`→`reproduced`-on-CI
(CDC sandbox has no 1.85+ cargo).

## Exit

`ledger.md` closed; CDC-verified (`cdc-verification.md`). The faithful-import capability + fields +
gate exist and are fixture-green, so **s04** can run it on the live corpus to repair the 44 stubs
and land `source`/`author`/`version` for real. On close, bubble up to `../arc-plan.md` (MF-2/MF-3
plan against this capability).
