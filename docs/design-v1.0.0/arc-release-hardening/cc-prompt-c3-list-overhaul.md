# cc-prompt — RH C-3: `odm list` overhaul

> **Arc:** Release Hardening (UAT) · **Chunk:** C-3 · **Covers:** `F-4`,`F-5`,`F-6`,`F-7`,`F-8`,`F-9`
> **+ `F-15`** · **Kind:** surface · **Feeds:** RH-3. **Depends on:** RH-1 (the `oxur-term` renderer)
> **and** RH-2 (settled `design`/`research` type names). Branch off the C-2 tip.

## Goal

Make `odm list` read as a **plan**, not a dump: drop the number column, lead with date, add a
status column, render placement as a **branch-and-leaf tree**, de-number displayed names, bound
width with elision — and stop surfacing **retired/superseded** nodes as if they were live work.
All rendering stays on the C-1 `oxur-term` themed path.

## Changes (each maps to a finding)

1. **F-4 — drop the NUMBER column.** `number` remains frontmatter metadata; ULID stays identity.
2. **F-5 — DATE first.** Leftmost column = **created** date; flag **`--date={created|updated}`**
   (default `created`) switches it. Header reads `DATE`.
3. **F-7 — STATUS column after TYPE.** A compact rollup of the node's gate vector.
   **Decision to confirm:** what single value? Recommend the **furthest-reached gate name** in the
   node's gate-set (`—` if none reached), with **retired/superseded as an override value** (see
   F-15). Keep it one token; the full status vector is the LLM-command-surface arc's job, not this.
4. **F-8 — branch-and-leaf tree** for the NAME column. Render work nodes by containment
   (`project → arc → slice` via `part_of`) with indent/branch glyphs, replacing the current
   arc/slice **name-prefixing**. **Decision to confirm:** document nodes (`design`/`research`/`adr`/
   `note`) aren't in the `part_of` tree — render them as a separate flat group (recommend: a
   `documents` section below the work tree) rather than forcing them into it.
5. **F-6 — de-number names on display.** Strip embedded number-refs (e.g. "Slice 05 (Arc 06): …"
   → "…") when rendering, and add a naming convention *"names don't embed numbers"*; re-`self-host`
   regenerates de-numbered names. **Display-only** — do not rewrite existing node `name` fields in
   this chunk (that's a data change; keep C-3 to rendering + the convention).
6. **F-9 — max width + elision.** `[display] max_width` config + **`--width`** flag; anything past
   the limit is elided with a trailing ` ...` (cut at `width − 4` to leave room for ` ...`).
7. **F-15 — retired/superseded nodes.** `odm list` **default-excludes** nodes carrying a
   `retired:` block (and superseded nodes); **`--all` / `--include-retired`** opts them back in;
   when shown, STATUS reads `retired`/`superseded` and the row renders **dimmed**. **Audit** the
   corpus for what leaks today (node #1605 is the confirmed tombstone; a coarse scan flagged ~5
   superseded ODDs — confirm which carry a real retired/superseded marker vs. mere prose).

## New column order

`DATE | TYPE | STATUS | NAME (tree, de-numbered) | ID` — no NUMBER.

## Acceptance / ledger (RH-3)

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- `odm list` (capture attached) shows: no NUMBER col; DATE leftmost (+`--date=updated` works);
  STATUS after TYPE; NAME as an indented tree with **de-numbered** names; width elision at the
  configured/`--width` limit; all on the warm-orange `oxur-term` theme.
- **F-15:** default run **omits** node #1605 (and any other retired/superseded); `odm list --all`
  shows them dimmed with STATUS `retired`. Confirm via the corpus.
- `odm check` green on the corpus; counts unchanged by the *display* changes (list is a view).
- Each of F-4…F-9 + F-15 dispositioned in the bubble-up.

## Method / housekeeping

- One branch (e.g. `rh-c3-list-overhaul`, off the C-2 tip); one mergeable diff; five-iteration cap.
  CC implements on local 1.85+; cargo rows attested-by-CC → reproduced-on-CI.
- On close: bubble up to `arc-release-hardening/arc-plan.md` — RH-3 attested; F-4…F-9 + F-15 done;
  record the STATUS-summarization and doc-node-rendering decisions. Then **C-4** (command surface
  cleanup) and **C-5** (fold `self-host` into `migrate`) remain; F-16/F-17 stay open decisions
  (not C-3).
