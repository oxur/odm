# cc-prompt — RH C-8: Normalized status for `odm node list` (display-only)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-8 · **Covers:** `F-19` · **Kind:** surface,
> **display-only** · **Depends on:** C-3 (STATUS column + `list` render path) and **C-4** (the reorg —
> `list` is now **`odm node list`**). **Refreshed 2026-07-27** for the post-C-4 surface + the `--status`
> filter that F-15 already shipped (see §Interaction). Supersedes the pre-reorg draft.

## Goal

Make the STATUS column **comparable across node types** by rendering a normalized state derived from
each node's *position in its own gate ladder*, instead of the raw furthest-reached gate. Today a slice's
`tested` and an arc's `verified` both mean *done* but read differently, and an arc's `complete` reads
like an endpoint though `verified` is still ahead. **No model change** — a pure render-time derivation;
gates, evidence, and the `--json` gate vector are untouched.

## The derivation

For a node with gate-set `G = [g0 … gN]` (`gN` = terminal) and a set of reached gates:

- reached **`gN`** (terminal) → **`done`**
- reached some `gi`, `0 < i < N` (past the first rung, not terminal) → **`in-progress`**
- reached only `g0` (`planned`) or nothing → **`not-started`**
- **overlay (takes precedence):** a `retired:` block / withdrawn node → **`retired`** (preserve
  C-3/F-15 behaviour exactly); a superseded node → **`superseded`**.
- **optional (defer):** blocked-by an unsatisfied dependency → **`blocked`** — needs the graph, not
  just gate position. `not-started`/`in-progress` is fine for v1; `blocked` is a follow-up.

Worked examples: slice at `tested` → `done`; slice at `built` → `in-progress`; arc at `verified` →
`done`; arc at `complete` → **`in-progress`** (the point of the chunk — `complete` is not the endpoint);
project at `in-progress` → `in-progress`; #1605 → `retired`.

## Interaction to get right (new since the pre-reorg draft) — the `--status` flag

**F-15 already shipped `--status <VALUE>` as a *filter*** on `odm node list` (`--status retired` = only
the retired rows). So the old draft's idea of `--status=gate` to switch the *display* back to raw gates
**collides** — don't reuse `--status` for that. Two clean calls, both recommended:

1. **Filter accepts normalized values.** With the normalized state now the default display, `--status
   done` / `--status in-progress` / `--status not-started` should filter on the **derived** state
   (plus the existing `retired`/`superseded` overlays). Keeping a raw-gate filter too (`--status
   tested`) is fine if cheap, but the headline is that the filter vocabulary matches what the column
   now shows — otherwise `--status` and the STATUS cell would disagree.
2. **Raw-gate display is a *separate* flag**, not `--status`. If a raw-gate view is wanted at all,
   name it `--status-format={state|gate}` (default `state`). **Recommend deferring even that** — `show`
   and `--json` already carry the full gate vector for anyone who needs the raw ladder, so `list` can
   simply show the normalized state, full stop. Decide at kickoff.

## Changes

- One render-time helper (e.g. `derive_display_status(reached, gate_set, retired, superseded) ->
  DisplayStatus`) used by the `node list` STATUS cell. Colour/glyph on the C-1 `oxur-term` theme:
  `done` green (✓), `in-progress` amber (◐), `not-started` dim (○), `retired`/`superseded` dim. Keep it
  a **view** concept — no stored field, no gate.
- Wire the `--status` filter to the derived vocabulary (§Interaction #1).
- `show` and `--json` keep the **raw gate vector + furthest-reached gate** unchanged; `--json` **may**
  add the derived state as a convenience field so machines get both (recommend yes — it's the
  comparable signal an LLM consumer wants, and cheap).

## Decisions to confirm at kickoff

1. **State labels** — recommend `not-started` / `in-progress` / `done` (clear doneness triad).
   Alternatives: `planned/active/done`, `todo/doing/done`.
2. **`--status` filter vocabulary** — filter on the normalized state (recommended), and whether to also
   keep raw-gate filtering.
3. **Raw-gate display flag** — omit entirely (recommended; `show`/`--json` cover it) vs. a
   `--status-format=gate`.
4. **`blocked` overlay** — deferred (recommended) vs. in scope now (needs the graph).
5. **`--json` convenience field** — emit the derived state alongside the raw vector (recommended yes).

## Acceptance / ledger

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- `odm node list`: a done slice (`tested`) and a done arc (`verified`) **both read `done`**; an arc at
  `complete` reads `in-progress`; the project reads `in-progress`; #1605 reads `retired` under `--all`.
- `--status done` filters to the derived-done rows and agrees with the STATUS column (no filter/column
  mismatch).
- **No model change:** `git diff` touches no `nodes/**` frontmatter; `odm check` green; counts
  unchanged. `show`/`--json` still expose the raw gate vector (+ the optional derived field).
- Unit-test the helper directly: terminal→`done`, mid→`in-progress`, planned-only/none→`not-started`,
  retired/superseded overlays win, one case per node type.
- **F-19 dispositioned** in the bubble-up (RH-8); record the label + `--status`-vocabulary + `blocked`
  decisions.

## Method / housekeeping

- One branch (e.g. `rh-c8-normalized-status`, off the **C-4 tip on `release/1.0.x`**); one mergeable
  diff; five-iteration cap. Display-only, so small. CC on local 1.85+; cargo rows → CI. Reflexive check:
  `odm node list` on the **self-hosted store** reads sensibly across project/arc/slice/design rows.
- The arc-plan Chunks table already carries C-8 (added in v1.12) — just flip it done on close. CDC
  verifies (`cdc-verification.md`) on the real store: the cross-type comparability (a done slice and a
  done arc read alike) + no frontmatter touched.
- **Not in this chunk:** C-6 (check-hardening: G-2 + G-3 + L-3b). C-8 is display-only and independent.
