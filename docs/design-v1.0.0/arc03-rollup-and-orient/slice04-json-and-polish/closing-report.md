# Closing report — Arc 03 / Slice 04: `--json` + polish

> CC implementation closing report. Status: **proposed done** → CDC verifies the
> cargo rows via CI / a local 1.85+ toolchain, then runs the arc-level
> recomposition / silent-drop check. Impl commit `60f4d57`; docs commit (this
> report + ledger) follows. **On close, Arc 03 is done and the MVP (A1–A3) is
> complete.**

## What this slice built

A stable, documented, shape-locked `--json` for every A3 query, serialized from
the slice02 `Rollup` model (D-3 — no second derivation), plus the
errors-as-affordances polish pass. This is the machine contract ODD-0017's export
projection will target.

- **`crates/odm-cli/src/json.rs`** — `#[derive(Serialize)]` view structs that are
  a 1:1 projection of the model, each with a `From<&Model>` impl. Both the
  Markdown render and the JSON encode read the same `&Rollup`, so they cannot
  drift in data — only in encoding. Block reasons are internally tagged by
  `kind`; the `drift`/`deferred` slots serialize as present-but-empty.
- **`odm rollup --json`** is a non-writing output mode (serialize to stdout, no
  file). **`odm orient --json` / `brief --json`** serialize the orient view over
  the same model; the no-project fallbacks emit a valid envelope with a `hint`.
- **`schema` marker** (`check/v1` / `rollup/v1` / `orient/v1`) added additively to
  each envelope.
- **0013 §7.1** documents all three canonical schemas; doc bumped 1.8 → 1.9.

## Per-row ledger walk (9 rows)

- **J-1 — done.** `RollupJson: From<&Rollup>` serializes tree (gate-sequence
  status), ready+soft, blocked+reasons, tears+rationale, provenance, and the
  drift/deferred slots. `rollup_json_serializes_model` + the block-reason-variants
  test cover the content and all three reason kinds.
- **J-2 — done.** `OrientJson` serializes project/vision/focus, ready/blocked,
  integrity errors, and drift over the same model; `brief --json` is byte-identical
  to `orient --json`.
- **J-3 — done.** The `check` v2 envelope is shape-locked (top keys + finding +
  tear field sets) in `check_json_envelope_shape_locked`, and the two pre-existing
  check shape tests were updated for the additive `schema` key.
- **J-4 — done.** `rollup_json_shape_locked` and `orient_json_shape_locked` lock
  each envelope's keys, types, and nested field sets (tree node, provenance,
  drift, tagged block reason; focus, integrity).
- **J-5 — done.** `json_schema_version_marker` asserts the marker on all three
  envelopes. *(Flagged — see deviations.)*
- **J-6 — done.** 0013 §7.1 "JSON output schemas (canonical)" documents the three
  contracts; the grep matches.
- **J-7 — done.** `json_valid_on_empty_and_no_project` parses `rollup --json` on
  an empty corpus and `orient --json` on all three no-project branches (none/one/
  many), each exiting 0; fallbacks carry `project: null` + a `hint`.
- **J-8 — done.** `orient_rollup_affordances_name_fixes` asserts the human and
  JSON no-project/multi-project paths name `odm new project` / `odm use project`.
- **J-9 — done.** clippy `-D warnings` → exit 0; no `unsafe`; line coverage
  odm-core **98.69%**, odm-cli **94.40%** (json.rs 100%, orient.rs 98.22%,
  rollup.rs 98.82%); `fmt --check` clean; full workspace `cargo test` → **227
  passed, 0 failed**.

## Deviations / decisions flagged (not buried)

1. **`schema` marker — new convention, flagged for ratification (J-5).** I chose
   `"<command>/v1"` (`check/v1`, `rollup/v1`, `orient/v1`) and decided the marker
   versions the contract **from its introduction (this slice) forward** — the
   `check` envelope's two earlier unmarked evolutions (slice06 severity/code, arc02
   `tears`) are pre-history, so `check` is `v1` *of the marked contract*, not `v2`.
   The alternative was `check/v2` to reflect the envelope generation; I judged a
   uniform `/v1` across all three cleaner for consumers and documented the
   rationale in 0013 §7.1 and the json.rs module doc. **CDC's call to ratify or
   switch to `check/v2`.**
2. **Adding `schema` changed the `check` top-level key set.** This is additive for
   real consumers (they ignore unknown keys), but it *did* require updating three
   shape-lock tests (the two pre-existing check tests + the binary suite). Flagged
   so the CDC sees the key-set change was intentional, not drift.
3. **`rollup --json` does not write `ROLLUP.md`.** I treated `--json` as a
   non-writing query-output mode (consistent with every other `--json` command),
   so `odm rollup --json` prints to stdout and writes no file; `odm rollup` (no
   flag) still writes the file. Flag if `--json` should *also* regenerate the file.
4. **`orient --json` integrity carries errors only**, mirroring the human orient
   view (which surfaces only errors). A consumer wanting warnings calls
   `check --json`. Faithful to the view, not a superset.

## Uncertainties / things CDC should look at

- **The orient "many projects" JSON fallback does not list the projects** — it
  carries `project: null` + a `hint` naming `odm use project <ref>`, but not the
  candidate list (the human path lists them). A JSON consumer in this state would
  call `odm list --type project --json`. Flag if the list should be inline; I kept
  the key set fixed across states for a lockable shape.
- **`list`/`show`/`context` were left unmarked.** The slice scoped the marker to
  `check`/`rollup`/`orient`; the already-stable query schemas are noted in 0013
  §7.1 as candidates for a marker when ODD-0017 lands. Flag if they should be
  marked now for uniformity.
- **No remaining uncovered lines in the new `json.rs` (100%).** orient.rs/rollup.rs
  retain the previously-named rare/defensive arms; both ≥ 98% line.

## Iterations

One. The only rework was updating two pre-existing `check` shape tests for the
additive `schema` key (caught immediately by the full-suite run) and adding one
coverage test for the block-reason serialization variants.

On close, **Arc 03 is done and the MVP (A1–A3) is complete** — every query has a
stable, documented, shape-locked `--json`. This is the self-hosting trigger: the
plan migrates *into* `odm` as nodes (A6). CDC runs the arc-level recomposition /
silent-drop check before the arc is declared closed.
