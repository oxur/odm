# Closing report — Arc 03 / Slice 02: Rollup generation

> CC implementation closing report. Status: **proposed done** → CDC verifies the
> cargo rows via CI / a local 1.85+ toolchain. Impl commit `4a50ac2`; docs commit
> (this report + ledger + the gate-less-tree test) follows.

## What this slice built

The generated whole-plan view (ODD-0013 §6): a `rollup` **model** in `odm-core`
(`crates/odm-core/src/rollup.rs`) and an `odm rollup` command in `odm-cli`
(`crates/odm-cli/src/rollup.rs`). The model is the single source slice03
(`orient`) and slice04 (`--json`) consume (D-3); this slice renders Markdown only
(`--json` is slice04).

`Rollup::assemble(&[Frontmatter], &GateSets, threshold)` is a pure function — no
I/O, no cache (D-2). It **reuses** the existing odm-core ops and packages their
output into small owned view structs; it reimplements no graph or recompose
logic:

- way-finding **tree** ← `Recomposition` (reverse-`part_of` forest);
- per-node **status vector** in `GateSet::sequence()` order, absent gates
  not-reached (D-4);
- **ready** ← `NodeGraph::next`; **blocked** ← `NodeGraph::blocked`, each reason
  carrying the named edge;
- **active tears** ← `NodeGraph::active_tears`, sourced via the new
  `odm_core::graph::frontmatter_tears`, each with its `because`;
- **provenance** ← grouping every node on `Frontmatter::origin`;
- a **drift** slot (empty, renders the A5 placeholder) and a **deferred** slot
  (defined, always empty — no `deferred` status invented).

The CLI command full-scans the corpus, assembles the model, renders Markdown, and
writes `ROLLUP.md` at the repo root via `odm_store::atomic::write`
(write-temp-rename). The render carries no timestamp, so an unchanged corpus
regenerates byte-identical output. `--dry-run` previews to stdout and writes
nothing.

## Per-row ledger walk (11 rows)

- **R-1 — done.** `Rollup::assemble` assembles every section from a loaded corpus
  as a pure function (signature takes `&[Frontmatter]`, `&GateSets`, `Evidence`;
  builds graph/satisfaction/recomposition in-memory, touches no disk).
  `rollup_model_assembles_*` (2 tests) green at `4a50ac2`.
- **R-2 — done.** Tree is the total `part_of` forest; the test asserts the set of
  tree-placed ids equals the whole corpus and that children are id-sorted.
  `rollup_tree_total_single_parent_no_orphans` green.
- **R-3 — done.** Status vectors follow `GateSet::sequence()`; the test reaches a
  deliberately non-alphabetical gate set (`planned`, `complete`) and asserts the
  rendered order is the sequence, with absent gates `None`. A second test confirms
  a type with no gate-set has an empty vector. `rollup_status_gate_order_*` green.
- **R-4 — done.** Ready/blocked computed from `next`/`blocked`; the test asserts
  the blocked entry names its unsatisfied edge (`unsatisfied: #3 Early`).
  `rollup_ready_blocked_named_edges` green.
- **R-5 — done.** Active tears render with rationale; the test tears an A↔B cycle
  and asserts `because: cut the A-B cycle` in the rendered section.
  `rollup_active_tears_rationale_rendered` green.
- **R-6 — done.** Provenance groups planned/discovered/amendment; the test seeds
  one of each (discovered/amendment via the library, since `new` always sets
  planned) and checks each lands in its group. `rollup_origin_view_groups_by_provenance`
  green.
- **R-7 — done.** Drift section present, reads "Not yet tracked (A5)", no
  fabricated data. `rollup_drift_placeholder_no_fake_data` green.
- **R-8 — done.** No deferred section and no `deferred` status variant: the test
  asserts the rendered file contains no `deferred` substring (case-insensitive).
  `rollup_omits_deferred_until_a5` green.
- **R-9 — done.** Full-scan regenerate is idempotent: two runs over an unchanged
  corpus produce byte-identical files. `rollup_command_regenerates_idempotently`
  green.
- **R-10 — done.** Generated header present in both the `--dry-run` preview and
  the written file; `--dry-run` leaves no `ROLLUP.md`. `rollup_header_and_dry_run`
  green.
- **R-11 — done.** `clippy --all-targets --all-features --workspace -- -D warnings`
  → exit 0; no `unsafe` in `crates/odm-core/src` or `crates/odm-cli/src` (grep →
  no matches); line coverage **odm-core 98.69%**, **odm-cli 92.82%** (both ≥ 90%);
  `fmt --check` clean; full workspace `cargo test` → **201 passed, 0 failed**.

## Deviations from the slice doc (flagged, not buried)

1. **`assemble` takes a `threshold: Evidence` parameter** beyond the slice-doc's
   "corpus + `GateSets`" shorthand. Ready/blocked soft-satisfaction is defined
   relative to the satisfaction threshold (`Satisfaction::compute` requires it),
   so the model needs it to compute those sections. The threshold is the same
   config value `next`/`check` already read from `odm.toml` — a faithful
   extension, not new state. Flagged for the CDC's call; trivially adjustable if
   you'd rather the model default it.
2. **`frontmatter_tears` moved into odm-core** (`graph.rs`, now `pub`). The model
   needs it and the CLI's `check` had a private copy; rather than duplicate the
   on-disk-`TornEdge` → engine-`Tear` bridge across crates, I extracted one and
   pointed `check` at it. Pure dedup — `check`'s behavior is unchanged (its suite
   still green).

## Decisions worth recording

- **Blocked is partitioned on the ready frontier.** A soft-satisfied node is
  *ready* (soft deps never withhold it), but `NodeGraph::blocked` still reports
  the soft dep as a reason. Listing such a node in both Ready and Blocked would be
  wrong, so `assemble` excludes ready ids from the blocked set. Caught while
  reasoning about render branches for coverage, before it shipped.
- **`Drift` is an empty `#[non_exhaustive]` marker**, not a `tracked: bool`. A
  bool whose `true` arm is unreachable in A3 would be untested dead code; the
  empty slot is the honest representation of "present but carries no data", and
  `#[non_exhaustive]` lets A5 add fields without a breaking change.
- **Provenance covers every node, the tree does not.** `Recomposition` is the
  resolved forest, so an *orphan* (a `part_of` that doesn't resolve) is by
  definition not in the tree — it is a `check` concern (R-2 says "no orphans" in
  the tree). To avoid silently dropping such a node from the rollup entirely,
  provenance groups **all** nodes, so an orphan still appears there. See the
  uncertainty below.

## Uncertainties / things CDC should look at

- **Orphan visibility.** Per the above, an orphan node appears in Provenance but
  not in the Way-finding tree, and the rollup has no dedicated "orphans" section
  (none was specified for this slice). `odm check` is where orphans are surfaced
  as findings. If the rollup should also call them out explicitly, that is a
  small follow-up — flagging rather than assuming.
- **Two unreachable lines remain uncovered** (named for honesty, not padded
  around): `crates/odm-cli/src/rollup.rs:67` is the `?` error arm of the
  `--dry-run` `writeln!` to `err` (cannot fail for the in-memory buffer tests
  drive; would only fire on a real closed pipe), and
  `crates/odm-core/src/rollup.rs:251` is the defensive `None` fallback of
  `node_ref` in the blocked loop (unreachable by construction — the id comes from
  the same corpus — kept to stay panic-free). Both new files are otherwise
  ≥ 98.6% line.
- **`ROLLUP.md` is written committed at the repo root** (per the slice-doc design
  note — the shared way-finding view, distinct from the gitignored A4 `.odm/`
  cache). Reversible; flag if you'd prefer it gitignored.

## Iterations

One. Closed within the five-iteration cap; the only mid-slice rework was the
ready/blocked partition fix and the `Drift` simplification, both made before the
first close.
