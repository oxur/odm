# odm Command Inventory — existing + proposed

Compiled 2026-07-25 from: the four odm Cowork sessions ("odm as PM tool",
"odm v2 kickoff (arc1+2)", "odm v2 (arc3-6)", "ODM logo design"), the design
docs (ODD-0013 §7, ODD-0015, ODD-0017, ODD-0019, arc/slice docs,
`arc-release-hardening/`), and verified against source: `crates/odm-cli/src/lib.rs`
(current surface) and `legacy/oxur-odm/src/cli.rs` (legacy surface).

---

## 1. Current surface — implemented, verified in `odm-cli` (v1.0.0, through Arc 6)

Descriptions below are the actual clap `about` strings.

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

## 2. Proposed — decided in the Release Hardening (UAT) arc, not yet landed

```text
RENAMES (C-4):
  chain <REF> [TO]        Rename of `path` — avoids "filepath" confusion.
                          Name decided ("chain", over dep/deps, trace, route).
  project [--name=<NAME>] Rename of `context` — too general a name. Defaults
                          to the current project; --name for others.

BEHAVIOR CHANGES:
  new                     On re-run against an existing node: warn (not info) —
                          "project exists; for details run
                          `odm project --name=<name>`"                    (C-4)
  rollup                  Help must not hardcode `ROLLUP.md`; support both md
                          and json output + an output-name option
                          (defaults: "md" / "ROLLUP")                     (C-4)
  migrate                 Absorbs `self-host` as a special case — the two
                          commands become one                             (C-5)
  list                    Overhaul (C-3):
                            * drop the number column
                            * first column = creation date;
                              --date=updated switches to updated date
                            * "status" column after "type"
                            * numbers removed from titles
                            * branch-and-leaf ASCII tree indentation
                              (project → arcs → slices) replaces name-prefixing
                            * max-display-width config option + flag,
                              overflow elided with " ..."

TYPE RENAMES (C-2, affects list UI + schema markers):
  odd → design            UI type rename
  research                New node type

STYLING (C-1):
  (no command change)     oxur-cli / oxur-table theming across every table;
                          route A-vs-B still an OPEN joint investigation
```

---

## 3. Proposed — designed for future arcs (not yet scheduled or in draft)

```text
  export                  Projections/renderers out to other tools & formats —
                          "federate, don't convert"; the evangelism engine
                          (ODD-0017, post-A6 interop arc). Target for Markdown
                          table projections and ascii-dag renderings.
  viz                     On-demand visualization — ascii-dag node-link DAG
                          drawings (arcs 7+ visualization thread; tables stay
                          the committed/diffed representation).
  orient (A7.6 ext.)      Extend orient with descriptive telemetry + the
                          evidence matrix (two-clock telemetry / forecasting
                          thread; odm-telemetry & odm-forecast crates).
  start | work            FLOATED, UNDECIDED — an explicit begin-work command
                          that would auto-set a slice's `branch:` field. Open
                          question for A7 (alternatives: auto-set on first
                          set-gate, or on branch creation). A *manual* started
                          marker was rejected as "bookkeeping odm exists to
                          kill."
  (arc02 slice08)         Tear-rationale persistence: schema + CLI fix so
                          `tear --because` is durable and surfaced in `check`
                          (today the rationale is validated but NOT persisted).

DROPPED:
  reconcile --schedule    Scheduled reconcile — demoted by the ODD-0019
                          freshness redirect ("at most survives as an optional
                          off-command refresh").
```

---

## 4. Legacy surface — `legacy/oxur-odm` (pre-rebuild; preserved as harvest source)

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

Carried into v1.0.0 (rebuilt semantics): `list`, `show`, `new`, `rename`,
`check` (expanded far beyond legacy `validate`). Dropped with the
state-directory/dustbin model: `add`, `add-batch`, `add-headers`, `transition`,
`sync-location`, `update-index`, `index`, `scan`, `search`, `info`, `remove`,
`replace`, `debug`.

---

## Notes & corrections surfaced while compiling

* `brief` is already **implemented** as a visible alias of `orient` (the
  transcripts had it as proposed; the code settles it).
* `tear --because` gap is real in current code: rationale validated via
  `Tear::new` but the schema's `tears` vector has no rationale field.
* Not commands (frequent false positives): `odm-graph`/`odm-core`/etc. are
  crates; `depends_on` etc. are edge kinds; `odd|adr|note` are node types;
  `odm dep` was a rejected rename candidate; `odm assert` is the `assert_cmd`
  test harness.
