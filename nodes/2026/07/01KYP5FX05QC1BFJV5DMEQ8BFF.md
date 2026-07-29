---
id: 01KYP5FX05QC1BFJV5DMEQ8BFF
number: 554081700
type: artifact
schema: artifact/v1.1
name: Slice 07 closing report — Live repair run
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice07-live-run/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SXTKPJ2GN9SXYEMMN
---
# Slice 07 closing report — Live repair run

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 07 · **Feeds:** MF-2, MF-3, MF-5
> **Realizes:** ODD-0025 §2.1 (hard gate), §2.2 (`source` on every migrated node), §2.3
> (project/synthesis exclusion), §2.8 (update-in-place) · **Assignment:** `cc-prompt.md`
> **Ledger:** `ledger.md` (F-1…F-11) · **Implemented by:** CC · **Date:** 2026-07-28
> **Branches:** `release/1.0.x` (the F-1 doc fix, commit `b901b12`) + `odm` (the live store, commit
> `7b4eb57`, atop known-good `e2ab628`) · **Evidence class:** live, direct-read-of-committed-store
> (LEDGER-DISCIPLINE v2.0 §B class-(b)); CDC reproduction pending.

## What shipped

**The arc's first live mutation** — fired the s06-built, CDC-verified `odm migrate` flow on the real
`.worktrees/odm` store, behind the full snapshot → dry-run → adjudicate → fire → verify protocol:

1. **Opened with the F-1 doc fix** (CDC v2.5): corrected `self_host_inner`'s comment on
   `release/1.0.x` to state that the repair-before-import order is a composability/report-attribution
   choice, not a correctness invariant (s06 already proved both orderings converge). Committed
   separately (`b901b12`) before touching the store.
2. **Captured the before-manifest and known-good SHA**: 60 nodes (1 project, 6 arc, 39 slice, 9
   design, 5 research — corrected from a small miscount in the slice-doc's baseline note), 0
   source-bearing, all `schema: */v1.0`, 1 retired node, `context.json → arc #1600`, a full sha256
   fingerprint of every node file, and the store's HEAD SHA `e2ab628`.
3. **Dry-ran and adjudicated**: `odm migrate docs/design-v1.0.0 --dry-run` reported 44 reconciled / 17
   created / 46 skipped, 0 errors. Traced every count against the corpus by direct inspection (not
   just against the slice-doc's "≈47" estimate) before firing — see the F-2 finding below.
4. **Fired for real, one commit**: `odm migrate docs/design-v1.0.0`, then committed the entire store
   worktree as a single commit (`7b4eb57`) atop the captured SHA, with the exact before/after deltas
   in the commit message and the revert command recorded.
5. **Verified the live outcome** exhaustively against the committed store — see the ledger.

**Result:** odm's own corpus goes from 44 stub bodies / 0 source / 6-of-12 arcs to **0 stubs / 61
source-bearing arc/slice nodes / all 12 arcs represented**, `check` green, `orient`/`rollup`
byte-stable, fully re-run-idempotent. No gate failed; no rollback was needed.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | Pre-flight | **done** | Doc fixed (`b901b12`); `1.0.x` green; store clean; before-manifest + SHA captured. |
| F-2 | Dry-run first | **done** | 44/17/46, 0 errors, adjudicated exactly (see finding below); store byte-identical after. |
| F-3 | Fired as one revertible commit | **done** | `7b4eb57` atop `e2ab628`; 61 files changed, matching 44+17 exactly. |
| F-4 | Zero stubs | **done** | 0 stubs on direct scan; two verbatim spot-checks against source. |
| F-5 | Source-bearing, project + retired excluded | **done** | 61/77 source-bearing; project and retired both confirmed zero-diff, zero-source. |
| F-6 | Full representation | **done** | 0 uncovered arc-plans/slice-docs; the "8/12" figure is a different, pre-existing detector's known limitation. |
| F-7 | Body-hash + schema v1.1 | **done** | Second run: 0 `BodyHashMismatch`; 61 nodes at `*/v1.1` exactly. |
| F-8 | `check` green, `orient`/`rollup` stable | **done** | `check` exit 0 (8 warnings, not errors); both byte-stable across two runs. |
| F-9 | `context.json` re-pointed | **done** | Unchanged, deliberately — `migrate` never writes it; re-pointing is a separate operator act. |
| F-10 | Re-run idempotent | **done** | Second run: 0/0/63, store untouched. |
| F-11 | Rollback + findings discipline | **done** | No rollback needed; four pre-existing findings disclosed (below); s08 deferral confirmed intact. |

**Rows: 11. Done: 11. Deferred: 0. No-op: 0.** No silent drops — the slice-doc's "Out" items (the 14
design/research nodes' `source`, loose-doc coverage, the project's synthesis re-cast, the arc-close
reconcile demo) are confirmed untouched: all 14 design/research nodes still `*/v1.0` with 0 source
(unchanged file-for-file, verified by `git diff` showing no touch to any node whose type isn't
project/arc/slice).

## Verification

| Check | Result |
|-------|--------|
| `1.0.x` green before touching the store | `make check` — all green |
| Store worktree clean before the run | `git -C .worktrees/odm status --porcelain` — empty |
| Before-manifest fingerprint | sha256 composite `720d8981…` over all 60 node files |
| Dry-run | exit 0; 44 reconciled / 17 created / 46 skipped; 0 errors; store fingerprint unchanged after |
| Live run | exit 0; identical counts to the dry-run |
| Commit | `7b4eb57` on `odm`, atop `e2ab628`; 61 files (44 modified + 17 new) |
| Stub count after | 0 (direct scan; was 44) |
| Source-bearing after | 61/77 (project + retired + 14 design/research excluded, exactly) |
| Project node | zero diff, zero source — untouched |
| Retired node | zero diff, zero source — untouched |
| Doc-coverage (`--coverage`) | 0 uncovered arc-plan / slice-doc entries (both class sections absent from the report) |
| Second `migrate` run | exit 0; 0 `BodyHashMismatch`; 0 reconciled / 0 created / 63 skipped; store unchanged |
| Schema spread after | 61 × `*/v1.1` (12 arc + 49 slice); project `project/v1.0`; retired slice `slice/v1.0`; 14 design/research `*/v1.0` |
| `odm check` | exit 0; 0 errors, 8 warnings (not `--strict`) |
| `orient` × 2 | byte-identical |
| `rollup --dry-run` × 2 | byte-identical |
| `context.json` | unchanged, confirmed by empty `git diff` |

## Deviations / findings (flagged, per the working agreement)

None of these caused a gate failure or required a rollback. All four are **pre-existing** — none
introduced by this slice's code or by the live run itself — and are disclosed here because the live
run is what made them visible for the first time (fixtures never exercised the real corpus or a real
absolute path).

### Finding 1: `source.paths` stores absolute, machine-specific paths

Every `source.paths` entry — on both the 44 reconciled nodes and the 17 newly-created ones — is an
**absolute path** (e.g. `/Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/...`), not a
repo-relative one. Traced to `odm-cli/src/migrate.rs::resolve()`, which joins the resolved repo root
with the (relative) argument I passed unconditionally into an absolute `PathBuf`. This predates every
arc-migration-fidelity slice (present since at least s01/s03) and every fixture test used a `TempDir`
absolute path too, so nothing in this arc's fixture-only slices could have surfaced it — it only became
visible once real, portable-repo paths were the input. Not fixed here (out of this slice's scope — a
`resolve()`/`source.paths` design question, not a live-run defect); worth a named follow-up if
`source.paths` is ever expected to be diffable or comparable across machines/checkouts/CI.

### Finding 2: `representation()`'s named-arc heuristic gap is now visibly confusing in the rendered report

`--coverage`'s summary line reads "Representation: 8/12 arc dirs" even though the **doc-coverage**
section (the actual, source-based, exact criterion F-6 cares about) shows 0 uncovered arc-plans. This
is the exact limitation disclosed in slice05 (`arc_coordinate` cannot derive a coordinate for a named
arc's directory name, so `representation()`'s structural heuristic still counts the 4 named arcs as
"missing" even though they now have real, source-bearing nodes). It was a documented, accepted
limitation in code comments; seeing it produce a superficially alarming "8/12" in a real report — while
the actually-correct number sits one section away with no cross-reference — is a UX/report-clarity gap
worth naming for whoever next touches `coverage.rs`'s rendering (s08 is the natural home, since it
already touches coverage enforcement).

### Finding 3: the `provenance_absence` detector checks a key that no longer exists

`--coverage`'s "Provenance absence: 77" line reports **every single node**, including all 61
source-bearing ones. The detector (`coverage.rs::provenance_absence`) checks for a literal
`provenance:` frontmatter key — the name ODD-0025 §2.0 deliberately renamed to `source:` at s02, to
keep 0013's `provenance` (derived lineage) reservation clean. The detector's own doc comment anticipated
the field being "typed" under the same name, not renamed, so it now checks for a key that will never
exist by design. This predates arc-migration-fidelity slice05 (unchanged since s01) and is unrelated to
this slice's own work; not fixed here (a detector-correctness bug, not a live-run outcome) but disclosed
because a stale "77 nodes missing provenance" reading directly contradicts the "61 source-bearing" fact
one line above it in the same report, which is exactly the kind of drift a "no file left behind" tool
cannot afford in its own output.

### Finding 4: `check`'s 8 new warnings are genuine, pre-existing plan-decomposition facts, not migration defects

`odm check` surfaced 8 warnings after the run — `undecomposed-parent` on the project and arc `#1600`,
`undeveloped-stub`/`advanced-without-decomposition` on 4 of the freshly-imported named arcs. Traced
each: the 4 named arcs (`arc-llm-command-surface`, `arc07`, `arc08`, the release-hardening arc) genuinely
have **no slice subdirectories** in the plan tree yet — `check` is correctly reporting that these arcs
are still undecomposed *in the plan itself*, not that migration missed anything. These facts were
invisible before this run only because the arcs had no nodes at all; the run didn't create the gap, it
made an existing one visible. Warnings only (`check` exit 0, not `--strict`) — no gate failure, no
rollback.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV, class-(b) reproduction)

**Did s07 make the corpus faithful and fully represented?** Yes, at exactly the scope the arc-plan
(v2.6) scoped this slice to: every plan (project/arc/slice) node is now either faithful-and-source-bearing
or correctly excluded (project, retired). Zero stubs, zero body-hash failures, all 12 plan arcs
represented, `check` green, `orient`/`rollup` reproducible, fully idempotent. **This is where MF-2,
MF-3, and MF-5 stop being fixture-verified claims and become live, reproduced facts** — per
LEDGER-DISCIPLINE v2.0 §B, class-(b) composition rows are always independently reproduced at their own
scale, never inherited from a slice's fixture attestation, and that reproduction happened here, on the
committed store, not on a fixture.

**What the live run revealed that the fixtures didn't** (the four findings above, plus):

5. **The dry-run's actual count (44) diverging from the plan's estimate (≈47) was resolvable by
   inspection in minutes, not a real gap** — but it is a small reminder that a slice-doc's baseline
   numbers, gathered by a human/CDC pass over the corpus, can drift slightly from a fresh, mechanical
   recount by the time CC runs the protocol. The dry-run-adjudicate step is exactly the safeguard that
   catches this class of drift before it matters; it worked as designed here.
6. **A live corpus surfaces detector staleness that fixtures structurally cannot** — Findings 1–3 are
   all cases where a fixture (small, synthetic, freshly-authored per test) could never have exercised
   the actual accumulated drift (absolute paths from a real checkout; a heuristic gap against a real
   named-arc population; a detector checking a field name that was renamed two model-versions ago
   against a corpus that's carried the old assumption since s01). Recommend: whenever a slice's fixture
   suite is green but the change is about to touch something a fixture cannot represent (a real,
   long-lived corpus; a real absolute filesystem path; a detector's *name* assumption), budget time for
   exactly this kind of live-corpus reading pass, even when — especially when — nothing is expected to
   surface.

**The slice-scale silent-drop diff:** scope-as-specified (`slice-doc.md`'s "In"/"Out") vs.
scope-as-delivered — no drops. The 14 design/research nodes and the ~211 loose docs are confirmed
untouched (still `*/v1.0`, still 0 source, verified by `git diff` touching no node outside the
project/arc/slice family) and explicitly handed to **s08**, not silently left for someone to
rediscover as a `check` gap later. The project's synthesis re-cast (s09) and the arc-close reconcile
demonstration (s10) were not pulled forward.

**Recommended arc-ledger update:** **MF-2, MF-3, and MF-5 move `planned → done`** — their live-corpus
outcome is now reproduced (zero stubs; every non-excluded plan node source-bearing; all 12 arcs +
their slices represented), per LEDGER-DISCIPLINE v2.0 §B. Evidence: this closing report + the committed
store (`odm` branch, commit `7b4eb57`) as the direct-read artifact CDC verifies against. MF-1 (doc-coverage
check wired into `odm check`) and MF-6 (`artifact` node minting) remain **planned** — that enforcement
step, plus the 14 design/research nodes' `source` and the ~211 loose-doc coverage, are **s08**'s job, as
scoped. Findings 1–4 above are recommended as **named follow-ups**, not blocking — none is a
correctness defect in what this slice delivered.
