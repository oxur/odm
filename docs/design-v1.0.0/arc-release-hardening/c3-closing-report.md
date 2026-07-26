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
           │         │            │ ── work ──                                   │
 2026-07-07│ project │ in-progress│ odm v1.0.0 — Project Plan (arc roadmap)      │ 01KWXMBBTJCJ…
 2026-07-07│ arc     │ verified   │ ├─ Substrate & node CRUD (plan-of-record)    │ 01KWXMBBTKKT…
 2026-07-07│ slice   │ tested     │ │  ├─ Workspace scaffolding (plan-of-record) │ 01KWXMBBTKSH…
 2026-07-07│ arc     │ in-progress│ └─ Migrate, self-host & PM-skill             │ 01KWXMBBTKNA…
 2026-07-07│ slice   │ tested     │    ├─ `migrate` importer core                │ 01KWXMBBTKPJ…
           │         │            │ ── documents ──                              │
 2025-12-27│ design  │ final      │ Oxur Design Documentation CLI - Build Plan   │ 01KWWGS8HDHK…
 2026-06-20│ research│ final      │ Research: A markdown/git-native, dependen ...│ 01KWWGS8HD25…
 Total: 59 node(s) shown — 1 filtered or withdrawn (--all shows every node)
```

## Findings, one by one

| Finding | What shipped |
|---------|--------------|
| **F-4** drop NUMBER | Column gone. `number` stays frontmatter metadata and a CLI handle (`odm show 13` still works); the ULID is identity. |
| **F-5** DATE first | Leftmost column, showing `created`; **`--date={created\|updated}`** switches it. |
| **F-7** STATUS after TYPE | The furthest-reached gate, ordered by the **configured sequence** — not by the record's gate list, which the index stores gate-name sorted (alphabetical, not chronological). `—` when nothing is reached. |
| **F-8** branch-and-leaf tree | Work nodes render by `part_of` containment with `├─`/`└─`/`│` glyphs; document nodes follow under a `── documents ──` header. Name-prefixing is gone. |
| **F-6** de-numbered names | `"Slice 05 (Arc 06): UAT — CLI feedback"` renders as `"UAT — CLI feedback"`. **Display-only** — no stored `name` was rewritten. The convention *"names don't embed numbers"* is now ODD-0013 §2.1 (v2.1). |
| **F-9** width + elision | **`--width`** flag and **`[display] max_width`** in `odm.toml` (default 64); longer names are cut at `width − 4` and marked ` ...`, so the cell lands exactly on the limit. |
| **F-15** retired/superseded | **Excluded by default**; `--all` (alias `--include-retired`) brings them back with STATUS `retired`/`superseded` and the row **dimmed**. |

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

## Notes worth the operator's attention

1. **The summary line says what it is not showing.** With rows withheld it reads
   `Total: 59 node(s) shown — 1 filtered or withdrawn (--all shows every node)`. Hiding rows
   silently is exactly the L-2 hazard that produced F-15 in the first place, so the view states
   its own omission rather than letting a reader infer the corpus is smaller than it is.
2. **`--json` is deliberately unfiltered by `--all`.** The machine path emits every node and its
   `retired` field and lets the consumer decide; the default-hiding is a human-view affordance.
   That also keeps `--json` a stable contract for the LLM-surface arc.
3. **A filtered view still renders.** `--type slice` would otherwise vanish into an empty tree
   (its parents are filtered out), so a node whose parent is not in the shown set becomes a root
   of that view — flat, not missing.
4. **F-15's superseded half has no corpus instance.** The audit found **exactly one** retired
   node (#1605) and **zero** supersedes edges. The prompt's "coarse scan flagged ~5 superseded
   ODDs" was a false positive: those five files match the *word* "superseded" in their bodies
   (ODD-0013's own frontmatter example, and prose). Superseded-exclusion is implemented and
   unit-tested, but it is exercised by fixtures, not by the live corpus.
5. **Names still carry "(plan-of-record)" suffixes.** Not a number-reference, so out of F-6's
   scope; if those should go too, that is a data change for a re-`self-host`, not a display rule.

## Verification

All local, Rust 1.85+ (attested-by-CC):

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **53 binaries ok, 0 failed** (+9 new `listview` CLI tests, +6 new unit tests) |
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
