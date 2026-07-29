---
id: 01KYP5FZ5R6VCX8FPRFZ1WN4G3
number: 590537700
type: artifact
schema: artifact/v1.1
name: C-3 — CDC verification (chunk close)
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/C-3-cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# C-3 — CDC verification (chunk close)

> **Chunk:** RH C-3 — `odm list` overhaul · **Verifies:** RH-3 · **Covers:** F-4…F-9 + F-15 ·
> **Branch:** `rh-c3-list-overhaul` (`01c5767`, off the C-2 tip `c9d6fd5`) · **Date:** 2026-07-26
> **Verifier:** CDC, independent of CC. Structural rows reproduced from the committed capture +
> code; cargo rows attested-by-CC → reproduced-on-CI.

## Verdict

**C-3 delivered; RH-3 `attested` confirmed.** Every finding reproduces against the capture and
the source, the index format bump is sound, and the F-15 audit correction is **independently
re-verified**. No defects; two noted items (both already disclosed by CC); CI is the only gate
left for `reproduced`.

## Independent checks (reproduced by CDC)

| Check | Method | Result |
|-------|--------|--------|
| Columns (F-4/F-5/F-7) | `c3-capture` + `LIST_COLUMNS` | `DATE \| TYPE \| STATUS \| NAME \| ID`, **no NUMBER**; STATUS = furthest-reached gate ✓ |
| Tree (F-8) | capture | containment tree (`project → arc → slice` via `part_of`, `├─/└─/│` glyphs); **doc nodes in a separate `documents` group** ✓ |
| De-number (F-6) | capture | names de-numbered on display (display-only; convention added) ✓ |
| Width/elision (F-9) | code + `--all --width 44` capture | `[display] max_width` + `--width`, elision with ` ...` ✓ |
| Retired filter (F-15) | capture default vs `--all` | default **omits #1605** ("59 shown — 1 filtered"); `--all` shows it `retired`, dimmed ✓ |
| Index bump | `snapshot.rs` | `FORMAT_VERSION = 4`; written + checked; mismatch → rebuild self-heal (`found != FORMAT_VERSION`) ✓ |
| `check` unchanged | CC-attested | green, 60 nodes (a view change moves no counts) |
| Tests | build | +15 tests green (per-finding coverage) |

## F-15 audit correction — independently reproduced

CC corrected the prompt's "~5 superseded ODDs" to **zero**. I reproduced it precisely:

- **Retired nodes:** exactly **1** — node #1605 (the L-2 tombstone). ✓
- **Active `supersedes` edges:** **0**. The only `supersedes:` frontmatter key in the corpus is
  `supersedes: null` (node #13, a schema-comment default). The crude "45 supersede matches" are
  **prose** (design-doc bodies: "supersede, don't delete", "CRUD (…/supersede)", #1605's retired
  reason). So superseded-exclusion ships tested **against fixtures only**, never live data — a
  correct, disclosed limitation, not a gap.

My original F-15 row already caveated "only 1605 confirmed structurally"; CC's audit closed the
caveat and this pass confirms it. Good disposition — recorded, not quietly dropped.

## Noted (already disclosed; not defects)

1. **`(plan-of-record)` name suffix.** Displayed names still carry a `(plan-of-record)` suffix.
   Correctly **outside F-6** (not a number-reference). Cosmetic; removing it is a data change for
   a re-self-host, not a display rule. Candidate low-priority cleanup (or an F-18 if you want it
   tracked) — flagging, not minting.
2. **Transparency wins worth keeping.** The view **reports what it withheld** ("N filtered or
   withdrawn") — the same L-2 silent-omission hazard that *raised* F-15, now designed against in
   F-15's own fix. And a node whose parent is filtered becomes a **root** of that view (`--type
   slice` renders flat rather than vanishing). Both are the right calls.

## Ledger disposition

- **RH-3 → `attested` confirmed** (independent capture + code verification). Verify pointer → this
  doc alongside `c3-closing-report.md` (closer ≠ verifier). Flips `reproduced` on CI.
- **F-4…F-9 + F-15 → done.**
- **Remaining:** C-4 (command surface), C-5 (fold `self-host` into `migrate`); **F-16** (upstream
  `oxur-term` TTY guard) and **F-17** (body-snapshot drift) still open decisions.
