---
id: 01KYP5G0DWD86NXHBMWMJEHAEE
number: 554131200
type: artifact
schema: artifact/v1.1
name: RH C-5 (cutover) — CDC verification + arc-store-home close endorsement
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/c5-cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# RH C-5 (cutover) — CDC verification + arc-store-home close endorsement

> **Verifies:** RH-5 · SH-6 (the arc-store-home dogfood close) · F-14/F-18/F-20/L-3a · **Landed:**
> `release/1.0.x` @ `deb816d` (linear) · **Date:** 2026-07-26 · **Verifier:** CDC, independent of CC —
> a clean container clone (git bundle of `release/1.0.x` + the orphan `odm` branch), store worktree
> materialized, built debug binary, **git 2.43** / cargo 1.95. Plus an **independent fresh-context
> arc-gate** (subagent, no prior exposure) on the whole arc.

## Verdict

**C-5 verified — the dogfood cutover is clean and correct; arc-store-home is closed.** odm now lives in
its own store, the corpus is intact and untouched-but-relocated-onto-the-orphan-branch, and the
derivation overhaul (fold + dates + names + vision) all landed. **PASS-WITH-NOTES** — the one note is a
**ledger-hygiene gap** the arc-gate caught (SH-7 left `open` + a phantom v1.6 changelog entry), which I
have now **reconciled** in `arc-plan.md`. No engineering defects. This is the arc's true close, and it
earns it.

**Process note:** C-5 landed **directly on `release/1.0.x`**, so this verification is post-land, not
pre-merge. Nothing I found requires a revert; had it, it would be a fix-forward. Consistent with the
rhythm the arc-store-home slices used (commit → CDC-verify), just on the release branch.

## Reproduced by CDC (clean clone, git 2.43)

| Claim | Result |
|-------|--------|
| **Dogfood state** | `odm.toml` is a **locator only** (no `[gates]`/`[display]` sections — the "operational half in config.toml" comment is the only match); working-branch `nodes/` **gone**; `.worktrees/odm` holds `config.toml` + `nodes/` with **60** `.md`. The `odm` branch is a true orphan (`merge-base odm release/1.0.x` → none). |
| **`check` cold** | `✓ check: ok (60 node(s), no problems)` from the repo root — resolution redirects into the home. |
| **Vision + focus (L-3a)** | `odm orient` renders the real VISION body (the project-plan §1 DoD text), not the "no vision text yet" placeholder; `odm use arc 1600` → `orient` CURRENT FOCUS shows arc #1600 — **defect 1 fixed** (use wrote the store root, orient now reads it too). |
| **Ids preserved (G-1-safe)** | `check` green ⇒ every `part_of`/edge resolves; 60 nodes, none minted. Nothing keyed on the unresolved id scheme. |
| **Nothing relocates** | all 60 files under `nodes/2026/07/` (the ULID-mint shard) despite `created` spanning back to 2025-12 — the shard is identity-derived, not date-derived. `moved=0` guard. **CC's correction to my brief is right** (my brief wrongly said relocate to the created-date shard). |
| **Dates (F-20)** | `created:` spans **2025-12-27 → 2026-07-25**, 14 distinct days (was 45× `2026-07-07`). |
| **Names (F-18)** | exactly **one** `(plan-of-record\|build plan)` hit — node **#15**, a `design` (document) node whose genuine H1 title is "…(build plan)". Zero work nodes carry a role suffix. CC flagged this one rather than corrupting a doc's title — correct. |
| **Config split** | `.worktrees/odm/config.toml` carries `docs_directory` + `[gates.*]` + `[display]`; `odm.toml` carries none of them. |
| **`.odm` gitignore (defects 2/3)** | store `.gitignore` = `/.odm/*` then `!/.odm/context.json`; `git -C .worktrees/odm ls-files .odm/` returns **only** `context.json` — caches ignored, focus committed so a fresh clone inherits it. |
| **Tests** | `odm-migrate` (the C-5 replan/re-stamp/mapping/selfhost code) **55 passed**; store suites `store_init`/`store_attach_sync`/`store_rename` **34 passed** post-cutover. (clippy/release-build not re-run — 2-min env timeout; CC reports clean, prior slices confirm the discipline.) |

## The three dogfood-found defects — endorsed

The cutover earned its keep by surfacing three bugs **no prior slice could have found**, because every
slice ran in a repo whose store *was* the invocation root — the very identity the cutover breaks:

1. **`use`/`orient` root mismatch** — `use` wrote the store root, `orient` read the invocation root;
   identical everywhere until odm's own store moved out of the invocation root. Fixed by deriving the
   path from the `Store` handle so no caller can pass the wrong one; regression tests run against a
   *redirected* store (CC strengthened them once, because asserting on the whole orient render passes
   with an empty focus — node names also appear under READY). Verified fixed.
2. **`store init` scaffolded no `.gitignore`** — the derived index was offered for commit.
3. **The first fix for (2) was wrong** — ignoring all of `.odm/` would withhold `context.json` from
   fresh clones, defeating the project's own success test. Now ignore-directory-plus-exception (a
   deny-list "fails open"). This is the sharpest of the three: the fix for a leak that would have
   *broken the DoD*, caught because the cold-sweep cache delete also deleted the focus. Verified: the
   exception ships.

## Independent fresh-context arc-gate — PASS-WITH-NOTES

A subagent with no prior exposure independently verified all six load-bearing dogfood claims against the
running binary and git (not the docs), and **endorsed both implementer judgment calls**:

- **(a) ULID-shard files are not relocated by corrected `created` dates** — sound, and it would have
  made the same call: the shard is an immutable function of identity, deliberately decoupled from
  mutable frontmatter, which is exactly the move-on-edit churn the design exists to kill. Relocating
  would break `Store::load` and make a *correctable* field load-bearing for storage. Endorse.
- **(b) shared `context.json`** — right v1.0 default (single-operator dogfood; directly discharges the
  fresh-session DoD; consistent with a shared, ff-only store branch). The contention risk CC flagged is
  real. **Recommended longer-term model (endorsed): split the one concept in two** — a *project focus*
  (deliberate, shared, committed: "the project is on arc X", what a newcomer needs) vs. a *working
  focus* (per-user, ephemeral, gitignored, e.g. `.odm/context.local.json`); `orient` shows the project
  focus and overlays a local one when set. Decouples the DoD from multi-operator contention. **Record as
  a tracked post-1.0 follow-up** (sibling to G-9 concurrent-session races), not a normalized assumption.

## The one finding — ledger hygiene (now reconciled)

The arc-gate's only real note: `arc-store-home/arc-plan.md` still showed **SH-7 `open`** and its version
history stopped at **v1.5** ("SH-6 stays open, pending RH C-5"), while the closing report already cited
"arc-plan v1.6: SH-6 attested" — a bump that never happened. Substance had landed (SH-6 → `attested`,
P-14 complete); the plan-of-record just wasn't reconciled. **Fixed by CDC:** SH-7 → `done`, and a v1.6
version-history entry recording the close (this verification + the arc-gate). The arc now *shows*
closed, matching what it *is*. (Bubble-up is normally the closer's; this was pure reconciliation the
verification surfaced, so I closed the loop and recorded that I did.)

## Ledger

- **RH-5 → attested; SH-6 → attested; SH-7 → done.** F-14/F-18/F-20/L-3a **done**; C-7 retired into C-5
  (the re-stamp-vs-fresh-derivation nuance recorded); **project-plan P-14 complete**.
- **arc-store-home is CLOSED** — code-complete + compose-verified + dogfood-reproduced + arc-gated. The
  only thing outstanding is the durable CI `reproduced` (cargo rows + both-git-arms matrix), which
  **rides the push** (origin is SSH, unreachable from the cloud session).
- **Silent-drop diff:** none. The shared-context.json follow-up + the F-16/F-17/carried store-group
  items are recorded, not dropped.
- **Next in RH:** C-4 (command reorg + ODD-0023), C-6 (check-hardening: G-2 + G-3 + L-3b), C-8 (F-19).
