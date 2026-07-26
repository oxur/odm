# cc-prompt — RH C-8: Normalized status for `odm list` (display-only)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-8 · **Covers:** `F-19` · **Kind:** surface,
> **display-only** · **Depends on:** C-3 (STATUS column + the `list` render path).
> **Chunk number:** C-8 — C-6 (G-2 tear-rationale) and C-7 (F-18 names) are taken; no collision.

## Goal

Make `odm list`'s STATUS **comparable across node types** by rendering a normalized state derived
from each node's *position in its own gate ladder*, instead of the raw furthest-reached gate.
Today a slice's `tested` and an arc's `verified` both mean *done* but read differently, and an
arc's `complete` reads like an endpoint though `verified` is still ahead. **No model change** —
this is a pure render-time derivation; gates, evidence, and `--json`'s gate vector are untouched.

## The derivation

For a node with gate-set `G = [g0 … gN]` (`gN` = terminal) and a set of reached gates:

- reached **`gN`** (terminal) → **`done`**
- reached some `gi`, `0 < i < N` (past the first rung, not terminal) → **`in-progress`**
- reached only `g0` (`planned`) or nothing → **`not-started`**
- **overlay (takes precedence):** node has a `retired:` block / is withdrawn → **`retired`**
  (preserve C-3/F-15 behaviour exactly)
- **optional (flag at kickoff):** node is blocked by an unsatisfied dependency → **`blocked`**.
  This needs the graph, not just gate position — **defer unless cheap**; `not-started`/`in-progress`
  is fine for v1 and `blocked` can be a follow-up.

Worked examples: slice at `tested` → `done`; slice at `built` → `in-progress`; arc at `verified`
→ `done`; arc at `complete` → `in-progress`; the project at `in-progress` → `in-progress`;
node #1605 → `retired`.

## Decisions to confirm at kickoff

1. **State labels** — recommend `not-started` / `in-progress` / `done` (clear doneness triad).
   Alternatives: `planned/active/done`, `todo/doing/done`. Pick one.
2. **`blocked`** — in scope now (needs the graph) or deferred? Recommend deferred.
3. **Column behaviour** — recommend `list` STATUS shows the normalized state by default, with an
   optional `--status=gate` to fall back to the raw gate name. `show` and `--json` **always** keep
   the full gate vector + furthest-reached gate; `--json` may additionally emit the derived state
   as a convenience field (machines get both).

## Changes

- One render-time helper (e.g. `derive_display_status(reached, gate_set, retired) -> DisplayStatus`)
  used by the `list` STATUS cell. Colour/glyph it on the C-1 `oxur-term` theme: `done` green (✓),
  `in-progress` amber (◐), `not-started` dim (○), `retired` dim. Keep it a *view* concept — do not
  add a stored field or a gate.
- Unit-test the helper directly: terminal→`done`, mid→`in-progress`, planned-only/none→`not-started`,
  retired overlay wins, one case per node type.

## Acceptance / ledger

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- `odm list`: a done slice (`tested`) and a done arc (`verified`) **both read `done`**; an arc at
  `complete` reads `in-progress`; the project reads `in-progress`; #1605 reads `retired` under `--all`.
- **No model change:** `git diff` touches no `nodes/**` frontmatter; `odm check` green; counts
  unchanged. `show`/`--json` still expose the raw gate vector.
- F-19 dispositioned in the bubble-up; record the label + `blocked` + column-flag decisions.

## Method / housekeeping

- One branch (e.g. `rh-c8-normalized-status`, off the C-3 tip); one mergeable diff; five-iteration
  cap. Display-only, so it should be small. CC implements on local 1.85+; cargo rows
  attested-by-CC → reproduced-on-CI.
- Add C-6/C-7/C-8 to the arc-plan Chunks table when the pipeline is next tidied, so it stays
  authoritative (they currently live in the Findings Log + cc-prompts).
