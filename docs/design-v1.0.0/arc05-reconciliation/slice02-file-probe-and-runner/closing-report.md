# Closing report — Slice 02 (Arc 05): `file` probe + probe-runner

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc05-slice02-file-probe-and-runner`, branched off
> `arc05-slice01-desired-facts-probe` (slice01 had not merged to `main`; this is
> the prompt's named fallback). Rebase onto `main` once slice01 merges.

## Per-row walk

**G-1 — the `file` probe — done (attested).** `ProbeSpec::File { path, expect:
FileExpect { exists, sha256?, size? } }` + a `Probe` impl in
`crates/odm-reconcile/src/file.rs`. `Holds` when every declared expectation is
met; `Drifted` on missing-when-`exists:true` (and present-when-`exists:false`),
wrong size, or wrong hash; `Error` when the path is unevaluable. Content hashing
uses `sha2::{Digest, Sha256}` — the workspace crate, same algorithm as the index,
no reimplementation. `cargo test -p odm-reconcile file_probe` → 6 passed; `grep
sha2|Sha256` confirms reuse.

**G-2 — positioned malformed-spec error — done (attested).** The `file` spec
parses in `odm-core` (frontmatter), so the test lives there. Missing `path`
(serde missing-field), non-hex `sha256` (a `deserialize_with` semantic check),
and an unknown `expect` sub-field (`deny_unknown_fields` on `FileExpect`) each
yield a positioned `FrontmatterError::Yaml` — consistent with slice01 F-2. `cargo
test -p odm-core file_probe_spec_malformed_errors_with_position` → ok.

**G-3 — per-node runner — done (attested).** `Runner::run_node(&Document) ->
NodeReport` collects a `FactResult { fact_id, outcome }` per declared fact, in
order; a factless node yields an empty report (`is_empty()`), never an error.
`runner_collects_per_node` + `runner_factless_node_is_empty` → ok.

**G-4 — corpus runner with read-through freshness — done (attested).**
`Runner::run_corpus() -> Result<CorpusReport, StoreError>` runs every node's
facts, collecting `(node_id, fact_id, outcome)`. It calls `Store::load_all`
afresh each invocation, so a newly written fact is seen with no manual rebuild —
demonstrated by `runner_corpus_read_through` (seed, run, add a node, run again,
observe it).

**G-5 — store-not-index read (the invariant-honoring row) — done (attested).**
The runner reads `desired_facts` from `odm_store::Store`; the index is **not**
extended. `grep -rnE "index_frontmatters|IndexRecord|FORMAT_VERSION"
crates/odm-reconcile/src` → no matches (the guarded identifiers are kept out of
the crate's source *including prose*, so the grep remains a genuine tripwire —
see *Deviations*). The carried A4 adapter-fidelity invariant is honored **by
non-triggering**: no `IndexRecord`/adapter/fidelity-test/`FORMAT_VERSION` change.
Implemented as the slice-doc recommended; not overridden to index-extend.

**G-6 — result model keeps drift vs error distinct — done (attested).**
`ProbeOutcome` is preserved per fact; `OutcomeCounts { holds, drifted, errored }`
aggregates per node (`NodeReport::counts`) and per corpus (`CorpusReport::counts`)
without flattening, and `CorpusReport::iter` exposes every `(node_id,
fact_result)`. `report_distinguishes_drift_from_error` → `{1, 1, 1}` for a
holding/drifted/errored corpus.

**G-7 — gates — done (attested).** clippy `-D warnings` → exit 0; no `unsafe`;
coverage (line) `file.rs` 97.01%, `runner.rs` 96.92%, `shell.rs` 100% — all ≥ 90.
Full workspace `cargo test` green (40 suites, no regression).

Rows: 7. Done: 7. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item of `slice-doc.md` shipped: the `file` probe (exists / hash /
size, with `sha2` reuse), the per-node and per-corpus runner with read-through
freshness, the store-not-index read path (G-5), and the drift-vs-error-preserving
result model. The `ProbeSpec` evolution exercised slice01's not-`#[non_exhaustive]`
choice (a compile error at each match site until `File` was handled). Every "out"
item stayed out: no `reconcile` command, no `--json`, no exit codes, no
rollup/orient wiring, no `affects`/deferred/scheduled, and — as a recorded design
call, not a deferral — **no** index extension for `desired_facts`.

Disclosed (non-silent) consequential edits, all in the diff:
1. **slice01's `odm-reconcile` `lib.rs` was modularized** into `probe.rs` /
   `shell.rs` / `file.rs` / `runner.rs` with `lib.rs` as the re-exporting root.
   The public API (`Probe`, `ProbeOutcome`, `ShellProbe`, …) is unchanged, so
   slice01's tests compile untouched.
2. **The odm-core round-trip test's `match` gained a `ProbeSpec::File` arm** — the
   intended safety-net compile error, handled.

## Deviations / decisions flagged

- **The G-5 guard grep shaped the prose.** The criterion's Verify is `! grep …
  "index_frontmatters|IndexRecord|FORMAT_VERSION" …`. Explanatory doc comments
  naturally want to *name* those identifiers; doing so would make the negated grep
  fail and quietly defang the tripwire. Resolution: the crate docs describe the
  decision **without** the literal identifiers, and say so. The grep therefore
  still means "these index identifiers appear nowhere in the crate" — which is the
  real signal that the runner does not route through the index. Not an amendment;
  a deliberate phrasing to keep the ledger's guard honest.
- **No amendment was needed.** The store-not-index decision was implemented as
  recommended; the YAML shape fit serde cleanly; no row was wrong or impossible.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice02 deliver A-2's piece?** Yes. A-2 ("slice02 — file probe + probe
execution — closed") is the arc-ledger row this slice discharges. Both probe kinds
(shell from slice01, file here) now exist, and the runner executes facts
per-node and per-corpus into a drift/error-distinct result model. A-2 stays
`attested` until CDC reproduces.

**2. What it reveals for slice03 (`odm reconcile`).** The result model is
deliberately shaped for slice03's needs and de-risks it:
  - **Severity/exit-code mapping is a pure function of `OutcomeCounts`.** slice03
    can map `errored > 0` → one exit code/severity and `drifted > 0` → another, with
    no model surgery. The drift-vs-error split is already preserved end-to-end.
  - **`--json` has an obvious shape:** `CorpusReport` → `{ nodes: [{ node_id,
    results: [{ fact_id, outcome }] }] }`, with `outcome` a tagged
    `holds|drifted{expected,observed}|error{reason}`. slice03 should add a `schema`
    marker (`reconcile/v1`) per the §7.1 convention and lock it with a shape test.
  - **`ProbeOutcome` is not yet `Serialize`.** slice03 will need serde on
    `ProbeOutcome`/`FactResult`/the report for `--json`. Left off deliberately
    (slice02 is library-only, no wire contract yet) — flagged so slice03 adds it
    with the schema marker rather than ad hoc.
  - **Human output ordering** is store-order (creation-id) for nodes,
    declaration-order for facts — stable and deterministic, ready to render.

**3. What it reveals for slice04 (drift in rollup/orient).** The rollup's A3
`drift: { tracked: false }` placeholder can now be fed real data: a corpus run
produces `OutcomeCounts` and per-node drift. **Open question surfaced for slice04:**
the rollup is a *hot view* built off the **index**, but reconcile reads the
**store** (G-5). slice04 must decide how drift reaches the rollup without
re-introducing the index-vs-store tension — likely the rollup invokes a corpus
reconcile run on demand (store read) and folds the result in, rather than caching
drift in the index. Worth settling explicitly in the slice04 doc; it is the
natural next pressure point on the v1.3 index-integration decision.

**4. Reusable finding.** "Unreadable" is hard to test portably (root ignores
`chmod`); sha256-of-a-directory and NUL-in-path are deterministic, root-insensitive
Error triggers. Useful for any future probe with an I/O-error branch.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-2 row evidence + a v1.5 version-history entry), not only here.
