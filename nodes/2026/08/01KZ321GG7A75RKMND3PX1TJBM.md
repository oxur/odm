---
id: 01KZ321GG7A75RKMND3PX1TJBM
number: 551639500
type: artifact
schema: artifact/v1.1
name: 'Closing Report — Slice 03 (Store-as-Source): native authoring (programmatic surface)'
created: 2026-08-03
updated: 2026-08-03
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice03-native-authoring/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-03
edges:
  part_of: 01KZ321DTA40A5S3FQ6VZQB9XC
---
# Closing Report — Slice 03 (Store-as-Source): native authoring (programmatic surface)

> Verified by: CC (this session). All 10 ledger rows attested via real end-to-end `odm-cli`
> integration tests (driving `dispatch` in-process). F-3/F-7's real-corpus leg not run —
> `.worktrees/odm` is not checked out in this implementation worktree, the same blocker every
> slice this session has hit. Closed 2026-08-03 on `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative.

**F-1/F-2 (the metadata partial + validate-before-write).** `odm-cli/src/metadata.rs`:
`MetadataPartial { part_of, tags, status }`, `#[serde(deny_unknown_fields)]`, loaded from `.json`
via `serde_json` or `.toml` via `toml` into the one canonical form. The author-vs-odm boundary
(ODD-0013 v2.6) is enforced **by the type's shape** — there is no field for `id`/`number`/`path`/
`source` at all, so a stray odm-owned key in a hand-written partial is a hard parse error before
the store is touched, not a value the type could silently accept and a second validation pass
would then have to catch. `name`/`type` deliberately stay `node new`'s existing positional
arguments (unchanged CLI shape) rather than partial fields — the partial covers exactly what that
shape doesn't.

**F-3/F-4 (`node new` overhaul).** Every node `node new` creates is now `origin: authored` +
`source: { class: authored }` (reusing slice 02's `Source::authored` unmodified) — not just ones
given a body or metadata. `--metadata=<file>`, `--content=<file.md>` (+ `--from-file` alias) /
`--body=<str>` (mutually exclusive, clap-enforced), `--dry-run`, `--json` all added. A bare
`node new` mints an authored stub (`# {name}`, the pre-existing stub-body convention), fillable
later via `set-body`. Parent resolution, metadata loading, and body reading all happen **before**
the idempotency check and before any write — a malformed call fails the same way whether or not
the target name already exists.

**F-5 (`node set`).** Field-addressed, `SETTABLE_FIELDS = [name, tags, part_of, status]` — any
other name (including every odm-owned one) is rejected before any write. `status` reuses
`Status::set_gate` at `Evidence::Asserted` — the same mechanism `node set-gate` already validates
through — rather than inventing a second status representation for ODD-0026 §2.5's
under-specified "status intent." `node set-gate` remains the way to record a stronger evidence
level or an actor (`--by`).

**F-6 (`node set-body`).** Replaces the whole body verbatim; the frontmatter's every other field
(`id`/`number`/`type`/`name`/`origin`/`tags`/`part_of`/`reserved`, proven by direct comparison in
the fixture) is untouched — only `updated` legitimately bumps, matching every other mutator in
this CLI (`rename`/`retire`/`tear`/`link`/`set-gate` all do the same). The cc-prompt's "byte-
identical" language is read as "every field except the edit-timestamp every other command also
bumps," not a literal freeze — flagged here rather than silently narrowed or silently over-read.

**F-7 (SS-4 end-to-end).** `ss4_round_trip_new_set_set_body_show_no_hand_edit`: `node new`
(metadata+content) → `node set` (status) → `node set-body` (replace) → `node show --json`, `check`
exit 0 after every step, no store file ever hand-edited. This is the acceptance anchor slice-doc
names explicitly, and it passes against the real CLI dispatch path, not a synthetic shortcut.

**F-8 (`--dry-run`/`--json`).** All three verbs tested for both: dry-run leaves the store file
byte-unchanged (old content present, new content absent — checked both directions, not just one),
and `--json` output parses.

**F-9 (check stays green; boundary end-to-end).** A multi-step `new` → `set` → `set-body` sequence
stays `check`-clean throughout, reusing slice 02's `content_validity`/`InconsistentAuthoredSource`
machinery completely unmodified — `origin: authored` + `Source::authored` from `node new` is
exactly the shape slice 02 already validates, so there was nothing new to wire on the check side.

**F-10.** `make format` (reflow only) + `make lint` (clippy `-D warnings`, one real fix — see
below — + rustfmt) + `make test` (full workspace, all crates + doctests) all green.

## Two regressions caught and fixed during this slice, not after

**`too_many_arguments` on `set`.** The first draft took 9 positional arguments (`store`, `root`,
`reference`, `field`, `value`, `dry_run`, `json`, `out`, `err`). Clippy's default limit is 7.
Grouped the command's own inputs into `SetArgs` (mirroring `new`'s existing `NewOptions`) — a
genuine readability improvement at the `lib.rs` call site (named fields instead of a positional
wall), not just satisfying the linter.

**`node new` minting `origin: authored` unconditionally broke two pre-existing tests that assumed
`Origin::Planned`.** Both were driving the *real* `node new` command (not a hand-built fixture) and
checking the origin it produced:

- `json_schema_crud_is_stable` (`cli.rs`) asserted `obj["origin"] == "planned"` on a node `node new`
  had just created. Updated to `"authored"`, with a comment naming the slice03 contract change.
- `rollup_origin_view_groups_by_provenance` (`rollup.rs`) built its "planned" provenance-group case
  via `run(root, &["node", "new", "slice", "Planned slice"])` — which, after this slice, no longer
  produces `Planned` at all. Rewrote the fixture: `Planned`/`Discovered`/`Amendment` are now all
  built directly via `seed()` (matching how the other two origins were already built), and the
  fourth group (`Authored`) is proven through the **real** `node new` CLI path — stronger coverage
  than before, since it now exercises the actual command's output shape rather than only hand-built
  fixtures for every group.

Both were found by running the full workspace test suite before considering the slice done, not by
CDC or the operator — the discipline that's supposed to catch exactly this class of "a model-wide
default change has a narrow blast radius that greps for the obvious pattern (`Origin::Planned`
literal construction) would miss, because the actual break is in what a *command* now produces, not
in a hand-built fixture."

## Scope discipline

Diff: `odm-cli/src/metadata.rs` (new), `odm-cli/src/commands.rs` (`new`/`set`/`set_body` +
`BodySource`/`NewOptions`/`SetArgs`/`SETTABLE_FIELDS`), `odm-cli/src/lib.rs` (the three
`NodeCommand` variants + dispatch wiring + `mod metadata`). No change to `odm-core`/`odm-migrate` —
this slice is purely the CLI authoring surface over the model slice 02 already accepted; no schema
change, no new `check` rule (slice 02's `InconsistentAuthoredSource` already covers everything
`node new` now produces). No `$EDITOR`/interactive surface (slice 07, explicitly out of scope — no
editor-spawn dependency added). No `./docs` deletion (slice 04). No read/query changes (LLM arc).

## Iterations

Two passes. First: the metadata partial, the three commands, and their fixtures, all green in
isolation (`cargo test -p odm-cli --test cli`). Second (self-caught via the full workspace suite,
not CDC): the two pre-existing tests above, fixed in one pass each. Well inside the five-iteration
cap.

## Bubble-up → `../arc-plan.md`

- **Slice 03 done** (SS-4 fixture-proven end-to-end; SS-4's real-corpus leg deferred to CDC),
  delivering the programmatic authoring surface: `node new`/`node set`/`node set-body` create and
  update an authored node entirely through `./bin/odm`, closing the `undeveloped-stub` gap (minting
  a child/slice is now one command) and unblocking slice 04's cutover dependency on a native
  authoring path.
- **What implementing it revealed that the arc-plan/ODD-0026 didn't fully specify**: (a) "status
  intent" (ODD-0026 §2.5's field list) has no concrete shape specified anywhere — resolved here as
  `Asserted`-evidence gate-reach via the existing `set-gate` mechanism, a reasonable but real
  interpretive choice, flagged in the ledger rather than silently picked; (b) `node new` minting
  `origin: authored` **unconditionally** (not conditionally on "was a body/metadata given") is a
  bigger behavior change than the slice-doc's prose emphasizes — it changes what *every* existing
  `node new` caller gets, including tests and any future tooling that assumes `Planned`. Worth a
  one-line note in ODD-0026 or the arc-plan for slice 04/07 readers: after slice 03, there is no CLI
  path left that mints `Origin::Planned` — only `migrate`'s importers and direct `Frontmatter`
  construction produce it now.
- **Silent-drop check:** all 10 ledger rows closed done (2 carrying a disclosed real-corpus-leg
  caveat: F-3, F-7). No rows dropped.
- **For slice 07 (interactive authoring ergonomics)**: `node edit --body`/`--meta`
  ($EDITOR-spawn) and `node edit --section` (markdown-section parsing) remain fully open — this
  slice added no editor dependency and no section-parsing logic, per the sizing split recorded in
  arc-plan v1.11.
- **For the operator's next real cycle**: `node new`/`node set`/`node set-body` are ready to author
  new arc/slice content on the real store directly, once convenient — the SS-4 round-trip (this
  slice's own F-7) is the exact shape to reproduce there.
