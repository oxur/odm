# Slice 03 ledger -- native authoring commands (programmatic surface)

Per `LEDGER-DISCIPLINE.md` sec. A. Code slice: rows reach `reproduced` where a test/round-trip
demonstrates them; cargo/exec + `make check` are `attested -> CI` (no macOS toolchain in the CDC
sandbox).

| ID | Criterion | Verify | Significance | Origin | Status |
|----|-----------|--------|--------------|--------|--------|
| F-1 | Canonical metadata partial loads from **both** JSON and TOML (dispatch on extension) into one form; an identical partial in either format yields an identical node | fixture: same partial as `.json` and `.toml` -> byte-identical node | serious | ODD-0026 sec. 2.5 | done | `odm-cli/src/metadata.rs::MetadataPartial` + `load()` (dispatch on `.json`/`.toml` extension, one canonical `serde`-derived type). Unit: `metadata::tests::json_and_toml_partials_deserialize_identically`. CLI-level: `cli.rs::metadata_json_and_toml_partials_produce_equivalent_nodes` — a JSON-sourced and a TOML-sourced node (separate stores, both landing at `#1`) compare equal on `tags`/`status`/`origin`. `cargo test -p odm-cli`: green. | "byte-identical node" read as "identical on every author-owned field" — `id`/`number`/`created` legitimately differ per store/run |
| F-2 | **Validate-before-write**: a partial setting an odm-owned field (`id`/`number`/`path`/`source`) is **rejected before any write**; a schema-invalid partial (bad `type`/`part_of`) is rejected | fixture: each rejection case errors, store unchanged | serious | ODD-0013 v2.6 boundary | done | The boundary is enforced by the **type's shape**, not a second pass: `MetadataPartial` has no field for any odm-owned key at all, and `#[serde(deny_unknown_fields)]` turns a stray `id`/`source`/etc. into a hard parse error inside `metadata::load` — before the store is even touched. Fixtures: `metadata::tests::an_odm_owned_field_is_a_hard_parse_error`; CLI: `new_rejects_a_metadata_partial_setting_an_odm_owned_field`, `new_rejects_a_metadata_partial_with_an_unresolvable_part_of` (bad `part_of`), `new_rejects_a_status_intent_naming_an_unknown_gate` (bad `status`), `set_rejects_an_odm_owned_field_name` (`node set 1 id ...` etc., all four odm-owned names). Every rejection asserts `md_paths(...).len() == 0` — nothing written. | |
| F-3 | `node new <type> <name> --metadata=<file> --content=<file.md>` mints an `origin: authored` node (odm-owned `id`/`number`/placement/`source`), body = content verbatim; `check` green; reads back via `node show` | fixture + e2e: create -> show -> check exit 0 | serious | arc SS-4 | done | `cli.rs::new_from_metadata_and_content_mints_an_authored_node_and_round_trips` — `--metadata`+`--content`, asserts `origin: authored`, `tags`/`part_of` from the partial, body byte-verbatim in the store file, `check` exit 0. Also `new_from_file_alias_reads_the_same_way_as_content` (`--from-file` alias) and `new_with_inline_body_writes_it_verbatim` (`--body`). | |
| F-4 | Bare `node new <type> <name>` mints an **authored stub**, returns the ref; body fillable afterward | fixture: bare new -> authored stub node, ref returned | serious | arc SS-4 | done | `cli.rs::bare_new_mints_an_authored_stub` — no metadata/content/body flags; `origin: authored`, stub body `# {name}` (the existing stub-body convention, `is_stub_body`-recognized), fillable afterward via `set-body` (proven together in `ss4_round_trip_…`). | |
| F-5 | `node set <ref> <field> <value>` updates an author-owned field (`status`/`tags`/`part_of`/`name`), re-validates, `check` stays green; setting an odm-owned field is rejected | fixture: set status/tags OK; set id/source rejected | serious | arc SS-4 | done | `commands::set` + `SETTABLE_FIELDS = [name, tags, part_of, status]` (any other field name — including every odm-owned one — rejected before any write). `status` reuses `Status::set_gate` at `Evidence::Asserted` (the same mechanism `node set-gate` validates through; `set-gate` remains how to record a stronger evidence level) rather than inventing a second status representation — flagged explicitly, see Notes. Fixtures: `set_updates_author_owned_fields_and_leaves_the_body_untouched` (all four fields, body untouched throughout), `set_rejects_an_odm_owned_field_name`, `set_status_rejects_an_unknown_gate`. | **scoping note**: "status" is not the full evidence-tracked gate vector — it is `Asserted`-only intent, reusing existing machinery rather than inventing a parallel representation; flagged, not silently narrowed |
| F-6 | `node set-body <ref> --from-file=<md>`/`--body` replaces the whole body as pure markdown; frontmatter untouched (the seam preserved); reads back via `node show` | fixture: set-body -> body changed, frontmatter byte-identical | serious | ODD-0026 sec. 2.5 | done | `commands::set_body`. Fixtures: `set_body_replaces_the_body_and_leaves_frontmatter_fields_untouched` (asserts `id`/`number`/`type`/`name`/`origin`/`tags`/`part_of`/`reserved` all identical before/after — only `updated` legitimately bumps, see Notes), `set_body_inline_also_replaces_the_body`. | **"byte-identical" read as** "every field except `updated`" — every other mutator in this CLI (`rename`/`retire`/`tear`/`link`/`set-gate`) already bumps `updated` on edit; a literal freeze would be the one inconsistent command. Flagged, not silently narrowed. |
| F-7 | **SS-4 demonstrated**: a node body is **created AND updated** entirely via `./bin/odm`, no hand-edit of store files; round-trips through `node show` | e2e: new (metadata+content) -> set -> set-body -> show, no hand-edit | serious | arc SS-4 | done -- real leg CLOSED (2026-08-03) | `cli.rs::ss4_round_trip_new_set_set_body_show_no_hand_edit` — `node new` (metadata+content) → `node set` (status) → `node set-body` (replace) → `node show --json`, `check` exit 0 after **every** step, no file ever hand-edited. Real-corpus leg (the identical round-trip against `.worktrees/odm`) not run — see F-3's Notes / the standing blocker every slice this session has hit. **Real leg CLOSED 2026-08-03**: the first real-corpus use of the authoring command -- `node set-body` on the live bootstrap node #543468700 (`workbench/sync-bootstrap.sh`) -> `store commit` (701668b, `~1 artifact`, clean index) -> `check` 0 errors. | |
| F-8 | `--dry-run` (writes nothing) and `--json` (machine-readable) on `new`/`set`/`set-body` | fixture: dry-run leaves store unchanged; `--json` parses | correctness | house LLM-ergonomics | done | `new_dry_run_writes_nothing_and_json_parses`, `set_dry_run_writes_nothing_and_json_parses`, `set_body_dry_run_writes_nothing_and_json_parses` — each asserts the store file is untouched (old content present, new content absent) and the `--json` output parses as valid JSON. | |
| F-9 | Created/edited authored nodes pass `check` and honor the author-vs-odm boundary end-to-end | e2e: check exit 0 after each op; boundary enforced | serious | slice 02 + ODD-0013 v2.6 | done | `authored_nodes_stay_check_clean_through_a_multi_step_sequence` (new → set → set-body, `check` exit 0 throughout) plus every F-2 rejection fixture (the boundary half). Reuses slice 02's `content_validity`/`InconsistentAuthoredSource` machinery unmodified — `origin: authored` + `Source::authored` is exactly the shape slice 02 already validates. | |
| F-10 | `make format`/`lint`/`test` green; no new clippy warnings | CC attests; CI reproduces | correctness | house | done | `make format` (reflow only) + `make lint` (clippy `-D warnings` incl. the `too_many_arguments` fix — `set`'s 9 args grouped into `SetArgs`/`NewOptions` — + rustfmt, clean) + `make test` (full workspace, all crates + doctests) — see `closing-report.md` for the exact run. | |

## What Worked

- **The type's shape *is* the boundary enforcement — no second validation pass needed.**
  `MetadataPartial` has a field for every author-owned key and *no field at all* for any
  odm-owned one, so `#[serde(deny_unknown_fields)]` rejects `id`/`number`/`source`/`path`
  automatically, at parse time, before any store interaction. F-2's fixtures came out simple
  because there was nothing to separately implement — they mostly prove the type does what
  its shape already guarantees.
- **`node set`'s field-addressed surface reused `Status::set_gate` instead of inventing a
  second status representation for "status intent."** ODD-0026 §2.5 names "status intent" as
  an author-owned field but doesn't specify its exact shape; rather than build a new,
  parallel status model, `status` in both the metadata partial and `node set` is a gate name
  applied at `Evidence::Asserted` through the *same* `Status::set_gate` `node set-gate`
  already validates through. One mechanism, two entry points (a quick "asserted" intent via
  `set`, a fuller evidence-and-actor record via `set-gate`) — flagged explicitly in the
  ledger rather than silently narrowing ODD-0026's abstract field list.
- **Reusing `NodeJson`/`resolve`/`today` by keeping `new`/`set`/`set_body` inside
  `commands.rs`** (rather than a separate module, which would have needed `resolve` bumped to
  `pub(crate)`) matched the codebase's own established convention — every other node-mutation
  verb (`link`, `tear`, `set_gate`, `decomposed`, `rename`, `retire`) already lives there. Zero
  visibility friction, and `--json` on the three new commands reuses the exact `NodeJson`
  shape `node show`/`list` already emit, so there was no new JSON schema to design or
  document.
- **Clippy's `too_many_arguments` caught a real readability smell, not just a lint to
  silence.** `set`'s first draft took 9 positional arguments; grouping the command's own
  inputs into `SetArgs` (mirroring `new`'s existing `NewOptions`) fixed the lint and made the
  call site at `lib.rs` self-documenting (named fields instead of a positional wall) — the fix
  was a genuine improvement, not just satisfying the linter.

## Closure

Closed 2026-08-03. Verified by: CC (this session) — attested for all 10 rows (real end-to-end
`odm-cli` integration tests driving `dispatch` in-process, not just unit fixtures — the F-2
boundary tests, F-7's SS-4 round-trip, and F-9's multi-step sequence all go through the actual
CLI dispatch path). F-3/F-7's real-corpus leg (the identical round-trip against `.worktrees/odm`)
is **not run** — the standing blocker every slice this session has hit (the store lives on the
`odm` orphan branch, not checked out in this implementation worktree); disclosed in F-3/F-7's own
rows rather than silently assumed. One regression caught and fixed during this slice, not by CDC:
`node new` minting `origin: authored` for every node (not just body/metadata-carrying ones) broke
one pre-existing test (`json_schema_crud_is_stable`, expected `"planned"`) and one pre-existing
rollup fixture (`rollup_origin_view_groups_by_provenance`, which built its "planned" case via the
real CLI `node new` — no longer possible, since that command doesn't mint `Planned` anymore);
both updated to the new, correct contract rather than the old one preserved by coincidence. `make
format` + `make lint` (clippy `-D warnings`, incl. fixing `set`'s `too_many_arguments`) + `make
test` (full workspace, all crates + doctests) all green. CDC reproduction, plus the real-corpus
leg, are the open items. Rows: 10. Done: 10 (2 carrying a disclosed real-corpus-leg caveat: F-3,
F-7). Deferred: 0. No-op: 0.
