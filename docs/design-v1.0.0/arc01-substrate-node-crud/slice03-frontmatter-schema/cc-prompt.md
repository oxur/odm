# CC Prompt — Slice 03 (Arc 01): Frontmatter schema + round-trip

Define the on-disk node format and make it round-trip-stable. This slice also makes
the YAML-library decision deferred from slice01.

> **Start condition:** slice02 (identity types) CDC-closed. Else hold.

## Read first
1. `slice03-frontmatter-schema/ledger.md` (10 rows).
2. `slice-doc.md` (same dir) — incl. the YAML-lib decision + isolation rule.
3. `docs/design/01-draft/0013-odm-architecture-design.md` **§2.3** (the normative
   schema + canonical field order) and **§3** (edge kinds).

## Load skills
- **rust-guidelines**: `11-anti-patterns.md`, `02-api-design.md`, `05-type-design.md`,
  `03-error-handling.md`, `13-documentation.md`.
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task
- A `frontmatter` module: parse `---`-delimited YAML + markdown body; emit in the
  canonical field order from §2.3 such that **`parse ∘ emit = identity`**.
- Schema: id, number, type, name, created, updated, `origin`, `reserved`, `tags`,
  `component`, and the **edges block** (`part_of`, `depends_on` + optional
  `satisfied_at`, `blocked_by`, `consumes`, `verifies`, `supersedes {node, kind}`,
  `affects`, `tears`).
- **Unknown-key preservation** (e.g. `#[serde(flatten)]` catch-all) so status /
  desired_facts round-trip cleanly before their slices model them.
- **YAML lib:** `serde_yaml_ng` (fallback `serde_norway` if stale) added to
  `[workspace.dependencies]`, and **isolated behind the `frontmatter` module** — no
  YAML-crate type in any public API.

## Constraints (flag, don't silently change)
- `parse ∘ emit = identity` is the headline invariant — proptest it, including
  unknown-key preservation.
- Do NOT model status vectors or desired_facts here (later slices) — but they must
  survive round-trip as preserved keys.
- No `unsafe`; typed parse errors (`thiserror`) with position where feasible;
  coverage ≥ 90%.

## Deliverables
Green test/clippy/coverage; `ledger.md` evidence per row; `closing-report.md`
(per-row walk for all 10, What Worked, uncertainties — incl. the YAML-lib
maintenance check). Feature branch (`slice03-frontmatter-schema`); not `main`.

## Working agreement
Amend don't work around; five-iteration cap; proposed-done → CDC via CI/local 1.85+
before slice04 opens.
