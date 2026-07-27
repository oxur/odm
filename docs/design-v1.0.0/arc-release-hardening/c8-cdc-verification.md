# RH C-8 (normalized status) — CDC verification

> **Verifies:** RH-8 (C-8) · `F-19` (+ an unplanned **L-1** delivery) · **Landed:** `release/1.0.x`
> @ `db5cb31` (linear: `f34632f` → `db5cb31`) · **Date:** 2026-07-27 · **Verifier:** CDC, independent
> of CC — clean container clone, store worktree materialized, exercised on the self-hosted store.

## Verdict

**C-8 verified — clean, and it delivered more than F-19.** The normalized STATUS ships and reads as
intended; the comparability payoff reproduces. The important part is **CC's deviation, which was
correct and which I owe an admission on** — and which incidentally closes the LLM arc's #1 blocking
gap. No defects.

## The deviation — CC was right, my brief was wrong

My C-8 brief recommended "normalize `list`, omit any raw-gate view — `show`/`--json` already carry the
raw ladder." **They did not.** CC checked a real gated node before writing and found `show`/`--json`
reported **no gates in any form** — `list`'s STATUS column was the *only* place the ladder ever
surfaced. Normalizing it alone would have made the gate ladder **unreachable from the CLI entirely** —
a loss wearing the mask of a simplification. CC made my own acceptance text true instead (which
asserted `show`/`--json` expose the gate vector), rather than ship the trade. That is exactly the
call I'd want, and the brief was wrong to lean on an unverified claim.

**I need to name the pattern honestly, because it's the third instance.** My briefs keep asserting
*current-tool behaviour* from the design docs / memory without running the binary — and CC keeps
catching it by probing reality first: C-4 (the inventory documented a `store sync` that never existed),
C-5 (I said "relocate files to the created-date shard"; the shard is ULID-derived), and now C-8
(`show`/`--json` "already carry the ladder"; they carried nothing). The pull is consistent: I write
from the plan, not from the running surface. The fix is equally consistent — **any current-state claim
a brief leans on gets checked against the binary before it ships.** Logging it here so it's accountable,
not just noted.

## Reproduced by CDC

| Check | Result |
|-------|--------|
| **Normalized STATUS** | `odm node list` STATUS shows `planned`/`active`/`done` (CC's label triad; `planned` doubles as the `g0` gate name and the not-started state — coherent). The project reads `active`; done arcs and done slices both read **`done`** (green), an arc at `complete` reads `active` — the comparability payoff, verified on the real corpus. |
| **`show` exposes the ladder** | `odm node show 1600` → `status: active` + a `gates:` block: `[x] planned — 2026-07-07 (asserted)` / `[x] in-progress …` / `[ ] complete` / `[ ] verified` — reached-state, date, evidence level, full ladder. |
| **`--json` read-back** | `node show --json` gained `status` + a `gates` array of `{gate, reached, evidence}`. Machine-readable gate vector. |
| **Edge cases** | `--status done` → 50 rows (normalized vocabulary); `--status tested` → 38 rows (raw gate) — both work, as designed. `adr` (no gate-set) → `—` not `planned`, and `--json` omits the field rather than emitting the em-dash (unit-tested; the live corpus has no `adr` node to show it, but the derivation is sound). |
| **Comparability as an assertion** | 11 unit tests over the real §5.1 ladders, including an `assert_eq!` that the two "done" spellings (`tested`, `verified`) resolve equal — comparability is a test, not a comment. |
| **No model change** | `nodes/**` untouched; `validate` green at 60. |
| **Build** | odm-cli 36 + odm-core 61 + store/migrate suites — all 0 failed in my clone; CC reports 58 binaries / 0 failed, clippy `-D warnings` clean, fmt clean, no `unsafe`. |

## The finding that matters — C-8 delivered **L-1**, the LLM arc's blocking gap

Because CC had to expose the gate vector in `show`/`--json` to keep C-8 from being a loss, **C-8
delivered the LLM-command-surface arc's #1 finding, L-1 (= G-4, that arc's slice 01):** *"Status is
write-only — multi-gate status vectors are odm's central innovation and no command reads one back."*
L-1's own proposal was verbatim what shipped: *"add `status` to `show` (text + `--json`), including
per-gate reached/evidence."* It is done. This is real over-delivery and it **re-scopes the LLM arc** —
its designated blocking slice is now largely satisfied ahead of schedule. Worth a deliberate update to
that arc's plan rather than leaving it to be re-discovered.

**What of L-1 remains (small, for the LLM arc, not C-8 defects):**
- `--json`'s `gates` array lists only **reached** gates; the text `show` lists the unreached rungs too
  (`[ ] complete`/`[ ] verified`). A machine consumer can't see "next gate is `complete`" from `--json`
  alone. Enumerating unreached gates (`reached: null`) would complete the read-back — fits the LLM
  arc's `--json`-contract work (with F-21 dates).
- L-1 also wanted the **computed satisfaction verdict** and, for a soft-satisfied dependency, the
  **weakest link**. That's the readiness/`blocked` analysis (needs the graph) — deferred, correctly not
  in a display-only chunk.

## One thing for your decision — the retired colour distinction

CC flags that the **per-gate colour palette retired with the raw STATUS column** (nothing could select
it once the column normalized), and with it the `complete`-green vs `verified`-bright-green distinction
you'd tuned. It now survives only as plain text in `show`'s ladder. Non-blocking and defensible (the
normalized column is the point), but if you want that gradient back on screen, the slots still exist and
`show`'s ladder is where it'd live now. **Your call** — leave it plain, or a small follow-up to colour
the ladder.

## Ledger

- **RH-8 (C-8) → attested** (CDC-reproduced on `release/1.0.x`; durable `reproduced` rides the push +
  CI). **F-19 done.** **Silent-drop diff:** none.
- **Cross-arc:** **L-1 / LLM-arc slice 01 (G-4) is largely delivered by C-8** — record it in the LLM
  arc's plan; the remainder is the `--json` unreached-gate enumeration + the satisfaction/weakest-link
  analysis.
- **RH remaining:** **C-6** (check-hardening: G-2 tear-rationale + G-3 + L-3b) — the last chunk before
  RH-6/RH-7 compose + arc close.
