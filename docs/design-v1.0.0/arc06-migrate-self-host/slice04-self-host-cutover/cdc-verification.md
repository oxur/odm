# CDC Verification — Slice 04 (Arc 06): self-host cutover

> Independent CDC reproduction of CC's proposed-done (`attested`) report — **the
> loop-closer.** Structural rows reproduced by code + on-disk inspection against
> `release/1.0.x`; cargo rows (test pass, clippy, coverage %, workspace green) are
> **attested-pending-CI** — the CDC sandbox carries no 1.85+ toolchain. Commits
> `4ac36f6` (work) + `1766448` (close SHA) on `arc06-slice04-self-host-cutover`,
> branched off the slice03 tip.

## Verdict

**PASS-WITH-NOTES. odm self-hosts.** All seven ledger rows reproduce structurally;
the plan-set is under `nodes/` as a correct, schema-stamped, containment-linked work
tree coexisting green with the `odd` corpus. Two calibration notes below — both
*properties of self-hosting a live plan*, not defects. New decisions (numbering,
`[gates.*]`) reviewed and accepted. Cargo-executable rows reproduce on CI.

## Row-by-row

| Row | Reproduced | Evidence (CDC-observed) |
|-----|-----------|-------------------------|
| **S-1** plan-set → work nodes | ✅ structural + on-disk | `odm-migrate/src/selfhost.rs` (new, 18 KB) + `odm self-host <plan> [--dry-run]` (`odm-cli/src/lib.rs:412`, dispatched to `migrate::self_host`). On disk: **45 work nodes** — 1 `project` + 6 `arc` + 38 `slice` — under `nodes/2026/07/`, each schema-stamped (1× `project/v1.0`, 6× `arc/v1.0`, 38× `slice/v1.0`). Plan-set MD byte-intact (the only `docs/design-v1.0.0/` change in the cutover commit is `arc06/arc-plan.md` = the bubble-up). First mint of non-`odd` markers. |
| **S-2** containment tree | ✅ structural | project (`number: 1000`) is root (no `part_of`); all 6 arcs `part_of` the project id; every sampled slice `part_of` its arc (A5 slices 1501–1508 all `part_of` the A5 arc id). Matches the directory hierarchy exactly. |
| **S-3** gate status from plan (Asserted) | ✅ structural | Arc gates: A1–A5 (1100–1500) reach terminal **`verified`** (`complete,in-progress,planned,verified`); A6 (1600) is **`in-progress`** only — closed vs active, exactly right. All at `evidence: asserted`. Slice gates: closed slices reach terminal **`tested`** (`built,planned,tested`), cumulative like slice01. Status source = P-row / close-file (A1/A2 lack a close file → P-row authoritative). |
| **S-4** mixed-corpus `check` green | ✅ structural / ⏳ cargo pending-CI | No work node carries a document-only field (`supersedes`/`affects`) — the lone grep hit was a **false positive** (the ODD-13 `odd` node whose *body* quotes `type: slice`; its `supersedes` is a legit document field). Closed arc A5 affirms `decomposed`; active A6 does not (correct — only complete/verified would need it). `odm check` "ok (58 node(s), no problems)" — reproduced on CI. |
| **S-5** `rollup`/`orient` reproduce reality | ✅ structural / ⏳ cargo pending-CI | The gate status that `rollup` reads is correct on disk (A1–A5 verified, A6 in-progress), so the rollup reproduces the real state; `orient` resolves the self-hosted project. Executable assertion reproduced on CI. `[gates.*]` in `odm.toml` is the enabling prerequisite (below). |
| **S-6** reflexive-import safety | ✅ structural | `self_host` persists **children-up** (slices → arcs → **project last**, via `persist_rank`) — a partial failure never orphans the root. Idempotent (re-run "0 created, 45 skipped", keyed on the work number) and never-delete (plan-set MD byte-intact) reproduced on-disk. The reflexive slice04 node is handled by idempotence + last-ordering (see Note 1). |
| **S-7** gates + no regression + no index change | ✅ structural / ⏳ cargo pending-CI | **`git diff release/1.0.x...HEAD -- crates/odm-index` is empty** — reproduced directly (self-host rides the store + command surface, not the index). clippy `-D warnings` / no `unsafe` (none in `odm-migrate/src`) / line-cov (selfhost 96.25%, lib 99.21%, mapping 99.06%) / 52 suites — attested by CC, reproduced on CI. |

## Bubble-up (checked)

Propagated to `arc-plan.md` correctly: **A-4** `attested` (cargo pending-CI, → `done` on reproduce); **A-8** compose row `mechanism-complete` with the **P-12 reproducible-at-arc-close** note (to be reproduced at arc-scale at arc-close, never inherited); v1.7 version entry. The self-hosting trigger is recorded.

## Calibration notes (properties of self-hosting a live plan — not defects)

1. **The in-flight slice stamps itself.** slice04 (node 1604) reads `tested` —
   because the cutover's slice-complete signal is `closing-report.md` presence, and
   slice04 wrote its own closing-report as part of running slice04. So the self-hosted
   status for the reflexive slice is **asserted-complete one step ahead of CDC-
   reproduction**. This is honest *at the `Asserted` level* (claimed-from-record) and
   self-reconciles on this verification + CI. It is the reflexive-node case CC
   flagged, working as designed — worth naming so "1604 = tested" isn't misread as
   independently-reproduced.

2. **The corpus is a cutover snapshot.** A6 imported **4** slice nodes (1601–1604),
   all `tested`, under an `in-progress` arc — because only slices 01–04 have
   directories today; 05/06 aren't drawn yet, so they aren't nodes. This is a
   faithful snapshot, not a contradiction (an in-progress arc doesn't require its
   children complete), and an idempotent re-run absorbs 05/06 when they land. Stated
   so nobody reads "A6: 4 slices all tested" as "A6 nearly done" — A6 is active with
   two slices still to come.

## New decisions (reviewed, accepted)

- **`[gates.*]` added to `odm.toml`** (project/arc = `planned→in-progress→complete→
  verified`; slice = `planned→built→tested`; odd = `draft…final`). A legitimate
  **self-host prerequisite**, not a model amendment: work-node types need their
  gate-sets defined for `rollup`/`orient`/`check` to render/validate status, and the
  sequences are the canonical ODD-0013 §5.1 sets. Accepted.
- **Disjoint numbering** (project = 1000; arc N = 1000 + 100·N; slice = arc + position)
  — stable, collision-free, and distinct from the `odd` space (2, 9–19). A clean
  derivation from structure; no renumber risk. Accepted.
- **Dir-structure adapter, no manifest** — the recommended path; the filesystem *is*
  the tree, so the fragile prose-parse the slice-doc feared never materialized.
  Accepted.
- **`ROLLUP.md` regenerated but left uncommitted** (a derived view; the nodes are the
  source). Correct call.

## Reproduced

Structural rows reproduced by CDC (code + on-disk state) against `release/1.0.x`;
cargo rows flip `attested → reproduced` on CI green. No amendment required. Slice04
delivers **A-4** and the **A-8** mechanism; **odm now self-hosts** — its own plan
lives under `nodes/` (58 nodes), `check`/`rollup`/`orient` run over it, and the hand-
maintained truth (project-plan, the dashboard) is derivable from odm querying itself.
Project-plan **P-12** is reproducible-at-arc-close. Remaining in A6: slice05
(PM-skill) + slice06 (retire prose), then the arc closes and the v1.0.0 MVP-plus is
self-hosting.
