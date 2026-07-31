---
id: 01KYTVE356DEKMWWEFHV3Q98Y8
number: 532438200
type: artifact
schema: artifact/v1.1
name: 'Slice 10 (Migration Fidelity) — CDC verification: Coverage live run (+ iteration 1)'
created: 2026-07-29
updated: 2026-07-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice10-coverage-live-run/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-31
edges:
  part_of: 01KYP5FRXJFZ3FSF123XS3FD9G
---
# Slice 10 (Migration Fidelity) — CDC verification: Coverage live run (+ iteration 1)

> **Arc:** Migration Fidelity · **Slice:** 10 · **Verifier:** CDC (independent) · **Date:** 2026-07-29 ·
> **Method:** LEDGER-DISCIPLINE v2.0 §A CDC protocol + PM Part IV bubble-up check. Live class-(b) rows
> reproduced by direct read of the committed store; runtime rows (`cargo`/`check`/`orient`/`rollup`/
> `migrate`) attested-by-CC → CI (macOS binaries, Linux bridge).
> **Under review:** the live run (`odm@355404a` atop `7226797`), **iteration 1** (`odm@2fc25f5` atop
> `355404a`; `release/1.0.x` commits `ff68186`/`3eddf38`/`6da7a31`), the ledger (F-1…F-21), and both
> closing reports.

## Verdict

**PASS — CDC-verified (post-iteration).** The live coverage run delivered its capability: the whole
`docs/` tree is enforced-covered (**371/371, 0 uncovered**), the mint is faithful and collateral-free, and
the gate is on with no red window. My independent reproduction found **one serious silent regression** —
4 ODD nodes minted with absolute `source.paths`, the s08 Finding-1 bug reintroduced — which **iteration 1
has fixed and, better, turned into an enforced invariant.** Reproduced post-iteration: **0 absolute
`source.paths` remain.** The five operator-directed scope additions are all real, tested, and disclosed;
the two drifted design nodes are correctly declined and routed to s12.

## 1. The live run (F-1…F-10) — reproduced

| Claim | CDC reproduction (direct read, `355404a` vs `7226797`) |
|-------|--------------------------------------------------------|
| One revertible commit | `355404a` atop known-good `7226797`; undo valid |
| Node delta | **291 Added + 12 Modified**; totals **78 → 369** |
| Added composition | 254 `artifact` + 4 `design` (ODDs 0022–0025 imported) + 31 `note` + 2 `slice` (slice09/10 plan) = 291 |
| Modified = backfill only | the 12 M are exactly the design/research backfill (7 design + 5 research) — **zero of the 62 pre-existing source-bearing nodes touched** |
| No collateral | 0 arc/slice/project nodes modified; project `#1000` + retired `#1605` last-touch unchanged (`b45b122`) |
| Gate flipped | `[coverage] scan_root = "docs"` now present in `config.toml` (was absent at s09) — no red window |
| Git-derived dates (F-13) | mints carry **real history** (2026-06-20 … 07-27), not migrate-day |
| Notes (F-12) | 31 `note` nodes, **all uncontained**, tagged by immediate subdir (`docs/dev/research/`→`research`, `docs/dev/skill/`→`skill`), 25 no-subdir untagged |
| Drift (F-6) | 12/14 design/research backfilled; **2 declined** (below) |

### Coverage — 371/371 independently reconciled

I recomputed the doc-coverage set-difference myself. It initially flagged **9 "uncovered,"** which I ran
down against the actual matcher (`doc_coverage`: primary exact `source.paths` **or** a structural
coordinate fallback — `project_exists`/`match_arc`/`match_slice`/`match_odd`). All 9 resolve: **8 are
fallback-covered** (project-plan → project node; the retired UAT slice-doc → `#1605`; the ODD nodes →
`match_odd` by number), and **1 is scan time-skew** (the slice10 `closing-report.md` postdates the mint —
it did not exist at CC's coverage run; my scan hit the later tree). So **371/371 is genuine.** *Nuance
worth recording:* the green rests partly on the **coordinate fallback** (the `number` matching s05 worked
to retire) for the ODD/project/retired cases — a designed, documented "pre-source fallback," transitional
for pre-source/drifted nodes; as s12 backfills the drifted ones and the iteration relativizes the ODDs,
those shift to the exact primary match.

### The 2 drifted design nodes (ODD-0013, ODD-0020) — decline confirmed correct

Both node bodies genuinely differ from their (repeatedly-amended) legacy files — reproduced: **0.81** and
**0.64** similarity. `backfill_source`'s hard body-hash gate **correctly refused** to write `source` over
a body it cannot verify. This is the gate working as designed, the same living-doc-drift class the s08 CDC
verification found on the active arc node. **Correctly routed to s12** (not a silent gap; MF-3 is honestly
"12/14, 2 → s12").

## 2. The five operator-directed additions (F-11…F-15) — verified + ratified

All five exist as real, tested code on `release/1.0.x`, and their live outcomes reproduce:
- **F-11** infra-file exclusion (`legacy::is_excluded` — `index.md`/templates) — the `doc_coverage`
  "excluded infrastructure" branch, confirmed.
- **F-12** `notes.rs` → 31 `NodeType::Note` mints, subdir-tagged, uncontained — reproduced above.
- **F-13** `git_derived_dates` wired into every creation/reconcile path — dates reproduced real.
- **F-14** `check_decomposition` counts `is_work()`-only children (`recompose.rs`) — the mint-attaches-
  artifacts-to-decomposed-arcs false positive is gone.
- **F-15** `odm list` promotes a slice/arc-attached artifact (display only, no store mutation).

**Assessment.** These grew s10 beyond its drawn scope, and I ratify CC's call. They were operator-directed
in real time, **disclosed** (closing report + ledger + arc-plan v2.15, not folded in silently),
**fixture-tested on `release/1.0.x` first, then fired**, and the alternative (closing s10 with `docs/dev`
uncovered and `check` red on infra files) was the worse, less honest slice. **Named process cost:** three
of the five (F-11/F-12/F-14) are strictly *capability*, which the cc-prompt reserved for s09 — they
skipped the capability/live-run separation that normally gets CDC-verified on fixtures *before* firing
live. Because that gate was bypassed, I verified their live outcomes extra-carefully (above) rather than
inheriting the fixture attestation — and they hold. The separation was traded for velocity under operator
direction; the outcome is sound, and the trade is disclosed, so this is a note, not a defect.

## 3. The finding + iteration 1 — regression fixed and *enforced*

**Finding (SERIOUS, was silent).** The 4 imported ODDs (0022–0025) were minted with **absolute**
`source.paths` (`/Users/oubiwann/…/.worktrees/1.0.x/docs/design/04-accepted/…`) — the CDC v2.8 Finding-1
bug s08 drove to 0, reintroduced on the one design/ODD import seam (`mapping::build_node`) that never
routed through `fidelity::relativize`. It passed `check` green because the coverage index relativizes
stored paths before matching, so an absolute path still matched — the s08 transition-tolerance **masked**
it, and no invariant enforced "a stored `source.paths` is relative." Undisclosed by CC (F-10 said "no
model drift"). Confirmed comprehensive: a full scan of all 369 nodes found these 4 and no others.

**Iteration 1 — reproduced (`odm@2fc25f5`, `release/1.0.x` `ff68186`/`3eddf38`/`6da7a31`):**
- **0 absolute `source.paths`** remain across all nodes (was 4). ✅ reproduced
- The 4 nodes rewritten **path-string only** — diff `355404a→2fc25f5` is 4 files × 1 line each,
  bodies/ids/schema/**dates** byte-identical (node #25's body confirmed intact). ✅ reproduced
- **Seam fixed:** `mapping::build_node` now calls `fidelity::relativize` (`mapping.rs:259`), with a doc
  comment naming exactly this regression. ✅ code
- **Enforced invariant** (the durable win): a new **unconditional** check rule — `Violation::AbsoluteSourcePath`
  → code `absolute-source-path` — that fires on any non-relative stored path. Its fixtures
  (`check_source_path_portability.rs`) cover absolute→**Error**, **`.worktrees/`-anchored→Error** (the
  superproject-mis-anchor case s08 also warned of — *broader* than I asked), and relative→green. Had this
  existed at s10, the regression would have gone red, not slipped through. ✅ code + fixtures
- **Live-fix mechanism** `canonicalize_source_paths` (`mapping.rs:504`) with 4 fixtures (rewrite-absolute,
  leave-canonical, skip-sourceless, dry-run-noop). ✅
- **One revertible commit** `2fc25f5` atop `355404a`; the guard was proven against the *real* regression
  before firing (flags 4 → 0). ✅
- **Two items correctly skipped + flagged:** the ODD-0025 §4 wording fix — declined because ODD-0025 *is*
  node #25's own source, so editing it would drift the very node being repaired (a genuinely sharp catch;
  it belongs in s12's re-snapshot, not a casual edit); and an unrelated pre-existing `ROLLUP.md` staleness
  — reverted, not opportunistically fixed (confirmed not modified on either branch). ✅

## 4. Ledger — CDC disposition (F-1…F-21)

- **F-1…F-9** (live run): reproduced (store state) / attested→CI (runtime). ✅
- **F-10** ("no model drift"): was **falsified** by the absolute-path regression → **remediated in
  iteration 1** (0 absolute remain + the enforced guard). Now ✅.
- **F-11…F-15** (additions): verified in code + live outcome; ratified with the process-cost note (§2). ✅
- **F-16…F-21** (iteration 1): the seam fix, the guard, the fixtures, the live canonicalize, the
  reproduce-bug proof, and the close — reproduced (store) / attested→CI (runtime). ✅

**No silent drops.** Every "Out" item (the capability = s09; synthesis/L-8b = s11; the arc-close reconcile
= s12) confirmed untouched. The 2 drifted nodes are a disclosed s12 item, not a gap.

## 5. Findings (severity-classified)

1. **Absolute `source.paths` on 4 ODD nodes** — **SERIOUS, RESOLVED (iteration 1).** Regression of s08's
   release-blocking portability invariant; now fixed *and* enforced by a check rule so it cannot silently
   recur. The strongest outcome available: not just data-repaired, but made mechanically impossible to
   regress unseen.
2. **Capability-in-live-run (F-11/F-12/F-14)** — **process note, no action.** Operator-directed, disclosed,
   fixture-first; the capability/live-run gate was traded for velocity, verified extra-carefully on the
   live outcome, holds.
3. **Coverage green rests partly on the coordinate fallback** — **INFO.** By design (pre-source fallback);
   shrinks as s12 backfills the drifted nodes (the iteration already moved the 4 ODDs to the primary match).
4. **Point-in-time mint vs. a growing corpus** — **INFO.** New docs written after a mint (e.g. these very
   close reports) are uncovered until the next mint/reconcile; inherent, → future mints / s12.

## 6. Bubble-up check (PM Part IV)

- **Did s10 enforce coverage live?** Yes — 371/371, `check` green because covered (not because the gate is
  off), no red window. **MF-1 / MF-6 → done.** **MF-3 → done for the achievable set** (12/14; 2 drifted → s12).
- **What the live run revealed:** real-scale interactions fixtures can't construct (the decomposition-drift
  false positive F-14; date-stamping F-13); operator review of rendered output caught two gaps a green
  suite couldn't; and — the CDC catch — a portability regression that a green check *masked*, now closed
  and guarded.
- **Post-draw scope amendments (operator-directed):** recorded in the s10 slice-doc (a dated amendments
  section) so the plan-of-record shows what was drawn vs. what was added, and why — keeping the original
  scope visible rather than retconning the five additions as always-planned.
- **Arc-plan change:** flip s10 → **CDC-verified PASS** (v2.16); route to **s12**: the 2 drifted design
  nodes (0013/0020) **and** the ODD-0025 §4 re-snapshot (it must be a reconcile, not an edit); **s11
  (synthesis + L-8b) is next**, unblocked. (The s09 ODD-0025 §4 LOW follow-up folds into that s12
  re-snapshot.)

## Closure

s10 **CDC-verified PASS** on 2026-07-29, post-iteration-1. Live run reproduced; the five additions
verified and ratified; the serious portability regression found by independent reproduction, fixed, and
turned into an enforced `check` invariant; the 2 drifted nodes correctly declined and routed to s12. s11
is next.

_Verified by: CDC (independent), 2026-07-29 — against `odm@2fc25f5` (atop `355404a` atop `7226797`) and
`release/1.0.x` `ff68186`/`3eddf38`/`6da7a31`._
