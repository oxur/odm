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
> **Status:** s01–s07 **closed** (2026-07-28); s04–s06 CDC-verified PASS, s07 attested-by-CC pending
> CDC. **The arc's first live mutation landed** (`odm` branch commit `7b4eb57`): odm's own corpus is
> now 0 stub bodies / 61 source-bearing plan nodes / all 12 arcs represented (was 44 stubs / 0 source
> / 6 of 12). MF-2/MF-3/MF-5 move to done. **s08 (coverage enforcement) next.** *Plan late, plan deep.*

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
| **s04 — scope + repair capability** ✅ CLOSED 2026-07-28 (CDC-verified) | **Cap removed** (`self_host` imports **all** arc/slice dirs; named arcs get a non-structural `number` handle; `coverage.rs`'s shared predicate updated). **Update-in-place repair** of stub nodes (ODD-0025 §2.8 — real body + `source`, preserve id/edges/status, via `persist` overwrite — no `delete`). **Schema-minor bump executed** (`v1.0→v1.1`, forward-compat proven). Fixture-verified; no live mutation. | no (fixture-only) | s03 |
| **s05 — source-based identity** ✅ CLOSED 2026-07-28 (CDC-verified) | Retire `number` as a **correctness key** (F12 — the number problem's root): key `self_host` idempotence + coverage matching on **`source.paths`**, not `(type, number)`; **backfill `source`** onto the already-faithful non-stub nodes (body unchanged, hash-gate-confirmed) so *every* node carries one; make the named-arc handle **name-derived + stable** (cosmetic display only). `number` becomes a pure label nothing keys on — the position-based fragility dissolves permanently. Fixture-verified. | no (fixture-only) | s04 |
| **s06 — live-run capability** ✅ CLOSED 2026-07-28 (attested-by-CC; CDC pending) | **Unified the two `source`-backfill paths** (v2.1 finding): a single `reconcile_source()` — gated, project-excluding, and (self-identified extension) retired-excluding — now backs both `self_host`'s `to_populate` transition and `repair()`. **Extended `odm migrate`'s self-host path**: `repair()` now runs before `self_host()` in `self_host_inner`, default-on, no new flag; `repair()`'s only callers were previously tests. **No new verb** (`self-host` folded into `migrate`, C-5); reuses `--dry-run`. **Fixture-verified end-to-end; no live mutation.** `context.json` re-pointing stays with s07 (structurally a live-store operator statement, not a migration artifact — disclosed in the closing report). | no (fixture-only) | s05 |
| **s07 — live repair run** ✅ CLOSED 2026-07-28 (attested-by-CC; CDC pending) | Fired the s06 flow on the **live** `.worktrees/odm` corpus behind the full snapshot → dry-run → adjudicate → fire → verify protocol: commit `7b4eb57` (`odm` branch, atop known-good `e2ab628`) repairs the 44 stubs, gated-backfills `source` on the faithful nodes (none existed beyond the stubs — the corpus's one non-stub slice was the retired tombstone), imports the 6 previously-excluded arcs + their 11 slices, stamps `v1.1`. **Opened with** the CDC v2.5 doc-comment fix (`release/1.0.x` commit `b901b12`). **`context.json` deliberately left unchanged** — `migrate` has no code path that writes it; re-pointing operator focus is a separate act this slice doesn't make on the operator's behalf. **Verified:** `check` green, 0 stubs, 0 uncovered arc-plans/slice-docs, 0 `BodyHashMismatch` on re-verify, `orient`/`rollup` byte-stable, fully re-run-idempotent, project + retired node confirmed untouched. **Four pre-existing findings disclosed** (not fixed here): absolute `source.paths`, a report-clarity gap in `representation()`'s named-arc heuristic, a stale `provenance_absence` detector (checks the pre-s02 key name), and 8 genuine (non-blocking) `check` warnings about arcs with no slice subdirectories yet. **Deferred to s08 as scoped:** the 14 design/research nodes' `source` + the ~211 loose-doc coverage. | **yes (live)** | s06 |
| **s08 — coverage enforcement** | Mint the supporting-doc child nodes as `artifact` nodes (F10 mint-all incl. the reports); **wire doc-coverage into `odm check`**; attach or top-level the design/research nodes (F7). | yes | s02, s03, s07 |
| **s09 — synthesis + L-8b** | The supersede-based synthesis step (concat hash-gated; editorial-merge attested); re-cast the project vision as a synthesis **superseding** the 1:1 `project-plan` node; reconcile the four L-8b ODDs. | yes | s02, s03 |
| **s10 — reconcile run** | Run the whole capability over odm's own corpus end-to-end; verify no doc uncovered, no orphan, every body-hash passes, all arcs/slices present, `check`/`orient`/`rollup` green. The arc composition + P-12 acceptance demonstration. | — | all |

*Sizing note:* the heaviest are **s05**/**s06** (identity + live run) and **s07** (children-mint + check-wiring); the live-mutation slices are
**s06**/**s07** — split at slice-activation if an open set won't fit
one context. s04 was split from its live run (s05) at activation (2026-07-27) precisely so the
destructive op is fixture-proven before it fires.

## Arc ledger (composition rows — opens here, closes in `closing-report.md`)

> LEDGER-DISCIPLINE v2.0 §B. Class-(b) rows are **reproduced at arc scale** (an end-to-end run on
> odm's corpus). Status ladder: `asserted < attested < reproduced < reconciled`; a `done` row
> reaches ≥ `reproduced`. All rows open **planned**.

| ID | Criterion | Verify | Significance | Status |
|----|-----------|--------|--------------|--------|
| MF-1 | Doc-coverage check exists and is green on `1.0.x/docs` — every `.md` has a node | run the check on the corpus | serious (no file left behind) | planned — s01 *detector* exists + ran (CDC-verified); the *enforced check* is s08's job |
| MF-2 | Body-hash gate green across all migrated nodes; zero stub bodies remain | re-migrate + gate; count stubs = 0 | serious | **done** — s07's live run: 0 stub bodies on the committed `odm` corpus (was 44); a second `migrate` run surfaces 0 `BodyHashMismatch` (`slice07-live-run/closing-report.md`); reproduced by direct read of commit `7b4eb57` |
| MF-3 | Every migrated node carries a `source` sub-map (+ preserved `author`/`version`) | grep/`check` over the store | correctness | **done for the plan-node scope** (project/arc/slice) — s07's live run: 61 of 61 eligible live nodes source-bearing (every arc/slice node; project + the retired tombstone correctly excluded by design, ODD-0025 §2.3); `author`/`version` N/A (frontmatter-less plan corpus) (`slice07-live-run/closing-report.md`); reproduced by direct read of commit `7b4eb57`. **Not yet done for the full criterion**: the 14 design/research nodes still carry no `source` — `discover()` structurally cannot reach that type family, so their backfill is explicitly **s08**'s job, not a silent gap |
| MF-4 | Frontmatter-fidelity check green over originally-present fields | run the check | correctness | planned — mapping specified in ODD-0025 §2.4 |
| MF-5 | All arcs (incl. the 6 previously-excluded) + all slices represented | count dirs vs nodes = 0 gap | serious | **done** — s07's live run: all 12 plan-tree arcs + their slices now have nodes; doc-coverage set-difference over `source.paths` = 0 uncovered arc-plans/slice-docs (`slice07-live-run/closing-report.md`); reproduced by direct read of commit `7b4eb57`. **Caveat disclosed, not a gap:** `coverage.rs`'s separate `representation()` heuristic still can't resolve a named arc's number from its directory name alone, so `--coverage`'s summary line still reads "8/12" even though the exact criterion (doc-coverage) is 12/12 — a report-clarity finding for a future coverage-rendering pass, not an unrepresented arc |
| MF-6 | Supporting-doc children minted; doc-coverage wired into `odm check` | `check` fails on a seeded uncovered doc | serious (loud-hole guard) | planned — F10 **mint-all** (ODD-0025 §2.6); s08 mints + wires |
| MF-7 | Synthesis lands as supersede lineage; project vision re-cast; bidirectional guaranteed | inspect edges; seed + verify | correctness | planned — model in ODD-0025 §2.3 |
| MF-8 | L-8b: ODD-0013/0017/0018 (+0019/0020) reconciled to authoritative states | inspect states | pre-ship gate | planned |
| MF-9 | **Compose:** odm self-hosts *faithfully* — `check`/`orient`/`rollup` green on a corpus with real bodies, full coverage, source records | project-scale reproduce at arc close | serious (P-12) | planned |

## Version History

### v2.7 — 2026-07-28 — s07 (live repair run) closed; the arc's first live mutation landed

**Slice 07 is closed** (`slice07-live-run/closing-report.md`; 11/11 ledger rows done, 0 deferred, 0
no-op; attested-by-CC, CDC reproduction pending — this is the arc's first live, class-(b) row, so CDC
reproduces by **direct read of the committed store**, not by re-running fixtures). Opened with the CDC
v2.5 doc-comment fix (`release/1.0.x` `b901b12`); captured a before-manifest (60 nodes, 0 source, all
`v1.0`, 1 retired, sha256 fingerprint, known-good SHA `e2ab628`); dry-ran and adjudicated (44
reconciled / 17 created / 46 skipped, exact against the corpus once traced by hand — the slice-doc's
"≈47" was a slightly-loose estimate, not a discrepancy); fired for real and committed the entire
`.worktrees/odm` worktree as **one** commit (`7b4eb57`, atop `e2ab628`, revert = `git reset --hard
e2ab628…`). **Verified on the committed store:** 0 stub bodies (was 44), 61 source-bearing plan nodes
(was 0), all 12 plan-tree arcs represented (0 uncovered arc-plans/slice-docs), 0 `BodyHashMismatch` on
re-verify, `check` exit 0, `orient`/`rollup` byte-stable, fully re-run-idempotent, project and the
retired tombstone both confirmed byte-for-byte untouched. `context.json` deliberately left unchanged
(`migrate` has no code path that writes it). **No gate failed; no rollback was needed.**

**What the live run revealed (four disclosed, non-blocking findings, not fixed here):** (1)
`source.paths` stores absolute, machine-specific paths (`odm-cli/src/migrate.rs::resolve()`,
pre-existing since s01/s03 — fixtures never surfaced it since `TempDir` paths are absolute too, just
ephemeral); (2) `representation()`'s named-arc heuristic gap (disclosed at s05) now visibly produces a
confusing "8/12" in `--coverage`'s summary line even though the exact doc-coverage criterion is 12/12
— a report-clarity gap, not a real one; (3) the `provenance_absence` detector still checks for a
literal `provenance:` key, which ODD-0025 §2.0 renamed to `source:` at s02 — it now reports all 77
nodes "provenance absent" forever, directly contradicting the correct "61 source-bearing" figure one
line above it in the same report; (4) `check`'s 8 new warnings are genuine, pre-existing
plan-decomposition facts (4 newly-visible named arcs genuinely have no slice subdirectories yet in the
plan tree) made visible for the first time now that those arcs have nodes — not migration defects.
Findings 1–3 are recommended follow-ups for whoever next touches `coverage.rs`'s rendering or
`resolve()` (s08 is the natural home for 2–3, since it already touches coverage). **MF-2, MF-3
(plan-node scope), and MF-5 move `planned → done`** — reproduced live, per LEDGER-DISCIPLINE v2.0 §B;
MF-3's design/research-node scope, MF-1, and MF-6 stay **planned**, explicitly handed to **s08**.

### v2.6 — 2026-07-28 — s07 (live repair run) drawn; scope refined to plan nodes, design/research → s08

CDC drew the s07 open set (`slice07-live-run/{slice-doc,ledger,cc-prompt}.md`) and ground-truthed the
live store to pin the run: 60 nodes / **0 source** / all `schema: */v1.0` / **44 stubs (6 arc + 38
slice** — every present arc is a stub) / 6 arcs / `context.json → 01KWXM…`; plan tree has **12** arcs
(6 to import) and 49 slice-docs. **Scoping refinement (folded into the s07 row above):** `discover()`
structurally produces only project/arc/slice PlanNodes, and `validate` does **not** require `source`
(confirmed in `odm-core/src/check.rs` — only `author`/`version` are type-restricted on work nodes), so
s07's `source`/stub outcome is **arc/slice-scoped** and the 14 design/research nodes + ~211 loose docs
stay source-less **without failing `check`** — their `source` + coverage-into-`check` is **s08**. The
plan's earlier "every node source-bearing" / "populate author/version" framing corrected accordingly
(author/version N/A on the frontmatter-less plan corpus). No scope *loss* — a precision fix at draw so
CC runs a live mutation against an accurate target. s07 is the class-(b) slice where **MF-2/MF-3/MF-5
reproduce at arc scale** (§B) — they flip `planned → done` only at s07 close, on the committed store.

### v2.5 — 2026-07-28 — s06 CDC-verified PASS; doc-only comment finding carried onto s07

**CDC verification: PASS** (`slice06-live-run-capability/cdc-verification.md`; 10/10 rows). Reproduced
by close code-read on the live machine: `reconcile_source()` is the one gated, project- **and**
retired-excluding policy both `repair()` and `self_host`'s transition route through (no second copy to
drift); the gate runs even under `--dry-run`; `self_host_inner` runs `repair()` then `self_host()`,
default-on; no `unsafe`; live store not checked out. Cargo-run behaviors (drift/dry-run/idempotence,
clippy, coverage) attested → CI. **One CDC finding, doc-only (fails no row):** `self_host_inner`'s doc
comment calls the repair→import order a load-bearing correctness invariant ("reordering would
reintroduce exactly the gap s06 closed"), but CC's own closing-report F-3 disproves it — the Preferred
unification makes `self_host()` alone reconcile a coordinate-matched node identically, so the order is
for composability/report-attribution, not per-node correctness (which rests on `reconcile_source`,
exercised both ways). Carried onto s07 as a one-line comment correction at its opening, not a reopen
of s06. s06 flips to **CDC-verified PASS**; MF-2/MF-3/MF-5 stay **planned** (their live-corpus outcome
is s07's, per §B).

### v2.4 — 2026-07-28 — s06 (live-run capability) closed; both CDC findings resolved

**Slice 06 is closed** (`slice06-live-run-capability/closing-report.md`; 10/10 ledger rows done, 0
deferred, 0 no-op; attested-by-CC, CDC reproduction pending). Delivered exactly what v2.1–v2.3 scoped:
a single `reconcile_source()` is now the **one** gated, project-excluding `source`-population policy,
backing both `repair()` and `self_host`'s coordinate→source transition (closing the v2.1 finding at
its root, not just at the one call site CDC named); `odm migrate`'s self-host path now runs `repair()`
**then** `self_host()` by default, no new flag, folding the reconcile count into the existing report
shape (closing the v2.2 finding — `repair()` now has a real caller). **Self-identified extension
beyond the literal CDC finding:** the same unification pass found `self_host`'s transition carried no
guard against a **retired** node either (a tombstone sharing a coordinate with a live plan directory)
— `repair()` had this by accident of its own pre-check; `reconcile_source` now excludes retired nodes
for both callers, closing the same *class* of gap the CDC finding named, not just its one instance.
**What implementing it revealed:** (1) given the "Preferred" unification choice, `self_host()` alone
(no `repair()` call) now reconciles a coordinate-matched node identically to `repair()`-then-
`self_host()` — the specified order is implemented exactly as directed and is the right shape for API
composability (`repair()` needed a real caller), but it is not, in this implementation, load-bearing
for *per-node correctness* the way the plan's framing assumed; disclosed rather than silently
smoothed over. (2) `context.json` re-pointing, mentioned in `slice-doc.md`'s prose but absent from
`cc-prompt.md`'s Task list and structurally a live-store operator statement rather than a migration
artifact, stayed with **s07** as the slice-breakdown table already specified — followed the operative
`cc-prompt.md` over the summary prose, flagged the inconsistency rather than silently picking one.
MF-2/MF-3/MF-5 stay **planned**, now pointing at `slice04`/`slice05`/`slice06`'s closing reports as
fixture-verified baseline evidence — the live-corpus outcome each asserts is **s07**'s job.

### v2.3 — 2026-07-28 — s06 entry point corrected: extend `odm migrate`, not a new command

Drawing the s06 open set (CDC), the command inventory (`arc-llm-command-surface/`) settled the CLI
question v2.2 left open: `self-host` was **removed** as a spelling (C-4) and **folded into `migrate`**
(C-5) — "one verb." So s06 does **not** add a command; it **extends `odm migrate`'s self-host path**
(`self_host_inner`, `odm-cli/src/migrate.rs`) to run `repair()` before `self_host()`, both already
`Mode`/`--dry-run`-aware. Confirmed against the code: `odm migrate` dispatches to `self_host()` only;
`repair()`'s sole callers are tests. slice-doc / ledger / cc-prompt drawn to match. No scope change —
a naming/shape correction folded in at draw so CC doesn't build against a stale "add a command" framing.

### v2.2 — 2026-07-28 — s06 split into live-run capability + live run (s07); repair had no CLI entry

Drawing s06 surfaced that the live run has **no way to be invoked**: `selfhost::repair` (built s04,
generalized s05) is a library function with **no CLI command** — the only wired migration entry is
`odm migrate <plan>` (which runs `self_host`, not `repair`). So s06 becomes the **live-run
capability**: (a) the v2.1 two-path fix (route `self_host`'s `to_populate` transition through the same
gated, project-excluding logic as `repair()`), and (b) **wire the full repair/reconcile flow into an
invocable, `--dry-run`-able CLI command** — both fixture-verified, no live mutation. The actual live
mutation is a new **s07 — live repair run**, deliberately its own slice so the capability is
CDC-verified before anything touches the real store (the "build then run" gate, applied to the arc's
highest-stakes action). Downstream renumbered: old s07/s08/s09 → **s08/s09/s10**. Not needless
deferral — the op literally cannot be run until it is wired.

### v2.1 — 2026-07-28 — s05 CDC-verified PASS; two-path source-backfill finding onto s06

**CDC verification: PASS** (`slice05-source-identity/cdc-verification.md`; 12/12 rows). Reproduced by
CDC: idempotence keys on `source.paths` (primary) with the `by_coordinate` one-time transition; the
named-arc handle is name-derived (FNV-1a slug hash, deterministic collision-probe, recomputable);
coverage matches `source.paths` exact; `number` keys nothing for correctness; the live store is
untouched (0 `source:`, 60 nodes, all `v1.0`). `repair()`'s unification of stub-repair + faithful
backfill under **one gated path** is endorsed, and CC's F-5 test drives the drift branch with two
independent bodies (better than s03/s04's primitive-only). **One CDC finding, carried onto s06:** a
*second* `source`-backfill path exists — `self_host`'s coordinate→source transition (`to_populate`)
adds `source` **ungated** and **without the project exclusion** `repair()` enforces. Invisible in
s05's fixtures, but at the live run it would stamp a 1:1 `source` onto the **synthesis project node**
(the exact case F-6 prevents, via a path F-6 doesn't cover) and could stamp `source` over a drifted
body without surfacing it. A spec gap the CDC slice-doc didn't anticipate (it fails no s05 row).
**s06 must unify/guard the two paths before the live mutation** (s06 row updated). MF-2/3/5 stay
planned.

### v2.0 — 2026-07-28 — s05 (source-based identity) closed; number retired as a correctness key

**Slice 05 is closed** (`slice05-source-identity/closing-report.md`; 12/12 ledger rows done, 0
deferred, 0 no-op; attested-by-CC, CDC reproduction pending). Delivered exactly what v1.9 scoped:
`self_host` idempotence now keys on **`source.paths`**, with a one-time coordinate→source transition
that backfills `source` onto a pre-slice05 legacy node in place; the s04 v1.8 named-arc re-run-
duplicate hazard is now structurally impossible (source-matching alone would prevent it, and the
new name-derived, collision-handled `named_arc_number(slug, taken)` additionally removes the number
*shift* that caused it); `repair` is generalized past the stub filter — a faithful non-stub node
lacking `source` gets one as a body-unchanged no-op, a drifted one surfaces `BodyHashMismatch` rather
than being silently backfilled, and the project node stays excluded (its body is a synthesis, ODD-0025
§2.3); `coverage.rs`'s doc-coverage matcher gained an exact `source.paths` check as primary, closing
CC's s04-disclosed named-arc coverage gap without any model change. **What implementing it revealed:**
(1) the stub-repair and non-stub-backfill cases collapse into one code path under one hash-gate call,
not two branches with separate error handling — computing the candidate body first (source text for a
stub, the existing body otherwise) and always gating it against the source is simpler *and* stronger
than a two-policy design; (2) the F-7 coverage-matching piece needed no split-escape to s07 as
v1.9 reserved — it turned out cheap once F-1 was in place, since every node already carried
`source.paths` from s03 onward, so the fix was in the matcher, not the model; (3) a genuinely
vanishing edge case is named for the record: `named_arc_number`'s collision-bump is deterministic
given a fixed slug set in a fixed processing order, but a *new* slug that collides with an
already-claimed slot could in principle still shift a later arc's number depending on insertion
order — not a live risk, and source-matching absorbs it regardless, but worth a one-line explanation
if a live run ever shows two cosmetic numbers disagreeing across runs. MF-2/MF-3/MF-5 stay
**planned**, now pointing at both `slice04-scope-repair-capability/closing-report.md` and
`slice05-source-identity/closing-report.md` as fixture-verified baseline evidence — the live-corpus
outcome each asserts is s06's job.

### v1.9 — 2026-07-28 — source-based identity inserted as s05 (retire number-as-a-correctness-key); live run → s06

Operator decision (2026-07-28): **retire `number` as a correctness key now, before any live work** —
the number kept causing problems precisely because a *derived* value was doing an *identity* job. A
new **s05 — source-based identity** is inserted ahead of the live run: `self_host` idempotence +
coverage matching key on **`source.paths`** (the stable identity s03 gave every node), not
`(type, number)`; the already-faithful non-stub nodes get a one-time **`source` backfill** (body
unchanged, hash-gate-confirmed) so *every* node carries one; and the named-arc handle becomes
**name-derived + stable**. Once `source` is the key, `number` is a pure display label nothing keys on,
so the position-based handle fragility (v1.8 finding) **dissolves permanently** rather than being
patched. Resolves the v1.8 pre-mint requirement at its root and promotes design-notes F12 from
"future refinement" to **done-now**. Downstream renumbered: old s05/s06/s07/s08 → **s06/s07/s08/s09**.
*(Should have been done when F12 first surfaced; deferring it is what let the number keep biting —
recorded so the lesson sticks.)*

### v1.8 — 2026-07-28 — s04 CDC-verified PASS; named-arc handle scheme flagged onto s05

**CDC verification: PASS** (`slice04-scope-repair-capability/cdc-verification.md`; 12/12 rows).
Reproduced by CDC on the live machine: the cap is **deleted** (grep clean), `schema.rs CURRENT = v1.1`
with forward-compat, the live store is untouched (0 nodes carry `source:`, all 60 still `v1.0`), and
`repair()` preserves id/edges/status/number by **cloning the frontmatter and mutating only the delta**
(a stronger guarantee than the slice-doc asked for). The s03 schema-bump carry-forward is **closed**
(executed here; blast radius smaller than feared — only 7 test literals, found by grep). **One
load-bearing finding, carried onto s05:** the named-arc `number` handle is **position-based**
(`named_arc_number(index) = 1900 + index·100`, from the CDC slice-doc spec) — collision-free and
correct for a one-shot, but (1) unstable under *adding* a named arc (a re-run then mints a **duplicate**
node, since idempotence keys on `(type, number)`), and (2) not recomputable from a name (so coverage
can't resolve named arcs — CC's disclosed gap). Both dissolve once the number stops being a correctness
key: **s05 must, before minting named-arc nodes on the live store, either make the handle name-derived
or move idempotence onto `source.paths` (F12 — now load-bearing, not optional).** CC's disposition of
the coverage gap (defer to s06's source-based matching, ODD-0025 §5) is concurred. MF-2/3/5 stay
planned — live outcomes s05 produces.

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
