# Slice 02 -- CDC verification (self-sourced planning nodes)

**Method:** LEDGER-DISCIPLINE v2.0 sec. A. Structural rows **reproduced by direct code read** at
commit `803c738` (ancestor of HEAD, worktree clean); test rows reproduced by **confirming the named
test functions exist and reading their assertions**; cargo/exec + full `make check` are
`attested -> CI` (no macOS toolchain in the CDC sandbox). **Verdict: PASS.** Verified 2026-08-03.

## Verdict

**PASS.** Slice 02 delivers the arc's core enabler: an authored node (`origin: authored`,
`source: { class: authored }`, no external `paths`) is a first-class, checked, migrate-safe citizen
of the store. 9/11 rows reproduced or attested as `done`; F-8/F-9 (amendment fold-in) correctly held
at `attested`; F-6's real-corpus leg honestly deferred. Two things are *better* than the ledger
asked for (below).

## Reproduced by code read

- **F-1 (schema).** `Origin::Authored` added with `to_str`/`from_str` (origin.rs). `Source`
  reshaped (frontmatter.rs): migration-only fields (`normalization`/`migrated_by`/`migrated_on`)
  are now `Option`, `paths` is `skip_serializing_if empty`, and a new `migrated_from: Vec<PathBuf>`
  holds converted-node provenance. `Source::authored(migrated_from)` + `is_authored()` are the
  canonical constructor/predicate. **Backward-compatible on read** (serde `default` + Option), so
  existing migrated-node YAML still parses and re-serializes unchanged -- no silent corpus churn.
- **F-2/F-3 (check).** `check_authored_source` lives in `content_validity` -- the store-loaded pass
  (confirmed wired into the CLI at `odm-cli/src/commands.rs:1760`), not the index-backed `check()`,
  because `source` is not in the `.odm/` index. CC's report that `undeveloped-stub`/`missing-source`
  were never source-gated findings is **correct** -- `recompose.rs`'s `UndevelopedStub` is a
  parent/child-count signal; there was nothing to suppress. F-3: the existing check families were
  untouched, so schema/edge/cycle/decomposition still fire on authored nodes.
- **F-4/F-5 (migrate/reconcile).** `self_host`'s new fourth tier `by_coordinate_authored`
  (selfhost.rs) recognizes an authored node by `(type, number)` -- since its `source.paths` is
  empty it was invisible to `by_source`/`by_coordinate` -- and skips it (`SkipReason::Authored`) with
  no source rewrite, no body churn, no re-fidelity. `reconcile` gained an explicit `origin() ==
  Authored` skip. This is the substantive change and **the duplicate-mint gap it closes was real**:
  without it the first `migrate --all` would have re-minted every authored node while its `./docs`
  file was still present. Caught by reading the matching logic before coding -- exactly the
  read-before-write discipline.
- **F-6/F-10 (conversion).** `convert_to_authored` (selfhost.rs) is idempotent (already-authored ->
  skip), dry-run-able, work-typed-only, excludes synthesis/retired/sourceless nodes, and preserves
  the former `source.paths` in `migrated_from` -- **sub-decision (i)**, the recommended lean and the
  direct application of ratified fork A. Wired as `odm migrate --to-authored`, deliberately separate
  from `--all`.

## Reproduced by test-assertion read

- **F-7 (no regression) -- the one I read in full.** `a_drifted_migrated_node_still_hard_fails_
  alongside_an_untouched_authored_one` (source_identity.rs) persists an authored arc + a genuinely-
  migrated arc with a **drifted body**, runs `repair`, and asserts `MigrateError::BodyHashMismatch`
  -- then asserts the authored sibling is untouched. The gate still fires on migrated content; the
  authored-recognition code does not mask it. This is the load-bearing no-regression proof, and it
  holds.
- **F-1/F-2/F-3/F-5 fixtures.** All 12 named test functions confirmed present (4 consistency-both-
  directions in check.rs; 3 authored-still-fails in check.rs; the reconcile no-touch test; 6 in
  convert_to_authored.rs; the self_host no-churn test; the e2e `migrate_to_authored_..._check_stays_
  green` in odm-cli). Not run here (no toolchain) -> `make check` green attested by CC -> CI.

## Two things better than the ledger asked

1. **`InconsistentAuthoredSource` is a bidirectional consistency check, not a suppression.** The
   ledger framed F-2 as "don't error on authored"; CC instead made "authored is a provenance
   *value*, never an absence" a **checked invariant** in both directions (origin:authored <-> source
   class authored, no stray migrated fields). That is the stronger, correct reading of ODD-0026
   sec. 2.1.
2. **The `--to-authored` / `--all` separation.** CC surfaced and disclosed a real workflow hazard
   the slice-doc did not name: converting a node freezes further `./docs` edits to it (it is
   self-sourced), so bundling conversion into `--all` would silently freeze the operator's ordinary
   planning workflow. Keeping it an explicit, previewable flag is the right call.

## Carried forward (not blocking)

- **F-8/F-9 amendment fold-in** into the accepted ODD-0013 / ODD-0025 remains an operator/CDC step;
  the code already implements what the stubs specify. Recommend folding them in (and, per the point
  below, capturing the `--to-authored` separation there).
- **The `--to-authored` vs `--all` separation is model-adjacent** and currently lives only in code
  comments + this slice's docs. Recommend a one-paragraph note in ODD-0026 (or the F-9 amendment) so
  the decision is discoverable from the model, not only from `convert_to_authored`'s doc comment.
- **F-6/SS-3 real-corpus leg (the acceptance anchor):** operator runs `odm migrate --to-authored
  --dry-run` on the real store to preview, then a real run, then `migrate --all` + `check` -- expect
  exit 0 with `./docs` still present. Same deferral pattern as every code slice this session.
- **Schema versioning (minor):** the `Source` shape changed but stayed backward-compatible; no
  schema marker bump was made. Worth a deliberate confirm during the ODD-0013 fold-in that no
  `schema:` version bump is required (ODD-0020) given the additive/optional change.
