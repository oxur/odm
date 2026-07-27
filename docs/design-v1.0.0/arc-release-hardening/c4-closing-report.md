# C-4 closing report — command-surface cleanup + the ODD-0023 reorg

> **Arc:** Release Hardening (UAT) · **Chunk:** C-4 · **Covers:** `F-10`/`F-11`/`F-12`/`F-13`
> folded with **ODD-0023** (three-tier reorg) · **Assignment:** `cc-prompt-c4-command-surface.md`
> **Companion authority:** `arc-llm-command-surface/odm-command-inventory.md`
> **Implemented by:** CC · **Date:** 2026-07-26 · **Branch:** `rh-c4-command-surface`
> (off the C-5 tip on `release/1.0.x`) · **Evidence class:** attested-by-CC (local 1.85+);
> cargo rows reproduce on CI.

## What shipped

```
$ odm -h
Commands:
  orient     Orient: vision → current focus → ready/blocked → integrity → drift [aliases: brief]
  rollup     Regenerate the plan rollup: the single cheap view of the whole plan
  next       Show the ready frontier (nodes whose dependencies are satisfied)
  blocked    Explain why a node is blocked or low-confidence
  chain      Show the critical chain from X, or the dependency path X → Y
  project    Show where you are in the plan: the current project and arc
  use        Set the current project or arc context
  validate   Validate the graph: schema, links, cycles, recomposition, order
  check      Check the plan against reality: `validate`, then `reconcile`
  reconcile  Reconcile declared `desired_facts` against reality: report drift
  migrate    Import a document corpus or a plan set into the node model
  node       Manage nodes: create, inspect, relate, and advance them
  store      Manage the store itself: where it lives and how it is created
```

Node entity management moved under `odm node <cmd>` (11 verbs); `odm store` was already born in
arc-store-home. The classification test is now answerable for a new command: **whole graph**
(top-level), **a node** (`node`), or **the container** (`store`).

| Finding | What landed |
|---------|-------------|
| **F-10** | `new` on an existing node **warns** (was info) and stays one line: `project exists: #1 "P" — for details run \`odm project --name="P"\``. It points at the command that answers the question instead of dumping details nobody asked for. |
| **F-11** | `context` → **`project`**, top-level, `--name` to inspect another project. |
| **F-12** | `path` → **`chain`** — "path" reads as a filesystem path; the question is "what has to happen, in order". |
| **F-13** | `rollup --format={md\|json}` + `--out <NAME>`; writes `<stem>.<ext>`, so `ROLLUP.md` is a default rather than a law, and the help says so. |
| **ODD-0023** | The three tiers, hard cut, `validate`/`check` split, group help. |

## Phase 0 — the model, and three reversals

ODD-0023 went **Draft → Accepted** (v1.1) *before* any code, per the model-first rule. Three of its
tentative recommendations were reversed, each recorded in the ODD with its reasoning rather than
quietly rewritten:

1. **`use` kept**, not retired. The draft called it redundant with `project`. **C-5 changed the
   ground**: `use arc X` now sets the CURRENT FOCUS `orient` reads back, and C-5's dogfood fixed the
   use/orient root split precisely so it works.
2. **`validate` is the pure command; `check` is the composite** — not "`validate` is a trivial alias
   and reconcile is opt-in behind `check --reconcile`". *(Operator's call, and the better one:
   the cheap operation gets its own honest name, so nobody has to remember a flag to avoid paying
   probe cost, and `check` gets to mean "go and look".)*
3. **Hard cut, no deprecation aliases** — reversing §6's one-release window.

## The `validate` / `check` split

| Verb | Does | Side effects | `--json` |
|------|------|--------------|----------|
| `validate` | graph validation only — schema, links, cycles, recomposition, order, staleness | none | `validate/v1` |
| `reconcile` | the probe pass only — unchanged | yes | `reconcile/v1` |
| `check` | `validate`, then `reconcile` | yes, via reconcile | `check/v2` |

**`check` stops at validate errors.** Probing a graph with dangling edges or cycles reports on a
state already known to be broken, at probe cost and with side effects. Warnings do not stop it.
Exit is the worse of the two phases.

**The skipped half is `null`, with an explicit `reconcile_skipped: true`** — absent must be
distinguishable from clean, or a consumer reads "no drift" where the truth is "we never looked".

**Schema ids were bumped, not reused** (ODD-0023 §6a). `check` changed meaning under the same name;
a consumer pinned to `check/v1` now gets a version it does not recognize, which is the correct
outcome. Silently keeping `check/v1` on either the pure or the composite command would have let a
machine consumer carry on believing it was reading the old thing.

## What the hard cut actually cost

**~290 test invocations**, rewritten mechanically to the new spellings (94 `new`, 47 `check`,
43 `list`, 31 `link`, 24 `set-gate`, …). That is the whole bill, and it is the argument for the
decision: the breakage a deprecation window insures against is cheap here — pre-1.0, one operator,
one consumer, and an inventory rewritten in the same pass — while the window itself would have put
two spellings in every help text, example and doc for a release, plus the notice machinery and its
tests.

Four call sites used a different form (`.arg("list")`, `.arg("context")`) and survived the first
sweep; they failed loudly at the next run, which is the right way for that to go wrong.

## Three things found by checking rather than assuming

The prompt's "inventory ↔ `--help` must agree" turned out to be load-bearing:

1. **The inventory listed a `store sync` command. There is none.** ff-sync is an *arm of*
   `store init`, chosen when the branch is already local (slice 03). The inventory had promoted an
   implementation arm to a verb that was never built.
2. **`store init`'s own help still said "Bootstrap only for now"** — stale since slice 03 added
   attach and ff-sync. Its own doc comment had been describing a two-slices-ago version of itself.
3. **The `self-host` CLI wrapper was dead code** once the verb was removed — caught by clippy, not
   by reading. The *library* derivation it called is still very much alive under `migrate`.

Parity is now **checked mechanically**, not by eye: every `[shipped]` row in the inventory's three
tier tables against the binary's own `--help`.

```
tier   binary  doc   verdict
top        11   11   MATCH
node       11   11   MATCH
store       2    2   MATCH
```

The Appendix now carries the **built** help verbatim (captured from `target/release/odm`), replacing
the pre-reorg flat help it was holding for traceability.

## Reflexive check (RH-6/RH-7) — on the self-hosted store

Run against odm's real corpus in its relocated home, not a scratch repo, per the prompt:

```
$ odm validate            ✓ validate: ok (60 node(s), no problems)
$ odm check               ✓ validate: ok (60 node(s), no problems)
                          ✓ reconcile: no drift (no desired_facts declared)
$ odm use arc 1600        ✓ context: arc = Migrate, self-host & PM-skill (01KWXMBBTK…)
$ odm orient   CURRENT FOCUS
                 arc #1600 Migrate, self-host & PM-skill — planned=asserted, …
$ odm node list --type arc   6 shown — 54 filtered or withdrawn
```

**The `use`/`orient` regression is the one that matters** — it is verified where the C-5 root split
actually hid, which a scratch repo cannot do.

## Deviations from the brief

| Brief | What happened | Why |
|-------|---------------|-----|
| Deprecation aliases for one release (§6) | **None** | Operator decision; ODD-0023 §6 reversed with the reasoning recorded. |
| `validate` as a trivial alias of `check`; `check --reconcile` opt-in | **Inverted**: `validate` pure, `check` composite | Operator's resolution, adopted into ODD-0023 §5 v1.1. |
| — | `store sync` row corrected; `store init` help corrected; `self-host` wrapper removed | Not in the brief; all three are inventory/help parity, which the brief did require. |
| — | "is a arc" → "is an arc" | An article bug I introduced in `project`, fixed at all three sites with a derived article rather than a hardcoded one. |

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **58 binaries ok, 0 failed** |
| `cargo clippy --all-targets --workspace --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none |
| Inventory ↔ `--help` | 11/11/2, MATCH in all three tiers |
| Corpus | `validate` green at 60 nodes — a surface change moved no nodes |

New tests: the composite's two paths (`check_json_embeds_both_phases_on_a_sound_graph`,
`check_json_skips_reconcile_when_validate_errors`), the schema-id contract for all five payloads,
and the F-10 pointer assertion.

## Silent-drop diff

None. Scoped out and absent: C-6 (G-2/G-3/L-3b check-hardening), C-8 (F-19), the LLM arc's node
verbs, F-21. `store status` remains open in ODD-0023 §7 and is explicitly non-blocking.

## Ledger

- **RH-4** (C-4 closed) — **attested** on this report; **reproduced** when CI runs the cargo rows.
- **F-10 / F-11 / F-12 / F-13** — dispositioned, done.
- **ODD-0023 Accepted** (v1.1) before the code, with the three reversals recorded.
- **Carried, unassigned:** `store status` (§7 open); `node tear`'s rationale is still validated but
  not persisted (pre-existing, `[future]`); the LLM arc's node verbs now have a home to land in.
