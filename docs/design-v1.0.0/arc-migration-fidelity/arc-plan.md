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
> the fidelity checks amend); `arc-migration-fidelity/design-notes.md` (the decision log this
> plan draws on — the slice-02 model ODD is written from it).
>
> **Status:** s01 (coverage-discovery) **closed & CDC-verified** (2026-07-27); **s02 (model) next**.
> *Plan late, plan deep* — each slice's open set (`slice-doc`/`ledger`/`cc-prompt`) is written when
> that slice becomes active, and its sizing is confirmed then (a slice that will not fit one context
> is two slices).

## Capability

Give odm a **complete, general, verifiable migration capability** — one that brings *every*
documentation file into the store faithfully, proves it did, and can be re-run across many
projects. Four properties define "faithful and verifiable," each enforced by a check rather than
trusted:

1. **1:1 verbatim bodies, hard-gated.** A migrated node's body **is** its source body. Migration
   hashes `trim(source_body)` and `trim(node_body)`; a mismatch is a **hard error** that fails the
   migration. The importer performs **no body transformation** (no synthesised `# {name}` H1, no
   header injection) — the old transform behaviour is the thing that produced the 44 stubs.
2. **Provenance on every node.** A `provenance` sub-map records `source_paths` (a list),
   `source_class`, `normalization`, and the migrating tool + version. Computed at migration time;
   no hashes stored (content is allowed to change — Version-History sections do).
3. **Frontmatter fidelity, schema-mapped.** The fields the source actually had are checked to map
   correctly onto the node's fields (e.g. legacy `state: Final` → the cumulative gate reach),
   over a versioned schema mapping. Only originally-present fields are checked.
4. **No file left behind, enforced.** A doc-coverage check — the inverse of `orphan` — asserts
   every `.md` in the migration source has a node. Its absence is *why* ~211 supporting docs went
   silently uncovered; wiring it into `check` makes that class of hole loud, permanently.

Synthesis (merging several docs into one) is **not** migration: it is a separate, later step that
mints a **new** node **superseding** its sources (`edges.supersedes` → a `Vec`, with the
reverse `superseded_by` derived and **tooling-guaranteed** on every synthesis). Concatenation
stays hash-gated; editorial/conceptual merges are verified by lineage + attestation.

Applied first to odm's own `1.0.x` corpus — taking self-hosting from skeleton (44 stub bodies, 6
of 11 arcs, ~211 uncovered docs, 0 provenance) to **100%**. Built general so the next project runs
the same capability.

## Exit criteria (arc acceptance — the composition check)

- **Coverage:** every `.md` under `1.0.x/docs/*` maps to a node; the doc-coverage check is green
  (no file left behind). Design/research nodes may be top-level *or* contained — both valid (F7).
  The arc's **own** report/verification artifacts are dispositioned per F10 (a node class,
  exemption, or ignore rule) so the enforced check does not flag them in perpetuity.
- **Body fidelity:** every migrated node's `trim(body)` hash equals its source's; the hard gate is
  green across the whole corpus; **no stub bodies remain**.
- **Provenance:** every migrated node carries a `provenance` sub-map.
- **Representation:** all real arcs (the 5 missing + A1–A6) and all slices are represented; scope
  is no longer capped at `arc_in_scope` A1–A6; supporting-doc children are minted.
- **Frontmatter fidelity:** the schema-mapping check is green over originally-present fields.
- **L-8b cleared:** ODD-0013/0017/0018 (+ the 0019/0020 amendments) reconciled to states that
  reflect their authority.
- **Reflexive:** `odm check` green; `orient`/`rollup` reproduce the hand-maintained truth; **P-12
  (odm self-hosts) is satisfiable at project close** against a *faithful* corpus, not a skeleton.

## Slice breakdown (one-line altitude; load-bearing order)

| Slice | Scope | Mints nodes? | Depends on |
|-------|-------|--------------|------------|
| **s01 — coverage-discovery** ✅ CLOSED 2026-07-27 (CDC-verified) | Build the read-only doc-coverage detector (inverse `orphan`) + sibling detectors (representation, body-fidelity, provenance-absence); run over all ~320 `1.0.x/docs` files → the **exact gap inventory** (the work-list). | no (read-only) | — |
| **s02 — model** | The ODD(s): `provenance` sub-map; `supersedes`→`Vec` + guaranteed bidirectional; supporting-doc **artifact-vs-work** node class; frontmatter-fidelity schema mapping; F4 normalization (trim + line-endings); F7 containment-optional for doc nodes; **F10 report-artifact coverage disposition** (a node class / exemption / ignore rule for the arc's own `coverage-report`/`closing-report`/`cdc-verification` files, so the s05 enforced check does not flag them forever — surfaced by s01, see v1.2). | no | s01 |
| **s03 — migration-fidelity core** | 1:1 verbatim body import + **hard body-hash gate** + provenance persistence + **no-transform** (kill stub synthesis). Both importers (`mapping.rs`, `selfhost.rs`). | building | s02 |
| **s04 — scope + repair** | Widen `self_host` past the `arc_in_scope` A1–A6 cap (all arcs/slices); **delete bodyless (stub) nodes, then re-migrate** (F8); re-point `context.json`; a real `delete` capability if `retire` won't serve. | yes | s03 |
| **s05 — coverage enforcement** | Mint the supporting-doc child nodes (`ledger`/`cc-prompt`/`cdc-verification`/`closing-report`/ADR/amendment/UAT); **wire doc-coverage into `odm check`**; attach or top-level the design/research nodes (F7). | yes | s02, s03, s04 |
| **s06 — synthesis + L-8b** | The supersede-based synthesis step (concat hash-gated; editorial-merge attested); re-cast the project vision as a synthesis **superseding** the 1:1 `project-plan` node; reconcile the four L-8b ODDs to authoritative states. | yes | s02, s03 |
| **s07 — reconcile run** | Run the whole capability over odm's own corpus end-to-end; verify no doc uncovered, no orphan, every body-hash passes, all arcs/slices present, `check`/`orient`/`rollup` green. The arc composition + P-12 acceptance demonstration. | — | all |

*Sizing note:* s03 and s05 are the heaviest; if either fails the one-context test when its open
set is drawn, it splits (fidelity-core vs. scope-repair; children-mint vs. check-wiring) — decided
at slice-activation, not pre-committed here.

## Arc ledger (composition rows — opens here, closes in `closing-report.md`)

> LEDGER-DISCIPLINE v2.0 §B. Class-(b) rows are **reproduced at arc scale** (an end-to-end run on
> odm's corpus). Status ladder: `asserted < attested < reproduced < reconciled`; a `done` row
> reaches ≥ `reproduced`. All rows open **planned**.

| ID | Criterion | Verify | Significance | Status |
|----|-----------|--------|--------------|--------|
| MF-1 | Doc-coverage check exists and is green on `1.0.x/docs` — every `.md` has a node | run the check on the corpus | serious (no file left behind) | planned — the s01 *detector* exists and ran (`slice01-coverage-discovery/coverage-report.md`, CDC-verified); the *enforced check* MF-1 asks for is s05's job |
| MF-2 | Body-hash gate green across all migrated nodes; zero stub bodies remain | re-migrate + gate; count stubs = 0 | serious | planned — s01 counted the stubs exactly (44, CDC-reproduced); s03 builds the gate |
| MF-3 | Every migrated node carries a `provenance` sub-map | grep/`check` over the store | correctness | planned — s01's provenance-absence detector confirms the baseline (0/60, CDC-reproduced); s02/s03 land the field |
| MF-4 | Frontmatter-fidelity check green over originally-present fields | run the check | correctness | planned |
| MF-5 | All arcs (incl. the 5 previously out-of-scope) + all slices represented | count dirs vs nodes = 0 gap | serious | planned — s01 counted the exact gap (6 arc dirs, 5 slice dirs unrepresented — the 6th arc is `arc-migration-fidelity` itself, shaped after the audit); s04/s05 mint |
| MF-6 | Supporting-doc children minted; doc-coverage wired into `odm check` | `check` fails on a seeded uncovered doc | serious (loud-hole guard) | planned — see F10 (v1.2): the check's design must disposition the arc's own report/verification artifacts, or it flags them forever |
| MF-7 | Synthesis lands as supersede lineage; project vision re-cast; bidirectional guaranteed | inspect edges; seed + verify | correctness | planned |
| MF-8 | L-8b: ODD-0013/0017/0018 (+0019/0020) reconciled to authoritative states | inspect states | pre-ship gate | planned |
| MF-9 | **Compose:** odm self-hosts *faithfully* — `check`/`orient`/`rollup` green on a corpus with real bodies, full coverage, provenance | project-scale reproduce at arc close | serious (P-12) | planned |

## Version History

### v1.2 — 2026-07-27 — s01 CDC-verified; report-artifact self-coverage escalated to an s02 fork (F10)

**CDC verification: PASS** (`slice01-coverage-discovery/cdc-verification.md`). All 9 ledger rows
carried to `reproduced` or a justified `attested`→CI; **F-5 (44 stubs), F-6 (60 provenance-absent),
the 60-node composition, and F-9's zero-`unsafe` were independently re-derived by CDC** against the
live corpus, not read from CC's summary. The v1.1 bubble-up (CC) stands; this entry ratifies it and
carries forward the one thing CC's flag (3) named as "correct, not a bug" but did not push into
s02's scope. **The fork (F10):** because `odm migrate --coverage` renders to stdout and the operator
redirects into `coverage-report.md` under `design-v1.0.0/`, every regeneration adds a
*permanently-uncovered* `.md` — the report plus this arc's own `closing-report`/`cdc-verification`
artifacts (the 326→328 source-count drift observed at verification is the first instance). When s05
wires doc-coverage into `odm check` (MF-6), **the arc's own reports will fail the enforced gate
forever unless s02's model decides their disposition** — a report/verification node class, an
explicit coverage exemption, or an ignore rule. Recorded in the s02 scope row, the Coverage exit
criterion, and MF-6 above. (Also belongs in `design-notes.md` §3 when s02's open set is drawn.)

### v1.1 — 2026-07-27 — s01 (coverage-discovery) closed; the arc's work-list is now exact

**Slice 01 is closed** (`slice01-coverage-discovery/closing-report.md`; 9/9 ledger rows done, 0
deferred, 0 no-op). It shipped a read-only `coverage` module + `odm migrate <docs> --coverage`
and ran it against the live corpus: **326 source docs, 60 covered, 266 uncovered; 6/12 arc dirs
+ 39/44 slice dirs represented; 44 stub bodies; 60/60 nodes missing provenance** — committed as
`slice01-coverage-discovery/coverage-report.md`, now the arc's authoritative work-list (every
divergence from the audit's ≈44/≈211/5 ballpark reconciled by name in that report, not rounded
off). **Which child surfaced it, and what it revealed:** (1) a real classification-boundary
question for s02's model — chunk-scale artifacts (`cN-closing-report.md` etc.) fold into their
slice-scale sibling class here, but s02's node-class design must decide this for real, not
inherit the choice silently; (2) `docs/dev/research/` is a third, distinct thing sharing the
word "research" with the ODD `research` tag and with `docs/dev/` generally — worth a
disambiguating line in s02's model doc; (3) the slice's own open-set files correctly appear as
uncovered/unrepresented in its own report (self-referential, not a bug). MF-1/MF-3/MF-5 stay
**planned** (the detector exists; the enforced check / minting / provenance field are s02–s05's
jobs) with pointers to this closing report as their baseline evidence.

### v1.0 — 2026-07-27 — arc shaped

Shaped from the `reconciliation-audit-2026-07-27.md` discovery and the migration/provenance
design thread (`design-notes.md`). Seven slices, load-bearing order s01→s07. All F-forks resolved
except F4 (a s02 detail): migration = 1:1 verbatim + hard body-hash gate; provenance sub-map
(computed, unstored); synthesis = separate supersede step; F8 = delete-bodyless-then-re-migrate;
F7 = containment-optional. Unblocked by ODD-0024 (G-1 closed). Release-blocking; subsumes L-8b.
