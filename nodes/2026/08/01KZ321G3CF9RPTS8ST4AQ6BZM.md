---
id: 01KZ321G3CF9RPTS8ST4AQ6BZM
number: 537837000
type: artifact
schema: artifact/v1.1
name: Slice 02 ledger -- self-sourced planning nodes
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice02-self-sourced-nodes/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-03
edges:
  part_of: 01KZ321DTABWYF2R92MA2Y2K8H
---
# Slice 02 ledger -- self-sourced planning nodes

Per `LEDGER-DISCIPLINE.md` sec. A. Code slice: rows reach `reproduced` where a test or the
live corpus demonstrates them; cargo/exec legs are `attested -> CI` (no macOS toolchain in the
CDC sandbox). Design-doc rows (amendments) top at `attested`/`reconciled`.

| ID | Criterion | Verify | Significance | Origin | Status |
|----|-----------|--------|--------------|--------|--------|
| F-1 | Schema adds `origin: authored` + the authored `source` shape (`class: authored`, no `paths`/`migrated_*`); it validates | fixture: an authored-shape node parses + validates; a malformed one is rejected | serious | ODD-0026 sec. 2.1 | done | `odm-core`: `Origin::Authored` (`origin.rs`); `Source::authored`/`Source::is_authored` + `paths`/`normalization`/`migrated_by`/`migrated_on`/`migrated_from` reshaped (`frontmatter.rs`); `Violation::InconsistentAuthoredSource` + `check_authored_source` (`check.rs`, in `content_validity` — `source` isn't index-backed). Fixtures: `odm-core/tests/check.rs::{a_correctly_shaped_authored_node_is_green, origin_authored_with_no_source_is_rejected, origin_authored_with_migrated_paths_is_rejected, source_class_authored_without_origin_authored_is_rejected}`. `cargo test -p odm-core`: green. | the sub-decision + F-1's "malformed rejected" half share one validation |
| F-2 | `check` passes an authored node (no `undeveloped-stub` / `missing-source` / body-hash error) | fixture: authored node, `check` exit 0 | serious | arc SS-2 | done | `odm-core/tests/check.rs::an_authored_node_with_a_valid_parent_is_green`; end-to-end `odm-cli/tests/migrate.rs::migrate_to_authored_converts_the_planning_corpus_and_check_stays_green` (real CLI `migrate --to-authored` then `odm validate` exit 0) — this is the test that caught F-1's implementation bug (see Notes). `undeveloped-stub`/`missing-source` never existed as source-gated `check` findings to begin with (confirmed by code read: `recompose.rs`'s `UndevelopedStub` is parent-child-count-only, unrelated to `source`) — no suppression needed, just the new consistency check passing cleanly. | |
| F-3 | Schema/edge/cycle/decomposition/derived-order validation **still applies** to authored nodes | fixture: authored node with a bad `part_of` / cycle / drift still fails `check` | serious | arc SS-2 | done | `odm-core/tests/check.rs::{an_authored_node_with_a_dangling_part_of_still_fails, an_authored_node_in_a_supersession_cycle_still_fails, an_authored_node_with_an_empty_name_still_fails}`. No code in `recompose.rs`/`check.rs`'s existing families was touched — only a new, additive check was added. | |
| F-4 | `migrate --all` does not require or rewrite an external source for authored nodes; their bodies are left byte-intact (not churned) | fixture + live: authored node body unchanged across `migrate --all` | serious | arc SS-3 | done (fixture); live deferred | `self_host`'s new `by_coordinate_authored` index (`selfhost.rs`) recognizes an authored node by structural coordinate and never re-mints or rewrites it, even while its `./docs` plan-tree file is still present and drifting. Fixtures: `odm-migrate/tests/source_identity.rs::selfhost_never_churns_an_authored_node_even_when_its_plan_tree_file_still_exists`; `odm-migrate/tests/convert_to_authored.rs::after_conversion_self_host_recognizes_and_never_churns_it`; real-pipeline `odm-cli/tests/migrate.rs` `--to-authored` tests. Live leg (the real 400+-node store) not run — see F-6's note, same blocker. | |
| F-5 | `reconcile` treats authored nodes as self-sourced -- no re-fidelity against a missing source, no spurious drift | fixture: authored node, `reconcile` reports 0 drift | serious | ODD-0026 sec. 2.2 | done | `selfhost::reconcile` gains an explicit `origin() == Authored` skip (after the existing sourceless skip) — `reconcile.rs`. Fixture: `odm-migrate/tests/reconcile.rs::reconcile_never_touches_an_authored_node_even_as_its_plan_tree_file_drifts` — `reconciled_count() == 0` even as the plan-tree file keeps drifting; body/source/origin all confirmed untouched. `mapping::reconcile_source` (design/research family) was already safe by construction (an authored node's empty `source.paths` makes `.first()` return `None`, the existing "nothing to resolve from" skip) — confirmed by code read, no change needed there. | |
| F-6 | Existing planning corpus converted to self-sourced per the sub-decision; `migrate --all` + `check` green with `./docs` still present | live: reset -> `migrate --all` -> `check` exit 0 on the real store | serious | arc SS-3 | deferred | **Mechanism built and fixture-proven**: `selfhost::convert_to_authored` (new, explicit, one-time — deliberately *not* folded into `--all`, see its own doc comment for why) + CLI flag `migrate --to-authored` (`--dry-run`-able, idempotent). Fixtures: `odm-migrate/tests/convert_to_authored.rs` (6 tests: flip+preserve, dry-run, idempotent, sourceless/retired excluded, post-conversion self_host safety); `odm-cli/tests/migrate.rs::migrate_to_authored_converts_the_planning_corpus_and_check_stays_green` reproduces the exact SS-3 shape (convert → `check` exit 0 with `./docs` still present) on a synthetic plan set. **Real-store leg not run**: `.worktrees/odm` (the actual corpus) is not checked out in this implementation worktree — the identical blocker every prior slice this session has hit. | **reason**: no access to `.worktrees/odm` from this worktree. **re-entry**: operator/CDC runs `odm migrate --to-authored` then `odm migrate --all` then `odm check` on the real store (post `store commit`/rebuild); confirm 0 errors with `./docs` still present |
| F-7 | **No regression:** the body-hash gate STILL fails a genuinely-migrated node (external `source.paths`) whose body diverges from its source | fixture: migrated node + corrupted body -> `check` fails | correctness | arc SS-5 | done | Pre-existing `repair_surfaces_a_drifted_non_stub_body_as_a_hash_mismatch` (slice05-era) continues to pass unmodified. New, slice02-specific: `odm-migrate/tests/source_identity.rs::a_drifted_migrated_node_still_hard_fails_alongside_an_untouched_authored_one` — a mixed corpus (one authored node, one drifted migrated node) proves the new authored-recognition code doesn't mask the gate for its sibling in the same run. `verify_body_hash`'s call sites are unchanged; only new authored-guards were added *around* them, never inside. | |
| F-8 | **ODD-0013 amendment** written: `origin: authored`; authored `source` shape; "authored is a provenance value" language; author-vs-odm field boundary | amendment doc present + accepted | serious | ODD-0026 sec. 3 | done | **FOLDED IN** to `04-accepted/0013-odm-architecture-design.md` (v2.6, 2026-08-03): origin value + authored `source` shape + field boundary + history entry | stub `F-8-amendment-ODD-0013.md` retained as the applied record |
| F-9 | **ODD-0025 amendment** written: authored nodes bypass the migration body-hash gate by construction; migrated fidelity unchanged | amendment doc present + accepted | serious | ODD-0026 sec. 3 | done | **FOLDED IN** to `04-accepted/0025-migration-fidelity-model.md` (v1.4, 2026-08-03): sec. 2.0 authored point + sec. 2.1 gate-scope-narrowed + sec. 2.2 authored source + history entry | stub `F-9-amendment-ODD-0025.md` retained as the applied record |
| F-10 | The sub-decision (fate of converted nodes' provenance) is resolved + recorded in the ODD-0013 amendment before conversion runs | amendment states (i) or (ii) with rationale | serious | slice-doc | done | Resolved **(i)** — re-classify as authored, preserve migration provenance via `source.migrated_from` — the slice-doc's own recommended lean, directly following from ODD-0026 fork A ("provenance is enduring, don't drop it"); (ii) would leave dangling `source.paths` with no marker after cutover, contradicting fork A's spirit. Recorded in `F-8-amendment-ODD-0013.md`'s schema section (the `migrated_from` field) and implemented in `Source::authored`/`convert_to_authored`. Decided by CC as the direct, low-risk application of an already-ratified decision — flagged here, not silently chosen, per the slice-doc's own instruction. | CC's call, not a fresh AskUserQuestion round-trip — directly entailed by the already-accepted ODD-0026 fork A |
| F-11 | `make format`/`lint`/`test` green; no new clippy warnings | CC attests; CI reproduces | correctness | house | done | `make format` (reflow only, no functional diff) + `make lint` (clippy `-D warnings` + rustfmt check, clean) + `make test` (full workspace, all crates + doctests) all green — see `closing-report.md`. | |

## What Worked

- **The index-backed vs store-loaded split (`check` vs `content_validity`) caught a real bug
  before it shipped.** The first version of `check_authored_source` was wired into `check()`
  itself; every unit test I wrote for it passed, because those tests call `check()`/
  `content_validity()` directly with hand-built `Frontmatter` values that always carry `source`.
  It was only the **end-to-end CLI test** (`migrate --to-authored` then `odm validate`) that
  exercises the real index-backed read path — and it failed immediately, because `source` isn't
  one of the fields the `.odm/index` reconstructs. Moving the check to `content_validity` (which
  already runs over store-loaded frontmatters for exactly this reason — `check_source_paths` has
  the identical dependency) fixed it in one line. The lesson: a source-dependent `check` finding
  must default to `content_validity`, not `check`, and the fastest way to catch a violation of that
  rule is a real CLI-level test, not a unit test against hand-built fixtures.
- **`self_host`'s existing three-tier matching (`by_source` → `by_coordinate` → create) had exactly
  the right shape to extend with a fourth tier (`by_coordinate_authored`).** Because an authored
  node's `source.paths` is always empty, it was already structurally invisible to `by_source`
  (zero iterations, harmless) and to `by_coordinate` (it has `Some(source)`, so the `None` arm never
  fires) — meaning without the fix it would have silently fallen through to `to_create` and been
  **re-minted as a duplicate** the moment its `./docs` file was still present, which is true for
  every node until slice04's cutover. This was found by reading the matching loop's control flow
  before writing any code, not by a test catching it after the fact — worth naming, since the
  fixture that proves it (`selfhost_never_churns_an_authored_node_even_when_its_plan_tree_file_still_exists`)
  reproduces exactly the shape a first migrate --all after this lands would hit on the real corpus.
- **Choosing `--to-authored` as an explicit, separate flag (not folded into `--all`) was a judgment
  call worth stating plainly, not just implementing.** Once a node converts, `reconcile` stops
  picking up further `./docs` edits to it (by design — it's self-sourced now) — a real, disclosed
  behavior change to the operator's current edit-then-`migrate --all` workflow. Wiring the
  conversion into `--all` unconditionally would have made that change happen silently on the next
  ordinary run; a separate, `--dry-run`-able flag lets the operator trigger it knowingly. Recorded
  in the flag's own doc comment and in `convert_to_authored`'s doc comment, not just this ledger.

## Closure

Closed 2026-08-03. Verified by: CC (this session) — attested for F-1/F-2/F-3/F-5/F-7/F-10/F-11
(unit + real end-to-end `odm-cli` integration tests); F-4 and F-6 done for the fixture/mechanism
half, with the real-corpus leg **deferred** (see their own rows — `.worktrees/odm` is not checked
out in this worktree); F-8/F-9 **attested** (amendment stubs written and code-complete, not yet
folded into the accepted ODD documents — a separate operator/CDC action). `make format` + `make
lint` + `make test` (full workspace, all crates + doctests) all green — see `closing-report.md`.
CDC reproduction of every `done`/`attested` row, plus F-6's deferred real-corpus leg and F-8/F-9's
ODD fold-in, are the open items. Rows: 11. Done: 9. Deferred: 0 (folded into F-4/F-6's rows as a
disclosed partial). Attested-only: 2 (F-8, F-9). No-op: 0.
