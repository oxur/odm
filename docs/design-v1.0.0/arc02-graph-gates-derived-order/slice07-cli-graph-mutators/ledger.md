# Slice 07 (Arc 02): CLI graph-mutators

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M-1 | `link X <edge> Y` adds the edge on source X, persisted; reverse derived (not written) | `cargo test -p odm-cli link_adds_edge` (in-process) → ok | serious | 0013 §7/§3 | done | `c557790`: `link_adds_edge` → 1 passed. `depends_on` + B's id written on source A; B's file carries no reverse edge. | |
| M-2 | `link` covers `depends_on` (+`--satisfied-at`), `blocked_by`, `consumes`, `verifies`, `affects`, `part_of` | `cargo test -p odm-cli link_edge_kinds` → ok | serious | 0013 §3 | done | `c557790`: `link_edge_kinds` → 1 passed. All six edge fields on the source; `--satisfied-at tested` → `satisfied_at` qualified dep. | |
| M-3 | `link X part_of Y` enforces single-parent (replaces existing parent, not appends) | `cargo test -p odm-cli link_part_of_single_parent` → ok | serious | 0013 §3 / Q-4 | done | `c557790`: `link_part_of_single_parent` → 1 passed. Re-link replaces (P2, not P1); old parent id gone from the file. | |
| M-4 | `unlink X <edge> Y` removes the edge; unlinking an absent edge is a clear no-op | `cargo test -p odm-cli unlink_removes_edge` → ok | correctness | 0013 §7 | done | `c557790`: `unlink_removes_edge` → 1 passed. Edge removed; a second unlink reports `no-op` (not an error). | |
| M-5 | Endpoints resolve by id \| number \| unique name-prefix; unresolvable → typed error naming the fix | `cargo test -p odm-cli mutator_ref_resolution` → ok | serious | 0013 §7 | done | `c557790`: `mutator_ref_resolution` → 1 passed. name-prefix/number/full-id all resolve; bad ref → "no node with number 99 … odm list". | Reuses slice05 `resolve`. |
| M-6 | `set-gate X <gate>` records via `Status::set_gate`; out-of-set gate → `UnknownGate` w/ affordance; default evidence `asserted`; `evidence_dates` first-reach recorded | `cargo test -p odm-cli set_gate_cli` → ok | serious | 0013 §5.1 / slice03/05.1 | done | `c557790`: `set_gate_cli` → 1 passed. Default `asserted`; `evidence_dates` written; bad gate → "unknown gate … allowed … odm set-gate"; explicit `--evidence`/`--by` recorded. | |
| M-7 | `tear X depends_on Y --because <r>` creates the tear; empty rationale → `MissingRationale` w/ affordance | `cargo test -p odm-cli tear_cli` → ok | serious | 0013 §4.3 | done | `c557790`: `tear_cli` → 1 passed. Torn edge recorded in `tears:`; whitespace `--because` → "needs a rationale … --because". | Rationale validated via `Tear::new`, not persisted (schema gap) — flagged. |
| M-8 | `new --parent <ref>` sets `part_of` to the resolved parent | `cargo test -p odm-cli new_with_parent` → ok | correctness | 0013 §7 | done | `c557790`: `new_with_parent` → 1 passed. `new arc --parent 1` → `part_of` = #1's id. Unresolvable parent fails before writing (`mutator_edge_cases`). | |
| M-8b | `decomposed X --children <ref…>` affirms decomposition (wraps `affirm_decomposed`; records children + date); persists | `cargo test -p odm-cli decomposed_cli` → ok | serious | slice06 CDC / 0013 §4.5 | done | `c557790`: `decomposed_cli` → 1 passed. Explicit `--children` and current-children (derived reverse `part_of`) forms both persist `decomposed:`; non-parent-capable node → "only a project or arc". | CDC-added row; clears check v2 drift/advance-without findings. |
| M-9 | `--dry-run` writes nothing; `--yes` runs non-interactively (all mutators) | `cargo test -p odm-cli mutators_dry_run_and_yes` → ok | correctness | 0013 §7 | done | `c557790`: `mutators_dry_run_and_yes` (+ `mutator_edge_cases`) → passed. link/set-gate/tear/new/decomposed `--dry-run` announce + write nothing; `--yes` runs. | |
| M-10 | Mutations persist atomically (odm-store) and round-trip on reload | `cargo test -p odm-cli mutation_roundtrip` → ok | serious | 0013 §6 | done | `c557790`: `mutation_roundtrip` → 1 passed. part_of + qualified depends_on + gate survive a fresh dispatch reload and re-parse. | Atomic write-temp-rename (odm-store). |
| M-11 | A graph built purely via the CLI (link/set-gate/tear) answers `next`/`blocked` correctly | `cargo test -p odm-cli cli_built_graph_queries` (end-to-end, self-host smoke) → ok | serious | self-hosting | done | `c557790`: `cli_built_graph_queries` → 1 passed. CLI-built A→B: pre-satisfy `next`=[B] / `blocked 1`=unsatisfied; after `set-gate 2 tested`, `next`=[A] / `blocked 1`=nothing-holding. | The headline self-host test. |
| M-12 | Every mutator failure names the exact fix (errors-as-affordances) | `cargo test -p odm-cli mutator_errors_name_fix` → ok | serious | 0013 §7 / 0001 | done | `c557790`: `mutator_errors_name_fix` → 1 passed. Bad endpoint → `odm list`; out-of-set gate → `odm set-gate`; empty rationale → `--because`. | |
| M-13 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) | `cargo clippy -p odm-cli --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-cli --summary-only --ignore-filename-regex '(odm-core\|odm-store\|odm-graph)/'` → **line** ≥ 90% (target 95%) | serious | CLAUDE.md | done | `c557790`: clippy → exit 0 (after bundling `set_gate` args into `GateReach`); no `unsafe`; fmt clean; coverage TOTAL **line 90.99%** (commands.rs 91.66%). | `run()` uncovered in-process — see report. |

## What Worked

- **Thin CLI over odm-core ops.** Every mutator is a resolve → mutate → persist
  shell around an existing model op (`edges_mut`, `Status::set_gate`,
  `Tear::new`, `affirm_decomposed`); no model logic was reimplemented in the CLI.
- **The slice05 resolver carried the whole endpoint surface.** id | number |
  name-prefix resolution and its affordance came for free by reusing `resolve`,
  so every mutator names `odm list` on a bad ref with no extra code.
- **Single-parent and idempotency are just `Option`/`contains`.** `part_of` is
  `edges.part_of = Some(id)` (replace); list edges dedup via `contains` before
  push; `depends_on` retains-then-pushes so a re-link can update `satisfied-at`.
- **`decomposed` with no `--children` closes the check-v2 loop.** It affirms
  against the current reverse-`part_of` children — exactly the action check v2's
  drift / advanced-without fix affordance names, now runnable end-to-end.
- **In-process `dispatch` made the self-host smoke test (M-11) trivial.** Build
  the graph with `link`/`set-gate`, then query `next`/`blocked` in the same
  harness — no spawned binary, real round-trips through odm-store.

## Closure

Closed at `c557790` on `2026-06-24`. CDC: pending (cargo rows reproduced by CDC
in CI or a local 1.85+ toolchain). All `done` states are *proposed done* pending
that independent verification. Total rows: 14. Done: 14. Deferred: 0. No-op: 0.
**Arc 02 done on close.**

**Flagged for CDC.** (1) **Tear rationale not persisted:** `--because` is
*validated* via `Tear::new` (empty → rejected) but the on-disk `tears`
(`Vec<Dependency>`) has no rationale field, so the text is dropped — the
tear-rationale schema gap deferred since slice04. (2) **M-8b `decomposed` added
by CDC:** implemented as `decomposed X [--children <ref…>]` wrapping
`affirm_decomposed`; with no `--children` it affirms against the node's current
containment children (the form that clears a check-v2 finding), and it rejects
non-parent-capable nodes. (3) **`--yes` is a no-op runner:** no mutator prompts
interactively, so `--yes` simply runs (same as omitting it) — consistent with
the existing CRUD mutators. (4) **In-process exit/round-trip tests, not
`assert_cmd`:** odm-cli is library-only; tests drive `dispatch` and read the
persisted files directly (the slice05 pattern). (5) **`run()` uncovered:** the
real-stdout/cwd wrapper can't be exercised in-process; TOTAL line 90.99% clears
the bar but the 95% target would need binary-level tests.
