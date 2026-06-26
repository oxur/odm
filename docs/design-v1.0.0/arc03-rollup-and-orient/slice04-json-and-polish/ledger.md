# Slice 04 (Arc 03): `--json` + polish

> Per LEDGER_DISCIPLINE. Cargo rows reproduced on a local 1.95.0 toolchain (the
> 1.85+ floor is met here); CDC re-runs them via CI / a local 1.85+ toolchain for
> the independent gate. CDC-authored acceptance rows; CC filled Status/Evidence/
> Notes per commit. Five-iteration cap (closed in 1).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| J-1 | `odm rollup --json` serializes the `Rollup` model (tree, status, ready/blocked+soft, tears+rationale, provenance, drift/deferred slots) as valid JSON over the **same model** as the Markdown render | `cargo test -p odm-cli rollup_json_serializes_model` → ok | serious | 0013 §6/§7 / arc-plan D-3 | done | `60f4d57`; `rollup_json_serializes_model` → 1 passed (gate-sequence status, soft-sat flag, tear rationale, discovered/amendment provenance); `rollup_json_block_reason_variants` → 1 passed (all three block-reason kinds). `RollupJson: From<&Rollup>` (`json.rs`). | `#[derive(Serialize)]` views built from `&Rollup`; no second derivation. |
| J-2 | `odm orient --json` and `brief --json` serialize the orient view (vision, focus, ready/blocked, integrity, drift) as valid JSON over the same model | `cargo test -p odm-cli orient_json_serializes_view` → ok | serious | 0013 §4.1/§7 | done | `60f4d57`; `orient_json_serializes_view` → 1 passed (project/vision/focus, integrity `orphan` error, and `brief --json` == `orient --json`). | Same model + the `check` aggregation; `OrientJson` in `json.rs`. |
| J-3 | The `check` v2 envelope is pinned: a shape test locks its keys (`ok`, `errors`, `warnings`, `findings[]`, `tears[]`) + finding/tear field sets | `cargo test -p odm-cli check_json_envelope_shape_locked` → ok | serious | slice01 forward note / slice02 ruling 4 | done | `60f4d57`; `check_json_envelope_shape_locked` → 1 passed (locks `[errors, findings, ok, schema, tears, warnings]`; finding fields `[code, detail, fix, name, node, number, severity]`; tear fields `[because, from, to]`). Also re-locked in `cli.rs`/`oxur-odm` tests. | Now includes the additive `schema` key (see J-5). |
| J-4 | The `rollup` and `orient` `--json` envelopes are shape-locked by tests (keys + types) | `cargo test -p odm-cli rollup_json_shape_locked` + `orient_json_shape_locked` → ok | serious | 0013 §7 ("stable, documented schemas") | done | `60f4d57`; both → 1 passed (rollup keys `[blocked, deferred, drift, provenance, ready, schema, tears, tree]` + tree-node + provenance + drift field sets + tagged block reason; orient keys `[blocked, drift, focus, hint, integrity, project, ready, schema, vision]` + focus/integrity field sets). | |
| J-5 | An additive `schema_version` marker is present on the `check`/`rollup`/`orient` `--json` envelopes | `cargo test -p odm-cli json_schema_version_marker` → ok | serious | interop forward-compat / 0017 | done | `60f4d57`; `json_schema_version_marker` → 1 passed (`check/v1`, `rollup/v1`, `orient/v1`). | Additive — existing consumers unaffected. **Flagged for ratification** (slice-doc): value form `<command>/v1`, versioning the contract from introduction forward (check's two prior unmarked evolutions are pre-history). |
| J-6 | 0013 §7 documents the canonical `--json` schemas for `check`, `rollup`, and `orient` | `grep -nE 'JSON output schemas\|schema.*rollup\|schema.*orient' docs/design/01-draft/0013-odm-architecture-design.md` → matches | polish | 0013 §7 | done | `60f4d57`; grep → 4 matches (the new `### 7.1 JSON output schemas (canonical)` + the three per-command bullets). Doc bumped 1.8 → 1.9. | The contract ODD-0017 export targets. |
| J-7 | `--json` stays valid (parseable) and never bare-errors on the **empty corpus** and **no-project** paths | `cargo test -p odm-cli json_valid_on_empty_and_no_project` → ok | correctness | never-bare-errors / 0013 §7 | done | `60f4d57`; `json_valid_on_empty_and_no_project` → 1 passed (empty corpus → empty `tree`; the three orient branches none/one/many all parse and exit 0; fallbacks carry `project: null` + a `hint`). | |
| J-8 | Errors-as-affordances sweep: every `orient`/`rollup` message + no-project/empty path names an exact fix command | `cargo test -p odm-cli orient_rollup_affordances_name_fixes` → ok | polish | 0001 / 0013 §7 | done | `60f4d57`; `orient_rollup_affordances_name_fixes` → 1 passed (human + JSON no-project paths name `odm new project`; multi-project paths name `odm use project`). | Consistency pass across the A3 surface. |
| J-9 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-core + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | done | `60f4d57`; clippy `--all-targets --all-features --workspace` → exit 0; `unsafe` grep → no matches (exit 1); `cargo llvm-cov` **line**: odm-core **98.69%**, odm-cli **94.40%** (json.rs 100%, orient.rs 98.22%, rollup.rs 98.82%). | `fmt --check` clean; full workspace `cargo test` → 227 passed. |

## What Worked

- **The model-as-single-source design (D-3) made `--json` almost mechanical.**
  Both Markdown and JSON read `&Rollup`; the JSON layer is `#[derive(Serialize)]`
  views with `From<&Model>` impls — a 1:1 projection that *cannot* drift from the
  human render because there is no second derivation, only a second encoding.
- **Shared view structs in one `json.rs`** meant `rollup` and `orient` serialize
  the same `NodeRef`/`ready`/`blocked` shapes from one definition — no parallel
  structures to keep in sync.
- **Shape-lock tests via `serde_json::Value` + sorted key sets** are a cheap,
  durable contract: any accidental field add/rename/removal fails CI. This is
  exactly what the `check` envelope lacked when it silently evolved twice.
- **Keeping the orient envelope's key set fixed across all states** (resolved vs.
  the three no-project fallbacks, distinguished by nullable `project`/`hint`)
  made J-7 a clean property rather than a special-cased schema.

## Closure

Closed at commit `60f4d57` on 2026-06-26 (CC implementation). CDC verification:
pending (cargo rows via CI / local 1.85+); CDC then runs the **arc-level
recomposition / silent-drop check** before Arc 03 is declared closed. Total rows:
9. Done: 9. Deferred: 0. No-op: 0. **On close: Arc 03 done → MVP A1–A3 complete.**
