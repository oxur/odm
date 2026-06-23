# Closing Report — Slice 05 (Arc 01): Node CRUD commands

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 06 opens.

- **Implementation commit:** `f33c983` (CRUD); **CI fix:** `d00f436` (gix reflog).
- **Branch:** `slice05-node-crud` (based on `slice04-store-layer`; not pushed;
  not merged to `main`).
- **Scope delivered:** `new`/`list`/`show`/`rename`/`retire`/`supersede`,
  `use`/`context`, with `--dry-run`/`--yes`/`--json`.
- **Result:** 11 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test -p odm-cli` → 23 assert_cmd tests pass;
  `cargo clippy -p odm-cli --all-targets -- -D warnings` → exit 0; odm-cli line
  coverage 98.6% / region 95.7%. Umbrella `odm --version` still works; workspace
  clippy/fmt clean.

## Per-row walk

| ID | Status | Evidence (re-runnable at `f33c983`) |
|----|--------|-------------------------------------|
| K-1 | done | `cargo test -p odm-cli new_persists` → 1 passed; one `.md` persisted, next-number allocation. |
| K-2 | done | `cargo test -p odm-cli new_idempotent` → 1 passed; re-run prints `exists:`, count stays 1. |
| K-3 | done | `cargo test -p odm-cli list_filters` → 1 passed; tag/component filters also covered in `extra`. |
| K-4 | done | `cargo test -p odm-cli show_node` → 1 passed; full field/edge/children rendering in `extra`. |
| K-5 | done | `cargo test -p odm-cli rename_keeps_id_and_path` → 1 passed; same path before/after, id intact. |
| K-6 | done | `cargo test -p odm-cli retire_preserves_file` → 1 passed; file kept, `retired:` recorded. |
| K-7 | done | `cargo test -p odm-cli supersede_with_kind` → 1 passed; edge on newer node with `kind`. |
| K-8 | done | `cargo test -p odm-cli context_use_and_show` → 1 passed; `.odm/context.json`; mismatch rejected. |
| K-9 | done | `cargo test -p odm-cli dry_run_and_yes` → 1 passed; dry-run writes nothing; `--yes` runs. |
| K-10 | done | `cargo test -p odm-cli json_schema_crud` → 1 passed; exact key set + stable values. |
| K-11 | done | clippy exit 0; no `unsafe`; `cargo llvm-cov … --ignore-filename-regex '(odm-core\|odm-store\|odm-graph)/'` → line 98.60%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **New `retired` field on the §2.3 schema (odm-core).** `retire` needs a
   persisted, round-tripping marker, and the schema had none. I added a typed
   `Frontmatter.retired: Option<Retirement { reason, on }>` (skipped when
   absent, so slice03's canonical-order snapshot is unaffected) plus the
   mutating accessors `Document::frontmatter_mut`, `Frontmatter::{set_name,
   set_updated, edges_mut, retire}`. This **extends the normative §2.3 schema**
   (which doesn't list `retired`). **Recommend** §2.3 be updated to document
   `retired`; flagging rather than silently extending. (Open question for Arc 02:
   whether retirement becomes a terminal *gate* and this field is then folded in.)

2. **`supersede X --with Y` records the edge on Y, pointing at X.** Per §3,
   `supersedes` is stored on the *source* (the newer node). So `supersede X
   --with Y` = "Y supersedes X" → the edge lives on Y. The confirmation reads
   "#Y supersedes #X". Flagging the direction since the CLI phrasing
   ("supersede X") names the *old* node first.

3. **`--yes` is an accepted affirmation, not a prompt-bypass (no prompt exists).**
   The mutators are non-interactive by default (there is no destructive delete to
   guard — `retire` preserves the file), so `--yes` currently just satisfies the
   documented flag. `--dry-run` is the substantive half of K-9 (writes nothing).
   An interactive confirmation gate can be added later without an API change.

4. **Diagnostics routed to stderr by hand, not via `oxur_cli` output helpers.**
   CLAUDE.md says use `oxur_cli::common::output::{success,info,warning}`, but
   those print to **stdout**, which collides with this slice's explicit "data →
   stdout, diagnostics → stderr" constraint (and would pollute `--json`). The
   more-specific slice constraint wins: confirmations/dry-run notices go to
   stderr, query data to stdout, tables via `tabled`. Output is plain (no colour)
   so it is stable under `assert_cmd`; the "detect TTY before colour" rule is
   trivially satisfied. Flagging the CLAUDE.md deviation with that reason.

5. **A thin `odm-cli` binary was added for testing.** `cargo test -p odm-cli`
   does not build the umbrella's `odm`, so assert_cmd had nothing to spawn. The
   new `[[bin]] name = "odm-cli"` (a thin wrapper over the same `odm_cli::run`)
   makes the suite self-contained; the *published* binary remains `odm` from
   `oxur-odm`. Two binaries now share `run()` — intentional, not a duplicate
   surface.

6. **`new` idempotency key = (type, exact name); next number = max+1.** Re-running
   `new slice "X"` describes the existing node. Number allocation scans the
   corpus for the max and adds one (no index yet — Arc A4). Both are full-scan;
   fine at this stage.

## Uncertainties named

- **Reference resolution is first-match for numbers, unique-prefix for names.**
  Numbers are assumed unique (allocation guarantees it within a single writer);
  concurrent writers could in principle collide — out of scope here, noted.
- **`--json` schema stability is asserted by key-set + values, not a byte
  snapshot**, because the id is a random ULID. The documented schema is the
  `NodeJson` struct; if CDC wants a byte-exact snapshot, seed a fixed id via the
  library (as the `extra` tests do) and snapshot that.
- **CI fix rides in this branch.** The gix reflog fix (`d00f436`) lives here
  because slice05 branches from slice04; slice04's standalone branch still has
  the bug. When these integrate to `main`, the fix is included. Verified locally
  under a no-gitconfig environment; CDC should confirm on CI.
- **Sandbox has no Rust toolchain**, so all cargo evidence was produced on the
  local dev host; CDC should reproduce on CI / a local 1.85+ run.
