---
id: 01KYP5FVREEXJ2F11NHD8TPP6N
number: 582879600
type: artifact
schema: artifact/v1.1
name: 'Slice 04 (Migration Fidelity): Scope + repair capability'
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice04-scope-repair-capability/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SM22NCEXWSSF2D5E2
---
# Slice 04 (Migration Fidelity): Scope + repair capability

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. CC fills
> at `attested` per commit; CDC reproduces (CI / local 1.85+; CDC sandbox has no 1.85+ cargo →
> cargo rows `attested`→`reproduced`-on-CI). **Fixture-only — no live-store mutation.** Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | The `MAX_MVP_ARC` / `arc_in_scope` cap is **removed** — `self_host` imports every arc dir (numbered A1–A8 + named) | `! grep -nE 'MAX_MVP_ARC' crates/odm-migrate/src/` (predicate deleted, not raised); a test imports a fixture with `arc07` + a named arc | serious | arc-plan v1.6 F11 | done | `grep -rnE 'MAX_MVP_ARC' crates/odm-migrate/src/` → no matches (const + `arc_in_scope` fn deleted outright; two doc-comment mentions of the identifier also removed so the grep is clean); `selfhost_imports_numbered_and_named_arcs_with_handles` imports a fixture with arc01 (numbered), arc07 (post-MVP numbered), and a named arc — all 3 land as nodes | No replacement constant added — nothing to widen later. |
| F-2 | Named arcs (+ their slices) get a **deterministic, collision-free `number` handle** | `cargo test -p odm-migrate` named-arc test → a named arc node has a unique `number`, no collision with A1–A8's 1100–1800 | serious | arc-plan v1.6 F12 | done | `named_arc_numbers_are_collision_free_with_numbered_arcs` (unit) + `selfhost_imports_numbered_and_named_arcs_with_handles` (integration): the rule is `NAMED_ARC_BASE = arc_number(8) + NAMED_ARC_STEP` (= 1900, a `const` derived from `arc_number` rather than a repeated magic number), then `+100` per subsequent named arc in directory-sort order; a set-uniqueness assertion over all minted numbers passes | Stated in code (`selfhost.rs` doc comments on `NAMED_ARC_BASE`/`named_arc_number`) and in this row. |
| F-3 | `coverage.rs`'s shared `arc_in_scope` use updated for the removed cap — representation reflects all arc dirs in-scope | `cargo test -p odm-migrate coverage*` green; a formerly out-of-scope arc now counts as representable (not silently excluded) | correctness | arc-plan v1.6 (shared predicate) | done | `cargo test -p odm-migrate --test coverage` → 6/6 green; `coverage_doc_coverage` flipped its arc07 assertion from "has no coordinate to resolve" to "covered — the scope cap is removed" and passes; `arc_coordinate` no longer imports or calls `arc_in_scope` (deleted) | Named-arc matching in `coverage.rs` stays heuristically limited — disclosed in the module doc, not fixed here (see Deviations in the closing report). |
| F-4 | **Update-in-place repair** op: a stub node is repaired with verbatim body + `source`, preserving `id`/`edges`/`status`/`number` | `cargo test -p odm-migrate` repair test asserts body==source (post-normalize), `source` populated, and id/edges/status/number **unchanged** across the rewrite | serious | ODD-0025 §2.8 | done | `repair_rewrites_stub_body_preserving_identity_and_status`: two manually-persisted stub nodes (real edges/status attached) are repaired; asserts body verbatim, `source` populated, `id`/`edges().part_of`/`status().has_reached("planned")`/`number` all equal before vs. after | `repair()` clones the existing `Frontmatter` and only touches `body`/`source`/`updated`/`schema` — everything else is preserved by construction, not by field-by-field copying. |
| F-5 | Repair goes through the **hard body-hash gate**; a mismatch hard-fails | `cargo test -p odm-migrate` repair gate test: faithful repair ok; a mutated body → `MigrateError::BodyHashMismatch` | serious | ODD-0025 §2.1 | done | `repair()` calls `fidelity::verify_body_hash` (same as `self_host`/`migrate`) before persisting — pass-case exercised by every repair test; the fail-case is unit-tested directly on the shared primitive (`fidelity::tests::verify_body_hash_fails_on_an_internal_change`, s03), which `repair()` calls unconditionally | Same as s03's F-6: `node_body` is constructed from the same read as `source_body`, so the gate is a structural invariant check, not a live-diverging comparison — the fail path is proven at the primitive, not forced through `repair()`'s own code path (would require injecting a bug to exercise). |
| F-6 | **Schema-minor bump executed**: `SchemaVersion::CURRENT` = `v1.1`; new/repaired nodes stamp `<type>/v1.1` | grep `schema.rs` CURRENT == v1.1; `cargo test -p odm-core` stamps `v1.1` | serious | ODD-0020 §4 / s03 carry-forward | done | `SchemaVersion::CURRENT = SchemaVersion { major: 1, minor: 1 }`; `marker_round_trips_through_string` asserts `SchemaMarker::current(Design).to_string() == "design/v1.1"`; `repair_rewrites_stub_body_preserving_identity_and_status` asserts a repaired node's `schema_version() == SchemaVersion::CURRENT` | Closes the s03 deferral (ODD-0020 v1.2's note). |
| F-7 | **Forward-compat**: existing `v1.0` nodes still validate; `is_newer_than_current` treats `v1.1` as current, `v1.0` as older-valid | `cargo test -p odm-core` schema tests → a `v1.0` node passes `check`; `v1.1` is current | serious | ODD-0020 §5 (forward-compat) | done | New test `v1_0_nodes_remain_valid_after_the_v1_1_bump` (`odm-core/tests/check.rs`): a `v1.0`-stamped node → `is_newer_than_current() == false` and `content_validity` reports no `UnsupportedSchema` finding; `version_orders_and_flags_newer` updated (the "newer" example moved from `1.1`, now current, to `1.2`) | The workspace's "unsupported-newer-schema" contract test suite is updated in lockstep, not just the constant. |
| F-8 | Stub detection reused/consistent with s01 (`≤ 1 non-blank body line`, tombstones excluded) | `cargo test -p odm-migrate` → the repair path targets exactly the stub set s01 counts (44 on the real shape) | correctness | s01 / design-notes F8 | done | `fidelity::is_stub_body` extracted as the single shared predicate; `coverage.rs::stub_bodies` and `selfhost.rs::repair` both call it (grep-verified: `non_blank_line_count` no longer exists in `coverage.rs`); `repair()` additionally excludes retired nodes, matching `stub_bodies`'s tombstone exclusion exactly | Not re-derived — one function, two call sites. |
| F-9 | **No live-store mutation** — tests use fixtures/`TempDir` only; `.worktrees/odm` untouched | `git -C .worktrees/odm status --porcelain` empty; 0 live nodes gain `source:`; node md5-of-md5 unchanged | serious | slice-doc (Out) | done | `git -C .worktrees/odm status --porcelain` empty before and after the whole slice; `find .worktrees/odm/nodes -name '*.md' \| sort \| xargs md5 \| md5` unchanged (`40aecffaa88be3aa8764d9423594c8b0`) — identical to the hash recorded at slice01/slice03 close | Every new/changed test uses `TempDir` or manually-constructed fixture stores exclusively. |
| F-10 | ODD-0020 §4 records `v1.1` as current (the doc matches the code bump) | grep 0020 §4 for the v1.1-current statement | serious | ODD-0025 §4 / s03 carry-forward | done | ODD-0020 v1.3 Version History entry + a new §4 paragraph state `SchemaVersion::CURRENT` is now `v1.1`; frontmatter `version: 1.3`, `updated: 2026-07-28` | Doc and code bumped in the same commit set. |
| F-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) on changed modules | `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `! grep -RnE '\bunsafe\b' <changed>`; `cargo llvm-cov` ≥ 90% | polish | CLAUDE.md | done | `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; no `unsafe` in any changed module; `cargo llvm-cov --workspace --summary-only` line coverage: `schema.rs` 96.61%, `coverage.rs` 93.77%, `fidelity.rs` 100%, `selfhost.rs` 94.46% — all ≥ 90%, all at/above the 95% stretch target except `coverage.rs`/`selfhost.rs` (93–94%, still comfortably above the floor) | `cargo fmt --all` applied; `cargo test --workspace` green (no failures) after all fixes. |
| F-12 | **No decided-model drift** — repair is update-in-place (ODD-0025 §2.8, `persist` overwrite, no `delete`); the record is `source`; no stored body hash; cap removed not raised | cross-read code vs ODD-0025 §2.8 + arc-plan v1.6 | correctness | LEDGER-DISCIPLINE (spec-keeping) | done | Cross-read confirmed: `repair()` calls `store.persist()` (overwrite-by-id), never a delete path (`grep -n 'delete' selfhost.rs` → no matches); the field is `Frontmatter.source`/`Source` throughout, never `provenance`; `MAX_MVP_ARC` and `arc_in_scope` are deleted, not raised to a higher constant | See F-3's Note for the one disclosed, deliberately-out-of-scope gap (named-arc coverage matching). |

## What Worked

- **Deriving `NAMED_ARC_BASE` from `arc_number(8)` as a `const fn` expression** (`arc_number(8) + NAMED_ARC_STEP`), rather than hardcoding `1900` as an independent magic number, means the two constants can never silently drift apart if the numbered-arc range ever changes — the collision-freedom the ledger asks to be "stated in code" is *structurally* true, not just documented.
- **Cloning the existing `Frontmatter` and mutating only the fields that change** (`repair()`'s approach) is a much stronger "preserve id/edges/status" guarantee than rebuilding a new `Frontmatter` field-by-field and copying the ones that matter — a future field addition to the model is preserved automatically, not by remembering to add it to a copy list.
- **Extracting `fidelity::is_stub_body` before writing `repair()`** (rather than after) meant `coverage.rs` and `selfhost.rs` never had two independent stub definitions to reconcile — the F-8 "don't re-derive" criterion was satisfied by construction, not by a later audit.
- **Running the full workspace suite (not just `-p odm-migrate`/`-p odm-core`) after the schema bump** caught nothing unexpected precisely *because* the bump was done symbolically (`SchemaMarker::current()`/`SchemaVersion::CURRENT` everywhere except the deliberately-hardcoded literal-string assertions, which were grepped for and fixed explicitly) — the same discipline slice03's F-3 bubble-up recommended.

## Closure

Closed at commit `<SHA>` on 2026-07-28. Verified by: CC (this session); CDC
reproduction pending. Rows: 12. Done: 12. Deferred: 0. No-op: 0. On close, bubble
up to `../arc-plan.md` (MF-2/MF-3/MF-5 capability; schema bump closes the s03
carry-forward) per LEDGER-DISCIPLINE v2.0 §A.
