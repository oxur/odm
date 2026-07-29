---
id: 01KYP5FZ0NXS7BDPRH00CCS45F
number: 569547400
type: artifact
schema: artifact/v1.1
name: C-2 — CDC verification (chunk close)
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/C-2-cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# C-2 — CDC verification (chunk close)

> **Chunk:** RH C-2 — type taxonomy (`odd`→`design` + `research`) · **Verifies:** RH-2 ·
> **Covers:** F-2, F-3 · **Branch:** `rh-c2-type-taxonomy` (`cf4cf87`, off the C-1 tip `dc65082`)
> **Date:** 2026-07-26 · **Verifier:** CDC, independent of CC's implementation. Structural rows
> reproduced by corpus + code inspection; executable rows attested-by-CC and operator-confirmed
> (`odm check` + `make check` green), reproduced-on-CI.

## Verdict

**C-2 delivered; RH-2 `attested` confirmed.** Independent structural verification passes on every
acceptance criterion. Two disclosed items carried (both already flagged by CC), one recommended
for an F-number. Nothing blocks the close; CI is the only gate remaining for `reproduced`.

## Independent checks (reproduced by CDC)

| Check | Method | Result |
|-------|--------|--------|
| Type counts | frontmatter-only scan of `nodes/**` | **9 `design` + 5 `research` + 46 work = 60** ✓ (matches CC) |
| Zero `odd` in frontmatter | `grep '^type: odd'` / `'^schema: odd'` | **0 / 0** ✓ |
| Type↔schema pairing | per-node: `schema` startswith `type + "/"` | **0 mismatches / 60** ✓ |
| Research identity preserved | look up the 4 pre-existing research ULIDs | all 4 now `type: research`, **ULIDs unchanged** ✓ |
| Code | `node_type.rs` (`Odd`→`Design` + `Research`, `as_str`/`FromStr`), `mapping.rs` tag-classification, `restamp.rs` present, `RESEARCH_GATES` a named constant | ✓ |
| Config | `odm.toml`: `[gates.design]` + `[gates.research]`, no `[gates.odd]` | ✓ |
| Amendments landed first | ODD-0013 §2.2 + classification rule; ODD-0020 marker set `…design/v1.0, research/v1.0…` | ✓ (model-first order honoured) |
| Executable | build/test/clippy `-D warnings`/fmt; `odm check`; `make check` | green — CC-attested + **operator-confirmed** |

### Count note (a positive cross-check, not a discrepancy)

A raw `grep '^type:' nodes` reads **61**, one over the true **60**. The extra is a *body* line
`type: slice` inside the design node whose body is ODD-0013 §2.2's frontmatter example. The
frontmatter-only scan (stop at the first `---`) correctly reads 60. This false positive is itself
evidence that **re-stamp preserved node bodies verbatim** — which leads to the one finding worth
elevating.

## Findings / carried items

1. **Body-snapshot drift — recommend logging (F-17 candidate; already disclosed in
   `c2-closing-report.md`).** Re-stamp rewrote only the two dead frontmatter lines; node **bodies
   are import-time snapshots and are not refreshed**. Concretely: node #20's body still contains
   the *pre-amendment* `odd/v1.0` marker text, while its source doc (ODD-0020) now reads
   `design/v1.0`. This is arguably by-design (the source doc is the editable truth; the node is
   the build), but it is a **decision, not an oversight**, and it bears on the project DoD — a
   fresh context doing `odm show <node>` sees the snapshot, not the live source. **Recommend:**
   log as F-17 and decide explicitly — *snapshot-by-design (document it) vs. body-refresh on
   re-migrate*. Not release-blocking. (Flagging, not minting, per the F-15 collision lesson —
   your call on the number.)
2. **Node #21 imported — disclosed over-delivery, not drift.** C-2 brought a 14th document node
   (`0021`, research) into the corpus that predated C-2 but wasn't migrated. Reasonable inclusion,
   recorded by CC; noted here as scope-add-with-rationale, not a silent change.
3. **`restamp` bootstrap — sound, endorsed.** The no-alias decision meant `load_all` couldn't
   parse the very nodes the migration exists to fix; the single-purpose `odm-migrate::restamp`
   (rewrite the two dead lines → hand to the typed path, with a dry-run in-memory overlay)
   is the right minimal unblock. CC rehearsed it on a scratch copy of the real corpus first. It is
   single-use and droppable once every corpus has passed through it — a disclosed transient.

## Ledger disposition

- **RH-2 → `attested` confirmed** (independent structural verify + operator-confirmed check/build).
  Verify pointer should reference **this doc** alongside `c2-closing-report.md` (closer ≠ verifier).
  Flips `reproduced` on CI.
- **F-2, F-3 → done** (verified: `odm list` TYPE column shows `design`/`research`, never `odd`).
- Earlier work intact: arc-plan carries v1.4 (CDC reconciliation) + v1.5 (C-2); the dashboard's
  C-1-closed state committed as `dc65082`. Nothing lost across the branch cut.

## Remaining to flip RH-2 → `reproduced`/`done`

CI green on `rh-c2-type-taxonomy`. Then C-3 (`list` overhaul) is unblocked — settled type names,
and the home for **F-15** (default-exclude retired nodes + status column).
