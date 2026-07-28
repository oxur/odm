# Slice 02 (Migration Fidelity): The fidelity model (ODD-0025)

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row must reach ≥ `reproduced` at
> slice scale. **Design/model slice** — the deliverable is a document; "reproduced" means an
> independent reader confirmed the decision is recorded unambiguously and consistently (no code /
> `cargo` rows). Mints no nodes, changes no store schema, writes no code. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | ODD-0025 exists under `docs/design/01-draft/`, registered (frontmatter parses; `number: 25`, next-available; `state: Draft`; format matches a sibling e.g. 0024) | file exists; `awk` the frontmatter fence — `number`/`title`/`state`/`tags`/`version` present; `number`==25 and unused | serious | slice-doc | open | | Title ≈ "Migration Fidelity: provenance, synthesis, artifact nodes & frontmatter fidelity". |
| F-2 | **Provenance sub-map** specified: `source_paths` (list), `source_class`, `normalization`, `migrated_by`, `migrated_on` — computed at migration, **no stored hashes** | grep ODD for the schema block + the explicit "no stored hash / computed at migration" statement (F5) | serious | design-notes F5 | open | | Lands via `Frontmatter.extra` flatten before formal typing. |
| F-3 | **No-transform body rule** recorded: body = post-FM (FM docs) / whole file (FM-less); no synthesized H1 / header injection | grep ODD for the body definition + no-transform statement (F3) | serious | design-notes F3 | open | | The old transform is the root cause of the 44 stubs. |
| F-4 | **Body-hash gate** recorded: hard-fail on `sha256(normalize(source)) != sha256(normalize(node))`, migration-time only | grep ODD for the gate + hard-fail semantics | serious | design-notes F2/F5 | open | | |
| F-5 | **Normalization boundary** decided + recorded (F4): `trim + CRLF→LF`, nothing else; rationale (internal changes still fail; line-endings are platform artifacts) | grep ODD for the normalization rule + `normalization: trim+lf` in the provenance schema | serious | design-notes F4 (CDC-proposed) | open | | **Operator-confirm the ruling before authoring.** |
| F-6 | **Synthesis model** recorded: `supersedes`→`Vec`; `superseded_by` derived + tooling-guaranteed + a check; synthesis-type {concat hash-gated / editorial-merge attested / other}; two-axis vs `SupersedesKind` | grep ODD for the Vec change, the bidirectional-guarantee requirement, and the synthesis-type set | serious | design-notes F1/F2 | open | | Concatenation join must be deterministic (order, separator, per-source trim). |
| F-7 | **`artifact` (supporting-doc) node class** defined (F9): one class, `part_of` nearest **modeled** scale (slice / arc), **no "chunk" node scale**; distinct from work nodes | grep ODD for the class definition + the explicit "no chunk scale" statement | serious | design-notes F9 (CDC-proposed) | open | | **Operator-confirm.** Related to arc-plan's "artifact-vs-work node class". |
| F-8 | **Report self-coverage disposition** recorded (F10): authored reports = `artifact` nodes; regenerable `coverage-report.md` gets a coverage-exemption / ignore rule | grep ODD for both dispositions | serious | design-notes F10 (CDC-proposed) | open | | **Operator-confirm** — this is the sub-cell most wanting a call. Feeds MF-6 (s05). |
| F-9 | **Containment-optional** for design/research doc nodes recorded (F7); orphan/coverage check must not flag a legitimate top-level doc node | grep ODD for the containment-optional rule | correctness | design-notes F7 | open | | |
| F-10 | **Update-in-place fix vector** recorded (F8): match node→source by coordinate, rewrite body+provenance in place, hard-fail gate, preserve id/edges/status; stub = lone-H1 | grep ODD for the update-in-place spec | serious | design-notes F8 | open | | The spec s04 implements; "just re-run migrate" cannot repair (create-or-skip). |
| F-11 | **Frontmatter-fidelity schema mapping** present as a versioned table over originally-present fields; covers the legacy ODD field set; **resolves the `author` and `version` orphan cells** | ODD contains the mapping table; every legacy field (`number/title/created/updated/tags/component/state/supersedes/superseded-by/author/version`) has a target-or-rationale | serious | arc-plan (property 3) | open | | `state`→cumulative gate reach (already faithful, `mapping.rs::reach_cumulative`); `superseded-by`→derived. |
| F-12 | **0013/0020 amendment spec** section names, by section, what ODD-0013 (add `provenance`; add `artifact` type/class) and ODD-0020 (schema versions for both) must gain — precise enough for s03 to apply | grep ODD for the "Amendments required" section citing 0013 §x and 0020 §y | serious | slice-doc | open | | Specification only; physical edits land in s03. |
| F-13 | **Naming disambiguation** line present (`docs/dev/research/` vs `docs/dev/` vs `research` node type) | grep ODD for the three-way disambiguation | polish | design-notes note | open | | |
| F-14 | **No decided-fork drift**: every DECIDED item in `design-notes.md` §3 (F1-F3, F5-F8) is reflected in ODD-0025 without contradiction; nothing silently changed | cross-read `design-notes.md` §3 against ODD-0025, item by item | correctness | LEDGER-DISCIPLINE (spec-keeping) | open | | Any intentional change must be a tracked design-notes update, not a silent one. |

## What Worked

_(At slice close. Patterns that made the slice close cleanly.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 14. Done: _n_. Deferred: _n_. No-op: _n_. On close, bubble up to `../arc-plan.md`
(the model MF-2/MF-3/MF-4/MF-6/MF-7 plan against) per LEDGER-DISCIPLINE v2.0 §A / PM Part IV.
