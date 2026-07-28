# Arc — Migration Fidelity: faithful, verifiable, repeatable migration — plan-of-record

> **Named arc, canonical number deferred** (operator call, 2026-07-27) — same §2a treatment as
> Release Hardening / arc-store-home / LLM-command-surface. **v1.0.0 release-blocking:** it sits
> on the **P-12** DoD path (odm cannot credibly claim to self-host while its own corpus is 44
> empty stubs and 6 of 11 arcs) and it **subsumes the standing L-8b pre-ship gate**. Directory:
> `arc-migration-fidelity/` (provisional; settles with the numbering).
>
> **Unblocked by ODD-0024 (G-1 closed):** the id scheme is decided (ULID retained), so the
> minting freeze that gated this arc's node-creation slices is lifted.
>
> `depends_on:` A6·slice04 (the self-hosted corpus this repairs); ODD-0024 (minting unfrozen).
> **Refs:** `reconciliation-audit-2026-07-27.md` (the discovery); ODD-0013/0020 (the node model
> the fidelity checks amend); **ODD-0025** (the fidelity model, Accepted — s02's deliverable);
> `arc-migration-fidelity/design-notes.md` (the decision log this plan draws on).
>
> **Status:** s01–s04 **closed** (2026-07-28); s04 delivered the scope + repair
> **capability** (fixture-proven); **s05 (live repair run) next** — the destructive op fires on
> the real corpus there, deliberately separated from where it was built (operator call, v1.6).
> *Plan late, plan deep* — each slice's open set is written when it becomes active, and its
> sizing is confirmed then.

## Capability

Give odm a **complete, general, verifiable migration capability** — one that brings *every*
documentation file into the store faithfully, proves it did, and can be re-run across many
projects. Four properties define "faithful and verifiable," each enforced by a check rather than
trusted:

1. **1:1 verbatim bodies, hard-gated.** A migrated node's body **is** its source body. Migration
   hashes `normalize(source_body)` and `normalize(node_body)` (`normalize` = trim + CRLF→LF); a
   mismatch is a **hard error** that fails the migration. The importer performs **no body
   transformation** (no synthesised `# {name}` H1, no header injection) — the old transform
   behaviour is the thing that produced the 44 stubs.
2. **A `source` record + preserved `author`/`version` on every migrated node.** A `source` sub-map
   records paths (a list), class, normalization, and the migrating tool + version — computed at
   migration, no hashes stored (content is allowed to change — Version-History sections do). Legacy
   `author`/`version` are preserved as **typed fields** (not dropped, not git-derived). *(0013
   reserves `provenance` for **derived** lineage; this stored record is the distinct `source` axis —
   ODD-0025 §2.0/§2.2, Accepted.)*
3. **Frontmatter fidelity, schema-mapped.** The fields the source actually had are checked to map
   correctly onto the node's fields (e.g. legacy `state: Final` → the cumulative gate reach),
   over a versioned schema mapping. Only originally-present fields are checked.
4. **No file left behind, enforced.** A doc-coverage check — the inverse of `orphan` — asserts
   every `.md` in the migration source has a node. Its absence is *why* ~211 supporting docs went
   silently uncovered; wiring it into `check` makes that class of hole loud, permanently. **No
   scope cap** — the importer walks every arc/slice dir that exists (the `MAX_MVP_ARC` A1–A6 cap
   that caused the hole is removed, v1.6); `number` is a non-structural human handle, so named arcs
   just get one assigned.

Synthesis (merging several docs into one) is **not** migration: it is a separate, later step that
mints a **new** node **superseding** its sources (`edges.supersedes` → a `Vec`, with the
reverse `superseded_by` derived and **tooling-guaranteed** on every synthesis). Concatenation
stays hash-gated; editorial/conceptual merges are verified by lineage + attestation.

Applied first to odm's own `1.0.x` corpus — taking self-hosting from skeleton (44 stub bodies, 6
of 11 arcs, ~211 uncovered docs, 0 source records) to **100%**. Built general so the next project
runs the same capability.

## Exit criteria (arc acceptance — the composition check)

- **Coverage:** every `.md` under `1.0.x/docs/*` maps to a node; the doc-coverage check is green
  (no file left behind). Design/research nodes may be top-level *or* contained — both valid (F7).
  The arc's **own** report/verification artifacts are minted as `artifact` nodes (F10 mint-all,
  ODD-0025 §2.6) so the enforced check does not flag them in perpetuity.
- **Body fidelity:** every migrated node's `normalize(body)` hash equals its source's; the hard gate
  is green across the whole corpus; **no stub bodies remain**.
- **Source record:** every migrated node carries a `source` sub-map, and preserved `author`/`version`
  where the source had them (ODD-0025 §2.2).
- **Representation:** all real arcs (the 6 previously-excluded + A1–A6) and all slices are
  represented; scope is **uncapped** (`MAX_MVP_ARC` removed); named arcs carry an assigned `number`
  handle; supporting-doc children are minted.
- **Frontmatter fidelity:** the schema-mapping check is green over originally-present fields.
- **L-8b cleared:** ODD-0013/0017/0018 (+ the 0019/0020 amendments) reconciled to states that
  reflect their authority.
- **Reflexive:** `odm check` green; `orient`/`rollup` reproduce the hand-maintained truth; **P-12
  (odm self-hosts) is satisfiable at project close** against a *faithful* corpus, not a skeleton.

## Slice breakdown (one-line altitude; load-bearing order)

| Slice | Scope | Mints nodes? | Depends on |
|-------|-------|--------------|------------|
| **s01 — coverage-discovery** ✅ CLOSED 2026-07-27 (CDC-verified) | Build the read-only doc-coverage detector (inverse `orphan`) + sibling detectors; run over all `1.0.x/docs` → the **exact gap inventory**. | no (read-only) | — |
| **s02 — model** ✅ CLOSED 2026-07-27 (ODD-0025 Accepted) | The model ODD: `source` sub-map (§2.0); `author`/`version` typed fields; `supersedes`→`Vec`; the `artifact` node type; frontmatter-fidelity mapping; F4/F7/F10 resolved. → **ODD-0025**. | no | s01 |
| **s03 — migration-fidelity core** ✅ CLOSED 2026-07-27 (CDC-verified) | 1:1 verbatim import + **hard body-hash gate** + `source`/`author`/`version` typing + **no-transform**. Both importers + odm-core typing. | no (fixture-only) | s02 |
| **s04 — scope + repair capability** ✅ CLOSED 2026-07-28 | **Cap removed** (`self_host` imports **all** arc/slice dirs; named arcs get a non-structural `number` handle; `coverage.rs`'s shared predicate updated). **Update-in-place repair** of stub nodes (ODD-0025 §2.8 — real body + `source`, preserve id/edges/status, via `persist` overwrite — no `delete`). **Schema-minor bump executed** (`v1.0→v1.1`, forward-compat proven). Fixture-verified; no live mutation. | no (fixture-only) | s03 |
| **s05 — live repair run** | Fire the s04 capability on the **live** `.worktrees/odm` corpus: repair the 44 stubs, import the 6 previously-excluded arcs + their slices, populate `source`/`author`/`version`, stamp `v1.1`, re-point `context.json`. Verify `check` green, 0 stubs, all arcs/slices represented, `orient`/`rollup` reproduce. | **yes (live)** | s04 |
| **s06 — coverage enforcement** | Mint the supporting-doc child nodes as `artifact` nodes (F10 mint-all incl. the reports); **wire doc-coverage into `odm check`**; attach or top-level the design/research nodes (F7). | yes | s02, s03, s05 |
| **s07 — synthesis + L-8b** | The supersede-based synthesis step (concat hash-gated; editorial-merge attested); re-cast the project vision as a synthesis **superseding** the 1:1 `project-plan` node; reconcile the four L-8b ODDs. | yes | s02, s03 |
| **s08 — reconcile run** | Run the whole capability over odm's own corpus end-to-end; verify no doc uncovered, no orphan, every body-hash passes, all arcs/slices present, `check`/`orient`/`rollup` green. The arc composition + P-12 acceptance demonstration. | — | all |

*Sizing note:* the heaviest are **s04** (capability), **s06** (children-mint + check-wiring), and
the two live-mutation slices **s05**/**s06** — split at slice-activation if an open set won't fit
one context. s04 was split from its live run (s05) at activation (2026-07-27) precisely so the
destructive op is fixture-proven before it fires.

## Arc ledger (composition rows — opens here, closes in `closing-report.md`)

> LEDGER-DISCIPLINE v2.0 §B. Class-(b) rows are **reproduced at arc scale** (an end-to-end run on
> odm's corpus). Status ladder: `asserted < attested < reproduced < reconciled`; a `done` row
> reaches ≥ `reproduced`. All rows open **planned**.

| ID | Criterion | Verify | Significance | Status |
|----|-----------|--------|--------------|--------|
| MF-1 | Doc-coverage check exists and is green on `1.0.x/docs` — every `.md` has a node | run the check on the corpus | serious (no file left behind) | planned — s01 *detector* exists + ran (CDC-verified); the *enforced check* is s06's job |
| MF-2 | Body-hash gate green across all migrated nodes; zero stub bodies remain | re-migrate + gate; count stubs = 0 | serious | planned — s03 built the gate, s04 built + fixture-verified the repair path that clears a stub (`slice04-scope-repair-capability/closing-report.md`); **s05's live run** produces "zero stubs" |
| MF-3 | Every migrated node carries a `source` sub-map (+ preserved `author`/`version`) | grep/`check` over the store | correctness | planned — s03 types + populates the fields, s04's repair path populates it on an existing node too (fixture-verified); the live 60-node corpus gets them in **s05** |
| MF-4 | Frontmatter-fidelity check green over originally-present fields | run the check | correctness | planned — mapping specified in ODD-0025 §2.4 |
| MF-5 | All arcs (incl. the 6 previously-excluded) + all slices represented | count dirs vs nodes = 0 gap | serious | planned — s04 removed the cap + proved named-arc handle assignment is collision-free (fixture-verified, `slice04-scope-repair-capability/closing-report.md`); **s05** mints the missing arcs/slices live |
| MF-6 | Supporting-doc children minted; doc-coverage wired into `odm check` | `check` fails on a seeded uncovered doc | serious (loud-hole guard) | planned — F10 **mint-all** (ODD-0025 §2.6); s06 mints + wires |
| MF-7 | Synthesis lands as supersede lineage; project vision re-cast; bidirectional guaranteed | inspect edges; seed + verify | correctness | planned — model in ODD-0025 §2.3 |
| MF-8 | L-8b: ODD-0013/0017/0018 (+0019/0020) reconciled to authoritative states | inspect states | pre-ship gate | planned |
| MF-9 | **Compose:** odm self-hosts *faithfully* — `check`/`orient`/`rollup` green on a corpus with real bodies, full coverage, source records | project-scale reproduce at arc close | serious (P-12) | planned |

## Version History

### v1.7 — 2026-07-28 — s04 (scope + repair capability) closed; ready for the live run

**Slice 04 is closed** (`slice04-scope-repair-capability/closing-report.md`; 12/12 ledger rows
done, 0 deferred, 0 no-op). Delivered exactly what v1.6 restructured this slice to: the
`MAX_MVP_ARC`/`arc_in_scope` cap is deleted (not raised) — `self_host` now imports every arc/slice
directory it finds, numbered or named; named arcs get a deterministic, collision-free `number`
handle (`NAMED_ARC_BASE = arc_number(8) + NAMED_ARC_STEP`, a `const`-derived value, not an
independent magic number); a new `selfhost::repair` op performs ODD-0025 §2.8's update-in-place
repair (verbatim body + `source` written into the *same* node, `id`/`edges`/`status`/`number` all
preserved, via `Store::persist` overwrite — no `delete` anywhere); and the ODD-0020 schema-minor
bump s03 deferred is executed (`SchemaVersion::CURRENT` now `v1.1`, with a new test proving an
existing `v1.0` node stays valid). Entirely fixture-verified; the live corpus's byte-hash is
unchanged throughout. **What implementing it revealed:** (1) cloning the existing `Frontmatter`
and mutating only the touched fields is a stronger, more reusable "preserve everything else"
pattern than field-by-field reconstruction — worth defaulting to for any future node-update
operation (e.g. s06's supporting-doc work, if it ever needs to update rather than only create);
(2) `coverage.rs`'s heuristic arc-matching has a concrete, now-nameable gap — it still can't
resolve a **named** arc's assigned number (that depends on the full sorted list of named-arc
directories, not a single directory name), so a named arc's docs will keep reporting
"uncovered" even after `self_host` mints it a real node; disclosed in code and flagged as
explicit s06+ scope (wire the matcher against `source.paths` once that's the exact-match
mechanism ODD-0025 §5 anticipates), not silently left to be rediscovered as a `check` false
positive. MF-2/MF-3/MF-5 stay **planned** — the capability exists and is fixture-proven in full;
the live-corpus outcome each asserts is s05's job.

### v1.6 — 2026-07-27 — s04 restructured: cap killed; capability/live-run split; numbers = handles

Three operator decisions (in-session, superseding a past-CDC framing):

1. **The `MAX_MVP_ARC` / `arc_in_scope` cap is removed outright.** It was a hardcoded scope-lock
   that bought nothing, was the root cause of the 6-missing-arcs hole (design-notes: "scoped to the
   MVP arcs and never widened"), and directly contradicts the "no file left behind" property — a cap
   and that property cannot both be true. `self_host` now imports every arc/slice dir. (`arc_in_scope`
   is shared with `coverage.rs:512`, so removal also makes the detector stop treating those arcs as
   out-of-scope — more honest.)
2. **`number` is a non-structural human handle**, not identity (the ULID) and not order (the
   dependency DAG; 0013 §2.3). So named arcs (`arc-store-home`, `arc-release-hardening`,
   `arc-llm-command-surface`, `arc-migration-fidelity`) simply get an assigned handle by a
   deterministic rule — **no deferred "canonical numbering" decision is required** (an earlier CDC
   over-escalated this). *(Bonus, noted not scoped: s03's `source` field is a more principled
   idempotence key than `(type, number)` — a future refinement.)*
3. **s04 splits into capability-then-live-run.** s04 builds + fixture-proves the cap-removal +
   update-in-place repair + schema bump with **no** live mutation; a new **s05 — live repair run**
   fires it on the real corpus. Downstream renumbered: old s05/s06/s07 → **s06/s07/s08**. This also
   corrects the old s04-row's stale "delete bodyless nodes then re-migrate / a real `delete`
   capability" — ODD-0025 §2.8's update-in-place (via `persist` overwrite, preserving id/edges/status)
   needs no delete. Decisions logged in `design-notes.md` §3 (F11/F12).

### v1.5 — 2026-07-27 — s03 CDC-verified PASS; two findings ratified upward

**CDC verification: PASS** (`slice03-fidelity-core/cdc-verification.md`; 12/12 rows). Reproduced by
CDC on the live machine: the stub synthesis is gone, the live store is untouched (0 nodes carry
`source:`, still 60 nodes), the three fields are typed, `fidelity.rs` uses `trim+lf` with no stored
hash, and `de_opt_version` (the `1.0`→`"1"` fix) is sound and tested; cargo rows attested→CI. **The
F-3 correction was CDC's error, correctly caught** — the s03 ledger F-3 bundled `source` with
`author`/`version` as document-only, contradicting F-7; ODD-0025 §2.2 (`source` on *every migrated
node*) is right, and CC corrected toward it. Ratified CC's v1.4 bubble-up and carried up two findings:
**(a) ODD version-field hygiene** — bumped 0013→2.4 / 0020→v1.2 frontmatter + entry (corrected on disk
2026-07-27); **(b) branch deviation** — resolved by the fast-forward to `release/1.0.x`. Hardened the
schema-minor bump onto s04's row (now s04-capability, executed there).

### v1.4 — 2026-07-27 — s03 (fidelity core) closed; the faithful-import capability is fixture-proven

**Slice 03 is closed** (`slice03-fidelity-core/closing-report.md`; 12/12 ledger rows done, 1 with a
corrected criterion, 0 deferred, 0 no-op). Delivered ODD-0025's model as code: `odm-core` gained
typed `source`/`author`/`version` fields (round-trip + per-type validity); both importers now import
bodies **verbatim** — the stub-synthesis root cause is gone — behind a shared hard body-hash gate
(`odm-migrate::fidelity`), and populate the `source` record. ODD-0013 (v2.4) and ODD-0020 (v1.2)
carry the ODD-0025 §4 amendments. Fixture-verified; the live corpus untouched. Revealed: (1) the
F-3/F-7 internal inconsistency (see v1.5); (2) the `version: 1.0`→`"1"` YAML-float hazard, fixed via
`legacy::de_opt_version`; (3) the schema-minor bump is bigger than a rider — now owned by s04.
MF-2/MF-3 stay planned (live-corpus outcomes).

### v1.3 — 2026-07-27 — s02 (model) closed; ODD-0025 Accepted; provenance→source, author/version typed, F10 mint-all

**Slice 02 is closed** (14/14 rows; authored CDC-seat, operator-gated). Deliverable **ODD-0025**
(Accepted). Authoring revealed: (1) `provenance` was the wrong name (0013 reserves it for derived
lineage) → renamed **`source`** (§2.0 origin/source/provenance split); (2) `author`/`version` become
typed fields (operator corrected a proposed drop); (3) F10 resolved **mint-all**; (4) the
`note`-vs-`artifact` sub-fork → a new `artifact` type. MF-2/3/4/6/7 point at ODD-0025 as baseline.

### v1.2 — 2026-07-27 — s01 CDC-verified; report self-coverage escalated to F10

**CDC verification: PASS** (`slice01-coverage-discovery/cdc-verification.md`; 9 rows). F-5 (44 stubs),
F-6 (60 provenance-absent), the 60-node composition, and zero-`unsafe` independently re-derived by
CDC. Escalated **F10**: the arc's own generated reports would fail the enforced doc-coverage gate
forever unless dispositioned (resolved mint-all in v1.3).

### v1.1 — 2026-07-27 — s01 (coverage-discovery) closed; the arc's work-list is now exact

**Slice 01 is closed** (9/9 rows). Shipped the read-only `coverage` module + `odm migrate --coverage`;
live inventory **326 source docs, 266 uncovered, 6/12 arc dirs + 39/44 slice dirs represented, 44
stub bodies, 60/60 provenance-absent** — committed as the arc's authoritative work-list. Revealed the
chunk-artifact classification question, the three meanings of "research", and the self-referential
report finding — all fed s02.

### v1.0 — 2026-07-27 — arc shaped

Shaped from `reconciliation-audit-2026-07-27.md` + the migration/provenance design thread. Seven
slices, load-bearing order. F-forks resolved except F4 (a s02 detail). Unblocked by ODD-0024.
Release-blocking; subsumes L-8b.
