# Arc — Release Hardening: CLI, types, naming & output (UAT-driven) — plan-of-record

> **Named arc, canonical number deferred** (operator call, 2026-07-07). This is v1.0.0
> release-blocking work surfaced by hands-on UAT of the self-hosted tool. It is **not**
> numbered into the A-sequence: A1–A6 are the MVP core, A7 (telemetry) / A8 (forecasting) are
> the other CDC's active post-MVP arcs, and the numbering scheme itself is **under review in
> this very arc** — so locking an A-number now would be premature. Directory:
> `arc-release-hardening/` (provisional; settles when the numbering decision does).
>
> **A6 is paused after slice04** and resumes at slice05 (PM-skill) once this arc wraps, so the
> PM-skill and the prose-retirement target the *settled* command surface (arc06 arc-plan v1.9).
>
> Refs: A6 slice03 (ODD-0020 schema markers) + slice04 (self-host); ODD-0013 (node types,
> gate-sets, command surface); the deliberate "no `oxur-cli` dep" call in `CLAUDE.md` (now
> being reversed here). `depends_on:` A6 slice04 (the self-hosted corpus is what UAT runs on).

## Capability

Harden the self-hosted `odm` CLI for the v1.0.0 release. Coloured, themed output via a
**shared styling/table crate** (extracted from `oxur-cli` so we get the established theming
without pulling the language/compiler stack); a clearer **type taxonomy** (`odd` → `design`,
plus a new `research` type); a **tree-structured, de-numbered `list`** (branch-and-leaf
placement, date-first, status column, width-elision); consistent **command naming**
(`context`→`project`, `path`→`chain`, quiet-idempotent `new`, format-agnostic `rollup`); and
**`self-host` folded into `migrate`**. Feedback is triaged **surface** (a cc-prompt) vs
**model** (an ODD-0013/0020 amendment or an ADR first), then dispositioned as chunked
cc-prompts, re-validated by re-running `odm self-host`.

## The flow (this arc runs differently)

1. **Duncan gives feedback** in batches (this is batch 1 of an expected ~3–4).
2. **CDC triages** each item → **surface** (cc-prompt) / **model** (amend ODD-0013/0020 or
   write an ADR first) / **question** (decide before it becomes work). Logged as an **F-row**.
3. **CDC chunks** related findings into coherent cc-prompts (**C-rows**).
4. **CC implements**; CDC verifies; where types/names/numbering changed, **re-run
   `odm self-host`** so the corpus reflects the settled surface and stays `check`-green.
5. **Repeat** as more batches land — F-rows and chunks accrete; chunks may split or grow.

## Chunks (candidate cc-prompts, dependency-ordered)

| Chunk | Scope | Kind | Covers (F-rows) | Depends on |
|-------|-------|------|-----------------|------------|
| **C-1 — Adopt Oxur table styling/theming** | Colours + the warm-orange Oxur theme for all output. **Route OPEN** (needs joint investigation + decision, then an ADR): **(A)** depend on `oxur-cli` lib-only (`default-features = false` → `table` + `common::output`, no compiler stack) — matches odm's CLAUDE.md verbatim; **(B)** re-extract a standalone `oxur-table` crate (reverse the late-2025 fold) and depend on just that. See the record below. | model/arch (ADR) | F-1 | — (foundational) |
| **C-2 — Type taxonomy** | `odd` → `design`; add `research` type; re-stamp the 13 migrated nodes; reclassify research docs; gate-sets + schema markers + migrate mapping. | model (ODD-0013 + ODD-0020) | F-2, F-3 | — (foundational) |
| **C-3 — `odm list` overhaul** | Drop the number column; date-first + `--date=updated`; status column after type; branch-and-leaf tree (drop name-prefixing); max-width config+flag with ` ...` elision; names lose number-refs. | surface | F-4, F-5, F-6, F-7, F-8, F-9 | C-1, C-2 |
| **C-4 — Command surface cleanup** | `context`→`project` (+`--name`, current default); `path`→`chain`; `new` warns-not-displays on re-run; `rollup` help + md/json output + `--out`/format name (defaults `md`/`ROLLUP`). | surface | F-10, F-11, F-12, F-13 | (light) C-1 |
| **C-5 — Fold `self-host` into `migrate`** | Consolidate: `self-host` becomes a case of `migrate` (e.g. `migrate --plan` / autodetect); one verb. | surface/medium | F-14 | — |

**Order:** C-1 + C-2 first (foundational — the renderer and the type names everything else
uses) → C-3 → C-4 / C-5 slottable anytime. More batches → more chunks.

## Exit criteria (arc acceptance)

- Every captured feedback item (F-row) is **dispositioned** — shipped, amended, or explicitly
  deferred with a reason.
- **Model items are amended first:** `odd`→`design` + `research` in ODD-0013/0020; the
  styling-crate dependency reversal recorded as an ADR (or an ODD-0013 note).
- Surface fixes shipped; **CI green**; clippy `-D warnings`; no `unsafe`; no regression.
- Where types/names/numbering changed, **`odm self-host` is re-run** and the regenerated
  corpus is `check`-green (the reflexive validation loop).
- The CLI surface / type taxonomy / naming is **settled** enough that A6 slice05 (PM-skill)
  can document it and slice06 (retire prose) can point at it without churn.

## Arc Ledger

> Per LEDGER-DISCIPLINE v2.0 §B. Class-(a) chunk-closed rows (RH-1…RH-5) accrue as chunks
> close; class-(b) compose rows (RH-6/RH-7) reproduced at arc scale, never inherited;
> class-(c) bubble-up (RH-8). IDs are **stable** — new chunks append, never renumber.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| RH-1 | C-1 (shared styling/table crate) closed | ptr: C-1 `cdc-verification.md` | serious | arc-plan | open | | attested-on-close. Foundational — all output renders through it. |
| RH-2 | C-2 (type taxonomy: `odd`→`design` + `research`) closed | ptr: C-2 `cdc-verification.md` | serious | arc-plan | open | | attested-on-close. Needs ODD-0013 + ODD-0020 amendment first. |
| RH-3 | C-3 (`odm list` overhaul) closed | ptr: C-3 `cdc-verification.md` | serious | arc-plan | open | | attested-on-close. Depends on RH-1 + RH-2. |
| RH-4 | C-4 (command surface cleanup) closed | ptr: C-4 `cdc-verification.md` | serious | arc-plan | open | | attested-on-close. |
| RH-5 | C-5 (fold `self-host` into `migrate`) closed | ptr: C-5 `cdc-verification.md` | serious | arc-plan | open | | attested-on-close. |
| RH-6 | **Compose:** the self-hosted CLI is coherent + themed + UAT-validated end-to-end | arc-scale demo: run `odm list`/`orient`/`project`/`chain` on the real corpus — coloured, tree-structured, renamed, de-numbered | serious | arc-plan / UAT | open | | reproduce at arc scale. |
| RH-7 | **Compose:** re-running `odm self-host` after the type/name changes yields a `check`-green corpus with `design`/`research` types + de-numbered names | arc-scale demo: re-self-host → `odm check` green; nodes carry `design/v1.0` etc. | serious | arc-plan / ODD-0013/0020 | open | | reproduce at arc scale. The reflexive validation loop. |
| RH-8 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | open | | accrues as chunks close. |

Closes in `arc-release-hardening/closing-report.md`: per-chunk walk + composition verdict,
independently gated. A failed compose row spawns a remediation chunk, not a re-pass.

## Findings Log (F-rows)

> One row per piece of operator feedback. Triage: **surface** (cc-prompt) / **model** (ODD
> amendment or ADR first) / **question**. Accretes across UAT batches.
>
> **Source:** Duncan's raw first-pass, non-authoritative feedback is preserved verbatim in
> `uat-punch-list.md` (same dir). The F-rows below are the CDC triage of that list; the punch
> list is the *input*, this is the *working disposition*.

### Batch 1 — 2026-07-07

| ID | Finding | Triage | Chunk | Disposition | Status |
|----|---------|--------|-------|-------------|--------|
| F-1 | Output is plain (no colours); adopt the `oxur-cli` table styling/theming (an established pattern) — **very important for v1**; OK to split the table/terminal code out of `oxur-cli` | **model/arch** | C-1 | **Route OPEN (A vs B — needs joint investigation + discussion, then an ADR).** Realigns code to the *original intent* (see record). | open — route TBD |
| F-2 | Don't surface `odd` as a type in the UI → use **`design`** | **model** | C-2 | Rename `NodeType::Odd`→`Design`; schema marker `odd/v1.0`→`design/v1.0`; gate-set `[gates.odd]`→`[gates.design]`; re-stamp the 13 nodes → ODD-0013/0020 amendment | open |
| F-3 | Add a **`research`** type; reclassify research docs from `odd`/`design` → `research` | **model** | C-2 | New `NodeType::Research` + gate-set + schema marker; reclassify the relevant migrated nodes | open |
| F-4 | Remove the **number column** from `odm list` | surface | C-3 | Drop the column (the `number` field stays as metadata; ULID is identity) | open |
| F-5 | First column = **date (creation)**; flag `--date=updated` switches to the updated date | surface | C-3 | Date-first column + `--date={created\|updated}` flag (default created) | open |
| F-6 | Remove **number-references from titles** (confusing as time moves on) | surface / naming | C-3 | De-number names on display + a naming convention "names don't embed numbers"; re-self-host regenerates | open |
| F-7 | Add a **status column** after `type` | surface | C-3 | Status column (from the node's gate rollup) after type | open |
| F-8 | Drop the arc/slice **name-prefixing**; use **branch-and-leaf** ASCII/indent tree (project → arcs → slices) to show placement | surface | C-3 | Tree-rendered `list` with indent/branch glyphs; placement replaces prefixes | open |
| F-9 | **Max display width** config option + flag; elide past it with ` ...` (past width − 4 for ` ...`) | surface | C-3 | `[display] max_width` config + `--width` flag + elision | open |
| F-10 | `odm new` idempotent is good, but **displaying info on re-run** is a confusing antipattern → warn: "project exists; for details run 'odm project --name=<name>'" | surface | C-4 | Quiet-idempotent `new`: on existing, warn + point at `project` (depends on F-11) | open |
| F-11 | `odm context` is too general → rename to **`odm project`** (current project default; others via `--name=`) | surface | C-4 | Rename command `context`→`project` (+`--name`) | open |
| F-12 | `odm path` reads as "filepath" → rename to **`odm chain`** *(decided)* | surface | C-4 | Rename `path`→`chain` ("the critical chain / X→Y path") | open |
| F-13 | `odm rollup` help hardcodes `ROLLUP.md` → it supports md **and** json + an optional output name (defaults `md` / `ROLLUP`) | surface | C-4 | Fix help; `--format={md\|json}` + `--out <name>` (defaults `md` / `ROLLUP`) | open |
| F-14 | `odm self-host` is a special case of `odm migrate` → **combine**; support the self-host case within `migrate` | surface/medium | C-5 | Fold `self-host` into `migrate` (autodetect plan-set vs legacy, or `--plan`); one verb | open |

## Amendments raised

> Model-level findings that change the design of record. Draft the amendment **before** the
> cc-prompt.

| Ref | What changes | Surfaced by |
|-----|--------------|-------------|
| ODD-0013 | Node-type taxonomy: `odd`→`design`; add `research`; per-type gate-sets updated | F-2, F-3 |
| ODD-0020 | Schema markers: `design/v1.0`, `research/v1.0`; re-stamp path for the rename | F-2, F-3 |
| ADR (new) | Adopt Oxur table styling — **route A (oxur-cli lib-only) vs B (re-extract `oxur-table`)** — decision pending joint investigation. **Not** a "reversal" of intent: odm's CLAUDE.md + ODD-0012/0013 §11 already spec oxur-cli/tabled output; the *code* drifted. | F-1 |

### Record — prior oxur-table history (found 2026-07-07)

The oxur-cli styling was **discussed and built before**; F-1 realigns code to the
original intent, it is not new:

- **ODD-0001 (Oxur Letter of Intent)** lists `oxur-table/` as a standalone crate —
  "Table formatting utility ✅ IMPLEMENTED."
- **oxur design doc 0015, "oxur-table API (re)Design"** — a *Final* doc (2025-12-31);
  a full table API redesign exists to lean on.
- **`oxur-cli/src/table/README.md`** states it plainly: *"In late 2025 this module was
  in its own crate but as oxur-cli started to take shape, oxur-table was moved to
  oxur-cli/src/table."* It was **used by `oxd`** (odm's direct ancestor) for its `list`
  — the warm-orange theme Duncan remembers. Recent dev notes `0016/0017-cli-table-cleanup`.
- **odm's design intent already mandates it:** ODD-0012 + ODD-0013 §11 spec `odm-cli`
  output as "oxur-cli/tabled"; odm's `CLAUDE.md` says depend on `oxur-cli`
  `default-features = false` for `common::output` + `table`, *don't* enable `binary`.
  The compiler-stack fear is already avoided by `default-features = false` (lang/comp/repl/
  clap are all behind the `binary` feature). **The code drifted** to raw `tabled` + `writeln!`
  with no oxur-cli and no theme → the plain output Duncan sees.
- **Open scoping question** for the ADR: table **only**, or table **+ terminal output
  helpers** (`common::output`: success/error/info/warning)? Duncan said "table/**terminal**"
  → likely both.
- The API to lean on (`oxur-cli/src/table/README.md`): `OxurTable::new(data).render()`,
  generic over `Tabled`, `ColoredString` cells, TOML theme (ANSI + hex), warm-orange default.

**Next-session action:** jointly investigate A vs B (read oxur-0015; weigh coupling to
oxur-cli's release cadence vs the upstream work of publishing a standalone crate), decide,
then write the ADR + the C-1 cc-prompt.

## Dependencies & git

- **Consumes:** A6 slice04's self-hosted corpus (what UAT runs on) + the whole command
  surface (A1–A3).
- **Base branch:** the chunks build on the latest migrate + self-host + schema code, which
  lives on the **A6 slice04 tip** (`arc06-slice04-self-host-cutover`, off `release/1.0.x`,
  unmerged). **Open item for C-1's cc-prompt:** branch off the A6 tip directly, *or* merge the
  green A6-so-far to `release/1.0.x` first and branch from there. (Recommend deciding at C-1.)
- **Reflexive loop:** after C-2/C-3, re-run `odm self-host` to regenerate the corpus with the
  new type names + de-numbered names, and re-verify `check`-green (RH-7).

## Method

One cc-prompt per chunk; CC implements, CDC verifies; cargo rows via CI / local 1.85+;
five-iteration cap. Chunk closes bubble up to this arc-plan; the arc closes with its own
`closing-report.md` + composition check. **On close, A6 resumes at slice05 (PM-skill)** against
the settled surface.

## Version History

### v1.1 — 2026-07-07
**C-1 reframed after finding the record.** F-1 is not a "reverse the no-oxur-cli decision" —
odm's own intent (CLAUDE.md + ODD-0012/0013 §11) already mandates oxur-cli/tabled output; the
*code* drifted to raw `tabled`. And `oxur-table` was **originally its own crate** (ODD-0001;
Final API-redesign oxur-0015; folded into `oxur-cli/src/table` late 2025; used by `oxd`).
So C-1 becomes a **two-route open decision** — (A) depend on `oxur-cli` lib-only, or (B)
re-extract a standalone `oxur-table` — needing **joint investigation + discussion before the
ADR** (Duncan's call: it touches the oxur repo). Record captured under "Amendments raised."
Surfaced by: Duncan's "did we discuss this before?" + a repo/records search.

### v1.0 — 2026-07-07
Arc created from UAT batch 1 (14 findings). Extracted from A6 (which had briefly held it as a
slice05; arc06 v1.8→v1.9). Named arc, canonical number deferred (operator call: don't renumber
the other CDC's A7/A8, and numbering is under review here). Chunks C-1…C-5 drawn from the
triage; F-1…F-14 logged; ODD-0013/0020 amendments + a styling-crate ADR flagged. Decisions
settled with Duncan: named-no-renumber placement; `odm path`→`odm chain`. Surfaced by: the
move into UAT.
