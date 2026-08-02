---
id: 01KYZTRECSZRK6SRC3Q45ETQ48
number: 560811200
type: artifact
schema: artifact/v1.1
name: 'Arc — Migration Fidelity: closing report (arc-close / composition + CDC gate)'
created: 2026-07-31
updated: 2026-07-31
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-01
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
---
# Arc — Migration Fidelity: closing report (arc-close / composition + CDC gate)

> **Arc:** Migration Fidelity (named, number-deferred) · **Closed by:** CDC (independent) ·
> **Date:** 2026-07-31 · **Method:** LEDGER-DISCIPLINE v2.0 §B (arc scale) + PM Part IV.
> **Store-state reproduced by direct git read** of the shipped `odm` corpus (`odm@e06fffe`, 387 nodes);
> runtime rows (`check`/`orient`/`rollup`/`migrate` execution) attested-by-CC → CI (the store's binaries
> are macOS; this gate reads the store from a Linux bridge and cannot execute them). **Scope of this
> report:** the MF-9 composition check + the arc bubble-up. It does **not** itself run the two remaining
> runtime acts (the final reconcile-and-freeze and the P-12 self-host demonstration) — those need the Mac
> binary and are specified as a runbook in §7; the arc **closes green when they run**.

## Verdict

**The arc composes — odm self-hosts *faithfully* on its own plan corpus (MF-9 met), with two runtime
preconditions and one honestly-scoped deferral.** Reproduced by direct read: the shipped store's planning
corpus (project/arc/slice nodes) is byte-faithful to current sources, fully covered, source-bearing, and
carries the modeled vision as an attested synthesis. Eight of the nine arc-ledger rows (MF-1,2,3,5,6,7,8,9)
are **done — reproduced**. The ninth, **MF-4 (frontmatter fidelity), is capability-complete and proven on
the four design nodes this arc migrated, with a pre-existing 14-node `version`-drop in the RH-era design
corpus correctly routed to L-8** (§4) — not a defect this arc introduced.

Two things must run on the Mac before P-12 is demonstrated (§7): **(1)** the **final reconcile-and-freeze**
(CDC s13 Finding CDC-F1) to close the two by-design living-plan-tail drifts so the demonstrated corpus is
byte-faithful end-to-end; **(2)** the **P-12 self-host acceptance demonstration** (a fresh session orients
from `odm orient` alone). Neither is discretionary and neither is code work — they are the arc's final
runtime acts. `make check` is confirmed green on the operator's machine.

## 1. What the arc set out to do, and did

Migration Fidelity was shaped from the 2026-07-27 reconciliation audit, which found odm's self-hosted
corpus was a skeleton: 44 empty stub bodies, 6 of 11 arcs missing, ~211 uncovered docs, 0 provenance. The
arc's job (project-plan §2a): make migration **faithful, verifiable, repeatable** — 1:1 verbatim bodies
under a hard body-hash gate, a `source` sub-map on every node, frontmatter-fidelity via a schema-mapping,
and a doc-coverage check (no file left behind) — and repair odm's own plan corpus to 100%. Thirteen slices
delivered it:

- **s01–s03** — coverage discovery (the exact work-list), the fidelity *model* (ODD-0025), and the
  fidelity *core* (1:1 import + the hard body-hash gate + `source`/`author`/`version` typing).
- **s04–s08** — scope-repair capability, source-identity, the live-run capability + the **live repair run**
  (44 stubs → 0, the 6 excluded arcs imported), and source-path portability (`docs/…`-relative).
- **s09–s10** — the enforced doc-coverage `check` rule + its **live firing** (mint-all + gate on;
  s10-it1's CDC-caught absolute-path regression fixed and permanently guarded).
- **s11–s13** — synthesis (`supersedes`→`Vec`, bidirectional lineage, the two synthesis regimes) + L-8b;
  the reconcile *capability*; and the **live reconcile + vision mint** — every drifted node re-snapshotted,
  the vision minted as an attested editorial-merge synthesis over a byte-1:1 `project-plan` node.

Every slice s04–s13 is CDC-verified PASS; s01–s03 closed with CDC verification on s01/s03. The
capability/live-run split (s06/s07, s09/s10, s12/s13) held throughout and repeatedly proved its worth — CC
independently re-derived the s12/s13 cut.

## 2. Composition check (MF-9) — reproduced against the shipped store

The exit criteria, each reproduced or routed:

| Exit criterion | Disposition |
|---|---|
| **Coverage** — every `.md` under `docs/*` maps to a node; check green | **met** — set-difference at `e06fffe`: 385/386 scanned docs primary-covered, the 1 residual the retired A6·s05 UAT tombstone (fallback-covered by slice number); index/template conventionally excluded. 0 of 20 arc-fidelity slice1x docs uncovered. The arc's own report/verification artifacts are `artifact` nodes (F10 mint-all), so the enforced check won't flag them in perpetuity. |
| **Body fidelity** — `normalize(body)` hash == source; no stubs | **met, modulo the living tail** — corpus-wide recompute vs current sources: 383 faithful / **2 drifted** / 0 stubs. The 2 (`arc-plan.md` arc node, slice10 `ledger.md`) are the by-design living-plan tail (sources edited after reconcile; §2.9-sanctioned, `check`-invisible). **CDC-F1: the final freeze closes them (§7).** |
| **Source record** — every migrated node carries `source` (+ author/version where present) | **met** for the plan corpus (project/arc/slice all source-bearing; project + tombstone excluded by design); design/research nodes carry `source` + author. See MF-4 (§4) on `version`. |
| **Representation** — all arcs + slices, uncapped, named arcs numbered, children minted | **met** — 12 arc + 56 slice + 2 project nodes; 268 artifact + 31 note supporting-doc children; `--coverage` reads 0 unrepresented arc/slice dirs (was the pre-s09 "8/12" undercount). `MAX_MVP_ARC` removed (s07). |
| **Frontmatter fidelity** — schema-mapping green over originally-present fields | **capability done + proven on this arc's migrations; 14-node RH-era `version`-drop → L-8** (§4). |
| **L-8b cleared** — ODD-0013/0017/0018 (+0019/0020) reconciled to authoritative states | **met** — s11 corrected doc-tree states + `git mv` to `04-accepted/`; s13 reconciled the node gate vectors live, 0017/0018 re-discovered by identity + paths rewritten and resolving. |
| **Reflexive** — `check` green; `orient`/`rollup` reproduce truth; P-12 satisfiable against a *faithful* corpus | **met on the runtime side** (`make check` green, operator-confirmed; `orient`/`rollup` byte-stable → CI) **pending the freeze + the P-12 demo (§7)** so the demonstrated corpus is byte-faithful, not skeleton. |

## 3. Arc ledger — CDC disposition (MF-1…MF-9)

MF-1 (coverage check green), MF-2 (0 stubs / body gate), MF-3 (source records), MF-5 (representation),
MF-6 (children minted + check wired), MF-7 (synthesis lineage + vision re-cast), MF-8 (L-8b) — all **done,
reproduced** by direct read of the shipped store (evidence in the arc-plan rows + each slice's
`cdc-verification.md`). MF-9 (**compose**) — **met** per §2, modulo the §7 runtime preconditions. MF-4 —
see §4.

## 4. Finding CDC-ARC-1 (MF-4): frontmatter `version` dropped on 14 RH-era design nodes — routed to L-8

The composition check reproduced the §2.4 frontmatter-fidelity mapping across all 18 design/research nodes.
**Result: 4 faithful, 14 with a dropped `version` field.** Each of the 14 has a `version:` in its *source*
frontmatter (e.g. ODD-0013 `version: 2.5`, ODD-0020 `version: 1.3`) that is **absent** from its node;
every other §2.4 field (number, title→name, author, component, tags, state→cumulative gate reach) is
faithful, and their **bodies are hash-faithful** (this is a frontmatter-field gap, not a body-hash gap).

**Root cause, established by git provenance (not inference):** the 14 were all first written in
`odm@b45b122` (2026-07-26, *"The odm corpus moves into its own store (RH C-5 / SH-6)"*) — the **Release
Hardening store cutover, before this arc existed**, by the pre-ODD-0025 importer that did not yet preserve
`version`. The 4 faithful nodes (ODD-0022/0023/0024/0025) were migrated by **this arc's slice10** with the
current importer and are fully §2.4-faithful. The later `--all` reconcile re-snapshotted the 14 nodes'
*bodies* (reconcile touches body, not frontmatter fields), so the RH-era `version`-drop was never
backfilled.

**Why this is a routed deferral, not a close-blocker:**
- **Scope.** The arc's charter (project-plan §2a) is the migration *capability* + repairing odm's own
  *plan* corpus. The **full design-corpus migration is L-8** — project-plan line 311 files it explicitly
  under *"L-8 → pre-release housekeeping + a design-corpus migrate"* (the slice docs' "L-8a"). The arc
  never scoped a re-migration of the RH-era design corpus.
- **Capability is proven.** The §2.4 mapping demonstrably preserves `version` — this arc's own 4 design
  migrations show it. The gap is old data, not a broken mechanism.
- **Self-host is unaffected.** odm self-hosts on its *plan* corpus (frontmatter-less; §2.4 N/A). The design
  ODDs are supporting docs; the `version`-drop is invisible to `odm check` (no standing frontmatter-fidelity
  rule — correct, since §2.4 is migration-time-only like the body gate) and does not touch P-12.

**Routed to L-8** (design-corpus migrate): a re-migration of the 14 RH-era design/research nodes through the
current §2.4 importer restores `version`. **Hardening suggestion:** L-8 should also add a standing
`frontmatter-fidelity` / `provenance-field` `check` rule — the analogue of s10's `absolute-source-path`
rule — so a dropped mapped-field can never again pass silently. Absent that rule, this class of drift is
migration-time-only and invisible to `check` by design.

## 5. Findings carried / dispositioned

- **CDC-F1 (s13, routed here → §7):** the two living-plan-tail body drifts — the **final reconcile-and-freeze
  closes them**. Hard precondition for a byte-faithful P-12 demo.
- **CDC-ARC-1 (this report, §4):** the 14-node RH-era `version`-drop → **L-8**, + the standing-check
  hardening suggestion.
- **Doc-hygiene (s13 CDC-F2):** the s13 slice `ledger.md`/`closing-report.md` close against `26bea1d`; the
  shipped store is `e06fffe`. Recorded; the shipped chain is `2fc25f5`→`26bea1d`→`e1e94bf`→`e06fffe`.
- No other open findings. Every slice's disclosed deviations (s10-it1 absolute-path regression;
  s13's self-caught schema-drop, `check` undercount, and `wrong-type-field` fix) were verified closed in
  their respective `cdc-verification.md`s.

## 6. Bubble-up (PM Part IV → `../../project-plan.md`)

- **Migration Fidelity → CLOSED** in project-plan §2a (was "shaped, not started").
- **P-6 / P-12:** the self-host DoD is *satisfiable against a faithful corpus* — reproduced on the plan
  corpus — and becomes **demonstrated** when §7's P-12 run completes. MF's close is what turns P-12 from
  "reproducible-at-arc-close" into a live demonstration.
- **P-13 (arc bubble-up findings):** two forward-carried — **CDC-ARC-1 → L-8** (design-corpus `version`
  re-migration + a standing frontmatter-fidelity check), and **CDC-F1 → the freeze (§7)**.
- **L-8b:** subsumed and cleared by this arc (MF-8). **L-8 (design-corpus migrate) remains**, now with the
  precise CDC-ARC-1 work-item attached.

## 7. Runbook — the two remaining runtime acts (Mac binary; then arc closes green)

These are the only steps left. They need the `odm` binary (macOS) and must run **after** all arc-close doc
edits settle (this report, the arc-plan flip, the project-plan bubble-up), because editing the plan docs
re-drifts their nodes. Order matters: **freeze last, then demonstrate.**

**(A) Final reconcile-and-freeze** — closes CDC-F1's two living-plan-tail drifts (and any drift these
arc-close edits introduce) so the shipped corpus is byte-faithful:

```
# on release/1.0.x, store clean, plan docs no longer being edited:
odm migrate --all --dry-run        # adjudicate: expect ONLY body re-snapshots of the edited plan docs
                                    #   (arc-plan.md, slice10 ledger.md, project-plan.md's #1001) — no
                                    #   id/schema/edge churn, no unexpected create
odm migrate --all                   # fire the freeze; commit on the odm branch
# verify: recompute the body gate vs current sources → 0 drifted; odm check exit 0; orient/rollup byte-stable
```

**(B) P-12 self-host acceptance demonstration** — a *fresh* session (no prior context) orients from the
store alone:

```
odm orient                          # must render the full, current plan (arcs/slices/gates/next-actions)
                                    #   from the reconciled corpus, unaided — the self-host claim, live
```

Paste the freeze adjudication + `check`/`orient` output back and I'll do the final CDC read (0 drift
reproduced, P-12 confirmed) and stamp the arc **CLOSED**. Until then the arc is **CDC-verified &
composition-confirmed, pending the freeze + P-12 demo.**

## Closure

Migration Fidelity is **composition-confirmed and CDC-verified** on 2026-07-31 against `odm@e06fffe`:
thirteen slices compose into a faithful, verifiable, repeatable migration and a self-hosting plan corpus.
Eight arc-ledger rows done-reproduced; MF-4 capability-done with the RH-era `version`-drop routed to L-8
(CDC-ARC-1); the two runtime acts (freeze + P-12) specified in §7. **When §7 runs green, Migration Fidelity
closes and the P-12 self-host DoD is demonstrated against a faithful corpus — not a skeleton.**

_Closed by: CDC (independent), 2026-07-31 — store state reproduced by direct git read of `odm@e06fffe`
(chain `2fc25f5`→`26bea1d`→`e1e94bf`→`e06fffe`); runtime rows attested-by-CC → CI; §7 pending the Mac._
