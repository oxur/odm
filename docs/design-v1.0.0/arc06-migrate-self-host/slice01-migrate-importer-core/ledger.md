# Slice 01 (Arc 06): `migrate` importer core

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). Five-iteration cap. **Arc opener** — the importer core, tested on fixtures; slice02
> runs it on odm's real docs.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M-1 | `odm-migrate` crate + `odm migrate <legacy-path> [--dry-run]` command exist; workspace member (`[workspace.dependencies]` + `[workspace.lints]`, no version literals); builds; depends on `odm-core` + `odm-store` | `cargo build -p odm-migrate` → ok AND `grep -nE "\.workspace = true" crates/odm-migrate/Cargo.toml` AND `! grep -nE '= "[0-9]' crates/odm-migrate/Cargo.toml` AND `cargo test -p odm-cli migrate_command_exists` → ok | serious | arc-plan / CLAUDE.md | done | attested — crate builds; manifest has 11 `.workspace = true` refs, zero version literals; `Migrate { legacy_path, dry_run }` wired in `odm-cli` dispatch; `migrate_command_exists` → ok (dry-run plans, commit persists 6) | Mirrors A4 slice01 creating `odm-index`. Added to `[workspace] members` + publish order. |
| M-2 | The legacy → new **mapping** is faithful: legacy `number` → preserved as the node's `number` + a fresh **ULID** identity; `state` scalar → the **`odd` gate-set position**; `supersedes`/`superseded-by` pair → a `supersedes` **edge** (with kind); `title`/`author`/`created`/`updated`/`tags`/`component` carried; `NodeType::Odd`; state-directory dropped | `cargo test -p odm-migrate maps_legacy_fields_to_node` → ok (a fixture ODD → a node with number preserved, gate at the mapped position, supersedes edge, type odd) | serious | arc-plan / 0013 §9 | done | attested — `maps_legacy_fields_to_node` → ok: #5 preserves number, type `odd`, cumulative `odd` gates draft→final, `supersedes`→#4's fresh ULID (kind obsoletes), tags/component carried, author in `extra`; state-dir dropped. Progression = cumulative reach at `Evidence::Asserted` (see closing-report deviation) | Pin the exact per-`state` → `odd`-gate mapping against the `odd` gate-set. Legacy number preserved is also the idempotence key (M-3). |
| M-3 | **Idempotent** describe-or-create: re-running `migrate` creates **nothing** new — a legacy doc whose `number` already exists as an `odd` node is **skipped**, not duplicated | `cargo test -p odm-migrate migrate_is_idempotent` → ok (migrate a fixture corpus, then migrate again → second run creates 0, reports skipped) | serious | arc-plan (idempotence key, v1.2) | done | attested — `migrate_is_idempotent` → ok: 1st run creates 6, 2nd creates 0 / skips 6 (all `AlreadyExists`), no duplicates on disk. Keyed on the preserved legacy `number` (existing ∪ minted-this-run) | ULID identity is fresh (can't be re-minted) → idempotence keys on the preserved legacy `number`, not the id. |
| M-4 | **`--dry-run`** reports the plan (per-doc create/skip + counts) and writes **nothing** (no `nodes/` created, no git change) | `cargo test -p odm-migrate migrate_dry_run_writes_nothing` → ok (dry-run over a fixture → 0 nodes on disk; plan printed) | serious | arc-plan / 0013 §9 | done | attested — `migrate_dry_run_writes_nothing` → ok: plan lists all 6, `load_all` empty, no `nodes/` directory created | Also the reflexive-safety primitive slice03 leans on (preview before the self-host cutover). |
| M-5 | **Never-delete / supersede-not-delete**: no legacy file is removed or mutated; dustbin docs (`rejected`/`withdrawn`/`superseded`) import as superseded/retired nodes (git preserves history), legacy files intact | `cargo test -p odm-migrate migrate_never_deletes_legacy` + `dustbin_imports_as_superseded` → ok (after migrate, every legacy fixture file still exists; a dustbin doc → a superseded/retired node) | serious | arc-plan / 0013 §9 / supersede-not-delete | done | attested — `migrate_never_deletes_legacy` → ok: byte-snapshot of a copied corpus is identical after a commit run. `dustbin_imports_as_superseded` → ok: #4 (Superseded) + #6 (Rejected) import as retired nodes (reason preserved); the `supersedes` edge lives on #5, not #4 | The importer reads legacy + writes new only. Git is the history; dustbin is a node state, not a file deletion. |
| M-6 | Malformed/edge legacy input (missing `number`, unknown `state`, dangling `supersedes` target) → a **clear reported error / skip, never a panic or silent drop** | `cargo test -p odm-migrate migrate_malformed_reports_not_panics` → ok (an edge-case fixture → a positioned error / a reported skip, migrate continues or fails cleanly) | serious | project error convention (CLAUDE.md) | done | attested — `migrate_malformed_reports_not_panics` → ok: missing `number` + unknown `state` → reported `Unmappable` skips; dangling `supersedes` → warning + node still imported (no dangling edge); run completes. `migrate_malformed_frontmatter_is_a_reported_skip` → ok: unterminated frontmatter → `Malformed` skip, not a panic | A silent drop here would corrupt the self-host corpus later — the importer must be loud. |
| M-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for `odm-migrate` (+ any `odm-cli` migrate-command path) | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-migrate/src` AND `cargo llvm-cov --summary-only -p odm-migrate` → **line** ≥ 90% | serious | CLAUDE.md | done | attested — clippy `-D warnings` exit 0; no `unsafe` in `crates/odm-migrate/src`; line cov: lib.rs 99.56%, mapping.rs 99.05%, legacy.rs 91.03% (all ≥ 90); `odm-cli` migrate.rs 96.61% | |

## What Worked

- **A pure library + a thin CLI seam.** `odm-migrate` returns a structured
  `MigrationReport` and never renders; `odm-cli`'s `migrate.rs` owns the
  `writeln!`+`tabled` presentation. The importer is testable end-to-end without a
  spawned binary, and the render is swappable — the same split A5 relied on.
- **Two-pass id resolution made supersession clean.** Pass 1 reserves a fresh
  ULID per creation (into a `number → id` map seeded with the store's existing
  `odd` nodes); pass 2 resolves `supersedes`/`superseded-by` against that full map
  and attaches the edge. Idempotence and edge-resolution share the *same* map, so
  a re-run skips (number already mapped) and edges always point at real ids.
- **The counting-fixture discipline, lifted to bytes.** `migrate_never_deletes_legacy`
  copies the corpus, snapshots every file's bytes, migrates, and asserts the
  snapshot is unchanged — *proving* never-delete/never-mutate, not asserting it.
- **`Unmappable` vs `Malformed` vs `AlreadyExists` are distinct skip reasons.**
  Every legacy doc lands in exactly one of created/skipped, each skip carries a
  typed reason, and a dangling supersession is a *warning* (the node still
  imports) — loud, never a silent drop (M-6).
- **The model's forward-compat catch-all carried `author`.** Rather than an
  ad-hoc schema change, the legacy `author` (no typed field) rides in
  `Frontmatter`'s `extra` map via a small `insert_extra` — the mechanism the
  schema already had for unmodeled keys.

## Deviations / decisions flagged

1. **`state` → cumulative gate reach (not a single gate).** A progression state
   reaches every `odd` gate from `draft` up to and including its mapped gate, at
   `Evidence::Asserted` on the legacy `updated` date. Rationale: a `Final` doc has
   *passed through* the earlier gates; a lone `final=…` with the rest `–` would
   misrepresent it, and `Asserted` is the honest level for a historical migration
   (claimed from the record, not independently reproduced). Flagged because "state
   → gate **position**" could also read as a single gate.
2. **`deferred` grouped with the dustbin (retire).** The `odd` gate-set (ODD-0013
   §5.1) has no `deferred`/post-`final` gate, and the node `deferred` marker needs
   a `reenter_when` fact a migrated ODD lacks. So `deferred`/`rejected`/`withdrawn`/
   `superseded` all → a **retirement** whose `reason` preserves *which* it was.
   `deferred ≠ withdrawn` semantically — a **candidate amendment** if the operator
   wants a first-class `deferred` disposition (e.g. a `deferred` gate). Flagged, not
   silently decided.
3. **`author` carried into `extra`, not a typed field.** ODD-0013 §2.3 dropped a
   typed `author`; M-2 requires carrying it. Chose the forward-compat `extra` map
   (lossless, round-trips) over a schema change. Promoting `author` to a typed
   field for `odd`/`adr` nodes is a **candidate amendment** for a later slice.
4. **Legacy `version` not carried onto the node.** It is not in M-2's carried-field
   list and the untouched legacy file (never deleted — M-5) + git preserve it.
   Noted so the omission is a decision, not a drop.
5. **Single `supersedes` edge per node.** The model's `Edges.supersedes` is
   `Option` (one target). A node that would supersede >1 target keeps the
   lowest-numbered and **warns** on the rest. Not hit by the fixtures; flagged for
   the real corpus (slice02).
6. **Fixtures only; canonical `odd` gate-set.** Per scope, tests run on synthetic
   fixtures under `test-data/`, not `docs/design`. The mapping uses the canonical
   `odd` sequence (ODD-0013 §5.1); `odm-cli` prefers a repo-configured
   `[gates.odd]` if present (the root `odm.toml` defines none today). The
   odd-vs-work numbering-space question stays deferred to slice02 (arc-plan v1.2).

## Closure

Closed at commit `<SHA>` on 2026-07-06. Verified by: CC (proposed-done, attested);
CDC to reproduce. Rows: 7. Done: 7. Deferred: 0. No-op: 0.
On close → bubble up to `arc-plan.md` (A-1) per LEDGER-DISCIPLINE v2.0 §A.
