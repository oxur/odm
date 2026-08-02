---
id: 01KYP5H2CHHF54CB0FZ833QCW6
number: 561911000
type: artifact
schema: artifact/v1.1
name: Command-surface UAT checklist — the dual sign-off worksheet
created: 2026-07-27
updated: 2026-08-01
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/command-surface-uat-checklist.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-01
---
# Command-surface UAT checklist — the dual sign-off worksheet

> **Purpose.** The shared worksheet for the **command-by-command dual UAT** that gates close-out
> (SKILL writing). Every command that presents data gets **two** sign-offs, both required:
> **Human (Duncan)** — themed / tabled / styled well + the name makes sense; **LLM (CDC)** — helpful,
> informative, correct, and optimised for my use. Close-out is unlocked only when every row is
> **✓ / ✓**.
>
> **Grounding.** Every row below was produced by **running the command against the real self-hosted
> store** (`release/1.0.x` @ `8de6d0d`, 60 nodes) and reading its actual output — not from memory.
> The **LLM column is my first pass, done here** (my half of the dual UAT); **the Human column is
> yours to fill in your terminal** (the colour/theme judgment needs a real TTY — I pipe, so I can see
> *table vs. text* but not the in-terminal colour of status lines).
>
> **Legend.** LLM: **✓** signed (good for my use) · **⚠** flagged (issue named) · **–** n/a.
> Output form: **table** (themed `tabled`) · **struct** (structured text) · **line** (one/few lines) ·
> **md** (generated markdown) · **json** (via `--json`).

## 1. Read / query commands (the priority — these present data)

| Command | Output form | LLM | LLM note (my half) | Human ☐ |
|---|---|---|---|---|
| `orient` | struct (VISION/FOCUS/READY/BLOCKED/INTEGRITY/DRIFT) | ⚠ | Structure is good. Two issues: (a) it dumps the **entire vision body inline** every call — heavy for a command I run reflexively; a 2-line summary + "`node show 1000` for full" would be better (it already elides very long ones). (b) **READY lists the retired `#1605`** — the F-22 leak. | ☐ |
| `next` | `line`s (`#NN name`); `--json` `{node,number,effective_evidence,soft}` | ⚠⚠ | The weakest read command. **No type labels** (`#17` design and `#1600` arc look identical), **no ordering / rationale** (a dependency-ordered tool whose ordering command returns a set), and `--json` **omits the name entirely**. Also lists the retired **#1605**. = UAT **L-4** + **F-22**. | ☐ |
| `blocked <ref>` | line | ⚠ | For a ready node prints `nothing holding #X` — but can't distinguish **ready-and-well-supported** from **ready-on-an-asserted-only-dep**, which is exactly what the evidence ladder is for. The *ready half* is unexplained. = UAT **L-5**. | ☐ |
| `chain <ref>` | line(s) | ⚠ | For a node with no upstream it returns **just the node itself on one line** — indistinguishable from an error or a no-op. Needs an explicit "no dependency chain / this is a root" message. = UAT **L-5**. | ☐ |
| `project` | `line`s (`project: (none)` / `arc: #…`) | ✓ | Clear. Minor: `project: (none)` when only an arc context is set reads slightly oddly, but it's honest. | ☐ |
| `node show <ref>` | struct (id / dates / **status + gate ladder** / part_of / children) | ✓ | **Exemplary — the model the others should match.** The C-8/L-1 gate ladder (`[x] planned — date (evidence)`) is exactly the read-back I need. | ☐ |
| `node list` | **table** (DATE│TYPE│STATUS│NAME-tree│ID) | ✓ | The reference surface: themed, tree, de-numbered, normalized status. `--json` ⚠ **omits `created`/`updated`** (the human table leads with a date the machine path can't see) = **F-21**. | ☐ |
| `rollup` (`--dry-run`) | md (way-finding tree) | ⚠ | Generates `ROLLUP.md`. Two things: it prints **every node's full gate string inline** (`draft=asserted, under-review=asserted, …`) — very verbose; and there's **no flat greppable `id→number→type→name` form** for a consumer with no binary. = UAT **L-7**. | ☐ |
| `validate` | struct (findings + **fix affordances**) | ✓ | Clear and actionable — `[warning] #X …` + a `fix:` line each. The C-6 hardening reads well. | ☐ |
| `check` | struct (validate + reconcile) | ✓ | The C-4 composite; `reconcile_skipped` honesty flag in `--json`. Good. | ☐ |
| `reconcile` | line (`✓ reconcile: no drift (…)`) | ✓ | Clear; states *why* when there's nothing to do (no `desired_facts`). | ☐ |
| `use <k> <ref>` | line (`✓ context: arc = …`) | ✓ | Clear confirmation; the focus it sets is read back by `orient`/`project`. | ☐ |
| `migrate <path>` | (not exercised on data here) | – | Needs a UAT run against a real import (dry-run output + summary). Flag to exercise. | ☐ |

## 2. Mutators (node group) — confirmation output

`node new` (`→ would create arc #1 "…" (ULID)`), `rename`, `retire`, `supersede`, `link`, `unlink`,
`set-gate`, `decomposed`, `tear` (`✓ tore #1 depends_on … (because: …)`).

| | LLM | LLM note | Human ☐ |
|---|---|---|---|
| all node mutators | ✓ (spot-checked `new`, `tear`) | Consistent affordance style — `→ would …` under `--dry-run`, `✓ <verb>ed …` on success; `tear` now surfaces its rationale (C-6). Recommend a quick pass over each on a scratch store to confirm one-by-one, but the pattern is sound. | ☐ |

## 3. Store group + help

| Command | LLM | Note | Human ☐ |
|---|---|---|---|
| `store init` / `store rename` | ✓ | Verified end-to-end in arc-store-home (three-way init; rename mirrors observed state). | ☐ |
| `--help` (all tiers) | ✓ | Three-tier `--help` matches the inventory (11/11/2, mechanical parity). | ☐ |

## 4. Naming (your "does the name make sense" pass — mostly settled by C-4, your call)

Post-C-4 the surface reads coherently: `orient` · `rollup` · `next` · `blocked` · `chain` (was
`path`) · `project` (was `context`) · `use` · `validate` / `check` / `reconcile` · `migrate`, then
`node <verb>` and `store <verb>`. My only naming half-flags for your judgment: **`chain`** (does it
read as "dependency chain" or still ambiguous?), and **`blocked <ref>`** answers "why is X blocked"
but there's no symmetric "why is X *ready*" verb (that's the L-5 gap — a naming *and* capability
question). Everything else I'd sign as sensible, but naming is your column.

## 5. What my flags mean for sequencing (the convergence)

My ⚠ flags are **not scattered** — they land almost entirely on **known findings that are already the
LLM-command-surface arc's slices**:

- **`next` typed + ordered + `--json` name** → LLM arc **slice 02** (L-4).
- **`blocked`/`chain` explain the *ready* half; distinguish no-chain from error** → LLM arc **slice
  02** (L-5).
- **retired nodes leak into `next` + `orient` READY** → **F-22** (the C-3 withdrawn-node filter must
  reach the readiness set) — small, and a natural rider on slice 02.
- **`--json` `created`/`updated`** → **F-21** (LLM arc's `--json`-contract work).
- **`rollup` flat form + verbosity** → LLM arc **slice 03** (L-7).
- **`orient` vision verbosity** → small `orient` polish (its own or slice 02-adjacent).

So the **LLM arc is exactly the vehicle for my (LLM) half of this dual UAT** — its slices *are* my
flags. The **Human half is the output-quality/theme pass** you run per row above (esp. the `struct`/
`line`/`md` commands — is their in-terminal styling up to `list`'s standard, and should `next` look
like a table?). A row closes when both columns are ✓; when every row closes, close-out unlocks.

**Recommended order:** the already-✓-on-my-side commands (`node show`, `list`, `validate`, `check`,
`reconcile`, `project`, `use`, mutators, `store`) need only your human pass. The ⚠ ones (`next`,
`blocked`, `chain`, `rollup`, `orient`, the `--json` dates, F-22) are the LLM-arc build work — and
each gets *both* our sign-offs as it lands. That's the close-out gate.

## 6. Batch-2 additions (Duncan, 2026-08-01) — grounded at 387 nodes, not the §-header's `8de6d0d`/60

> **Re-ground note.** §1–§5 were run against `8de6d0d` (60 nodes). The store is now ~387 nodes, so the
> read-command outputs (esp. `next`, `orient`, `rollup`) are materially bigger — these three items are
> from Duncan running the current binary. A full re-ground of §1 against the live store is itself a
> punch item (below).

| # | Item | Kind | LLM note / disposition | Route |
|---|---|---|---|---|
| B2-1 | **`next` dumps a huge, flat, ungrouped list** — doesn't answer "what could we work on next." If that much output is *correct*, it must be **grouped** and use the **tree/branching glyphs** `node list` uses to show relationships. | command-surface (extends L-4) | Two questions folded in: (a) *correctness* — should `next` really return that many? A dependency-ordered tool's "next" should be the **ready frontier**, not every unblocked node; the retired-#1605 leak (F-22) is one symptom that the readiness set is too loose. (b) *presentation* — once the set is right, render it **typed + grouped + tree**, matching `node list` (the reference surface). This supersedes the terse L-4 "typed + ordered" with the stronger "typed + **grouped-tree** + ordered." | LLM arc **slice 02** (L-4, upgraded) |
| B2-2 | **Numeric IDs are retired from display (surface-wide)** — `next` (and every command) shows ULIDs only | decided (was: are the numbers functional?) | **Answered by direct read of the store (2026-08-01): every edge target in every node is a ULID** — `part_of`/`depends_on`/`blocked_by`/`supersedes` all reference ULIDs; **zero** relations use the numeric handle. The `number` (e.g. `1001`, `7773600`) is a **human-facing coordinate only** — used for `node show <n>`, display, and the coverage structural-fallback match-by-number. So the numbers are *not* wired to anything relational; they're a display affordance. **Operator decision (2026-08-01): the numeric handles are retired from display entirely** — every command that shows an ID shows the **ULID** (the sole functional identity). This applies across the surface (`next`, `orient` READY/BLOCKED, `rollup`, anywhere a `#NN` currently appears), not just `next`. **Input too (operator, standing expectation): `node show` takes a ULID, not a number** — the numeric handle retires from *both* display and input. So the only remaining use of `number` is the **internal coverage match-by-number structural fallback** (migration-time, not user-facing); it stays in the model for now but is invisible to the user, and its eventual removal is a separate model question. **Ergonomic refinement for the LLM arc:** accept a **ULID prefix** (git-short-SHA style) so `node show 01KYSX` resolves — full 26-char ULIDs are copy-paste-only otherwise. Scope: a surface-wide sweep (display shows ULIDs; `<ref>` args accept ULID/prefix), not a per-command tweak. | LLM arc **slice 02** |
| B2-3 | **Semantic redundancy in document titles** — e.g. `(plan-of-record)` on `arc-plan.md`/`slice-doc.md` titles (an arc-plan *is* the plan of record; the parenthetical restates the filename's role). ~67 occurrences across the plan tree. | doc convention (not command-surface) | Real wart. Also includes type-name echoes in titles. **Going forward: drop `(plan-of-record)` and similar role-restating parentheticals from authored titles** (CDC will stop adding them — noted, mea culpa: s14/s15 slice-docs carry it). A bulk cleanup of the existing ~67 is a separate low-risk sweep — worth doing before the SKILL-writing close-out so the corpus reads clean. | doc-hygiene sweep (pre-close-out); CDC to stop adding immediately |

## 7. Punch-list status (2026-08-01)

**This** file is the live tracker; `arc-release-hardening/uat-punch-list.md` is its **raw predecessor**
(Duncan's first-pass batch-1), superseded by the triage in §5 here. **It has *not* been updated during
the arc-migration-fidelity arc (s01–s15)** — correctly, because MF is a *different* arc (migration
faithfulness), and none of its slices touch the command-surface items parked here. Those items
(`next`, `blocked`, `chain`, `rollup` polish, `orient` verbosity, F-21/F-22, and now B2-1…B2-3) remain
routed to the **LLM-command-surface arc**, which has not yet run. So: the MF fixes are tracked in
`arc-migration-fidelity/{arc-plan,*/ledger}.md`; the command-surface punch items are tracked *here* and
await their arc. Nothing is lost — but nothing here is getting *fixed* until the LLM arc starts.
