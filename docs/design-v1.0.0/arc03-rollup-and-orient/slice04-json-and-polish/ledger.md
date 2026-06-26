# Slice 04 (Arc 03): `--json` + polish

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+ (sandbox has no
> 1.85 toolchain). CDC-authored acceptance rows; CC fills Status/Evidence/Notes per
> commit. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| J-1 | `odm rollup --json` serializes the `Rollup` model (tree, status, ready/blocked+soft, tears+rationale, provenance, drift/deferred slots) as valid JSON over the **same model** as the Markdown render | `cargo test -p odm-cli rollup_json_serializes_model` → ok | serious | 0013 §6/§7 / arc-plan D-3 | open | | `#[derive(Serialize)]` views + `to_string_pretty` (established pattern); no second derivation. |
| J-2 | `odm orient --json` and `brief --json` serialize the orient view (vision, focus, ready/blocked, integrity, drift) as valid JSON over the same model | `cargo test -p odm-cli orient_json_serializes_view` → ok | serious | 0013 §4.1/§7 | open | | |
| J-3 | The `check` v2 envelope is pinned: a shape test locks its keys (`ok`, `errors`, `warnings`, `findings[]`, `tears[]`) + finding/tear field sets | `cargo test -p odm-cli check_json_envelope_shape_locked` → ok | serious | slice01 forward note / slice02 ruling 4 | open | | Catches accidental schema drift (it has evolved twice un-pinned). |
| J-4 | The `rollup` and `orient` `--json` envelopes are shape-locked by tests (keys + types) | `cargo test -p odm-cli rollup_json_shape_locked` + `orient_json_shape_locked` → ok | serious | 0013 §7 ("stable, documented schemas") | open | | |
| J-5 | An additive `schema_version` marker is present on the `check`/`rollup`/`orient` `--json` envelopes | `cargo test -p odm-cli json_schema_version_marker` → ok | serious | interop forward-compat / 0017 | open | | Additive — existing consumers unaffected. **Flagged for ratification** (see slice-doc). |
| J-6 | 0013 §7 documents the canonical `--json` schemas for `check`, `rollup`, and `orient` | `grep -nE 'JSON output schemas\|schema.*rollup\|schema.*orient' docs/design/01-draft/0013-odm-architecture-design.md` → matches | polish | 0013 §7 | open | | Doc row — the contract 0017 export targets. |
| J-7 | `--json` stays valid (parseable) and never bare-errors on the **empty corpus** and **no-project** paths | `cargo test -p odm-cli json_valid_on_empty_and_no_project` → ok | correctness | never-bare-errors / 0013 §7 | open | | Includes the three orient no-project branches with `--json`. |
| J-8 | Errors-as-affordances sweep: every `orient`/`rollup` message + no-project/empty path names an exact fix command | `cargo test -p odm-cli orient_rollup_affordances_name_fixes` → ok | polish | 0001 / 0013 §7 | open | | Consistency pass across the A3 surface. |
| J-9 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-core + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

_(Filled in at slice close. Total rows: 9. On close: Arc 03 done → MVP A1–A3 complete.)_
