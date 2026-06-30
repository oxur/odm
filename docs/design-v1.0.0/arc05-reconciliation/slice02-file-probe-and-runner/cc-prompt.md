# CC Prompt — Slice 02 (Arc 05): `file` probe + probe-runner

Second slice of A5. slice01 landed the model + `Probe` trait + the **shell** probe. Add the
**`file`** probe and the **probe-runner** that executes a node's (and the corpus's)
`desired_facts` and collects results. No `reconcile` command / `--json` / exit codes /
rollup wiring this slice — slices 03–04.

> **Start condition:** slice01 is CDC-verified (CI flips it to reproduced). Branch off
> `main`: **`arc05-slice02-file-probe-and-runner`** (not `main`). If slice01 hasn't merged,
> branch off `arc05-slice01-…` and rebase later — flag which you did.

## Read first
1. `slice02-file-probe-and-runner/ledger.md` (7 rows) and `slice-doc.md` (same dir) — in
   particular **"The central design decision"**: read `desired_facts` from the **store, not
   the index**. That decision is made (with rationale); implement it, and if you disagree,
   raise it as an amendment rather than quietly indexing.
2. `../arc-plan.md` — A5 capability, Arc Ledger (this slice closes **A-2**), v1.3 (the
   index-integration call) + v1.4 (slice01 close), and the **carried adapter-fidelity
   invariant**. Note: this slice honors it *by non-triggering* (G-5) — you are not touching
   the index, and there is a row that proves it.
3. slice01's code: `crates/odm-reconcile/src/lib.rs` (`ProbeSpec`, `ProbeOutcome`, `Probe`,
   `ShellProbe`) and `crates/odm-core/src/desired.rs`. The `file` variant slots into the
   existing internally-tagged `ProbeSpec` (not `#[non_exhaustive]` — adding it will surface
   compile errors at every match site; that is the intended safety net, handle each).
4. `crates/odm-index` for how the workspace hashes files (`sha2`) — reuse it.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design + error handling.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence at `attested`; per-row walk
  + **Bubble-up to the arc** at close).

## Task
1. **`file` probe** (G-1): `ProbeSpec::File { path, expect: FileExpect { exists, sha256?,
   size? } }` + a `Probe` impl. `Holds` when all declared expectations met; `Drifted` on
   missing-when-`exists`/wrong-hash/wrong-size; `Error` when the path can't be evaluated.
   Reuse `sha2` for the hash. Keep `Error ≠ Drifted` (file gone = Drift; can't read = Error).
2. **Positioned malformed-spec error** (G-2) — consistent with slice01 F-2.
3. **Probe-runner** (G-3, G-4): `run_node` (per-node `(fact_id, outcome)`; factless = empty,
   not error) and `run_corpus` (`(node_id, fact_id, outcome)`), reading **current** store
   frontmatter (read-through fresh).
4. **Store-not-index read** (G-5): the runner loads `desired_facts` from the store; it does
   **not** route them through `index_frontmatters`/`IndexRecord`, and makes **no**
   `FORMAT_VERSION` change. (This is the invariant-honoring decision — see the slice-doc.)
5. **Result/report model** (G-6): per-fact three-way outcome, aggregated per node/corpus,
   drift and error kept distinct (slice03 needs both for severities/exit codes).
6. **Gates** (G-7): clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If a row is wrong/impossible, or you believe the
  store-not-index decision is wrong, raise an amendment with your reasoning — do not quietly
  add `desired_facts` to the index (that would trip the carried invariant and is the exact
  decision the slice-doc declined).
- **Stay in scope.** No `reconcile` command, no `--json`, no exit codes, no rollup/orient
  wiring, no `affects`/deferred/scheduled. Library API only this slice.
- **`Error ≠ Drifted` is non-negotiable** (as in slice01). A missing file is *drift*; an
  unreadable directory is *error*.
- **Reuse, don't reimplement** the file hashing (`sha2`).

## Deliverables
The `file` probe + runner + result model, with `ledger.md` evidence per row (`attested`);
a `closing-report.md` — per-row walk **plus the Bubble-up to the arc** (did slice02 deliver
A-2; what did it reveal — especially anything that sharpens slice03's `reconcile` command or
slice04's rollup-drift; the silent-drop diff). **Remember to propagate the bubble-up into
`arc-plan.md` (the A-2 row + a version-history entry), not only the closing-report** — the
PM Part IV slice-close arc-plan-update step (slice01 left that to CDC; close it yourself this
time). Feature branch `arc05-slice02-file-probe-and-runner`; not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-2) per LEDGER-DISCIPLINE v2.0 §A.
