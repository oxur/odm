# Slice 03 -- CDC verification (native authoring, programmatic surface)

**Method:** LEDGER-DISCIPLINE v2.0 sec. A. CC's implementation is **uncommitted (staged) in the
working tree** at verification time (HEAD is the slice-03 *drawing* commit `3f527dc`); rows
reproduced by reading the working-tree code + reading test assertions; cargo/exec + `make check` are
`attested -> CI` (no macOS toolchain in the CDC sandbox). **Verdict: PASS.** Verified 2026-08-03.

## Verdict

**PASS.** Slice 03 delivers the programmatic authoring surface and closes **SS-4** (create AND
update a node body via `./bin/odm`, no hand-edit). 9/10 rows reproduced by code+test read; F-10
attested -> CI. One design choice is *better* than the ledger asked (below); CC's three scoping
disclosures are all reasonable and flagged, not silent.

## Reproduced by code read

- **F-1/F-2 (metadata partial + boundary).** `odm-cli/src/metadata.rs`: `MetadataPartial { part_of,
  tags, status }` with `#[serde(deny_unknown_fields)]`; `load()` dispatches on `.json`/`.toml` into
  the one canonical type. **The boundary is enforced by the type's *shape*, not a second pass** --
  there is no field for `id`/`number`/`path`/`source` to bind to, so a stray odm-owned key is a
  hard parse error before the store is touched. This is the stronger reading of ODD-0013 v2.6's
  boundary. `name`/`type` are positional `node new` args (correctly not in the partial).
- **F-3/F-4 (`node new`).** `commands::new` mints `Origin::Authored` + `Source::authored(vec![])`
  for every created node (reusing slice 02's model unmodified); the body is written **verbatim**
  from the content/body source, or the stub `# {name}` when none is given. No synthesized-H1
  transform. `--metadata`/`--content`/`--from-file`/`--body`/`--dry-run`/`--json` all wired.
- **F-5 (`node set`).** `SETTABLE_FIELDS = [name, tags, part_of, status]`; any other field name
  bails before any write (the author-vs-odm boundary on the edit path). `status` records at
  `Evidence::Asserted` via the existing `Status::set_gate`.
- **F-6 (`node set-body`).** Clones the frontmatter, bumps only `set_updated(today())`, writes the
  new body -- the seam is preserved (odm re-attaches the unchanged frontmatter; the author supplies
  pure markdown).

## Reproduced by test-assertion read

- **F-7 (SS-4 round-trip) -- read in full.** `ss4_round_trip_new_set_set_body_show_no_hand_edit`:
  `node new` (project) -> `node new arc --metadata --content` -> `check` -> `node set status` ->
  `check` -> `node set-body --from-file` -> `check` -> `node show --json`, asserting
  `origin == authored`, the set gate, and the updated body in the store file. `check` is exit-0
  after **every** step; nothing is hand-edited. This is the acceptance anchor and it holds.
- **Fixtures present (9 named + 3 dry-run):** dual-format equivalence, the odm-owned-field
  rejections (new + set), authored-mint + round-trip, bare stub, set-leaves-body-untouched,
  set-body-leaves-frontmatter-untouched, multi-step check-clean. Confirmed present; not run here ->
  `make check` green attested by CC -> CI.
- **The two self-caught regressions are correct contract updates, not weakened tests.** Verified the
  working-tree diff: `node new`'s unconditional `origin: authored` broke a rollup-provenance fixture
  and a JSON-shape test that drove the real `node new` expecting `Planned`; both were fixed by
  *seeding* the non-authored origins directly (since `node new` no longer produces them) -- net
  `+1` assertion, no assertion deleted. No masking.

## One thing better than the ledger asked

**The boundary is structural, not procedural.** The ledger framed F-2 as "reject a partial that
sets an odm-owned field." CC made it impossible to *express* one: the partial type has no field for
odm-owned keys and `deny_unknown_fields` turns a stray key into a parse error. A boundary that can't
be represented can't be bypassed -- the right shape for the "odm owns identity/placement/provenance"
invariant.

## Carried forward (not blocking)

- **"status intent" is interpreted as `Asserted`-only** (reusing `set_gate`; `set-gate` remains for
  stronger evidence). ODD-0026 sec. 2.5 under-specified this; CC's reading is reasonable and
  flagged. Worth a one-line confirm in ODD-0026 (or the slice-07 draft) so the intent tier is
  recorded in the model, not only in code.
- **F-3/F-7 real-corpus leg -- CLOSED 2026-08-03:** the same `new -> set -> set-body -> show` round-trip on
  the live store, on the operator's rebuilt binary -- same deferral pattern as every code slice.
- **Slice 07 (interactive `$EDITOR`/section surface)** remains open, plan-late, off the SS-4/SS-6
  critical path.

CC's implementation is uncommitted (staged) at close -- the operator commits it with this
verification.
