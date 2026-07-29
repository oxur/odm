---
id: 01KYP5FSK92CK2ZJAMHS2GYRW7
number: 566639300
type: artifact
schema: artifact/v1.1
name: odm Command Inventory — three-tier surface (ODD-0023)
created: 2026-07-25
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-llm-command-surface/odm-command-inventory.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6S1WWVD7SJR5B76S0V
---
# odm Command Inventory — three-tier surface (ODD-0023)

Compiled 2026-07-25, **rewritten 2026-07-26 to the ODD-0023 three-tier
structure** (`node` / `store` groups + top-level workflow verbs). Sources: the
four odm Cowork sessions ("odm as PM tool", "odm v2 kickoff (arc1+2)", "odm v2
(arc3-6)", "ODM logo design"), the design docs (ODD-0013 §7, ODD-0015,
ODD-0017, ODD-0019, ODD-0022, ODD-0023, arc/slice docs,
`arc-release-hardening/`), verified against source: `crates/odm-cli/src/lib.rs`
(current surface) and `legacy/oxur-odm/src/cli.rs` (legacy surface).

This document is the **surface authority** and the companion to **ODD-0023**.
It carries two dimensions at once:

- **Tier** (ODD-0023) — where a command lives: *top-level* (verbs over the
  whole graph), **`odm node <cmd>`** (node entity management), or
  **`odm store <cmd>`** (store-container lifecycle).
- **Status** — how real it is today. Legend:

  | Tag | Meaning |
  |-----|---------|
  | `[shipped]` | in `odm-cli` today (flat surface; the tier is the *target* placement) |
  | `[C-4 rename]` | shipped, spelling changes in RH C-4 (`context`→`project`, `path`→`chain`) |
  | `[C-3]` `[C-4]` `[C-5]` | shipped, behaviour/UI change pending in that RH chunk |
  | `[C-2]` | node **type** rename/addition (affects `list`/`new`/`show`), not a command |
  | `[C-1]` | theming only — no command change |
  | `[store-home]` | **new** command from `arc-store-home` (born under `odm store`) |
  | `[llm-arc]` | **new** node verb from `arc-llm-command-surface` |
  | `[future]` | designed, not yet scheduled |

> **The reorg landed in RH C-4** (2026-07-26). `--help` now shows the three
> tiers, and the Appendix carries the **built** help verbatim — this document
> and the binary agree, which is C-4's acceptance criterion. **There are no
> deprecation aliases:** ODD-0023 §6 was decided as a **hard cut**, so the old
> top-level spellings of node verbs (`odm show`, `odm list`, …) simply do not
> exist. One spelling per command.

---

## 1. Top-level — workflow / graph verbs

Verbs whose object is *the store as a whole*: orient, roll it up, ask what's
next, check it, reconcile it, migrate into it. An LLM or human "operates the
plan" through these. (ODD-0023 §4.) Every query supports `--json` with
versioned schemas (`orient/v1`, `rollup/v1`, `check/v1`, `reconcile/v1`, …).

| Command | Status | Description |
|---------|--------|-------------|
| `orient` (alias `brief`) | `[shipped]` | Vision → current focus → ready/blocked → integrity → drift. **The default command** — bare `odm` runs it (cheap incremental drift pass first, no volatile probes; never bare-errors). |
| `rollup` | `[shipped]` `[C-4]` | Regenerate the plan view. Writes `<out>.<ext>` at the invocation root: `--format={md\|json}` decides the extension, `--out <NAME>` the stem (defaults `md` / `ROLLUP` → `ROLLUP.md`). `--dry-run` renders to stdout; bare `--json` still means "json on stdout". |
| `next` | `[shipped]` | Show the ready frontier (nodes whose dependencies are satisfied). *(LLM arc L-4: ordering/typing improvements planned.)* |
| `blocked <REF>` | `[shipped]` | Explain why a node is blocked or low-confidence. *(LLM arc L-5: currently blocked-half only.)* |
| `chain <REF> [TO]` | `[shipped]` `[C-4]` | Was `path`. Dependency path: the critical chain from X, or a path X → Y. (Renamed to avoid "filepath" confusion; "chain" chosen over dep/deps/trace/route.) |
| `validate` | `[shipped]` `[C-4]` | Validate the whole graph: schema, links, cycles, recomposition, order, staleness (`affects`). **Pure** — no probes, no writes. `--strict` promotes warnings to CI failures. Exit: 0 clean / 1 violations / 2 error. `--json` → **`validate/v1`**. *This is what `check` did before C-4* (ODD-0023 §5). |
| `check` | `[shipped]` `[C-4]` | **The composite:** `validate`, then `reconcile` — "is my plan actually true?". Validation **errors stop the run** (probing a graph with cycles or dangling edges reports on a known-bad state, at probe cost); warnings do not. Exit is the worse of the two phases. `--json` → **`check/v2`**, embedding a `validate` section and a `reconcile` one, with `reconcile: null` + `reconcile_skipped: true` when it stopped early. |
| `reconcile` | `[shipped]` | Reconcile declared `desired_facts` against reality: re-run volatile probes, report drift, re-stamp `last_checked`. `--strict` fails on "couldn't check". |
| `migrate <PATH>` | `[shipped]` | Import a legacy number-/state-directory corpus into the node model (idempotent, keyed on preserved legacy number; never deletes/mutates legacy files). `--dry-run`. **Absorbed `self-host`** (C-5): the tree's shape selects the derivation, `--plan`/`--legacy` force it. The `self-host` spelling was removed in C-4 — one verb. |
| `project [--name=<NAME>]` | `[shipped]` `[C-4]` | Was `context`. "Where am I in the plan" query; defaults to the current project, `--name` for others. Stays top-level. |
| `help` / `--version` | `[shipped]` | `-h/--help`, `-V/--version`. |

| `use <project\|arc> <REF>` | `[shipped]` | Set the current project or arc — the CURRENT FOCUS `orient` reads back. **Kept** (ODD-0023 §5 v1.1, reversing the draft's tentative retire): RH C-5 gave it a real workflow and fixed the use/orient root split so it works. It acts on whole-graph session state, like a cursor. |

**Conventions** (unchanged): refs resolve by id | number | unique name-prefix
everywhere; data → stdout, diagnostics → stderr; errors-as-affordances (every
failure names the command that fixes it).

**Node `--json` (C-8, additive):** node payloads carry `status` — the normalized
state, the same token `node list` shows — and `gates`, the raw ladder in
sequence order with each reached gate's date and evidence. Both are omitted
rather than faked when the node's type has no gate-set, so a consumer can tell
"no ladder" from "at the start of one". Every previously-emitted key is
unchanged.

---

## 2. `odm node <cmd>` — node entity management

CRUD + relationships + gate advancement on a *specific node*. All mutators
support `--dry-run` and `--yes`.

| Command | Status | Description |
|---------|--------|-------------|
| `node new <TYPE> <NAME>` | `[shipped]` `[C-4]` | Create a node (idempotent: re-running describes rather than duplicating). Types: `project\|arc\|slice\|design\|adr\|note\|research` (`[C-2]`: `odd`→`design`, `+research`). `--parent <REF>` sets `part_of`. **C-4 (F-10):** on re-run against an existing node it **warns** (not info) and stays a one-liner — `project exists: #1 "P" — for details run `odm project --name="P"`` — rather than dumping details nobody asked for. |
| `node show <REF>` | `[shipped]` `[C-8]` | Show a node, its edges, and its way-finding (parent + children). **C-8:** now also prints the normalized `status:` and the full **gate ladder** — every rung of the node's own gate-set, reached (`[x]`, with date + evidence) or not (`[ ]`). This is where the raw ladder lives now that `node list` shows a normalized state; before C-8 neither `show` nor `--json` reported gates at all. |
| `node list` | `[shipped]` `[C-3]` `[C-8]` | List nodes, filtered (`--type <T>` `--tag <TAG>` `--component <C>`). **C-3 overhaul:** drop the number column; first column = creation date (`--date=updated` switches); "status" column after "type"; numbers removed from titles; branch-and-leaf ASCII tree (project → arcs → slices) replaces name-prefixing; max-display-width config + flag, overflow elided with " …". `[C-2]` type renames show in the UI. **C-8 (F-19):** STATUS shows a **normalized state** — `planned` / `active` / `done`, plus the `retired`/`superseded` overlays — derived from the node's *position in its own gate ladder*, so a done arc (`verified`) and a done slice (`tested`) read alike and an arc at `complete` reads `active`. A type with no ladder (an `adr`) shows `—`. `--status <VALUE>` accepts **both** the normalized vocabulary and raw gate names. |
| `node rename <REF> <NAME>` | `[shipped]` | Rename a node (name only — id and path unchanged). |
| `node retire <REF> --because <WHY>` | `[shipped]` | Retire a node (withdraw it; file preserved, never deleted). |
| `node supersede <REF> --with <REF> --kind <obsoletes\|updates>` | `[shipped]` | Record that one node supersedes another. |
| `node link <X> <EDGE> <Y>` | `[shipped]` | Add an edge on the source (reverse derived, never written). Edges: `depends_on` (`--satisfied-at <GATE>`), `blocked_by`, `consumes`, `verifies`, `affects`, `part_of` (single-parent, replace semantics). |
| `node unlink <X> <EDGE> <Y>` | `[shipped]` | Remove an edge from the source (absent edge → clear no-op). |
| `node set-gate <REF> <GATE>` | `[shipped]` | Record that a node reached a gate (validated against its type's gate-set). `--by <WHO>`, `--evidence <asserted\|attested\|reproduced\|reconciled>`. |
| `node tear <X> depends_on <Y> --because <WHY>` | `[shipped]` `[future]` | Declare a deliberately-assumed dependency (breaks a cycle; rationale required). **Gap:** rationale is validated but **not persisted** — schema + surfacing in `check` is A6 slice08 / future (see §5). |
| `node decomposed <REF> [--children REF…]` | `[shipped]` | Affirm a parent's children fully account for its scope (§4.5); no `--children` → current children. |
| `node history <REF>` | `[llm-arc]` | New — node change history (git-backed). arc-llm-command-surface. |
| `node diff <REF> [REV]` | `[llm-arc]` | New — diff a node across revisions. arc-llm-command-surface. |
| `node info <REF>` | `[llm-arc]` | New — machine-oriented node facts (`--json` contract, F-21 dates ride here). arc-llm-command-surface. |
| `node search <QUERY>` | `[llm-arc]` | New — content/metadata search over nodes. arc-llm-command-surface. |

> **Resolved (ODD-0023 §7 v1.1):** no top-level shortcuts, permanent or
> transitional — `odm node show` is the only spelling, and `odm show` is an
> unrecognized subcommand. `odm node list` lists nodes; graph-wide views stay
> `orient`/`rollup`. Bare `odm node` prints this group's subcommands on stdout
> and exits `0`, rather than erroring: it is a fair question with a real answer.

---

## 3. `odm store <cmd>` — store-container lifecycle

Manages the store *container* — the orphan `odm` branch in its worktree
(ODD-0022) — as distinct from the nodes inside it. **All new** in
`arc-store-home`; born under `odm store` so the surface never has to be renamed.

| Command | Status | Description |
|---------|--------|-------------|
| `store init` | `[shipped]` | Three arms, chosen from what is already there: **bootstrap** (create the orphan branch + worktree + `config.toml`), **attach** (a remote has it — check out, never re-orphan, never re-scaffold), **ff-sync** (already local; fast-forward only, divergence stops rather than merging). `--worktree`/`--branch` name it; `--dry-run`, `--json`. Realizes ODD-0022 §4.3; arc-store-home slices 02–03. |
| `store sync` | — | **Not a command.** Verified against the binary in C-4: ff-sync is an *arm of* `store init` (above), chosen when the branch is already local. There is no `odm store sync`; the inventory previously listed one. |
| `store rename` | `[shipped]` | Rename the store worktree/branch (the one gix operation that shells out — ODD-0022 §5). arc-store-home slice 04. |
| `store status` | `[future]` | **Still open (ODD-0023 §7):** "where is my store / is it synced" — a `store` verb, or folded into `orient`? Explicitly non-blocking; nothing depends on the answer and no such command exists. |

---

## 4. Cross-cutting changes (no new command)

**Node type renames — C-2** (affects `list` UI + `new` + schema markers):
`odd → design` (UI type rename), `research` (new node type). Tracked as done in
the RH arc.

**Styling — C-1** (no command change): `oxur-term` theming across every table
(warm-orange theme via `colored`; the re-extraction is recorded in
`adr-c1-oxur-table-re-extraction.md`). F-16 (unconditional table ANSI) is an
upstream `oxur-term` follow-up.

---

## 5. Future arcs — designed, not yet scheduled or in draft

```text
  export                  Projections/renderers out to other tools & formats —
                          "federate, don't convert"; the evangelism engine
                          (ODD-0017, post-A6 interop arc). Markdown table
                          projections + ascii-dag renderings. (LLM arc L-6
                          reachability/feasibility folds in here.)
  viz                     On-demand visualization — ascii-dag node-link DAG
                          drawings (arcs 7+ visualization thread; tables stay
                          the committed/diffed representation).
  orient (A7.6 ext.)      Extend orient with descriptive telemetry + the
                          evidence matrix (two-clock telemetry / forecasting
                          thread; odm-telemetry & odm-forecast crates).
  start | work            FLOATED, UNDECIDED — an explicit begin-work command
                          that would auto-set a slice's `branch:` field. Open
                          for A7 (alternatives: auto-set on first set-gate, or
                          on branch creation). A *manual* started marker was
                          rejected as "bookkeeping odm exists to kill."
  node tear (persist)     Tear-rationale persistence: schema + CLI fix so
                          `tear --because` is durable and surfaced in `check`
                          (today validated but NOT persisted). A6 slice08.

DROPPED:
  reconcile --schedule    Scheduled reconcile — demoted by the ODD-0019
                          freshness redirect ("at most survives as an optional
                          off-command refresh").
```

---

## 6. Legacy surface — `legacy/oxur-odm` (pre-rebuild; preserved as harvest source)

Descriptions are the legacy clap `about` strings. Global flag:
`-d, --docs-dir <PATH>` (default `docs`).

```text
odm (legacy)
Design-document manager (number/state-directory model)

  list        (ls)         List all design documents
                           [--state --verbose --removed --dev --component
                            --tags --limit --all]
  show <NUM>               Show a specific document  [--metadata-only]
  new <TITLE>              Create a new design document
                           [--author --component --tags]
  validate    (check)      Validate all documents  [--fix]
  index       (gen-index)  Generate the index file  [--format markdown|json]
  add-headers (headers)    Add or update YAML frontmatter headers
  transition  (mv)         Transition document to a new state
  sync-location (sync)     Move document to directory matching its state header
  update-index (sync-index) Synchronize the index with documents on filesystem
  add <PATH>               Add a new document with full processing
                           [--dev --subdir --force --state --dry-run
                            --interactive -y/--yes --preview]
  add-batch <PATTERNS...>  Add multiple documents (glob patterns)
                           [--dry-run --interactive]
  scan        (rescan)     Scan filesystem and update document state
                           [--fix --verbose]
  search <QUERY> (grep)    Search documents  [--state --metadata
                            -I/--case-sensitive]
  info [SUB]               Show tool information and documentation
                           (subs: states, fields, config, stats, dirs)
  remove <DOC> (rm)        Remove a document (moves to dustbin)
  rename <OLD> <NEW>       Rename a document file (preserves number)
  replace <OLD> <NEW>      Replace a document preserving its ID  [-V/--version]
  debug <SUB>              Debug and introspection commands:
                             state [NUM]   Show state info [--format]
                             checksums     Show checksums and dirty files
                             stats         Show repository statistics
                             diff          Diff between state and filesystem
                             orphans       Show orphaned entries
                             verify <NUM>  Verify a specific document
```

**Carried into v1.0.0** (rebuilt semantics, now placed by tier):
`list`/`show`/`new`/`rename` → **`odm node`**; `validate`→`check` (expanded far
beyond legacy) → **top-level**; legacy `search` prefigures `node search`
(`[llm-arc]`); legacy `debug diff` prefigures `node diff` (`[llm-arc]`).
**Dropped** with the state-directory/dustbin model: `add`, `add-batch`,
`add-headers`, `transition`, `sync-location`, `update-index`, `index`, `scan`,
`info` (legacy sense), `remove`, `replace`, `debug`.

---

## 7. Debatable classifications (decided in ODD-0023 §5 — pointer, not re-litigated here)

All decided in ODD-0023 v1.1 (**Accepted**, 2026-07-26) and shipped in C-4:

- **`use`** → **kept**, top-level (reversal: C-5 gave it the CURRENT FOCUS workflow).
- **`context` → `project`** → renamed; stays **top-level**; `--name` inspects another project.
- **`check` + `reconcile`** → **reversal**: `validate` is the *pure* command (not an alias) and
  `check` is the *composite* `validate`-then-`reconcile`, stopping at validate errors. Schema ids
  bumped to `validate/v1` / `check/v2` (§6a) because `check` changed meaning under the same name.
- **`self-host` → `migrate`** → folded in C-5; the spelling **removed** in C-4.
- **`next` / `blocked` / `chain`** → graph queries; **top-level**.
- **`list` scope** → `odm node list`; graph-wide views stay `orient`/`rollup`.
- **Aliases** → **reversal**: hard cut, none at all.

---

## Appendix — the built `--help`, verbatim (three tiers, post-C-4)

Captured from `target/release/odm` on 2026-07-26, after the C-4 reorg. This is
the parity evidence: the tiers above are what the binary actually prints, not
what the reorg intended to produce.

### `odm -h`

```text
The Odd Document Manager

Usage: odm [COMMAND]

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
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help (see more with '--help')
  -V, --version  Print version
```

### `odm node -h`

```text
Manage nodes: create, inspect, relate, and advance them

Usage: odm node [COMMAND]

Commands:
  new         Create a node (idempotent: re-running describes rather than duplicating)
  list        List nodes as a plan: date, type, status, containment tree
  show        Show a node, its edges, and its way-finding (parent + children)
  rename      Rename a node (name only — id and path are unchanged)
  retire      Retire a node (withdraw it; the file is preserved, never deleted)
  supersede   Record that one node supersedes another
  link        Add an edge on the source node (reverse is derived, never written)
  unlink      Remove an edge from the source node (absent edge → a clear no-op)
  set-gate    Record that a node has reached a gate (validated against its gate-set)
  decomposed  Affirm that a parent's children fully account for its scope (§4.5)
  tear        Declare a deliberately-assumed dependency edge (breaks a cycle)
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help (see more with '--help')
```

### `odm store -h`

```text
Manage the store itself: where it lives and how it is created

Usage: odm store [COMMAND]

Commands:
  init    Create or refresh the store's home: a worktree holding an orphan branch
  rename  Rename the store's worktree directory and/or its branch
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help (see more with '--help')
```

---

## Notes & corrections surfaced while compiling

* `brief` is already **implemented** as a visible alias of `orient` (the
  transcripts had it as proposed; the code settles it).
* `tear --because` gap is real in current code: rationale validated via
  `Tear::new` but the schema's `tears` vector has no rationale field →
  `node tear` persistence is future (§5).
* The three-tier split is **naming/placement only** — no change to the node
  model, gates, edges, or storage (ODD-0023 §8). The store's data model is
  untouched by the reorg; `store` commands were the reason the group exists,
  not a consequence of it.
* **ODD numbering:** this reorg is **ODD-0023**; G-1's ID-scheme ODD should
  take the next free number (confirm, to avoid an id collision).
* Not commands (frequent false positives): `odm-graph`/`odm-core`/etc. are
  crates; `depends_on` etc. are edge kinds; `odd|adr|note` are node types;
  `odm dep` was a rejected rename candidate; `odm assert` is the `assert_cmd`
  test harness.
