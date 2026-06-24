# Closing Report — Slice 07 (Arc 02): CLI graph-mutators

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain). **On close, Arc 02 is complete and odm is self-host-usable.**

- **Implementation commit:** `c557790`.
- **Branch:** `arc02-slice07-cli-graph-mutators` (based on
  `arc02-slice06-check-v2`; not pushed; not merged to `main`).
- **Scope delivered:** the CLI graph-mutators — `link`, `unlink`, `set-gate`,
  `tear`, the CDC-added `decomposed`, and `new --parent` — each wiring an
  existing odm-core op and persisting atomically via odm-store. A plan can now be
  authored and advanced entirely through `odm`, no hand-editing.
- **Result:** 14 rows, all `done`. 0 deferred, 0 no-op. CDC added M-8b
  (`decomposed`) during review; satisfied.
- **Aggregate gates:** `cargo test -p odm-cli` → 55 pass (15 new mutator tests +
  CRUD/derived-order/check unregressed); clippy `-D warnings` → exit 0; no
  `unsafe`; coverage TOTAL line 90.99% (commands.rs 91.66%).

## Per-row walk

| ID | Status | Evidence (re-runnable at `c557790`) |
|----|--------|-------------------------------------|
| M-1 | done | `cargo test -p odm-cli link_adds_edge` → 1 passed; edge on source, no reverse written. |
| M-2 | done | `cargo test -p odm-cli link_edge_kinds` → 1 passed; all six edge kinds + `--satisfied-at`. |
| M-3 | done | `cargo test -p odm-cli link_part_of_single_parent` → 1 passed; re-link replaces the parent. |
| M-4 | done | `cargo test -p odm-cli unlink_removes_edge` → 1 passed; removal + absent-edge no-op. |
| M-5 | done | `cargo test -p odm-cli mutator_ref_resolution` → 1 passed; id/number/name-prefix; bad ref names `odm list`. |
| M-6 | done | `cargo test -p odm-cli set_gate_cli` → 1 passed; default `asserted`, `evidence_dates`, UnknownGate affordance, explicit evidence/by. |
| M-7 | done | `cargo test -p odm-cli tear_cli` → 1 passed; tear recorded; empty rationale rejected with affordance. |
| M-8 | done | `cargo test -p odm-cli new_with_parent` → 1 passed; `--parent` sets `part_of`. |
| M-8b | done | `cargo test -p odm-cli decomposed_cli` → 1 passed; explicit + current-children affirmation; non-parent-capable rejected. |
| M-9 | done | `cargo test -p odm-cli mutators_dry_run_and_yes` (+ `mutator_edge_cases`) → passed; dry-run writes nothing, `--yes` runs. |
| M-10 | done | `cargo test -p odm-cli mutation_roundtrip` → 1 passed; mutations survive a fresh reload + re-parse. |
| M-11 | done | `cargo test -p odm-cli cli_built_graph_queries` → 1 passed; a purely CLI-built graph answers `next`/`blocked` before and after satisfying a dep. |
| M-12 | done | `cargo test -p odm-cli mutator_errors_name_fix` → 1 passed; each failure names a command. |
| M-13 | done | clippy `-D warnings` → exit 0; no `unsafe`; `cargo llvm-cov -p odm-cli …` → TOTAL line 90.99%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **Tear rationale is validated but not persisted.** `tear … --because <r>`
   runs the rationale through `Tear::new` (so an empty/whitespace rationale is
   rejected with an affordance), but the on-disk `tears` field is a
   `Vec<Dependency>` with no rationale slot, so the text is dropped after
   validation. This is the tear-rationale schema gap deferred since slice04 (and
   worked around in slice06's `check`). When the schema gains a rationale,
   `tear` should persist it.

2. **M-8b `decomposed` was added by CDC and implemented here.** `decomposed X
   [--children <ref…>]` wraps `Frontmatter::affirm_decomposed`. With explicit
   `--children`, it affirms against those resolved nodes; with none, it affirms
   against X's current containment children (derived reverse `part_of`) — the
   form that directly clears a `check` v2 decomposition-drift /
   advanced-without-decomposition finding. It rejects non-parent-capable nodes
   (`slice`/documents) with an affordance. This closes the loop check v2 opened
   (its fix strings named "affirm `decomposed`" with no command to run).

3. **`--yes` is a no-op runner.** No mutator prompts interactively, so `--yes`
   simply proceeds (identical to omitting it). This matches the existing CRUD
   mutators (`new`/`rename`/`retire`/`supersede`), which also accept and ignore
   `--yes`. If interactive confirmation is ever added, `--yes` is the bypass.

4. **In-process tests, not `assert_cmd`.** odm-cli is library-only (the `odm`
   binary is in `oxur-odm`), so the tests drive `dispatch` and assert on the
   persisted node files directly — the slice05 pattern. Round-trips through
   odm-store are real (atomic write-temp-rename); only the process boundary is
   elided.

5. **`link` re-link replaces a same-target `depends_on`.** To let a re-link
   update `--satisfied-at`, linking `depends_on` to a target already depended on
   removes the old entry and adds the new one (rather than erroring or
   duplicating). Other edge kinds dedup (push only if absent). `part_of` replaces
   (single parent). Flagged as the idempotency/replace policy.

## Uncertainties named

- **`run()` is uncovered** (the real-stdout/cwd wrapper); everything it delegates
  to is covered. TOTAL line 90.99% clears the ≥90% bar; the 95% target would
  need binary-level (`assert_cmd`) tests against `oxur-odm`.
- **No edge-existence precondition on `tear`.** `tear X depends_on Y` records the
  torn edge whether or not a `depends_on X→Y` exists; `check` uses tears only to
  exclude edges from cycle detection, so a tear with no matching edge is inert,
  not an error. Flagged in case CDC wants a "tear of a non-existent dependency"
  warning.
- **`set-gate` does not enforce gate ordering or monotonic evidence.** It records
  any in-set gate at any evidence (slice03's recorder semantics); sequence/
  monotonicity policy lives in the satisfaction model, not the CLI (per the
  slice-doc's explicit out-of-scope note).
- **Sandbox/CI parity.** All cargo evidence was produced on a local toolchain;
  CDC should reproduce on CI / a local 1.85+ run, from a clean
  `target/llvm-cov-target`.

## Arc 02 status

With this slice closed (pending CDC), **Arc 02 is complete and odm is
self-host-usable**: nodes, edges, gates, tears, and decomposition affirmations
can all be created and advanced through the CLI alone, and the resulting graph
answers `next`/`blocked`/`path`/`check`. The post-A3 migration of the plan into
odm now has the CLI surface it needs.
