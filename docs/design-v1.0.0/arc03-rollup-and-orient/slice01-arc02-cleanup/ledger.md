# Slice 01 (Arc 03): Arc 02 cleanup

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+ (sandbox has no
> 1.85 toolchain). CDC-authored acceptance rows; CC fills Status/Evidence/Notes per
> commit. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| C-1 | `tears` entries are a typed struct carrying the torn `edge` (a `Dependency`) **and** the `because` rationale (was a bare `Vec<Dependency>`) | `cargo test -p odm-core tear_carries_rationale` → ok | serious | slice07 CDC #1 / 0013 §4.3 | done | `cargo test -p odm-core tear_carries_rationale` → 1 passed (`1baa3b8`). `TornEdge { edge, because }` in frontmatter.rs; `Edges::tears: Vec<TornEdge>`. | Named `TornEdge`, distinct from odm-graph's pure `Tear<N>` (cycle.rs), per the cc-prompt. |
| C-2 | `odm tear X depends_on Y --because <r>` **persists** the rationale on source X (not just validates it) | `cargo test -p odm-cli tear_persists_rationale` (in-process dispatch) → ok | serious | slice07 CDC #1 | done | `cargo test -p odm-cli tear_persists_rationale` → 1 passed. On-disk #1 carries `tears:` + `because:` + the text; a re-tear refreshes the rationale (still one entry). | `commands::tear` writes a `TornEdge`; was previously dropped after `Tear::new` validation. |
| C-3 | A populated `tears` round-trips: `parse ∘ emit = identity` | `cargo test -p odm-core tears_roundtrip_identity` (proptest) → ok | serious | 0013 §2.3 | done | `cargo test -p odm-core tears_roundtrip_identity` → 1 passed (128 proptest cases); mixes bare/qualified edges, asserts emitted YAML carries `tears:`/`because:`. | |
| C-4 | Empty `tears` is omitted on emit; arc01/02 nodes (no tears) round-trip byte-identically | `cargo test -p odm-core empty_tears_roundtrip` → ok | correctness | back-compat | done | `cargo test -p odm-core empty_tears_roundtrip` → 1 passed; emit omits `tears:` and `emit∘parse∘emit` is byte-identical. | Confirmed **no on-disk node carries a bare-form tear** (`grep -rln 'tears:' --include='*.md'` outside docs/legacy/target → none; no `nodes/` dir). Migration is a **no-op** (pre-release; bare form lived only in test fixtures, now updated). |
| C-5 | `check`'s active-tears listing surfaces each tear's rationale | `cargo test -p odm-cli check_lists_tear_rationale` → ok | serious | 0013 §4.3 | done | `cargo test -p odm-cli check_lists_tear_rationale` → 1 passed; human output prints `active tears (N):` with `because:`; `--json` gains an additive `tears: [{from,to,because}]`. | Uses `NodeGraph::active_tears` (new wrapper) so only tears naming a real edge are listed. |
| C-6 | A binary-level `assert_cmd` suite exists in `oxur-odm/tests/` and exercises the real `odm` process (`run()`) end-to-end | `cargo test -p oxur-odm --test cli` → ok | serious | slice06/07 CDC | done | `cargo test -p oxur-odm --test cli` → 6 passed (`oxur-odm/tests/cli.rs`). Drives `Command::cargo_bin("odm")` with `current_dir` set; reaches `run()`/real `ExitCode`. | dev-deps `assert_cmd`/`predicates`/`tempfile`/`serde_json` added to oxur-odm. |
| C-7 | Real `odm check` exits `EXIT_OK` (0) on a clean graph and `EXIT_VIOLATIONS` (1) on violations | `cargo test -p oxur-odm --test cli check_exit_codes` → ok | serious | slice06 CDC #1 | done | `check_exit_ok_on_clean_graph` (code 0) + `check_exit_violations_on_dirty_graph` (code 1) + `usage_error_exits_two` (code 2) → all pass. | Verifies the `u8`→`ExitCode` path end-to-end. (Split across named tests rather than one `check_exit_codes`; flagged.) |
| C-8 | Recomposition severities recalibrated: orphan + decomposition-drift = `Error`; undeveloped-stub + advanced-without-decomposition = `Warning` | `cargo test -p odm-cli check_recomposition_severities` → ok | serious | slice06 CDC #2 | done | `cargo test -p odm-cli check_recomposition_severities` → 1 passed; asserts each code's `severity` in `--json` (orphan/drift=error, stub/advanced-without=warning). | `recompose_severity(&Issue)` in commands.rs. |
| C-9 | An undeveloped-stub / advanced-without-decomposition finding does **not** fail default `check` (exit 0) but **does** fail `--strict` (exit 1) | `cargo test -p odm-cli stub_warns_default_fails_strict` → ok | serious | slice06 CDC #2 | done | `cargo test -p odm-cli stub_warns_default_fails_strict` → 1 passed; warning-only corpus → exit 0 default, exit 1 `--strict`. | Matches staleness / soft-satisfaction treatment. |
| C-10 | 0013 reconciled: §2.3/§4.3 show `tears` carrying a `because` rationale; §2.3/§4.5 show `decomposed` as typed `Decomposition { on, children }` (not the scalar) | `grep -n 'because' docs/design/01-draft/0013-odm-architecture-design.md` (tears example/§4.3) AND `grep -n 'Decomposition { on, children }' docs/design/01-draft/0013-odm-architecture-design.md` → both match | polish | doc-honesty | done | `grep 'because'` → §2.3 schema (line 105) + §4.3 prose (171–172); `grep 'Decomposition { on, children }'` → §4.5 (line 217). §2.3 schema now shows typed `tears`/`decomposed`. Doc bumped 1.7→1.8. | Q-7 appendix keeps "drift-guarded `decomposed: complete`" as shorthand (out of the named §-scope; flagged). |
| C-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-core, odm-cli, and oxur-odm | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/*/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | done | clippy `--all-targets -- -D warnings` → exit 0; `grep -RnE '\bunsafe\b' crates/*/src` → none; `cargo llvm-cov --workspace` line: **odm-core 98.68%, odm-cli 91.94%, oxur-odm 100%**; TOTAL 94.65%; `cargo fmt --check` clean; full `cargo test --workspace` → 187 passed. | `run()` (odm-cli/lib.rs, 91.49% line) is now covered **by the oxur-odm assert_cmd suite** — measured workspace-wide (a `-p odm-cli`-only run can't reach it). |

## What Worked

- **The bare-form-tear migration was genuinely a no-op.** A `grep` for `tears:`
  across on-disk `*.md` (outside docs/legacy/target) returned nothing, and there
  is no `nodes/` directory — odm is not yet self-hosting. The only bare-form
  tears lived in four test fixtures, which were updated to the `TornEdge` form.
  This let `TornEdge` be struct-only (rationale always required) instead of
  carrying a back-compat deserializer with an awkward empty-`because` state.
- **Naming the persistence type `TornEdge` cleanly sidestepped the `Tear<N>`
  collision.** Frontmatter carries `TornEdge { edge, because }`; graph-build maps
  it to odm-graph's pure `Tear<Id>` (now with the real rationale). The rationale
  now flows disk → cycle-detector → `check`, where before it was dropped after
  `Tear::new` validation.
- **The `assert_cmd` suite finally closed the `run()` coverage gap.** Setting the
  spawned command's `current_dir` to a `TempDir` is all it took to drive a real
  end-to-end graph (`new`/`link`/`tear`/`check`) against the built binary, hitting
  the `u8`→`ExitCode` path the in-process `dispatch` tests structurally cannot.
- **The severity recalibration was a one-function change** (`recompose_severity`)
  that reused the existing Error/Warning/`--strict` machinery — no new policy
  code. The existing `check_recomposition_variants` test still passes unchanged
  (decomposition-drift keeps it at exit 1).

## Uncertainties / flagged deviations

- **C-7 verify name.** The ledger named one test `check_exit_codes`; the suite
  splits it into `check_exit_ok_on_clean_graph` + `check_exit_violations_on_dirty_graph`
  (+ `usage_error_exits_two` for the exit-2 path). Same coverage, clearer names.
- **`--json` envelope gained a `tears` array.** Additive (existing
  `ok`/`errors`/`warnings`/`findings` keys unchanged), so v2 consumers still work;
  `check_json_schema` updated to expect the new key. Flagged as a disclosed,
  additive schema evolution (continuing the slice06 CDC note #4).
- **Doc Q-7 appendix** still reads "drift-guarded `decomposed: complete`" as a
  shorthand. It is outside the cc-prompt's named scope (§2.3/§4.3/§4.5), and
  "drift-guarded … assertion" already implies the enrichment, so it was left.
- **Toolchain.** The sandbox note ("no 1.85") did not apply here — a local 1.85+
  toolchain (rustc 1.95.0) was present, so all cargo rows were **reproduced**,
  not merely attested. CDC should still re-run on CI for parity.

## Closure

All 11 rows **done** (0 deferred, 0 no-op-skipped). Implementation commit
`1baa3b8`. Gates: `cargo test --workspace --all-features` → 187 passed; clippy
`--all-targets -- -D warnings` → exit 0; no `unsafe`; `cargo fmt --check` clean;
line coverage odm-core 98.68% / odm-cli 91.94% / oxur-odm 100% (TOTAL 94.65%).
CC's `done` is **proposed done** → CDC verifies (cargo rows via CI / local 1.85+).
Unblocks slice02 (tear rationale is now renderable in the rollup; `check`
severities are calibrated before `orient` surfaces them). Total rows: 11.
