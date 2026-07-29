---
id: 01KYP5FZTJ7KQKKSD4CBK6ZNYR
number: 567337700
type: artifact
schema: artifact/v1.1
name: C-2 closing report — Type taxonomy (`odd`→`design` + `research`)
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/c2-closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# C-2 closing report — Type taxonomy (`odd`→`design` + `research`)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-2 · **Covers:** `F-2`, `F-3` · **Feeds:** RH-2
> **Assignment:** `cc-prompt-c2-type-taxonomy.md` · **Amendments:** `C-2-amendment-ODD-0013.md`,
> `C-2-amendment-ODD-0020.md` (both folded in **before** the code — the model-first rule)
> **Implemented by:** CC · **Date:** 2026-07-26 · **Branch:** `rh-c2-type-taxonomy`
> (off `release/1.0.x` @ `dc65082`, the C-1 tip) · **Evidence class:** attested-by-CC
> (local 1.85+); cargo rows reproduce on CI.

## Decisions taken at start

1. **`research` gate-set: mirrors `design`** (operator, 2026-07-26 — the recommended default).
   Recorded in ODD-0013 §5.1 with its rationale: the importer maps a source doc's state
   directory onto a gate reach, so a shared sequence lets a research doc sit in any state
   directory with no bespoke mapping. `RESEARCH_GATES` is a *named* constant equal to
   `DESIGN_GATES`, so tightening it later is a one-line change, not a refactor.
2. **No legacy `odd` read-alias** (ODD-0020 §4, option A). `"odd"` returns the ordinary parse
   error after C-2; the corpus is hard re-stamped in the same change and `check` proves nothing
   is left behind.
3. **Node #21 imported** (operator, 2026-07-26) — see "The node-count question" below.

## Amendments (landed first)

| Doc | Version | What changed |
|-----|---------|--------------|
| **ODD-0013** | 1.9 → **2.0** | §2.2 node types: `odd` → `design`, added `research`, and the tag-based classification rule; §5.1 gate-sets `[gates.odd]` → `[gates.design]` + `[gates.research]` with the mirror rationale; §9 migration table. Version-history section added (the doc had none). |
| **ODD-0020** | 1.0 → **1.1** | Marker set: `odd/v1.0` removed, `design/v1.0` + `research/v1.0` added; §4 gained the **re-stamp-on-re-run** rule and the no-read-alias decision. Its own `tags: [change-me]` placeholder fixed. |

## Code

### `odm-core`

- **`node_type.rs`** — `NodeType::Odd` → `Design`; `NodeType::Research` added. Every arm
  updated: `as_str`, `FromStr`, `is_document`, `valid_child_types`, serde, doc comments. The
  `FromStr` doc states plainly that `"odd"` is *not* accepted and why.
- **`schema.rs`** — marker docs/tests moved to `design/v1.0`; `design`/`research` are valid
  per-type markers by construction (the marker parses a `NodeType`).

### `odm-migrate`

- **`mapping.rs`** — `ODD_GATES` → `DESIGN_GATES`, `RESEARCH_GATES` added;
  `canonical_odd_gates` → `canonical_design_gates` + `canonical_research_gates`. New
  **`classify_type(tags) -> NodeType`**: `research` iff the tags contain `research`
  (case-insensitive), else `design`. New **`DocGates`** carries both sets so `build_node`
  validates a node's reach against the set that governs *its* type — the one place that would
  notice if the two sequences ever diverge.
- **`restamp.rs` (new)** — the taxonomy re-stamp. See below; this is the substantive addition.
- **`lib.rs`** — `migrate_with_gates` now takes `DocGates`; `existing_odd_numbers` →
  `existing_doc_numbers` (filters `Design | Research`); `Created` carries the classified
  `node_type` so the report says `research 01KY…` instead of a hardcoded `odd`.

### `odm-cli`

- `migrate.rs` resolves `[gates.design]` and `[gates.research]` independently, each falling
  back to its canonical sequence.
- `new`'s help lists `project|arc|slice|design|research|adr|note`.

### Config + source data

- `odm.toml`: `[gates.odd]` → `[gates.design]` (sequence unchanged) + `[gates.research]`.
- Placeholder `tags: [change-me]` fixed on **0011** and **0021** (both research — they would
  otherwise have misclassified as `design`) and on **0012** (a design doc; the amendment
  flagged it as non-critical but it is fixed too). No `change-me` remains in `docs/design`.

## The re-stamp: why it needed its own module

Step 7 of the prompt allowed for it — *"if migrate does not update type/schema in place on
re-run, add the in-place re-stamp"* — but the shape of the problem is worth recording, because
it is a direct consequence of decision 2 above:

**A hard re-stamp with no read-alias cannot be done by an ordinary typed pass.** Once `"odd"`
is gone from the enum, an on-disk `type: odd` node does not parse — so `store.load_all()`
fails, and *nothing* downstream (the schema backfill, the idempotence scan, the import) can
read the corpus it is supposed to fix. The migration cannot read its own input.

`restamp.rs` resolves that by doing the **minimum raw-text work to unblock parsing** — it
rewrites the two frontmatter lines that carry the dead type name, `type:` and `schema:` — and
then hands off to the ordinary typed path for everything else (stamp the schema, refresh tags,
persist canonically). It is a line rewrite, not a YAML round-trip: no other line, and no part
of the body, is touched by it. Two properties follow, both tested:

- the body is untouched — node #20's body legitimately contains the string `odd/v1.0` (it *is*
  ODD-0020) and survives verbatim;
- identity is untouched — id, number, gates and edges all carry over.

**Classification uses the source doc's tags, not the node's**, because a node's tags were
copied at import time and can predate a correction to the source — exactly the case here, where
0011 carried `change-me`. Falls back to the node's own tags when no source doc matches.

**`--dry-run` needed an overlay.** Under dry-run nothing is written, so the files still carry
the dead type and the rest of the preview would fail on the very state it is proposing to fix.
`corpus_after_restamp` reads the corpus through the re-stamp's in-memory documents, so the
preview reports what *would* happen. With nothing to re-stamp it is exactly `load_all()`.

## The node-count question (raised before acting)

The prompt and the ODD-0013 amendment both expected **13 document nodes → 9 design + 4
research**. The corpus has 13 nodes but `docs/design` holds **14** documents: `0021-research-…`
was authored after the last migrate and had never been imported. Re-running `migrate`
therefore also *creates* node #21 — and `CLAUDE.md` freezes node minting until the **G-1**
ID-scheme decision.

Raised rather than silently resolved either way. **Operator decision: import it.** The corpus
is meant to mirror the doc set, so leaving 0021 out means `odm list` under-reports it; and if
G-1 changes the ID scheme, re-keying is a bulk pass where 60 nodes versus 59 costs nothing.

Result: **14 document nodes (9 `design` + 5 `research`)**, 60 total. The amendment's "research
set = 0011, 0014, 0016, 0018" holds for the pre-existing nodes; #21 is the fifth, and its tags
were fixed in the same change so it classified correctly on first import.

## Verification

All local, Rust 1.85+ (attested-by-CC):

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **52 binaries ok, 0 failed** (incl. 5 new `restamp` unit tests) |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added |

**Rehearsed on a scratch copy first.** The whole re-stamp was run against a copy of the real
`nodes/` + `docs/` before the live corpus was touched — dry-run, then commit, then `check`.

**On the real corpus:**

```
$ odm migrate docs/design
✓ migrate: 1 created, 13 upgraded, 13 skipped, 0 warning(s)
$ odm self-host docs/design-v1.0.0
✓ self-host: 0 created, 46 skipped          # unchanged by C-2
$ odm check
✓ check: ok (60 node(s), no problems)
```

**Acceptance evidence:**

- **No `odd` remains.** Every node's *frontmatter* scanned for `type: odd` / `schema: odd/`:
  zero hits. (A naive `grep -rl 'odd/v1.0' nodes` matches one file — node #20's **body**, which
  is the text of ODD-0020 itself. Frontmatter is what the acceptance is about.)
- **Type/schema pairing is consistent** across all 60 nodes:

  | count | type | schema |
  |---|---|---|
  | 39 | `slice` | `slice/v1.0` |
  | 9 | `design` | `design/v1.0` |
  | 6 | `arc` | `arc/v1.0` |
  | 5 | `research` | `research/v1.0` |
  | 1 | `project` | `project/v1.0` |

- **Identity preserved.** The four pre-existing research nodes kept their original ULIDs
  (`#11 01KWWGS8HD25CQE3BX5FQEW84Q`, `#14 …P058PPZN2YKD1K88`, `#16 …7T0FXYB88KKKCC07`,
  `#18 …H9B0BZXEXMNQC3ZZ`) — the re-stamp changed type and schema, nothing else.
- **Capture:** `c2-capture-odm-list.ansi` (`cat` it in a truecolor terminal) holds
  `odm list --type design`, `--type research` and `odm check` — F-2/F-3 visibly resolved, no
  `odd` in the TYPE column.

## Worth the operator's attention

1. **Re-stamping does not refresh node bodies.** `migrate` re-stamps frontmatter; a node's body
   stays the snapshot taken at import. Node #20's body is therefore the *pre-amendment* text of
   ODD-0020. That is consistent with the model (the source doc is the source; the node is a
   snapshot), but if bodies should track their sources, that is a separate decision — not
   raised as an F-row because it predates C-2 and is arguably by design.
2. **The re-stamp is single-use by nature.** It exists to remove a type name that no longer
   exists. Once every corpus is re-stamped it is dead weight, and it can be dropped in a later
   cleanup — `restamp.rs`'s header says so.
3. **C-3 is now unblocked**: the type names are settled, and `F-15` (default-exclude retired
   nodes + the status column) lands there.

## Ledger

- **`RH-2` (C-2 closed)** — ready to close: **attested** on this report; **reproduced** when CI
  runs the cargo rows green.
- **`F-2`** (`odd` → `design`) and **`F-3`** (add `research`) — dispositioned: **shipped**.
- Bubble-up to `arc-plan.md`: the `research` gate-set decision (mirror) and the no-read-alias
  decision recorded; the node-count/#21 decision recorded; RH-2 attested.
