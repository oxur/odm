# cc-prompt — RH C-2: Type taxonomy (`odd`→`design` + `research`)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-2 · **Covers:** `F-2`, `F-3` · **Kind:** model
> (amend first) · **Feeds:** RH-2. **Foundational — gates C-3** (which needs the settled type
> names + the status column, and lands F-15's retired-node filter).
> **Amend before you code** (RH arc-plan §Amendments): apply the **ODD-0013** and **ODD-0020**
> amendment stubs (`C-2-amendment-ODD-0013.md`, `C-2-amendment-ODD-0020.md`) first — they carry
> the design decisions this prompt implements.

## Goal

Rename the document node type **`odd` → `design`**, add a **`research`** type, and re-stamp the
self-hosted corpus so `odm list` shows `design`/`research` (never `odd`) — with gate-sets, schema
markers, and the migrate mapping all consistent, and `odm check` green.

## One decision to confirm at start

**`research` gate-set:** mirror `design` (recommended, zero migration churn — see the ODD-0013
amendment rationale) unless the operator opts for a tighter research lifecycle. Default: mirror.

## Changes

### Code (`odm-core`, `odm-migrate`)

1. **`crates/odm-core/src/node_type.rs`** — rename variant `Odd` → `Design`; add `Research`.
   Update every arm: `as_str` (`"odd"`→`"design"`, add `"research"`), `FromStr`
   (`"odd"`→`Design`, add `"research"`→`Research`), serde, and the doc comments. Decide the
   legacy `"odd"` alias per the ODD-0020 amendment — **recommend NO alias** (hard re-stamp), so
   `"odd"` returns the normal parse error after C-2.
2. **`crates/odm-core/src/schema.rs`** — the marker test at ~line 153 and any `odd`-referencing
   schema logic → `design`; ensure `design`/`research` are valid per-type markers.
3. **`crates/odm-migrate/src/mapping.rs`** — `build_node` (currently hardcodes `NodeType::Odd`,
   ~line 136) classifies **`research` iff source `tags` include `research`, else `design`**, and
   selects the matching gate-set for the cumulative reach (pass `design_gates` / `research_gates`
   instead of the single `odd_gates`). Rename the `ODD_GATES` const → `DESIGN_GATES` (+ add
   `RESEARCH_GATES`).
4. Sweep for stragglers: `grep -rn 'NodeType::Odd\|"odd"\|ODD_GATES\|odd_gates' crates` → none
   left outside legacy-alias handling (if any).

### Config (`odm.toml`)

5. `[gates.odd]` → `[gates.design]` (same sequence); add `[gates.research]` (mirror `design`).

### Data cleanup (source docs — required for correct classification)

6. Fix placeholder `tags: [change-me]`:
   - **`0011-research-…md`** → tags including **`research`** (else it misclassifies as `design`).
   - **`0012-…project-definition…md`** → design-appropriate tags (non-critical, but fix it).
   Confirm the research set resolves to exactly **0011, 0014, 0016, 0018**; the other 9 → `design`.

### Re-stamp the corpus (self-hosting is the migration)

7. Re-run **`odm migrate`** (the doc nodes) and **`odm self-host`** (the work nodes) so the 13
   document nodes regenerate as **9 `design` + 4 `research`**, each with `schema: design/v1.0`
   or `research/v1.0`. If migrate does not update type/schema in place on re-run, add the
   in-place re-stamp (supersede-not-delete already governs this path). Verify **no `odd` type or
   `odd/v1.0` marker remains** anywhere under `nodes/`.

## Acceptance / ledger (RH-2)

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- `odm list` shows `design` and `research` in the TYPE column — **no `odd`** (F-2/F-3 visibly
  resolved). Attach a capture.
- `odm check` green on the corpus (was 59 nodes); `grep -rl 'type: odd\|odd/v1.0' nodes` → empty.
- The 4 research nodes carry `type: research` + `schema: research/v1.0`; the 9 design nodes carry
  `design`/`design/v1.0`.
- ODD-0013 + ODD-0020 amendments folded in with their version-history entries **before** the code
  landed (the model-first rule).

## Method / housekeeping

- One branch (e.g. `rh-c2-type-taxonomy`, off the C-1 tip / `release/1.0.x`), one mergeable diff;
  five-iteration cap. CC implements on local 1.85+; cargo rows attested-by-CC → reproduced-on-CI.
- On close: bubble up to `arc-release-hardening/arc-plan.md` — RH-2 attested; F-2/F-3
  dispositioned; note the `research` gate-set decision taken. **Then C-3** (`list` overhaul) can
  build on the settled type names — and is where **F-15** (default-exclude retired nodes + status
  column) lands.
