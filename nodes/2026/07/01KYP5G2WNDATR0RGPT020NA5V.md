---
id: 01KYP5G2WNDATR0RGPT020NA5V
number: 525903300
type: artifact
schema: artifact/v1.1
name: Arc closing report — Release Hardening (`arc-release-hardening`)
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# Arc closing report — Release Hardening (`arc-release-hardening`)

> **Realizes:** the UAT punch-list (F-1…F-21) + the routed L/G findings · **Chunks:** C-1…C-8, all
> closed (C-7 retired into C-5) · **Date:** 2026-07-27 · **Written by:** CC
> **Status:** **complete** — RH-6 reproduced, RH-7 attested, RH-8 dispositioned.
> **Evidence class:** attested-by-CC on a real toolchain (local 1.85+, git 2.39.5); every cargo row
> is **not yet CI-reproduced** — see *Verification* for why that is honest rather than pending.

## What the arc set out to do

odm reached v1.0.0 feature-complete and was then **used**, by a human operator and by an LLM session,
against its own self-hosted corpus. Two UAT passes produced a punch-list — **F-1…F-21** from the human
pass, **L-1…L-9** from the LLM pass, and **G-1…G-12** from a workflow-gap review — and this arc's job
was to work that list down to a surface fit to ship.

The organizing decision was to **triage surface from model**. Most findings were presentation
(colour, column layout, command names); a few were data or schema (creation dates, node names, the
store's home); one was a decision the project could not proceed past (the ID scheme, G-1). Chunks were
cut so that each churned one thing once — the command surface changes in a single pass, the corpus is
rewritten a single time — rather than letting the same file be edited by three chunks in a row.

**Scope grew once, deliberately.** arc-store-home's dogfood cutover (SH-6) was folded into C-5 because
C-5 already rewrote the corpus, and doing both in one pass meant writing it once. That is why an arc
about hardening ends up owning odm's move onto its own store branch.

## Per-chunk walk

| Row | Chunk | Outcome | Evidence |
|-----|-------|---------|----------|
| **RH-1** | **C-1** — theming | odm's tables and status lines run through **`oxur-term`** (a themed-table crate re-extracted for this purpose); `oxur-cli` shed entirely. Warm-orange theme, per-state colour. | `C-1-cdc-verification.md`, `c1-closing-report.md`, ADR `adr-c1-oxur-table-re-extraction.md` |
| **RH-2** | **C-2** — type taxonomy | `odd` → **`design`**; **`research`** added. Model first (ODD-0013 v2.0, ODD-0020 v1.1), then a re-stamp of the live corpus. | `C-2-cdc-verification.md`, `c2-closing-report.md` |
| **RH-3** | **C-3** — `list` overhaul | F-4…F-9 + F-15: number column dropped, date column added (`--date`), STATUS column, de-numbered names, branch-and-leaf containment tree, width bound, retired/superseded withheld by default. | `C-3-cdc-verification.md`, `c3-closing-report.md` |
| **RH-4** | **C-4** — command surface | **ODD-0023 Accepted (v1.1)** then the code: three tiers (top-level workflow verbs / `odm node` / `odm store`), F-10…F-13 renames, and the **`validate`/`check` split**. Hard cut, no aliases. | `c4-closing-report.md` |
| **RH-5** | **C-5** — fold + cutover | One corpus rewrite: `self-host` folded into `migrate` (F-14), names normalized (F-18), **real git dates** (F-20), the project's `# Vision` body (L-3a), **and the SH-6 relocation** onto the orphan `odm` branch. Every ULID preserved. | `c5-closing-report.md` |
| **RH-9** | **C-8** — normalized status | STATUS shows position in the node's own ladder — `planned`/`active`/`done` — so a done arc and a done slice read alike. Also gave `show`/`--json` the gate ladder, which **delivered the LLM arc's blocking slice 01 (L-1)**. | `c8-closing-report.md` |
| **RH-10** | **C-6** — `validate` hardening | G-3 (`undecomposed-parent`, warn-by-default) and L-3b (`no-vision`). G-2 turned out to be **already implemented**. | `c6-closing-report.md` |
| — | **C-7** — name normalization | **Retired into C-5**, which already rewrote the corpus. Recorded, not dropped. | `cc-prompt-c7-name-normalization.md` |

## Composition verdict

**RH-6 — reproduced.** The whole RH surface in one pass on the self-hosted store, every chunk's work
visible and coherent. Re-run independently at `ba6aa45` while writing this report:

```
orient        VISION renders (L-3a) · CURRENT FOCUS = arc #1600 (C-5's focus fix)
              READY lists 5 design nodes (C-2) · INTEGRITY ok
project       #1600 …                       (C-4 rename of `context`)
chain 1600    #1600 Migrate, self-host …    (C-4 rename of `path`)
validate      0 error(s), 2 warning(s)      exit 0   (C-6's G-3 warns; C-4's pure verb)
check         validate + reconcile          exit 0   (C-4's inversion)
node list     DATE│TYPE│STATUS│NAME-tree│ID
              warm-orange theme (C-1) · design/research/project/slice types (C-2)
              branch-and-leaf tree + de-numbered names (C-3) · `odm node` group (C-4)
              real dates from 2026-06-20 (C-5/F-20) · no `(plan-of-record)` (C-5/F-18)
              normalized active/done STATUS (C-8)
```

**RH-7 — attested.** The reflexive loop: the C-5 cutover re-derived odm's own corpus into the store
home, `check`-green at **60 nodes**, with `design`/`research` types and de-numbered names. Recorded
nuance, carried from C-5: the corpus was **re-stamped in place, not minted fresh** — a truly fresh
re-derivation would mint new ULIDs, break every edge, and trip the G-1 freeze. The loop validates the
*derivation logic* via the re-stamp; that is the strongest form available until G-1 is decided.

**RH-8 — dispositioned.** Every finding accounted for below.

## The findings ledger, closed out

**21 F-rows.** 18 done, 3 carried with reasons:

- **F-1** dispositioned (Route B: `oxur-term`); **F-2…F-15, F-18, F-19, F-20** done across C-1…C-8.
- **F-16** — themed tables emit ANSI unconditionally. **Carried, unassigned.** The right home is a
  `NO_COLOR`/TTY guard **upstream in `oxur-term`**, so odm and oxur agree rather than each growing a
  local rule. `--json` is the machine path today, so this is not release-blocking.
- **F-17** — re-stamp does not refresh node bodies. **Carried as an open question**, not a defect:
  snapshot-by-design (source doc = editable truth, node = the build) versus refresh-on-re-migrate. It
  needs a decision, not code.
- **F-21** — `--json` omits `created`/`updated`. **Routed to the LLM arc**, because it is a `--json`
  contract change and belongs with that arc's read-back work rather than being slipped in unannounced.

**Routed L rows.** L-3a (vision) done in C-5; L-3b (no-vision rule) done in C-6; **L-1 (status is
write-only) delivered by C-8** — see below; L-9 doc-addressed in ODD-0013 §2.1; L-2's retired-node
half done as F-15; L-4/L-5/L-7 planned in the LLM arc; L-6 reclassified to release-engineering;
**L-8b (reconcile 4 ODDs) remains a pre-ship gate** and is carried forward.

**Routed G rows.** G-2 and G-3 done in C-6; G-7 routed to arc-store-home and largely resolved by
design; **G-1 (the ID scheme) is still pending and still gates minting any new node** — unchanged by
this arc, and C-5 was explicitly built to preserve every ULID so it would not trip that freeze. G-4 =
L-1, delivered. G-5/G-6/G-8…G-12 queued beyond 1.0.

## What the arc found in itself

The honest half of the record: **this arc's own work surfaced ten defects that its chunks then fixed.**
They cluster into three shapes, and the shapes are more useful than the list.

**1. A path that could not be exercised where it was written.** The C-5 dogfood trio is the clearest
run of these, and all three were invisible until odm actually moved onto its own store:

- `odm use arc X` printed `✓`, wrote the file, and `odm orient` still reported *"(no current arc)"* —
  `use` resolved the **store** root, `orient` the **invocation** root. Identical for every repo and
  every test, until the two parted for the first time.
- `store init` scaffolded no `.gitignore`, so a store branch offered its derived index for commit.
- The first fix for that ignored all of `.odm/`, which would have withheld `context.json` — the
  current focus — from every fresh clone, defeating the project's own success test.

**2. A premise nobody had checked.** Three chunks were briefed on facts that were not true:

- **C-8** was told `show`/`--json` already carried the raw gate ladder, justifying a normalized-only
  column. Neither reported gates **at all** — so normalizing alone would have made the rung
  unreachable from the CLI. Fixing that *delivered L-1*, the LLM arc's blocking slice, as a side
  effect of checking a sentence.
- **C-6** was told the tear rationale was a data-loss bug with no schema field. The field existed,
  round-tripped, and was already documented in ODD-0013 §4.3 — the amendment had nothing to amend.
- **C-6** was also told odm's corpus had no `decomposed` assertions and would light up. Five of seven
  parents had them; the real gap was two nodes.
- **C-4**'s inventory-vs-`--help` parity check found a documented `store sync` command **that was
  never built**, `store init` help stale since slice 03, and a dead `self-host` wrapper.

**3. Success and failure looking identical.** C-3's tree-glyph bug — a `├─` pointing at a row filtered
out after the tree was derived — was caught by the operator reading output, not by a test. The fix was
to filter *before* deriving, and the regression tests were confirmed failing against the unfixed
source before being accepted.

The through-line the arc-store-home report named — **assert both halves** — held here too, and grew a
companion: **check the brief's facts before implementing them.** Four of the ten came from reading the
corpus or running the binary for ninety seconds before writing code.

## Verification

| Check | Result |
|-------|--------|
| `cargo test --all-features --workspace` | **58 binaries, 0 failed** |
| `cargo clippy --all-targets --workspace --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added across the arc |
| odm's own corpus | `validate` **exit 0**, 60 nodes, 2 expected G-3 warns; `check` exit 0 |
| ids | all 60 pre-cutover ULIDs preserved through C-5 |

**On `reproduced`, stated plainly:** every cargo row above is **attested-by-CC on a real toolchain, not
CI-reproduced.** `release/1.0.x` is **38 commits ahead of `origin`** and unpushed; `origin` is SSH, so
pushing is the operator's action, not mine. The durable `reproduced` for all seven chunks rides that
push. Nothing here is blocked on it — but no one should read these rows as CI-green, because they are
not yet.

## Carried forward — recorded, not dropped

- **`store status`** (ODD-0023 §7) — "where is my store / is it synced": a `store` verb, or folded into
  `orient`? Explicitly non-blocking; nothing depends on the answer and no such command exists.
- **`context.json` is shared, not per-user.** That is what lets a fresh session inherit the focus (the
  project's success test), but two operators on different arcs would contend over one line. A two-level
  model — shared default, local override — is the post-1.0 shape.
- **#1600 must not affirm its decomposition until A6 closes.** C-6's G-3 rule warns on it, but
  affirming a parent still gaining children is what `decomposition-drift` — an **error** — catches
  later. **#1000 is the opposite case**: its six arcs are settled, so `odm node decomposed 1000` clears
  that warn honestly today.
- **L-8b — reconcile the four ODDs** before ship. A release-gate checklist item, still open.
- **The LLM arc's L-1 remainder** — `by`/`evidence_dates` fields, enumerating **unreached** gates in
  `--json`, and the satisfaction/weakest-link verdict. C-8 delivered the core; the arc's v1.2 records
  what is left.
- **F-16 / F-17 / F-21** as dispositioned above.
- **G-1 remains the gate on minting any new node.** Unchanged by this arc, and deliberately so.

## Bubble-up

- **`project-plan.md` §2a** — Release Hardening → **closed**; **§3** — the surface is **settled**.
- **Sequencing** — RH (closed) → **LLM command surface** (materially lighter: C-8 delivered its
  blocking slice 01) → **A6 resumes at slice05** (PM-skill) and slice06, now targeting a settled surface.
- **P-13** (bubble-up dispositioned) records RH's close. RH has no P-row of its own — it is a v1.0.x
  hardening arc, not an A-numbered DoD arc — but the **DoD surface is now settled for the P-7
  situational-awareness demo** at project close.

## Silent-drop diff

None. Everything scoped in shipped or is listed above with a reason. C-7 retired into C-5 by decision,
recorded at the time.
