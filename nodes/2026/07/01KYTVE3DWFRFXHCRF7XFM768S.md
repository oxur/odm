---
id: 01KYTVE3DWFRFXHCRF7XFM768S
number: 505225400
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 11 (Migration Fidelity): Synthesis capability + L-8b reconciliation'
created: 2026-07-29
updated: 2026-07-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice11-synthesis-l8b/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-31
edges:
  part_of: 01KYSX4R1GDJQXPPGFWXP5ZZWK
---
# CC Prompt — Slice 11 (Migration Fidelity): Synthesis capability + L-8b reconciliation

Build odm's **synthesis** step — a node that *supersedes* many sources, verified by regime — and **clear
the L-8b release-gate** (reconcile the four normative ODDs to their authoritative state dirs). **All in
code + docs, fixture-proven, with the live `.worktrees/odm` store untouched** — the live application (fire
the vision synthesis + re-snapshot every drifted node) is **s12**.

> **Start condition:** on `release/1.0.x` (green, s10 + iteration 1 merged). **Fixture + doc only — this
> slice writes no `odm`-branch commit and fires no synthesis on the live store.** The L-8b changes are
> `git mv`s + `state`-frontmatter edits under `docs/design/` (the doc tree), not store mutations.

## Read first

1. `slice11-synthesis-l8b/ledger.md` (10 rows) — the spec of "done."
2. `slice-doc.md` (esp. **Why** + the In/Out split); **ODD-0025** §2.3 (synthesis is a separate superseding
   step; `supersedes`→`Vec`; concatenation hash-gated vs editorial-merge attested; tooling-guaranteed
   bidirectional lineage) and **§4** (the amendments); **design-notes** F1/F2 (both DECIDED).
3. **`arc-release-hardening/uat-coverage-audit.md` §L-8** — the authoritative L-8b definition: which ODDs,
   and the state dir each one's authority implies. Confirm the exact target state per doc against it.
4. **The code you change:**
   - `crates/odm-core/src/frontmatter.rs` (`supersedes: Option<Supersedes>` → a `Vec`) + every read site
     (index/adapter/CLI); `superseded_by` stays derived.
   - the `check` path (`odm-cli/src/commands.rs` + wherever `check` assembles rules) — the new
     bidirectional-lineage rule.
   - `crates/odm-migrate/src/replan.rs::vision_from_plan` — the current bespoke project-vision extraction;
     replace it (in capability + fixture) with a modeled **editorial-merge synthesis** superseding a 1:1
     `project-plan` node. **Fixture only — do not fire live.**
   - the synthesis step itself (new code): records `source.synthesis` + multi-path `source`; the
     `concatenation` deterministic-join hash gate; the `editorial-merge` attestation path.

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `03-error-handling.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **`supersedes` → `Vec`** (F-1). Model + all read sites; keep `SupersedesKind`; `superseded_by` derived.
2. **Bidirectional-lineage invariant + `check` rule** (F-2). Every synthesis writes forward edges; reverse
   derived; `check` **Error** on a supersede whose target is missing or any lineage inconsistency.
3. **The synthesis step + two regimes** (F-3/F-4/F-5). Record `source.synthesis` + multi-path `source`;
   `concatenation` hard-gated against a **defined, documented deterministic join** (order/separator/
   per-source `normalize`); `editorial-merge` accepted only with intact lineage + a recorded attestation.
4. **Fixture-prove the project-vision re-cast** (F-6). On a `TempDir`: the vision becomes an
   editorial-merge synthesis superseding a **1:1 `project-plan` node** (faithful body, hard-gated).
   **No live store write.**
5. **L-8b — correct the four ODDs' authoritative `state:`** (F-7). The real fix is the **`state:` field,
   not the folder**: **update each doc's `state:` frontmatter to its true authority — confirm the target
   per doc against `arc-release-hardening/uat-coverage-audit.md` §L-8, don't guess** — for ODD-0013, 0017,
   0018 (folding in 0013's 0019/0020 amendments; e.g. 0013 `draft`→`accepted`). Then `git mv` each file
   into the matching `NN-state/` dir so the browsable view agrees (clean rename, history follows). The
   node's authority (its gate vector, derived from `state:`) reads the *stale* value until **s12
   reconciles** — do **not** reconcile the nodes or touch the store here.
6. **Fixtures** for F-1…F-6: list-supersede round-trip; broken-lineage → `check` Error; synthesis type +
   multi-path; concatenation gate pass/fail; editorial-merge lineage+attestation; the vision re-cast.

## Constraints (flag, don't silently change)

- **No live store mutation. No `odm`-branch commit. Fire no synthesis on `.worktrees/odm`.** All live work
  is s12.
- **Migration stays strictly 1:1 + hard-gated** (unchanged); synthesis is the *separate* superseding step,
  never migration. **Amend ODD-0025/0013 (cited), don't work around** if the deterministic-join or lineage
  model needs a line.
- **L-8b is a doc-tree change** — moving an ODD does not break coverage (`match_odd` covers by number, not
  path); do **not** try to also update the live nodes' `source.paths` here — that reconcile is **s12**.
- Don't pull **s12** (the live vision mint, all drift re-snapshot, the composition/P-12 demo) or **L-8a**
  (the post-1.0 design-corpus migration) forward.
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables

The code change on `release/1.0.x` (synthesis capability + the lineage check + the fixture vision re-cast)
and the L-8b `git mv`s/state edits; `ledger.md` evidence per row (`attested` → CI — cite test names,
counts, exits; for L-8b cite the moves + resulting state dirs); `closing-report.md` — per-row walk, the
deterministic-join definition, the L-8b before/after state-dir table, any ODD amendment made, **plus the
v2.0 Bubble-up** (did s11 land synthesis + clear L-8b; what it revealed; the silent-drop diff vs In/Out;
confirm s12 now carries the full live close). Branch: `release/1.0.x` only.

## Working agreement

Amend don't work around; flag every deviation; five-iteration cap. The live application is **already
carved out to s12**, so the capability + L-8b should fit one context — but if not, split (land
`supersedes`-Vec + the lineage check + synthesis regimes first; split the vision-recast fixture and/or
L-8b) and flag CDC. Your `done` is proposed-done — CDC reproduces the code + fixtures + the doc moves + CI.
On close, bubble up to `../arc-plan.md` (MF-7/MF-8 done; s12 next, carrying the full live close).
