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

> **The reorg itself has not landed.** Today's shipped `--help` is still flat;
> the verbatim current help is preserved in the **Appendix** for traceability
> until the C-4 reorg pass rewrites `--help`/`--json` to these tiers. Old
> top-level spellings survive one release as hidden deprecated aliases
> (ODD-0023 §6).

---

## 1. Top-level — workflow / graph verbs

Verbs whose object is *the store as a whole*: orient, roll it up, ask what's
next, check it, reconcile it, migrate into it. An LLM or human "operates the
plan" through these. (ODD-0023 §4.) Every query supports `--json` with
versioned schemas (`orient/v1`, `rollup/v1`, `check/v1`, `reconcile/v1`, …).

| Command | Status | Description |
|---------|--------|-------------|
| `orient` (alias `brief`) | `[shipped]` | Vision → current focus → ready/blocked → integrity → drift. **The default command** — bare `odm` runs it (cheap incremental drift pass first, no volatile probes; never bare-errors). |
| `rollup` | `[shipped]` `[C-4]` | Regenerate the plan view. C-4: help must not hardcode `ROLLUP.md`; support md **and** json output + an output-name option (defaults `md` / `ROLLUP`). `--dry-run` renders to stdout. |
| `next` | `[shipped]` | Show the ready frontier (nodes whose dependencies are satisfied). *(LLM arc L-4: ordering/typing improvements planned.)* |
| `blocked <REF>` | `[shipped]` | Explain why a node is blocked or low-confidence. *(LLM arc L-5: currently blocked-half only.)* |
| `chain <REF> [TO]` | `[C-4 rename]` | Was `path`. Dependency path: the critical chain from X, or a path X → Y. (Renamed to avoid "filepath" confusion; "chain" chosen over dep/deps/trace/route.) |
| `check` (alias `validate`) | `[shipped]` | Validate the whole graph: schema, links, cycles, recomposition, order, staleness (`affects`). `--strict` promotes warnings to CI failures. Exit: 0 clean / 1 violations / 2 error. **Pending (ODD-0023 §5):** `validate` alias is trivial; *check-always-reconciles* is a behaviour change (reconcile runs side-effecting probes) — recommend **`check --reconcile`** (opt-in), not every `check` paying probe cost. |
| `reconcile` | `[shipped]` | Reconcile declared `desired_facts` against reality: re-run volatile probes, report drift, re-stamp `last_checked`. `--strict` fails on "couldn't check". |
| `migrate <PATH>` | `[shipped]` `[C-5]` | Import a legacy number-/state-directory corpus into the node model (idempotent, keyed on preserved legacy number; never deletes/mutates legacy files). `--dry-run`. **C-5: absorbs `self-host`** as a special case — the two become one verb. |
| `project [--name=<NAME>]` | `[C-4 rename]` | Was `context`. "Where am I in the plan" query; defaults to the current project, `--name` for others. Stays top-level. |
| `help` / `--version` | `[shipped]` | `-h/--help`, `-V/--version`. |

**Pending removal — `use <project\|arc> <REF>`** `[shipped]`: sets the active
project/arc context. With `project` and the store model this may be
**redundant**; ODD-0023 §5 tentatively **retires** it unless a workflow needs
it. Decide in ODD-0023.

**Conventions** (unchanged): refs resolve by id | number | unique name-prefix
everywhere; data → stdout, diagnostics → stderr; errors-as-affordances (every
failure names the command that fixes it).

---

## 2. `odm node <cmd>` — node entity management

CRUD + relationships + gate advancement on a *specific node*. All mutators
support `--dry-run` and `--yes`.

| Command | Status | Description |
|---------|--------|-------------|
| `node new <TYPE> <NAME>` | `[shipped]` `[C-4]` `[C-2]` | Create a node (idempotent: re-running describes rather than duplicating). Types: `project\|arc\|slice\|design\|adr\|note\|research` (`[C-2]`: `odd`→`design`, `+research`). `--parent <REF>` sets `part_of`. **C-4:** on re-run against an existing node, **warn** (not info) — "exists; for details run `odm project --name=…`". |
| `node show <REF>` | `[shipped]` | Show a node, its edges, and its way-finding (parent + children). |
| `node list` | `[shipped]` `[C-3]` `[C-2]` | List nodes, filtered (`--type <T>` `--tag <TAG>` `--component <C>`). **C-3 overhaul:** drop the number column; first column = creation date (`--date=updated` switches); "status" column after "type"; numbers removed from titles; branch-and-leaf ASCII tree (project → arcs → slices) replaces name-prefixing; max-display-width config + flag, overflow elided with " …". `[C-2]` type renames show in the UI. |
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

> **Open (ODD-0023 §7):** do very common node verbs keep a permanent top-level
> shortcut (`odm show` ≡ `odm node show`), or only through the deprecation
> window? Recommendation: `odm node list` for nodes; graph-wide views stay
> `orient`/`rollup`.

---

## 3. `odm store <cmd>` — store-container lifecycle

Manages the store *container* — the orphan `odm` branch in its worktree
(ODD-0022) — as distinct from the nodes inside it. **All new** in
`arc-store-home`; born under `odm store` so the surface never has to be renamed.

| Command | Status | Description |
|---------|--------|-------------|
| `store init` | `[store-home]` | Three-way: **bootstrap** (create the orphan `odm` branch + `.worktrees/odm` worktree + `config.toml`), **attach** (existing remote branch), **sync** (ff-only). Realizes ODD-0022 §4.3; arc-store-home slice 02 (bootstrap) + slice 03 (attach/ff-sync). |
| `store sync` | `[store-home]` | Fast-forward / rebase the local store to upstream (team sync is plain git push/pull; this is the ergonomic wrapper). arc-store-home slice 03. |
| `store rename` | `[store-home]` | Rename the store worktree/branch (the one gix operation that shells out — ODD-0022 §5). arc-store-home slice 04. |
| `store status` | `[store-home]` `[future]` | **Open (ODD-0023 §7):** "where is my store / is it synced" — a `store` verb, or folded into `orient`? Undecided. |

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

- **`use`** → retire candidate (redundant with `project` + store model). *(§1
  Pending removal.)*
- **`context` → `project`** → RH **C-4** rename; stays **top-level**.
- **`check` + `reconcile`** → keep `validate` as an alias; make reconcile
  **opt-in** (`check --reconcile`) rather than every `check` paying probe cost.
- **`self-host` → `migrate`** → RH **C-5**; does not survive as its own verb.
- **`next` / `blocked` / `chain`** → graph queries; **top-level**.
- **`list` scope** → `odm node list`; graph-wide views stay `orient`/`rollup`.

---

## Appendix — current shipped `--help` (flat; verbatim, pre-reorg)

Kept for verification traceability until the C-4 reorg pass rewrites
`--help`/`--json` to the tiers above. This is the actual clap output today.

```text
odm 1.0.0
The Odd Document Manager

Usage: odm [COMMAND]

The subcommand is optional: bare `odm` runs `orient` (it never bare-errors),
performing the cheap incremental drift pass first (no volatile probes).

Orient / read  (every query supports --json with versioned schemas:
                orient/v1, rollup/v1, check/v1, reconcile/v1, ...):
  orient                  Orient: vision → current focus → ready/blocked →
                          integrity → drift. The default command.
                          [visible alias: brief]
  rollup                  Regenerate `ROLLUP.md`: the single cheap view of the
                          whole plan  [--dry-run renders to stdout]
  list                    List nodes, optionally filtered
                          [--type <T>] [--tag <TAG>] [--component <C>]
  show <REF>              Show a node, its edges, and its way-finding
                          (parent + children)
  next                    Show the ready frontier (nodes whose dependencies
                          are satisfied)
  blocked <REF>           Explain why a node is blocked or low-confidence
  path <REF> [TO]         Show a dependency path: the critical chain from X,
                          or a path X → Y

Context:
  use <project|arc> <REF> Set the current project or arc context
  context                 Show the current project/arc context

Create / edit  (every mutator supports --dry-run and --yes):
  new <TYPE> <NAME>       Create a node (idempotent: re-running describes
                          rather than duplicating). Types:
                          project|arc|slice|odd|adr|note
                          [--parent <REF> sets part_of]
  rename <REF> <NAME>     Rename a node (name only — id and path unchanged)
  retire <REF>            Retire a node (withdraw it; the file is preserved,
      --because <WHY>     never deleted)
  supersede <REF>         Record that one node supersedes another
      --with <REF>
      --kind <obsoletes|updates>

Graph mutators  (--dry-run, --yes):
  link <X> <EDGE> <Y>     Add an edge on the source node (reverse is derived,
                          never written). Edges: depends_on [--satisfied-at
                          <GATE>], blocked_by, consumes, verifies, affects,
                          part_of (single-parent, replace semantics)
  unlink <X> <EDGE> <Y>   Remove an edge from the source node (absent edge →
                          a clear no-op)
  set-gate <REF> <GATE>   Record that a node has reached a gate (validated
      [--by <WHO>]        against its type's gate-set)
      [--evidence <asserted|attested|reproduced|reconciled>]
  tear <X> depends_on <Y> Declare a deliberately-assumed dependency edge
      --because <WHY>     (breaks a cycle; rationale required)
  decomposed <REF>        Affirm that a parent's children fully account for
      [--children REF...] its scope (§4.5); no --children → current children

Integrity & drift:
  check                   Validate the whole graph: schema, links, cycles,
                          recomposition, order, staleness (`affects`)
                          [--strict promotes warnings to CI failures]
                          exit codes: 0 clean / 1 violations / 2 error
  reconcile               Reconcile declared desired_facts against reality:
                          re-run volatile probes, report drift, re-stamp
                          last_checked  [--strict fails on "couldn't check"]

Migration:
  migrate <LEGACY_PATH>   Import a legacy number-/state-directory ODD corpus
                          into the node model (idempotent, keyed on preserved
                          legacy number; never deletes/mutates legacy files)
                          [--dry-run]
  self-host <PLAN_PATH>   Import odm's own plan set (project + arcs + slices
                          under a design-vX.Y.Z/ tree) as work nodes
                          (idempotent; --dry-run)

Options:
  -h, --help              Print help
  -V, --version           Print version

Conventions:
  * Node refs resolve by id | number | unique name-prefix, everywhere.
  * Data → stdout, diagnostics → stderr.
  * Errors-as-affordances: every failure names the command that fixes it.
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
