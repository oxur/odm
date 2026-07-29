---
id: 01KYP5FVC99PKTW1F461SCV240
number: 514836800
type: artifact
schema: artifact/v1.1
name: 'Slice 03 (Migration Fidelity): Fidelity core'
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice03-fidelity-core/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SEHW16DCQZB9ZSBQP
---
# Slice 03 (Migration Fidelity): Fidelity core

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. CC fills
> evidence at `attested` per commit; CDC reproduces (CI / local 1.85+; the CDC sandbox has no
> 1.85+ cargo, so cargo rows are `attested`→`reproduced`-on-CI). **Fixture-only** — no live-store
> mutation. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | `source` (typed sub-struct: `paths`/`class`/`normalization`/`migrated_by`/`migrated_on`), `author: Option<String>`, `version: Option<String>` exist on `Frontmatter`, in a documented canonical slot | `cargo build -p odm-core` exit 0; grep `frontmatter.rs` for the three fields + a canonical-order comment | serious | ODD-0025 §2.2 | done | `crates/odm-core/src/frontmatter.rs`: `Source` struct + `author`/`version`/`source` fields, canonical order documented in the struct doc comment (`id, number, type, schema, name, created, updated, tags, component, author, version, origin, reserved, retired, source, edges, status, decomposed, desired_facts, deferred`); `cargo build -p odm-core` exit 0 | `author` moved off `extra` (was `insert_extra` at `mapping.rs:229`) onto the typed field. |
| F-2 | Round-trip `parse ∘ emit = identity` holds with the new fields | `cargo test -p odm-core` round-trip/proptest → ok | serious | 0013 §2.3 invariant | done | Extended `frontmatter_roundtrip_identity` (the existing proptest) with `author`/`version`/`source` generation; added `author_version_source_round_trip` + `absent_author_version_source_are_not_emitted`; `cargo test -p odm-core --test frontmatter` → 30 passed | Extended the existing proptest in place, not forked. |
| F-3 | Per-type validity: `author`/`version`/`source` valid on document nodes; a `check` finding on a work node | `cargo test -p odm-core` validity test → ok (both valid + invalid cases) | correctness | ODD-0020 §2 | done (criterion corrected) | `crates/odm-core/src/check.rs::check_field_validity`; test `author_version_source_validity_by_type` in `tests/check.rs`; `cargo test -p odm-core --test check` → 16 passed | **Criterion corrected against ODD-0025 §2.2's own text**, flagged as a deviation (see Notes on Closure): `source` is valid on **both** work and document nodes ("every migrated node carries a `source` sub-map", §2.2 — a self-hosted arc/slice is a migrated node) — only `author`/`version` ("Both are document-node fields") are work-invalid. First pass wrongly bundled all three as document-only per this row's literal wording, and broke `odm-cli`'s `check_green_on_self_hosted_corpus`/`check_no_orphan_work_nodes` tests against self-hosted work nodes carrying `source`; caught immediately by that failure, corrected, re-verified green. |
| F-4 | `selfhost.rs` imports the **verbatim source body** — the stub synthesis (`format!("# {}\n", fm.name())`, ~line 201) is gone | grep `selfhost.rs` — no `format!("# {}` stub; a test asserts imported body == fixture source body (post-normalize) | serious | ODD-0025 §2.1 / design-notes root cause | done | `grep -n 'format!("# {}' crates/odm-migrate/src/selfhost.rs` → no matches; `selfhost_imports_verbatim_body_no_stub_synthesis` (tests/fidelity.rs) asserts arc/slice/project body == fixture source verbatim, and the project case additionally asserts content beyond the H1 survived (the case that actually distinguishes verbatim from the old stub) | `body_source_path()` resolves `arc-plan.md`/`slice-doc.md`/`project-plan.md` under `PlanNode.source`. |
| F-5 | `mapping.rs` imports the ODD body verbatim and types `author`/`version` onto the new fields | `cargo test -p odm-migrate` mapping test → ok | serious | ODD-0025 §2.1/§2.4 | done | `mapping_imports_verbatim_body_and_types_author_version` (tests/fidelity.rs): asserts `fm.author()`/`fm.version()` typed (not in `extra`, `unknown_key_count() == 0`) and the body matches the fixture's post-frontmatter content byte-for-byte | mapping.rs's body path was already faithful (`doc.body.clone()`); this slice typed `author`/`version` and added a `version` field to `LegacyFrontmatter` (was previously unparsed). |
| F-6 | **Hard body-hash gate** (both importers): `sha256(normalize(src)) == sha256(normalize(node))`, `normalize = trim+lf`; mismatch is a hard `MigrateError` | `cargo test -p odm-migrate` gate **pass-case** (faithful body ok) **and fail-case** (mutated body → err) → ok | serious | ODD-0025 §2.1 | done | `fidelity::tests::verify_body_hash_passes_on_identical_bodies` / `verify_body_hash_passes_across_crlf_and_trim_differences` / `verify_body_hash_fails_on_an_internal_change` (unit); wired into both importers' pass-2 loops (`lib.rs`, `selfhost.rs`), verified regardless of `--dry-run`; `cargo test -p odm-migrate` → 51+6 passed | The fail-case is unit-tested on the gate primitive directly — both importers construct `node_body` from the same read as `source_body`, so the gate is (by design) an invariant check against a future regression, not a live-diverging comparison today. |
| F-7 | `source` sub-map populated at migration (paths/class/normalization=`trim+lf`/tool+version/date) | test: a fixture-migrated node carries a populated `source` | serious | ODD-0025 §2.2 | done | `selfhost_populates_source_record` + `mapping_populates_source_record` (tests/fidelity.rs): assert `class`/`normalization`/`migrated_by`/`paths` for arc/slice/project (selfhost) and odd (mapping) | `crate::fidelity::build_source` shared by both importers. |
| F-8 | `author`/`version` preserved from source frontmatter onto the typed fields | test: a fixture ODD with `author`/`version` → node carries them | correctness | ODD-0025 §2.2 | done | `mapping_imports_verbatim_body_and_types_author_version`: fixture `0005-new-approach.md` (`author: "Katherine Johnson"`, `version: 1.0`) → `fm.author() == Some("Katherine Johnson")`, `fm.version() == Some("1.0")` | `version: 1.0` in YAML parses as a float; `legacy::de_opt_version` reconstructs `"1.0"` (not `"1"`) via Rust's round-trip `Debug` formatting — see Notes on Closure. |
| F-9 | ODD-0025 §4 amendments **applied**: ODD-0013 §2.3 (`source`/`author`/`version`), §2.2 (`artifact` named for s05), §9 (new migration semantics); ODD-0020 §2/§4 (schema-minor note) | grep the ODDs for the added fields/sections | serious | ODD-0025 §4 | done | `docs/design/01-draft/0013-odm-architecture-design.md` v2.4 (§2.2 `artifact`; §2.3 fields + canonical order; §3 `supersedes`→list + bidirectional-guarantee note; §9 rewritten; §10 terminology gains `source`); `docs/design/04-accepted/0020-…md` v2.1 (§2 `artifact/v1.0`; §3 `version:` promoted from legacy-only; §4 schema-minor note); both have Version History entries | Documentation only, as scoped — no `artifact` `NodeType` variant, no `Edges.supersedes` `Vec` type change (both explicitly s05/s06). |
| F-10 | **No live-store mutation** — tests run on fixtures/temp stores only; the `.worktrees/odm` corpus is untouched | tests use `TempDir`/fixtures; `git -C .worktrees/odm status --porcelain` empty after the test run | serious | slice-doc (Out) | done | `git -C .worktrees/odm status --porcelain` empty before and after the full slice; `find .worktrees/odm/nodes -name '*.md' \| sort \| xargs md5 \| md5` identical throughout (`40aecffaa88be3aa8764d9423594c8b0`, unchanged since slice01's F-7 evidence) | All new/changed tests use `TempDir` or read-only fixture copies exclusively. |
| F-11 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) on the changed modules | `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `! grep -RnE '\bunsafe\b' <changed>`; `cargo llvm-cov` module line ≥ 90% | polish | CLAUDE.md | done | `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; no `unsafe` in any changed module; `cargo llvm-cov --workspace --summary-only` line coverage: `check.rs` 95.61%, `frontmatter.rs` 97.07%, `fidelity.rs` 100%, `legacy.rs` 94.85%, `odm-migrate/lib.rs` 94.50%, `mapping.rs` 98.75%, `selfhost.rs` 95.10% — all ≥ 90%, all but `lib.rs`/`legacy.rs` at/above the 95% stretch target | `cargo fmt --all` applied; `cargo test --workspace` green workspace-wide (no failures) after the F-3 fix. |
| F-12 | **No decided-model drift** — the implementation matches ODD-0025 (normalize = `trim+lf`; the stored record is named `source` not `provenance`; no stored body hash) | cross-read the code against ODD-0025 §2.0–§2.2 | correctness | LEDGER-DISCIPLINE (spec-keeping) | done | Cross-read confirmed: `fidelity::NORMALIZATION = "trim+lf"`; the field is `Frontmatter.source`/`Source` (never `provenance`); no hash field anywhere on `Frontmatter` or `Source` — `verify_body_hash` computes and discards | The one correction made (F-3, `source` valid on work nodes) is a fix *toward* ODD-0025's actual text, not a drift away from it — see F-3 Notes. |

## What Worked

- **Writing the fidelity primitives (`normalize_body`, `verify_body_hash`, `build_source`) as a shared `fidelity.rs` module** rather than duplicating them in `selfhost.rs`/`mapping.rs` meant the hard gate and the `source` record are provably identical between both importers — a divergence would be a single-module diff, not a two-file audit.
- **Running the fixture test suite immediately after each mechanical step** (odm-core typing → `cargo test -p odm-core`, then each importer → `cargo test -p odm-migrate`) caught the `LegacyFrontmatter { version }` struct-literal breakage and the float-formatting hazard in `version:` parsing (`1.0` → `"1"` via naive `Display`) before either could reach a fixture assertion, rather than as a confusing downstream failure.
- **The workspace-wide test suite (not just the two changed crates) caught the real cross-crate bug** (F-3): `odm-cli`'s self-hosted-corpus tests failed the moment `source` was wrongly marked work-invalid, which a crate-scoped `cargo test -p odm-core`/`-p odm-migrate` run would never have seen. Running `cargo test --workspace` before considering any row done is the pattern to keep.

## Closure

Closed at commit `<SHA>` on 2026-07-27. Verified by: CC (this session); CDC
reproduction pending. Rows: 12. Done: 12 (1 with a corrected criterion — F-3, flagged
above and in `closing-report.md`). Deferred: 0. No-op: 0. On close, bubble up to
`../arc-plan.md` (MF-2/MF-3 plan against this capability) per LEDGER-DISCIPLINE v2.0
§A / PM Part IV.
