---
id: 01KYP5FXS8GEFC0SCSPTQY5NDF
number: 538557100
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 09 (Migration Fidelity): Coverage enforcement *capability*'
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice09-coverage-enforcement/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYP5FRXJBKY1GHW4QF9JQZD7
---
# CC Prompt — Slice 09 (Migration Fidelity): Coverage enforcement *capability*

Build the machinery that makes **"no file left behind" a mechanically-enforced property** — the
`artifact` node type, the discovery reach for the artifact and design/research doc families, doc-coverage
wired into `odm check` as an Error, and the two coverage.rs fixes CDC flagged — **all in code and proven
on fixtures, with zero mutation of the live `.worktrees/odm` store.** The live mint + live backfill are
**s10** (the follow-on live-run slice); do **not** fire them here.

> **Start condition:** on `release/1.0.x` (green). **Fixture-only — this slice writes no `odm`-branch
> commit.** The one hard rule: **do not turn the enforcing coverage check on against the live corpus, and
> do not mint/backfill on the live store.** Both are s10, behind the snapshot → dry-run → fire protocol —
> because pre-mint the live corpus has ~211 uncovered docs, so a live-active enforcing check would go red.
> Prove the capability on `TempDir` fixtures; s10 fires it.

## Read first

1. `slice09-coverage-enforcement/ledger.md` (10 rows) — the spec of "done."
2. `slice-doc.md` (esp. **Why** + the In/Out split); **ODD-0025** §2.5 (`artifact` type + nearest-scale
   containment), §2.6 (mint-all incl. reports), §2.7 (optional containment), §2.2 (`source` sub-map).
3. **ODD-0013** §2.2 (artifact documented, v2.4 — the code variant is yours) and **ODD-0020** (`artifact/v1.0`
   named, v1.2 — the code is yours). The model decisions are **made**; implement against them.
4. s08's `slice08-source-path-portability/cdc-verification.md` (Findings 2–3 restated; the portable key
   your coverage check builds on) and s01's `slice01-coverage-discovery/coverage-report.md` (the exact
   inventory the check enforces and s10 will mint against).
5. **The code you change:**
   - `crates/odm-core/src/node_type.rs` — add the `NodeType::Artifact` variant (+ schema `artifact/v1.0`,
     per-type validity per ODD-0020).
   - `crates/odm-migrate/src/selfhost.rs::discover` — extend the doc-family reach to the artifact family
     **and** the `design`/`research` family (both unreachable/partial today).
   - `crates/odm-migrate/src/coverage.rs` — `representation()` (Finding 2: resolve a **named** arc by the
     s05 name-derived key), `provenance_absence`/`has_provenance_key` (Finding 3: retarget to `source:` or
     retire), and the doc-coverage set-difference that becomes the `check` rule.
   - the `odm check` path (`crates/odm-cli/src/commands.rs` + wherever `check` assembles findings) — wire
     doc-coverage in as an **Error**.

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `03-error-handling.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **`artifact` node type** (F-1/F-2). Add the `NodeType::Artifact` variant + `artifact/v1.0` schema +
   per-type field validity. Implement the §2.5 containment rule: a per-slice artifact `part_of` its slice;
   an arc-/chunk-level artifact `part_of` its **arc**. No `chunk`/`step` scale.
2. **Discovery reach — artifact family** (F-3). Extend `discover()` to find `ledger`/`cc-prompt`/
   `cdc-verification`/`closing-report`/ADR/amendment/UAT docs and mint each as `artifact`, 1:1 body +
   `source`, under the §2.1 body-hash gate, `part_of` nearest modeled scale. **Mint-all** (§2.6): no
   exemption, `coverage-report.md` included.
3. **Discovery reach — design/research family** (F-4). Extend discovery/backfill so the `design`/`research`
   nodes acquire a `source` (they can't be reached today). Containment **optional** (§2.7).
4. **Wire doc-coverage into `odm check`** (F-5). The s01 set-difference becomes a `check` rule returning an
   **Error** for any uncovered `.md`, built on **s08's portable relative `source.paths` key**. Fixture:
   seed an uncovered `.md` → Error; cover it → green; identical from two roots. **Implement + fixture-prove
   only — do not activate against the live store** (s10).
5. **coverage.rs Finding 2** (F-6). `representation()` resolves a named arc (and named-arc slices) via the
   s05 name-derived key so the summary reads a truthful **12/12**, not "8/12".
6. **coverage.rs Finding 3** (F-7). Retarget `provenance_absence` from the stale `provenance:` scan to
   **`source:` presence** (or retire it for the source-presence check). **Decide, justify in a doc comment,
   test.**
7. **Optional containment** (F-8). Ensure the coverage/`orphan` check does not flag a legitimately
   top-level doc node.
8. **Tests** (`TempDir`) for F-1…F-8: type round-trip + validity; nearest-scale containment; mint-all incl.
   reports; design/research `source` backfill; the enforcing check (seed-uncovered → Error, cover → green,
   cross-root stable); named-arc represented; `source`-presence check; top-level-not-orphan.

## Constraints (flag, don't silently change)

- **No live mutation. No `odm`-branch commit. Do not activate the enforcing check on the live corpus.** All
  live work is s10.
- **Implement against the made model** (ODD-0025 §2.5/§2.6/§2.7; the 0013/0020 artifact amendments). If a
  **new** model line is genuinely needed, **amend the ODD + cite it** — don't work around.
- **Build the coverage key on s08's relative `source.paths`** (cross-root stable), not a fresh absolute
  path.
- Don't pull **s10** (the live mint/backfill + live check activation), **s11** (synthesis + L-8b), or
  **s12** (reconcile run, incl. the living-doc-drift reconcile from s08's CDC verification) forward.
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables

The code change on `release/1.0.x`; `ledger.md` evidence per row (`attested` → CI — cite test names,
counts, command exits); `closing-report.md` — per-row walk, the Finding-2/3 fix summaries, the case
decision for F-7, any findings, **plus the v2.0 Bubble-up** (did s09 land the enforcement capability; what
building it revealed; the silent-drop diff vs In/Out; confirm s10 unblocked). Branch: `release/1.0.x`
only (no `odm` branch this slice).

## Working agreement

Amend don't work around; flag every deviation; five-iteration cap. The live run is **already carved out to
s10**, so the capability should fit one context — but if it still won't, split (e.g. land the `artifact`
type + discovery reach first, split the check-wiring + coverage.rs fixes) and flag CDC. Your `done` is
proposed-done — CDC reproduces the code + fixtures + CI (no live store this slice). On close, bubble up to
`../arc-plan.md` (enforcement capability lands; s10 next).
