# Slice 03 ledger -- native authoring commands (programmatic surface)

Per `LEDGER-DISCIPLINE.md` sec. A. Code slice: rows reach `reproduced` where a test/round-trip
demonstrates them; cargo/exec + `make check` are `attested -> CI` (no macOS toolchain in the CDC
sandbox).

| ID | Criterion | Verify | Significance | Origin | Status |
|----|-----------|--------|--------------|--------|--------|
| F-1 | Canonical metadata partial loads from **both** JSON and TOML (dispatch on extension) into one form; an identical partial in either format yields an identical node | fixture: same partial as `.json` and `.toml` -> byte-identical node | serious | ODD-0026 sec. 2.5 | open |
| F-2 | **Validate-before-write**: a partial setting an odm-owned field (`id`/`number`/`path`/`source`) is **rejected before any write**; a schema-invalid partial (bad `type`/`part_of`) is rejected | fixture: each rejection case errors, store unchanged | serious | ODD-0013 v2.6 boundary | open |
| F-3 | `node new <type> <name> --metadata=<file> --content=<file.md>` mints an `origin: authored` node (odm-owned `id`/`number`/placement/`source`), body = content verbatim; `check` green; reads back via `node show` | fixture + e2e: create -> show -> check exit 0 | serious | arc SS-4 | open |
| F-4 | Bare `node new <type> <name>` mints an **authored stub**, returns the ref; body fillable afterward | fixture: bare new -> authored stub node, ref returned | serious | arc SS-4 | open |
| F-5 | `node set <ref> <field> <value>` updates an author-owned field (`status`/`tags`/`part_of`/`name`), re-validates, `check` stays green; setting an odm-owned field is rejected | fixture: set status/tags OK; set id/source rejected | serious | arc SS-4 | open |
| F-6 | `node set-body <ref> --from-file=<md>`/`--body` replaces the whole body as pure markdown; frontmatter untouched (the seam preserved); reads back via `node show` | fixture: set-body -> body changed, frontmatter byte-identical | serious | ODD-0026 sec. 2.5 | open |
| F-7 | **SS-4 demonstrated**: a node body is **created AND updated** entirely via `./bin/odm`, no hand-edit of store files; round-trips through `node show` | e2e: new (metadata+content) -> set -> set-body -> show, no hand-edit | serious | arc SS-4 | open |
| F-8 | `--dry-run` (writes nothing) and `--json` (machine-readable) on `new`/`set`/`set-body` | fixture: dry-run leaves store unchanged; `--json` parses | correctness | house LLM-ergonomics | open |
| F-9 | Created/edited authored nodes pass `check` and honor the author-vs-odm boundary end-to-end | e2e: check exit 0 after each op; boundary enforced | serious | slice 02 + ODD-0013 v2.6 | open |
| F-10 | `make format`/`lint`/`test` green; no new clippy warnings | CC attests; CI reproduces | correctness | house | open |

## What Worked

_(At slice close.)_

## Closure

_(At slice close: per-row walk + bubble-up to the arc in `closing-report.md`; CDC re-run in
`cdc-verification.md`. F-3/F-7's live leg -- the same round-trip on the real store -- rides the
operator's rebuilt binary if the CDC sandbox cannot run it, same pattern as prior slices.)_

Closed at commit <SHA> on <date>. Verified by: <name/session>.
Rows: 10. Done: _. Deferred: _. No-op: _.
