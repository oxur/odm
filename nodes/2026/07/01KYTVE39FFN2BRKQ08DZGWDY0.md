---
id: 01KYTVE39FFN2BRKQ08DZGWDY0
number: 521363000
type: artifact
schema: artifact/v1.1
name: Slice 10 closing report — Coverage live run
created: 2026-07-29
updated: 2026-07-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice10-coverage-live-run/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-31
edges:
  part_of: 01KYP5FRXJFZ3FSF123XS3FD9G
---
# Slice 10 closing report — Coverage live run

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 10 · **Feeds:** MF-1, MF-3, MF-5,
> MF-6 · **Realizes:** ODD-0025 §2.5/§2.6/§2.7 (live), plus five operator-directed additions beyond the
> original scope (below) · **Assignment:** `cc-prompt.md`, `cc-prompt-iteration1.md` · **Ledger:**
> `ledger.md` (F-1…F-21: base F-1…F-15, iteration 1 F-16…F-21) · **Implemented by:** CC ·
> **Date:** 2026-07-29 (base + iteration 1, same day)
> **Branches:** `odm` (the live mint `355404a` atop known-good `7226797`; the iteration-1 corrective
> `2fc25f5` atop `355404a`) + `release/1.0.x` (base: six commits, the F-11…F-14 capability fixes + the
> F-15 display feature; iteration 1: two commits, `ff68186` the seam fix + guard + fixtures, `3eddf38`
> the `canonicalize_source_paths` live-fix mechanism) · **Evidence class:** live,
> direct-read-of-committed-store (class-(b)) for F-1…F-9 and F-18…F-19, fixture-attested (class-(a)) for
> F-11…F-17's code, reproduced against the live corpus for F-11…F-15/F-19's outcome.

## What shipped

**The live run itself** (F-1…F-10, as scoped): self-host picked up slice09/10's own plan nodes (2);
254 `artifact` nodes minted for the whole `docs/design-v1.0.0` supporting-doc corpus (ledger/cc-prompt/
cdc-verification/closing-report/other, mint-all, no exemption); 12 of the 14 sourceless design/research
nodes backfilled with `source` (2 correctly left untouched — drifted, see below); the 4 ODDs authored but
never migrated (0022–0025) imported; `[coverage] scan_root = "docs"` activated in the same commit. One
commit (`355404a`), fully idempotent, no collateral to the 62 pre-existing nodes/project/retired,
`orient`/`rollup` byte-stable.

**Five things this slice did not originally plan to do**, all operator-directed, all disclosed here
rather than silently absorbed:

1. **`docs/design/index.md` + `templates/*.md` excluded from doc-coverage** (F-11) — infrastructure
   files the legacy importer already never ingests; without this, the live coverage gate could never
   reach 0-uncovered.
2. **`docs/dev/**` minted as `NodeType::Note`** (F-12) — a new, general `odm-migrate` capability (not
   project-specific), 31 nodes, tagged by immediate subdirectory, deliberately uncontained.
3. **Real, git-derived `created`/`updated` dates** (F-13) — every creation/reconcile path was stamping
   "the day this ran" instead of the source file's actual history, despite the mechanism (RH F-20)
   already existing for the separate, manually-invoked `--replan` step. Operator-identified; fixed by
   wiring `git_derived_dates` into every path that had never used it.
4. **`check_decomposition`'s drift/stub/undecomposed-parent checks now count only work-type children**
   (F-14) — the mint attaching artifacts to already-`decomposed`-affirmed arcs was reading as
   decomposition drift, which would have blocked F-7's green `check` outright.
5. **`odm list` tree-nests a slice-/arc-attached `artifact`** (F-15) — a pure display change (no store
   mutation) so a supporting doc genuinely, deterministically connected to a slice or arc reads as part
   of that scope's row, not as an orphan in the flat reference group below the divider.

Items 3–5 were caught by the operator reviewing the live `odm list`/`check` output directly, not by any
test — each is disclosed below with what was found, what was fixed, and how it was verified.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | Pre-flight: green + clean + snapshot | **done** | SHA `7226797`, before-manifest recorded; re-verified identical after the mid-run discard-and-redo. |
| F-2 | Dry-run adjudicated | **done** | Adjudicated against a *fresh* live scan (s01's own inventory was stale); every dry-run count matched the real run exactly, twice (once before, once after the F-13/F-14 fixes forced a redo). |
| F-3 | Artifact mint-all | **done** | 254 nodes; `coverage-report.md` confirmed minted; full structural cross-check (251 files) confirms one node per file. |
| F-4 | Mint fidelity | **done** | 0 `BodyHashMismatch`; the gate is unconditional and hard (mint aborts entirely on any failure). |
| F-5 | Containment | **done** | 250/251 exact matches on a full cross-check; the 1 exception is a retired, sourceless tombstone, explained not defective. |
| F-6 | Design/research backfill | **done** | 12/14 backfilled; 2 (ODD-0013, ODD-0020) correctly left untouched — drifted, disclosed, deferred to s12. |
| F-7 | Coverage enforced + green | **done** | `[coverage] scan_root = "docs"` live; `check` exit 0; **371/371 covered, 0 uncovered** — the *whole* docs tree, wider than originally scoped, made possible by F-11/F-12. |
| F-8 | No collateral + idempotent | **done** | Project/retired last-touch unchanged (`b45b122`); 0/0/0/0 on re-run; `orient`/`rollup` byte-stable ×2. |
| F-9 | One revertible commit | **done** | `355404a` atop `7226797`; undo documented; scope growth (F-11…F-15) disclosed, not silent. |
| F-10 | No model drift, clean code | **done** | Clippy clean throughout; 0 `unsafe`; no ODD edited. |
| F-11 | Infra-file exclusion | **done** | `coverage.rs` + fixture test; live "odd" bucket dropped 6→4 as expected. |
| F-12 | Note mint-all | **done** | New `notes.rs`; 31 live mints, tagged, uncontained; 8 tests. |
| F-13 | Git-derived dates | **done** | New `git_derived_dates`, wired into every creation/reconcile path; 2 real-git-repo unit tests; live spot-check confirmed. Known residual (cross-repo extraction) disclosed, accepted by operator. |
| F-14 | Decomposition-drift fix | **done** | `is_work()`-only children; 2 tests; 0 live drift findings post-mint (was 4, reproduced pre-fix). |
| F-15 | `odm list` promotion | **done** | Pure display change; 9 tests; 250/251 live cross-check. |

**Rows: 15. Done: 15 (1 later corrected — see Iteration 1). Deferred: 0. No-op: 0.** No silent drops: the
slice-doc's "Out" items (the capability itself — that's s09; synthesis/L-8b — s11; the arc-close
reconcile run incl. living-doc-drift — s12) are confirmed untouched. The 2 drifted design/research nodes
(F-6) are a **new**, disclosed item for s12's reconcile to pick up, not a silent gap.

## Iteration 1 — source-path portability regression (CDC finding)

**What regressed.** CDC verification of the base close (above) reproduced the live store and found a
silent portability regression: the 4 design nodes minted this slice (#22 store-home, #23 command-surface,
#24 id-scheme, #25 migration-fidelity) carried **absolute** `source.paths` — CDC v2.8 Finding 1's exact
bug, which s08 fixed and drove to 0 across the whole corpus. Root cause: the design/ODD import path
(`mapping::build_node`) was the one node-creation seam never routed through `fidelity::relativize`,
unlike `artifact.rs`/`notes.rs`/`mapping::backfill_source`, which already relativized correctly. `odm
check` passed green at the base close only because the doc-coverage matcher relativizes stored paths
*before* comparing — the s08 transition-tolerance masked the regression, and nothing enforced "a stored
`source.paths` must be relative" as an invariant in its own right. This falsifies F-10's "no model drift"
claim from the base close (re-dispositioned in `ledger.md`, not silently overwritten).

**The fix.** Two parts, landed as two `release/1.0.x` commits before any live write:

1. **The seam** (`ff68186`): `mapping::build_node` now relativizes `source_path` against the plan-tree
   anchor before storing it — the one missing call site, brought in line with every other creation path.
2. **The durable invariant** (`ff68186`): `odm_core::check` gains an **unconditional** Error rule,
   `absolute-source-path` — any `source.paths` entry that is absolute or `.worktrees/`-anchored, on any
   node type, fails `check`. Unlike `[coverage] scan_root`, no config gate — there is no legitimate
   absolute case. This is the point of iteration 1: had this rule existed at the base close, the
   regression would have gone red immediately, not slipped through silently.
3. **The live-fix mechanism** (`3eddf38`): `mapping::canonicalize_source_paths`, `backfill_source`'s
   complement — that pass only ever touches *sourceless* nodes, and these 4 already had a (malformed)
   `source`, so it was invisible to every existing reconcile pass. Wired into the default `odm migrate`
   flow alongside `backfill_source`, with its own dry-run-aware, idempotent CANONICALIZE report.

**The guard, proven against the real regression (F-18).** Built the freshly-fixed release binary and ran
`odm check` against the live store **before** the corrective rewrite (still at `355404a`, still carrying
the 4 absolute paths): the new rule fired exactly 4 times, naming exactly `#22`/`#23`/`#24`/`#25` and no
others across the full 369-node corpus. This is the strongest evidence available that the guard catches
the *actual* regression, not just a synthetic fixture shape.

**Before/after counts (F-19, the live rewrite).** Snapshot fingerprint
`be63aa49b738eb0eadb7b9cf2f4d6df1212a0ab7ce202ca745e4f267e927b485` over 369 node files at `355404a`.
`odm migrate docs/design --dry-run` adjudicated to exactly **4 canonicalize / 0 create / 0 upgrade / 2
drift (skipped, the pre-existing ODD-0013/0020 case, untouched)**; fingerprint unchanged after the
dry-run. Fired as commit `2fc25f5` on `odm`, atop `355404a`. `git diff 355404a..2fc25f5`: exactly 4
files, exactly one `source.paths` line changed in each, nothing else — same ids, same bodies, same
schema, same dates. Node count 369 → 369 (0 net create/delete). Post-fire: `odm check` shows **0**
`absolute-source-path` findings (was 4); the 2 pre-existing, unrelated `uncovered-doc` findings (this
iteration's own two not-yet-artifact-minted planning docs) are unchanged before/after. `odm orient` is
byte-identical across 2 runs. A second `odm migrate docs/design` reports `0 path(s) canonicalized` —
idempotent. Coverage: 371 covered (unchanged from the base close), 373 total (+2 for this iteration's
own docs, disclosed, not this regression's concern).

## Ledger — per-row walk (iteration 1)

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-16 | Seam fixed | **done** | `mapping::build_node` now relativizes; new regression test green. |
| F-17 | Durable invariant enforced | **done** | Unconditional `absolute-source-path` Error rule; 7 new fixture tests (4 `odm-core`, 3 `odm-cli`). |
| F-18 | Reproduce-the-bug evidence | **done** | Live `odm check` before the rewrite: exactly 4 findings, naming exactly the 4 known nodes. After: 0. |
| F-19 | Live rewrite | **done** | One revertible commit `2fc25f5` atop `355404a`; `git diff` confirms path-string-only change; idempotent; `orient` byte-stable. |
| F-20 | Optional ODD-0025 §4 companion | **deferred (flagged)** | Would introduce fresh body drift on node #25, one of the 4 in-scope nodes — skipped per the cc-prompt's own allowance. |
| F-21 | `ROLLUP.md` staleness (disclosed) | **deferred (flagged)** | Stale since before this iteration; incidental regeneration during verification reverted to keep the diff scoped. |

**Rows: 6. Done: 4. Deferred (flagged, disclosed): 2. No-op: 0.**

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace --all-features` | 0 failed, checked after every code change (7 checkpoints) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean throughout (only the pre-existing, unrelated `proc-macro-error2` notice) |
| `unsafe` in every file this slice touched | none |
| Snapshot SHA / before-manifest | `7226797` / `24b1fa4f13df638aa3da6b2db449425ac971003ddfe81773f5a18c769b24ef1d` (78 files) |
| Dry-run vs. fire, all 4 commands | identical counts both times (self-host 2/64, artifacts 254, legacy-design 12+2+4+14, notes 31) |
| Live commit | `355404a` on `odm`, atop `7226797`; 304 files (291 new + 12 modified + config.toml) |
| `odm check` post-commit | exit 0; 0 errors; the same 8 pre-existing warnings, unchanged |
| `odm migrate docs --coverage` post-commit | 371/371 covered, 0 uncovered, 0/0 representation gap |
| Project (`#1000`) / retired (`#1605`) last-touch | unchanged, still `b45b122` |
| Re-run idempotence (all 4 commands) | 0 created / 0 reconciled / 0 minted (2 drifted persist, correctly, each time) |
| `orient` / `rollup --dry-run` ×2 | byte-identical |
| Structural cross-check, artifact containment | 250/251 files match exactly; 1 explained (F-5) |
| Structural cross-check, note tagging | 31/31 notes correctly tagged/uncontained |
| **Iteration 1** — `cargo test --workspace --all-features` | 0 failed, checked after each of the 2 iteration-1 commits |
| **Iteration 1** — `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean |
| **Iteration 1** — `cargo fmt --check` | clean |
| **Iteration 1** — Snapshot fingerprint | `be63aa49b738eb0eadb7b9cf2f4d6df1212a0ab7ce202ca745e4f267e927b485` (369 files), unchanged after `--dry-run` |
| **Iteration 1** — Dry-run vs. fire | identical: 4 canonicalize / 0 create / 0 upgrade / 2 drift (skipped, pre-existing) |
| **Iteration 1** — Live commit | `2fc25f5` on `odm`, atop `355404a`; 4 files, 1 line each |
| **Iteration 1** — `odm check`, `absolute-source-path` findings | before: 4 (naming exactly #22/#23/#24/#25); after: 0 |
| **Iteration 1** — Node count | 369 → 369 (0 net create/delete) |
| **Iteration 1** — `odm orient` ×2 | byte-identical |
| **Iteration 1** — Re-run idempotence | `0 path(s) canonicalized` on second `odm migrate docs/design` |
| **Iteration 1** — Coverage post-fire | 371/373 covered (371 unchanged from base close; +2 total = this iteration's own 2 not-yet-minted docs, disclosed) |

## Deviations / findings (flagged, per the working agreement)

### Scope grew by five items, all operator-directed, none silent

The cc-prompt's own constraint was explicit: "No capability code here... a capability gap here is an
s09 fix, not new s10 scope." Three of the five additions (F-11, F-12, F-14) are, strictly, capability —
they were not built in s09. Two (F-13, F-15) are fixes to pre-existing or newly-surfaced defects, not
new capability. All five were:

- **Discovered live**, not planned in advance — by the operator reviewing actual `docs/dev` content,
  the actual `node list` output, and the actual date columns, not by re-reading the ledger.
- **Explicitly authorized** by the operator in real time, each with a direct decision (docs/dev → Note,
  index/template exclusion, arc-level promotion in addition to slice-level) rather than assumed.
- **Built with the same rigor as everything else this arc**: fixture-tested on `release/1.0.x` first,
  full workspace green, *then* fired/verified against the live store — never committed live untested.
- **Disclosed here**, in the ledger, and in the arc-plan bubble-up below — not folded quietly into the
  original F-1…F-10 scope as if they had always been planned.

This is judged the right call rather than a process failure: the alternative — closing s10 exactly as
originally scoped, with `docs/dev` uncovered, `index.md`/templates permanently red, migrate-time dates
wrong on every future import, decomposition drift misfiring on every arc with mixed children, and
useful supporting docs reading as orphaned in the CLI — would have been a *worse*, less honest slice
close than one that grew under direct operator review and said so.

### Two design/research nodes are drifted, not backfilled — deferred to s12

ODD-0013 and ODD-0020 have both been actively amended throughout this rebuild (this very arc amended
0013 and 0020 multiple times), so their live node bodies — migrated once, early — no longer match the
current legacy file content. `backfill_source`'s hard body-hash gate correctly refuses to silently
backfill `source` over a body it cannot verify still matches. This is not a bug surfaced by this
slice; it is the gate working exactly as designed, applied for the first time against real drifted
content. **s12's reconcile run** (already scoped to handle the living-doc-drift the s08 CDC
verification found on an active arc node) now has a second, structurally identical case to resolve:
these 2 design nodes.

### The date-history residual (F-13)

After wiring `git_derived_dates` in, the operator asked whether the dates were still not matching
expectation, hypothesizing rename-tracking (`git log --follow`) as the cause. Investigation found no
rename-tracking discrepancy *within this repository's own history* — every rename found (`docs/dev/odm/`
→ `docs/dev/`, state-directory moves, etc.) shares its date with the file's true original creation, in
every case checked. The likely explanation — this repo was extracted from the Oxur monorepo, and if
that extraction was a plain copy rather than a history-preserving split, this repo's git log simply does
not go back before the extraction commit for that content — is outside what `git log` in this repository
can recover, and outside `--follow`'s reach (it cannot see across a repository boundary). **Operator
decision: accepted as-is, not chased further.** The dates are still a strict improvement over the prior
"always today" behavior, and are honestly the best this repository's own history can produce.

### Iteration 1: two disclosed, deferred items (F-20, F-21)

**F-20 — the optional ODD-0025 §4 companion, skipped.** `cc-prompt-iteration1.md` explicitly allowed
folding in a one-line ODD-0025 §4 wording fix (`artifact/v1.0` → the actually-delivered `artifact/v1.1`)
*if trivial*. The text edit itself is trivial, but ODD-0025 is node #25's own source file — one of the
exact 4 nodes this iteration's hard gate requires stay byte-identical. Editing it would silently drift
node #25's body from its (now-edited) source, the same living-doc-drift class s08's CDC verification
already found on the active arc-plan node. Rather than introduce a *new* instance of that drift inside an
iteration whose entire discipline is "only the path string changes," this was skipped and flagged per the
cc-prompt's own explicit allowance — a real, worthwhile fix, just not one that belongs inside this
iteration's tight blast radius. Natural home: s12 (reconcile), alongside the 2 already-drifted design
nodes from F-6.

**F-21 — `ROLLUP.md` found stale on `release/1.0.x`, unrelated to this regression.** Running `odm rollup`
twice for the F-19 byte-stability check regenerated the committed `ROLLUP.md` with a 1042-insertion diff
— it still shows pre-RH-C-2 taxonomy labels (`odd #9`) against a corpus that has read `design`/`research`
since the C-2 restamp, so it has been stale since at least before this session's live runs began, not
caused by this iteration. Reverted the incidental regeneration (`git checkout -- ROLLUP.md`) so this
iteration's `release/1.0.x` diff stays scoped to its own two commits. Flagged as a separate, disclosed
housekeeping item — not silently regenerated here, not silently left dirty either.

### `[coverage] scan_root` is wider than the original ledger anticipated

The original F-7 language expected activating coverage would need careful scoping (the s09 cc-prompt's
own concern, and CC's initial plan, was to narrow `scan_root` to `docs/design-v1.0.0` only to dodge the
`docs/dev` and infra-file gaps). F-11 and F-12 removed the need for that narrowing entirely — the live
`scan_root = "docs"` covers the whole tree, exactly as MF-1's original "green on `1.0.x/docs`" language
wanted, not a scoped-down subset.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV, class-(b) reproduction)

**Did s10 enforce coverage live?** Yes: `odm check` is green (exit 0) on the committed corpus because
every doc is now covered (371/371), not because the gate is off — the whole point, delivered with no red
window between mint and gate-flip. MF-1 and MF-6 move to **done** (live-enforced, not just capability).
MF-3 moves to **done for the achievable criterion** — 12 of 14 design/research nodes now source-bearing
live; the remaining 2 are a disclosed, structural drift case routed to s12, not a silent gap.

**What the live run revealed that fixtures didn't:**

1. **A live corpus at real scale surfaces interactions no fixture combination anticipates.** The
   decomposition-drift false-positive (F-14) only exists because real arcs have *both* affirmed
   decomposition *and* a large volume of newly-minted document-family children at once — a shape no
   slice09 fixture happened to construct. The date-stamping gap (F-13) was invisible in fixtures because
   fixture corpora are never git-tracked with meaningful history. Both are exactly the kind of finding
   "prove it on fixtures, fire it live" protocols exist to still catch at the live step, not skip.
2. **Operator review of the actual rendered output is not optional polish — it found two real
   gaps** (dates, artifact-orphaning) that a fully-green test suite did not and could not surface, because
   both are about whether the *result* reads correctly to a human, not whether the code matches its own
   spec. Recommended as standing practice for any future live-mutation slice at this scale: read the
   actual `odm list`/`check` output before calling it done, not just the exit codes.
3. **A drifted node under the hard body-hash gate is a real, expected occurrence once the corpus is old
   enough** — 2 of 14 in this one pass. s12's reconcile-run scope should assume more of these will
   surface, not treat the 2 found here as exhaustive.

**The slice-scale silent-drop diff:** scope-as-specified vs. scope-as-delivered — no drops, five
additions, all disclosed above and in the ledger. Every s10-scoped "In" item landed; every "Out" item
(the capability itself, synthesis/L-8b, the arc-close reconcile run) is confirmed untouched.

**Recommended arc-ledger update:**

- **MF-1** ("doc-coverage check exists and is green") → **done**. `odm check` exit 0, 371/371 covered.
- **MF-6** ("supporting-doc children minted; doc-coverage wired into `check`") → **done**.
- **MF-3** ("every migrated node carries a `source` sub-map") → **done for the achievable set** (12/14
  design/research nodes; the remaining 2 routed to s12 as a drift case, not a gap).
- **MF-5**'s report-clarity resolution (s09) stays resolved; the live corpus now also has 0 representation
  gap (12/12 arcs, all slices, confirmed).
- **s11 (synthesis + L-8b) is next**, unblocked.
- **s12 (reconcile run)** picks up: the living-doc-drift reconcile for the active arc node (s08 CDC
  finding), the 2 drifted design/research nodes (ODD-0013, ODD-0020) this slice found and correctly
  declined to silently backfill, **and now also** F-20's ODD-0025 §4 wording touch-up (deferred here for
  the same drift-avoidance reason).

## Bubble-up addendum — iteration 1 (CDC finding + fix)

**Did iteration 1 deliver what CDC's finding required?** Yes. CDC reproduced the live store after the
base close and found the exact CDC v2.8 Finding 1 bug had recurred on one seam (4 nodes, absolute
`source.paths`), masked by the coverage matcher's own transition tolerance. This iteration: (1) fixed the
seam so no future `migrate` run can reproduce it; (2) added an unconditional `check` rule so the *class*
of bug — not just this instance — is caught immediately, not silently, at any future recurrence; (3)
proved the rule against the real regression before fixing it (4 → 0), not just against synthetic
fixtures; (4) fired the corrective rewrite as one revertible, verified commit. **MF-1/MF-6's "coverage
enforced live" now rests on a durably self-checking foundation** — the specific gap that let this
regression through green is itself now a hard-gated invariant.

**What this revealed that the base close's own verification didn't catch:** the base close's F-10 row
("no model drift") was asserted from a code-level review (grep for `unsafe`, confirm the identity axis
unchanged) rather than from re-running `check` with a rule that could have caught this specific
regression — because that rule didn't exist yet. This is not a process failure so much as the textbook
case LEDGER-DISCIPLINE's CDC-independence exists for: the doer's self-verification and the independent
verifier's reproduction-from-artifacts can catch different things, and here they did. The fix is
structural (a durable check rule), not just a one-time data correction, precisely so this class of gap
doesn't require a second independent catch next time.

**Arc-ledger update (supersedes/extends the base bubble-up above):** MF-1/MF-6 stay **done**, now with
the added qualifier "durably enforced" (the invariant that made the regression possible is itself
checked). s10 does **not** flip to CDC-verified PASS on the base close alone — it requires CDC to
reproduce iteration 1's two claims (0 absolute `source.paths` remain; the `absolute-source-path` rule is
present and unconditional) against the committed store and code before the slice is fully closed.
