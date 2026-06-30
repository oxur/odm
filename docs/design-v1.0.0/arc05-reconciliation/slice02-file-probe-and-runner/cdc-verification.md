# CDC Verification — Arc 05 / Slice 02: `file` probe + probe-runner

> Independent verification of CC's closed ledger (impl + close on
> `arc05-slice02-file-probe-and-runner`, commits `e4ca702` + `0e74431`), per
> LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo
> rows route to CI / a local 1.85+ run.

## Environment constraint (disclosed)

CDC's sandbox has no 1.85+ toolchain; CC built + ran on local rustc 1.95.0 (rows
**attested**, reproduced via CI). Branch was cut off `arc05-slice01-…` (slice01 unmerged) —
**rebase onto `main` once slice01 merges** (CC flagged; tracked).

## Row dispositions

**Row count:** 7 opened, 7 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **G-1** — `ProbeSpec::File { path, expect }` + `FileProbe` (`file.rs`); content hash via
  `sha2::{Digest, Sha256}` (`file.rs:7`, `hex_sha256`) — no reimpl. 6 tests
  (`tests/file_and_runner.rs:64–173`): holds (exists+size+sha256), drift on missing /
  hash-mismatch / wrong-size, error on unreadable / invalid-path. `Error ≠ Drifted` held:
  missing-file = Drift, unhashable = Error. ✔
- **G-2** — `file_probe_spec_malformed_errors_with_position` (`tests/frontmatter.rs:731`):
  missing `path` (serde), non-hex `sha256` (`deserialize_with`), unknown sub-field
  (`deny_unknown_fields`) each → positioned `FrontmatterError::Yaml`. Spec parses in
  `odm-core`, so the test lives there — correct. ✔
- **G-3** — `Runner::run_node → NodeReport` (`runner.rs:134`); factless node → empty report
  (`runner_factless_node_is_empty`), not an error. ✔
- **G-4** — `Runner::run_corpus → CorpusReport` via `Store::load_all` each call
  (`runner.rs:5`); `runner_corpus_read_through` seeds, runs, adds a fact, re-runs, observes
  it — read-through freshness with no manual rebuild. ✔
- **G-5 (the invariant-honoring row)** — `grep -rnE
  "index_frontmatters|IndexRecord|FORMAT_VERSION" crates/odm-reconcile/src` → **no matches**;
  the runner reads via `odm_store::Store` + `load_all` (`runner.rs`). The index is **not**
  extended; the carried A4 adapter-fidelity invariant is honored **by non-triggering**. ✔
  *(See ruling 1 — CC's guard design.)*
- **G-6** — `OutcomeCounts { holds, drifted, errored }` (`runner.rs:86`) keeps the three
  kinds distinct; `report_distinguishes_drift_from_error` proves a 1-holds/1-drift/1-error
  corpus exposes all three; per-fact `ProbeOutcome` preserved via `CorpusReport::iter()`.
  Ready for slice03 severities/exit codes. ✔
- **G-7 (no `unsafe`)** — grep empty in `odm-reconcile/src` (also workspace-denied). ✔

**Attested by CC (local 1.95.0), pending CI:** clippy `-D warnings` → exit 0; line coverage
**file.rs 97% / runner.rs 97% / shell.rs 100%**; 40 suites green workspace-wide.
→ **PENDING CI.**

## Rulings on CC's flagged decisions

1. **G-5 guard kept the index identifiers out of the crate *entirely, including prose*, so
   the negated grep stays a real tripwire. Accepted — and a genuinely thoughtful touch.** A
   guard grep that the code can defeat by mentioning the identifier in a comment is no guard;
   keeping the rationale in the *planning* docs (slice-doc) and the code comment
   identifier-free makes the tripwire meaningful. Small, acceptable cost: the crate's own doc
   comments describe the store-read decision without naming the index machinery. The store
   read is real (`load_all`, not a re-exported index path), so grep + actual-usage together
   are a solid guarantee, not just a lexical check.
2. **Root-insensitive Error tests (directory-hash + embedded-NUL path) instead of `chmod
   000`. Accepted — correct engineering.** CI often runs as root, where `chmod 000` doesn't
   deny, so a permission-based test would be flaky/vacuous. A directory-as-file-content and a
   NUL-in-path are deterministic "couldn't check" triggers on any platform/user — they keep
   `Error ≠ Drifted` honestly tested without flakiness.
3. **`lib.rs` modularized into `probe`/`shell`/`file`/`runner` (public API unchanged).**
   Accepted — sensible as the crate grew past one probe; re-exports preserve the surface.
4. **Store-not-index implemented as recommended, not overridden.** The decision I made in the
   slice-doc held under implementation; CC concurred. ✔

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-2: the `file` probe + the runner + the report model, exactly
  as scoped. Drift is now *computable* end-to-end; slice03 renders it.
- **Silent-drop diff honest?** ✔ — 7/7; the guard-grep prose constraint and the test-trigger
  choices are disclosed in the closing-report.
- **Findings + arc-plan?** ✔ — **CC propagated the bubble-up itself this time** (A-2 row +
  v1.5), closing the PM Part IV gap CDC had to cover for slice01. Two forward-carries
  recorded: (a) slice03 must add `Serialize` to `ProbeOutcome`/report under a `reconcile/v1`
  marker (deliberately omitted here — no wire contract yet); (b) **a new slice04 open
  question** — the rollup is a hot index view but reconcile reads the store, so slice04 must
  decide how drift reaches the rollup.

## CDC note on the slice04 open question

CC's recommended resolution — **`rollup` runs an on-demand corpus reconcile and folds the
result in, rather than caching drift in the index** — is the right one, and it is the direct
consequence of slice02's store-not-index decision: drift truth lives in `reconcile` (the
store-read action), not in the index. Caching drift in the index would re-open precisely the
index-vs-store tension slice02 closed, and would make a *derived, time-varying* fact
(drift) into index state (which is supposed to mirror file state, not probe results). I
concur; slice04's slice-doc will settle it formally (and it keeps the A4 invariant
un-triggered for the same reason).

## Verdict

**Arc 05 / Slice 02 CDC-verified on structure; all flags ruled; cargo rows pending CI.**
The `file` probe and the probe-runner land cleanly, the `Error ≠ Drifted` distinction is
preserved into the aggregate (`OutcomeCounts`), and the store-not-index decision held — with
a guard row (G-5) that will catch any future drift toward the index. slice03 (`odm
reconcile`) is well-teed-up: severity/exit-code mapping is a pure function of `OutcomeCounts`,
and the `--json` shape is obvious once `ProbeOutcome` gains `Serialize` under `reconcile/v1`.
A-2 is attested-on-close; flips to `done` on CI green.

CDC: planning thread, 2026-06-30. Iterations used: 1.
