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
