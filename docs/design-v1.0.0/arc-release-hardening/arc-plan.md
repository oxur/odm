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
| **C-1 — Adopt Oxur table styling/theming** | Colours + the warm-orange Oxur theme for all output. **Route DECIDED — B** (ADR `adr-c1-oxur-table-re-extraction.md` §0; see F-1): re-extracted upstream as **`oxur-term`** (table + terminal helpers); odm depends on it alone (`oxur-cli` shed). Routes weighed were **(A)** `oxur-cli` lib-only vs **(B, chosen)** a standalone crate — see the record below. | model/arch (ADR) | F-1 | — (foundational) |
| **C-2 — Type taxonomy** **DONE 2026-07-26** | `odd` → `design`; added `research`; ODD-0013 v2.0 + ODD-0020 v1.1 amended first; re-stamped the corpus in place (**9 `design` + 5 `research`**, identities preserved) via a new `odm-migrate::restamp` pass; `[gates.design]` + `[gates.research]` (mirrored). `check` green at 60 nodes, no `odd` left. See `c2-closing-report.md`. | model (ODD-0013 + ODD-0020) | F-2, F-3 | — (foundational) |
| **C-3 — `odm list` overhaul** **DONE 2026-07-26** | Shipped: `DATE\|TYPE\|STATUS\|NAME\|ID` — no NUMBER; `--date={created\|updated}`; STATUS = furthest-reached gate; branch-and-leaf containment tree with documents as their own group; display-only de-numbering (+ the ODD-0013 §2.1 convention); `--width`/`[display] max_width` elision; retired/superseded excluded by default (`--all` shows them dimmed). Index `FORMAT_VERSION` 3→4 (`created`+`retired`). See `c3-closing-report.md`. | surface | F-4, F-5, F-6, F-7, F-8, F-9, **F-15** | C-1, C-2 |
| **C-4 — Command surface cleanup** | `context`→`project` (+`--name`, current default); `path`→`chain`; `new` warns-not-displays on re-run; `rollup` help + md/json output + `--out`/format name (defaults `md`/`ROLLUP`). | surface | F-10, F-11, F-12, F-13 | (light) C-1 |
| **C-5 — Fold `self-host` into `migrate` + the dogfood cutover** **DONE 2026-07-26** | `self-host` folded into `migrate` (autodetect plan-set vs legacy; `--plan`/`--legacy` force); **F-20** real git dates (2026-06-20 → 2026-07-25, 13+ distinct days); **F-18** clean names (44/45; the 1 hit is #15's genuine doc title); **L-3a** project `# Vision` body + committed focus; **the SH-6 cutover** — odm's corpus relocated onto the orphan `odm` branch, **every ULID preserved** (in-place re-stamp, G-1-safe), `odm.toml` reduced to a locator, `nodes/` retired from the working branch. CDC-reproduced + fresh-context arc-gate PASS-WITH-NOTES. See `c5-closing-report.md` + `c5-cdc-verification.md`. | surface/medium + data + cutover | F-14, **F-18**, **F-20**, **L-3a** (+ SH-6) | arc-store-home |
| ~~**C-7 — Source name-normalization**~~ **RETIRED into C-5** | The F-18 name work folded into C-5's single re-stamp (corpus rewritten once, not twice). The model rule — ODD-0013 §2.1 "names embed no metadata" (v2.2/v2.3) — **stands**; only the standalone chunk retired. | — | (F-18 → C-5) | — |
| **C-6 — `check` hardening** | Three check-rule additions in **one pass** over `check` (churn once): **G-2** — persist tear-rationale + surface it (`tear --because` validates but the schema drops it → a data-loss bug on a *phantom* slice route; add the field); **G-3** — decomposed/orphan rule (a parent with children but no `decomposed` assertion, or an assertion that disagrees with reverse-`part_of` → warn; `--strict` errors); **L-3b** — a project with **no `# Vision` body** is a finding (the DoD depends on it; sibling to L-3a's data fix). | model (G-2 schema field) + check rules | **G-2, G-3, L-3b** | — |
| **C-8 — Normalized display status** | A **display-only** `not-started` / `in-progress` / `done` state derived from ladder *position* (+ the F-15 `retired` overlay), so STATUS is comparable across types; the raw gate vector stays in `show`/`--json`. **No model change** — render-time derivation. **DONE 2026-07-27** (`c8-closing-report.md`): labels shipped as **`planned`/`active`/`done`** (operator's call over `not-started/in-progress/done`); `--status` accepts **both** the derived vocabulary and raw gates, so `--status done` matches the column and `--status tested` still works; `--status-format=gate` **not built** and `blocked` **deferred**. | surface (render-only) | F-19 | C-3 |

**Order:** C-1 + C-2 first (foundational — the renderer and the type names everything else
uses) → C-3 → **C-5 (done — the cutover, with arc-store-home)** → **C-4 (surface reorg + ODD-0023)**
→ C-6 (check-hardening) + C-8 (normalized status), slottable. More batches → more chunks.

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
- **Pre-release housekeeping (L-8b):** reconcile the four normative design docs whose *directory*
  no longer reflects their authority — ODD-0013 (the current architecture, in `01-draft/`) with its
  amendments 0019/0020 (in `04-accepted/`), plus 0017/0018 — before v1.0.0 ships. The `odd`/design
  corpus should not keep the truth-in-directory disease the node model cured (ODD-0013 §9). *(Routed
  from the UAT coverage audit. Its siblings are dispositioned there too: **L-8a** — migrating the
  design corpus into `design`-type nodes — and **L-6** — prebuilt binaries / `cargo-dist` — are
  **post-1.0** follow-ups; **G-7** auto-commit is **resolved by the store-home**, since odm's commits
  now target the store worktree, isolated from the code branch.)*

## Arc Ledger

> Per LEDGER-DISCIPLINE v2.0 §B. Class-(a) chunk-closed rows (RH-1…RH-5) accrue as chunks
> close; class-(b) compose rows (RH-6/RH-7) reproduced at arc scale, never inherited;
> class-(c) bubble-up (RH-8). IDs are **stable** — new chunks append, never renumber.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| RH-1 | C-1 (shared styling/table crate) closed | ptr: `C-1-cdc-verification.md` | serious | arc-plan | **attested** | CDC structural verify (`C-1-cdc-verification.md`): `oxur-term` is odm's sole oxur dep (`oxur-cli` dropped), no raw `tabled` table-building remains, rendering routes through `odm-cli/src/table.rs` (`TableStyleConfig::default().apply_to_table`) + status via `term.rs`/`oxur_term::common::output`; **themed `odm list` reproduced visually** (2026-07-26 capture, 59 nodes, warm-orange). | Flips `reproduced`/`done` on: commit of the staged diff + CC cargo/clippy/fmt green (local 1.85+) + `odm check` green + CI. Foundational — all output renders through it. Disclosed deviation (not a defect): impl uses the lower-level `Builder`+`apply_to_table` (the `oxur-odm` path), not high-level `OxurTable`, because `OxurTable` lacks theme-injection + a text footer — upstream follow-up noted in `table.rs`. |
| RH-2 | C-2 (type taxonomy: `odd`→`design` + `research`) closed | ptr: `C-2-cdc-verification.md` (+ `c2-closing-report.md`) | serious | arc-plan | **attested** | `c2-closing-report.md` (2026-07-26): ODD-0013 v2.0 + ODD-0020 v1.1 landed **before** the code; workspace build/test/clippy/fmt green; corpus re-stamped in place — 60 nodes, type/schema pairing consistent for all, **zero `odd` in any frontmatter**; the 4 pre-existing research nodes kept their ULIDs; `odm check` green; capture in `c2-capture-odm-list.ansi` | attested-by-CC → **reproduced** when CI runs the cargo rows. Model-first rule honoured. Decisions recorded: `research` mirrors `design`; no legacy `odd` read-alias; node #21 imported (see change-log v1.5). **CDC independently verified 2026-07-26** (structural: 60 nodes, 0 type/schema pairing mismatches, zero `odd` frontmatter, the 4 research ULIDs preserved, `restamp` sound; `odm check` + `make check` operator-confirmed green) — `C-2-cdc-verification.md`. |
| RH-3 | C-3 (`odm list` overhaul) closed | ptr: `C-3-cdc-verification.md` (+ `c3-closing-report.md`) | serious | arc-plan | **attested** | `c3-closing-report.md` (2026-07-26): build/test (53 binaries, +15 tests)/clippy/fmt green; `odm list` renders `DATE\|TYPE\|STATUS\|NAME-tree\|ID`, de-numbered, width-elided, on the C-1 theme; **#1605 omitted by default**, shown `retired`+dimmed under `--all`; `odm check` green, 60 nodes (a view change moves no counts); capture in `c3-capture-odm-list.ansi` | attested-by-CC → **reproduced** when CI runs the cargo rows. Decisions: STATUS = furthest-reached gate; document nodes as a flat group below the work tree. Index format bumped 3→4 (`created` is *not* ULID-derivable for migrated nodes; `retired` is a flag). **CDC independently verified 2026-07-26** (capture + code confirm columns/tree/STATUS/width/de-number; `FORMAT_VERSION`=4 with version-mismatch rebuild; F-15 audit reproduced — exactly 1 retired (#1605), 0 active `supersedes` edges, the ~5 was prose) — `C-3-cdc-verification.md`. |
| RH-4 | C-4 (command surface cleanup **+ the ODD-0023 reorg**) closed | ptr: C-4 `cdc-verification.md` (+ `c4-closing-report.md`) | serious | arc-plan | **attested** | `c4-closing-report.md` (2026-07-26): build/test (**58 binaries, 0 failed**)/clippy `-D warnings`/fmt green; no `unsafe`. **ODD-0023 Accepted (v1.1) before the code.** Three tiers shipped — top-level workflow verbs, `odm node <cmd>` (11 verbs), `odm store <cmd>`. F-10 (`new` warns + points at `project --name=`, no dump), F-11 (`context`→`project`, `--name`), F-12 (`path`→`chain`), F-13 (`rollup --format={md\|json}` + `--out`; `ROLLUP.md` is a default, not a law). **Inventory ↔ `--help` parity checked mechanically: 11/11/2, MATCH.** Reflexive check run on the **self-hosted store** (not a scratch repo): validate green at 60, composite `check` runs both phases, `use`/`orient` agree — the C-5 regression verified where the root split actually hid. | attested-by-CC → **reproduced** on CI. **Three ODD-0023 reversals, each recorded with reasoning:** `use` **kept** (C-5 gave it the CURRENT FOCUS workflow, which did not exist when the draft called it redundant); **`validate` is the pure command and `check` the composite** (operator's call, inverting the draft's `check --reconcile` opt-in — the cheap operation gets its own name, so no flag is needed to avoid probe cost, and `check` means *go and look*; it stops at validate **errors**, and the skipped half is `null` + `reconcile_skipped` so absent stays distinguishable from clean; schema ids bumped to **`validate/v1`**/**`check/v2`** because `check` changed meaning under the same name); and **hard cut, no aliases** (cost: ~290 test invocations rewritten — the whole bill, and cheap against two spellings in every doc for a release). **Three parity defects found by checking rather than assuming:** the inventory listed a `store sync` command that was never built (ff-sync is an arm of `store init`), `store init`'s help still said "bootstrap only for now" — stale since slice 03 — and the `self-host` CLI wrapper was dead once its verb went. |
| RH-5 | C-5 (fold `self-host` into `migrate`, **+ the dogfood cutover**) closed | ptr: C-5 `cdc-verification.md` (+ `c5-closing-report.md`) | serious | arc-plan | **attested** | `c5-closing-report.md` (2026-07-26): build/test (**58 binaries, 0 failed**, +8 tests)/clippy `-D warnings`/fmt green. F-14 (fold: `migrate` autodetects plan-set vs legacy, `--plan`/`--legacy` force), F-18 (44/45 work-node names normalized), F-20 (dates now span **2026-06-20 → 2026-07-25**, 13 distinct days, against 45× `2026-07-07`), L-3a (project `# Vision` body + committed focus). The cutover moved 60 nodes onto the orphan `odm` branch: **every ULID preserved** (pre/post sets diff-empty), `check` green at 60 in the home, `odm.toml` reduced to `[store]`. | attested-by-CC → **reproduced** on CI. **Three defects found by the dogfood, all invisible beforehand:** `orient` read the *invocation* root for the context while `use` wrote the *store* root — identical until the two parted, so `use` succeeded and `orient` reported "(no current arc)"; `store init` scaffolded no `.gitignore`, offering the derived index for commit on a shared branch; and the first fix for that ignored all of `.odm/`, which would have withheld the current focus from every fresh clone — defeating the project's own success test. |
| RH-6 | **Compose:** the self-hosted CLI is coherent + themed + UAT-validated end-to-end | arc-scale demo: run `odm list`/`orient`/`project`/`chain` on the real corpus — coloured, tree-structured, renamed, de-numbered | serious | arc-plan / UAT | open | | reproduce at arc scale. |
| RH-7 | **Compose:** re-running `odm self-host` after the type/name changes yields a `check`-green corpus with `design`/`research` types + de-numbered names | arc-scale demo: re-self-host → `odm check` green; nodes carry `design/v1.0` etc. | serious | arc-plan / ODD-0013/0020 | **attested** | Satisfied *in the home* after the C-5 cutover: `check` green at 60 nodes, types are `design`/`research`, names de-numbered, `schema: <type>/v1.0` markers intact. | reproduce at arc scale. The reflexive validation loop. **Nuance for the record:** the corpus is **re-stamped in place**, not minted fresh — so this validates the *derivation logic* via the re-stamp. A truly fresh re-derivation (new ids) is **out of scope pending G-1**, and would in fact break every edge; C-5 preserved all 60 ULIDs deliberately. |
| RH-8 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | open | | accrues as chunks close. **C-1 (v1.2):** F-1 dispositioned (Route B / `oxur-term`); F-15 surfaced while validating C-1's themed `list` (routed to C-3, not a C-1 defect) — logged, not dropped. **v1.3:** F-16 (unconditional table ANSI) re-seated from a simultaneous-write id collision — logged, unassigned. **C-2 (v1.6):** F-2/F-3 dispositioned; **F-17** (body-snapshot drift) logged — a migrate/self-host decision, not C-3. **C-3 (v1.7):** F-4…F-9 + F-15 dispositioned; the F-15 audit corrected the "~5 superseded ODDs" to **zero** (prose matches, not markers) — recorded rather than quietly dropped. **C-3 CDC (v1.8):** independently reproduced — 1 retired (#1605), 0 active `supersedes` edges (the ~5 was prose). **C-3 review → F-18:** work-node names embed doc metadata (`(plan-of-record)` suffix, 32/47) — routed to a new **C-7** (source name-normalization + §2.1 rule); C-6 stays reserved for G-2. **C-3 review → F-19:** STATUS (furthest gate) isn't comparable across types → new **C-8**, a display-only normalized `not-started/in-progress/done` state. **C-3 review → F-20/F-21 (v1.11):** `self-host` discards the plan's real creation dates (data loss → **C-5**); `--json` omits `created`/`updated` (unassigned). Both surfaced by reading C-3's DATE column; neither is a C-3 defect. **C-8 (v1.14):** F-19 dispositioned **done**. Labels are **`planned`/`active`/`done`**; `--status` accepts the derived *and* raw vocabularies so the flag and the column cannot disagree while F-15's raw spellings keep working; `blocked` deferred (needs the graph); `--status-format=gate` not built. **One deviation, and the important one:** the chunk's case for a normalized-only column was that `show`/`--json` already carried the raw ladder — they did not carry it at all, so the rung would have become unreachable from the CLI. Fixed in-chunk by giving `show` a gate ladder and `--json` a `status` + `gates` pair, which is what the brief's own acceptance text already assumed. The per-gate colour palette retired with the raw column (nothing could select it); its slots live on, and the `complete`-green / `verified`-bright-green distinction now survives only in `show`. **C-4 (v1.13):** F-10…F-13 dispositioned **done**, folded with **ODD-0023 → Accepted (v1.1)**. Three of the ODD's tentative calls were **reversed** and the reversals recorded rather than quietly rewritten: `use` kept (C-5 gave it a workflow the draft could not have known about), `validate`/`check` inverted so the pure operation owns the honest name, and a hard cut instead of a deprecation window. **Three parity defects surfaced by the inventory↔`--help` check** — a documented `store sync` command that was never built, `store init` help stale since slice 03, and the now-dead `self-host` CLI wrapper — none of them a C-4 defect, all fixed here because the chunk owns surface parity. Parity is now verified mechanically (11/11/2) rather than by reading. **C-5 (v1.12):** F-14/F-18/F-20/L-3a dispositioned **done**, and **SH-6 closed jointly with arc-store-home** — the cutover. C-7 formally retires into C-5. Three new findings, all from the dogfood itself and all fixed in-chunk: **(a)** `orient` resolved the CLI context against the *invocation* root while `use`/`context` used the *store* root — invisible while the two were the same directory, and the reason `use` could print success and `orient` still report "(no current arc)"; fixed by deriving the path from the `Store` handle so no caller can pass the wrong root. **(b)** `store init` scaffolded no `.gitignore`, so a store branch offered its derived index for commit. **(c)** the first fix for (b) ignored all of `.odm/`, which would have withheld `context.json` — the current focus — from every fresh clone, defeating the project's stated success test that a fresh session orients from `odm orient` alone; the rule is now ignore-the-directory-plus-one-exception, because a list of caches fails open (`.odm/drift` was already missed). **Two brief corrections recorded, not silently absorbed:** the re-stamp relocates nothing (the month shard is a function of the ULID, not the date), and document node #15's `(build plan)` is its real H1 title rather than importer metadata, so it stays. |

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
| F-1 | Output is plain (no colours); adopt the `oxur-cli` table styling/theming (an established pattern) — **very important for v1**; OK to split the table/terminal code out of `oxur-cli` | **model/arch** | C-1 | **DECIDED: Route B — `oxur-term` extracted** (table **+** terminal helpers; ADR `adr-c1-oxur-table-re-extraction.md` §0). odm depends on `oxur-term` alone (`oxur-cli` shed). C-1 wires odm's tables + status lines through it; themed `list` reproduced. | **dispositioned** (RH-1 attested-on-close) |
| F-2 | Don't surface `odd` as a type in the UI → use **`design`** | **model** | C-2 | **Shipped.** `NodeType::Odd`→`Design`; `odd/v1.0`→`design/v1.0`; `[gates.odd]`→`[gates.design]`; corpus re-stamped in place (ids preserved). `odm list` shows `design`, never `odd`. | **done** — 2026-07-26 |
| F-3 | Add a **`research`** type; reclassify research docs from `odd`/`design` → `research` | **model** | C-2 | **Shipped.** `NodeType::Research` + `[gates.research]` (mirrors `design`) + `research/v1.0`; classification is **tag-based** (`research` in `tags`), so it survives renames. 5 research nodes: #11, #14, #16, #18, #21. | **done** — 2026-07-26 |
| F-4 | Remove the **number column** from `odm list` | surface | C-3 | **Shipped.** Column dropped; `number` stays frontmatter metadata + a CLI handle, ULID is identity. | **done** — 2026-07-26 |
| F-5 | First column = **date (creation)**; flag `--date=updated` switches to the updated date | surface | C-3 | **Shipped.** `DATE` leads (created by default); `--date={created\|updated}`. Needed `created` in the index — it is not ULID-derivable for migrated nodes. | **done** — 2026-07-26 |
| F-6 | Remove **number-references from titles** (confusing as time moves on) | surface / naming | C-3 | **Shipped (display-only).** `list` strips `Slice NN …`/`Arc NN …` prefixes; the convention **"names don't embed numbers"** is ODD-0013 §2.1 (v2.1). Stored names untouched — a re-`self-host` regenerates them. | **done** — 2026-07-26 |
| F-7 | Add a **status column** after `type` | surface | C-3 | **Shipped.** `STATUS` = the furthest-reached gate in the node's own gate-set (ordered by the *configured sequence*, since the index stores gates name-sorted), `—` when none; `retired`/`superseded` override. | **done** — 2026-07-26 |
| F-8 | Drop the arc/slice **name-prefixing**; use **branch-and-leaf** ASCII/indent tree (project → arcs → slices) to show placement | surface | C-3 | **Shipped.** Containment tree via `part_of` with `├─`/`└─`/`│` glyphs. Document nodes have no containment parent, so they render as a flat `── documents ──` group below the work tree rather than being forced into it (operator decision). | **done** — 2026-07-26 |
| F-9 | **Max display width** config option + flag; elide past it with ` ...` (past width − 4 for ` ...`) | surface | C-3 | **Shipped.** `[display] max_width` in `odm.toml` (default 64) + `--width`; cut at `width − 4` so the elided cell lands exactly on the limit. | **done** — 2026-07-26 |
| F-10 | `odm new` idempotent is good, but **displaying info on re-run** is a confusing antipattern → warn: "project exists; for details run 'odm project --name=<name>'" | surface | C-4 | Quiet-idempotent `new`: on existing, warn + point at `project` (depends on F-11) | **done** (C-4, 2026-07-26): `new` on an existing node warns (was info) and stays a one-liner pointing at `odm project --name=<name>` — the details command, rather than a dump nobody asked for |
| F-11 | `odm context` is too general → rename to **`odm project`** (current project default; others via `--name=`) | surface | C-4 | Rename command `context`→`project` (+`--name`) | **done** (C-4, 2026-07-26): `context` → `project`, top-level, current by default with `--name` for another project (whose arc is deliberately not shown — the current arc belongs to the current selection) |
| F-12 | `odm path` reads as "filepath" → rename to **`odm chain`** *(decided)* | surface | C-4 | Rename `path`→`chain` ("the critical chain / X→Y path") | **done** (C-4, 2026-07-26): `path` → `chain` |
| F-13 | `odm rollup` help hardcodes `ROLLUP.md` → it supports md **and** json + an optional output name (defaults `md` / `ROLLUP`) | surface | C-4 | Fix help; `--format={md\|json}` + `--out <name>` (defaults `md` / `ROLLUP`) | **done** (C-4, 2026-07-26): `rollup --format={md\|json}` + `--out <NAME>` writing `<stem>.<ext>`; the help presents `ROLLUP.md` as the default it is |
| F-14 | `odm self-host` is a special case of `odm migrate` → **combine**; support the self-host case within `migrate` | surface/medium | C-5 | Fold `self-host` into `migrate` (autodetect plan-set vs legacy, or `--plan`); one verb | **done** (C-5, 2026-07-26): `detect_corpus()` autodetects; `--plan`/`--legacy` force; `self-host` is a spelling of the same path. |

### Batch 2 — 2026-07-26 (surfaced during C-1/C-2/C-3 — implementation and validation)

| ID | Finding | Triage | Chunk | Disposition | Status |
|----|---------|--------|-------|-------------|--------|
| F-15 | **`odm list` shows retired/superseded nodes with no distinction.** Node **1605** (`Slice 05 (Arc 06): UAT — CLI feedback`) carries a `retired:` frontmatter block (retired 2026-07-25 — the **L-2** stale-slice-list tombstone) yet renders identically to live work, reading as A6's "slice 05" — the exact "trust the filesystem over the plan-of-record" hazard the node was created to document. Now the tool itself surfaces the tombstone as live work. | surface | C-3 | `odm list` **default-excludes** retired/superseded nodes; `--all` / `--include-retired` opts them back in, and **`--status <VALUE>`** shows only the rows at one status (`--status retired` = exactly the withheld set); when shown, a `retired`/`superseded` value in C-3's **status column (F-7)** + a dimmed style. The *default-visibility* call is the new decision (F-7 already covers "make it distinct"). **Audit** whether other superseded nodes leak — a coarse scan flagged ~5 migrated ODDs matching the same pattern (only 1605 confirmed structurally). | **done** — 2026-07-26. **Shipped:** default-excluded; `--all`/`--include-retired` shows them with STATUS `retired`/`superseded`, dimmed. **Audit result:** exactly **one** retired node (#1605) and **zero** supersedes edges corpus-wide — the ~5 was a false positive (those files match the *word* "superseded" in bodies/prose, not a marker). Superseded-exclusion ships tested against fixtures, not live data. |
| F-16 | **Themed tables emit ANSI unconditionally**, so `odm list > file` / `\| less` carries escape sequences. The table theme rides on `tabled::settings::Color`, which writes escapes regardless of the sink; the status lines *do* degrade to plain text off a terminal (`colored` honours TTY-detection + `NO_COLOR`). The two halves of odm's output therefore disagree about when colour is appropriate. | surface / question | (unassigned) | Raised by C-1 and deliberately **not** absorbed into it — a TTY guard is a behaviour change, not the rendering swap C-1 was scoped to. A `NO_COLOR`/TTY guard is best landed **upstream in `oxur-term`** (around `apply_to_table`) so odm and oxur agree rather than each growing a local rule. Weigh against the pass-2 LLM findings on machine consumption; `--json` is the machine path today, so this is not release-blocking. | open — decide |
| F-17 | **Re-stamp / re-migrate does not refresh node bodies.** After the C-2 rename, node #20's body still carries the *pre-amendment* `odd/v1.0` marker text while its source doc (ODD-0020) reads `design/v1.0` — node bodies are import-time snapshots. Bears on the DoD: `odm show <node>` shows the snapshot, not the live source. | model / question | (unassigned — migrate/self-host) | **Decide:** snapshot-by-design (document it; source doc = editable truth, node = the build) **vs** refresh bodies on re-migrate. Not release-blocking. Ptr: `C-2-cdc-verification.md` §Findings; `c2-closing-report.md`. | open |
| F-18 | **Node names embed document metadata.** Work-node names carry a trailing `(plan-of-record)` role-suffix — **32 of 47** work nodes; with number-prefixes, that's metadata in the identity label. `self-host` derived names verbatim from plan-doc H1 headings (`# Arc 01 — X (plan-of-record)`). Names are the thing, not its coordinates or its file's role. (Zero *document* nodes carry the suffix.) | surface / model | **C-7** (self-host name-derivation + ODD-0013 §2.1) | Generalize §2.1 from "names don't embed numbers" to **"names embed no metadata"** (numbers, containment refs, doc-role labels); the importer **normalizes on mint** (strip number-prefix + role-suffix), then **re-self-host** regenerates clean names — F-6's display-strip becomes belt-and-suspenders. A source fix, not a render rule. **C-6 is reserved for G-2** (tear-rationale) — this is C-7, no collision. | **done** (C-5, 2026-07-26; C-7 retired into C-5): ODD-0013 §2.1 at **v2.3** landed first, then `normalize_name` in the derivation + an in-place re-stamp. **44 of 45** work-node names changed; **zero** work nodes still carry a role suffix. Deliberately conservative: one leading separator consumed, so `"Slice 05 (Arc 06): UAT — CLI feedback"` → `"UAT — CLI feedback"`, and `"Arc 02 cleanup (plan-of-record)"` → `"Arc 02 cleanup"` (the coordinate-looking prefix is the slice's actual subject, and a qualifier like `(v-major rebuild)` is not a role). **One residual, by design:** document node #15 `odm — Arc/Slice Breakdown (build plan)` — that is the document's own H1 title, not importer metadata; stripping it would rename a document to something it is not. Flagged for CDC. |
| F-19 | **STATUS is type-relative — not comparable across rows.** `list` STATUS shows the furthest-reached *gate*, but each type climbs a different ladder (slice `planned→built→tested`; arc/project `planned→in-progress→complete→verified`). A slice's `tested` and an arc's `verified` both mean *done* yet look different, and an arc's `complete` is **not** done though it reads like an endpoint — "am I done?" isn't answerable from the cell. (The plateau itself is correct: those are terminal gates, not a stall.) | surface | **C-8** | Add a **display-only** normalized state derived from ladder *position* — `not-started` / `in-progress` / `done`, plus the F-15 `retired` overlay (`blocked` optional, needs the graph). Shown in `list`; raw gate vector stays in `show`/`--json`. **No model change** — pure render-time derivation. | **done** (C-8, 2026-07-27): a done arc (`verified`) and a done slice (`tested`) now read alike, an arc at `complete` reads `active`. Derivation is a pure function over the ladder — 11 unit tests, including a `assert_eq!` that the two done spellings agree. **The brief's premise for dropping the raw gate was false and was fixed here:** `show`/`--json` did **not** carry the gate vector — neither reported gates at all — so normalizing alone would have made the ladder unreachable from the CLI. `node show` now prints the state plus every rung reached/unreached, and `--json` carries `status` + a `gates` array. Two unspecified cases decided against borrowing meaning: a type with **no** ladder (an `adr`) reports `—`, not `planned`, and `--json` omits the field rather than emitting the display em-dash. | 
| F-20 | **`self-host` discards the plan's real creation dates — data loss.** Work nodes are stamped `created`/`updated` = the **cutover date**: 45 of 46 read `2026-07-07`, one `2026-07-25`. The plan was not written in a day — git has the arcs spanning **2026-06-20 → 2026-07-25**, with per-slice directories dated individually. `odm list`'s DATE column is therefore uninformative for the entire plan, and read as a display bug before it proved to be missing data. A6 slice04 chose this deliberately (`selfhost.rs`: *"these work nodes are created now; the plan docs' own history lives in git"*) — the premise is true, but nothing ever reads it back. | **data / model** | **C-5** | Derive `created` from the **earliest git add-date** of the node's plan directory (`git log --diff-filter=A --reverse`), falling back to today for an untracked path. Needs an **in-place re-stamp** as well as the code fix: re-running `self-host` skips existing nodes (keyed on `(type, number)`), so the 46 already minted will not repair themselves — the two-part shape C-2 hit. Routed to C-5, which already rewrites this code, so the corpus is rewritten once not twice. **`migrate` is unaffected** — document nodes preserve their legacy `created` (verified on #13: `2026-06-20`, distinct from its `updated` `2026-06-26`). | **done** (C-5, 2026-07-26): `created` = earliest git add-date of the node's **own** plan directory (so a slice dates from its own dir, not its arc's), `updated` = latest commit date, via the store's worktree module. **44 dates corrected**; the range is now **2026-06-20 → 2026-07-25 across 13 distinct days**. #13 unchanged at `2026-06-20`/`2026-06-26`, as predicted. **The re-stamp does not relocate anything** — the month shard derives from the *ULID's* timestamp, not the frontmatter date, so preserving ids preserves the shard; the brief expected a move, and forcing one would have broken `Store::load`. Retired nodes are skipped: re-deriving #1605 from its own tombstone directory would have rewritten the record of the error. |
| F-21 | **`--json` omits both dates.** `odm list --json` emits `component, id, name, number, origin, part_of, reserved, retired, supersedes, tags, type` — no `created`, no `updated`. A machine consumer cannot see when anything was made or last touched, though the human table has led with a date since C-3. | surface | (unassigned) | Add `created`/`updated` to `NodeJson`. Small and contained, but a **`--json` contract change**, so it belongs with the LLM-command-surface arc's read-back work rather than being slipped in unannounced. Bears on the project DoD ("a fresh session reaches full situational awareness"): a session reading `--json` currently cannot date anything. | open |

Cross-ref (F-15): `uat-report-llm-pass-batch2.md` **L-2**; node `01KYDAHHHZAHNMQY47A4VBSHJD` (#1605). Not a C-1 defect — C-1 is rendering-only and this predates it; surfaced *because* the themed `list` made the corpus legible.

Cross-ref (F-16): `c1-closing-report.md` §"Consequences worth the operator's attention" #1.
Logged by CC during C-1 implementation. It first landed under the `F-15` id in a simultaneous
write with the CDC's Batch-2 entry; the CDC entry keeps `F-15` (ids are stable once published)
and this finding was re-seated at **F-16**. Same finding, new number — nothing was dropped.

## Amendments raised

> Model-level findings that change the design of record. Draft the amendment **before** the
> cc-prompt.

| Ref | What changes | Surfaced by |
|-----|--------------|-------------|
| ODD-0013 | Node-type taxonomy: `odd`→`design`; add `research`; per-type gate-sets updated | F-2, F-3 |
| ODD-0020 | Schema markers: `design/v1.0`, `research/v1.0`; re-stamp path for the rename | F-2, F-3 |
| ADR (new) | **DECIDED — Route B: `oxur-term` extracted** (table + terminal helpers), odm depends on it alone. `adr-c1-oxur-table-re-extraction.md` (§0 Decision update). Was: "route A vs B pending joint investigation." | F-1 |

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
  unmerged). **Settled at C-1:** work branched off `release/1.0.x` as **`rh-c1-adopt-oxur-term`** (commits `fd6af52`→`0e9fd63`→`0d74240`); later chunks branch from here or `release/1.0.x`.
- **Reflexive loop:** after C-2/C-3, re-run `odm self-host` to regenerate the corpus with the
  new type names + de-numbered names, and re-verify `check`-green (RH-7).

## Method

One cc-prompt per chunk; CC implements, CDC verifies; cargo rows via CI / local 1.85+;
five-iteration cap. Chunk closes bubble up to this arc-plan; the arc closes with its own
`closing-report.md` + composition check. **On close, A6 resumes at slice05 (PM-skill)** against
the settled surface.

## Version History

### v1.12 — 2026-07-26
**C-5 closed (RH-5 attested) — the dogfood cutover; chunk table + routing reconciled.** C-5 shipped
the fold (`self-host`→`migrate`, autodetect/`--plan`/`--legacy`), real git dates (F-20), clean names
(F-18), the project vision body (L-3a), **and** the SH-6 cutover — odm's corpus relocated onto the
orphan `odm` branch with **every ULID preserved** (in-place re-stamp, so nothing was minted and G-1
stays clear; a fresh re-derivation would have broken every edge). CDC reproduced it independently and
an **independent fresh-context arc-gate** returned **PASS-WITH-NOTES** — closing **arc-store-home**
jointly (SH-6). Three dogfood-only defects were found and fixed in-chunk (the `use`/`orient` root split;
`store init`'s missing index gitignore; and the over-broad first fix that would have withheld
`context.json` from clones). The **Chunks table** is reconciled to reality: **C-5 → DONE** (now carries
F-14/F-18/F-20/L-3a + SH-6); **C-7 retired into C-5** (the §2.1 model rule stands); **C-6** promoted to a
proper row and **expanded to a `check`-hardening bundle — G-2 (tear-rationale) + G-3 (decomposed/orphan)
+ L-3b (no-vision finding)** — so `check` churns once; **C-8** (F-19 normalized status) added as a row.
**Exit criteria** gained the **L-8b** pre-release reconciliation (the four state-drifted ODDs), with
L-8a/L-6 recorded post-1.0 and **G-7 resolved by the store-home**. Surfaced by: the C-5 close + folding
the UAT coverage-audit routing into the plan-of-record. *(L-3a's routing ratification is retroactively
satisfied — it shipped in C-5.)*

### v1.11 — 2026-07-26
**F-20 and F-21 logged — both surfaced by reading C-3's DATE column, neither a C-3 defect.**
**F-20 is data loss:** `self-host` stamps work nodes `created` = the cutover date, so 45 of 46
read `2026-07-07` and the column says nothing about the plan's actual history. A6 slice04 made
that call deliberately — *"the plan docs' own history lives in git"* — and the premise holds
(git has the arcs spanning **2026-06-20 → 2026-07-25**, per-slice directories dated
individually), but nothing ever reads it back. **`migrate` is unaffected**: document nodes
preserve their legacy `created`, verified on #13 (`created` 2026-06-20 vs `updated` 2026-06-26
— the one node in the corpus where the two differ, which is also why the loss stayed invisible).
Routed to **C-5**, which already rewrites this code, because the fix needs an **in-place
re-stamp** too — re-running `self-host` skips existing nodes, so the 46 will not repair
themselves (the shape C-2 hit). **F-21:** `--json` emits neither date, so a machine consumer
cannot date anything — left unassigned as a `--json` contract change for the LLM-surface arc.
Surfaced by: the operator asking which date the DATE column shows.

### v1.10 — 2026-07-26
**F-19 logged → C-8 (normalized status, display-only).** Operator review of `odm list --all` noted STATUS plateauing at `tested`/`verified` — *correct* (terminal gates of the slice/arc ladders, not a stall), but it exposed that STATUS (furthest-reached gate) is **type-relative and not comparable across rows**. F-19 adds a **display-only** normalized state (`not-started/in-progress/done` + the F-15 `retired` overlay), derived from ladder position at render time — no model change; the raw gate vector stays in `show`/`--json`. Chunk **C-8** (C-6=G-2, C-7=F-18 — no collision). cc-prompt drafted (`cc-prompt-c8-normalized-status.md`). Surfaced by: operator UI-iteration review + the gate-set confirmation.

### v1.9 — 2026-07-26
**F-18 logged; naming rule generalized.** Reviewing C-3's de-numbering surfaced that work-node *names* still embed document metadata — a `(plan-of-record)` role-suffix on 32/47 work nodes, inherited verbatim by `self-host` from plan-doc H1 headings. F-6 handled numbers *on display*; **F-18 fixes names at the source**: generalize ODD-0013 §2.1 ("names don't embed numbers" → **"names embed no metadata"**), normalize on mint in the importer, and re-self-host. Routed to a new **C-7** — **C-6 is reserved for G-2** (tear-rationale), avoiding a chunk-id collision. Amendment + cc-prompt drafted (`C-7-amendment-ODD-0013.md`, `cc-prompt-c7-name-normalization.md`). Surfaced by: the C-3 CDC review + operator confirmation.

### v1.8 — 2026-07-26
**C-3 CDC-verified.** Independent CDC verification (`C-3-cdc-verification.md`) reproduced C-3's close from the capture + code: the `DATE|TYPE|STATUS|NAME-tree|ID` layout, containment tree + `documents` group, furthest-reached-gate STATUS, display-only de-numbering, width elision, and the F-15 retired filter (default omits #1605; `--all` shows it dimmed). Index `FORMAT_VERSION` 3→4 confirmed sound (version-mismatch → rebuild self-heal). **F-15 audit independently reproduced:** exactly one retired node (#1605), zero active `supersedes` edges corpus-wide — the crude "~5 superseded" was prose (the lone `supersedes:` field is `null`). RH-3 Verify pointer → the CDC doc. Noted (not acted): displayed names retain a `(plan-of-record)` suffix — cosmetic, outside F-6, a re-self-host cleanup candidate. Surfaced by: the C-3 CDC verification.

### v1.7 — 2026-07-26
**C-3 closed (RH-3 attested); F-4…F-9 + F-15 dispositioned.** *(Amended in review: a
tree-glyph bug fixed, `--status` added, the group headers replaced by a rule — see the tail of
this entry.)* `odm list` now reads as a plan:
`DATE|TYPE|STATUS|NAME|ID` — no NUMBER, date-first (`--date={created|updated}`), STATUS as the
furthest-reached gate, a branch-and-leaf containment tree with **document nodes as their own
flat group** (they have no `part_of` parent — operator decision), display-only de-numbering
plus the new ODD-0013 §2.1 convention *"names don't embed numbers"*, and `--width` /
`[display] max_width` elision. **F-15**: retired/superseded nodes are excluded by default and
shown dimmed under `--all`; the summary line states what it withheld, since silently hiding
rows is the very L-2 hazard that raised the finding. Two decisions taken: STATUS = furthest
gate (not a progress count); documents grouped, not interleaved. **Index `FORMAT_VERSION` 3→4**
— `created` and `retired` had no home in the record, and `created` is *not* ULID-derivable for
migrated nodes (their id was minted at import, their `created` is the legacy date); an older
snapshot self-heals via `RebuildNeeded(VersionMismatch)`. **F-15 audit result:** exactly one
retired node (#1605), **zero** supersedes edges — the "~5 superseded ODDs" were files matching
the word in prose, so the superseded path ships tested against fixtures only. Corpus unmoved at
60 nodes (a view change moves no counts). Evidence: `c3-closing-report.md`. Surfaced by: C-3
implementation (CC).

**Five review amendments (operator, same day).** (1) **Tree-glyph bug fixed** — the first cut
derived the tree and *then* dropped withdrawn rows, so hiding #1605 left `self-host cutover`
with a `├─` pointing at an unrendered row. `├─`/`└─` encodes "last among siblings" and
root-ness encodes "my parent is on screen", both properties of the *visible* set, so every row
filter now runs **before** the structure is derived; the same flaw would have left the children
of a hidden parent indented under nothing. Two regression tests, both confirmed to fail against
the unfixed source — the existing F-15 tests passed either way, which is why it survived the
first pass. Spotted by the operator reading the output. (2) **`--status <VALUE>` added** — the
summary reported a withheld row but gave no way to *see* it; the flag filters on the STATUS
token and implies `--all` for a withdrawn value. (3) **Group headers → a rule**: the
`── work ──` / `── documents ──` label rows are replaced by one full-width divider — crossing
the column separators at `┼`, drawn in the separator colour so it and the verticals read as one
grid, leading and trailing space intact. (4) **`--group plan|reference` added**, listing one
family at a time. The model's names for the families are *work* and *document* (ODD-0013 §2.2);
`plan`/`reference` are their **display** names, chosen because "document" reads as general
English though precise in odm, and recorded in **ODD-0013 §2.2 (v2.2)** so UI and model
vocabulary stay explicitly paired — the drift F-2 caught with `odd`. (5) **The 0.3.5 STATUS
palette restored**: the STATUS cell is coloured by the *same* `state_to_fg_color` helper
`oxur-odm` called (it moved `oxur-cli`→`oxur-term` in C-1), so the colours cannot drift by being
retyped — `draft` yellow, `accepted`/`final` green, and so on, colour only, as the original had
it. The work sequences postdate that palette, so they were mapped onto its **slots** (operator
decision): `planned` yellow, `in-progress`/`built` cyan, `complete`/`tested` green, `verified`
bright green. `complete` green + `verified` bright green deliberately keeps ODD-0013 §5.1's
"done at its layer" ≠ "verified live" split visible — the colours must not re-collapse what the
gate model separates. Note the terminal gate differs by type (`verified` project/arc, `tested`
slice, `final` design/research), so green means *done at that node's layer*, not *finished*.
Dimming wins over the colour on a withdrawn row. A **TYPE palette** was added alongside it —
`project` magenta, `arc` violet, `slice` blue, `design` orange, `research` red (truecolor;
`design`/`research` share saturation and luminosity, differing only in hue; `adr`/`note` left
uncoloured rather than assigned by omission), and **DATE/ID muted** to ~58% of their band so the
middle three columns carry the eye — derived from the theme's row colours, so the alternating
stripe survives — drawn from a different colour
system than STATUS on purpose, so *what kind of thing is this* and *how far along is it* stay
visually separable.

### v1.6 — 2026-07-26
**C-2 CDC-verified; F-17 logged.** Independent CDC structural verification (`C-2-cdc-verification.md`) reproduced C-2's close — 60 nodes (9 design + 5 research + 46 work), zero `odd` in frontmatter, type/schema pairing clean, the 4 research ULIDs preserved, `restamp` bootstrap sound; `odm check` + `make check` operator-confirmed green. RH-2 Verify pointer → the CDC doc (closer ≠ verifier). New finding **F-17** (Batch 2): re-stamp does not refresh node *bodies* (node #20 keeps pre-amendment `odd/v1.0` text vs its source's `design/v1.0`) — snapshot-by-design vs body-refresh is an open migrate/self-host decision, not C-3. Surfaced by: the C-2 CDC verification.

### v1.5 — 2026-07-26
**C-2 closed (RH-2 attested); F-2/F-3 dispositioned.** Model-first honoured: **ODD-0013 v2.0**
(node types `odd`→`design`, added `research`, tag-based classification, gate-sets) and
**ODD-0020 v1.1** (marker set, re-stamp-on-re-run, no read-alias) landed *before* the code.
Three decisions taken and recorded: (1) **`research` mirrors `design`** — operator call, zero
migration churn, and `RESEARCH_GATES` is a named constant so tightening it later is one line;
(2) **no legacy `odd` read-alias** — hard re-stamp, `check` proves nothing is left; (3) **node
#21 imported** — `docs/design` held **14** documents against the corpus's 13 (`0021-research-…`
was authored after the last migrate), so a re-run also mints it. That met the `CLAUDE.md` G-1
minting freeze, so it was raised rather than resolved silently; the operator chose to import.
Corpus is now **60 nodes: 9 `design` + 5 `research`** + 46 work nodes, `check` green, ids
preserved. Implementation note: the no-alias decision forced a new `odm-migrate::restamp` pass
— once `"odd"` is gone from the enum the old nodes no longer *parse*, so the migration could
not read its own input; the pass rewrites the two dead-type frontmatter lines and hands off to
the typed path. Evidence: `c2-closing-report.md`. Surfaced by: C-2 implementation (CC).

### v1.4 — 2026-07-26
**Reconciled two internal inconsistencies CC flagged at v1.3.** The **C-1 chunk row** still read "Route OPEN" against F-1's "DECIDED — Route B", and the **base-branch bullet** still read "Open item for C-1's cc-prompt" though C-1 had already branched (`rh-c1-adopt-oxur-term`, off `release/1.0.x`). Both updated to the settled state — CDC text, reconciled by CDC; no finding or ledger change. Surfaced by: CC's v1.2/v1.3 close notes.

### v1.3 — 2026-07-26
**F-16 seated (id-collision repair).** CC and CDC wrote the Batch-2 table simultaneously and
both claimed **F-15**. The CDC's entry (retired/superseded nodes undifferentiated → C-3) keeps
the id — ids are stable once published — and CC's finding, *themed tables emit ANSI
unconditionally while status lines degrade off a TTY*, is re-seated as **F-16** (unassigned;
the fix belongs upstream in `oxur-term`). Both findings are live; neither was dropped. The
evidence for F-16 was never lost — it has been in `c1-closing-report.md` §Consequences #1
since the chunk closed. Surfaced by: the operator noticing the collision.

### v1.2 — 2026-07-26
**C-1 close started (RH-1 attested); F-1 dispositioned Route B; F-15 logged.** The styling-crate
route resolved to **B**: `oxur-term` was extracted upstream (table + `common` terminal helpers;
ADR `adr-c1-oxur-table-re-extraction.md`), and C-1 wired odm's tables (`odm-cli/src/table.rs` →
`TableStyleConfig::default().apply_to_table`) and status lines (`term.rs` / `oxur_term::common::output`)
through it, with `oxur-cli` dropped as a dependency. **RH-1 → attested** on CDC structural verify +
a themed-`list` capture (2026-07-26, 59 nodes); flips `reproduced`/`done` on commit + CC
cargo/clippy/fmt + `odm check` green + CI. **F-1 → dispositioned.** New finding **F-15** logged
(Batch 2): `odm list` shows retired/superseded nodes undifferentiated — node 1605's L-2 tombstone
renders as live work; routed to **C-3** (default-exclude + `--all`; status marker via F-7). Disclosed
C-1 deviation: lower-level `Builder`+`apply_to_table` used, not high-level `OxurTable` (theme/footer
gap) — upstream follow-up noted. Surfaced by: C-1 implementation + hands-on validation of the themed
`list`. **Base-branch decision (RH §Dependencies):** work is on `rh-c1-adopt-oxur-term`.

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
