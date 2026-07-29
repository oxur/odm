---
id: 01KWXMBBTKM4PA275KRNSWQTWB
number: 1501
type: slice
schema: slice/v1.1
name: '`desired_facts` schema + `Probe` trait + shell probe'
created: 2026-06-30
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice01-desired-facts-probe/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KWXMBBTKQ3QE7FGTM80MYNDH
status:
  built:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  planned:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  tested:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
---
# Slice 01 (Arc 05): `desired_facts` schema + `Probe` trait + shell probe

> Plan-of-record for the first slice of A5 (Reconciliation). Opens the arc: the data
> model a node uses to *declare* desired state, the trait that *probes* reality, the
> result model that expresses *drift*, and the first probe impl (shell). No runner, no
> `reconcile` command, no rollup wiring yet — those are slices 02–04.

## Goal

A node can **declare** what should be true (`desired_facts` in frontmatter), and code can
**probe** one declared fact against reality and get back a typed **outcome**
(holds / drifted / error). This is the substrate the rest of the arc composes on: the
probe-runner (slice02), `odm reconcile` (slice03), and drift-in-rollup (slice04) all build
on the model and trait this slice lands.

This is the **marquee state-drift killer's** foundation (ODD-0001 C2, the prod-DB 503):
the whole point is that a plan can carry checkable claims about the world, not just prose.

## Scope — in

1. **`desired_facts` frontmatter field** (`odm-core`): an optional list; each entry carries
   a node-local `id`, a human `describe`, and a `probe` spec. Absent/empty is the default
   (a node with no declared facts). Round-trips through the frontmatter serde layer
   (parse → emit → parse is stable). Works identically on **any node type, including the
   project node** (program-level acceptance facts need no separate layer — resolved,
   arc-plan v1.3).
2. **Probe spec model** (`odm-core` or `odm-reconcile` — CC's call, see below): a
   `kind`-tagged spec. This slice defines the **shell** variant; `file` is slice02. A
   malformed spec (unknown `kind`, missing required field) is a **build/parse error
   carrying `Position`** (project error convention).
3. **The `Probe` trait + result model** (`odm-reconcile`, new crate): a trait that takes a
   spec and returns a `ProbeOutcome`:
   - `Holds` — observed reality matches the declared expectation;
   - `Drifted { expected, observed }` — it diverges (the finding that matters);
   - `Error { reason }` — the probe could not be evaluated (command not found, I/O error).
   The three-way split (not a `bool`) is load-bearing: "couldn't check" ≠ "checked, drifted."
4. **The shell probe** — first `Probe` impl: runs an author-declared command and maps
   `exit code` (and an optional stdout match) to the outcome. Holds when the expectation is
   met, Drifted when it diverges, Error when the command cannot run.
5. **The `odm-reconcile` crate** — scaffolded as a workspace member: manifest references
   `[workspace.dependencies]` (no version literals), inherits `[workspace.lints]`, builds
   clean. (Mirrors how A4 slice01 created `odm-index`.)

## Scope — out (named, not dropped)

- **The probe-runner** (execute *all* of a node's / the corpus's facts, collect results) —
  **slice02**. This slice exercises the shell probe directly in unit tests, not via a runner.
- **The `file` probe** (checksum/size/mtime detector repurposed) — **slice02**.
- **`odm reconcile`** (the command, human + `--json`, exit codes) — **slice03**.
- **Drift in rollup/orient** (replace the A3 placeholder) — **slice04**.
- **`affects` / stale-doc check** — **slice05** (the edge already exists; the check is new).
- **Deferred surfacing, scheduled reconcile** — **slice06 / slice07**.
- **Index integration of `desired_facts`** (does `reconcile` read facts via the index →
  `IndexRecord` + adapter + fidelity test + `FORMAT_VERSION` bump?) — **deferred to the
  first reader (slice02/03)**; that slice must honor the carried adapter-fidelity invariant
  (arc-plan). slice01 adds the field to the **frontmatter only**.

## Proposed shape (CC may amend — flag, don't work around)

```yaml
# in a node's frontmatter
desired_facts:
  - id: db-reachable
    describe: "the prod DB answers a trivial query"
    probe:
      kind: shell
      run: "pg_isready -h prod -t 2"
      expect:
        exit: 0          # required for shell; optional stdout match may be added
```

`ProbeOutcome` (the result model) and the `Probe` trait are the contract slices 02–04 use;
keep them minimal and additive-friendly (a new probe `kind` must not break the enum's wire
shape — this is a frontmatter field, so the serde-evolution caution from A4 slice02 applies:
prefer explicit, always-serialized fields).

## Verification approach

Unit tests at the model + probe level (no runner yet):

- Schema round-trip on several node types incl. the project node; absent/empty default.
- Malformed spec → error with `Position`.
- The `Probe` trait + `ProbeOutcome` exist and express all three variants.
- The shell probe against **real local commands**: `true`/`exit 0` → Holds; a command whose
  exit diverges from `expect` → Drifted{expected, observed}; a non-existent command → Error.
- Crate builds as a workspace member; clippy `-D warnings`; no `unsafe`; coverage ≥ 90%.

Cargo rows are CC-`attested` on commit and route to CI / a local 1.85+ run for
reproduction (the sandbox has no 1.85+ toolchain).

## Exit criteria

The ledger's rows F-1…F-7 all reach a final status. In short: a node declares
`desired_facts`; a malformed fact is a positioned error; the `Probe` trait + three-way
`ProbeOutcome` exist; the shell probe returns the right outcome for holds/drift/error; the
`odm-reconcile` crate builds clean; the trust model is documented; gates pass.

## Trust model (resolved — arc-plan v1.3)

The shell probe runs **author-declared commands locally with the user's own privileges** —
the same trust as a git hook or `make` target in your own repo. **No sandbox in the MVP.**
The boundary is documented in the probe's doc comment and here. A `--no-exec`/allowlist is a
deferred guardrail for an untrusted-corpus scenario, out of MVP scope.
