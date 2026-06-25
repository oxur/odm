# Closing Report — Slice 01 (Arc 03): Arc 02 cleanup

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain). This slice closes the three Arc 02 CDC follow-ups and
> reconciles the two 0013 passages they touch.

- **Implementation commit:** `1baa3b8`.
- **Branch:** `arc03-slice01-arc02-cleanup` (based on `main`, which was
  CI-green; not pushed; not merged to `main`).
- **Scope delivered:** tear-rationale persistence (`TornEdge`), a binary-level
  `assert_cmd` suite for the real `odm` process, `check` recomposition-severity
  recalibration, and the 0013 §2.3/§4.3/§4.5 reconciliation.
- **Result:** 11 rows, all `done`. 0 deferred, 0 no-op-skipped.
- **Aggregate gates (reproduced locally on rustc 1.95.0):**
  `cargo test --workspace --all-features` → **187 passed**; `cargo clippy
  --all-targets -- -D warnings` → exit 0; no `unsafe` in `crates/*/src`;
  `cargo fmt --check` clean; line coverage **odm-core 98.68% / odm-cli 91.94% /
  oxur-odm 100%** (TOTAL 94.65%).

## Per-row walk

| ID | Status | Evidence (re-runnable at `1baa3b8`) |
|----|--------|-------------------------------------|
| C-1 | done | `cargo test -p odm-core tear_carries_rationale` → 1 passed. `TornEdge { edge: Dependency, because: String }` added to `odm-core/src/frontmatter.rs`; `Edges::tears` is now `Vec<TornEdge>`. Named distinctly from odm-graph's pure `Tear<N>` (cycle.rs). |
| C-2 | done | `cargo test -p odm-cli tear_persists_rationale` → 1 passed. `commands::tear` writes a `TornEdge` to the source node (was: validate via `Tear::new`, then drop). A re-tear of the same target refreshes the rationale; exactly one entry persists. |
| C-3 | done | `cargo test -p odm-core tears_roundtrip_identity` → 1 passed (128 proptest cases; mixes bare/qualified edges). Asserts emitted YAML carries `tears:` and `because:`, and `parse ∘ emit = identity`. |
| C-4 | done | `cargo test -p odm-core empty_tears_roundtrip` → 1 passed. Empty `tears` is omitted on emit (the `skip_serializing_if = "Vec::is_empty"` is retained on the field); a no-tears node round-trips byte-identically. **Migration is a no-op:** no `nodes/` dir exists and no on-disk `*.md` carries a `tears:` key — bare-form tears lived only in four test fixtures, now updated. |
| C-5 | done | `cargo test -p odm-cli check_lists_tear_rationale` → 1 passed. `check` prints `active tears (N):` with each rationale; `--json` gains an additive `tears: [{from,to,because}]`. Uses a new `NodeGraph::active_tears` wrapper (lists only tears naming a real ordering edge). |
| C-6 | done | `cargo test -p oxur-odm --test cli` → **6 passed**. New `crates/oxur-odm/tests/cli.rs` drives `Command::cargo_bin("odm")` with `current_dir` set to a `TempDir`, reaching `run()` and the real `ExitCode`. dev-deps `assert_cmd`/`predicates`/`tempfile`/`serde_json` added to `oxur-odm/Cargo.toml`. |
| C-7 | done | `check_exit_ok_on_clean_graph` → exit 0; `check_exit_violations_on_dirty_graph` → exit 1; `usage_error_exits_two` → exit 2. Verifies the `u8`→`ExitCode` path end-to-end. (Split into named tests rather than a single `check_exit_codes` — flagged.) |
| C-8 | done | `cargo test -p odm-cli check_recomposition_severities` → 1 passed. Asserts each finding's `severity` in `--json`: orphan = error, decomposition-drift = error, undeveloped-stub = warning, advanced-without-decomposition = warning. Logic in `recompose_severity(&Issue)`. |
| C-9 | done | `cargo test -p odm-cli stub_warns_default_fails_strict` → 1 passed. A warning-only corpus (advanced child-less arc) → exit 0 in default `check`, exit 1 under `--strict`. Matches staleness / soft-satisfaction treatment. |
| C-10 | done | `grep -n 'because' …0013…` → §2.3 schema (105) + §4.3 prose (171–172); `grep -n 'Decomposition { on, children }' …0013…` → §4.5 (217). Both match. §2.3 schema now shows typed `tears` and `decomposed`; doc bumped 1.7 → 1.8, `updated: 2026-06-25`. |
| C-11 | done | clippy `--all-targets -- -D warnings` → exit 0; `grep -RnE '\bunsafe\b' crates/*/src` → none; `cargo llvm-cov --workspace` line coverage odm-core 98.68% / odm-cli 91.94% / oxur-odm 100%; `cargo fmt --check` clean; full workspace test 187 passed. |

## 0013 reconciliation (the diff)

Three hunks in `docs/design/01-draft/0013-odm-architecture-design.md`:

1. **Frontmatter:** `version: 1.7 → 1.8`, `updated: 2026-06-20 → 2026-06-25`.
2. **§2.3 (normative schema):** the bare `tears: []` example becomes the typed
   `{ edge, because }` form (with the "omitted when empty" note), and a typed
   `decomposed: { on, children }` block is added — the schema now matches the
   realized `TornEdge` and `Decomposition` types.
3. **§4.3:** prose now states the tear entry is `{ edge, because }` and that the
   rationale is *persisted* (not merely validated), and that `check` lists active
   tears **with their rationale**.
4. **§4.5:** "assert `decomposed: complete`" becomes "affirm a typed
   `decomposed: Decomposition { on, children }`", noting the recorded child set
   (the drift-guard enrichment realized in arc02 slice05).

## What Worked

- **The bare-form-tear migration was genuinely a no-op** — verified by `grep`
  (no on-disk node carries `tears:`; no `nodes/` dir). That let `TornEdge` be a
  plain struct with an always-required rationale, rather than carrying a
  back-compat deserializer with an awkward empty-`because` state.
- **`TornEdge` (persistence) vs `Tear<N>` (engine)** kept the two concerns
  distinct: frontmatter holds `TornEdge { edge, because }`; graph-build maps it to
  `Tear<Id>` carrying the real rationale, which now flows into both the cycle
  detector and `check`'s active-tears listing.
- **`assert_cmd` + `current_dir` closed the `run()` gap** with a small, real
  end-to-end suite — the first tests to reach the binary's `ExitCode`.
- **The severity recalibration reused existing machinery** (one
  `recompose_severity` function); `check_recomposition_variants` passes unchanged.

## Uncertainties named

- **C-7 verify name.** Delivered as `check_exit_ok_on_clean_graph` +
  `check_exit_violations_on_dirty_graph` (+ `usage_error_exits_two`) rather than a
  single `check_exit_codes`. Same coverage, clearer names.
- **`--json` envelope gained a `tears` array** (additive; existing keys
  unchanged). `check_json_schema` updated to expect the new key. A disclosed,
  backward-compatible schema evolution (continuing slice06 CDC note #4).
- **Q-7 appendix** in 0013 still reads "drift-guarded `decomposed: complete`" as
  shorthand — outside the named §-scope, and the phrase already implies the
  enrichment, so it was left untouched.
- **Toolchain parity.** All cargo rows were **reproduced** on a local rustc
  1.95.0 (the sandbox's "no 1.85" caveat did not apply here). CDC should still
  re-run on CI from a clean `target/llvm-cov-target` for parity.

## Status

The three Arc 02 follow-ups are closed and 0013 is honest. This unblocks slice02
(the rollup): tear rationale is now persisted and renderable, and `check`
severities are calibrated before `orient` surfaces them. Pending CDC verification.
