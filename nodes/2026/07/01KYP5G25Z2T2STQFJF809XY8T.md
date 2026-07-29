---
id: 01KYP5G25Z2T2STQFJF809XY8T
number: 545027800
type: artifact
schema: artifact/v1.1
name: 'cc-prompt — RH C-5 (expanded): self-host → migrate + node-derivation overhaul'
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/cc-prompt-c5-selfhost-derivation.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# cc-prompt — RH C-5 (expanded): self-host → migrate + node-derivation overhaul

> **Arc:** Release Hardening (UAT) · **Chunk:** C-5 (expanded) · **Covers:** `F-14` (fold),
> **`F-18`** (names), **`F-20`** (dates) · **Kind:** surface/medium + model + data.
> **Supersedes** the standalone `cc-prompt-c7-name-normalization.md` — its work folds in here so
> the corpus is regenerated **once, not three times**. The `C-7-amendment-ODD-0013.md` (§2.1)
> **stays** — it's the model change and lands first. **C-6 (G-2 tear-rationale)** and **F-21
> (`--json` dates → LLM-command-surface arc)** are **not** in this chunk.

## Why combined

`F-14` (fold `self-host` into `migrate`), `F-18` (clean names), and `F-20` (real git dates) all
touch the **same** work-node derivation path and each would otherwise trigger its own re-stamp of
the 46 existing nodes. Done together: **one reworked import path, one re-stamp**. Separately: two
or three corpus rewrites, more drift surface. This chunk is the "self-host derivation" unit.

## Ordering (this is the point — follow it)

**1. Model first.** Land `C-7-amendment-ODD-0013.md` — §2.1 "names embed no metadata" (v2.2) —
before any code. Dates need **no schema change** (`created`/`updated` already exist; F-20 is a
*derivation* fix, not a new field); add a one-line note in `selfhost` docs that dates now derive
from git, superseding the old "history lives in git" comment.

**2. One reworked derivation path.** Consolidate `self-host` into `migrate` (F-14 — e.g.
`migrate --plan` / autodetect, one verb), and in that single derivation:
- **Names (F-18):** `normalize_name` strips the `"<Type> NN —"` number-prefix and the known
  role-suffixes (`(plan-of-record)`, `(build plan)`); preserve genuine descriptive parentheticals.
  Shared by the work-node and doc-node paths.
- **Dates (F-20):** `created` = the **earliest git add-date** of the node's plan directory
  (`git log --diff-filter=A --reverse -- <path>`); `updated` = its **latest** commit date;
  fall back to `today()` only for an untracked path (disclose the fallback). Per-slice dirs give
  per-node granularity (arc01/slice01 = 2026-06-20 … arc05/slice08 = 2026-07-06). Nodes with no
  dedicated dir fall back to their arc's date.

**3. One re-stamp.** A re-run alone **won't repair** the 46 existing nodes — `self-host` skips on
`(type, number)` (the two-part shape C-2 hit). Add an explicit re-stamp pass (reuse/extend the
C-2 `restamp` pattern) that rewrites **names + dates together** across all work nodes in one
sweep. Diff should touch only `name:` / `created:` / `updated:` lines — ids, numbers, gates,
edges, bodies unchanged.

**4. Verify** (below), once.

## Acceptance / ledger

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- **Names:** `grep -rE "^name:.*\(plan-of-record\)" nodes` → empty; no number-prefix/role-suffix.
- **Dates:** work-node `created` spans **2026-06-20 → 2026-07-25** (not 45× `2026-07-07`);
  `updated` reflects real last-touch; `migrate`/doc nodes unchanged (verify #13 still 06-20/06-26).
- **Fold:** `self-host`'s behaviour is reachable through `migrate`; one verb.
- **Single rewrite:** the re-stamp is one commit/diff; `odm check` green; node counts unchanged.
- ODD-0013 §2.1 at v2.2 landed **before** the code. F-14/F-18/F-20 dispositioned in the bubble-up.

## Decisions to confirm at kickoff

1. **Fold shape** — `migrate --plan` explicit flag vs autodetect plan-set-vs-legacy (F-14).
2. **Name strip-list** — the exact role-suffix set (`(plan-of-record)`, `(build plan)`, …) and
   "metadata not qualifiers" boundary (from C-7).
3. **Date granularity/fallback** — per-dir git add-date + the arc-date fallback for source-less
   nodes; whether a future plan-doc `created:` frontmatter should override git (hybrid).

## Method / housekeeping

- One branch (e.g. `rh-c5-selfhost-derivation`, off the latest RH tip); one mergeable diff; one
  re-stamp; five-iteration cap. CC implements on local 1.85+; cargo rows → CI.
- Arc-plan reconciliation (do when the working tree is clear of concurrent edits): the Chunks
  table should show C-5 expanded (absorbs C-7), C-6 (G-2), C-8 (F-19); the standalone C-7 row
  retires into C-5; F-14/F-18/F-20 point here.
