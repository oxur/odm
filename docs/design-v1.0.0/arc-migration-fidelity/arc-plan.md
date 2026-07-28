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
> **Status:** s01–s03 **closed** (2026-07-27); s03 **CDC-verified PASS**
> (`slice03-fidelity-core/cdc-verification.md`); **s04 (scope + repair) next**. *Plan late, plan
> deep* — each slice's open set (`slice-doc`/`ledger`/`cc-prompt`) is written when that slice becomes
> active, and its sizing is confirmed then (a slice that will not fit one context is two slices).

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
   silently uncovered; wiring it into `check` makes that class of hole loud, permanently.

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
| **s02 — model** ✅ CLOSED 2026-07-27 (ODD-0025 Accepted) | The model ODD: `source` sub-map (renamed from provenance, §2.0); `author`/`version` typed fields; `supersedes`→`Vec` + guaranteed bidirectional; the `artifact` node type; frontmatter-fidelity mapping; F4/F7/F10 resolved. → **ODD-0025**. | no | s01 |
| **s03 — migration-fidelity core** ✅ CLOSED 2026-07-27 (CDC-verified) | 1:1 verbatim body import + **hard body-hash gate** + **`source`/`author`/`version` typing & persistence** + **no-transform** (kill stub synthesis). Both importers (`mapping.rs`, `selfhost.rs`) + odm-core frontmatter typing (ODD-0025 §4 → 0013 §2.3, 0020). Delivered in one context (no split needed). | no (fixture-only) | s02 |
| **s04 — scope + repair** | Widen `self_host` past the `arc_in_scope` A1–A6 cap (all arcs/slices); **delete bodyless (stub) nodes, then re-migrate** (F8); re-point `context.json`; a real `delete` capability if `retire` won't serve. **+ own the ODD-0020 schema-minor bump** for `source`/`author`/`version` (s03 deferred it — see v1.5; **not deferrable past the live mint**, its own ledger row). | yes | s03 |
| **s05 — coverage enforcement** | Mint the supporting-doc child nodes as `artifact` nodes (`ledger`/`cc-prompt`/`cdc-verification`/`closing-report`/ADR/amendment/UAT + the reports, F10 mint-all); **wire doc-coverage into `odm check`**; attach or top-level the design/research nodes (F7). | yes | s02, s03, s04 |
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
| MF-2 | Body-hash gate green across all migrated nodes; zero stub bodies remain | re-migrate + gate; count stubs = 0 | serious | planned — s03 built + fixture-verified the gate (`slice03-fidelity-core/`, CDC-verified); "zero stubs" is a live-corpus outcome s04's re-migration produces |
| MF-3 | Every migrated node carries a `source` sub-map (+ preserved `author`/`version`) | grep/`check` over the store | correctness | planned — s03 typed the fields + both importers populate them, fixture-verified for all 4 source classes (CDC-verified); the live 60-node corpus gets them for real in s04 |
| MF-4 | Frontmatter-fidelity check green over originally-present fields | run the check | correctness | planned — mapping specified in ODD-0025 §2.4 |
| MF-5 | All arcs (incl. the 5 previously out-of-scope) + all slices represented | count dirs vs nodes = 0 gap | serious | planned — s01 counted the exact gap (6 arc dirs, 5 slice dirs unrepresented — the 6th arc is `arc-migration-fidelity` itself, shaped after the audit); s04/s05 mint |
| MF-6 | Supporting-doc children minted; doc-coverage wired into `odm check` | `check` fails on a seeded uncovered doc | serious (loud-hole guard) | planned — F10 **resolved mint-all** (ODD-0025 §2.6): every report incl. `coverage-report.md` gets an `artifact` node, no exemption; s05 mints + wires |
| MF-7 | Synthesis lands as supersede lineage; project vision re-cast; bidirectional guaranteed | inspect edges; seed + verify | correctness | planned — model in ODD-0025 §2.3 |
| MF-8 | L-8b: ODD-0013/0017/0018 (+0019/0020) reconciled to authoritative states | inspect states | pre-ship gate | planned |
| MF-9 | **Compose:** odm self-hosts *faithfully* — `check`/`orient`/`rollup` green on a corpus with real bodies, full coverage, source records | project-scale reproduce at arc close | serious (P-12) | planned |

## Version History

### v1.5 — 2026-07-27 — s03 CDC-verified PASS; two findings ratified upward

**CDC verification: PASS** (`slice03-fidelity-core/cdc-verification.md`; 12/12 rows). Reproduced by
CDC on the live machine: the stub synthesis is gone (`selfhost.rs`), the live store is untouched (0
nodes carry `source:`, still 60 nodes), the three fields are typed, `fidelity.rs` uses `trim+lf` with
no stored hash, and `de_opt_version` (the `1.0`→`"1"` fix) is sound and tested; cargo rows
attested→CI. **The F-3 correction was CDC's error, correctly caught** — my s03 ledger F-3 bundled
`source` with `author`/`version` as document-only, contradicting my own F-7; ODD-0025 §2.2's text
(`source` on *every migrated node*) is right, and CC corrected toward it. Ratifies CC's v1.4
bubble-up and carries up two findings it did not: **(a) ODD version-field hygiene** — the amendment
bumped the histories but not the docs' own SoT `version:` fields (0013 frontmatter still `2.3` vs a
`v2.4` entry; **0020's new entry is mis-numbered `v2.1`** where the sequence was v1.0→v1.1, so it
should be **v1.2**, and its frontmatter still says `1.1`) — minor, recommend correcting given this
arc is about version-as-SoT; **(b) branch deviation** — s01–s03 all landed on
`arc-migfidelity-slice01-coverage` rather than per-slice branches (disclosed) — an operator call on
whether to keep the running-branch pattern from s04 on. And it hardens finding (3) from v1.4: the
**ODD-0020 schema-minor bump is on s04's critical path** (s04's row updated) — the live mint stamps
nodes with the new fields, so an un-bumped marker would redefine the current schema version rather
than version the change.

### v1.4 — 2026-07-27 — s03 (fidelity core) closed; the faithful-import capability is fixture-proven

**Slice 03 is closed** (`slice03-fidelity-core/closing-report.md`; 12/12 ledger rows done, 1 with a
corrected criterion, 0 deferred, 0 no-op). Delivered ODD-0025's model as code: `odm-core` gained
typed `source`/`author`/`version` fields (round-trip + per-type validity); both importers
(`mapping.rs`, `selfhost.rs`) now import bodies **verbatim** — the stub-synthesis root cause
(`format!("# {name}\n")`) is gone — behind a shared, hard body-hash gate
(`odm-migrate::fidelity`), and populate the `source` record. ODD-0013 (v2.4) and ODD-0020 (v2.1)
carry the ODD-0025 §4 amendments. Entirely fixture-verified; the live `.worktrees/odm` corpus is
untouched (byte-identical throughout, matching s01's baseline). **What implementing it revealed:**
(1) **a real internal inconsistency between two ledger rows** — F-3's wording bundled
`author`/`version`/`source` as uniformly document-node-only, but F-7 requires `selfhost.rs` to
populate `source` on the work nodes it mints; ODD-0025 §2.2's own text (not its ledger summary)
resolves this — `source` is valid on every migrated node, work or document, only `author`/`version`
are document-only. Corrected in code and flagged, not silently worked around; **worth a standing
note for s05**, which will extend per-type validity again (`artifact`) and should check the ODD's
prose directly rather than a ledger row's compressed restatement. (2) **A real data-fidelity hazard**
in parsing the legacy `version:` field — YAML reads `version: 1.0` as a float, and Rust's default
float formatting silently drops the trailing zero (`1.0` → `"1"`); fixed via a dedicated
round-trip-preserving deserializer (`legacy::de_opt_version`), caught before it could corrupt a real
migration. (3) The ODD-0025 §4 schema-minor-bump ask is **recorded, not executed** — bumping
`SchemaMarker::current()` has its own workspace-wide blast radius on the "unsupported-schema" check
contract, deliberately left for a dedicated follow rather than riding along on this slice's field
additions. MF-2/MF-3 stay **planned** — the capability exists and is fixture-proven; "zero stubs" /
"every node has one" are live-corpus outcomes s04 produces.

### v1.3 — 2026-07-27 — s02 (model) closed; ODD-0025 Accepted; provenance→source, author/version typed, F10 mint-all

**Slice 02 is closed** (`slice02-fidelity-model/closing-report.md`; 14/14 ledger rows done;
authored CDC-seat, operator-gated — Duncan confirmed every decision 2026-07-27). Deliverable:
**ODD-0025 — Migration Fidelity** (Accepted, `docs/design/04-accepted/`). **What authoring revealed
(slice → arc feedback):** (1) `provenance` was the wrong name — 0013 reserves it for *derived*
lineage ("never a stored scalar"); the stored migration record is renamed **`source`** (ODD-0025
§2.0's origin/source/provenance split), rewording Capability property 2, the Source-record exit
criterion, and MF-3 above; (2) **`author`/`version` become new typed document-node fields** — the
operator corrected a proposed drop (author because git returns the migrator, not the source author;
version because it is SoT quick-access) — which **expands s03's scope** to type three new frontmatter
fields + amend 0013 §2.3 (s03 row updated, with a split flag); (3) **F10 resolved `mint-all`** —
every report incl. `coverage-report.md` gets an `artifact` node, no exemption (MF-6 note updated);
(4) the `note`-vs-`artifact` sub-fork surfaced (0013 already has a `note` type) and resolved to a new
`artifact` type. MF-2/3/4/6/7 stay **planned**, pointing at ODD-0025 as their design baseline.

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
