# Slice 02 (Arc 05): `file` probe + probe-runner

> Plan-of-record for A5 slice02. slice01 landed the model + trait + the **shell** probe.
> This slice adds the second probe (**file**) and the **probe-runner** that executes a
> node's (and the corpus's) `desired_facts` and collects results — the substrate
> `odm reconcile` (slice03) renders. No command / `--json` / exit codes / rollup wiring
> yet (slices 03–04).

## Goal

Two things: (1) a **`file` probe** — check a declared file against an expectation
(exists / content-hash / size); (2) a **probe-runner** that takes a node (or the whole
corpus), runs each declared fact's probe, and collects `(node, fact, outcome)` results.
After this slice, drift is *computable* end-to-end (declared facts → run → outcomes); slice03
turns it into the `odm reconcile` command.

## Scope — in

1. **The `file` probe** — a second `ProbeSpec` variant + `Probe` impl (`odm-reconcile`):
   - Spec: `kind: file`, a `path` (relative to the repo root — the odm working dir), and an
     `expect` carrying **exists** (default `true`) + optional **sha256** (hex) + optional
     **size** (bytes). Absent optional fields are not checked.
   - Outcome: `Holds` when every declared expectation is met; `Drifted { expected, observed }`
     when the file diverges (missing when `exists:true`, wrong hash, wrong size); `Error`
     when the path can't be evaluated (I/O error, permission). The slice01 `Error ≠ Drifted`
     distinction carries: "the file is gone" is **Drift** (a real declared-vs-observed
     finding); "I couldn't read the directory at all" is **Error**.
   - Hashing reuses the workspace `sha2` (the same algorithm the index uses) — not a
     reimplementation.
2. **The probe-runner** (`odm-reconcile`):
   - `run_node` — execute one node's `desired_facts`, collect `(fact_id, outcome)`; a node
     with no facts yields an **empty** result (a no-op, never an error).
   - `run_corpus` — execute every node's facts, collect `(node_id, fact_id, outcome)`.
   - A **result/report model** that, per fact, preserves the three-way outcome and, in
     aggregate (per node / corpus), lets slice03 assign severities + exit codes (drift and
     error must remain distinguishable in the aggregate — no flattening to a count).
3. **`ProbeSpec` evolution** — adding the `file` variant exercises slice01's deliberate
   choice (internally-tagged, *not* `#[non_exhaustive]`): every match site that handled only
   `Shell` is now a compile error until it handles `File`. That is the intended safety net.

## The central design decision — how the runner reads `desired_facts`

The arc-plan (v1.3) flagged this as the slice02/03 call, and it is the load-bearing one:
**does the runner read `desired_facts` via the A4 index seam (`index_frontmatters`) or
directly from the store?**

**Decision (CDC): read from the store, directly. The index is NOT extended with
`desired_facts` this slice.** Rationale:

- **Reconcile is an on-demand, I/O-bound *action*, not a hot *view*.** Its dominant cost is
  running the probes (subprocess spawns, file stats, eventually network). A frontmatter read
  from the store is noise beside that. A4's "stop re-walking the tree" win targets the
  *frequent, fast* read/filter/graph paths (`list`/`orient`/`check`/`rollup`) — not an
  infrequent action whose cost is the probes themselves.
- **`desired_facts` is a heavy nested structure** (a list of facts, each with a probe spec).
  Putting it in every `IndexRecord` bloats the snapshot for data only one infrequent command
  reads — against the stat-cache-accelerator spirit. It is the same call A4 already made for
  `list --json` (the full-node dump stays `load_all`; the index is the filter/sort
  accelerator, not a general document store — ODD-0014 §3.5).
- **The carried adapter-fidelity invariant is honored by *non-triggering*.** Because slice02
  does **not** read `desired_facts` off the index, it does **not** need to extend the index
  record / adapter / fidelity test. This is not a silent skip — it is the invariant working:
  the field is read where it lives (the store), so the index equivalence guarantee is
  untouched. A dedicated ledger row (G-5) proves the read path is the store, so a *future*
  accidental switch to `index_frontmatters` (which would silently return facts-less
  frontmatters) is caught.

**Ratification point (Duncan / CC may override):** the alternative is to add `desired_facts`
to `IndexRecord` (+ adapter + fidelity test + `FORMAT_VERSION 3→4`) for A4-consistency. I
recommend against it for the reasons above, but it is a real architecture fork — flag it if
you see a hot path (e.g. a future `orient` that must *count* facts per node cheaply) that
would justify the index cost. (Even then, a cheap `has_desired_facts: bool` / count in the
record beats embedding the full nested structure.)

> The runner still benefits from a fresh corpus: it should read **current** frontmatter
> (a newly written fact is seen without a manual rebuild). Reading from the store gives that
> for free — there is no index staleness to reconcile because the index isn't in this path.

## Scope — out (named, not dropped)

- **`odm reconcile`** the command (human + `--json`, exit codes, severities) — **slice03**.
  slice02 delivers the runner + result model as *library* API; slice03 is the CLI surface.
- **Drift in rollup/orient** (replace the A3 placeholder) — **slice04**.
- **`affects` / stale-doc check** — **slice05**; **deferred / scheduled** — **slice06/07**.
- **Extending the index with `desired_facts`** — **declined** (see the decision above); not
  deferred-to-a-later-slice but a recorded design call, revisited only if a hot path needs it.

## Verification approach

Unit + small integration tests (a temp store with seeded nodes + real files):

- `file` probe: holds (matching file), drift (missing / wrong hash / wrong size), error
  (unreadable path); malformed `file` spec → positioned error (slice01 F-2 consistency).
- runner: per-node collection; empty for a facts-less node; corpus-wide collection;
  read-through freshness (write a fact, run, see it).
- the store-not-index read path (grep + the read-through test).
- result model keeps drift vs error distinct in the aggregate.
- clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

Cargo rows are CC-`attested` and route to CI / a local 1.85+ run for reproduction.

## Exit criteria

Ledger rows G-1…G-7 reach a final status: the `file` probe works (holds/drift/error) and
reuses `sha2`; a malformed file spec is positioned; the runner collects per-node and
per-corpus with read-through freshness; the runner reads facts from the **store, not the
index** (invariant honored by non-triggering); the result model keeps drift vs error
distinct; gates pass.
