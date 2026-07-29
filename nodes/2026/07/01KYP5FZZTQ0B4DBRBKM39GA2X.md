---
id: 01KYP5FZZTQ0B4DBRBKM39GA2X
number: 590538600
type: artifact
schema: artifact/v1.1
name: C-3 closing report — `odm list` overhaul
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/c3-closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# C-3 closing report — `odm list` overhaul

> **Arc:** Release Hardening (UAT) · **Chunk:** C-3 · **Covers:** `F-4`…`F-9` **+ `F-15`**
> **Feeds:** RH-3 · **Assignment:** `cc-prompt-c3-list-overhaul.md`
> **Implemented by:** CC · **Date:** 2026-07-26 · **Branch:** `rh-c3-list-overhaul`
> (off the C-2 tip `c9d6fd5`) · **Evidence class:** attested-by-CC (local 1.85+); cargo rows
> reproduce on CI.

## Decisions confirmed at start

1. **`STATUS` = the furthest-reached gate** in the node's own gate-set, `—` when none, with
   `retired`/`superseded` as overrides (operator — the recommended option). One token; the full
   gate vector stays the LLM-command-surface arc's job.
2. **Document nodes render as their own flat group** below the work tree (operator — the
   recommended option). They genuinely have no `part_of` parent, so nothing is invented to put
   them in the tree.

## The shape

```
DATE | TYPE | STATUS | NAME (tree, de-numbered) | ID
```

```
 NODES
DATE       │TYPE     │STATUS      │NAME                                          │ID
 2026-07-07│ project │ in-progress│ odm v1.0.0 — Project Plan (arc roadmap)      │ 01KWXMBBTJCJ…
 2026-07-07│ arc     │ verified   │ ├─ Substrate & node CRUD (plan-of-record)    │ 01KWXMBBTKKT…
 2026-07-07│ slice   │ tested     │ │  ├─ Workspace scaffolding (plan-of-record) │ 01KWXMBBTKSH…
 2026-07-07│ arc     │ in-progress│ └─ Migrate, self-host & PM-skill             │ 01KWXMBBTKNA…
 2026-07-07│ slice   │ tested     │    └─ `migrate` importer core                │ 01KWXMBBTKPJ…
 ──────────┼─────────┼────────────┼──────────────────────────────────────────────┼───────────── 
 2025-12-27│ design  │ final      │ Oxur Design Documentation CLI - Build Plan   │ 01KWWGS8HDHK…
 2026-06-20│ research│ final      │ Research: A markdown/git-native, dependen ...│ 01KWWGS8HD25…
 Total: 59 node(s) shown — 1 filtered or withdrawn (--all shows every node)
```

The two groups are separated by a **rule**, not a label (operator's call during review — a
`── work ──` header row was redundant with the tree itself). The rule crosses the column
separators at `┼` and is drawn in the *separator* colour, so it and the verticals read as one
grid rather than as a row of content; it keeps the same leading and trailing space every other
row has. It is emitted as a single spanned cell, because the separator between cells is
tabled's to draw and would otherwise lay a `│` through it.

## Findings, one by one

| Finding | What shipped |
|---------|--------------|
| **F-4** drop NUMBER | Column gone. `number` stays frontmatter metadata and a CLI handle (`odm show 13` still works); the ULID is identity. |
| **F-5** DATE first | Leftmost column, showing `created`; **`--date={created\|updated}`** switches it. |
| **F-7** STATUS after TYPE | The furthest-reached gate, ordered by the **configured sequence** — not by the record's gate list, which the index stores gate-name sorted (alphabetical, not chronological). `—` when nothing is reached. |
| **F-8** branch-and-leaf tree | Work nodes render by `part_of` containment with `├─`/`└─`/`│` glyphs; document nodes follow below a full-width rule. Name-prefixing is gone. |
| **F-6** de-numbered names | `"Slice 05 (Arc 06): UAT — CLI feedback"` renders as `"UAT — CLI feedback"`. **Display-only** — no stored `name` was rewritten. The convention *"names don't embed numbers"* is now ODD-0013 §2.1 (v2.1). |
| **F-9** width + elision | **`--width`** flag and **`[display] max_width`** in `odm.toml` (default 64); longer names are cut at `width − 4` and marked ` ...`, so the cell lands exactly on the limit. |
| *(added in review)* `--group` | **`--group plan\|reference`** narrows to one family — the work tree, or the design/research/adr/note material. Display names for the model's *work*/*document* families, paired in ODD-0013 §2.2 (v2.2). |
| *(added in review)* STATUS palette | The STATUS cell carries the **`oxur-odm` 0.3.5 state colours** — `draft` yellow, `under-review` cyan, `revised` blue, `accepted`/`final` green, `active` bright green, `deferred` magenta, `rejected`/`withdrawn`/`superseded` red — reused via the same `oxur_term::table::helpers::state_to_fg_color` the original called, on the same column, colour only (0.3.5 used no weight there). The **work sequences postdate that palette**, so they were mapped onto its *slots*: `planned` yellow, `in-progress`/`built` cyan, `complete`/`tested` green, `verified` bright green. |
| *(added in review)* TYPE palette | One hue per node type, so a long listing sorts by kind without being read: `project` magenta, `arc` violet, `slice` blue, `design` orange, `research` red. Truecolor rather than the basic ANSI slots — violet and orange have no basic slot, and the values are tuned to stay legible on the theme's `#451A03` band. `design` and `research` are the **same** saturation and luminosity (HSL 84.7% / 61.6%), differing only in hue, so the pair reads as one family at one weight. `adr`/`note` are left uncoloured rather than being assigned a hue by omission. |
| *(added in review)* muted DATE/ID | The date and the id render at ~58% of their row's band colour, so TYPE / STATUS / NAME carry the eye. Derived **from the theme's own row colours** rather than fixed, so the alternating stripe survives — two bands in, two bands out — and a theme change carries through instead of silently desynchronising. |
| **F-15** retired/superseded | **Excluded by default**; `--all` (alias `--include-retired`) brings them back with STATUS `retired`/`superseded` and the row **dimmed**; **`--status <VALUE>`** shows only the rows at one status — `--status retired` is exactly the set a default listing withholds. |

## The index needed two fields

`list`'s human path is index-backed (A4 slice04) and must stay that way — no full corpus
parse. Two C-3 columns had nothing to read:

- **`created`** — the index carried only `updated`. It is *not* derivable from the ULID here: a
  migrated node's `created` is its **legacy** date while its id was minted at import, so the two
  genuinely differ (node #2 reads `2025-12-27`, its ULID is from 2026-07).
- **`retired`** — a flag, not the whole `Retirement`: `list` needs only "is this live work?",
  and a consumer wanting the reason reads the node.

Both were added to `IndexRecord` and **`FORMAT_VERSION` bumped 3 → 4**. That is the sanctioned
path: the index is a derived cache, an older snapshot loads as
`RebuildNeeded(VersionMismatch)` and is rebuilt cold. The version sentinel test now asserts 4
and records the bump history, so no future bump can land silently.

`part_of` and `Supersedes` were already in the record's edges, so the tree and the superseded
set needed no further enrichment.

## A flaw found in review, and fixed

The first cut derived the tree and *then* dropped withdrawn rows. But `├─` versus `└─` encodes
**"last among siblings"**, and root-ness encodes **"my parent is on screen"** — both are
properties of the *visible* set, not the stored one. Filtering afterwards therefore left stale
structure: with #1605 hidden, `self-host cutover` kept a `├─` pointing at a row that was not
rendered.

```
before                              after
  ├─ schema versioning                ├─ schema versioning
  ├─ self-host cutover   ← dangling   └─ self-host cutover   ← closes the branch
── documents ──                     ── documents ──
```

Fixed by applying **every** row filter before the structure is derived. The same flaw had a
second, latent face: hiding a *parent* would have left its children indented under an absent
row. Both are now regression-tested (`hiding_the_last_child_promotes_its_previous_sibling…`,
`hiding_a_parent_reroots_its_children`), and both tests were confirmed to fail against the
unfixed source — the pre-existing F-15 tests passed either way, which is why the bug survived
the first pass. Spotted by the operator reading the rendered output.

## Notes worth the operator's attention

1. **The summary line says what it is not showing.** With rows withheld it reads
   `Total: 59 node(s) shown — 1 filtered or withdrawn (--all shows every node)`. Hiding rows
   silently is exactly the L-2 hazard that produced F-15 in the first place, so the view states
   its own omission rather than letting a reader infer the corpus is smaller than it is.
2. **`--group plan|reference` lists one family at a time.** Added on operator request. The
   model names these families *work* and *document* (ODD-0013 §2.2); `plan`/`reference` are
   their **display** names — chosen because "document" reads as general English even though it
   is precise in odm, and because the second family is what the plan is *grounded in*, not what
   is executed. The pairing is recorded in ODD-0013 §2.2 (v2.2) so the two vocabularies cannot
   drift apart unnoticed — which is exactly what F-2 caught with `odd`.
3. **The state palette is reused, not restated.** `state_to_fg_color` is the *same function*
   0.3.5 called — it moved from `oxur-cli` to `oxur-term` in C-1 — so the document colours
   cannot drift from the original by being retyped.

   The **work sequences did not exist** when that palette was written, so they were mapped onto
   its slots rather than given colours of their own (operator decision):

   | Slot | Document | Work | SGR |
   |------|----------|------|-----|
   | recorded, not started | `draft` | `planned` | 33 yellow |
   | under way | `under-review` | `in-progress`, `built` | 36 cyan |
   | done at its layer | `accepted`, `final` | `complete`, `tested` | 32 green |
   | strongest evidence | `active` | `verified` | 92 bright green |

   Keeping `complete` green and `verified` bright green is the point: ODD-0013 §5.1 split
   "done at its layer" from "verified live" precisely because collapsing them hid a production
   failure, and the colours should not re-collapse what the gate model separates. Note the
   terminal gate differs by type — `verified` for project/arc, `tested` for slice, `final` for
   design/research — so "green" does not always mean "finished"; it means done at that node's
   layer. The two palettes are disjoint, so the document one always wins where it applies. On a
   withdrawn row the **dimming wins** over the status colour, greying the row being the
   stronger signal.
4. **Two palettes, two questions.** TYPE answers *what kind of thing is this?* and STATUS
   answers *how far along is it?*, so they are deliberately drawn from different colour
   systems: TYPE in truecolor hues, STATUS in the basic ANSI slots 0.3.5 used. A row can
   therefore carry both without the eye having to disentangle which colour means what. On a
   withdrawn row **both** give way to the dimming.
5. **`--status <VALUE>` answers "show me what was hidden".** Added on operator request during
   review: the summary said a row had been withheld but gave no way to look at it. The flag
   filters on the rendered STATUS token and implies `--all` for a withdrawn value, so
   `--status retired` lands directly on the withheld set. It generalises past F-15 —
   `--status built` narrows to work at one gate — and, like every other filter, is applied
   before the tree is derived.
6. **`--json` is deliberately unfiltered by `--all`.** The machine path emits every node and its
   `retired` field and lets the consumer decide; the default-hiding is a human-view affordance.
   That also keeps `--json` a stable contract for the LLM-surface arc.
7. **A filtered view still renders.** `--type slice` would otherwise vanish into an empty tree
   (its parents are filtered out), so a node whose parent is not in the shown set becomes a root
   of that view — flat, not missing.
8. **F-15's superseded half has no corpus instance.** The audit found **exactly one** retired
   node (#1605) and **zero** supersedes edges. The prompt's "coarse scan flagged ~5 superseded
   ODDs" was a false positive: those five files match the *word* "superseded" in their bodies
   (ODD-0013's own frontmatter example, and prose). Superseded-exclusion is implemented and
   unit-tested, but it is exercised by fixtures, not by the live corpus.
9. **Names still carry "(plan-of-record)" suffixes.** Not a number-reference, so out of F-6's
   scope; if those should go too, that is a data change for a re-`self-host`, not a display rule.

## Verification

All local, Rust 1.85+ (attested-by-CC):

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **53 binaries ok, 0 failed** (+21 new `listview` CLI tests, +6 new unit tests) |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added |

**Corpus (a view change only — counts unmoved):**

```
$ odm check
✓ check: ok (60 node(s), no problems)
$ odm list        # default
 Total: 59 node(s) shown — 1 filtered or withdrawn (--all shows every node)
$ odm list --all
 Total: 60 node(s)
```

**F-15 confirmed on the real corpus:** the default listing omits **#1605**
(`Slice 05 (Arc 06): UAT — CLI feedback`, the L-2 tombstone); `odm list --all` shows it as
`retired`, de-numbered to `UAT — CLI feedback`, and dimmed (`ESC[90m` on every cell — asserted
in the test suite against a live row, which carries no such code).

**Capture:** `c3-capture-odm-list.ansi` — the default view, `--all --width 44`, and
`--type slice --date updated`, plus `odm check`. `cat` it in a truecolor terminal.

## Ledger

- **`RH-3` (C-3 closed)** — ready to close: **attested** on this report; **reproduced** when CI
  runs the cargo rows green.
- **`F-4`, `F-5`, `F-6`, `F-7`, `F-8`, `F-9`, `F-15`** — dispositioned: **shipped**.
- Bubble-up to `arc-plan.md`: the STATUS and doc-node decisions; the index format bump; the
  F-15 audit result (one retired node, zero supersedes edges — the ~5 was a false positive).
- Unaffected and still open: **F-16** (unconditional table ANSI), **F-17** (body-snapshot
  drift). Remaining chunks: **C-4** (command surface) and **C-5** (fold `self-host` into
  `migrate`).
