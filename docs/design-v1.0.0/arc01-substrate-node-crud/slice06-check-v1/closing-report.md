# Closing Report — Slice 06 (Arc 01): `check` v1 + link-integrity

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before Arc 01 is declared complete.

- **Implementation commit:** `5b39c54`.
- **Branch:** `slice06-check-v1` (based on `slice05-node-crud`; not pushed; not
  merged to `main`).
- **Scope delivered:** `odm check` — required-field completeness, link-integrity,
  supersession-chain integrity — with exit codes `0`/`1`/`2`, fix affordances,
  and a `--json` report. **Closes Arc 01.**
- **Result:** 9 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test -p odm-cli` → 30 tests pass (8 new check);
  `cargo test -p odm-core --test check` → 9 pass; clippy (both crates) `-D
  warnings` → exit 0; no `unsafe`; coverage line 95.84% / region 92.49% across
  odm-cli+odm-core (`check.rs` 99% line). Workspace clippy/fmt clean; `odm
  --version` and `odm check` (clean → exit 0) verified on the real binary.

## Per-row walk

| ID | Status | Evidence (re-runnable at `5b39c54`) |
|----|--------|-------------------------------------|
| L-1 | done | `cargo test -p odm-cli check_missing_field` → 1 passed; empty `name` → `missing-field`, exit 1. |
| L-2 | done | `cargo test -p odm-cli check_dangling_part_of` → 1 passed; absent `part_of` target → flagged. |
| L-3 | done | `cargo test -p odm-cli check_dangling_edge` → 1 passed; absent edge target → flagged (all 8 kinds in odm-core test). |
| L-4 | done | `cargo test -p odm-cli check_supersession_chain` → 1 passed (self-supersede); cycle + terminating chain in odm-core tests. |
| L-5 | done | `cargo test -p odm-cli check_clean_passes` → 1 passed; clean corpus → `check: ok`, exit 0. |
| L-6 | done | `cargo test -p odm-cli check_exit_codes_v1` → 1 passed; 0 clean / 1 violations / clap → 2 usage. |
| L-7 | done | `cargo test -p odm-cli check_errors_name_fix_v1` → 1 passed; every finding has a `fix:`; empty-name → `odm rename`. |
| L-8 | done | `cargo test -p odm-cli check_json_v1` → 1 passed; stable `{ok, findings:[…]}` schema; clean → `ok:true`. |
| L-9 | done | clippy (odm-cli+odm-core) exit 0; no `unsafe`; `cargo llvm-cov … --ignore-filename-regex '(odm-store\|odm-graph)/'` → line 95.84%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **Required-field check v1 = non-empty `name` (data-driven, per-type table).**
   The slice03 schema makes id/number/type/name/created/updated/origin
   *mandatory at parse time* — a file missing one fails to load (a store error),
   so it never reaches the validator. The field that can be present-but-empty is
   `name`, and it has a real Arc-01 fix (`odm rename`). I implemented the rule via
   a `required_fields(type) -> &[&str]` table (currently `["name"]` for all
   types), so v2 can add type-specific requirements (e.g. gate presence) without
   restructuring. **Flagging** that "per node type" is, at v1, a type-aware
   mechanism with a uniform rule — not yet type-varying. If CDC wants a richer v1
   rule (e.g. work nodes must declare `part_of`), note its fix would reference the
   `link` command, which is Arc 02 — hence deferred.

2. **Exit-code model: `0` clean, `1` `check` violations, `2` error (incl. clap
   usage).** `dispatch` now returns `anyhow::Result<u8>`; `run` maps `Ok(code) ->
   ExitCode::from(code)` and `Err -> 2`. clap already exits `2` on argument
   errors. This makes "ran, found problems" (1) distinct from "couldn't run" (2),
   the conventional linter split. **Side effect:** slice05 command *errors*
   (unknown ref, bad type) now exit `2` instead of the old generic `1` — more
   principled, and no slice05 test pinned the code. `oxur-odm`'s `main` is now
   `fn main() -> ExitCode { odm_cli::run() }`.

3. **Tests are in-process (`dispatch`), not `assert_cmd`.** Same situation and
   resolution as slice05 (user-approved): odm-cli is library-only, so the check
   tests parse a `Cli` and call `dispatch` against a `TempDir`, asserting the
   returned exit code and captured output. The ledger's `cargo test -p odm-cli
   check_… → ok` commands all pass; only the "(assert_cmd)" hint on L-1 is
   superseded. (Recommend CDC fold this into the same hint-amendment noted for
   slice05.)

4. **Fix affordances are commands where one exists, else a precise file edit.**
   errors-as-affordances (L-7) wants the exact fix. Empty `name` → `odm rename
   <id> "<name>"` (a real command). But dangling refs / self-supersede /
   supersession cycles are fixed by editing the node's `edges` — and the
   `link`/`unlink` mutators that would do that from the CLI are **Arc 02**. So
   those affordances name the precise file and field to edit (e.g. "edit
   `nodes/…/<id>.md`: repoint `edges.part_of` …"). Every finding still carries an
   actionable `fix:`; some are manual edits until Arc 02 adds the commands.
   Flagging the manual-edit affordances.

5. **`check` reports to stdout (it is the query's data); exit code is the
   signal.** Consistent with the data→stdout rule: the findings report (human or
   `--json`) is the command's output; CI consumes the exit code. No diagnostics
   are written to stderr by `check` on the success/violations paths (only a true
   operational error → stderr via `run`).

## Uncertainties named

- **Supersession-cycle attribution.** A detected cycle is reported once, attributed
  to the smallest-id node in the cycle, with the full cycle in the finding. That
  is deterministic and de-duplicated, but the "owning" node is an arbitrary
  (smallest-id) choice — fine for a structural report; noted in case CDC expects
  per-node attribution.
- **`check` operates on the parsed corpus.** If a node *file* is malformed enough
  not to parse, `load_all` errors and `check` exits `2` ("couldn't load") rather
  than reporting a per-file finding — it does not partial-load. Reporting
  unparseable files individually is a possible v2 nicety; v1 treats an unloadable
  corpus as an operational error.
- **Coverage clean-state caveat (carried from slice05).** `cargo llvm-cov` must be
  run from a clean `target/llvm-cov-target`; a stale instrumented dir produced a
  bogus number until cleared. The 95.84% figure is from a clean rebuild.
- **Sandbox has no Rust toolchain**, so all cargo evidence was produced on the
  local dev host; CDC should reproduce on CI / a local 1.85+ run.

## Arc 01 status

On CDC sign-off this is the last slice of Arc 01. The substrate is in place:
stable ULID identity (02), round-trip frontmatter schema (03), the
`nodes/YYYY/MM/<ULID>.md` git-native store (04), node CRUD + context (05), and
this structural `check` (06). Arc 02 builds the graph engine and `check` v2 on
top, extending — not rewriting — `odm_core::check`.
