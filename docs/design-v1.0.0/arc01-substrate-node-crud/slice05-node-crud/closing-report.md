# Closing Report — Slice 05 (Arc 01): Node CRUD commands

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 06 opens.

- **Commits:** `f33c983` (initial CRUD) → `af190ab` (in-process test refactor,
  user-approved — see decision 5); plus `d00f436` (slice04 gix-reflog CI fix).
  Evidence below is reproducible at the branch tip `af190ab`.
- **Branch:** `slice05-node-crud` (based on `slice04-store-layer`; not pushed;
  not merged to `main`).
- **Scope delivered:** `new`/`list`/`show`/`rename`/`retire`/`supersede`,
  `use`/`context`, with `--dry-run`/`--yes`/`--json`.
- **Result:** 11 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised
  (the assert_cmd → in-process switch was a user-directed change, logged below).
- **Aggregate gates:** `cargo test -p odm-cli` → 22 in-process tests pass;
  `cargo clippy -p odm-cli --all-targets -- -D warnings` → exit 0; odm-cli
  coverage line 96.17% / region 90.31% (odm-core/store/graph excluded). Umbrella
  `odm --version` still works; workspace clippy/fmt clean.

## Per-row walk

| ID | Status | Evidence (re-runnable at `af190ab`) |
|----|--------|-------------------------------------|
| K-1 | done | `cargo test -p odm-cli new_persists` → 1 passed; one `.md` persisted, next-number = max+1. |
| K-2 | done | `cargo test -p odm-cli new_idempotent` → 1 passed; re-run prints `exists:`, count stays 1. |
| K-3 | done | `cargo test -p odm-cli list_filters` → 1 passed; tag/component filters in `show_renders_all_fields_and_children`. |
| K-4 | done | `cargo test -p odm-cli show_node` → 1 passed; full field/edge/children rendering in the rich-fixture test. |
| K-5 | done | `cargo test -p odm-cli rename_keeps_id_and_path` → 1 passed; same path before/after, id intact. |
| K-6 | done | `cargo test -p odm-cli retire_preserves_file` → 1 passed; file kept, `retired:` recorded. |
| K-7 | done | `cargo test -p odm-cli supersede_with_kind` → 1 passed; edge on newer node with `kind`. |
| K-8 | done | `cargo test -p odm-cli context_use_and_show` → 1 passed; `.odm/context.json`; mismatch rejected. |
| K-9 | done | `cargo test -p odm-cli dry_run_and_yes` → 1 passed; dry-run (new/retire/supersede) writes nothing; `--yes` runs. |
| K-10 | done | `cargo test -p odm-cli json_schema_crud` → 1 passed; exact key set + stable values. |
| K-11 | done | clippy exit 0; no `unsafe`; `cargo llvm-cov … --ignore-filename-regex '(odm-core\|odm-store\|odm-graph)/'` → line 96.17%, region 90.31%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **New `retired` field on the §2.3 schema (odm-core).** `retire` needs a
   persisted, round-tripping marker, and the schema had none. I added a typed
   `Frontmatter.retired: Option<Retirement { reason, on }>` (skipped when
   absent, so slice03's canonical-order snapshot is unaffected) plus the
   mutating accessors `Document::frontmatter_mut`, `Frontmatter::{set_name,
   set_updated, edges_mut, retire}`. This **extends the normative §2.3 schema**
   (which doesn't list `retired`). **Recommend** §2.3 document `retired`.
   (Open question for Arc 02: whether retirement becomes a terminal *gate* and
   this field folds in.)

2. **`supersede X --with Y` records the edge on Y, pointing at X.** Per §3,
   `supersedes` is stored on the *source* (the newer node), so the edge lives on
   Y and the confirmation reads "#Y supersedes #X". Flagging the direction since
   the CLI phrasing names the *old* node first.

3. **`--yes` is an accepted affirmation, not a prompt-bypass.** The mutators are
   non-interactive by default (no destructive delete to guard — `retire`
   preserves the file), so `--yes` just satisfies the documented flag.
   `--dry-run` (writes nothing) is the substantive half of K-9. An interactive
   confirmation gate can be added later without an API change.

4. **Diagnostics routed to `err`, not via `oxur_cli` output helpers.** CLAUDE.md
   says use `oxur_cli::common::output::{success,info,warning}`, but those print
   to **stdout**, colliding with this slice's explicit "data → stdout,
   diagnostics → stderr" constraint (and would pollute `--json`). The
   more-specific slice constraint wins: query data → `out` (stdout), mutation
   confirmations/dry-run notices → `err` (stderr), tables via `tabled`. Output is
   plain (no colour), so "detect TTY before colour" is trivially satisfied and
   output is stable to assert on. Flagging the CLAUDE.md deviation with that
   reason.

5. **Tests are in-process (`odm_cli::dispatch`), not `assert_cmd` — and there is
   no separate CLI binary.** The cc-prompt/ledger named `assert_cmd`. My first
   pass added a thin `odm-cli` binary so `cargo test -p odm-cli` had something to
   spawn (the published `odm` lives in `oxur-odm`, which `-p odm-cli` doesn't
   build). On review that duplicate binary was a smell, so — **with explicit
   user approval** — I refactored to dependency-injected output: `run` wires
   stdout/stderr + the real cwd; tests construct a `Cli` and call
   `dispatch(cli, root, out, err)` against a `TempDir` with captured buffers.
   The `odm-cli` binary and the `assert_cmd`/`predicates` dev-deps are gone;
   odm-cli is library-only and the published binary is unchanged (`odm`). The
   ledger's Verify commands (`cargo test -p odm-cli <name> → ok`) all still pass;
   only the parenthetical "(assert_cmd)" hint is superseded. This is the Rust
   CLI-guidelines shape (testable library + thin bin) and removes a global-cwd
   race risk. **Flagging in case CDC wants the ledger's `assert_cmd` hint
   formally amended.**

6. **`new` idempotency key = (type, exact name); next number = max+1.** Re-running
   `new slice "X"` describes the existing node. Number allocation scans the
   corpus for the max and adds one (no index yet — Arc A4). Both are full-scan;
   fine at this stage.

## Uncertainties named

- **Reference resolution is first-match for numbers, unique-prefix for names.**
  Numbers are assumed unique (allocation guarantees it within a single writer);
  concurrent writers could in principle collide — out of scope, noted.
- **`--json` schema stability is asserted by key-set + values, not a byte
  snapshot**, because the id is a random ULID. The documented schema is the
  `NodeJson` struct; if CDC wants a byte-exact snapshot, seed a fixed id via the
  library (as the rich-fixture test does) and snapshot that.
- **Coverage measurement caveat (now bitten and resolved).** `cargo llvm-cov`
  reuses an instrumented build dir; after deleting the `odm-cli` binary, a stale
  instrumented `main.rs` plus duplicated objects produced a bogus ~50% number
  until `target/llvm-cov-target` was cleared. The 96.17% figure is from a clean
  rebuild. CDC should likewise run from a clean coverage state.
- **CI fix rides in this branch.** The gix reflog fix (`d00f436`) lives here
  because slice05 branches from slice04; slice04's standalone branch still has
  the bug. When these integrate to `main`, the fix is included. Verified locally
  under a no-gitconfig environment (`HOME` empty, `GIT_CONFIG_NOSYSTEM=1`); CDC
  should confirm on CI.
- **Sandbox has no Rust toolchain**, so all cargo evidence was produced on the
  local dev host; CDC should reproduce on CI / a local 1.85+ run.
